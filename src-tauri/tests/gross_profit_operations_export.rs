use printpdf::PdfDocument;
use repuestos_autos::{
    commands::gross_profit_operations::{
        export_gross_profit_operations, ExportSaveResult, GrossProfitOperationsExportRequest,
        GrossProfitOperationsExportResponse,
    },
    infrastructure::sqlite::open_seeded_catalog,
};
use rusqlite::params;

fn request() -> GrossProfitOperationsExportRequest {
    GrossProfitOperationsExportRequest {
        from: repuestos_autos::commands::gross_profit_operations::GrossProfitExportDateBound {
            local_date: "2024-03-10".into(),
            utc: "2024-03-10T05:00:00.000Z".into(),
            utc_offset_minutes: 300,
        },
        to_exclusive:
            repuestos_autos::commands::gross_profit_operations::GrossProfitExportDateBound {
                local_date: "2024-03-11".into(),
                utc: "2024-03-11T04:00:00.000Z".into(),
                utc_offset_minutes: 240,
            },
    }
}

fn sale(db: &rusqlite::Connection, id: i64, at: &str, cost: Option<i64>) {
    db.execute("INSERT INTO sales (id, request_id, status, total_centavos, confirmed_at) VALUES (?1, ?2, 'confirmed', 2500, ?3)", params![id, format!("sale-{id}"), at]).unwrap();
    db.execute("INSERT INTO sale_lines (id, sale_id, product_id, sku_snapshot, product_name_snapshot, quantity, negotiated_unit_price_centavos, minimum_unit_price_snapshot_centavos, list_price_snapshot_centavos, unit_cost_snapshot_centavos, line_total_centavos) VALUES (?1, ?1, 1, 'SKU-HIST', 'Producto histórico', 1, 2500, 0, 2500, ?2, 2500)", params![id, cost]).unwrap();
}

fn sale_with_name(db: &rusqlite::Connection, id: i64, product_name: &str) {
    db.execute("INSERT INTO sales (id, request_id, status, total_centavos, confirmed_at) VALUES (?1, ?2, 'confirmed', 2500, '2024-03-10 10:00:00')", params![id, format!("sale-{id}")]).unwrap();
    db.execute("INSERT INTO sale_lines (id, sale_id, product_id, sku_snapshot, product_name_snapshot, quantity, negotiated_unit_price_centavos, minimum_unit_price_snapshot_centavos, list_price_snapshot_centavos, unit_cost_snapshot_centavos, line_total_centavos) VALUES (?1, ?1, 1, 'SKU-HIST', ?2, 1, 2500, 0, 2500, 1000, 2500)", params![id, product_name]).unwrap();
}

fn returned(db: &rusqlite::Connection, id: i64, sale_id: i64) {
    db.execute("INSERT INTO post_sale_requests (id, request_id, operation_kind, sale_id, payload_version, canonical_payload, payload_sha256) VALUES (?1, ?2, 'return', ?3, 1, X'01', printf('%064d', ?1))", params![id, format!("return-{id}"), sale_id]).unwrap();
    db.execute("INSERT INTO sale_returns (id, sale_id, occurred_at) VALUES (?1, ?2, '2024-03-10 12:00:00')", params![id, sale_id]).unwrap();
    db.execute("INSERT INTO inventory_movements (id, product_id, sale_id, sale_line_id, movement_type, quantity_delta) VALUES (?1, 1, ?2, ?2, 'return', 1)", params![id + 10_000, sale_id]).unwrap();
    db.execute("INSERT INTO sale_return_lines (return_id, sale_id, sale_line_id, product_id, quantity, movement_id) VALUES (?1, ?2, ?2, 1, 1, ?3)", params![id, sale_id, id + 10_000]).unwrap();
}

fn pdf(db: &rusqlite::Connection) -> Vec<u8> {
    let mut bytes = Vec::new();
    let result = export_gross_profit_operations(db, request(), "2024-03-12T10:00:00Z", |data| {
        bytes.extend_from_slice(data);
        ExportSaveResult::Saved
    });
    assert_eq!(result, GrossProfitOperationsExportResponse::Success);
    bytes
}

