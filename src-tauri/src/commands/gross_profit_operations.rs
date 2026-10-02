use printpdf::{matrix::TextMatrix, ops::{Op, PdfFontHandle}, text::TextItem, BuiltinFont, Mm, PdfDocument, PdfPage, PdfSaveOptions, Pt};
use serde::{Deserialize, Serialize};

use crate::{
    application::reporting::{
        load_gross_profit_operations, DashboardRange, GrossProfitOperationsPage,
        GrossProfitOperationsPagination, GrossProfitOperationsReader, GrossProfitOperationRow,
        GrossProfitOperationsExportReadError, ReportingError,
    },
    infrastructure::sqlite::dashboard_repository::SqliteDashboardReader,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrossProfitOperationsRequest {
    pub from_utc: String,
    pub to_exclusive_utc: String,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrossProfitOperationsExportRequest {
    pub from: GrossProfitExportDateBound,
    pub to_exclusive: GrossProfitExportDateBound,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrossProfitExportDateBound {
    pub local_date: String,
    pub utc: String,
    pub utc_offset_minutes: i32,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GrossProfitOperationsResponse {
    Success { report: GrossProfitOperationsPage },
    Error(GrossProfitOperationsError),
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct GrossProfitOperationsError {
    pub code: &'static str,
    pub message: &'static str,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GrossProfitOperationsExportResponse {
    Success,
    Cancelled,
    Error(GrossProfitOperationsError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportSaveResult { Saved, Cancelled, Failed }

pub fn export_gross_profit_operations(
    connection: &rusqlite::Connection,
    request: GrossProfitOperationsExportRequest,
    generated_at: &str,
    save: impl FnOnce(&[u8]) -> ExportSaveResult,
) -> GrossProfitOperationsExportResponse {
    if !valid_export_bounds(&request.from, &request.to_exclusive) || !valid_timestamp(generated_at) {
        return export_error("invalid_request", "El período del informe no es válido.");
    }
    let range = match DashboardRange::parse_gross_profit(&request.from.utc, &request.to_exclusive.utc) {
        Ok(value) => value,
        Err(_) => return export_error("invalid_request", "El período del informe no es válido."),
    };
    let rows = match SqliteDashboardReader::new(connection).read_all_gross_profit_operations(&range) {
        Ok(rows) => rows,
        Err(GrossProfitOperationsExportReadError::LimitExceeded) => return export_limit_error(),
        Err(GrossProfitOperationsExportReadError::Reporting(_)) => return export_error("export_failed", "No se pudo generar el informe PDF."),
    };
    let pdf = match render_gross_profit_pdf(&rows, &request, generated_at) {
        Ok(pdf) if pdf.len() <= 16 * 1024 * 1024 => pdf,
        Ok(_) | Err(PdfRenderError::LimitExceeded) => return export_limit_error(),
        Err(PdfRenderError::InvalidData) => return export_error("export_failed", "No se pudo generar el informe PDF."),
    };
    match save(&pdf) {
        ExportSaveResult::Saved => GrossProfitOperationsExportResponse::Success,
        ExportSaveResult::Cancelled => GrossProfitOperationsExportResponse::Cancelled,
        ExportSaveResult::Failed => export_error("export_failed", "No se pudo guardar el informe PDF."),
    }
}

fn valid_local_date(value: &str) -> Option<time::Date> {
    (value.len() == 10).then(|| time::Date::parse(value, &time::macros::format_description!("[year]-[month]-[day]")).ok()).flatten()
}

fn valid_export_bounds(from: &GrossProfitExportDateBound, to: &GrossProfitExportDateBound) -> bool {
    fn matches(bound: &GrossProfitExportDateBound) -> bool {
        let Some(date) = valid_local_date(&bound.local_date) else { return false; };
        if !(-840..=840).contains(&bound.utc_offset_minutes) { return false; }
        let Ok(utc) = time::OffsetDateTime::parse(&bound.utc, &time::format_description::well_known::Rfc3339) else { return false; };
        if !bound.utc.ends_with(".000Z") || utc.offset() != time::UtcOffset::UTC || utc.nanosecond() != 0 { return false; }
        let expected = date.with_time(time::Time::MIDNIGHT).assume_utc() + time::Duration::minutes(i64::from(bound.utc_offset_minutes));
        expected == utc
    }
    matches(from) && matches(to) && from.local_date < to.local_date && from.utc < to.utc
}

fn valid_timestamp(value: &str) -> bool {
    time::OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339).is_ok()
}

fn export_error(code: &'static str, message: &'static str) -> GrossProfitOperationsExportResponse {
    GrossProfitOperationsExportResponse::Error(GrossProfitOperationsError { code, message })
}

fn export_limit_error() -> GrossProfitOperationsExportResponse {
    export_error("resource_limit", "El informe supera los límites de recursos y no se guardó.")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PdfRenderError { LimitExceeded, InvalidData }

fn render_gross_profit_pdf(
    rows: &[GrossProfitOperationRow],
    request: &GrossProfitOperationsExportRequest,
    generated_at: &str,
) -> Result<Vec<u8>, PdfRenderError> {
    const PAGE_W: f32 = 297.0;
    const PAGE_H: f32 = 210.0;
    const LEFT: f32 = 8.0;
    const BOTTOM: f32 = 14.0;
    const LINE: f32 = 4.0;
    const COLS: [(f32, f32, &str); 7] = [
        (8.0, 31.0, "Fecha y hora"), (39.0, 41.0, "Operación"),
        (80.0, 61.0, "Producto (nombre + SKU)"), (141.0, 20.0, "Cantidad"),
        (161.0, 35.0, "Precio final"), (196.0, 42.0, "Costo histórico"),
        (238.0, 51.0, "Ganancia bruta"),
    ];
    let mut total = 0_i64;
    let mut unknown = 0_i64;
    for row in rows {
        if let Some(profit) = row.signed_gross_profit_centavos {
            total = total.checked_add(profit).ok_or(PdfRenderError::InvalidData)?;
        } else {
            unknown = unknown.checked_add(1).ok_or(PdfRenderError::InvalidData)?;
        }
    }
    let total_pages_estimate = rows.len().checked_div(25).and_then(|n| n.checked_add(1)).ok_or(PdfRenderError::InvalidData)?;
    if total_pages_estimate > 1_000 { return Err(PdfRenderError::LimitExceeded); }
    let mut pages: Vec<Vec<(usize, Vec<String>)>> = vec![Vec::new()];
    let mut y = 152.0;
    for (index, row) in rows.iter().enumerate() {
        let operation = match row.return_id {
            Some(id) => format!("Devolución #{id} · Venta #{}", row.sale_id),
            None => format!("Venta #{}", row.sale_id),
        };
        let cost = row.unit_cost_snapshot_centavos.map_or_else(|| "No disponible".to_string(), money);
        let profit = row.signed_gross_profit_centavos.map_or_else(|| "No calculable".to_string(), money);
        let values = [row.occurred_at.clone(), operation, format!("{} (SKU {})", row.product_name, row.sku), format!("{:+}", row.signed_quantity), money(row.negotiated_unit_price_centavos), cost, profit];
        let lines = values.iter().enumerate().map(|(i, value)| wrap(value, chars(COLS[i].1, 6.5))).collect::<Vec<_>>();
        let height = lines.iter().map(Vec::len).max().unwrap_or(1).max(1);
        if height > 30 { return Err(PdfRenderError::LimitExceeded); }
        if y - height as f32 * LINE < BOTTOM {
            if pages.len() >= 1_000 { return Err(PdfRenderError::LimitExceeded); }
            pages.push(Vec::new()); y = 152.0;
        }
        pages.last_mut().ok_or(PdfRenderError::InvalidData)?.push((index, values.to_vec()));
        y -= height as f32 * LINE + 2.0;
    }
    let count = pages.len();
    let mut output = Vec::with_capacity(count);
    for (page_index, page_rows) in pages.iter().enumerate() {
        let mut ops = Vec::new();
        text(&mut ops, "Repuestos Autos", 15.0, LEFT, 199.0);
        text(&mut ops, "Informe de ganancia bruta por operaciones", 10.0, LEFT, 190.0);
        let to_local_date = time::Date::parse(&request.to_exclusive.local_date, &time::macros::format_description!("[year]-[month]-[day]")).map_err(|_| PdfRenderError::InvalidData)?.previous_day().ok_or(PdfRenderError::InvalidData)?;
        text(&mut ops, &format!("Período: {} al {} (inclusive)", request.from.local_date, to_local_date), 8.0, LEFT, 181.0);
        text(&mut ops, &format!("Generado: {generated_at}"), 8.0, LEFT, 174.0);
        text(&mut ops, &format!("Ganancia bruta total: {}", money(total)), 9.0, LEFT, 165.0);
        if unknown > 0 { text(&mut ops, &format!("Cálculo parcial: {unknown} operación(es) con costo histórico no disponible; ganancia no calculable."), 7.0, LEFT, 158.0); }
        for (x, _, label) in COLS { text(&mut ops, label, 6.5, x, 151.0); }
        let mut row_y = 145.0;
        for (_, values) in page_rows {
            let line_groups = values.iter().enumerate().map(|(i, value)| wrap(value, chars(COLS[i].1, 6.5))).collect::<Vec<_>>();
            let height = line_groups.iter().map(Vec::len).max().unwrap_or(1);
            for (column, lines) in line_groups.iter().enumerate() {
                for (line, value) in lines.iter().enumerate() {
                    text(&mut ops, value, 6.5, COLS[column].0, row_y - line as f32 * LINE);
                }
            }
            row_y -= height as f32 * LINE + 2.0;
        }
        text(&mut ops, &format!("Página {} de {count}", page_index + 1), 7.0, 255.0, 8.0);
        output.push(PdfPage::new(Mm(PAGE_W), Mm(PAGE_H), ops));
    }
    let mut doc = PdfDocument::new("Informe de ganancia bruta");
    doc.pages = output;
    Ok(doc.save(&PdfSaveOptions::default(), &mut Vec::new()))
}

fn money(centavos: i64) -> String {
    let absolute = centavos.unsigned_abs();
    format!("Bs {}{}.{:02}", if centavos < 0 { "-" } else { "" }, absolute / 100, absolute % 100)
}
fn chars(width: f32, size: f32) -> usize { ((width * 72.0 / 25.4) / size).floor().max(1.0) as usize }
fn wrap(value: &str, limit: usize) -> Vec<String> {
    let chars = value.chars().collect::<Vec<_>>();
    if chars.is_empty() { return vec![String::new()]; }
    chars.chunks(limit.max(1)).map(|chunk| chunk.iter().collect()).collect()
}
fn text(ops: &mut Vec<Op>, value: &str, size: f32, x: f32, y: f32) {
    ops.extend([Op::StartTextSection, Op::SetFont { font: PdfFontHandle::Builtin(BuiltinFont::Helvetica), size: Pt(size) }, Op::SetTextMatrix { matrix: TextMatrix::Raw([1.0, 0.0, 0.0, 1.0, Pt::from(Mm(x)).0, Pt::from(Mm(y)).0]) }, Op::ShowText { items: vec![TextItem::Text(value.to_string())] }, Op::EndTextSection]);
}

pub fn gross_profit_operations(
    connection: &rusqlite::Connection,
    request: GrossProfitOperationsRequest,
) -> GrossProfitOperationsResponse {
    let range = match DashboardRange::parse_gross_profit(&request.from_utc, &request.to_exclusive_utc) {
        Ok(range) => range,
        Err(_) => return invalid_request(),
    };
    let pagination = match GrossProfitOperationsPagination::validate(request.page, request.page_size) {
        Ok(pagination) => pagination,
        Err(_) => return invalid_request(),
    };
    match load_gross_profit_operations(&SqliteDashboardReader::new(connection), range, pagination) {
        Ok(report) => GrossProfitOperationsResponse::Success { report },
        Err(ReportingError::InvalidRange) => invalid_request(),
        Err(ReportingError::PersistedDataInvalid | ReportingError::Persistence) => {
            GrossProfitOperationsResponse::Error(GrossProfitOperationsError {
                code: "persistence_failure",
                message: "The gross-profit operations could not be loaded.",
            })
        }
    }
}

fn invalid_request() -> GrossProfitOperationsResponse {
    GrossProfitOperationsResponse::Error(GrossProfitOperationsError {
        code: "invalid_request",
        message: "The gross-profit operations request is invalid.",
    })
}
