# TicketPrinterApp 🖨️ - Windows / Linux

Aplicación de escritorio nativa desarrollada con **Tauri v2**, **Rust** y un frontend minimalista en **HTML5/JS**. Está diseñada especialmente para el mostrador de **Estado Play** con el objetivo de gestionar ventas rápidas e imprimir tickets o tarjetas de presentación de forma instantánea en ticketeras térmicas de **58mm** o **80mm** mediante conexión **USB** o **Bluetooth**.

---

## ✨ Características Principales

* **Pestañas de Navegación Nativas:** Interfaz súper limpia y cómoda con barra lateral (`🛒 Ventas` y `⚙️ Ajustes`).
* **Previsualización en Tiempo Real:** El simulador de ticket del lateral derecho responde al instante a cada cambio en los productos o ajustes.
* **Ajustes Persistentes (localStorage):** Emula el comportamiento de las `SharedPreferences` de Android para recordar tu encabezado, pie de página y configuración de impresión al cerrar la app.
* **Código QR Dinámico:** Si no subís un QR personalizado, la app autogenera uno dinámico en tiempo real que contiene codificado el total actual de la venta.
* **Detección Inteligente de Puertos:** * **USB:** Impresión nativa con "conectar y usar" (en Windows interactúa directo con la cola de impresión [winspool]; en Linux usa `/dev/usb/lp0`).
    * **Bluetooth:** El backend de Rust filtra y selecciona automáticamente la ticketera Bluetooth mapeada que esté activa en tus puertos COM.
* **Tarjeta de Presentación:** Opción rápida para imprimir un ticket de contacto directo de la tienda (solo logotipo, datos del negocio y pie de página).

---

## 🚀 Requisitos de Desarrollo

### En Ubuntu / Linux (Entorno de Desarrollo)
Antes de compilar, necesitas tener instaladas las librerías de desarrollo del sistema operativo para interactuar con la interfaz gráfica de WebKit y los puertos serie físicos:

```bash
# Actualizar el sistema e instalar dependencias gráficas y de udev
sudo apt update
sudo apt install -y libsoup-3.0-dev libwebkit2gtk-4.1-dev libudev-dev
