use serde::Deserialize;
use base64::{Engine as _, engine::general_purpose::STANDARD};

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::HANDLE;
#[cfg(target_os = "windows")]
use windows_sys::Win32::Graphics::Printing::{
    OpenPrinterW, ClosePrinter, StartDocPrinterW, EndDocPrinter,
    StartPagePrinter, EndPagePrinter, WritePrinter, DOC_INFO_1W,
};

#[derive(Deserialize)]
pub struct ProductoInput {
    name: String,
    qty: i32,
    price: f64,
}

#[derive(Deserialize)]
pub struct TicketInput {
    header: String,
    footer: String,
    show_date: bool,
    print_qr: bool,
    logo_image: Option<String>,
    qr_image: Option<String>,
    paper_size: String,
    target_device: String, 
    products: Vec<ProductoInput>,
    total: f64,
}

// ==========================================
// 🛠️ HELPER: CONVERTIR IMAGEN A ESC/POS
// ==========================================
fn procesar_imagen_escpos(base64_str: &str, max_width: u32) -> Vec<u8> {
    // 1. Limpiar el encabezado "data:image/png;base64,"
    let b64_data = if let Some(idx) = base64_str.find(',') {
        &base64_str[idx + 1..]
    } else {
        base64_str
    };

    // 2. Decodificar Base64
    let decoded = match STANDARD.decode(b64_data) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    // 3. Cargar imagen y convertir a Blanco y Negro
    let img = match image::load_from_memory(&decoded) {
        Ok(i) => i,
        Err(_) => return Vec::new(),
    };

    // Ajustar el ancho para que sea compatible con la térmica (múltiplo de 8)
    let width = (max_width / 8) * 8;
    let img = img.thumbnail(width, 500); // Achicar manteniendo proporción
    let img = img.into_luma8(); // Escala de grises pura
    
    let (w, h) = img.dimensions();
    let width_bytes = w / 8;

    let mut bytes = Vec::new();
    
    // 4. Comando ESC/POS de Raster Image (GS v 0)
    bytes.extend_from_slice(&[29, 118, 48, 0]); 
    bytes.push((width_bytes % 256) as u8); // xL
    bytes.push((width_bytes / 256) as u8); // xH
    bytes.push((h % 256) as u8); // yL
    bytes.push((h / 256) as u8); // yH

    // 5. Mapear píxeles a bits
    let mut current_byte: u8 = 0;
    let mut bit_count = 0;

    for pixel in img.pixels() {
        let is_black = pixel[0] < 128; // Si es más oscuro que gris medio, es negro
        current_byte <<= 1;
        if is_black {
            current_byte |= 1;
        }
        bit_count += 1;

        if bit_count == 8 {
            bytes.push(current_byte);
            current_byte = 0;
            bit_count = 0;
        }
    }
    
    // Salto de línea extra para despegar la imagen
    bytes.extend_from_slice(&[27, 74, 30]); 
    bytes
}

// ==========================================
// 🔄 COMANDO 1: ESCANEAR IMPRESORAS
// ==========================================
#[tauri::command]
fn escanear_impresoras() -> Vec<String> {
    let mut impresoras = Vec::new();
    if let Ok(output) = std::process::Command::new("powershell")
        .args(&["-Command", "(Get-Printer).Name"])
        .output()
    {
        let result = String::from_utf8_lossy(&output.stdout);
        for line in result.lines() {
            let name = line.trim();
            if !name.is_empty() {
                impresoras.push(name.to_string());
            }
        }
    }
    impresoras
}

