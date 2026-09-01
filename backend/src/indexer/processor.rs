use std::sync::Arc;

use super::event_decoder::{ContractEventDecoder, IndexEvent};
use super::store::IndexerStore;

/// Main indexer processor.
///
/// Polls the Stellar network for new events from the ProofFlow contract,
/// decodes them, and persists projections to the store.
pub struct IndexerProcessor {
    store: IndexerStore,
    contract_id: String,
}

impl IndexerProcessor {
    pub fn new(store: IndexerStore, contract_id: String) -> Self {
        Self { store, contract_id }
    }

    /// Process a batch of raw events from the Stellar RPC.
    ///
    /// Each event is a (topic, data, ledger_sequence, event_hash) tuple.
    /// Processing is idempotent: events that have already been processed
    /// are skipped.
    pub async fn process_batch(
        &self,
        events: Vec<RawContractEvent>,
    ) -> Result<ProcessResult, String> {
        let mut processed = 0u64;
        let mut skipped = 0u64;
        let mut errors = Vec::new();

        for raw in events {
            // Idempotency check
            if self.store.is_event_processed(&raw.event_hash).await {
                skipped += 1;
                continue;
            }

            // Decode
            let event = ContractEventDecoder::decode(&raw.topic, &raw.data);

            // Validate (skip malformed events but don't silently discard)
            if let IndexEvent::Unknown { topic, .. } = &event {
                errors.push(format!("unknown event topic: {topic}"));
                self.store.mark_event_processed(&raw.event_hash).await;
                continue;
            }

            // Persist
            if let Err(e) = self.store.persist_event(&event, raw.ledger_sequence).await {
                errors.push(format!("persist error: {e}"));
                continue;
            }

            // Mark processed
            self.store.mark_event_processed(&raw.event_hash).await;
            processed += 1;
        }

        Ok(ProcessResult {
            processed,
            skipped,
            errors,
        })
    }

    pub fn store(&self) -> &IndexerStore {
        &self.store
    }

    pub fn contract_id(&self) -> &str {
        &self.contract_id
    }
}

/// A raw event from the Stellar network.
#[derive(Debug, Clone)]
pub struct RawContractEvent {
    pub topic: String,
    pub data: String,
    pub ledger_sequence: u64,
    pub event_hash: String,
}

/// Result of processing a batch of events.
#[derive(Debug)]
pub struct ProcessResult {
    pub processed: u64,
    pub skipped: u64,
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn process_batch_happy() {
        let store = IndexerStore::new();
        let processor = IndexerProcessor::new(store, "CONTRACT_ID".to_string());

        let events = vec![
            RawContractEvent {
                topic: "JOB_CR".to_string(),
                data: r#"{"job_id":1,"client":"CAAAA..."}"#.to_string(),
                ledger_sequence: 100,
                event_hash: "hash1".to_string(),
            },
            RawContractEvent {
                topic: "USR_REG".to_string(),
                data: r#"{"address":"CBBBB...","role":"client"}"#.to_string(),
                ledger_sequence: 100,
                event_hash: "hash2".to_string(),
            },
        ];

        let result = processor.process_batch(events).await.unwrap();
        assert_eq!(result.processed, 2);
        assert_eq!(result.skipped, 0);
        assert!(result.errors.is_empty());
    }

    #[tokio::test]
    async fn process_batch_idempotent() {
        let store = IndexerStore::new();
        let processor = IndexerProcessor::new(store, "CONTRACT_ID".to_string());

        let events = vec![RawContractEvent {
            topic: "JOB_CR".to_string(),
            data: r#"{"job_id":1,"client":"CAAAA..."}"#.to_string(),
            ledger_sequence: 100,
            event_hash: "hash1".to_string(),
        }];

        let result = processor.process_batch(events.clone()).await.unwrap();
        assert_eq!(result.processed, 1);

        // Second time — same events should be skipped
        let result = processor.process_batch(events).await.unwrap();
        assert_eq!(result.processed, 0);
        assert_eq!(result.skipped, 1);
    }

    #[tokio::test]
    async fn process_batch_unknown_topic() {
        let store = IndexerStore::new();
        let processor = IndexerProcessor::new(store, "CONTRACT_ID".to_string());

        let events = vec![RawContractEvent {
            topic: "UNKNOWN".to_string(),
            data: "{}".to_string(),
            ledger_sequence: 100,
            event_hash: "hash1".to_string(),
        }];

        let result = processor.process_batch(events).await.unwrap();
        assert_eq!(result.processed, 0);
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].contains("unknown event topic"));
    }
}
