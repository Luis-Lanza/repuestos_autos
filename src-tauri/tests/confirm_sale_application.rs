use repuestos_autos::application::sales::{
    ApplicationConfirmSaleRequest, ApplicationRequestedLine, ConfirmSaleError,
    ConfirmSaleRepository, ConfirmSaleUseCase, PersistedLine, PersistedSaleSummary, Reservation,
};
use repuestos_autos::domain::sales::{Payment, PaymentInput, SaleLine};
use repuestos_autos::domain::{MoneyCentavos, Quantity, RequestId};
use rusqlite::{Connection, Transaction};
use std::cell::RefCell;
struct RepositoryDouble {
    calls: RefCell<Vec<&'static str>>,
    write_marker_on_reserve: bool,
    reservation: Result<Reservation, ConfirmSaleError>,
    resolution: Result<Vec<SaleLine>, ConfirmSaleError>,
    persistence: Result<PersistedSaleSummary, ConfirmSaleError>,
}
type IdentityFields = (Option<String>, Option<i64>, Option<Vec<u8>>, Option<String>);

fn complete_reservation(
    reservation: Reservation,
    operation_kind: &str,
    payload_version: i64,
    canonical_payload: &[u8],
    payload_sha256: &str,
) -> Reservation {
    let generated = (
        Some(operation_kind.into()),
        Some(payload_version),
        Some(canonical_payload.to_vec()),
        Some(payload_sha256.into()),
    );
    let use_generated = |identity: IdentityFields| {
        if identity.0.is_none()
            && identity.1.is_none()
            && identity.2.is_none()
            && identity.3.is_none()
        {
            generated.clone()
        } else {
            identity
        }
    };
    match reservation {
        Reservation::Reserved => Reservation::Reserved,
        Reservation::ExistingConfirmed {
            summary,
            operation_kind,
            payload_version,
            canonical_payload,
            payload_sha256,
        } => {
            let (operation_kind, payload_version, canonical_payload, payload_sha256) =
                use_generated((
                    operation_kind,
                    payload_version,
                    canonical_payload,
                    payload_sha256,
                ));
            Reservation::ExistingConfirmed {
                summary,
                operation_kind,
                payload_version,
                canonical_payload,
                payload_sha256,
            }
        }
        Reservation::ExistingIncomplete {
            operation_kind,
            payload_version,
            canonical_payload,
            payload_sha256,
        } => {
            let (operation_kind, payload_version, canonical_payload, payload_sha256) =
                use_generated((
                    operation_kind,
                    payload_version,
                    canonical_payload,
                    payload_sha256,
                ));
            Reservation::ExistingIncomplete {
                operation_kind,
                payload_version,
                canonical_payload,
                payload_sha256,
            }
        }
        Reservation::ExistingCorrupt {
            operation_kind,
            payload_version,
            canonical_payload,
            payload_sha256,
        } => {
            let (operation_kind, payload_version, canonical_payload, payload_sha256) =
                use_generated((
                    operation_kind,
                    payload_version,
                    canonical_payload,
                    payload_sha256,
                ));
            Reservation::ExistingCorrupt {
                operation_kind,
                payload_version,
                canonical_payload,
                payload_sha256,
            }
        }
    }
}

