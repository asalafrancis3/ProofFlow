use async_trait::async_trait;

use super::error::ContractError;
use super::types::*;

/// Transaction parameters for a contract mutation call.
#[derive(Debug, Clone)]
pub struct TxParams {
    pub function: String,
    pub args: Vec<serde_json::Value>,
    pub auth_required: String,
}

/// Adapter trait for ProofFlow contract interactions.
///
/// All contract calls flow through this single boundary.
/// API handlers → domain services → `ProofFlowContractAdapter` → Soroban.
///
/// The trait is async to support the underlying Stellar RPC calls.
#[async_trait]
pub trait ProofFlowContractAdapter: Send + Sync {
    // ── Read-only queries ─────────────────────────────────────────────────────

    async fn get_admin(&self) -> Result<String, ContractError>;
    async fn get_user(&self, address: &str) -> Result<User, ContractError>;
    async fn is_verifier(&self, address: &str) -> Result<bool, ContractError>;
    async fn get_job(&self, job_id: u64) -> Result<Job, ContractError>;
    async fn get_milestone(&self, job_id: u64, index: u32) -> Result<Milestone, ContractError>;
    async fn get_escrow(&self, job_id: u64) -> Result<Escrow, ContractError>;
    async fn get_reputation(&self, address: &str) -> Result<Reputation, ContractError>;
    async fn get_dispute(&self, job_id: u64, dispute_id: u32) -> Result<Dispute, ContractError>;

    // ── Mutation builders ─────────────────────────────────────────────────────
    //
    // These methods prepare the transaction parameters. The actual signing
    // and submission happens at a higher layer (the wallet/secret-key holder).

    async fn build_create_job_tx(
        &self,
        req: &CreateJobRequest,
    ) -> Result<TxParams, ContractError>;

    async fn build_fund_job_tx(
        &self,
        client: &str,
        job_id: u64,
    ) -> Result<TxParams, ContractError>;

    async fn build_activate_job_tx(
        &self,
        client: &str,
        job_id: u64,
    ) -> Result<TxParams, ContractError>;

    async fn build_cancel_job_tx(
        &self,
        client: &str,
        job_id: u64,
        reason: &str,
    ) -> Result<TxParams, ContractError>;

    async fn build_submit_evidence_tx(
        &self,
        req: &SubmitEvidenceRequest,
    ) -> Result<TxParams, ContractError>;

    async fn build_approve_milestone_tx(
        &self,
        verifier: &str,
        job_id: u64,
        milestone_idx: u32,
    ) -> Result<TxParams, ContractError>;

    async fn build_reject_milestone_tx(
        &self,
        verifier: &str,
        job_id: u64,
        milestone_idx: u32,
        reason: &str,
    ) -> Result<TxParams, ContractError>;

    async fn build_release_escrow_tx(
        &self,
        job_id: u64,
        milestone_idx: u32,
    ) -> Result<TxParams, ContractError>;

    async fn build_file_dispute_tx(
        &self,
        req: &FileDisputeRequest,
    ) -> Result<TxParams, ContractError>;

    async fn build_resolve_dispute_tx(
        &self,
        req: &ResolveDisputeRequest,
    ) -> Result<TxParams, ContractError>;
}
