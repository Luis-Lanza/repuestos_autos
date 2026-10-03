use printpdf::{
    matrix::TextMatrix,
    ops::{Op, PdfFontHandle},
    text::TextItem,
    BuiltinFont, Mm, PdfDocument, PdfPage, PdfSaveOptions, Pt,
};
use serde::{Deserialize, Serialize};

use crate::{
    application::inventory::movement_ledger::{
        MovementLedgerError, MovementLedgerPage, MovementLedgerProductOption,
        MovementLedgerProductOptionPage, MovementLedgerProductOptionsQuery, MovementLedgerQuery,
        MovementLedgerReader, MovementLedgerRow, MovementType, DEFAULT_PAGE_SIZE, MAX_EXPORT_ROWS,
        MAX_PAGE_SIZE, MAX_PRODUCT_OPTIONS_PAGE_SIZE,
    },
    commands::{
        pdf_format,
        pdf_table::{measure_helvetica_pt, wrap_helvetica},
    },
    infrastructure::sqlite::movement_ledger_repository::SqliteMovementLedgerReader,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MovementLedgerRequest {
    pub from_utc: String,
    pub to_exclusive_utc: String,
    #[serde(default)]
    pub product_id: Option<i64>,
    #[serde(default)]
    pub movement_type: Option<String>,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_page() -> u32 {
    1
}
fn default_page_size() -> u32 {
    DEFAULT_PAGE_SIZE
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MovementLedgerResponse {
    Success {
        rows: Vec<MovementLedgerRow>,
        page: u32,
        page_size: u32,
        has_more: bool,
    },
    Error(MovementLedgerCommandError),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MovementLedgerProductOptionsRequest {
    pub query: String,
    pub page: u32,
    #[serde(default = "default_product_options_page_size")]
    pub page_size: u32,
}

fn default_product_options_page_size() -> u32 {
    MAX_PRODUCT_OPTIONS_PAGE_SIZE
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MovementLedgerProductOptionsResponse {
    Success {
        products: Vec<MovementLedgerProductOption>,
        page: u32,
        page_size: u32,
        has_more: bool,
    },
    Error(MovementLedgerCommandError),
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct MovementLedgerCommandError {
    pub code: &'static str,
    pub message: &'static str,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MovementLedgerExportRequest {
    pub from_utc: String,
    pub to_exclusive_utc: String,
    #[serde(default)]
    pub product_id: Option<i64>,
    #[serde(default)]
    pub movement_type: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MovementLedgerExportResponse {
    Success,
    Cancelled,
    Error(MovementLedgerCommandError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportSaveResult {
    Saved,
    Cancelled,
    Failed,
}

pub fn export_movement_ledger(
    connection: &rusqlite::Connection,
    request: MovementLedgerExportRequest,
    generated_at: &str,
    save: impl FnOnce(&[u8]) -> ExportSaveResult,
) -> MovementLedgerExportResponse {
    let movement_type = match request.movement_type.as_deref() {
        Some(value) => match MovementType::parse(value) {
            Some(value) => Some(value),
            None => {
                return export_error("invalid_filter", "The movement ledger filter is invalid.")
            }
        },
        None => None,
    };
    let reader = SqliteMovementLedgerReader::new(connection);
    let mut rows = Vec::new();
    let page_count = MAX_EXPORT_ROWS.div_ceil(MAX_PAGE_SIZE as usize) as u32;
    for page in 1..=page_count {
        let query = match MovementLedgerQuery::new(
            &request.from_utc,
            &request.to_exclusive_utc,
            request.product_id,
            movement_type,
            page,
            MAX_PAGE_SIZE,
        ) {
            Ok(query) => query,
            Err(reason) => return export_map_error(reason),
        };
        let result = match reader.list(&query) {
            Ok(result) => result,
            Err(reason) => return export_map_error(reason),
        };
        rows.extend(result.rows);
        if !result.has_more {
            break;
        }
        if rows.len() >= MAX_EXPORT_ROWS {
            return export_error(
                "export_limit_exceeded",
                "More than 2,000 movements match. Narrow the filters and try again.",
            );
        }
    }
    let pdf = match render_movement_ledger_pdf(&rows, generated_at, &request) {
        Ok(pdf) => pdf,
        Err(()) => {
            return export_error(
                "export_failed",
                "The movement ledger could not be exported.",
            )
        }
    };
    match save(&pdf) {
        ExportSaveResult::Saved => MovementLedgerExportResponse::Success,
        ExportSaveResult::Cancelled => MovementLedgerExportResponse::Cancelled,
        ExportSaveResult::Failed => {
            export_error("export_failed", "The movement ledger could not be saved.")
        }
    }
}

const PDF_PAGE_W_MM: f32 = 297.0;
const PDF_PAGE_H_MM: f32 = 210.0;
const PDF_ROW_START_MM: f32 = 132.0;
const PDF_BOTTOM_MM: f32 = 17.0;
const PDF_LINE_HEIGHT_MM: f32 = 4.5;
const PDF_ROW_GAP_MM: f32 = 4.0;
const PDF_BODY_PT: f32 = 9.0;
const PDF_COLUMNS: [(f32, f32, &str, bool); 6] = [
    (8.0, 29.0, "N.º / fecha y hora", false),
    (37.0, 50.0, "Producto / SKU", false),
    (87.0, 35.0, "Tipo", false),
    (122.0, 19.0, "Variación", true),
    (141.0, 23.0, "Existencia resultante", true),
    (164.0, 125.0, "Detalle registrado", false),
];

#[derive(Debug)]
struct PdfCell {
    font_size_pt: f32,
    lines: Vec<String>,
}

#[derive(Debug)]
struct PdfRow {
    cells: Vec<PdfCell>,
    line_count: usize,
    movement_id: i64,
}

#[derive(Debug)]
struct PdfRowFragment {
    row_index: usize,
    first_line: usize,
    line_count: usize,
    top_mm: f32,
    continued: bool,
}

fn render_movement_ledger_pdf(
    rows: &[MovementLedgerRow],
    generated_at: &str,
    filters: &MovementLedgerExportRequest,
) -> Result<Vec<u8>, ()> {
    let layout_rows = rows.iter().map(layout_movement_row).collect::<Vec<_>>();
    let mut pages: Vec<Vec<PdfRowFragment>> = vec![Vec::new()];
    let mut y = PDF_ROW_START_MM;
    for (row_index, row) in layout_rows.iter().enumerate() {
        let mut first_line = 0;
        while first_line < row.line_count {
            let available = ((y - PDF_BOTTOM_MM + 0.25) / PDF_LINE_HEIGHT_MM)
                .floor()
                .max(0.0) as usize;
            if available == 0 {
                if pages.len() >= 1_000 {
                    return Err(());
                }
                pages.push(Vec::new());
                y = PDF_ROW_START_MM;
                continue;
            }
            let line_count = (row.line_count - first_line).min(available);
            pages.last_mut().ok_or(())?.push(PdfRowFragment {
                row_index,
                first_line,
                line_count,
                top_mm: y,
                continued: first_line > 0,
            });
            first_line += line_count;
            y -= line_count as f32 * PDF_LINE_HEIGHT_MM;
            if first_line < row.line_count {
                if pages.len() >= 1_000 {
                    return Err(());
                }
                pages.push(Vec::new());
                y = PDF_ROW_START_MM;
            } else {
                y -= PDF_ROW_GAP_MM;
            }
        }
    }
    let generated = local_timestamp(generated_at);
    let from = local_timestamp(&filters.from_utc);
    let to = local_timestamp(&filters.to_exclusive_utc);
    let product_filter = filters
        .product_id
        .map_or_else(|| "Todos".to_string(), |id| id.to_string());
    let movement_filter = filters
        .movement_type
        .as_deref()
        .map(movement_label)
        .unwrap_or("Todos");
    let mut document_pages = Vec::with_capacity(pages.len());
    for (page_number, page_rows) in pages.iter().enumerate() {
        let mut ops = Vec::new();
        push_pdf_text(&mut ops, "Repuestos Autos", 15.0, 8.0, 202.0);
        push_pdf_text(
            &mut ops,
            "Libro de movimientos de inventario",
            12.0,
            8.0,
            194.0,
        );
        push_pdf_text(
            &mut ops,
            &format!("Período: desde {from} (incluido) hasta {to} (exclusivo)"),
            8.0,
            8.0,
            185.0,
        );
        push_pdf_text(
            &mut ops,
            &format!("Producto ID: {product_filter} | Tipo de movimiento: {movement_filter}"),
            8.0,
            8.0,
            178.0,
        );
        push_pdf_text(&mut ops, &format!("Generado: {generated}"), 8.0, 8.0, 171.0);
        header_background(&mut ops);
        draw_rule(&mut ops, 8.0, 153.0, 281.0);
        let header_lines = PDF_COLUMNS
            .iter()
            .map(|(_, width, label, _)| wrap_helvetica(label, (width - 2.0) * 72.0 / 25.4, 8.0))
            .collect::<Vec<_>>();
        let header_line_count = header_lines.iter().map(Vec::len).max().unwrap_or(1);
        for ((x, _, _, _), lines) in PDF_COLUMNS.iter().zip(&header_lines) {
            let first_y = 146.5 + (header_line_count - lines.len()) as f32 * 2.0;
            for (line_index, line) in lines.iter().enumerate() {
                push_pdf_text(
                    &mut ops,
                    line,
                    8.0,
                    x + 1.0,
                    first_y - line_index as f32 * 4.0,
                );
            }
        }
        draw_rule(&mut ops, 8.0, 140.0, 281.0);
        for fragment in page_rows {
            let row = &layout_rows[fragment.row_index];
            if fragment.continued {
                push_pdf_text(
                    &mut ops,
                    &format!("Continuación del movimiento n.º {}", row.movement_id),
                    7.0,
                    8.0,
                    fragment.top_mm + 3.5,
                );
            }
            for (index, cell) in row.cells.iter().enumerate() {
                for line_index in fragment.first_line..fragment.first_line + fragment.line_count {
                    if let Some(line) = cell.lines.get(line_index) {
                        let (x, width, _, right) = PDF_COLUMNS[index];
                        let x = if right {
                            x + width
                                - 1.0
                                - measure_helvetica_pt(line, cell.font_size_pt) * 25.4 / 72.0
                        } else {
                            x + 1.0
                        };
                        push_pdf_text(
                            &mut ops,
                            line,
                            cell.font_size_pt,
                            x,
                            fragment.top_mm
                                - (line_index - fragment.first_line) as f32 * PDF_LINE_HEIGHT_MM,
                        );
                    }
                }
            }
            draw_rule(
                &mut ops,
                8.0,
                fragment.top_mm - fragment.line_count as f32 * PDF_LINE_HEIGHT_MM + 0.25,
                281.0,
            );
        }
        let table_bottom = page_rows.last().map_or(140.0, |fragment| {
            fragment.top_mm - fragment.line_count as f32 * PDF_LINE_HEIGHT_MM + 0.25
        });
        let left_edge = PDF_COLUMNS[0].0;
        let (last_column_x, last_column_width, _, _) = PDF_COLUMNS[PDF_COLUMNS.len() - 1];
        draw_vertical(&mut ops, left_edge, 153.0, table_bottom);
        draw_vertical(
            &mut ops,
            last_column_x + last_column_width,
            153.0,
            table_bottom,
        );
        for (x, _, _, _) in PDF_COLUMNS.iter().skip(1) {
            draw_vertical(&mut ops, *x, 153.0, table_bottom);
        }
        push_pdf_text(
            &mut ops,
            &format!("Página {} de {}", page_number + 1, pages.len()),
            8.0,
            265.0,
            8.0,
        );
        document_pages.push(PdfPage::new(Mm(PDF_PAGE_W_MM), Mm(PDF_PAGE_H_MM), ops));
    }
    let mut document = PdfDocument::new("Libro de movimientos de inventario");
    document.pages = document_pages;
    let bytes = document.save(&PdfSaveOptions::default(), &mut Vec::new());
    if bytes.len() > 16 * 1024 * 1024 {
        return Err(());
    }
    Ok(bytes)
}

fn local_timestamp(value: &str) -> String {
    let normalized = if value.contains('T') {
        value.to_owned()
    } else {
        format!("{}Z", value.replace(' ', "T"))
    };
    pdf_format::format_timestamp_local(&normalized)
        .unwrap_or_else(|_| "Hora local no disponible".to_owned())
}

fn movement_label(value: &str) -> &'static str {
    match value {
        "opening_stock" => "Stock inicial",
        "stock_entry" => "Ingreso de stock",
        "sale" => "Venta",
        "return" => "Devolución",
        "adjustment" => "Ajuste de inventario",
        "cancellation" => "Anulación",
        _ => "Tipo no disponible",
    }
}

fn layout_movement_row(row: &MovementLedgerRow) -> PdfRow {
    let occurred = local_timestamp(&row.occurred_at);
    let detail = match (row.reason.as_deref(), row.note.as_deref()) {
        (Some(reason), Some(note)) if reason != note => format!("Motivo: {reason}\nNota: {note}"),
        (Some(reason), _) => format!("Motivo: {reason}"),
        (_, Some(note)) => format!("Nota: {note}"),
        _ => String::new(),
    };
    let values = [
        format!("#{}\n{}", row.movement_id, occurred),
        format!("{}\nSKU: {}", row.product_name, row.product_sku),
        movement_label(&row.movement_type).to_owned(),
        format!("{:+}", row.quantity_delta),
        row.resulting_quantity
            .map_or_else(|| "No registrado".to_string(), |value| value.to_string()),
        detail,
    ];
    let cells = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let (_, width, _, _) = PDF_COLUMNS[index];
            let inset = if index == 5 { 2.0 } else { 1.5 };
            let max_width = (width - inset * 2.0) * 72.0 / 25.4;
            PdfCell {
                font_size_pt: PDF_BODY_PT,
                lines: wrap_helvetica(value, max_width, PDF_BODY_PT),
            }
        })
        .collect::<Vec<_>>();
    let line_count = cells
        .iter()
        .map(|cell| cell.lines.len())
        .max()
        .unwrap_or(1)
        .max(1);
    PdfRow {
        cells,
        line_count,
        movement_id: row.movement_id,
    }
}

fn push_pdf_text(ops: &mut Vec<Op>, text: &str, size: f32, x_mm: f32, y_mm: f32) {
    ops.push(Op::SetFillColor {
        col: printpdf::Color::Rgb(printpdf::color::Rgb::new(0.0, 0.0, 0.0, None)),
    });
    ops.extend([
        Op::StartTextSection,
        Op::SetFont {
            font: PdfFontHandle::Builtin(BuiltinFont::Helvetica),
            size: Pt(size),
        },
        Op::SetTextMatrix {
            matrix: TextMatrix::Raw([
                1.0,
                0.0,
                0.0,
                1.0,
                Pt::from(Mm(x_mm)).0,
                Pt::from(Mm(y_mm)).0,
            ]),
        },
        Op::ShowText {
            items: vec![TextItem::Text(text.to_string())],
        },
        Op::EndTextSection,
    ]);
}

fn header_background(ops: &mut Vec<Op>) {
    use printpdf::{
        color::Rgb,
        graphics::{PaintMode, Rect},
        Color,
    };
    ops.push(Op::SetFillColor {
        col: Color::Rgb(Rgb::new(0.90, 0.93, 0.97, None)),
    });
    ops.push(Op::DrawRectangle {
        rectangle: Rect {
            x: Pt::from(Mm(8.0)),
            y: Pt::from(Mm(140.0)),
            width: Pt::from(Mm(281.0)),
            height: Pt::from(Mm(13.0)),
            mode: Some(PaintMode::Fill),
            winding_order: None,
        },
    });
}

fn draw_vertical(ops: &mut Vec<Op>, x: f32, top: f32, bottom: f32) {
    use printpdf::graphics::{Line, LinePoint, Point};
    ops.push(Op::DrawLine {
        line: Line {
            points: vec![
                LinePoint {
                    p: Point {
                        x: Pt::from(Mm(x)),
                        y: Pt::from(Mm(top)),
                    },
                    bezier: false,
                },
                LinePoint {
                    p: Point {
                        x: Pt::from(Mm(x)),
                        y: Pt::from(Mm(bottom)),
                    },
                    bezier: false,
                },
            ],
            is_closed: false,
        },
    });
}

fn draw_rule(ops: &mut Vec<Op>, x: f32, y: f32, width: f32) {
    use printpdf::graphics::{Line, LinePoint, Point};
    ops.push(Op::DrawLine {
        line: Line {
            points: vec![
                LinePoint {
                    p: Point {
                        x: Pt::from(Mm(x)),
                        y: Pt::from(Mm(y)),
                    },
                    bezier: false,
                },
                LinePoint {
                    p: Point {
                        x: Pt::from(Mm(x + width)),
                        y: Pt::from(Mm(y)),
                    },
                    bezier: false,
                },
            ],
            is_closed: false,
        },
    });
}

fn export_error(code: &'static str, message: &'static str) -> MovementLedgerExportResponse {
    MovementLedgerExportResponse::Error(MovementLedgerCommandError { code, message })
}

fn export_map_error(reason: MovementLedgerError) -> MovementLedgerExportResponse {
    match reason {
        MovementLedgerError::InvalidRange => export_error(
            "invalid_range",
            "The movement ledger date range is invalid.",
        ),
        MovementLedgerError::InvalidFilter => {
            export_error("invalid_filter", "The movement ledger filter is invalid.")
        }
        MovementLedgerError::InvalidPage => {
            export_error("invalid_page", "The movement ledger page is invalid.")
        }
        MovementLedgerError::PersistedDataInvalid | MovementLedgerError::Persistence => {
            export_error(
                "persistence_failure",
                "The movement ledger could not be loaded.",
            )
        }
    }
}

pub fn list_movement_ledger_product_options(
    connection: &rusqlite::Connection,
    request: MovementLedgerProductOptionsRequest,
) -> MovementLedgerProductOptionsResponse {
    let query = match MovementLedgerProductOptionsQuery::new(
        &request.query,
        request.page,
        request.page_size,
    ) {
        Ok(query) => query,
        Err(MovementLedgerError::InvalidFilter) => {
            return product_options_error("invalid_filter", "The product search query is invalid.")
        }
        Err(reason) => return product_options_map_error(reason),
    };
    match SqliteMovementLedgerReader::new(connection).product_options(&query) {
        Ok(MovementLedgerProductOptionPage {
            products,
            page,
            page_size,
            has_more,
        }) => MovementLedgerProductOptionsResponse::Success {
            products,
            page,
            page_size,
            has_more,
        },
        Err(reason) => product_options_map_error(reason),
    }
}

fn product_options_error(
    code: &'static str,
    message: &'static str,
) -> MovementLedgerProductOptionsResponse {
    MovementLedgerProductOptionsResponse::Error(MovementLedgerCommandError { code, message })
}

fn product_options_map_error(reason: MovementLedgerError) -> MovementLedgerProductOptionsResponse {
    match reason {
        MovementLedgerError::InvalidFilter => {
            product_options_error("invalid_filter", "The product search query is invalid.")
        }
        MovementLedgerError::InvalidPage => {
            product_options_error("invalid_page", "The product search page is invalid.")
        }
        MovementLedgerError::InvalidRange
        | MovementLedgerError::PersistedDataInvalid
        | MovementLedgerError::Persistence => product_options_error(
            "persistence_failure",
            "The movement ledger products could not be loaded.",
        ),
    }
}

pub fn list_movement_ledger(
    connection: &rusqlite::Connection,
    request: MovementLedgerRequest,
) -> MovementLedgerResponse {
    let movement_type = match request.movement_type {
        Some(value) => match MovementType::parse(&value) {
            Some(value) => Some(value),
            None => return error("invalid_filter", "The movement ledger filter is invalid."),
        },
        None => None,
    };
    let query = match MovementLedgerQuery::new(
        &request.from_utc,
        &request.to_exclusive_utc,
        request.product_id,
        movement_type,
        request.page,
        request.page_size,
    ) {
        Ok(query) => query,
        Err(reason) => return map_error(reason),
    };
    let reader = SqliteMovementLedgerReader::new(connection);
    match reader.list(&query) {
        Ok(MovementLedgerPage {
            rows,
            page,
            page_size,
            has_more,
        }) => MovementLedgerResponse::Success {
            rows,
            page,
            page_size,
            has_more,
        },
        Err(reason) => map_error(reason),
    }
}

fn error(code: &'static str, message: &'static str) -> MovementLedgerResponse {
    MovementLedgerResponse::Error(MovementLedgerCommandError { code, message })
}

fn map_error(reason: MovementLedgerError) -> MovementLedgerResponse {
    match reason {
        MovementLedgerError::InvalidRange => error(
            "invalid_range",
            "The movement ledger date range is invalid.",
        ),
        MovementLedgerError::InvalidFilter => {
            error("invalid_filter", "The movement ledger filter is invalid.")
        }
        MovementLedgerError::InvalidPage => {
            error("invalid_page", "The movement ledger page is invalid.")
        }
        MovementLedgerError::PersistedDataInvalid | MovementLedgerError::Persistence => error(
            "persistence_failure",
            "The movement ledger could not be loaded.",
        ),
    }
}
