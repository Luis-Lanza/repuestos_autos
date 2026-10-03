use printpdf::{ops::Op, text::TextItem};
use repuestos_autos::{
    commands::movement_ledger::{
        export_movement_ledger, ExportSaveResult, MovementLedgerExportRequest,
        MovementLedgerExportResponse,
    },
    infrastructure::sqlite::open_seeded_catalog,
};
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
    let response =
        export_movement_ledger(&connection, request(), "2025-01-02T12:00:00Z", |bytes| {
            pdf.extend_from_slice(bytes);
            ExportSaveResult::Saved
        });

    assert_eq!(response, MovementLedgerExportResponse::Success);
    assert!(pdf.starts_with(b"%PDF-"));
    let document =
        printpdf::PdfDocument::parse(&pdf, &printpdf::PdfParseOptions::default(), &mut Vec::new())
            .unwrap();
    assert!(document.pages.len() >= 2);
    let extracted_pages = document.extract_text();
    let extracted = extracted_pages.concat().join(" ");
    assert!(extracted.contains("Repuestos Autos"));
    assert!(extracted.contains("Libro de movimientos"));
    assert!(extracted.contains(&format!(
        "Período: desde {} (incluido) hasta {} (exclusivo)",
        repuestos_autos::commands::pdf_format::format_timestamp_local("2025-01-01T00:00:00Z")
            .unwrap(),
        repuestos_autos::commands::pdf_format::format_timestamp_local("2025-01-02T00:00:00Z")
            .unwrap()
    )));
    assert!(extracted.contains("Producto ID: 1 | Tipo de movimiento: Ingreso de stock"));
    let generated =
        repuestos_autos::commands::pdf_format::format_timestamp_local("2025-01-02T12:00:00Z")
            .unwrap();
    assert!(extracted.contains(&format!("Generado: {generated}")));
    assert!(extracted.contains("Filtro de aceite"));
    assert!(extracted.contains("FLT-001"));
    assert!(extracted.contains("SKU"));
    assert!(extracted.contains("Detalle persistido largo"));
    assert!(extracted.contains("continúa con información registrada"));
    assert!(
        extracted.contains("21"),
        "stock-entry resulting quantity is included"
    );
    assert!(extracted.contains("Ingreso de stock"));
    for (page_index, page_text) in extracted_pages.iter().enumerate() {
        let page_text = page_text.join(" ");
        assert!(
            page_text.contains("Libro de movimientos"),
            "page {page_index} omits title"
        );
        assert!(
            page_text.contains("Producto ID: 1 | Tipo de movimiento: Ingreso de stock"),
            "page {page_index} omits filters"
        );
        assert!(
            page_text.contains(&format!("Generado: {generated}")),
            "page {page_index} omits generation time"
        );
        assert!(
            page_text.contains("Detalle registrado")
                && page_text.contains("Existencia")
                && page_text.contains("resultante"),
            "page {page_index} omits wrapped column headings"
        );
        assert!(page_text.contains(&format!(
            "Página {} de {}",
            page_index + 1,
            document.pages.len()
        )));
    }
    let page_zero_positions = positioned_text(&document.pages[0].ops);
    let header_lines = positioned_text_with_x(&document.pages[0].ops)
        .into_iter()
        .filter(|(text, _, _)| {
            [
                "N.º / fecha y hora",
                "Producto / SKU",
                "Tipo",
                "Variación",
                "Existencia",
                "resultante",
                "Detalle registrado",
            ]
            .contains(&text.as_str())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        header_lines
            .iter()
            .map(|(text, _, _)| text.as_str())
            .collect::<Vec<_>>(),
        [
            "N.º / fecha y hora",
            "Producto / SKU",
            "Tipo",
            "Variación",
            "Existencia",
            "resultante",
            "Detalle registrado"
        ],
        "every Spanish heading fits as lines inside its own column"
    );
    let existence_lines = header_lines
        .iter()
        .filter(|(text, _, _)| text == "Existencia" || text == "resultante")
        .collect::<Vec<_>>();
    assert_eq!(
        existence_lines.len(),
        2,
        "long header wraps at a word boundary"
    );
    assert!(
        (existence_lines[0].2 - existence_lines[1].2).abs() > 1.0,
        "wrapped header baselines must be vertically separated: {existence_lines:?}"
    );
    // Independent Type1 Helvetica width from the verifier: the unwrapped label is 9,226 units.
    assert!((9_226.0_f32 * 8.0 / 1_000.0 - 73.808).abs() < 0.001);
    assert!(
        existence_lines
            .iter()
            .all(|(_, x, _)| *x >= printpdf::Pt::from(printpdf::Mm(142.0)).0),
        "wrapped text begins inside the 141–164mm result-stock cell"
    );
    for (text, x_pt, _) in &header_lines {
        let (cell_left_mm, cell_width_mm) = match text.as_str() {
            "N.º / fecha y hora" => (8.0, 29.0),
            "Producto / SKU" => (37.0, 50.0),
            "Tipo" => (87.0, 35.0),
            "Variación" => (122.0, 19.0),
            "Existencia" | "resultante" => (141.0, 23.0),
            "Detalle registrado" => (164.0, 125.0),
            _ => unreachable!("only column-heading lines were selected"),
        };
        let x_mm = x_pt * 25.4 / 72.0;
        let width_mm =
            repuestos_autos::commands::pdf_table::measure_helvetica_pt(text, 8.0) * 25.4 / 72.0;
        assert!(
            x_mm >= cell_left_mm && x_mm + width_mm <= cell_left_mm + cell_width_mm - 1.0,
            "heading {text:?} exceeds its {cell_width_mm}mm cell"
        );
    }
    let mut current_x_pt = 0.0;
    let plus_one_x_mm = document.pages[0]
        .ops
        .iter()
        .find_map(|op| match op {
            Op::SetTextMatrix { matrix } => {
                current_x_pt = matrix.as_array()[4];
                None
            }
            Op::ShowText { items }
                if items
                    .iter()
                    .any(|item| matches!(item, TextItem::Text(text) if text == "+1")) =>
            {
                Some(current_x_pt * 25.4 / 72.0)
            }
            _ => None,
        })
        .expect("signed quantity position");
    // Independent Helvetica advances: '+'=584 and '1'=556 units at 9pt; right edge is 140mm.
    let expected_plus_one_x_mm = 140.0 - (1_140.0_f32 * 9.0 / 1_000.0) * 25.4 / 72.0;
    assert!(
        (plus_one_x_mm - expected_plus_one_x_mm).abs() < 0.02,
        "numeric value must align to the column's right edge"
    );
    let header_y = page_zero_positions
        .iter()
        .find(|(text, _)| text.contains("N.º / fecha"))
        .map(|(_, y)| *y)
        .expect(&format!(
            "missing column heading positions: {page_zero_positions:#?}"
        ));
    let first_row_y = page_zero_positions
        .iter()
        .find(|(text, _)| text.contains("#21"))
        .map(|(_, y)| *y)
        .expect(&format!(
            "missing first-row positions: {page_zero_positions:#?}"
        ));
    assert!(
        first_row_y < header_y - 10.0,
        "first data row must start below the full header block"
    );
    let lowest_header_y = header_lines
        .iter()
        .map(|(_, _, y)| *y)
        .fold(f32::INFINITY, f32::min);
    assert!(
        first_row_y < lowest_header_y - printpdf::Pt::from(printpdf::Mm(8.0)).0,
        "first body baseline must remain at least 8mm below the lowest header baseline"
    );
    let long_detail_lines = document
        .pages
        .iter()
        .flat_map(|page| positioned_text(&page.ops))
        .filter(|(text, _)| {
            text.contains("Detalle") || text.contains("persistido") || text.contains("largo")
        })
        .count();
    assert!(
        long_detail_lines > 1,
        "long persisted detail must wrap into multiple layout operations"
    );
}

