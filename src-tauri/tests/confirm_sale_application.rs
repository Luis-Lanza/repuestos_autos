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
impl ConfirmSaleRepository for RepositoryDouble {
    fn reserve_or_load(
        &self,
        transaction: &Transaction<'_>,
        _: &RequestId,
    ) -> Result<Reservation, ConfirmSaleError> {
        self.calls.borrow_mut().push("reserve");
        if self.write_marker_on_reserve {
            transaction
                .execute("INSERT INTO rollback_markers DEFAULT VALUES", [])
                .map_err(|_| ConfirmSaleError::Persistence)?;
        }
        self.reservation.clone()
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
    let repository = repository_double(Ok(Reservation::ExistingConfirmed(persisted.clone())));
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
fn duplicate_products_are_rejected_without_opening_a_repository_operation() {
    let mut connection = Connection::open_in_memory().unwrap();
    let repository = repository_double(Ok(Reservation::Reserved));
    let result = ConfirmSaleUseCase::new(&mut connection, &repository)
        .confirm(request(vec![requested_line(1), requested_line(1)]));
    assert_eq!(result, Err(ConfirmSaleError::DuplicateProduct));
    assert!(repository.calls.borrow().is_empty());
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
    let repository = repository_double(Ok(Reservation::ExistingIncomplete));
    let result = ConfirmSaleUseCase::new(&mut connection, &repository)
        .confirm(request(vec![requested_line(1)]));
    assert_eq!(result, Err(ConfirmSaleError::Persistence));
    assert_eq!(*repository.calls.borrow(), ["reserve"]);
}

#[test]
fn corrupt_reservation_stops_before_resolution() {
    let mut connection = Connection::open_in_memory().unwrap();
    let repository = repository_double(Ok(Reservation::ExistingCorrupt));
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

    for reservation in [
        Reservation::ExistingIncomplete,
        Reservation::ExistingCorrupt,
    ] {
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
