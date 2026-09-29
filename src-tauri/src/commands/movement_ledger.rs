use printpdf::{
    matrix::TextMatrix,
    ops::{Op, PdfFontHandle},
    text::TextItem,
    BuiltinFont, Mm, PdfDocument, PdfPage, PdfSaveOptions, Pt,
};
use serde::{Deserialize, Serialize};

use crate::{
    application::inventory::movement_ledger::{
        MovementLedgerError, MovementLedgerPage, MovementLedgerQuery, MovementLedgerReader,
        MovementLedgerProductOption, MovementLedgerRow, MovementType, DEFAULT_PAGE_SIZE, MAX_EXPORT_ROWS, MAX_PAGE_SIZE,
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

fn default_page() -> u32 { 1 }
fn default_page_size() -> u32 { DEFAULT_PAGE_SIZE }

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

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MovementLedgerProductOptionsResponse {
    Success { products: Vec<MovementLedgerProductOption> },
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
            None => return export_error("invalid_filter", "The movement ledger filter is invalid."),
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
        Err(()) => return export_error("export_failed", "The movement ledger could not be exported."),
    };
    match save(&pdf) {
        ExportSaveResult::Saved => MovementLedgerExportResponse::Success,
        ExportSaveResult::Cancelled => MovementLedgerExportResponse::Cancelled,
        ExportSaveResult::Failed => export_error("export_failed", "The movement ledger could not be saved."),
    }
}

const PDF_LEFT_MM: f32 = 15.0;
const PDF_RIGHT_MM: f32 = 195.0;
const PDF_ROW_START_MM: f32 = 202.0;
const PDF_BOTTOM_MM: f32 = 22.0;
const PDF_LINE_HEIGHT_MM: f32 = 4.0;
const PDF_ROW_GAP_MM: f32 = 4.0;

#[derive(Debug)]
struct PdfCell {
    x_mm: f32,
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
            let available_lines = ((y - PDF_BOTTOM_MM) / PDF_LINE_HEIGHT_MM).floor().max(0.0) as usize;
            if available_lines == 0 {
                pages.push(Vec::new());
                y = PDF_ROW_START_MM;
                continue;
            }
            let line_count = (row.line_count - first_line).min(available_lines);
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
                pages.push(Vec::new());
                y = PDF_ROW_START_MM;
            } else {
                y -= PDF_ROW_GAP_MM;
            }
        }
    }

    let page_count = pages.len();
    let mut document_pages = Vec::with_capacity(page_count);
    let product_filter = filters.product_id.map_or_else(|| "All".to_string(), |id| id.to_string());
    let movement_filter = filters.movement_type.as_deref().unwrap_or("All");
    for (page_number, page_rows) in pages.iter().enumerate() {
        let mut ops = Vec::new();
        push_pdf_text(&mut ops, "Repuestos Autos", 18.0, PDF_LEFT_MM, 282.0);
        push_pdf_text(&mut ops, "Movement Ledger", 13.0, PDF_LEFT_MM, 270.0);
        push_pdf_text(&mut ops, &format!("Period: {} to (exclusive) {}", filters.from_utc, filters.to_exclusive_utc), 8.0, PDF_LEFT_MM, 256.0);
        push_pdf_text(&mut ops, &format!("Product ID: {product_filter} | Movement type: {movement_filter}"), 8.0, PDF_LEFT_MM, 246.0);
        push_pdf_text(&mut ops, &format!("Generated: {generated_at}"), 8.0, PDF_LEFT_MM, 236.0);
        push_pdf_text(&mut ops, &format!("Page {} of {page_count}", page_number + 1), 8.0, 160.0, 15.0);

        let columns = [
            (PDF_LEFT_MM, 30.0, "ID / date / time"),
            (45.0, 48.0, "Current product / SKU"),
            (93.0, 22.0, "Type"),
            (115.0, 14.0, "Delta"),
            (129.0, 22.0, "Result"),
            (151.0, PDF_RIGHT_MM - 151.0, "Stored detail"),
        ];
        let header_lines = columns.iter().map(|(_, width, label)| wrap_text(label, chars_for_width(*width, 6.5))).collect::<Vec<_>>();
        for (column_index, (x_mm, _, _)) in columns.iter().enumerate() {
            for (line_index, line) in header_lines[column_index].iter().enumerate() {
                push_pdf_text(&mut ops, line, 6.5, *x_mm, 220.0 - line_index as f32 * PDF_LINE_HEIGHT_MM);
            }
        }
        for fragment in page_rows {
            let row = &layout_rows[fragment.row_index];
            if fragment.continued {
                push_pdf_text(
                    &mut ops,
                    &format!("Continuation of movement #{}", row.movement_id),
                    7.0,
                    PDF_LEFT_MM,
                    210.0,
                );
            }
            for cell in &row.cells {
                for line_index in fragment.first_line..fragment.first_line + fragment.line_count {
                    if let Some(line) = cell.lines.get(line_index) {
                        push_pdf_text(
                            &mut ops,
                            line,
                            cell.font_size_pt,
                            cell.x_mm,
                            fragment.top_mm - (line_index - fragment.first_line) as f32 * PDF_LINE_HEIGHT_MM,
                        );
                    }
                }
            }
        }
        document_pages.push(PdfPage::new(Mm(210.0), Mm(297.0), ops));
    }

    let mut document = PdfDocument::new("Movement Ledger");
    document.pages = document_pages;
    Ok(document.save(&PdfSaveOptions::default(), &mut Vec::new()))
}

