use serde::Deserialize;
use base64::{Engine as _, engine::general_purpose::STANDARD};

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::HANDLE;
#[cfg(target_os = "windows")]
use windows_sys::Win32::Graphics::Printing::{
    OpenPrinterW, ClosePrinter, StartDocPrinterW, EndDocPrinter,
    StartPagePrinter, EndPagePrinter, WritePrinter, DOC_INFO_1W,
};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

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
    image_size: Option<String>, 
}

// ==========================================
// ENVOLVER TEXTO (WORD WRAP)
// ==========================================
fn envolver_texto(texto: &str, ancho_maximo: usize) -> String {
    let mut resultado = String::new();
    
    for linea in texto.lines() {
        let mut longitud_actual = 0;
        
        for palabra in linea.split_whitespace() {
            let longitud_palabra = palabra.chars().count();

            if longitud_actual > 0 && longitud_actual + 1 + longitud_palabra > ancho_maximo {
                resultado.push('\n');
                longitud_actual = 0;
            } else if longitud_actual > 0 {
                resultado.push(' ');
                longitud_actual += 1;
            }

            resultado.push_str(palabra);
            longitud_actual += longitud_palabra;
        }
        resultado.push('\n');
    }
    resultado
}

// ==========================================
// 🛠️ HELPER: CONVERTIR IMAGEN A ESC/POS
// ==========================================
fn procesar_imagen_escpos(base64_str: &str, max_width: u32) -> Vec<u8> {
    let b64_data = if let Some(idx) = base64_str.find(',') {
        &base64_str[idx + 1..]
    } else {
        base64_str
    };

    let decoded = match STANDARD.decode(b64_data) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    let img = match image::load_from_memory(&decoded) {
        Ok(i) => i,
        Err(_) => return Vec::new(),
    };

    let width = (max_width / 8) * 8;
    let img = img.thumbnail(width, 500); 
    let img = img.into_luma8(); 
    
    let (w, h) = img.dimensions();
    let width_bytes = w / 8;

    let mut bytes = Vec::new();
    
    bytes.extend_from_slice(&[29, 118, 48, 0]); 
    bytes.push((width_bytes % 256) as u8); 
    bytes.push((width_bytes / 256) as u8); 
    bytes.push((h % 256) as u8); 
    bytes.push((h / 256) as u8); 

    let mut current_byte: u8 = 0;
    let mut bit_count = 0;

    for pixel in img.pixels() {
        let is_black = pixel[0] < 128; 
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
    
    bytes.extend_from_slice(&[27, 74, 30]); 
    bytes
}

// ==========================================
// 🔄 COMANDO 1: ESCANEAR IMPRESORAS
// ==========================================
#[tauri::command]
fn escanear_impresoras() -> Vec<String> {
    let mut impresoras = Vec::new();
    
    let mut comando = std::process::Command::new("powershell");
    comando.args(&["-Command", "(Get-Printer).Name"]);

    #[cfg(target_os = "windows")]
    comando.creation_flags(0x08000000);

    if let Ok(output) = comando.output() {
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

    // Cálculos dinámicos de tamaño y márgenes ---
    let ancho_caracteres = if ticket.paper_size == "80" { 48 } else { 32 };

    let logo_width = match ticket.image_size.as_deref() {
        Some("chico") => 150,
        Some("medio") => 250,
        _ => 350, // grande o fallback
    };

    let qr_width = match ticket.image_size.as_deref() {
        Some("chico") => 120,
        Some("medio") => 180,
        _ => 250, // grande o fallback
    };

    // Imprimir LOGO con tamaño dinámico
    if let Some(logo) = &ticket.logo_image {
        if !logo.is_empty() {
            bytes.extend_from_slice(&procesar_imagen_escpos(logo, logo_width));
        }
    }

    // Encabezado procesado para no cortar palabras
    let header_prolijo = envolver_texto(&ticket.header, ancho_caracteres);
    bytes.extend_from_slice(format!("{}\n", header_prolijo).as_bytes());

    if ticket.show_date && !es_tarjeta_presentacion {
        let fecha = chrono::Local::now().format("%d/%m/%Y %H:%M").to_string();
        bytes.extend_from_slice(format!("Fecha: {}\n", fecha).as_bytes());
    }
    
    let divisor = if ticket.paper_size == "80" { "------------------------------------------------\n" } else { "--------------------------------\n" };
    bytes.extend_from_slice(divisor.as_bytes());

    // Productos
    if !es_tarjeta_presentacion {
        bytes.extend_from_slice(&[27, 97, 0]); // Alineación Izquierda fija para la lista
        
        for prod in &ticket.products {
            let subtotal = (prod.qty as f64) * prod.price;
            
            if prod.qty > 1 {
                // Renglón 1: 2x Cable USB-c
                bytes.extend_from_slice(format!("{}x {}\n", prod.qty, prod.name).as_bytes());
                
                // Renglón 2: Unitario a la izquierda, Subtotal a la derecha
                let izq = format!("  (${:.2} c/u)", prod.price);
                let der = format!("${:.2}", subtotal);
                
                // Magia matemática: rellenamos con espacios el centro
                let chars_totales = izq.chars().count() + der.chars().count();
                let espacios_faltantes = if ancho_caracteres > chars_totales { ancho_caracteres - chars_totales } else { 1 };
                let espacios = " ".repeat(espacios_faltantes);
                
                bytes.extend_from_slice(format!("{}{}{}\n", izq, espacios, der).as_bytes());
                
            } else {
                // Formato para 1 unidad: Todo en un renglón
                let izq = format!("{}x {}", prod.qty, prod.name);
                let der = format!("${:.2}", subtotal);
                
                // Magia matemática para alinear a los bordes
                let chars_totales = izq.chars().count() + der.chars().count();
                let espacios_faltantes = if ancho_caracteres > chars_totales { ancho_caracteres - chars_totales } else { 1 };
                let espacios = " ".repeat(espacios_faltantes);
                
                bytes.extend_from_slice(format!("{}{}{}\n", izq, espacios, der).as_bytes());
            }
        }
        
        // --- RESTAURAMOS LAS LÍNEAS Y EL TOTAL ---
        bytes.extend_from_slice(divisor.as_bytes());
        bytes.extend_from_slice(&[27, 97, 2]); // Alineación Derecha para el texto del total
        bytes.extend_from_slice(&[27, 69, 1]); // Negrita On
        bytes.extend_from_slice(format!("TOTAL: ${:.2}\n", ticket.total).as_bytes());
        bytes.extend_from_slice(&[27, 69, 0]); // Negrita Off
        bytes.extend_from_slice(divisor.as_bytes());
    }

    bytes.extend_from_slice(&[27, 97, 1]); // Centrado de nuevo para el QR

    // Imprimir QR con tamaño dinámico
    if ticket.print_qr && !es_tarjeta_presentacion {
        if let Some(qr) = &ticket.qr_image {
            if !qr.is_empty() {
                bytes.extend_from_slice(&procesar_imagen_escpos(qr, qr_width));
            }
        }
    }

    // Pie de página procesado para no cortar palabras
    if !es_tarjeta_presentacion {
        let footer_prolijo = envolver_texto(&ticket.footer, ancho_caracteres);
        bytes.extend_from_slice(format!("{}\n", footer_prolijo).as_bytes());
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