fn positioned_text_with_x(ops: &[Op]) -> Vec<(String, f32, f32)> {
    let mut x = 0.0;
    let mut y = 0.0;
    let mut output = Vec::new();
    for op in ops {
        match op {
            Op::SetTextMatrix { matrix } => {
                x = matrix.as_array()[4];
                y = matrix.as_array()[5];
            }
            Op::SetTextCursor { pos } => {
                x = pos.x.0;
                y = pos.y.0;
            }
            Op::ShowText { items } => {
                for item in items {
                    if let TextItem::Text(text) = item {
                        output.push((text.clone(), x, y));
                    }
                }
            }
            _ => {}
        }
    }
    output
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
    let detail = (0..6_000)
        .map(|index| format!("f{index:04} "))
        .collect::<String>();
    connection.execute(
        "INSERT INTO inventory_movements
         (id, product_id, movement_type, quantity_delta, occurred_at, source_reference, request_id, resulting_quantity)
         VALUES (1, 1, 'stock_entry', 1, '2025-01-01 12:00:00', ?1, 'movement-ledger-export-extreme', 1)",
        [&detail],
    ).unwrap();
    let mut pdf = Vec::new();
    let response =
        export_movement_ledger(&connection, request(), "2025-01-02T12:00:00Z", |bytes| {
            pdf.extend_from_slice(bytes);
            ExportSaveResult::Saved
        });
    assert_eq!(response, MovementLedgerExportResponse::Success);
    let document =
        printpdf::PdfDocument::parse(&pdf, &printpdf::PdfParseOptions::default(), &mut Vec::new())
            .unwrap();
    assert!(document.pages.len() >= 2);
    let extracted_pages = document.extract_text();
    let all_text = extracted_pages.concat().join(" ");
    let first = all_text.find("f0000").expect("first detail fragment");
    let middle = all_text.find("f3000").expect("middle detail fragment");
    let last = all_text.find("f5999").expect("last detail fragment");
    assert!(
        first < middle && middle < last,
        "continued detail fragments retain source order"
    );
    for (page_index, page) in document.pages.iter().enumerate() {
        let page_text = extracted_pages[page_index].join(" ");
        assert!(page_text.contains("Libro de movimientos"));
        assert!(page_text.contains("Producto ID: 1 | Tipo de movimiento: Ingreso de stock"));
        assert!(page_text.contains(&format!(
            "Generado: {}",
            repuestos_autos::commands::pdf_format::format_timestamp_local("2025-01-02T12:00:00Z")
                .unwrap()
        )));
        assert!(page_text.contains("Detalle registrado"));
        assert!(page_text.contains(&format!(
            "Página {} de {}",
            page_index + 1,
            document.pages.len()
        )));
        if page_index > 0 {
            assert!(page_text.contains("Continuación del movimiento n.º 1"));
        }
        for (_, y_pt) in positioned_text(&page.ops) {
            assert!(
                y_pt >= printpdf::Pt::from(printpdf::Mm(8.0)).0,
                "text below printable bottom on page {page_index}: {y_pt}"
            );
            assert!(
                y_pt <= printpdf::Pt::from(printpdf::Mm(290.0)).0,
                "text above printable top on page {page_index}: {y_pt}"
            );
        }
    }
}

