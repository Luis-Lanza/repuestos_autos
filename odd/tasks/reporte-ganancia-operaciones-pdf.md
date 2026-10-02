# Reporte de ganancia con operaciones en PDF

## Objetivo
Permitir consultar y exportar un reporte de ganancia bruta por período que muestre, en una tabla ordenada, cada venta y devolución que explica el resultado.

## Decisiones
- Toda la interfaz y el PDF estarán en español.
- El reporte usa fechas locales inclusivas, igual que el reporte de ganancia existente.
- Sólo incluye operaciones que afectan ganancia realizada: ventas confirmadas y devoluciones. Las cancelaciones se excluyen; no incluye entradas ni ajustes de inventario.
- Cada fila muestra fecha/hora, identificador “Venta #ID” o “Devolución #ID · Venta #ID”, producto con nombre y SKU histórico, cantidad con signo, precio final histórico, costo histórico y ganancia bruta de la línea.
- Las devoluciones se atribuyen a la fecha de su evento, incluso si la venta original es anterior.
- El costo proviene exclusivamente de `unit_cost_snapshot_centavos`; nunca se infiere del costo actual del producto.
- Si falta costo histórico, la fila y el resumen lo declaran sin fabricar ganancia.
- La tabla está paginada para mantener la aplicación ágil; el PDF incluye todas las operaciones del período aplicado, no sólo la página actual.
- La exportación PDF refleja exactamente el período y el conjunto de operaciones aplicadas en pantalla.

## Tareas
- [x] R1 Mapear el exportador PDF existente, el detalle de ventas/devoluciones y los límites de paginación/impresión.
- [x] R2 Crear la consulta y contrato IPC sin rutas del detalle de operaciones de ganancia por período, con paginación validada y pruebas de persistencia/IPC. Verificado con `cargo test --manifest-path src-tauri/Cargo.toml --test gross_profit_operations` y `cargo test --manifest-path src-tauri/Cargo.toml`.
- [ ] R3 Integrar la tabla accesible y el estado de carga/error/exportación en Reportes.
- [ ] R4 Generar y guardar el PDF en español con totales, advertencias y operaciones paginadas.
- [ ] R5 Verificar contabilidad, PDF, UI, límites y exportación en Windows.

## Criterios de aceptación
- El usuario puede elegir un período, ver total y operaciones ordenadas, y exportar el mismo resultado a PDF.
- Ventas, devoluciones, costos faltantes y cancelaciones respetan las reglas contables declaradas.
- El PDF es legible, ordenado, está completamente en español y no incluye datos ajenos al reporte.
- La exportación no depende de datos de costo actuales ni modifica información operativa.