#[test]
fn exports_every_matching_event_from_historical_snapshots_in_order_with_spanish_paginated_table() {
    let db = open_seeded_catalog().unwrap();
    for id in 1..=23 {
        sale(&db, id, "2024-03-10 10:00:00", Some(1000));
    }
    sale(&db, 24, "2024-03-10 09:00:00", None);
    returned(&db, 50, 1);
    db.execute("UPDATE products SET sku='SKU-ACTUAL', name='Producto actual', purchase_price_centavos=9900 WHERE id=1", []).unwrap();
    let bytes = pdf(&db);
    assert!(bytes.starts_with(b"%PDF-"));
    let document = PdfDocument::parse(&bytes, &Default::default(), &mut Vec::new()).unwrap();
    assert!(document.pages.len() >= 2);
    let pages = document.extract_text();
    let full = pages.concat().join(" ");
    for heading in [
        "Informe de ganancia bruta",
        "Período:",
        "Generado:",
        "Ganancia bruta total:",
        "Fecha y hora",
        "Operación",
        "Producto",
        "Cantidad",
        "Precio final",
        "Costo histórico",
        "Ganancia bruta",
    ] {
        assert!(full.contains(heading), "missing {heading}");
    }
    assert!(
        full.contains("10/03/2024 al 10/03/2024 (inclusive)"),
        "period is DD/MM/YYYY and inclusive: {full}"
    );
    let generated =
        repuestos_autos::commands::pdf_format::format_timestamp_local("2024-03-12T10:00:00Z")
            .unwrap();
    assert!(
        full.contains(&format!("Generado: {generated}")),
        "generated timestamp uses local display format: {full}"
    );
    assert!(
        full.contains("Ganancia bruta total: Bs 330.00"),
        "historical total remains unchanged: {full}"
    );
    assert!(
        full.contains("Hora local no disponible"),
        "invalid stored timestamps have an honest Spanish marker: {full}"
    );
    assert!(
        full.contains("Venta n.º 1") && full.contains("Devolución n.º 50"),
        "operation labels are Spanish"
    );
    assert!(full.contains("Producto histórico"));
    assert!(!full.contains("Producto actual"));
    assert!(!full.contains("SKU-ACTUAL"));
    assert!(full.contains("Devolución n.º 50 · Venta n.º 1"));
    assert!(full.contains("No disponible"));
    assert!(full.contains("No calculable"));
    assert!(full.contains("Cálculo parcial"));
    for id in 1..=24 {
        assert!(
            full.contains(&format!("Venta n.º {id}")),
            "operation {id} was omitted"
        );
    }
    for (index, page) in pages.iter().enumerate() {
        let text = page.join(" ");
        for heading in [
            "Fecha y hora",
            "Operación",
            "Producto",
            "Cantidad",
            "Precio final",
            "Costo histórico",
            "Ganancia bruta",
        ] {
            assert!(text.contains(heading), "page {index} missing {heading}");
        }
        assert!(text.contains(&format!("Página {} de {}", index + 1, pages.len())));
    }
    let return_pos = full.find("Devolución n.º 50").unwrap();
    assert!(
        return_pos < full.find("Venta n.º 23").unwrap(),
        "newer return is ordered before older sale rows"
    );
}

