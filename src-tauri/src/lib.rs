use serde::Deserialize;

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
    paper_size: String,
    target_device: String, 
    products: Vec<ProductoInput>,
    total: f64,
}

// ==========================================
// 🔄 COMANDO 1: ESCANEAR IMPRESORAS DE WINDOWS
// ==========================================
#[tauri::command]
fn escanear_impresoras() -> Vec<String> {
    let mut impresoras = Vec::new();
    
    // Le pedimos la lista de impresoras a Windows nativamente
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
// 🖨️ COMANDO 2: IMPRESIÓN DIRECTA (WinSpool)
// ==========================================
#[tauri::command]
fn imprimir_ticket(ticket: TicketInput, es_tarjeta_presentacion: bool) -> Result<String, String> {
    if ticket.target_device.is_empty() {
        return Err("No seleccionaste ninguna impresora.".to_string());
    }

    // 1. Armamos los bytes del ticket igual que antes
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[27, 64]);
    bytes.extend_from_slice(&[27, 97, 1]);
    bytes.extend_from_slice(format!("{}\n", ticket.header).as_bytes());

    if ticket.show_date && !es_tarjeta_presentacion {
        let fecha = chrono::Local::now().format("%d/%m/%Y %H:%M").to_string();
        bytes.extend_from_slice(format!("Fecha: {}\n", fecha).as_bytes());
    }
    
    let divisor = if ticket.paper_size == "80" { "------------------------------------------------\n" } else { "--------------------------------\n" };
    bytes.extend_from_slice(divisor.as_bytes());

    if !es_tarjeta_presentacion {
        bytes.extend_from_slice(&[27, 97, 0]);
        for prod in &ticket.products {
            let subtotal = (prod.qty as f64) * prod.price;
            bytes.extend_from_slice(format!("{}x {}\n", prod.qty, prod.name).as_bytes());
            bytes.extend_from_slice(&[27, 97, 2]);
            bytes.extend_from_slice(format!("${:.2}\n", subtotal).as_bytes());
            bytes.extend_from_slice(&[27, 97, 0]);
        }
        bytes.extend_from_slice(divisor.as_bytes());
        bytes.extend_from_slice(&[27, 97, 2]);
        bytes.extend_from_slice(&[27, 69, 1]);
        bytes.extend_from_slice(format!("TOTAL: ${:.2}\n", ticket.total).as_bytes());
        bytes.extend_from_slice(&[27, 69, 0]);
        bytes.extend_from_slice(divisor.as_bytes());
    }

    bytes.extend_from_slice(&[27, 97, 1]);
    bytes.extend_from_slice(format!("{}\n", ticket.footer).as_bytes());
    bytes.extend_from_slice(&[27, 100, 5]);
    bytes.extend_from_slice(&[29, 86, 66, 0]);

    // 2. ENVIAMOS LOS DATOS A WINDOWS (Modo RAW)
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
        .invoke_handler(tauri::generate_handler![escanear_impresoras, imprimir_ticket]) // Actualizamos el nombre de la función acá
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}