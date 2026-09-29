use repuestos_autos::{
    commands::movement_ledger::{
        export_movement_ledger, ExportSaveResult, MovementLedgerExportRequest,
        MovementLedgerExportResponse,
    },
    infrastructure::sqlite::open_seeded_catalog,
};
use printpdf::{ops::Op, text::TextItem};
use rusqlite::Connection;

fn request() -> MovementLedgerExportRequest {
    MovementLedgerExportRequest {
        from_utc: "2025-01-01T00:00:00Z".into(),
        to_exclusive_utc: "2025-01-02T00:00:00Z".into(),
        product_id: Some(1),
        movement_type: Some("stock_entry".into()),
    }
}

fn insert_rows(connection: &Connection, count: i64) {
    for id in 1..=count {
        connection.execute(
            "INSERT INTO inventory_movements
             (id, product_id, movement_type, quantity_delta, occurred_at, source_reference, request_id, resulting_quantity)
             VALUES (?1, 1, 'stock_entry', 1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                id,
                format!("2025-01-01 {:02}:{:02}:{:02}", id / 3600, (id / 60) % 60, id % 60),
                (id == 1).then_some("Detalle persistido largo sin espacios 12345678901234567890 y continúa con información registrada para verificar ajuste seguro de columna"),
                format!("movement-ledger-export-{id}"),
                id,
            ],
        ).unwrap();
    }
}

#[test]
fn export_uses_active_filters_and_returns_paginated_pdf_bytes() {
    let connection = open_seeded_catalog().unwrap();
    insert_rows(&connection, 21);
    connection.execute("UPDATE products SET name = ?1 WHERE id = 1", ["Filtro de aceite premium para motor de alto rendimiento con identificador extendido"]).unwrap();
    let mut pdf = Vec::new();
    let response = export_movement_ledger(&connection, request(), "2025-01-02T12:00:00Z", |bytes| {
        pdf.extend_from_slice(bytes);
        ExportSaveResult::Saved
    });

    assert_eq!(response, MovementLedgerExportResponse::Success);
    assert!(pdf.starts_with(b"%PDF-"));
    let document = printpdf::PdfDocument::parse(
        &pdf,
        &printpdf::PdfParseOptions::default(),
        &mut Vec::new(),
    ).unwrap();
    assert!(document.pages.len() >= 2);
    let extracted_pages = document.extract_text();
    let extracted = extracted_pages.concat().join(" ");
    assert!(extracted.contains("Repuestos Autos"));
    assert!(extracted.contains("Movement Ledger"));
    assert!(extracted.contains("Period: 2025-01-01T00:00:00Z to (exclusive) 2025-01-02T00:00:00Z"));
    assert!(extracted.contains("Product ID: 1 | Movement type: stock_entry"));
    assert!(extracted.contains("Generated: 2025-01-02T12:00:00Z"));
    assert!(extracted.contains("Filtro de aceite"));
    assert!(extracted.contains("FLT-001"));
    assert!(extracted.contains("SKU"));
    assert!(extracted.contains("Detalle persistido largo"));
    assert!(extracted.contains("continúa con información registrada"));
    assert!(extracted.contains("21"), "stock-entry resulting quantity is included");
    assert!(extracted.contains("stock_entry"));
    for (page_index, page_text) in extracted_pages.iter().enumerate() {
        let page_text = page_text.join(" ");
        assert!(page_text.contains("Movement Ledger"), "page {page_index} omits title");
        assert!(page_text.contains("Product ID: 1 | Movement type: stock_entry"), "page {page_index} omits filters");
        assert!(page_text.contains("Generated: 2025-01-02T12:00:00Z"), "page {page_index} omits generation time");
        assert!(page_text.contains("Stored detail"), "page {page_index} omits column headings");
        assert!(page_text.contains(&format!("Page {} of {}", page_index + 1, document.pages.len())));
    }
    let page_zero_positions = positioned_text(&document.pages[0].ops);
    let header_y = page_zero_positions.iter().find(|(text, _)| text.contains("ID / date")).map(|(_, y)| *y).expect(&format!("missing column heading positions: {page_zero_positions:#?}"));
    let first_row_y = page_zero_positions.iter().find(|(text, _)| text.contains("00:00:21")).map(|(_, y)| *y).expect(&format!("missing first-row positions: {page_zero_positions:#?}"));
    assert!(first_row_y < header_y - 10.0, "first data row must start below the full header block");
    let long_detail_lines = document.pages.iter().flat_map(|page| positioned_text(&page.ops)).filter(|(text, _)| text.contains("Detalle") || text.contains("persistido") || text.contains("largo")).count();
    assert!(long_detail_lines > 1, "long persisted detail must wrap into multiple layout operations");
}

fn positioned_text(ops: &[Op]) -> Vec<(String, f32)> {
    let mut y = 0.0;
    let mut output = Vec::new();
    for op in ops {
        match op {
            Op::SetTextMatrix { matrix } => {
                y = matrix.as_array()[5];
            }
            Op::SetTextCursor { pos } => {
                y = pos.y.0;
            }
            Op::ShowText { items } => {
                for item in items {
                    if let TextItem::Text(text) = item {
                        output.push((text.clone(), y));
                    }
                }
            }
            _ => {}
        }
    }
    output
}

