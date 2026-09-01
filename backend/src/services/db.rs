/// ProofFlow database schema — Redis key patterns and data structures.
///
/// The on-chain contract is the source of truth for financial state.
/// Redis provides fast query-optimized projections for the API layer.
use serde::{Deserialize, Serialize};

// ── Redis key patterns ────────────────────────────────────────────────────────
//
// All keys are prefixed with `pf:` (ProofFlow) to namespace in shared Redis.
//
// Users:
//   pf:user:{address}              → Hash (address, role, name, registered_at)
//   pf:users_by_role:{role}        → Set of addresses
//
// Jobs:
//   pf:job:{job_id}                → Hash (id, client, title, desc, status, ...)
//   pf:jobs_by_client:{address}    → Sorted Set (score=created_at, member=job_id)
//   pf:jobs_by_status:{status}     → Set of job_ids
//   pf:job_counter                 → String (next job_id)
//
// Milestones:
//   pf:milestone:{job_id}:{index}  → Hash (job_id, index, title, amount, status, ...)
//   pf:milestones_by_job:{job_id}  → List (ordered milestone indices)
//
// Escrows:
//   pf:escrow:{job_id}             → Hash (job_id, total_funded, total_released, ...)
//
// Disputes:
//   pf:dispute:{job_id}:{did}      → Hash (job_id, dispute_id, raised_by, status, ...)
//   pf:disputes_by_job:{job_id}    → Set of dispute_ids
//
// Reputation:
//   pf:reputation:{address}        → Hash (completed_jobs, attestation, disputes, ...)
//
// Verifiers:
//   pf:verifiers                   → Set of addresses
//   pf:is_verifier:{address}       → String "1" or absent
//
// Indexer:
//   pf:indexer:cursor              → String (last processed ledger sequence)
//   pf:indexer:processed:{hash}    → String "1" (idempotency, with TTL)

pub struct KeyPatterns;

impl KeyPatterns {
    pub fn user(address: &str) -> String {
        format!("pf:user:{address}")
    }

    pub fn users_by_role(role: &str) -> String {
        format!("pf:users_by_role:{role}")
    }

    pub fn job(job_id: u64) -> String {
        format!("pf:job:{job_id}")
    }

    pub fn jobs_by_client(address: &str) -> String {
        format!("pf:jobs_by_client:{address}")
    }

    pub fn jobs_by_status(status: &str) -> String {
        format!("pf:jobs_by_status:{status}")
    }

    pub fn job_counter() -> String {
        "pf:job_counter".to_string()
    }

    pub fn milestone(job_id: u64, index: u32) -> String {
        format!("pf:milestone:{job_id}:{index}")
    }

    pub fn milestones_by_job(job_id: u64) -> String {
        format!("pf:milestones_by_job:{job_id}")
    }

    pub fn escrow(job_id: u64) -> String {
        format!("pf:escrow:{job_id}")
    }

    pub fn dispute(job_id: u64, dispute_id: u32) -> String {
        format!("pf:dispute:{job_id}:{dispute_id}")
    }

    pub fn disputes_by_job(job_id: u64) -> String {
        format!("pf:disputes_by_job:{job_id}")
    }

    pub fn reputation(address: &str) -> String {
        format!("pf:reputation:{address}")
    }

    pub fn verifiers() -> String {
        "pf:verifiers".to_string()
    }

    pub fn is_verifier(address: &str) -> String {
        format!("pf:is_verifier:{address}")
    }

    pub fn indexer_cursor() -> String {
        "pf:indexer:cursor".to_string()
    }

    pub fn indexer_processed(event_hash: &str) -> String {
        format!("pf:indexer:processed:{event_hash}")
    }
}

// ── Stored data shapes ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredUser {
    pub address: String,
    pub role: String,
    pub name: String,
    pub registered_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredJob {
    pub id: u64,
    pub client: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub total_funded: u64,
    pub milestone_count: u32,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMilestone {
    pub job_id: u64,
    pub index: u32,
    pub title: String,
    pub description: String,
    pub amount: u64,
    pub status: String,
    pub worker: String,
    pub evidence_uri: String,
    pub has_evidence: bool,
    pub submitted_at: u64,
    pub resolved_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEscrow {
    pub job_id: u64,
    pub total_funded: u64,
    pub total_released: u64,
    pub total_frozen: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredDispute {
    pub job_id: u64,
    pub milestone_idx: u32,
    pub dispute_id: u32,
    pub raised_by: String,
    pub reason: String,
    pub status: String,
    pub resolution: String,
    pub has_resolution: bool,
    pub created_at: u64,
    pub resolved_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredReputation {
    pub address: String,
    pub completed_jobs: u64,
    pub successful_attestations: u64,
    pub disputes_involved: u64,
    pub disputes_won: u64,
    pub total_earned: u64,
    pub score: u64,
    pub updated_at: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_patterns() {
        assert_eq!(KeyPatterns::user("GABC"), "pf:user:GABC");
        assert_eq!(KeyPatterns::job(42), "pf:job:42");
        assert_eq!(KeyPatterns::milestone(1, 0), "pf:milestone:1:0");
        assert_eq!(KeyPatterns::escrow(7), "pf:escrow:7");
        assert_eq!(KeyPatterns::dispute(1, 2), "pf:dispute:1:2");
        assert_eq!(KeyPatterns::reputation("GABC"), "pf:reputation:GABC");
        assert_eq!(KeyPatterns::jobs_by_status("funded"), "pf:jobs_by_status:funded");
        assert_eq!(KeyPatterns::verifiers(), "pf:verifiers");
        assert_eq!(KeyPatterns::indexer_cursor(), "pf:indexer:cursor");
    }

    #[test]
    fn stored_types_serialization_roundtrip() {
        let user = StoredUser {
            address: "GABC".to_string(),
            role: "client".to_string(),
            name: "Alice".to_string(),
            registered_at: 1000,
        };
        let json = serde_json::to_string(&user).unwrap();
        let back: StoredUser = serde_json::from_str(&json).unwrap();
        assert_eq!(back.address, "GABC");
        assert_eq!(back.role, "client");

        let job = StoredJob {
            id: 1,
            client: "GABC".to_string(),
            title: "Build website".to_string(),
            description: "A website".to_string(),
            status: "funded".to_string(),
            total_funded: 10000,
            milestone_count: 3,
            created_at: 1000,
            updated_at: 1000,
        };
        let json = serde_json::to_string(&job).unwrap();
        let back: StoredJob = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, 1);
        assert_eq!(back.total_funded, 10000);
    }
}
