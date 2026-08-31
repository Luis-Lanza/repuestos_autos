use rusqlite::Transaction;
use sha2::{Digest, Sha256};

use crate::domain::sales::{
    plan_cancellation, plan_return, CancellationPlan, CancellationPlanLine, OriginalSaleLine,
    PostSaleDomainError, RequestedReturnLine, ReturnPlan, ReturnPlanLine, SaleCorrectionState,
};
use crate::domain::RequestId;

const PAYLOAD_VERSION: i64 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PostSaleOperation {
    Return,
    Cancellation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaleLifecycleStatus {
    Confirmed,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersistedRequest {
    pub request_id: String,
    pub operation: PostSaleOperation,
    pub sale_id: i64,
    pub payload_version: i64,
    pub canonical_payload: Vec<u8>,
    pub payload_sha256: String,
}

impl PersistedRequest {
    pub fn matches(&self, other: &Self) -> bool {
        self.operation == other.operation
            && self.sale_id == other.sale_id
            && self.payload_version == other.payload_version
            && self.canonical_payload == other.canonical_payload
            && self.payload_sha256 == other.payload_sha256
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateReturnRequest {
    pub request_id: RequestId,
    pub sale_id: i64,
    pub lines: Vec<RequestedReturnLine>,
    persisted: PersistedRequest,
}

impl CreateReturnRequest {
    pub fn new(
        request_id: RequestId,
        sale_id: i64,
        mut lines: Vec<RequestedReturnLine>,
    ) -> Result<Self, PostSaleError> {
        if sale_id <= 0
            || lines.is_empty()
            || lines
                .iter()
                .any(|line| line.sale_line_id <= 0 || line.quantity <= 0)
        {
            return Err(PostSaleError::InvalidRequest);
        }
        lines.sort_by_key(|line| line.sale_line_id);
        if lines
            .windows(2)
            .any(|pair| pair[0].sale_line_id == pair[1].sale_line_id)
        {
            return Err(PostSaleError::Domain(
                PostSaleDomainError::DuplicateSaleLine,
            ));
        }
        let mut bytes = format!("post-sale/v1\nreturn\n{sale_id}\n{}\n", lines.len()).into_bytes();
        for line in &lines {
            bytes
                .extend_from_slice(format!("{}:{}\n", line.sale_line_id, line.quantity).as_bytes());
        }
        Ok(Self {
            persisted: identity(&request_id, PostSaleOperation::Return, sale_id, bytes),
            request_id,
            sale_id,
            lines,
        })
    }

    pub fn persisted_request(&self) -> &PersistedRequest {
        &self.persisted
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CancelSaleRequest {
    pub request_id: RequestId,
    pub sale_id: i64,
    pub reason: String,
    persisted: PersistedRequest,
}

impl CancelSaleRequest {
    pub fn new(request_id: RequestId, sale_id: i64, reason: String) -> Result<Self, PostSaleError> {
        if sale_id <= 0 {
            return Err(PostSaleError::InvalidRequest);
        }
        let reason = reason.trim().to_owned();
        if reason.is_empty() {
            return Err(PostSaleError::Domain(
                PostSaleDomainError::CancellationReasonRequired,
            ));
        }
        let bytes = format!(
            "post-sale/v1\ncancellation\n{sale_id}\n{}:{reason}",
            reason.len()
        )
        .into_bytes();
        Ok(Self {
            persisted: identity(&request_id, PostSaleOperation::Cancellation, sale_id, bytes),
            request_id,
            sale_id,
            reason,
        })
    }

    pub fn persisted_request(&self) -> &PersistedRequest {
        &self.persisted
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReturnResult {
    pub request_id: String,
    pub return_id: i64,
    pub sale_id: i64,
    pub status: SaleLifecycleStatus,
    pub occurred_at: String,
    pub lines: Vec<ReturnPlanLine>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CancellationResult {
    pub request_id: String,
    pub cancellation_id: i64,
    pub sale_id: i64,
    pub status: SaleLifecycleStatus,
    pub occurred_at: String,
    pub reason: String,
    pub lines: Vec<CancellationPlanLine>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostSaleFacts {
    pub state: SaleCorrectionState,
    pub original_lines: Vec<OriginalSaleLine>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PostSaleError {
    InvalidRequest,
    Domain(PostSaleDomainError),
    RequestConflict,
    PersistenceFailure,
}

pub trait PostSaleRepository {
    fn find_request(
        &self,
        transaction: &Transaction<'_>,
        request_id: &str,
    ) -> Result<Option<PersistedRequest>, PostSaleError>;
    fn load_facts(
        &self,
        transaction: &Transaction<'_>,
        sale_id: i64,
    ) -> Result<PostSaleFacts, PostSaleError>;
    fn load_return_result(
        &self,
        transaction: &Transaction<'_>,
        request: &PersistedRequest,
    ) -> Result<ReturnResult, PostSaleError>;
    fn load_cancellation_result(
        &self,
        transaction: &Transaction<'_>,
        request: &PersistedRequest,
    ) -> Result<CancellationResult, PostSaleError>;
    fn persist_return(
        &self,
        transaction: &Transaction<'_>,
        request: &PersistedRequest,
        plan: &ReturnPlan,
    ) -> Result<ReturnResult, PostSaleError>;
    fn persist_cancellation(
        &self,
        transaction: &Transaction<'_>,
        request: &PersistedRequest,
        plan: &CancellationPlan,
    ) -> Result<CancellationResult, PostSaleError>;
}

pub trait PostSaleTransaction {
    fn repository_transaction(&self) -> &Transaction<'_>;
    fn commit(&mut self) -> Result<(), PostSaleError>;
    fn rollback(&mut self) -> Result<(), PostSaleError>;
}

pub trait PostSaleTransactionFactory {
    type Transaction<'transaction>: PostSaleTransaction
    where
        Self: 'transaction;

    fn begin_immediate(&mut self) -> Result<Self::Transaction<'_>, PostSaleError>;
}

pub trait PostSaleLifecycleUseCase {
    fn create_return(
        &mut self,
        request: CreateReturnRequest,
    ) -> Result<ReturnResult, PostSaleError>;
    fn cancel_sale(
        &mut self,
        request: CancelSaleRequest,
    ) -> Result<CancellationResult, PostSaleError>;
}

pub struct PostSaleUseCase<'transactions, 'repository, Transactions, Repository> {
    transactions: &'transactions mut Transactions,
    repository: &'repository Repository,
}

impl<'transactions, 'repository, Transactions, Repository>
    PostSaleUseCase<'transactions, 'repository, Transactions, Repository>
where
    Transactions: PostSaleTransactionFactory,
    Repository: PostSaleRepository,
{
    pub fn new(
        transactions: &'transactions mut Transactions,
        repository: &'repository Repository,
    ) -> Self {
        Self {
            transactions,
            repository,
        }
    }
}

impl<Transactions, Repository> PostSaleLifecycleUseCase
    for PostSaleUseCase<'_, '_, Transactions, Repository>
where
    Transactions: PostSaleTransactionFactory,
    Repository: PostSaleRepository,
{
    fn create_return(
        &mut self,
        request: CreateReturnRequest,
    ) -> Result<ReturnResult, PostSaleError> {
        let transaction = self.transactions.begin_immediate()?;
        let outcome = return_in(
            transaction.repository_transaction(),
            self.repository,
            request,
        );
        finish(transaction, outcome)
    }

    fn cancel_sale(
        &mut self,
        request: CancelSaleRequest,
    ) -> Result<CancellationResult, PostSaleError> {
        let transaction = self.transactions.begin_immediate()?;
        let outcome = cancel_in(
            transaction.repository_transaction(),
            self.repository,
            request,
        );
        finish(transaction, outcome)
    }
}

fn finish<T: PostSaleTransaction, ResultType>(
    mut transaction: T,
    outcome: Result<ResultType, PostSaleError>,
) -> Result<ResultType, PostSaleError> {
    match outcome {
        Ok(result) => match transaction.commit() {
            Ok(()) => Ok(result),
            Err(_) => transaction
                .rollback()
                .map(|_| Err(PostSaleError::PersistenceFailure))
                .map_err(|_| PostSaleError::PersistenceFailure)?,
        },
        Err(error) => transaction
            .rollback()
            .map(|_| Err(error))
            .map_err(|_| PostSaleError::PersistenceFailure)?,
    }
}

fn return_in(
    transaction: &Transaction<'_>,
    repository: &impl PostSaleRepository,
    request: CreateReturnRequest,
) -> Result<ReturnResult, PostSaleError> {
    let persisted = request.persisted_request();
    if let Some(result) = replay(transaction, repository, persisted, |existing| {
        repository.load_return_result(transaction, existing)
    })? {
        return Ok(result);
    }
    let facts = repository.load_facts(transaction, request.sale_id)?;
    let plan = plan_return(facts.state, &facts.original_lines, &request.lines)
        .map_err(PostSaleError::Domain)?;
    repository.persist_return(transaction, persisted, &plan)
}

fn cancel_in(
    transaction: &Transaction<'_>,
    repository: &impl PostSaleRepository,
    request: CancelSaleRequest,
) -> Result<CancellationResult, PostSaleError> {
    let persisted = request.persisted_request();
    if let Some(result) = replay(transaction, repository, persisted, |existing| {
        repository.load_cancellation_result(transaction, existing)
    })? {
        return Ok(result);
    }
    let facts = repository.load_facts(transaction, request.sale_id)?;
    let plan = plan_cancellation(facts.state, &facts.original_lines, &request.reason)
        .map_err(PostSaleError::Domain)?;
    repository.persist_cancellation(transaction, persisted, &plan)
}

fn replay<ResultType>(
    transaction: &Transaction<'_>,
    repository: &impl PostSaleRepository,
    request: &PersistedRequest,
    load: impl FnOnce(&PersistedRequest) -> Result<ResultType, PostSaleError>,
) -> Result<Option<ResultType>, PostSaleError> {
    match repository.find_request(transaction, &request.request_id)? {
        Some(existing) if existing.matches(request) => load(&existing).map(Some),
        Some(_) => Err(PostSaleError::RequestConflict),
        None => Ok(None),
    }
}

fn identity(
    request_id: &RequestId,
    operation: PostSaleOperation,
    sale_id: i64,
    canonical_payload: Vec<u8>,
) -> PersistedRequest {
    PersistedRequest {
        request_id: request_id.as_uuid().to_string(),
        operation,
        sale_id,
        payload_version: PAYLOAD_VERSION,
        payload_sha256: format!("{:x}", Sha256::digest(&canonical_payload)),
        canonical_payload,
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::*;
    use rusqlite::{Connection, TransactionBehavior};

    fn id() -> RequestId {
        RequestId::parse("550E8400-E29B-41D4-A716-446655440001").unwrap()
    }
    fn line(id: i64, quantity: i64) -> RequestedReturnLine {
        RequestedReturnLine {
            sale_line_id: id,
            quantity,
        }
    }

    #[test]
    fn return_identity_orders_lines_and_normalizes_the_request_id() {
        let left = CreateReturnRequest::new(id(), 9, vec![line(2, 1), line(1, 3)]).unwrap();
        let right = CreateReturnRequest::new(id(), 9, vec![line(1, 3), line(2, 1)]).unwrap();
        assert_eq!(left.lines, vec![line(1, 3), line(2, 1)]);
        assert_eq!(left.persisted_request(), right.persisted_request());
        assert_eq!(
            left.persisted_request().request_id,
            "550e8400-e29b-41d4-a716-446655440001"
        );
    }

    #[test]
    fn identity_detects_changed_payload_facts_and_hashes() {
        let base = CreateReturnRequest::new(id(), 9, vec![line(1, 3)]).unwrap();
        let quantity = CreateReturnRequest::new(id(), 9, vec![line(1, 2)]).unwrap();
        let sale = CreateReturnRequest::new(id(), 10, vec![line(1, 3)]).unwrap();
        let cancellation = CancelSaleRequest::new(id(), 9, " x ".into()).unwrap();
        let reason = CancelSaleRequest::new(id(), 9, "y".into()).unwrap();
        let mut bad_hash = base.persisted_request().clone();
        bad_hash.payload_sha256 = "0".repeat(64);
        assert!(!base
            .persisted_request()
            .matches(quantity.persisted_request()));
        assert!(!base.persisted_request().matches(sale.persisted_request()));
        assert!(!base
            .persisted_request()
            .matches(cancellation.persisted_request()));
        assert!(!cancellation
            .persisted_request()
            .matches(reason.persisted_request()));
        assert!(!base.persisted_request().matches(&bad_hash));
        assert_eq!(cancellation.reason, "x");
    }

    #[derive(Clone, Copy)]
    enum Completion {
        Commit,
        OperationFails,
        CommitFails,
        CommitAndRollbackFail,
    }

    #[derive(Default)]
    struct Lifecycle {
        begins: usize,
        commits: usize,
        rollbacks: usize,
    }

    struct FakeTransactionFactory {
        connection: Connection,
        completion: Completion,
        lifecycle: Rc<RefCell<Lifecycle>>,
    }

    struct FakeTransaction<'connection> {
        transaction: Transaction<'connection>,
        completion: Completion,
        lifecycle: Rc<RefCell<Lifecycle>>,
    }

    impl PostSaleTransaction for FakeTransaction<'_> {
        fn repository_transaction(&self) -> &Transaction<'_> {
            &self.transaction
        }

        fn commit(&mut self) -> Result<(), PostSaleError> {
            self.lifecycle.borrow_mut().commits += 1;
            match self.completion {
                Completion::Commit | Completion::OperationFails => Ok(()),
                Completion::CommitFails | Completion::CommitAndRollbackFail => {
                    Err(PostSaleError::PersistenceFailure)
                }
            }
        }

        fn rollback(&mut self) -> Result<(), PostSaleError> {
            self.lifecycle.borrow_mut().rollbacks += 1;
            match self.completion {
                Completion::CommitAndRollbackFail => Err(PostSaleError::PersistenceFailure),
                _ => Ok(()),
            }
        }
    }

    impl PostSaleTransactionFactory for FakeTransactionFactory {
        type Transaction<'connection> = FakeTransaction<'connection>;

        fn begin_immediate(&mut self) -> Result<Self::Transaction<'_>, PostSaleError> {
            self.lifecycle.borrow_mut().begins += 1;
            let transaction = self
                .connection
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(|_| PostSaleError::PersistenceFailure)?;
            Ok(FakeTransaction {
                transaction,
                completion: self.completion,
                lifecycle: Rc::clone(&self.lifecycle),
            })
        }
    }

    struct FakePostSaleRepository {
        completion: Completion,
    }

    impl PostSaleRepository for FakePostSaleRepository {
        fn find_request(
            &self,
            _: &Transaction<'_>,
            _: &str,
        ) -> Result<Option<PersistedRequest>, PostSaleError> {
            Ok(None)
        }

        fn load_facts(&self, _: &Transaction<'_>, _: i64) -> Result<PostSaleFacts, PostSaleError> {
            Ok(PostSaleFacts {
                state: SaleCorrectionState::Confirmed,
                original_lines: vec![OriginalSaleLine {
                    sale_line_id: 1,
                    product_id: 1,
                    sold_quantity: 2,
                    returned_quantity: 0,
                }],
            })
        }

        fn load_return_result(
            &self,
            _: &Transaction<'_>,
            _: &PersistedRequest,
        ) -> Result<ReturnResult, PostSaleError> {
            Err(PostSaleError::PersistenceFailure)
        }

        fn load_cancellation_result(
            &self,
            _: &Transaction<'_>,
            _: &PersistedRequest,
        ) -> Result<CancellationResult, PostSaleError> {
            Err(PostSaleError::PersistenceFailure)
        }

        fn persist_return(
            &self,
            _: &Transaction<'_>,
            request: &PersistedRequest,
            plan: &ReturnPlan,
        ) -> Result<ReturnResult, PostSaleError> {
            if matches!(self.completion, Completion::OperationFails) {
                return Err(PostSaleError::Domain(PostSaleDomainError::SaleCancelled));
            }
            Ok(ReturnResult {
                request_id: request.request_id.clone(),
                return_id: 1,
                sale_id: request.sale_id,
                status: SaleLifecycleStatus::Confirmed,
                occurred_at: "now".into(),
                lines: plan.lines.clone(),
            })
        }

        fn persist_cancellation(
            &self,
            _: &Transaction<'_>,
            _: &PersistedRequest,
            _: &CancellationPlan,
        ) -> Result<CancellationResult, PostSaleError> {
            Err(PostSaleError::PersistenceFailure)
        }
    }

    fn return_request() -> CreateReturnRequest {
        CreateReturnRequest::new(id(), 1, vec![line(1, 1)]).unwrap()
    }

    fn factory(completion: Completion) -> (FakeTransactionFactory, Rc<RefCell<Lifecycle>>) {
        let lifecycle = Rc::new(RefCell::new(Lifecycle::default()));
        (
            FakeTransactionFactory {
                connection: Connection::open_in_memory().unwrap(),
                completion,
                lifecycle: Rc::clone(&lifecycle),
            },
            lifecycle,
        )
    }

    #[test]
    fn return_use_case_commits_successful_work() {
        let (mut transactions, lifecycle) = factory(Completion::Commit);
        let repository = FakePostSaleRepository {
            completion: Completion::Commit,
        };
        let result = PostSaleUseCase::new(&mut transactions, &repository)
            .create_return(return_request())
            .unwrap();
        assert_eq!(result.return_id, 1);
        let lifecycle = lifecycle.borrow();
        assert_eq!(
            (lifecycle.begins, lifecycle.commits, lifecycle.rollbacks),
            (1, 1, 0)
        );
    }

    #[test]
    fn return_use_case_rolls_back_operation_failures() {
        let (mut transactions, lifecycle) = factory(Completion::OperationFails);
        let repository = FakePostSaleRepository {
            completion: Completion::OperationFails,
        };
        let error = PostSaleUseCase::new(&mut transactions, &repository)
            .create_return(return_request())
            .unwrap_err();
        assert_eq!(
            error,
            PostSaleError::Domain(PostSaleDomainError::SaleCancelled)
        );
        assert_eq!(lifecycle.borrow().rollbacks, 1);
    }

    #[test]
    fn return_use_case_rolls_back_commit_failures() {
        let (mut transactions, lifecycle) = factory(Completion::CommitFails);
        let repository = FakePostSaleRepository {
            completion: Completion::CommitFails,
        };
        let error = PostSaleUseCase::new(&mut transactions, &repository)
            .create_return(return_request())
            .unwrap_err();
        assert_eq!(error, PostSaleError::PersistenceFailure);
        assert_eq!(
            (lifecycle.borrow().commits, lifecycle.borrow().rollbacks),
            (1, 1)
        );
    }

    #[test]
    fn return_use_case_maps_commit_and_rollback_failures_to_persistence_failure() {
        let (mut transactions, lifecycle) = factory(Completion::CommitAndRollbackFail);
        let repository = FakePostSaleRepository {
            completion: Completion::CommitAndRollbackFail,
        };
        let error = PostSaleUseCase::new(&mut transactions, &repository)
            .create_return(return_request())
            .unwrap_err();
        assert_eq!(error, PostSaleError::PersistenceFailure);
        assert_eq!(
            (lifecycle.borrow().commits, lifecycle.borrow().rollbacks),
            (1, 1)
        );
    }
}