impl ConfirmSaleRepository for RepositoryDouble {
    fn reserve_or_load(
        &self,
        transaction: &Transaction<'_>,
        _: &RequestId,
        operation_kind: &str,
        payload_version: i64,
        canonical_payload: &[u8],
        payload_sha256: &str,
    ) -> Result<Reservation, ConfirmSaleError> {
        self.calls.borrow_mut().push("reserve");
        if self.write_marker_on_reserve {
            transaction
                .execute("INSERT INTO rollback_markers DEFAULT VALUES", [])
                .map_err(|_| ConfirmSaleError::Persistence)?;
        }
        self.reservation.clone().map(|reservation| {
            complete_reservation(
                reservation,
                operation_kind,
                payload_version,
                canonical_payload,
                payload_sha256,
            )
        })
    }
    fn resolve_lines(
        &self,
        _: &Transaction<'_>,
        _: &[ApplicationRequestedLine],
    ) -> Result<Vec<SaleLine>, ConfirmSaleError> {
        self.calls.borrow_mut().push("resolve");
        self.resolution.clone()
    }
    fn persist_confirmed(
        &self,
        _: &Transaction<'_>,
        _: &RequestId,
        _: &repuestos_autos::domain::sales::Sale,
    ) -> Result<PersistedSaleSummary, ConfirmSaleError> {
        self.calls.borrow_mut().push("persist");
        self.persistence.clone()
    }
}
fn money(value: i64) -> MoneyCentavos {
    MoneyCentavos::new(value).unwrap()
}
fn request(lines: Vec<ApplicationRequestedLine>) -> ApplicationConfirmSaleRequest {
    ApplicationConfirmSaleRequest {
        request_id: RequestId::parse("550e8400-e29b-41d4-a716-446655440100").unwrap(),
        lines,
        payment: PaymentInput {
            amount_tendered: Some(money(3_000)),
            qr_applied: None,
        },
    }
}
fn requested_line(product_id: i64) -> ApplicationRequestedLine {
    ApplicationRequestedLine {
        product_id,
        quantity: Quantity::new(1).unwrap(),
        captured_unit_price: money(2_500),
        captured_revision: 0,
        acknowledged_price: None,
        acknowledged_revision: None,
    }
}
fn summary() -> PersistedSaleSummary {
    PersistedSaleSummary {
        sale_id: 1,
        request_id: RequestId::parse("550e8400-e29b-41d4-a716-446655440100").unwrap(),
        status: "confirmed".into(),
        confirmed_at: "2025-01-01T00:00:00Z".into(),
        lines: vec![PersistedLine {
            product_id: 1,
            sku: "SKU-1".into(),
            product_name: "Product".into(),
            quantity: Quantity::new(1).unwrap(),
            negotiated_unit_price: money(2_500),
            minimum_unit_price_snapshot: money(2_500),
            line_total: money(2_500),
        }],
        payments: vec![Payment::cash(money(2_500), money(3_000), money(500)).unwrap()],
        total: money(2_500),
    }
}
fn empty_identity() -> IdentityFields {
    (None, None, None, None)
}

fn existing_confirmed(summary: PersistedSaleSummary) -> Reservation {
    let (operation_kind, payload_version, canonical_payload, payload_sha256) = empty_identity();
    Reservation::ExistingConfirmed {
        summary: Some(summary),
        operation_kind,
        payload_version,
        canonical_payload,
        payload_sha256,
    }
}

fn existing_incomplete() -> Reservation {
    let (operation_kind, payload_version, canonical_payload, payload_sha256) = empty_identity();
    Reservation::ExistingIncomplete {
        operation_kind,
        payload_version,
        canonical_payload,
        payload_sha256,
    }
}

fn existing_corrupt() -> Reservation {
    let (operation_kind, payload_version, canonical_payload, payload_sha256) = empty_identity();
    Reservation::ExistingCorrupt {
        operation_kind,
        payload_version,
        canonical_payload,
        payload_sha256,
    }
}

fn resolved_line() -> SaleLine {
    SaleLine::priced(1, Quantity::new(1).unwrap(), money(2_500)).unwrap()
}
fn repository_double(reservation: Result<Reservation, ConfirmSaleError>) -> RepositoryDouble {
    RepositoryDouble {
        calls: RefCell::new(Vec::new()),
        write_marker_on_reserve: false,
        reservation,
        resolution: Ok(vec![resolved_line()]),
        persistence: Ok(summary()),
    }
}