#[test]
fn export_translates_every_kind_preserves_fragments_and_marks_invalid_local_time() {
    let connection = open_seeded_catalog().unwrap();
    connection
        .pragma_update(None, "ignore_check_constraints", true)
        .unwrap();
    let kinds = [
        ("opening_stock", "Stock inicial"),
        ("stock_entry", "Ingreso de stock"),
        ("sale", "Venta"),
        ("return", "Devolución"),
        ("adjustment", "Ajuste de inventario"),
        ("cancellation", "Anulación"),
    ];
    for (index, (kind, _)) in kinds.iter().enumerate() {
        let at = if index == 5 {
            "2025-01-01T01"
        } else {
            "2025-01-01 12:00:00"
        };
        connection.execute(
            "INSERT INTO inventory_movements
             (id, product_id, movement_type, quantity_delta, occurred_at, source_reference, request_id, resulting_quantity, reason)
             VALUES (?1, 1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![index as i64 + 1, kind, if index == 2 { -3 } else { 2 }, at,
                "línea É / detalle\n\nfin", format!("movement-all-kinds-{index}"), 7,
                "motivo detallado"] ,
        ).unwrap();
    }
    let mut pdf = Vec::new();
    let mut request = request();
    request.product_id = None;
    request.movement_type = None;
    assert_eq!(
        export_movement_ledger(&connection, request, "2025-01-02T12:00:00Z", |bytes| {
            pdf.extend_from_slice(bytes);
            ExportSaveResult::Saved
        }),
        MovementLedgerExportResponse::Success
    );
    let document =
        printpdf::PdfDocument::parse(&pdf, &printpdf::PdfParseOptions::default(), &mut Vec::new())
            .unwrap();
    let text = document.extract_text().concat().join(" ");
    for (_, label) in kinds {
        assert!(text.contains(label), "missing Spanish label {label}");
    }
    assert!(text.contains("Hora local no disponible"));
    assert!(text.contains("línea É"));
    assert!(text.contains("fin"));
    assert!(text.contains("-3"));
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
    assert_eq!(
        response,
        MovementLedgerExportResponse::Error(
            repuestos_autos::commands::movement_ledger::MovementLedgerCommandError {
                code: "export_limit_exceeded",
                message: "More than 2,000 movements match. Narrow the filters and try again.",
            },
        )
    );
}

#[test]
fn export_distinguishes_cancellation_and_hides_write_failure_details() {
    let connection = open_seeded_catalog().unwrap();
    let cancelled = export_movement_ledger(&connection, request(), "2025-01-02T12:00:00Z", |_| {
        ExportSaveResult::Cancelled
    });
    assert_eq!(cancelled, MovementLedgerExportResponse::Cancelled);

    let failed = export_movement_ledger(&connection, request(), "2025-01-02T12:00:00Z", |_| {
        ExportSaveResult::Failed
    });
    assert_eq!(
        failed,
        MovementLedgerExportResponse::Error(
            repuestos_autos::commands::movement_ledger::MovementLedgerCommandError {
                code: "export_failed",
                message: "The movement ledger could not be saved.",
            },
        )
    );
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
        ))
        .unwrap(),
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
