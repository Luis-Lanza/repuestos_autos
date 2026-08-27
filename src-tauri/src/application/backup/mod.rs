mod contracts;

use std::collections::HashMap;
use std::path::Path;

pub use contracts::{
    ConfirmRestoreResult, OperationalFacts, PrepareRestoreResult, ProtectiveBackup,
    RestoreCandidate, RestoreCandidateStore, RestoreError, RestoreSummary, RestoreTokenSource,
};

#[derive(Clone)]
struct PendingRestore {
    candidate: RestoreCandidate,
    expires_at: u64,
}

pub struct BackupCoordinator<Store, Protection, Tokens> {
    store: Store,
    protection: Protection,
    tokens: Tokens,
    token_ttl_seconds: u64,
    pending: HashMap<String, PendingRestore>,
}

impl<Store, Protection, Tokens> BackupCoordinator<Store, Protection, Tokens>
where
    Store: RestoreCandidateStore,
    Protection: ProtectiveBackup,
    Tokens: RestoreTokenSource,
{
    pub fn new(
        store: Store,
        protection: Protection,
        tokens: Tokens,
        token_ttl_seconds: u64,
    ) -> Self {
        Self {
            store,
            protection,
            tokens,
            token_ttl_seconds,
            pending: HashMap::new(),
        }
    }

    pub fn prepare(&mut self, source: &Path, now_seconds: u64) -> PrepareRestoreResult {
        let candidate = match self.store.prepare(source) {
            Ok(candidate) => candidate,
            Err(error) => return PrepareRestoreResult::Failed(error),
        };
        let token = self.tokens.next_token();
        self.pending.insert(
            token.clone(),
            PendingRestore {
                candidate: candidate.clone(),
                expires_at: now_seconds.saturating_add(self.token_ttl_seconds),
            },
        );
        PrepareRestoreResult::Prepared {
            token,
            summary: RestoreSummary::from(&candidate),
        }
    }

    pub fn confirm(
        &mut self,
        token: &str,
        confirmed: bool,
        now_seconds: u64,
    ) -> ConfirmRestoreResult {
        let Some(pending) = self.pending.get(token).cloned() else {
            return ConfirmRestoreResult::Failed(RestoreError::TokenInvalid);
        };
        if now_seconds >= pending.expires_at {
            self.pending.remove(token);
            return ConfirmRestoreResult::Failed(RestoreError::TokenExpired);
        }
        if !confirmed {
            return ConfirmRestoreResult::Failed(RestoreError::ConfirmationRequired);
        }
        self.pending.remove(token);
        let rechecked = match self.store.recheck(&pending.candidate) {
            Ok(candidate) => candidate,
            Err(error) => return ConfirmRestoreResult::Failed(error),
        };
        if rechecked != pending.candidate {
            return ConfirmRestoreResult::Failed(RestoreError::InvalidBackup);
        }
        if let Err(error) = self.protection.create_and_validate() {
            return ConfirmRestoreResult::Failed(error);
        }
        ConfirmRestoreResult::ReadyForReplacement {
            candidate: pending.candidate,
        }
    }
}
