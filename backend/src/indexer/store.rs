use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use super::event_decoder::IndexEvent;

/// Persistent store for indexed data.
///
/// This is the off-chain query state. The on-chain contract is the source
/// of truth for all financial state (escrow balances, milestone amounts).
/// This store holds query-optimized projections for the API layer.
pub struct IndexerStore {
    inner: Arc<RwLock<IndexerState>>,
}

#[derive(Default)]
struct IndexerState {
    /// Last processed ledger sequence.
    pub last_ledger: u64,
    /// Processed event hashes (for idempotency).
    pub processed_events: HashMap<String, bool>,
    /// Users indexed by address.
    pub users: HashMap<String, super::event_decoder::IndexEvent>,
}

impl IndexerStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(IndexerState::default())),
        }
    }

    /// Check if an event has already been processed (idempotency).
    pub async fn is_event_processed(&self, event_hash: &str) -> bool {
        let state = self.inner.read().await;
        state.processed_events.contains_key(event_hash)
    }

    /// Mark an event as processed.
    pub async fn mark_event_processed(&self, event_hash: &str) {
        let mut state = self.inner.write().await;
        state.processed_events.insert(event_hash.to_string(), true);
    }

    /// Update the last processed ledger sequence.
    pub async fn update_last_ledger(&self, ledger: u64) {
        let mut state = self.inner.write().await;
        state.last_ledger = ledger;
    }

    /// Get the last processed ledger sequence.
    pub async fn last_ledger(&self) -> u64 {
        let state = self.inner.read().await;
        state.last_ledger
    }

    /// Persist an indexed event (update projections).
    pub async fn persist_event(&self, event: &IndexEvent, ledger: u64) -> Result<(), String> {
        let mut state = self.inner.write().await;

        match event {
            IndexEvent::UserRegistered { address, role } => {
                state.users.insert(
                    address.clone(),
                    IndexEvent::UserRegistered {
                        address: address.clone(),
                        role: role.clone(),
                    },
                );
            }
            _ => {
                // Other events update job/milestone/escrow projections
                // which will be implemented when the database layer is added.
            }
        }

        state.last_ledger = ledger;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn idempotency_check() {
        let store = IndexerStore::new();
        assert!(!store.is_event_processed("hash1").await);
        store.mark_event_processed("hash1").await;
        assert!(store.is_event_processed("hash1").await);
    }

    #[tokio::test]
    async fn ledger_tracking() {
        let store = IndexerStore::new();
        assert_eq!(store.last_ledger().await, 0);
        store.update_last_ledger(100).await;
        assert_eq!(store.last_ledger().await, 100);
    }
}