fn assert_failure_rolls_back(
    mut repository: RepositoryDouble,
    request: ApplicationConfirmSaleRequest,
    expected: ConfirmSaleError,
    calls: &[&'static str],
) {
    let mut connection = Connection::open_in_memory().unwrap();
    connection
        .execute("CREATE TABLE rollback_markers (id INTEGER PRIMARY KEY)", [])
        .unwrap();
    repository.write_marker_on_reserve = true;

    let result = ConfirmSaleUseCase::new(&mut connection, &repository).confirm(request);

    assert_eq!(result, Err(expected));
    assert_eq!(*repository.calls.borrow(), calls);
    let persisted_markers: i64 = connection
        .query_row("SELECT COUNT(*) FROM rollback_markers", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(persisted_markers, 0);
}
#[test]
fn existing_confirmation_short_circuits_before_line_resolution_or_payment_derivation() {
    let mut connection = Connection::open_in_memory().unwrap();
    let persisted = summary();
    let repository = repository_double(Ok(existing_confirmed(persisted.clone())));
    let result = ConfirmSaleUseCase::new(&mut connection, &repository)
        .confirm(request(vec![requested_line(999)]))
        .unwrap();
    assert_eq!(result, persisted);
    assert_eq!(*repository.calls.borrow(), ["reserve"]);
}
#[test]
fn confirmation_reserves_then_resolves_then_persists_derived_payment_facts() {
    let mut connection = Connection::open_in_memory().unwrap();
    let repository = repository_double(Ok(Reservation::Reserved));
    ConfirmSaleUseCase::new(&mut connection, &repository)
        .confirm(request(vec![requested_line(1)]))
        .unwrap();
    assert_eq!(
        *repository.calls.borrow(),
        ["reserve", "resolve", "persist"]
    );
}
#[test]
fn confirmed_retry_returns_stored_facts_before_validating_duplicate_products() {
    let mut connection = Connection::open_in_memory().unwrap();
    let persisted = summary();
    let repository = repository_double(Ok(existing_confirmed(persisted.clone())));
    let result = ConfirmSaleUseCase::new(&mut connection, &repository)
        .confirm(request(vec![requested_line(1), requested_line(1)]))
        .unwrap();
    assert_eq!(result, persisted);
    assert_eq!(*repository.calls.borrow(), ["reserve"]);
}
#[test]
fn newly_reserved_duplicate_products_are_rejected_before_line_resolution() {
    let mut connection = Connection::open_in_memory().unwrap();
    let repository = repository_double(Ok(Reservation::Reserved));
    let result = ConfirmSaleUseCase::new(&mut connection, &repository)
        .confirm(request(vec![requested_line(1), requested_line(1)]));
    assert_eq!(result, Err(ConfirmSaleError::DuplicateProduct));
    assert_eq!(*repository.calls.borrow(), ["reserve"]);
}
#[test]
fn resolution_and_persistence_failures_stop_later_repository_calls() {
    let mut connection = Connection::open_in_memory().unwrap();
    let mut repository = repository_double(Ok(Reservation::Reserved));
    repository.resolution = Err(ConfirmSaleError::ProductMissing);
    let result = ConfirmSaleUseCase::new(&mut connection, &repository)
        .confirm(request(vec![requested_line(1)]));
    assert_eq!(result, Err(ConfirmSaleError::ProductMissing));
    assert_eq!(*repository.calls.borrow(), ["reserve", "resolve"]);
    assert!(ConfirmSaleUseCase::new(
        &mut connection,
        &repository_double(Ok(Reservation::Reserved)),
    )
    .confirm(request(vec![requested_line(1)]))
    .is_ok());
}
#[test]
fn missing_or_inactive_products_stop_before_payment_derivation_and_persistence() {
    let mut connection = Connection::open_in_memory().unwrap();
    for error in [
        ConfirmSaleError::ProductMissing,
        ConfirmSaleError::ProductInactive,
    ] {
        let mut repository = repository_double(Ok(Reservation::Reserved));
        repository.resolution = Err(error.clone());
        let result = ConfirmSaleUseCase::new(&mut connection, &repository)
            .confirm(request(vec![requested_line(1)]));
        assert_eq!(result, Err(error));
        assert_eq!(*repository.calls.borrow(), ["reserve", "resolve"]);
    }
}
#[test]
fn invalid_payment_stops_before_persistence() {
    let mut connection = Connection::open_in_memory().unwrap();
    let repository = repository_double(Ok(Reservation::Reserved));
    let mut invalid_payment_request = request(vec![requested_line(1)]);
    invalid_payment_request.payment = PaymentInput {
        amount_tendered: None,
        qr_applied: Some(money(2_501)),
    };
    let result =
        ConfirmSaleUseCase::new(&mut connection, &repository).confirm(invalid_payment_request);
    assert_eq!(result, Err(ConfirmSaleError::QrExceedsTotal));
    assert_eq!(*repository.calls.borrow(), ["reserve", "resolve"]);
}
#[test]
fn incomplete_reservation_stops_before_resolution() {
    let mut connection = Connection::open_in_memory().unwrap();
    let repository = repository_double(Ok(existing_incomplete()));
    let result = ConfirmSaleUseCase::new(&mut connection, &repository)
        .confirm(request(vec![requested_line(1)]));
    assert_eq!(result, Err(ConfirmSaleError::Persistence));
    assert_eq!(*repository.calls.borrow(), ["reserve"]);
}

#[test]
fn corrupt_reservation_stops_before_resolution() {
    let mut connection = Connection::open_in_memory().unwrap();
    let repository = repository_double(Ok(existing_corrupt()));
    let result = ConfirmSaleUseCase::new(&mut connection, &repository)
        .confirm(request(vec![requested_line(1)]));
    assert_eq!(result, Err(ConfirmSaleError::Persistence));
    assert_eq!(*repository.calls.borrow(), ["reserve"]);
}
#[test]
fn application_failures_stop_in_order_and_roll_back_the_reservation() {
    let valid_request = || request(vec![requested_line(1)]);

    let mut invalid_payment = valid_request();
    invalid_payment.payment = PaymentInput {
        amount_tendered: None,
        qr_applied: Some(money(2_501)),
    };
    assert_failure_rolls_back(
        repository_double(Ok(Reservation::Reserved)),
        invalid_payment,
        ConfirmSaleError::QrExceedsTotal,
        &["reserve", "resolve"],
    );

    for error in [
        ConfirmSaleError::ProductMissing,
        ConfirmSaleError::ProductInactive,
    ] {
        let mut repository = repository_double(Ok(Reservation::Reserved));
        repository.resolution = Err(error.clone());
        assert_failure_rolls_back(repository, valid_request(), error, &["reserve", "resolve"]);
    }

    for reservation in [existing_incomplete(), existing_corrupt()] {
        assert_failure_rolls_back(
            repository_double(Ok(reservation)),
            valid_request(),
            ConfirmSaleError::Persistence,
            &["reserve"],
        );
    }

    let mut persistence_failure = repository_double(Ok(Reservation::Reserved));
    persistence_failure.persistence = Err(ConfirmSaleError::Persistence);
    assert_failure_rolls_back(
        persistence_failure,
        valid_request(),
        ConfirmSaleError::Persistence,
        &["reserve", "resolve", "persist"],
    );
}

#[test]
fn application_rejects_an_existing_identity_mismatch_before_replay() {
    let mut connection = Connection::open_in_memory().unwrap();
    let repository = repository_double(Ok(Reservation::ExistingConfirmed {
        summary: Some(summary()),
        operation_kind: Some("confirm_sale".into()),
        payload_version: Some(1),
        canonical_payload: Some(
            b"15:confirm_sale/v11:11:21:14:25001:04:null4:null5:value4:30004:null".to_vec(),
        ),
        payload_sha256: Some(
            "da12ce94ede784b9510eb0cfde85a0b5d89c2280ddf2b05f34069c1b8c465868".into(),
        ),
    }));

    let result = ConfirmSaleUseCase::new(&mut connection, &repository)
        .confirm(request(vec![requested_line(1)]));

    assert_eq!(result, Err(ConfirmSaleError::RequestConflict));
    assert_eq!(*repository.calls.borrow(), ["reserve"]);
}

#[test]
fn stale_catalog_price_rolls_back_the_reserved_sale_before_any_fact_is_persisted() {
    let mut repository = repository_double(Ok(Reservation::Reserved));
    repository.resolution = Err(ConfirmSaleError::StaleCatalogPrice {
        product_id: 1,
        current_unit_price: money(2_700),
        current_revision: 1,
    });
    assert_failure_rolls_back(
        repository,
        request(vec![requested_line(1)]),
        ConfirmSaleError::StaleCatalogPrice {
            product_id: 1,
            current_unit_price: money(2_700),
            current_revision: 1,
        },
        &["reserve", "resolve"],
    );
}
