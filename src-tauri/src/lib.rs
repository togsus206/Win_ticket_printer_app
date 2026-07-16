use serde::{Deserialize, Serialize};
use serialport;
use std::io::Write;

// Estructura para listar puertos serie / COM detectados
#[derive(Serialize)]
pub struct SerialPortInfo {
    port_name: String,
    port_type: String,
    es_probable_impresora: bool,
}

// Estructuras para mapear los datos del ticket que vienen de JavaScript
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
    paper_size: String, // "58" o "80"
    connection_type: String, // "usb" o "bluetooth"
    target_device: String, // Nombre de puerto COM o de Impresora USB
    products: Vec<ProductoInput>,
    total: f64,
}

// ==========================================
// 🔄 COMANDO 1: ESCANEO INTELIGENTE DE PUERTOS
// ==========================================
#[tauri::command]
fn escanear_puertos() -> Vec<SerialPortInfo> {
    let mut lista_puertos = Vec::new();

    if let Ok(ports) = serialport::available_ports() {
        for p in ports {
            let name_lower = p.port_name.to_lowercase();
            
            // Filtro inteligente de palabras clave para autoselección
            let es_probable = name_lower.contains("mtp") 
                || name_lower.contains("pt-") 
                || name_lower.contains("pos") 
                || name_lower.contains("thermal") 
                || name_lower.contains("printer")
                || name_lower.contains("rfcomm") // Linux
                || name_lower.contains("com");   // Windows

            let tipo = match p.port_type {
                serialport::SerialPortType::UsbPort(_) => "USB".to_string(),
                serialport::SerialPortType::BluetoothPort => "Bluetooth".to_string(),
                _ => "Puerto COM".to_string(),
            };

            lista_puertos.push(SerialPortInfo {
                port_name: p.port_name,
                port_type: tipo,
                es_probable_impresora: es_probable,
            });
        }
    }

    lista_puertos
}

// ==========================================
// 🖨️ COMANDO 2: IMPRESIÓN (WINDOWS / LINUX)
// ==========================================
#[tauri::command]
fn imprimir_ticket(ticket: TicketInput, es_tarjeta_presentacion: bool) -> Result<String, String> {
    // 1. Crear el búfer de bytes ESC/POS
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&[27, 64]); // Inicializar
    bytes.extend_from_slice(&[27, 97, 1]); // Centrado
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
    bytes.extend_from_slice(&[27, 100, 5]); // Avance
    bytes.extend_from_slice(&[29, 86, 66, 0]); // Corte

    // 2. ENVIAR LOS BYTES
    let target = if ticket.target_device.is_empty() {
        "POS-58".to_string()
    } else {
        ticket.target_device.clone()
    };

    // Si es un puerto COM (Windows) o /dev/ (Linux), usamos serialport (Aplica a USB Serial y Bluetooth)
    let is_serial = target.to_uppercase().starts_with("COM") || target.starts_with("/dev/");
    
    if is_serial || ticket.connection_type == "bluetooth" {
        let mut port = serialport::new(&target, 9600)
            .timeout(std::time::Duration::from_millis(1000))
            .open()
            .map_err(|e| format!("No se pudo abrir el puerto {}: {}", target, e))?;

        port.write_all(&bytes)
            .map_err(|e| format!("Error al escribir en la impresora: {}", e))?;
    } else {
        // IMPRESIÓN NATIVA ESTÁNDAR (Sin librerías raras)
        #[cfg(target_os = "windows")]
        {
            // En Windows, si usamos el nombre (ej. POS-58), escribimos directo al recurso local de impresión
            let path = format!("\\\\localhost\\{}", target);
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .open(&path)
                .map_err(|_| format!("Para imprimir por USB en Windows, asegurate de compartir la impresora desde el panel de control con el nombre: {}", target))?;
            
            std::io::Write::write_all(&mut file, &bytes)
                .map_err(|e| format!("Error al enviar el ticket a Windows: {}", e))?;
        }

        #[cfg(target_os = "linux")]
        {
            let path = if target == "POS-58" { "/dev/usb/lp0".to_string() } else { target.clone() };
            let mut file = std::fs::File::create(&path)
                .map_err(|_| "Error al acceder a la impresora en Linux.".to_string())?;
            std::io::Write::write_all(&mut file, &bytes)
                .map_err(|e| format!("Error de escritura USB: {}", e))?;
        }
    }

    Ok("¡Enviado a la impresora correctamente!".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![escanear_puertos, imprimir_ticket])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}