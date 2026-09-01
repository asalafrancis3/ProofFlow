use soroban_sdk::{symbol_short, Address, Env, Symbol};

use crate::types::{Resolution, UserRole};

// ── Event topic symbols (all ≤ 9 chars) ──────────────────────────────────────

pub const JOB_CREATED: Symbol = symbol_short!("JOB_CR");
pub const JOB_FUNDED: Symbol = symbol_short!("JOB_FND");
pub const JOB_ACTIVATED: Symbol = symbol_short!("JOB_ACT");
pub const JOB_CANCELLED: Symbol = symbol_short!("JOB_CNC");
pub const JOB_SETTLED: Symbol = symbol_short!("JOB_STL");

pub const MS_CREATED: Symbol = symbol_short!("MS_CR");
pub const MS_SUBMITTED: Symbol = symbol_short!("MS_SUB");
pub const MS_APPROVED: Symbol = symbol_short!("MS_APR");
pub const MS_REJECTED: Symbol = symbol_short!("MS_REJ");
pub const MS_RELEASED: Symbol = symbol_short!("MS_RLS");

pub const ESC_CREATED: Symbol = symbol_short!("ESC_CR");
pub const ESC_FUNDED: Symbol = symbol_short!("ESC_FND");
pub const ESC_RELEASED: Symbol = symbol_short!("ESC_RLS");
pub const ESC_FROZEN: Symbol = symbol_short!("ESC_FRZ");
pub const ESC_UNFROZEN: Symbol = symbol_short!("ESC_UNF");

pub const DISP_FILED: Symbol = symbol_short!("DISP_FL");
pub const DISP_RESOLVED: Symbol = symbol_short!("DISP_RS");

pub const REP_UPDATED: Symbol = symbol_short!("REP_UPD");
pub const USR_REGISTERED: Symbol = symbol_short!("USR_REG");
pub const VER_ADDED: Symbol = symbol_short!("VER_ADD");
pub const VER_REMOVED: Symbol = symbol_short!("VER_REM");

// ── Event emitters ────────────────────────────────────────────────────────────

pub struct Events;

impl Events {
    pub fn job_created(env: &Env, job_id: u64, client: &Address, total_funded: u128) {
        env.events()
            .publish((JOB_CREATED, job_id), (client.clone(), total_funded));
    }

    pub fn job_funded(env: &Env, job_id: u64, amount: u128) {
        env.events().publish((JOB_FUNDED, job_id), amount);
    }

    pub fn job_activated(env: &Env, job_id: u64) {
        env.events().publish((JOB_ACTIVATED, job_id), ());
    }

    pub fn job_cancelled(env: &Env, job_id: u64) {
        env.events().publish((JOB_CANCELLED, job_id), ());
    }

    pub fn job_settled(env: &Env, job_id: u64) {
        env.events().publish((JOB_SETTLED, job_id), ());
    }

    pub fn milestone_created(env: &Env, job_id: u64, index: u32, amount: u128, worker: &Address) {
        env.events()
            .publish((MS_CREATED, job_id, index), (worker.clone(), amount));
    }

    pub fn milestone_submitted(env: &Env, job_id: u64, index: u32, worker: &Address) {
        env.events().publish((MS_SUBMITTED, job_id, index), worker.clone());
    }

    pub fn milestone_approved(env: &Env, job_id: u64, index: u32, verifier: &Address) {
        env.events().publish((MS_APPROVED, job_id, index), verifier.clone());
    }

    pub fn milestone_rejected(env: &Env, job_id: u64, index: u32, verifier: &Address) {
        env.events().publish((MS_REJECTED, job_id, index), verifier.clone());
    }

    pub fn milestone_released(env: &Env, job_id: u64, index: u32, amount: u128, worker: &Address) {
        env.events()
            .publish((MS_RELEASED, job_id, index), (worker.clone(), amount));
    }

    pub fn escrow_created(env: &Env, job_id: u64) {
        env.events().publish((ESC_CREATED, job_id), ());
    }

    pub fn escrow_funded(env: &Env, job_id: u64, amount: u128) {
        env.events().publish((ESC_FUNDED, job_id), amount);
    }

    pub fn escrow_released(env: &Env, job_id: u64, milestone_idx: u32, amount: u128, recipient: &Address) {
        env.events()
            .publish((ESC_RELEASED, job_id, milestone_idx), (recipient.clone(), amount));
    }

    pub fn escrow_frozen(env: &Env, job_id: u64, dispute_id: u32) {
        env.events().publish((ESC_FROZEN, job_id), dispute_id);
    }

    pub fn escrow_unfrozen(env: &Env, job_id: u64, dispute_id: u32) {
        env.events().publish((ESC_UNFROZEN, job_id), dispute_id);
    }

    pub fn dispute_filed(env: &Env, job_id: u64, milestone_idx: u32, dispute_id: u32, raised_by: &Address) {
        env.events()
            .publish((DISP_FILED, job_id, milestone_idx, dispute_id), raised_by.clone());
    }

    pub fn dispute_resolved(env: &Env, job_id: u64, milestone_idx: u32, dispute_id: u32, resolution: &Resolution) {
        env.events()
            .publish((DISP_RESOLVED, job_id, milestone_idx, dispute_id), resolution.clone());
    }

    pub fn reputation_updated(env: &Env, address: &Address, old_score: u64, new_score: u64) {
        env.events()
            .publish((REP_UPDATED, address.clone()), (old_score, new_score));
    }

    pub fn user_registered(env: &Env, address: &Address, role: &UserRole) {
        env.events().publish((USR_REGISTERED, address.clone()), role.clone());
    }

    pub fn verifier_added(env: &Env, address: &Address, added_by: &Address) {
        env.events().publish((VER_ADDED, address.clone()), added_by.clone());
    }

    pub fn verifier_removed(env: &Env, address: &Address, removed_by: &Address) {
        env.events().publish((VER_REMOVED, address.clone()), removed_by.clone());
    }
}