#[test]
fn numeric_columns_use_nine_point_text_and_align_to_their_actual_right_edges() {
    use printpdf::{matrix::TextMatrix, ops::Op, text::TextItem};
    let db = open_seeded_catalog().unwrap();
    sale(&db, 1, "2024-03-10 10:00:00", Some(1000));
    let document = PdfDocument::parse(&pdf(&db), &Default::default(), &mut Vec::new()).unwrap();
    let mut font_size = 0.0;
    let mut matrix = [0.0; 6];
    let mut aligned = 0;
    let mut body_font = false;
    for page in &document.pages {
        for op in &page.ops {
            match op {
                Op::SetFont { size, .. } => {
                    font_size = size.0;
                    body_font |= (font_size - 9.0).abs() < 0.01;
                }
                Op::SetTextMatrix {
                    matrix: TextMatrix::Raw(value),
                } => matrix = *value,
                Op::ShowText { items } => {
                    for item in items {
                        let TextItem::Text(value) = item else {
                            continue;
                        };
                        let edge_mm = match value.as_str() {
                            "Bs 25.00" => Some(221.0),
                            "Bs 10.00" => Some(257.0),
                            "Bs 15.00" => Some(289.0),
                            _ => None,
                        };
                        if let Some(edge_mm) = edge_mm {
                            assert!((font_size - 9.0).abs() < 0.01);
                            let width_pt =
                                repuestos_autos::commands::pdf_table::measure_helvetica_pt(
                                    value, font_size,
                                );
                            let expected = printpdf::Pt::from(printpdf::Mm(edge_mm - 1.0)).0;
                            assert!(
                                (matrix[4] + width_pt - expected).abs() < 0.1,
                                "{value} right edge was {}, expected {expected}",
                                matrix[4] + width_pt
                            );
                            aligned += 1;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    assert!(body_font, "table body uses readable 9pt text");
    assert_eq!(aligned, 3, "price, cost, and profit values all align");
}

#[test]
fn wraps_a_512_character_accented_snapshot_inside_the_product_cell() {
    use printpdf::{matrix::TextMatrix, ops::Op, text::TextItem};

    let db = open_seeded_catalog().unwrap();
    sale_with_name(&db, 1, &"É".repeat(512));
    let document = PdfDocument::parse(&pdf(&db), &Default::default(), &mut Vec::new()).unwrap();
    let expected_width_pt = 209.8;
    let mut characters = String::new();
    for page in &document.pages {
        let mut size = 0.0;
        let mut matrix = [0.0; 6];
        for op in &page.ops {
            match op {
                Op::SetFont {
                    size: font_size, ..
                } => size = font_size.0,
                Op::SetTextMatrix {
                    matrix: TextMatrix::Raw(value),
                } => matrix = *value,
                Op::ShowText { items } => {
                    for item in items {
                        let TextItem::Text(value) = item else {
                            continue;
                        };
                        let accented_chars = value.chars().filter(|ch| *ch == 'É').count();
                        if accented_chars > 0 {
                            let width_pt = accented_chars as f32 * 667.0 * size / 1000.0;
                            assert!(
                                width_pt <= expected_width_pt,
                                "product fragment exceeds cell: {width_pt}pt"
                            );
                            assert!(
                                matrix[4] + width_pt < printpdf::Pt::from(printpdf::Mm(167.0)).0,
                                "product overlaps quantity column"
                            );
                            characters.push_str(&"É".repeat(accented_chars));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    assert_eq!(
        characters,
        "É".repeat(512),
        "observed {} product glyphs across {} pages",
        characters.chars().count(),
        document.pages.len()
    );
}

#[test]
fn one_page_spanning_snapshot_keeps_every_chunk_in_order_and_repeats_headers() {
    let db = open_seeded_catalog().unwrap();
    let name = (0..128).map(|_| "É\nÇ\n").collect::<String>();
    sale_with_name(&db, 1, name.trim_end());
    let document = PdfDocument::parse(&pdf(&db), &Default::default(), &mut Vec::new()).unwrap();
    let pages = document.extract_text();
    assert!(pages.len() > 1, "long single row continues on later pages");
    for page in &pages {
        let text = page.join(" ");
        assert!(
            text.contains("Fecha y hora"),
            "continuation page repeats headers"
        );
        assert!(text.contains("Página "), "continuation page repeats footer");
    }
    let extracted = pages.concat().join(" ");
    for page in &document.pages {
        let mut matrix = [0.0; 6];
        let mut prior_baseline = None;
        for op in &page.ops {
            match op {
                printpdf::ops::Op::SetTextMatrix {
                    matrix: printpdf::matrix::TextMatrix::Raw(value),
                } => matrix = *value,
                printpdf::ops::Op::ShowText { items } => {
                    for item in items {
                        let printpdf::text::TextItem::Text(value) = item else {
                            continue;
                        };
                        if value == "É" || value == "Ç" {
                            assert!(matrix[4] >= printpdf::Pt::from(printpdf::Mm(92.0)).0);
                            assert!(matrix[4] < printpdf::Pt::from(printpdf::Mm(165.0)).0);
                            assert!(matrix[5] > printpdf::Pt::from(printpdf::Mm(17.0)).0);
                            if let Some(previous) = prior_baseline {
                                assert!(
                                    previous - matrix[5]
                                        >= printpdf::Pt::from(printpdf::Mm(4.5)).0 - 0.1,
                                    "hard-line fragments overlap vertically"
                                );
                            }
                            prior_baseline = Some(matrix[5]);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let ordered_glyphs = extracted
        .split_whitespace()
        .filter(|word| *word == "É" || *word == "Ç")
        .collect::<Vec<_>>();
    let expected = (0..128).flat_map(|_| ["É", "Ç"]).collect::<Vec<_>>();
    assert_eq!(
        ordered_glyphs, expected,
        "every hard-line chunk survives in order"
    );
    assert!(
        extracted.contains("Venta n.º 1"),
        "row data survives continuation"
    );
}

#[test]
fn save_cancellation_and_failure_are_bounded_and_never_expose_a_path() {
    let db = open_seeded_catalog().unwrap();
    let cancelled = export_gross_profit_operations(&db, request(), "2024-03-12T10:00:00Z", |_| {
        ExportSaveResult::Cancelled
    });
    assert_eq!(cancelled, GrossProfitOperationsExportResponse::Cancelled);
    let failed = export_gross_profit_operations(&db, request(), "2024-03-12T10:00:00Z", |_| {
        ExportSaveResult::Failed
    });
    assert_eq!(
        serde_json::to_value(failed).unwrap(),
        serde_json::json!({"kind":"error","code":"export_failed","message":"No se pudo guardar el informe PDF."})
    );
    let value = serde_json::to_string(&GrossProfitOperationsExportResponse::Cancelled).unwrap();
    assert!(!value.contains("path"));
}

#[test]
fn rendered_graphics_state_and_table_rules_remain_readable_on_every_page() {
    use printpdf::{graphics::Line, ops::Op};

    let db = open_seeded_catalog().unwrap();
    for id in 1..=23 {
        sale(&db, id, "2024-03-10 10:00:00", Some(1000));
    }
    let document = PdfDocument::parse(&pdf(&db), &Default::default(), &mut Vec::new()).unwrap();
    assert!(document.pages.len() > 1);
    let mut checked_text = 0;
    for (page_index, page) in document.pages.iter().enumerate() {
        let mut color = vec![0.0, 0.0, 0.0];
        let mut baselines = Vec::new();
        let mut horizontal_rules = Vec::new();
        let mut vertical_bottoms = Vec::new();
        let mut x = 0.0;
        let mut y = 0.0;
        for op in &page.ops {
            match op {
                Op::SetFillColor { col } => color = col.clone().into_vec(),
                Op::SetTextMatrix {
                    matrix: printpdf::matrix::TextMatrix::Raw(m),
                } => {
                    x = m[4] * 25.4 / 72.0;
                    y = m[5] * 25.4 / 72.0;
                }
                Op::ShowText { items } => {
                    for item in items {
                        if matches!(item, printpdf::text::TextItem::Text(value) if !value.is_empty())
                        {
                            assert!(color.len() == 3 && color.iter().all(|channel| *channel <= 0.2),
                                "page {page_index} text at ({x:.2},{y:.2}) has low-contrast effective fill {color:?}");
                            baselines.push(y);
                            checked_text += 1;
                        }
                    }
                }
                Op::DrawLine {
                    line: Line { points, .. },
                } if points.len() == 2 => {
                    let a = &points[0].p;
                    let b = &points[1].p;
                    let ay = a.y.0 * 25.4 / 72.0;
                    let by = b.y.0 * 25.4 / 72.0;
                    if (ay - by).abs() < 0.01 {
                        if a.x.0 < b.x.0 {
                            horizontal_rules.push(ay);
                        }
                    } else {
                        vertical_bottoms.push(ay.min(by));
                    }
                }
                _ => {}
            }
        }
        assert!(horizontal_rules.iter().any(|rule| *rule < 140.0));
        for rule in horizontal_rules.iter().filter(|rule| **rule < 140.0) {
            assert!(
                baselines
                    .iter()
                    .filter(|baseline| **baseline < 140.0)
                    .all(|baseline| (baseline - rule).abs() >= 2.8),
                "page {page_index} table separator at {rule:.2}mm intrudes into text clearance"
            );
        }
        let table_bottom = horizontal_rules
            .iter()
            .copied()
            .filter(|rule| *rule < 140.0)
            .fold(140.0_f32, f32::min);
        assert!(vertical_bottoms.iter().all(|bottom| (bottom - table_bottom).abs() < 0.1),
            "page {page_index} vertical rules must stop at table end {table_bottom}, got {vertical_bottoms:?}");
    }
    assert!(
        checked_text > 40,
        "every header, body, and footer text segment was examined"
    );

    let short_db = open_seeded_catalog().unwrap();
    sale(&short_db, 1, "2024-03-10 10:00:00", Some(1000));
    let short = PdfDocument::parse(&pdf(&short_db), &Default::default(), &mut Vec::new()).unwrap();
    let mut horizontal = Vec::new();
    let mut vertical = Vec::new();
    for op in &short.pages[0].ops {
        if let Op::DrawLine {
            line: Line { points, .. },
        } = op
        {
            if points.len() != 2 {
                continue;
            }
            let a = &points[0].p;
            let b = &points[1].p;
            let ay = a.y.0 * 25.4 / 72.0;
            let by = b.y.0 * 25.4 / 72.0;
            if (ay - by).abs() < 0.01 {
                horizontal.push(ay);
            } else {
                vertical.push(ay.min(by));
            }
        }
    }
    let actual_bottom = horizontal
        .into_iter()
        .filter(|y| *y < 140.0)
        .fold(140.0_f32, f32::min);
    assert!(
        vertical.iter().all(|y| (*y - actual_bottom).abs() < 0.1),
        "short report vertical rules end at its actual last row boundary"
    );
}

#[test]
fn rejects_date_bounds_that_do_not_match_local_midnight_and_dst_offsets() {
    let db = open_seeded_catalog().unwrap();
    let mut bad = request();
    bad.to_exclusive.utc_offset_minutes = 300;
    assert!(matches!(
        export_gross_profit_operations(&db, bad, "2024-03-12T10:00:00Z", |_| panic!(
            "must not save"
        )),
        GrossProfitOperationsExportResponse::Error(_)
    ));
    let mut mismatch = request();
    mismatch.from.utc = "2024-03-10T04:00:00.000Z".into();
    assert!(matches!(
        export_gross_profit_operations(&db, mismatch, "2024-03-12T10:00:00Z", |_| panic!(
            "must not save"
        )),
        GrossProfitOperationsExportResponse::Error(_)
    ));
}

#[test]
fn pdf_renders_negative_profit_and_total_with_a_visible_ascii_minus() {
    let db = open_seeded_catalog().unwrap();
    sale(&db, 1, "2024-03-10 10:00:00", Some(3000));
    let document = PdfDocument::parse(&pdf(&db), &Default::default(), &mut Vec::new()).unwrap();
    let text = document.extract_text().concat().join(" ");
    assert!(
        text.contains("Bs -5.00"),
        "negative profit and total must retain an ASCII minus: {text}"
    );
    assert!(
        !text.contains("?5.00"),
        "negative sign must not be replaced by an unsupported glyph"
    );
}

#[test]
fn rejects_cumulative_returns_above_sold_quantity_before_saving() {
    let db = open_seeded_catalog().unwrap();
    sale(&db, 1, "2024-03-10 10:00:00", Some(1000));
    db.execute_batch("DROP TRIGGER sale_return_lines_validate_insert;")
        .unwrap();
    returned(&db, 50, 1);
    returned(&db, 51, 1);
    let result = export_gross_profit_operations(&db, request(), "2024-03-12T10:00:00Z", |_| {
        panic!("must not save")
    });
    assert_eq!(
        result,
        GrossProfitOperationsExportResponse::Error(
            repuestos_autos::commands::gross_profit_operations::GrossProfitOperationsError {
                code: "export_failed",
                message: "No se pudo generar el informe PDF.",
            }
        )
    );
}

#[test]
fn preserves_multiple_partial_returns_within_sold_quantity() {
    let db = open_seeded_catalog().unwrap();
    sale(&db, 1, "2024-03-10 10:00:00", Some(1000));
    db.execute_batch("DROP TRIGGER confirmed_sale_lines_immutable_price;")
        .unwrap();
    db.execute("UPDATE sale_lines SET quantity = 3 WHERE id = 1", [])
        .unwrap();
    returned(&db, 50, 1);
    returned(&db, 51, 1);
    let document = PdfDocument::parse(&pdf(&db), &Default::default(), &mut Vec::new()).unwrap();
    let text = document.extract_text().concat().join(" ");
    assert!(text.contains("Devolución n.º 50"));
    assert!(text.contains("Devolución n.º 51"));
}

#[test]
fn rejects_hostile_historical_text_before_saving_without_truncating_the_row() {
    let db = open_seeded_catalog().unwrap();
    sale_with_name(&db, 1, &"X".repeat(513));
    let result = export_gross_profit_operations(&db, request(), "2024-03-12T10:00:00Z", |_| {
        panic!("must not save")
    });
    assert_eq!(
        result,
        GrossProfitOperationsExportResponse::Error(
            repuestos_autos::commands::gross_profit_operations::GrossProfitOperationsError {
                code: "resource_limit",
                message: "El informe supera los límites de recursos y no se guardó.",
            }
        )
    );
}

#[test]
fn rejects_operation_count_over_resource_limit_before_saving() {
    let db = open_seeded_catalog().unwrap();
    for id in 1..=10_001 {
        sale(&db, id, "2024-03-10 10:00:00", Some(1000));
    }
    let result = export_gross_profit_operations(&db, request(), "2024-03-12T10:00:00Z", |_| {
        panic!("must not save")
    });
    assert_eq!(
        result,
        GrossProfitOperationsExportResponse::Error(
            repuestos_autos::commands::gross_profit_operations::GrossProfitOperationsError {
                code: "resource_limit",
                message: "El informe supera los límites de recursos y no se guardó.",
            }
        )
    );
}
