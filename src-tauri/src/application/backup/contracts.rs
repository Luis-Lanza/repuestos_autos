use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationalFacts {
    pub catalog_records: u64,
    pub confirmed_sales: u64,
    pub stock_balances: u64,
    pub movement_records: u64,
    pub schema_history_version: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RestoreCandidate {
    pub stage: PathBuf,
    pub schema_version: i64,
    pub size_bytes: u64,
    pub sha256: String,
    pub facts: OperationalFacts,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RestoreSummary {
    pub schema_version: i64,
    pub size_bytes: u64,
    pub facts: OperationalFacts,
}

impl From<&RestoreCandidate> for RestoreSummary {
    fn from(candidate: &RestoreCandidate) -> Self {
        Self {
            schema_version: candidate.schema_version,
            size_bytes: candidate.size_bytes,
            facts: candidate.facts.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RestoreError {
    InvalidBackup,
    UnsupportedSchema,
    ConfirmationRequired,
    TokenInvalid,
    TokenExpired,
    RestoreFailed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrepareRestoreResult {
    Prepared {
        token: String,
        summary: RestoreSummary,
    },
    Failed(RestoreError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfirmRestoreResult {
    ReadyForReplacement { candidate: RestoreCandidate },
    Failed(RestoreError),
}

pub trait RestoreCandidateStore {
    fn prepare(&self, source: &Path) -> Result<RestoreCandidate, RestoreError>;
    fn recheck(&self, candidate: &RestoreCandidate) -> Result<RestoreCandidate, RestoreError>;
}

pub trait ProtectiveBackup {
    fn create_and_validate(&self) -> Result<(), RestoreError>;
}

pub trait RestoreTokenSource {
    fn next_token(&mut self) -> String;
}