// ==========================================
// 🖨️ COMANDO 2: IMPRESIÓN DIRECTA
// ==========================================
#[tauri::command]
fn imprimir_ticket(ticket: TicketInput, es_tarjeta_presentacion: bool) -> Result<String, String> {
    if ticket.target_device.is_empty() {
        return Err("No seleccionaste ninguna impresora.".to_string());
    }

    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[27, 64]); // Reset
    bytes.extend_from_slice(&[27, 97, 1]); // Centrado

    // --- NUEVO: IMPRIMIR LOGO SI EXISTE ---
    if let Some(logo) = &ticket.logo_image {
        if !logo.is_empty() {
            // Ancho max: 300 puntos para que quede lindo en 58mm
            bytes.extend_from_slice(&procesar_imagen_escpos(logo, 300));
        }
    }

    // Encabezado
    bytes.extend_from_slice(format!("{}\n", ticket.header).as_bytes());

    if ticket.show_date && !es_tarjeta_presentacion {
        let fecha = chrono::Local::now().format("%d/%m/%Y %H:%M").to_string();
        bytes.extend_from_slice(format!("Fecha: {}\n", fecha).as_bytes());
    }
    
    let divisor = if ticket.paper_size == "80" { "------------------------------------------------\n" } else { "--------------------------------\n" };
    bytes.extend_from_slice(divisor.as_bytes());

    // Productos
    if !es_tarjeta_presentacion {
        bytes.extend_from_slice(&[27, 97, 0]); // Alineación Izquierda
        for prod in &ticket.products {
            let subtotal = (prod.qty as f64) * prod.price;
            bytes.extend_from_slice(format!("{}x {}\n", prod.qty, prod.name).as_bytes());
            bytes.extend_from_slice(&[27, 97, 2]); // Alineación Derecha
            bytes.extend_from_slice(format!("${:.2}\n", subtotal).as_bytes());
            bytes.extend_from_slice(&[27, 97, 0]); // Volver Izquierda
        }
        bytes.extend_from_slice(divisor.as_bytes());
        bytes.extend_from_slice(&[27, 97, 2]); 
        bytes.extend_from_slice(&[27, 69, 1]); // Negrita On
        bytes.extend_from_slice(format!("TOTAL: ${:.2}\n", ticket.total).as_bytes());
        bytes.extend_from_slice(&[27, 69, 0]); // Negrita Off
        bytes.extend_from_slice(divisor.as_bytes());
    }

    bytes.extend_from_slice(&[27, 97, 1]); // Centrado de nuevo

    // --- NUEVO: IMPRIMIR QR SI ESTÁ ACTIVADO ---
    if ticket.print_qr {
        if let Some(qr) = &ticket.qr_image {
            if !qr.is_empty() {
                // Ancho max: 200 puntos (más chico que el logo)
                bytes.extend_from_slice(&procesar_imagen_escpos(qr, 200));
            }
        }
    }

    // Pie de página (se oculta en tarjetas de presentación)
    if !es_tarjeta_presentacion {
        bytes.extend_from_slice(format!("{}\n", ticket.footer).as_bytes());
    }
    
    // Finalización: Avance y Corte
    bytes.extend_from_slice(&[27, 100, 5]);
    bytes.extend_from_slice(&[29, 86, 66, 0]);

    // ENVIAMOS A WINDOWS (Modo RAW)
    #[cfg(target_os = "windows")]
    unsafe {
        let mut printer_name: Vec<u16> = ticket.target_device.encode_utf16().chain(std::iter::once(0)).collect();
        let mut h_printer: HANDLE = std::mem::zeroed();

        if OpenPrinterW(printer_name.as_mut_ptr(), &mut h_printer, std::ptr::null_mut()) == 0 {
            return Err("No se encontró la impresora. Verifica que esté conectada.".to_string());
        }

        let mut doc_name: Vec<u16> = "Ticket Estado Play\0".encode_utf16().collect();
        let mut data_type: Vec<u16> = "RAW\0".encode_utf16().collect();

        let doc_info = DOC_INFO_1W {
            pDocName: doc_name.as_mut_ptr(),
            pOutputFile: std::ptr::null_mut(),
            pDatatype: data_type.as_mut_ptr(),
        };

        if StartDocPrinterW(h_printer, 1, &doc_info as *const _ as *const _) == 0 {
            ClosePrinter(h_printer);
            return Err("Windows rechazó el inicio del ticket.".to_string());
        }

        if StartPagePrinter(h_printer) == 0 {
            EndDocPrinter(h_printer);
            ClosePrinter(h_printer);
            return Err("Error al crear la página.".to_string());
        }

        let mut bytes_written = 0;
        let success = WritePrinter(h_printer, bytes.as_ptr() as *const _, bytes.len() as u32, &mut bytes_written);

        EndPagePrinter(h_printer);
        EndDocPrinter(h_printer);
        ClosePrinter(h_printer);

        if success == 0 {
            return Err("Windows bloqueó el envío de datos a la ticketera.".to_string());
        }
    }

    Ok("¡Impresión enviada correctamente!".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![escanear_impresoras, imprimir_ticket])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}