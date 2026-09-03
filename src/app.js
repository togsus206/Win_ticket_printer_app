
        const products = [];
        
        // Elementos del DOM de Ventas
        const productForm = document.getElementById('product-form');
        const prodNameInput = document.getElementById('prod-name');
        const prodQtyInput = document.getElementById('prod-qty');
        const prodPriceInput = document.getElementById('prod-price');
        const tableBody = document.getElementById('products-table-body');
        const totalAmountText = document.getElementById('total-amount');
        
        // Elementos del Ticket
        const ticketItemsContainer = document.getElementById('t-items');
        const ticketTotalText = document.getElementById('t-total');
        const ticketDateText = document.getElementById('t-date');

        // Referencias para Ajustes
        const cfgHeader = document.getElementById('cfg-header');
        const cfgFooter = document.getElementById('cfg-footer');
        const cfgShowDate = document.getElementById('cfg-show-date');
        const cfgQrFile = document.getElementById('cfg-qr-file');
        const btnDeleteQr = document.getElementById('btn-delete-qr');
        const paperRadios = document.getElementsByName('paper-size');

        const tHeader = document.getElementById('t-header');
        const tFooter = document.getElementById('t-footer');
        const tDate = document.getElementById('t-date');
        const ticketView = document.getElementById('ticket-view');
        const tQrPlaceholder = document.getElementById('t-qr-placeholder');
        const tLogoPlaceholder = document.getElementById('t-logo-placeholder');

        const cfgLogoFile = document.getElementById('cfg-logo-file');
        const btnDeleteLogo = document.getElementById('btn-delete-logo');
        const cfgShowQr = document.getElementById('cfg-show-qr');
        let customLogoBase64 = null;
        let customQrBase64 = null;
        let defaultQrBase64 = null;


        // --- SISTEMA DE PESTAÑAS (NAVEGACIÓN) ---
        function switchTab(tabName) {
            document.getElementById('tab-sales-btn').classList.remove('active');
            document.getElementById('tab-history-btn').classList.remove('active');
            document.getElementById('tab-settings-btn').classList.remove('active');
            
            document.getElementById('panel-sales').classList.remove('active');
            document.getElementById('panel-history').classList.remove('active');
            document.getElementById('panel-settings').classList.remove('active');

            if (tabName === 'sales') {
                document.getElementById('tab-sales-btn').classList.add('active');
                document.getElementById('panel-sales').classList.add('active');
            } else if (tabName === 'history') {
                document.getElementById('tab-history-btn').classList.add('active');
                document.getElementById('panel-history').classList.add('active');
                renderHistoryTable(); // Renderiza la tabla cada vez que entramos a verla
            } else {
                document.getElementById('tab-settings-btn').classList.add('active');
                document.getElementById('panel-settings').classList.add('active');
            }
        }

        // Actualizar la fecha del ticket en tiempo real
        function updateTicketDate() {
            const now = new Date();
            const formattedDate = now.toLocaleDateString() + ' ' + now.toLocaleTimeString([], {hour: '2-digit', minute:'2-digit'});
            ticketDateText.textContent = `Fecha: ${formattedDate}`;
        }
        updateTicketDate();

        // Agregar producto
        productForm.addEventListener('submit', (e) => {
            e.preventDefault();

            const name = prodNameInput.value.trim();
            const qty = parseInt(prodQtyInput.value);
            const price = parseFloat(prodPriceInput.value);

            // Solo verificamos que el nombre no esté vacío y que los otros sean números válidos
            if (!name || isNaN(qty) || isNaN(price)) {
                alert('Por favor, completa todos los campos numéricos correctamente.');
                return;
            }

            products.push({ name, qty, price });
            
            prodNameInput.value = '';
            prodQtyInput.value = '1';
            prodPriceInput.value = '';
            prodNameInput.focus();

            updateUI();
        });


        function updateUI() {
            tableBody.innerHTML = '';
            ticketItemsContainer.innerHTML = '';
            
            let total = 0;

            const formatCurrency = (val) => {
                return val < 0 ? `-$${Math.abs(val).toFixed(2)}` : `$${val.toFixed(2)}`;
            };

            products.forEach((prod, index) => {
                const subtotal = prod.qty * prod.price;
                total += subtotal;

                // DIBUJAR EN LA TABLA DE LA IZQUIERDA ---
                const tr = document.createElement('tr');
                tr.innerHTML = `
                    <td>${prod.name}</td>
                    <td>${prod.qty}</td>
                    <td>${formatCurrency(prod.price)}</td>
                    <td>${formatCurrency(subtotal)}</td>
                    <td class="actions-cell">
                        <button class="btn-primary" onclick="editProduct(${index})" style="padding: 6px 10px; font-size: 0.85rem; background-color: #f59e0b; margin-right: 5px;">✏️ Editar</button>
                        <button class="btn-danger" onclick="deleteProduct(${index})">Eliminar</button>
                    </td>
                `;
                tableBody.appendChild(tr);

                //  DIBUJAR EN EL TICKET (VISTA PREVIA) ---
                const ticketItem = document.createElement('div');
                ticketItem.className = 'ticket-item';
                ticketItem.style.width = '100%';
                ticketItem.style.marginBottom = '4px';

                if (prod.qty > 1) {
                    ticketItem.innerHTML = `
                        <div style="display: flex; flex-direction: column; width: 100%;">
                            <span>${prod.qty}x ${prod.name}</span>
                            <div style="display: flex; justify-content: space-between; color: #555; font-size: 0.9em; padding-left: 15px;">
                                <span>(${formatCurrency(prod.price)} c/u)</span>
                                <span style="color: black; font-weight: bold;">${formatCurrency(subtotal)}</span>
                            </div>
                        </div>
                    `;
                } else {
                    // Vista previa adaptativa para textos largos
                    ticketItem.innerHTML = `
                        <div style="display: flex; justify-content: space-between; width: 100%; flex-wrap: wrap; gap: 2px;">
                            <span style="flex: 1; min-width: 60%; word-break: break-word;">${prod.qty}x ${prod.name}</span>
                            <span style="font-weight: normal; text-align: right; flex-shrink: 0;">${formatCurrency(subtotal)}</span>
                        </div>
                    `;
                }
                
                ticketItemsContainer.appendChild(ticketItem);
            });

            const formattedTotal = total < 0 ? `-$${Math.abs(total).toFixed(2)}` : `$${total.toFixed(2)}`;
            totalAmountText.textContent = formattedTotal;
            ticketTotalText.textContent = formattedTotal;
            
            updateTicketDate();
            updateQR();
        }

        // Eliminar Producto
        window.deleteProduct = function(index) {
            products.splice(index, 1);
            updateUI();
        }

        // EDITAR PRODUCTO 
        window.editProduct = function(index) {
            // 1. Agarramos los datos del producto seleccionado
            const prod = products[index];
            
            // 2. Los devolvemos a los casilleros de arriba
            prodNameInput.value = prod.name;
            prodQtyInput.value = prod.qty;
            prodPriceInput.value = prod.price;
            
            // 3. Lo borramos de la lista de abajo para no duplicarlo
            products.splice(index, 1);
            updateUI();
            
            // 4. Hacemos que el teclado/cursor vaya directo al precio para corregir rápido
            prodPriceInput.focus();
        }

        // --- PERSISTENCIA LOCAL (SharedPreferences) ---
        function loadSettings() {
            cfgHeader.value = localStorage.getItem('header') || 'Estado Play\nCórdoba Capital';
            cfgFooter.value = localStorage.getItem('footer') || '¡Gracias por tu compra!';
            cfgShowDate.checked = localStorage.getItem('showDate') !== 'false';
            
            const savedPaper = localStorage.getItem('paperSize') || '58';
            paperRadios.forEach(radio => {
                if (radio.value === savedPaper) radio.checked = true;
            });

            customQrBase64 = localStorage.getItem('customQr') || null;
            if (customQrBase64) {
                btnDeleteQr.style.display = 'block';
            }

            customLogoBase64 = localStorage.getItem('customLogo') || null;
            if (customLogoBase64) {
                btnDeleteLogo.style.display = 'block';
            }

            document.getElementById('cfg-image-size').value = localStorage.getItem('imageSize') || 'medio';

            applySettings();
        }

        function applySettings() {
            tHeader.innerText = cfgHeader.value;
            tFooter.innerText = cfgFooter.value;
            tDate.style.display = cfgShowDate.checked ? 'block' : 'none';
            
            const is80mm = document.querySelector('input[name="paper-size"]:checked').value === '80';
            ticketView.style.width = is80mm ? '380px' : '290px';

            saveSettingsToLocalStorage();
            updateQR();
        }

        function saveSettingsToLocalStorage() {
            localStorage.setItem('header', cfgHeader.value);
            localStorage.setItem('footer', cfgFooter.value);
            localStorage.setItem('showDate', cfgShowDate.checked);
            localStorage.setItem('paperSize', document.querySelector('input[name="paper-size"]:checked').value);
            if (customQrBase64) {
                localStorage.setItem('customQr', customQrBase64);
            } else {
                localStorage.removeItem('customQr');
            }

            if (customLogoBase64) {
                localStorage.setItem('customLogo', customLogoBase64);
            } else {
                localStorage.removeItem('customLogo');
            }

            localStorage.setItem('imageSize', document.getElementById('cfg-image-size').value);
        }

        // Generación de Imágenes (Logo y QR) para Vista Previa y Rust
        function updateQR() {
            // Leemos qué tamaño se eligio en los Ajustes
            const sizeMode = document.getElementById('cfg-image-size') ? document.getElementById('cfg-image-size').value : 'medio';
            
            // Asignamos los píxeles para la pantalla según la opción
            let logoSize = sizeMode === 'chico' ? '80px' : (sizeMode === 'grande' ? '160px' : '120px');
            let qrSize = sizeMode === 'chico' ? '65px' : (sizeMode === 'grande' ? '110px' : '85px');

            // --- 1. Lógica del LOGO ---
            tLogoPlaceholder.innerHTML = '';
            if (customLogoBase64) {
                const imgLogo = document.createElement('img');
                imgLogo.src = customLogoBase64;
                imgLogo.style.width = logoSize; // Aplicamos el tamaño dinámico
                imgLogo.style.objectFit = 'contain';
                tLogoPlaceholder.appendChild(imgLogo);
            }

            // --- 2. Lógica del QR ---
            tQrPlaceholder.innerHTML = '';
            defaultQrBase64 = null; 

            if (!cfgShowQr || !cfgShowQr.checked) {
                return;
            }
            
            if (customQrBase64) {
                const imgQr = document.createElement('img');
                imgQr.src = customQrBase64;
                imgQr.style.width = qrSize; // Aplicamos el tamaño dinámico
                imgQr.style.height = qrSize;
                tQrPlaceholder.appendChild(imgQr);
            } else {
                const totalAmount = products.reduce((acc, p) => acc + (p.qty * p.price), 0);
                const qrContent = `Total: $${totalAmount.toFixed(2)}`;
                
                try {
                    const tempDiv = document.createElement('div');
                    
                    new QRCode(tempDiv, {
                        text: qrContent,
                        width: 120, 
                        height: 120,
                        colorDark : "#000000",
                        colorLight : "#ffffff"
                    });
                    
                    setTimeout(() => {
                        const canvas = tempDiv.querySelector('canvas');
                        if (canvas) {
                            defaultQrBase64 = canvas.toDataURL("image/png"); 
                            
                            const imgQr = document.createElement('img');
                            imgQr.src = defaultQrBase64;
                            
                            // Aplicamos el tamaño dinámico
                            imgQr.style.width = qrSize;
                            imgQr.style.height = qrSize;
                            
                            tQrPlaceholder.innerHTML = ''; 
                            tQrPlaceholder.appendChild(imgQr);
                        } else {
                            tQrPlaceholder.innerText = "Error leyendo Canvas de QR";
                        }
                    }, 50);
                    
                } catch (err) {
                    console.error("Error fatal:", err);
                    tQrPlaceholder.innerText = "Librería no compatible";
                }
            }
        }


        // Event Listeners para Ajustes
        cfgHeader.addEventListener('input', applySettings);
        cfgFooter.addEventListener('input', applySettings);
        cfgShowDate.addEventListener('change', applySettings);
        cfgShowQr.addEventListener('change', applySettings);
        paperRadios.forEach(radio => radio.addEventListener('change', applySettings));

        cfgQrFile.addEventListener('change', (e) => {
            const file = e.target.files[0];
            if (file) {
                const reader = new FileReader();
                reader.onload = function(event) {
                    customQrBase64 = event.target.result;
                    btnDeleteQr.style.display = 'block';
                    applySettings();
                };
                reader.readAsDataURL(file);
            }
        });

        btnDeleteQr.addEventListener('click', () => {
            customQrBase64 = null;
            cfgQrFile.value = '';
            btnDeleteQr.style.display = 'none';
            applySettings();
        });

        // --- GESTIÓN DEL ARCHIVO DEL LOGO ---
        cfgLogoFile.addEventListener('change', (e) => {
            const file = e.target.files[0];
            if (file) {
                const reader = new FileReader();
                reader.onload = function(event) {
                    customLogoBase64 = event.target.result; // Guardamos la imagen en Base64
                    btnDeleteLogo.style.display = 'block'; // Mostramos el botón rojo de eliminar
                    applySettings(); // Aplicamos los cambios
                };
                reader.readAsDataURL(file);
            }
        });

        // Evento para eliminar el logo
        btnDeleteLogo.addEventListener('click', () => {
            customLogoBase64 = null; // Vaciamos la variable
            cfgLogoFile.value = ''; // Limpiamos el input
            btnDeleteLogo.style.display = 'none'; // Ocultamos el botón rojo
            applySettings();
        });


        // Carga automática inicial de la impresora guardada
        document.addEventListener("DOMContentLoaded", () => {
            const savedPrinter = localStorage.getItem('selectedPrinter');
            if (savedPrinter) {
                document.getElementById('cfg-printer').innerHTML = `<option value="${savedPrinter}">${savedPrinter}</option>`;
            }
        });

        document.getElementById('cfg-printer').addEventListener('change', (e) => {
            localStorage.setItem('selectedPrinter', e.target.value);
        });

        document.getElementById('cfg-image-size').addEventListener('change', applySettings);

        // NUEVO ESCANEO INTELIGENTE (Windows API)
        async function scanPrinters() {
            try {
                const impresoras = await window.__TAURI__.core.invoke('escanear_impresoras');
                const select = document.getElementById('cfg-printer');
                select.innerHTML = '<option value="">Selecciona tu ticketera</option>';

                if (impresoras.length === 0) {
                    alert("No se detectaron impresoras instaladas en Windows.");
                    return;
                }

                impresoras.forEach(nombre => {
                    const option = document.createElement('option');
                    option.value = nombre;
                    option.textContent = nombre;
                    select.appendChild(option);
                });

                // Si ya había una guardada, la vuelve a seleccionar tras el escaneo
                const savedPrinter = localStorage.getItem('selectedPrinter');
                if (savedPrinter && impresoras.includes(savedPrinter)) {
                    select.value = savedPrinter;
                }

            } catch (error) {
                console.error("Error al escanear:", error);
                alert("Hubo un problema al buscar impresoras en Windows.");
            }
        }

        // PAYLOAD
        function getTicketDataPayload() {
            const paperSize = document.querySelector('input[name="paper-size"]:checked').value;
            const targetDevice = document.getElementById('cfg-printer').value;

            return {
                header: document.getElementById('cfg-header').value,
                footer: document.getElementById('cfg-footer').value,
                show_date: document.getElementById('cfg-show-date').checked,
                print_qr: document.getElementById('cfg-show-qr').checked, // Mandamos si quiere QR o no
                logo_image: customLogoBase64, // Mandamos el logo si existe
                qr_image: customQrBase64 ? customQrBase64 : defaultQrBase64, // Mandamos el personalizado o el genérico
                paper_size: paperSize,
                target_device: targetDevice,
                products: products,
                image_size: document.getElementById('cfg-image-size').value,
                total: products.reduce((acc, p) => acc + (p.qty * p.price), 0)
            };
        }


        // --- MANDAR A IMPRIMIR REAL ---
        async function printTicket() {
            const payload = getTicketDataPayload();
            
            if (payload.products.length === 0) {
                alert("Carga al menos un producto antes de imprimir.");
                return;
            }

            try {
                const result = await window.__TAURI__.core.invoke('imprimir_ticket', {
                    ticket: payload,
                    esTarjetaPresentacion: false
                });
                
                //Guardamos en el historial solo si no hubo error en Rust
                saveTicketToHistory(payload);

                alert(result);
            } catch (error) {
                console.error("Error al imprimir:", error);
                alert(error);
            }
        }

        async function printCard() {
            const payload = getTicketDataPayload();

            try {
                const result = await window.__TAURI__.core.invoke('imprimir_ticket', {
                    ticket: payload,
                    esTarjetaPresentacion: true
                });
                alert(result);
            } catch (error) {
                console.error("Error al imprimir tarjeta:", error);
                alert(error);
            }
        }

        // --- GESTIÓN DEL HISTORIAL DE TICKETS ---
        
        // Guarda el ticket impreso en el localStorage
        function saveTicketToHistory(payload) {
            let history = JSON.parse(localStorage.getItem('ticket_history')) || [];
            
            const ticketParaHistorial = Object.assign({}, payload);
            ticketParaHistorial.logo_image = null; 
            if (customQrBase64) {
                ticketParaHistorial.qr_image = null; // Borramos el QR solo si es personalizado
            }
            
            // Le agregamos marca de tiempo local
            const now = new Date();
            ticketParaHistorial.timestamp = now.toLocaleDateString() + ' ' + now.toLocaleTimeString([], {hour: '2-digit', minute:'2-digit'});
            ticketParaHistorial.id = Date.now(); 

            history.unshift(ticketParaHistorial); 
            
            // Usamos un bloque try-catch por si el historial se llega a llenar a futuro
            try {
                localStorage.setItem('ticket_history', JSON.stringify(history));
            } catch (error) {
                alert("Error de memoria: El historial está muy lleno. Por favor, andá a la pestaña Historial y vacialo.");
            }
        }

        // Función rápida para limpiar todos los filtros a la vez
        window.clearFilters = function() {
            if(document.getElementById('search-history')) document.getElementById('search-history').value = '';
            
            const startInput = document.getElementById('date-start');
            const endInput = document.getElementById('date-end');
            
            if(startInput) {
                startInput.value = '';
                startInput.max = ''; 
            }
            if(endInput) {
                endInput.value = '';
                endInput.min = ''; 
            }
            
            renderHistoryTable();
        }

        // Función para validar que las fechas tengan sentido
        window.validateDateRange = function(source) {
            const startInput = document.getElementById('date-start');
            const endInput = document.getElementById('date-end');

            // 1. Verificamos que no se crucen las fechas si ambas están llenas
            if (startInput.value && endInput.value) {
                if (startInput.value > endInput.value) {
                    alert("⚠️ La fecha 'Desde' no puede ser posterior a la fecha 'Hasta'.");
                    
                    // Si se equivocó, le corregimos el casillero automáticamente
                    if (source === 'start') {
                        startInput.value = endInput.value;
                    } else {
                        endInput.value = startInput.value;
                    }
                }
            }

            // 2. Bloqueamos los días inválidos en el calendario visualmente
            if (startInput.value) {
                endInput.min = startInput.value; // El 'Hasta' no puede ser menor al 'Desde'
            } else {
                endInput.min = "";
            }

            if (endInput.value) {
                startInput.max = endInput.value; // El 'Desde' no puede superar al 'Hasta'
            } else {
                startInput.max = "";
            }

            // 3. Ahora sí, filtramos la tabla
            renderHistoryTable();
        }

        // Dibuja la tabla con el historial (Filtros de texto y Fechas combinados)
        window.renderHistoryTable = function() {
            const historyTableBody = document.getElementById('history-table-body');
            historyTableBody.innerHTML = '';
            
            let history = JSON.parse(localStorage.getItem('ticket_history')) || [];

            // 1. Recopilamos qué hay en los inputs
            const searchInput = document.getElementById('search-history');
            const searchTerm = searchInput ? searchInput.value.toLowerCase().trim() : '';
            
            const dateStartInput = document.getElementById('date-start') ? document.getElementById('date-start').value : '';
            const dateEndInput = document.getElementById('date-end') ? document.getElementById('date-end').value : '';

            // 2. Si hay ALGO escrito o alguna fecha seleccionada, empezamos a filtrar
            if (searchTerm !== '' || dateStartInput !== '' || dateEndInput !== '') {
                
                // Preparamos las fechas de inicio y fin (si existen) a formato matemático (milisegundos)
                const startDate = dateStartInput ? new Date(dateStartInput + 'T00:00:00').getTime() : null;
                const endDate = dateEndInput ? new Date(dateEndInput + 'T23:59:59').getTime() : null;

                history = history.filter(t => {
                    let pasaFiltroTexto = true;
                    let pasaFiltroFecha = true;

                    // A. Revisión del texto
                    if (searchTerm !== '') {
                        const matchDate = t.timestamp.toLowerCase().includes(searchTerm);
                        const matchProducts = t.products.some(p => p.name.toLowerCase().includes(searchTerm));
                        pasaFiltroTexto = matchDate || matchProducts;
                    }

                    // B. Revisión del rango de fechas
                    if (startDate || endDate) {
                        // El ticket dice "31/7/2026 10:30". Lo cortamos y armamos una fecha real.
                        const partes = t.timestamp.split(' ')[0].split('/'); 
                        // Formato: Date(año, mes (arranca de 0), día)
                        const ticketTime = new Date(partes[2], partes[1] - 1, partes[0]).getTime();

                        if (startDate && ticketTime < startDate) pasaFiltroFecha = false;
                        if (endDate && ticketTime > endDate) pasaFiltroFecha = false;
                    }

                    // El ticket solo sobrevive si pasa ambas pruebas
                    return pasaFiltroTexto && pasaFiltroFecha;
                });
            }

            // Si después de filtrar no quedó nada
            if (history.length === 0) {
                const mensaje = (searchTerm !== '' || dateStartInput || dateEndInput) 
                    ? 'No se encontraron ventas en ese rango o búsqueda.' 
                    : 'No hay tickets registrados en el historial.';
                historyTableBody.innerHTML = `<tr><td colspan="4" style="text-align: center; color: #64748b; padding: 20px;">${mensaje}</td></tr>`;
                return;
            }

            // 3. Dibujamos los tickets que sobrevivieron al filtro
            let fechaActual = ""; 

            history.forEach(t => {
                const partesFecha = t.timestamp.split(' ');
                const fechaTicket = partesFecha[0]; 
                const horaTicket = t.timestamp.substring(fechaTicket.length).trim(); 

                if (fechaTicket !== fechaActual) {
                    const trSeparador = document.createElement('tr');
                    trSeparador.style.backgroundColor = 'var(--border)'; 
                    trSeparador.innerHTML = `
                        <td colspan="4" style="text-align: center; font-weight: bold; color: #334155; padding: 8px; font-size: 0.95rem;">
                            📅 Ventas del día: ${fechaTicket}
                        </td>
                    `;
                    historyTableBody.appendChild(trSeparador);
                    fechaActual = fechaTicket; 
                }
                
                const prodSummary = t.products.map(p => `${p.qty}x ${p.name}`).join(', ');

                const tr = document.createElement('tr');
                tr.innerHTML = `
                    <td><strong>${horaTicket}</strong></td>
                    <td style="max-width: 250px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" title="${prodSummary}">
                        ${prodSummary}
                    </td>
                    <td><strong>$${t.total.toFixed(2)}</strong></td>
                    <td class="actions-cell">
                        <div style="display: flex; gap: 8px; align-items: center; justify-content: flex-end;">

                            <button onclick="copyTicketProducts(${t.id}, this)" style="width: 100px; padding: 6px 0; font-size: 0.85rem; background-color: #3b82f6; color: white; border: none; border-radius: 4px; display: flex; align-items: center; justify-content: center; gap: 5px; margin: 0; cursor: pointer; transition: 0.3s;">
                                📋 Copiar
                            </button>
                            
                            <button class="btn-primary" onclick="reprintTicket(${t.id})" style="width: 120px; padding: 6px 0; font-size: 0.85rem; background-color: #0d9488; display: flex; align-items: center; justify-content: center; gap: 5px; margin: 0;">
                                🖨️ Reimprimir
                            </button>
                            
                            <button class="btn-danger" onclick="deleteHistoryItem(${t.id})" style="width: 120px; padding: 6px 0; font-size: 0.85rem; display: flex; align-items: center; justify-content: center; gap: 5px; margin: 0;">
                                ❌ Borrar
                            </button>
                            
                        </div>
                    </td>
                `;
                historyTableBody.appendChild(tr);
            });
        }

        // Elimina un ticket individual del historial
        window.deleteHistoryItem = function(id) {
            let history = JSON.parse(localStorage.getItem('ticket_history')) || [];
            history = history.filter(item => item.id !== id);
            localStorage.setItem('ticket_history', JSON.stringify(history));
            renderHistoryTable();
        }

        // Vacía todo el historial de una sola vez
        window.clearHistory = function() {
            if (confirm("¿Estás seguro de que quieres borrar todo el historial de ventas de Estado Play? Esta acción no se puede deshacer.")) {
                localStorage.removeItem('ticket_history');
                renderHistoryTable();
            }
        }

        // Reimprime de forma directa un ticket pasado usando Rust
        window.reprintTicket = async function(id) {
            const history = JSON.parse(localStorage.getItem('ticket_history')) || [];
            const ticketToPrint = history.find(item => item.id === id);

            if (!ticketToPrint) {
                alert("No se encontró el ticket seleccionado.");
                return;
            }

            ticketToPrint.logo_image = customLogoBase64;
            if (customQrBase64) {
                ticketToPrint.qr_image = customQrBase64;
            }

            try {
                // Mandamos a imprimir
                const result = await window.__TAURI__.core.invoke('imprimir_ticket', {
                    ticket: ticketToPrint,
                    esTarjetaPresentacion: false
                });
                alert("Reimpresión: " + result);
            } catch (error) {
                alert("Error al reimprimir: " + error);
            }
        }

        // Copiar cadena de texto 
        window.copyTicketProducts = function(id, btnElement) {
            const history = JSON.parse(localStorage.getItem('ticket_history')) || [];
            const ticket = history.find(item => item.id === id);

            if (!ticket) {
                alert("No se encontró el ticket.");
                return;
            }

            // Agarramos los productos y los unimos con un "+" en el medio
            const textoCopiar = ticket.products.map(p => `${p.qty}x ${p.name}`).join(' + ');

            // Usamos la API del sistema para mandarlo al portapapeles
            navigator.clipboard.writeText(textoCopiar).then(() => {
                
                // Guardamos cómo era el botón originalmente
                const originalText = btnElement.innerHTML;
                const originalColor = btnElement.style.backgroundColor;
                
                // Lo ponemos verde con el tilde
                btnElement.innerHTML = '✅ Copiado';
                btnElement.style.backgroundColor = '#10b981'; // Verde
                
                // A los 1.5 segundos, lo devolvemos a la normalidad
                setTimeout(() => {
                    btnElement.innerHTML = originalText;
                    btnElement.style.backgroundColor = originalColor;
                }, 1500);

            }).catch(err => {
                console.error("Error al copiar: ", err);
                alert("Hubo un error al intentar copiar el texto.");
            });
        }

        window.saveOnlineTicket = async function() {
            const payload = getTicketDataPayload(); 
            if (payload.products.length === 0) {
                alert("Carga al menos un producto antes de guardar.");
                return;
            }
            
            // 1. Guardamos en el historial
            saveTicketToHistory(payload);

            // 2. Le sacamos la "foto" al ticket
            try {
                const ticketElement = document.getElementById('ticket-view');
                
                // Le quitamos el zoom temporalmente
                ticketElement.style.transform = 'none';
                
                // MÁGIA: Obligamos al navegador a esperar 50 milisegundos para que 
                // termine de redibujar el ticket en tamaño real antes de sacarle la foto
                await new Promise(resolve => setTimeout(resolve, 50));
                
                // Usamos html2canvas para generar la imagen
                const canvas = await html2canvas(ticketElement, {
                    backgroundColor: '#ffffff',
                    scale: 2 // Escala para alta definición en WhatsApp
                });

                // Le devolvemos el zoom a tu pantalla
                ticketElement.style.transform = '';

                // Convertimos y descargamos
                const imageURL = canvas.toDataURL('image/png');
                const link = document.createElement('a');
                link.download = `Ticket_EstadoPlay_${Date.now()}.png`;
                link.href = imageURL;
                link.click(); 

                alert("¡Ticket guardado en el Historial y descargado como imagen para WhatsApp!");
            } catch (error) {
                console.error("Error al generar la imagen:", error);
                // Si llegara a fallar, le devolvemos el zoom para que no quede roto en pantalla
                document.getElementById('ticket-view').style.transform = '';
                alert("El ticket se guardó en el historial, pero hubo un error al crear la imagen.");
            }
        }
        // Vaciar todos los productos de la venta actual
        window.clearAllProducts = function() {
            if (products.length === 0) {
                return; // Si ya está vacía, no hacemos nada
            }
            
            if (confirm("¿Estás seguro de que querés eliminar todos los productos de esta venta?")) {
                products.length = 0; // Vaciamos el array de productos de un plumazo
                updateUI(); // Llamamos a la función que redibuja la tabla y pone el total en $0.00
            }
        }

        // Inicializar la carga de configuraciones al abrir
        loadSettings();