#[test]
fn export_fragments_extreme_unbroken_detail_across_bounded_pages_without_losing_text() {
    let connection = open_seeded_catalog().unwrap();
    let detail = "q".repeat(1_000);
    connection.execute(
        "INSERT INTO inventory_movements
         (id, product_id, movement_type, quantity_delta, occurred_at, source_reference, request_id, resulting_quantity)
         VALUES (1, 1, 'stock_entry', 1, '2025-01-01 12:00:00', ?1, 'movement-ledger-export-extreme', 1)",
        [&detail],
    ).unwrap();
    let mut pdf = Vec::new();
    let response = export_movement_ledger(&connection, request(), "2025-01-02T12:00:00Z", |bytes| {
        pdf.extend_from_slice(bytes);
        ExportSaveResult::Saved
    });
    assert_eq!(response, MovementLedgerExportResponse::Success);
    let document = printpdf::PdfDocument::parse(
        &pdf,
        &printpdf::PdfParseOptions::default(),
        &mut Vec::new(),
    ).unwrap();
    assert!(document.pages.len() >= 2);
    let extracted_pages = document.extract_text();
    let all_text = extracted_pages.concat().join(" ");
    assert_eq!(all_text.chars().filter(|character| *character == 'q').count(), 1_000);
    for (page_index, page) in document.pages.iter().enumerate() {
        let page_text = extracted_pages[page_index].join(" ");
        assert!(page_text.contains("Movement Ledger"));
        assert!(page_text.contains("Product ID: 1 | Movement type: stock_entry"));
        assert!(page_text.contains("Generated: 2025-01-02T12:00:00Z"));
        assert!(page_text.contains("Stored detail"));
        assert!(page_text.contains(&format!("Page {} of {}", page_index + 1, document.pages.len())));
        if page_index > 0 {
            assert!(page_text.contains("Continuation of movement #1"));
        }
        for (_, y_pt) in positioned_text(&page.ops) {
            assert!(y_pt >= printpdf::Pt::from(printpdf::Mm(10.0)).0, "text below printable bottom on page {page_index}: {y_pt}");
            assert!(y_pt <= printpdf::Pt::from(printpdf::Mm(290.0)).0, "text above printable top on page {page_index}: {y_pt}");
        }
    }
}

#[test]
fn export_rejects_more_than_two_thousand_matching_rows_without_saving() {
    let connection = open_seeded_catalog().unwrap();
    insert_rows(&connection, 2_001);
    let mut save_called = false;
    let response = export_movement_ledger(&connection, request(), "2025-01-02T12:00:00Z", |_| {
        save_called = true;
        ExportSaveResult::Saved
    });
    assert!(!save_called);
    assert_eq!(response, MovementLedgerExportResponse::Error(
        repuestos_autos::commands::movement_ledger::MovementLedgerCommandError {
            code: "export_limit_exceeded",
            message: "More than 2,000 movements match. Narrow the filters and try again.",
        },
    ));
}

#[test]
fn export_distinguishes_cancellation_and_hides_write_failure_details() {
    let connection = open_seeded_catalog().unwrap();
    let cancelled = export_movement_ledger(&connection, request(), "2025-01-02T12:00:00Z", |_| ExportSaveResult::Cancelled);
    assert_eq!(cancelled, MovementLedgerExportResponse::Cancelled);

    let failed = export_movement_ledger(&connection, request(), "2025-01-02T12:00:00Z", |_| ExportSaveResult::Failed);
    assert_eq!(failed, MovementLedgerExportResponse::Error(
        repuestos_autos::commands::movement_ledger::MovementLedgerCommandError {
            code: "export_failed",
            message: "The movement ledger could not be saved.",
        },
    ));
}

#[test]
fn export_response_serialization_preserves_the_frontend_contract() {
    assert_eq!(
        serde_json::to_value(MovementLedgerExportResponse::Success).unwrap(),
        serde_json::json!({ "kind": "success" }),
    );
    assert_eq!(
        serde_json::to_value(MovementLedgerExportResponse::Cancelled).unwrap(),
        serde_json::json!({ "kind": "cancelled" }),
    );
    assert_eq!(
        serde_json::to_value(MovementLedgerExportResponse::Error(
            repuestos_autos::commands::movement_ledger::MovementLedgerCommandError {
                code: "persistence_failure",
                message: "The movement ledger could not be loaded.",
            },
        )).unwrap(),
        serde_json::json!({
            "kind": "error",
            "code": "persistence_failure",
            "message": "The movement ledger could not be loaded."
        }),
    );
}

#[test]
fn export_requires_strict_filter_fields() {
    let unknown = serde_json::json!({
        "from_utc": "2025-01-01T00:00:00Z",
        "to_exclusive_utc": "2025-01-02T00:00:00Z",
        "page": 2
    });
    assert!(serde_json::from_value::<MovementLedgerExportRequest>(unknown).is_err());
}