fn layout_movement_row(row: &MovementLedgerRow) -> PdfRow {
    let detail = row.reason.as_deref().or(row.note.as_deref()).unwrap_or("");
    let values = [
        (PDF_LEFT_MM, 30.0, 6.5, format!("#{} / {}", row.movement_id, row.occurred_at)),
        (45.0, 48.0, 6.5, format!("{} / {}", row.product_name, row.product_sku)),
        (93.0, 22.0, 6.5, row.movement_type.clone()),
        (115.0, 14.0, 6.5, format!("{:+}", row.quantity_delta)),
        (129.0, 22.0, 5.5, row.resulting_quantity.map_or_else(|| "No registrado".to_string(), |value| value.to_string())),
        (151.0, PDF_RIGHT_MM - 151.0, 6.5, detail.to_string()),
    ];
    let cells = values.into_iter().map(|(x_mm, width_mm, font_size_pt, value)| PdfCell {
        x_mm,
        font_size_pt,
        lines: wrap_text(&value, chars_for_width(width_mm, font_size_pt)),
    }).collect::<Vec<_>>();
    let line_count = cells.iter().map(|cell| cell.lines.len()).max().unwrap_or(1).max(1);
    PdfRow {
        cells,
        line_count,
        movement_id: row.movement_id,
    }
}

fn chars_for_width(width_mm: f32, font_size_pt: f32) -> usize {
    // One font-size unit per character is a conservative width bound for the built-in Helvetica face.
    ((width_mm * 72.0 / 25.4) / font_size_pt).floor().max(1.0) as usize
}

fn wrap_text(value: &str, max_chars: usize) -> Vec<String> {
    let max_chars = max_chars.max(1);
    let characters = value.chars().collect::<Vec<_>>();
    if characters.is_empty() {
        return vec![String::new()];
    }
    let mut lines = Vec::new();
    let mut first = 0;
    while first < characters.len() {
        let mut end = (first + max_chars).min(characters.len());
        if end < characters.len() {
            if let Some(last_space) = characters[first..end].iter().rposition(|character| character.is_whitespace()) {
                if last_space > 0 {
                    end = first + last_space + 1;
                }
            }
        }
        lines.push(characters[first..end].iter().collect());
        first = end;
    }
    lines
}

fn push_pdf_text(ops: &mut Vec<Op>, text: &str, size: f32, x_mm: f32, y_mm: f32) {
    ops.extend([
        Op::StartTextSection,
        Op::SetFont { font: PdfFontHandle::Builtin(BuiltinFont::Helvetica), size: Pt(size) },
        Op::SetTextMatrix { matrix: TextMatrix::Raw([1.0, 0.0, 0.0, 1.0, Pt::from(Mm(x_mm)).0, Pt::from(Mm(y_mm)).0]) },
        Op::ShowText { items: vec![TextItem::Text(text.to_string())] },
        Op::EndTextSection,
    ]);
}

fn export_error(code: &'static str, message: &'static str) -> MovementLedgerExportResponse {
    MovementLedgerExportResponse::Error(MovementLedgerCommandError { code, message })
}

fn export_map_error(reason: MovementLedgerError) -> MovementLedgerExportResponse {
    match reason {
        MovementLedgerError::InvalidRange => export_error("invalid_range", "The movement ledger date range is invalid."),
        MovementLedgerError::InvalidFilter => export_error("invalid_filter", "The movement ledger filter is invalid."),
        MovementLedgerError::InvalidPage => export_error("invalid_page", "The movement ledger page is invalid."),
        MovementLedgerError::PersistedDataInvalid | MovementLedgerError::Persistence => export_error("persistence_failure", "The movement ledger could not be loaded."),
    }
}

pub fn list_movement_ledger_product_options(
    connection: &rusqlite::Connection,
) -> MovementLedgerProductOptionsResponse {
    match SqliteMovementLedgerReader::new(connection).product_options() {
        Ok(products) => MovementLedgerProductOptionsResponse::Success { products },
        Err(_) => MovementLedgerProductOptionsResponse::Error(MovementLedgerCommandError {
            code: "persistence_failure",
            message: "The movement ledger products could not be loaded.",
        }),
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
        Ok(MovementLedgerPage { rows, page, page_size, has_more }) => {
            MovementLedgerResponse::Success { rows, page, page_size, has_more }
        }
        Err(reason) => map_error(reason),
    }
}

fn error(code: &'static str, message: &'static str) -> MovementLedgerResponse {
    MovementLedgerResponse::Error(MovementLedgerCommandError { code, message })
}

fn map_error(reason: MovementLedgerError) -> MovementLedgerResponse {
    match reason {
        MovementLedgerError::InvalidRange => error("invalid_range", "The movement ledger date range is invalid."),
        MovementLedgerError::InvalidFilter => error("invalid_filter", "The movement ledger filter is invalid."),
        MovementLedgerError::InvalidPage => error("invalid_page", "The movement ledger page is invalid."),
        MovementLedgerError::PersistedDataInvalid | MovementLedgerError::Persistence => error(
            "persistence_failure",
            "The movement ledger could not be loaded.",
        ),
    }
}
