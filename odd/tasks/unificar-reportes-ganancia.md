# Unificar ganancia en Reportes

## Objetivo
Eliminar la sección independiente y sobredimensionada de ganancia bruta. Integrar el reporte de ganancia dentro del filtro existente `Tipo de movimiento` de Reportes mediante la opción visible `Ganancia bruta`.

## Decisiones
- Reportes conserva una sola zona de filtros y resultados.
- El selector comienza en `Todos` y agrega `Ganancia bruta` como modo de reporte de UI; no se envía como un tipo de movimiento de inventario al backend.
- Al elegir `Ganancia bruta`, el usuario aplica juntos `Desde`, `Hasta` y el tipo. Se muestran el total, la tabla histórica de ventas/devoluciones y `Exportar PDF` en el área de resultados existente.
- En modo ganancia se oculta el filtro de producto: el contrato actual no admite un filtro histórico por producto y no se debe simular con datos actuales.
- Los modos de inventario conservan su búsqueda de producto, tabla, paginación y exportación actuales.
- Ganancia conserva nombres/SKU/precios/costos históricos y fechas de eventos; no se convierte en filas de inventario ni consulta datos actuales del catálogo.

## Tareas
- [x] U1 Mapear los flujos de Reportes, ledger, ganancia, accesibilidad y contratos.
- [x] U2 Crear un orquestador único de filtros/modos y trasladar ganancia al área de resultados. Evidencia: `770b708 feat(reports): unify gross profit mode`.
- [x] U3 Eliminar la sección duplicada, preservar accesibilidad y validar regresiones de movimientos/ganancia/PDF. Evidencia: 38 pruebas enfocadas y `npm run typecheck:tests` pasaron; `git diff --check` pasó.
- [x] U3a Unificar posición y estilo de la acción contextual Exportar PDF entre modos. Evidencia: `e35a784 fix(reports): stabilize PDF export action`; 20 pruebas mounted enfocadas verifican posición, modo listo y acción deshabilitada durante carga/error/vacío; `git diff --check` pasó.
- [x] U4 Verificar el modo Ganancia bruta y el PDF unificado en Windows. Evidencia: validación manual del usuario confirmada como funcional y visualmente correcta.

## Criterios de aceptación
- Reportes muestra una sola zona de filtros; no hay un formulario de período independiente para ganancia.
- `Tipo de movimiento` incluye `Ganancia bruta` y no rompe los tipos de inventario.
- Ganancia muestra únicamente datos históricos correctos y exporta el período aplicado a PDF.
- Todo texto visible está en español y los estados de carga, error, vacío, paginación y accesibilidad permanecen claros.
