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
            local_date: "2024-03-10".into(), utc: "2024-03-10T05:00:00.000Z".into(), utc_offset_minutes: 300,
        },
        to_exclusive: repuestos_autos::commands::gross_profit_operations::GrossProfitExportDateBound {
            local_date: "2024-03-11".into(), utc: "2024-03-11T04:00:00.000Z".into(), utc_offset_minutes: 240,
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
        bytes.extend_from_slice(data); ExportSaveResult::Saved
    });
    assert_eq!(result, GrossProfitOperationsExportResponse::Success);
    bytes
}

#[test]
fn exports_every_matching_event_from_historical_snapshots_in_order_with_spanish_paginated_table() {
    let db = open_seeded_catalog().unwrap();
    for id in 1..=23 { sale(&db, id, "2024-03-10 10:00:00", Some(1000)); }
    sale(&db, 24, "2024-03-10 09:00:00", None);
    returned(&db, 50, 1);
    db.execute("UPDATE products SET sku='SKU-ACTUAL', name='Producto actual', purchase_price_centavos=9900 WHERE id=1", []).unwrap();
    let bytes = pdf(&db);
    assert!(bytes.starts_with(b"%PDF-"));
    let document = PdfDocument::parse(&bytes, &Default::default(), &mut Vec::new()).unwrap();
    assert!(document.pages.len() >= 2);
    let pages = document.extract_text();
    let full = pages.concat().join(" ");
    for heading in ["Informe de ganancia bruta", "Período:", "Generado:", "Ganancia bruta total:", "Fecha y hora", "Operación", "Producto", "Cantidad", "Precio final", "Costo histórico", "Ganancia bruta"] { assert!(full.contains(heading), "missing {heading}"); }
    assert!(full.contains("Producto histórico")); assert!(!full.contains("Producto actual")); assert!(!full.contains("SKU-ACTUAL"));
    assert!(full.contains("Devolución #50 · Venta #1")); assert!(full.contains("No disponible")); assert!(full.contains("No calculable"));
    assert!(full.contains("Cálculo parcial"));
    for id in 1..=24 { assert!(full.contains(&format!("Venta #{id}")), "operation {id} was omitted"); }
    for (index, page) in pages.iter().enumerate() {
        let text = page.join(" ");
        for heading in ["Fecha y hora", "Operación", "Producto", "Cantidad", "Precio final", "Costo histórico", "Ganancia bruta"] { assert!(text.contains(heading), "page {index} missing {heading}"); }
        assert!(text.contains(&format!("Página {} de {}", index + 1, pages.len())));
    }
    let return_pos = full.find("Devolución #50").unwrap();
    assert!(return_pos < full.find("Venta #23").unwrap(), "newer return is ordered before older sale rows");
}

#[test]
fn save_cancellation_and_failure_are_bounded_and_never_expose_a_path() {
    let db = open_seeded_catalog().unwrap();
    let cancelled = export_gross_profit_operations(&db, request(), "2024-03-12T10:00:00Z", |_| ExportSaveResult::Cancelled);
    assert_eq!(cancelled, GrossProfitOperationsExportResponse::Cancelled);
    let failed = export_gross_profit_operations(&db, request(), "2024-03-12T10:00:00Z", |_| ExportSaveResult::Failed);
    assert_eq!(serde_json::to_value(failed).unwrap(), serde_json::json!({"kind":"error","code":"export_failed","message":"No se pudo guardar el informe PDF."}));
    let value = serde_json::to_string(&GrossProfitOperationsExportResponse::Cancelled).unwrap();
    assert!(!value.contains("path"));
}

#[test]
fn rejects_date_bounds_that_do_not_match_local_midnight_and_dst_offsets() {
    let db = open_seeded_catalog().unwrap();
    let mut bad = request();
    bad.to_exclusive.utc_offset_minutes = 300;
    assert!(matches!(export_gross_profit_operations(&db, bad, "2024-03-12T10:00:00Z", |_| panic!("must not save")), GrossProfitOperationsExportResponse::Error(_)));
    let mut mismatch = request();
    mismatch.from.utc = "2024-03-10T04:00:00.000Z".into();
    assert!(matches!(export_gross_profit_operations(&db, mismatch, "2024-03-12T10:00:00Z", |_| panic!("must not save")), GrossProfitOperationsExportResponse::Error(_)));
}

#[test]
fn pdf_renders_negative_profit_and_total_with_a_visible_ascii_minus() {
    let db = open_seeded_catalog().unwrap();
    sale(&db, 1, "2024-03-10 10:00:00", Some(3000));
    let document = PdfDocument::parse(&pdf(&db), &Default::default(), &mut Vec::new()).unwrap();
    let text = document.extract_text().concat().join(" ");
    assert!(text.contains("Bs -5.00"), "negative profit and total must retain an ASCII minus: {text}");
    assert!(!text.contains("?5.00"), "negative sign must not be replaced by an unsupported glyph");
}

#[test]
fn rejects_cumulative_returns_above_sold_quantity_before_saving() {
    let db = open_seeded_catalog().unwrap();
    sale(&db, 1, "2024-03-10 10:00:00", Some(1000));
    db.execute_batch("DROP TRIGGER sale_return_lines_validate_insert;").unwrap();
    returned(&db, 50, 1);
    returned(&db, 51, 1);
    let result = export_gross_profit_operations(&db, request(), "2024-03-12T10:00:00Z", |_| panic!("must not save"));
    assert_eq!(result, GrossProfitOperationsExportResponse::Error(repuestos_autos::commands::gross_profit_operations::GrossProfitOperationsError {
        code: "export_failed", message: "No se pudo generar el informe PDF.",
    }));
}

#[test]
fn preserves_multiple_partial_returns_within_sold_quantity() {
    let db = open_seeded_catalog().unwrap();
    sale(&db, 1, "2024-03-10 10:00:00", Some(1000));
    db.execute_batch("DROP TRIGGER confirmed_sale_lines_immutable_price;").unwrap();
    db.execute("UPDATE sale_lines SET quantity = 3 WHERE id = 1", []).unwrap();
    returned(&db, 50, 1);
    returned(&db, 51, 1);
    let document = PdfDocument::parse(&pdf(&db), &Default::default(), &mut Vec::new()).unwrap();
    let text = document.extract_text().concat().join(" ");
    assert!(text.contains("Devolución #50"));
    assert!(text.contains("Devolución #51"));
}

#[test]
fn rejects_hostile_historical_text_before_saving_without_truncating_the_row() {
    let db = open_seeded_catalog().unwrap();
    sale_with_name(&db, 1, &"X".repeat(513));
    let result = export_gross_profit_operations(&db, request(), "2024-03-12T10:00:00Z", |_| panic!("must not save"));
    assert_eq!(result, GrossProfitOperationsExportResponse::Error(repuestos_autos::commands::gross_profit_operations::GrossProfitOperationsError {
        code: "resource_limit", message: "El informe supera los límites de recursos y no se guardó.",
    }));
}

#[test]
fn rejects_operation_count_over_resource_limit_before_saving() {
    let db = open_seeded_catalog().unwrap();
    for id in 1..=10_001 { sale(&db, id, "2024-03-10 10:00:00", Some(1000)); }
    let result = export_gross_profit_operations(&db, request(), "2024-03-12T10:00:00Z", |_| panic!("must not save"));
    assert_eq!(result, GrossProfitOperationsExportResponse::Error(repuestos_autos::commands::gross_profit_operations::GrossProfitOperationsError {
        code: "resource_limit", message: "El informe supera los límites de recursos y no se guardó.",
    }));
}
