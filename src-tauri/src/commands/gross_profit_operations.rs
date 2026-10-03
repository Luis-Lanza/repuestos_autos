use printpdf::{
    matrix::TextMatrix,
    ops::{Op, PdfFontHandle},
    text::TextItem,
    BuiltinFont, Mm, PdfDocument, PdfPage, PdfSaveOptions, Pt,
};
use serde::{Deserialize, Serialize};

use crate::{
    application::reporting::{
        load_gross_profit_operations, DashboardRange, GrossProfitOperationRow,
        GrossProfitOperationsExportReadError, GrossProfitOperationsPage,
        GrossProfitOperationsPagination, GrossProfitOperationsReader, ReportingError,
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
pub enum ExportSaveResult {
    Saved,
    Cancelled,
    Failed,
}

pub fn export_gross_profit_operations(
    connection: &rusqlite::Connection,
    request: GrossProfitOperationsExportRequest,
    generated_at: &str,
    save: impl FnOnce(&[u8]) -> ExportSaveResult,
) -> GrossProfitOperationsExportResponse {
    if !valid_export_bounds(&request.from, &request.to_exclusive) || !valid_timestamp(generated_at)
    {
        return export_error("invalid_request", "El período del informe no es válido.");
    }
    let range =
        match DashboardRange::parse_gross_profit(&request.from.utc, &request.to_exclusive.utc) {
            Ok(value) => value,
            Err(_) => {
                return export_error("invalid_request", "El período del informe no es válido.")
            }
        };
    let rows = match SqliteDashboardReader::new(connection).read_all_gross_profit_operations(&range)
    {
        Ok(rows) => rows,
        Err(GrossProfitOperationsExportReadError::LimitExceeded) => return export_limit_error(),
        Err(GrossProfitOperationsExportReadError::Reporting(_)) => {
            return export_error("export_failed", "No se pudo generar el informe PDF.")
        }
    };
    let pdf = match render_gross_profit_pdf(&rows, &request, generated_at) {
        Ok(pdf) if pdf.len() <= 16 * 1024 * 1024 => pdf,
        Ok(_) | Err(PdfRenderError::LimitExceeded) => return export_limit_error(),
        Err(PdfRenderError::InvalidData) => {
            return export_error("export_failed", "No se pudo generar el informe PDF.")
        }
    };
    match save(&pdf) {
        ExportSaveResult::Saved => GrossProfitOperationsExportResponse::Success,
        ExportSaveResult::Cancelled => GrossProfitOperationsExportResponse::Cancelled,
        ExportSaveResult::Failed => {
            export_error("export_failed", "No se pudo guardar el informe PDF.")
        }
    }
}

fn valid_local_date(value: &str) -> Option<time::Date> {
    (value.len() == 10)
        .then(|| {
            time::Date::parse(
                value,
                &time::macros::format_description!("[year]-[month]-[day]"),
            )
            .ok()
        })
        .flatten()
}

fn valid_export_bounds(from: &GrossProfitExportDateBound, to: &GrossProfitExportDateBound) -> bool {
    fn matches(bound: &GrossProfitExportDateBound) -> bool {
        let Some(date) = valid_local_date(&bound.local_date) else {
            return false;
        };
        if !(-840..=840).contains(&bound.utc_offset_minutes) {
            return false;
        }
        let Ok(utc) =
            time::OffsetDateTime::parse(&bound.utc, &time::format_description::well_known::Rfc3339)
        else {
            return false;
        };
        if !bound.utc.ends_with(".000Z")
            || utc.offset() != time::UtcOffset::UTC
            || utc.nanosecond() != 0
        {
            return false;
        }
        let expected = date.with_time(time::Time::MIDNIGHT).assume_utc()
            + time::Duration::minutes(i64::from(bound.utc_offset_minutes));
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
    export_error(
        "resource_limit",
        "El informe supera los límites de recursos y no se guardó.",
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PdfRenderError {
    LimitExceeded,
    InvalidData,
}

fn render_gross_profit_pdf(
    rows: &[GrossProfitOperationRow],
    request: &GrossProfitOperationsExportRequest,
    generated_at: &str,
) -> Result<Vec<u8>, PdfRenderError> {
    use crate::commands::{
        pdf_format,
        pdf_table::{measure_helvetica_pt, wrap_helvetica},
    };
    const PAGE_W: f32 = 297.0;
    const PAGE_H: f32 = 210.0;
    const BODY: f32 = 9.0;
    const LINE_MM: f32 = 4.5;
    const BOTTOM: f32 = 17.0;
    const COLS: [(f32, f32, &str, bool); 7] = [
        (8.0, 35.0, "Fecha y hora", false),
        (43.0, 48.0, "Operación", false),
        (91.0, 76.0, "Producto (nombre + SKU)", false),
        (167.0, 19.0, "Cantidad", true),
        (186.0, 35.0, "Precio final", true),
        (221.0, 36.0, "Costo histórico", true),
        (257.0, 32.0, "Ganancia bruta", true),
    ];
    let mut total = 0_i64;
    let mut unknown = 0_i64;
    for row in rows {
        if let Some(profit) = row.signed_gross_profit_centavos {
            total = total
                .checked_add(profit)
                .ok_or(PdfRenderError::InvalidData)?;
        } else {
            unknown = unknown.checked_add(1).ok_or(PdfRenderError::InvalidData)?;
        }
    }
    let inclusive_end = time::Date::parse(
        &request.to_exclusive.local_date,
        &time::macros::format_description!("[year]-[month]-[day]"),
    )
    .map_err(|_| PdfRenderError::InvalidData)?
    .previous_day()
    .ok_or(PdfRenderError::InvalidData)?;
    let start = pdf_format::format_date(&request.from.local_date)
        .unwrap_or_else(|_| "Fecha no disponible".into());
    let end = pdf_format::format_date(&inclusive_end.to_string())
        .unwrap_or_else(|_| "Fecha no disponible".into());
    let generated = pdf_format::format_timestamp_local(generated_at)
        .unwrap_or_else(|_| "Hora local no disponible".into());
    let mut pages: Vec<Vec<(Vec<Vec<String>>, usize)>> = vec![Vec::new()];
    let mut y = 141.0;
    for row in rows {
        let operation = match row.return_id {
            Some(id) => format!("Devolución n.º {id} · Venta n.º {}", row.sale_id),
            None => format!("Venta n.º {}", row.sale_id),
        };
        let occurred = pdf_format::format_timestamp_local(&row.occurred_at)
            .unwrap_or_else(|_| "Hora local no disponible".into());
        let cost = row
            .unit_cost_snapshot_centavos
            .map_or_else(|| "No disponible".to_string(), money);
        let profit = row
            .signed_gross_profit_centavos
            .map_or_else(|| "No calculable".to_string(), money);
        let values = [
            occurred,
            operation,
            format!("{} (SKU {})", row.product_name, row.sku),
            format!("{:+}", row.signed_quantity),
            money(row.negotiated_unit_price_centavos),
            cost,
            profit,
        ];
        let lines = values
            .iter()
            .enumerate()
            .map(|(i, value)| {
                let width_pt = (COLS[i].1 - 2.0) * 72.0 / 25.4;
                wrap_helvetica(value, width_pt, BODY)
            })
            .collect::<Vec<_>>();
        let height = lines.iter().map(Vec::len).max().unwrap_or(1).max(1);
        if height > 512 {
            return Err(PdfRenderError::LimitExceeded);
        }
        let mut start_line = 0;
        while start_line < height {
            if y - (LINE_MM + 2.0) < BOTTOM {
                if pages.len() >= 1_000 {
                    return Err(PdfRenderError::LimitExceeded);
                }
                pages.push(Vec::new());
                y = 141.0;
            }
            let available = ((y - BOTTOM - 2.0) / LINE_MM).floor().max(1.0) as usize;
            let end_line = (start_line + available.min(24)).min(height);
            let fragment = lines
                .iter()
                .map(|column| {
                    column
                        .iter()
                        .skip(start_line)
                        .take(end_line - start_line)
                        .cloned()
                        .collect()
                })
                .collect();
            let fragment_height = end_line - start_line;
            pages
                .last_mut()
                .ok_or(PdfRenderError::InvalidData)?
                .push((fragment, fragment_height));
            y -= fragment_height as f32 * LINE_MM + 2.0;
            start_line = end_line;
        }
    }
    let count = pages.len();
    let mut output = Vec::with_capacity(count);
    for (page_index, page_rows) in pages.iter().enumerate() {
        let mut ops = Vec::new();
        text(&mut ops, "Repuestos Autos", 15.0, 8.0, 199.0);
        text(
            &mut ops,
            "Informe de ganancia bruta por operaciones",
            12.0,
            8.0,
            190.0,
        );
        text(
            &mut ops,
            &format!("Período: {start} al {end} (inclusive)"),
            9.0,
            8.0,
            181.0,
        );
        text(&mut ops, &format!("Generado: {generated}"), 9.0, 8.0, 174.0);
        text(
            &mut ops,
            &format!("Ganancia bruta total: {}", money(total)),
            10.0,
            8.0,
            165.0,
        );
        if unknown > 0 {
            text(&mut ops, &format!("Cálculo parcial: {unknown} operación(es) con costo histórico no disponible; ganancia no calculable."), 8.0, 8.0, 158.0);
        }
        header_background(&mut ops);
        draw_rule(&mut ops, 8.0, 149.0, 281.0);
        for (x, _, label, _) in COLS {
            text(&mut ops, label, 8.0, x + 1.0, 143.0);
        }
        draw_rule(&mut ops, 8.0, 140.0, 281.0);
        let mut row_y = 135.0;
        for (line_groups, height) in page_rows {
            for (column, lines) in line_groups.iter().enumerate() {
                let (x, width, _, right) = COLS[column];
                for (line, value) in lines.iter().enumerate() {
                    let at_x = if right {
                        x + width - 1.0 - measure_helvetica_pt(value, BODY) * 25.4 / 72.0
                    } else {
                        x + 1.0
                    };
                    text(&mut ops, value, BODY, at_x, row_y - line as f32 * LINE_MM);
                }
            }
            row_y -= *height as f32 * LINE_MM + 2.0;
            draw_rule(&mut ops, 8.0, row_y + 1.0, 281.0);
        }
        for (x, _, _, _) in COLS.iter().skip(1) {
            draw_vertical(&mut ops, *x, 139.0, 17.0);
        }
        text(
            &mut ops,
            &format!("Página {} de {count}", page_index + 1),
            8.0,
            255.0,
            8.0,
        );
        output.push(PdfPage::new(Mm(PAGE_W), Mm(PAGE_H), ops));
    }
    let mut doc = PdfDocument::new("Informe de ganancia bruta");
    doc.pages = output;
    Ok(doc.save(&PdfSaveOptions::default(), &mut Vec::new()))
}

fn money(centavos: i64) -> String {
    let absolute = centavos.unsigned_abs();
    format!(
        "Bs {}{}.{:02}",
        if centavos < 0 { "-" } else { "" },
        absolute / 100,
        absolute % 100
    )
}
fn text(ops: &mut Vec<Op>, value: &str, size: f32, x: f32, y: f32) {
    ops.extend([
        Op::StartTextSection,
        Op::SetFont {
            font: PdfFontHandle::Builtin(BuiltinFont::Helvetica),
            size: Pt(size),
        },
        Op::SetTextMatrix {
            matrix: TextMatrix::Raw([1.0, 0.0, 0.0, 1.0, Pt::from(Mm(x)).0, Pt::from(Mm(y)).0]),
        },
        Op::ShowText {
            items: vec![TextItem::Text(value.to_string())],
        },
        Op::EndTextSection,
    ]);
}
fn header_background(ops: &mut Vec<Op>) {
    use printpdf::{
        color::Rgb,
        graphics::{PaintMode, Rect},
        Color, Op,
    };
    ops.push(Op::SetFillColor {
        col: Color::Rgb(Rgb::new(0.90, 0.93, 0.97, None)),
    });
    ops.push(Op::DrawRectangle {
        rectangle: Rect {
            x: Pt::from(Mm(8.0)),
            y: Pt::from(Mm(139.0)),
            width: Pt::from(Mm(281.0)),
            height: Pt::from(Mm(10.0)),
            mode: Some(PaintMode::Fill),
            winding_order: None,
        },
    });
}
fn draw_vertical(ops: &mut Vec<Op>, x: f32, top: f32, bottom: f32) {
    use printpdf::{
        graphics::{Line, LinePoint, Point},
        Op,
    };
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
    use printpdf::{
        graphics::{Line, LinePoint, Point},
        Op,
    };
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

pub fn gross_profit_operations(
    connection: &rusqlite::Connection,
    request: GrossProfitOperationsRequest,
) -> GrossProfitOperationsResponse {
    let range =
        match DashboardRange::parse_gross_profit(&request.from_utc, &request.to_exclusive_utc) {
            Ok(range) => range,
            Err(_) => return invalid_request(),
        };
    let pagination =
        match GrossProfitOperationsPagination::validate(request.page, request.page_size) {
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
