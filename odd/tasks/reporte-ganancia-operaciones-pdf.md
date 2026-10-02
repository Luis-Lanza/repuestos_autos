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
- [x] R3 Integrar el contrato paginado de operaciones y la tabla accesible con estados de carga/error y paginación en Reportes (sin implementar exportación PDF). Corrección de auditoría: paginación disponible mientras el resumen carga/falla; la tabla y el foco del paginador se conservan durante la carga de página; se rechazan solicitudes duplicadas y respuestas obsoletas. Verificado con `npx tsx --test --import ./test/react-dom.ts src/ui/reports/gross-profit-report-flow.test.ts src/ui/reports/gross-profit-report-screen.mounted.test.ts src/ui/w9-evidence-audit.test.ts`.
- [x] R4 Generar y guardar el PDF nativo en español con totales, advertencias y todas las operaciones del período aplicado; el diálogo permanece en Rust y el IPC no expone rutas. Correcciones de auditoría: signo ASCII visible, rechazo de devoluciones acumuladas superiores a la cantidad vendida y error explícito de límite de recursos con aviso en español. Verificado con las pruebas enfocadas de exportación Rust y UI/IPC, suite completa Rust/frontend, typecheck, build y diff.
- [ ] R5 Verificar contabilidad, PDF, UI, límites y exportación en Windows. Corrección de compilación desktop: el fixture compartido de licencia ahora presta `&Path`; la prueba desktop solicitada no pudo compilarse en Linux por falta de GTK/GLib development libraries.

## Criterios de aceptación
- El usuario puede elegir un período, ver total y operaciones ordenadas, y exportar el mismo resultado a PDF.
- Ventas, devoluciones, costos faltantes y cancelaciones respetan las reglas contables declaradas.
- El PDF es legible, ordenado, está completamente en español y no incluye datos ajenos al reporte.
- La exportación no depende de datos de costo actuales ni modifica información operativa.
