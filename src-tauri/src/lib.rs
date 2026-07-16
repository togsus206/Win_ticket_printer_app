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

    // Inicializar impresora [ESC @]
    bytes.extend_from_slice(&[27, 64]);
    
    // Encabezado Centrado
    bytes.extend_from_slice(&[27, 97, 1]); 
    bytes.extend_from_slice(format!("{}\n", ticket.header).as_bytes());

    // Fecha (si corresponde)
    if ticket.show_date && !es_tarjeta_presentacion {
        let fecha = chrono::Local::now().format("%d/%m/%Y %H:%M").to_string();
        bytes.extend_from_slice(format!("Fecha: {}\n", fecha).as_bytes());
    }
    
    // Separador según ancho de papel
    let divisor = if ticket.paper_size == "80" {
        "------------------------------------------------\n" // 80mm
    } else {
        "--------------------------------\n" // 58mm
    };
    bytes.extend_from_slice(divisor.as_bytes());

    // Lista de productos (Si no es tarjeta de presentación)
    if !es_tarjeta_presentacion {
        bytes.extend_from_slice(&[27, 97, 0]); // Alineado Izquierda
        for prod in &ticket.products {
            let subtotal = (prod.qty as f64) * prod.price;
            bytes.extend_from_slice(format!("{}x {}\n", prod.qty, prod.name).as_bytes());

            bytes.extend_from_slice(&[27, 97, 2]); // Alinear Derecha para el precio
            bytes.extend_from_slice(format!("${:.2}\n", subtotal).as_bytes());
            bytes.extend_from_slice(&[27, 97, 0]); // Volver a la izquierda
        }
        bytes.extend_from_slice(divisor.as_bytes());

        // Total
        bytes.extend_from_slice(&[27, 97, 2]); // Derecha
        bytes.extend_from_slice(&[27, 69, 1]);  // Negrita Activa
        bytes.extend_from_slice(format!("TOTAL: ${:.2}\n", ticket.total).as_bytes());
        bytes.extend_from_slice(&[27, 69, 0]);  // Negrita Desactivada
        bytes.extend_from_slice(divisor.as_bytes());
    }

    // Pie de página centrado
    bytes.extend_from_slice(&[27, 97, 1]);
    bytes.extend_from_slice(format!("{}\n", ticket.footer).as_bytes());

    // Avance de papel y corte
    bytes.extend_from_slice(&[27, 100, 5]); // Feed 5 líneas
    bytes.extend_from_slice(&[29, 86, 66, 0]); // Corte total/parcial

    // 2. ENVIAR LOS BYTES SEGÚN EL SISTEMA OPERATIVO Y CONEXIÓN
    if ticket.connection_type == "bluetooth" {
        // En Bluetooth ambos sistemas usan puertos COM / Serie de igual manera
        if ticket.target_device.is_empty() {
            return Err("Por favor, selecciona o escribe el puerto de la impresora.".to_string());
        }

        let mut port = serialport::new(&ticket.target_device, 9600)
            .timeout(std::time::Duration::from_millis(1000))
            .open()
            .map_err(|e| format!("No se pudo abrir el puerto {}: {}", ticket.target_device, e))?;

        port.write_all(&bytes)
            .map_err(|e| format!("Error al escribir en la impresora: {}", e))?;

    } else {
        // --- CONEXIÓN USB COMPILADA PARA WINDOWS ---
        #[cfg(target_os = "windows")]
        {
            let nombre_impresora = if ticket.target_device.is_empty() {
                "POS-58".to_string()
            } else {
                ticket.target_device.clone()
            };

            // winprint utiliza un Printer estructurado para enviar los bytes directamente
            let mut printer = winprint::Printer::new(&nombre_impresora)
                .map_err(|e| format!("No se pudo encontrar la impresora en Windows: {}", e))?;
                
            printer.write_all(&bytes)
                .map_err(|e| format!("Error al enviar datos a la cola de Windows: {}", e))?;
        }

        // --- CONEXIÓN USB COMPILADA PARA LINUX (Para tus pruebas en Ubuntu) ---
        #[cfg(target_os = "linux")]
        {
            let ruta_impresora = if !ticket.target_device.is_empty() {
                ticket.target_device.clone()
            } else {
                // Autodetección nativa de Linux
                if std::path::Path::new("/dev/usb/lp0").exists() {
                    "/dev/usb/lp0".to_string()
                } else if std::path::Path::new("/dev/usb/lp1").exists() {
                    "/dev/usb/lp1".to_string()
                } else {
                    return Err("No se encontró impresora USB en /dev/usb/lp0 o lp1.".to_string());
                }
            };

            let mut file = std::fs::File::create(&ruta_impresora)
                .map_err(|_| "Error al acceder a la impresora USB. ¿Tenés permisos del grupo lp?".to_string())?;
            
            file.write_all(&bytes)
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