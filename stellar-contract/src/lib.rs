#![no_std]

pub mod errors;
pub mod events;
pub mod types;
pub mod validation;

pub use errors::Error;
pub use types::{
    Attestation, AttestationOutcome, Dispute, DisputeStatus, Escrow, EscrowStatus, Job, JobStatus, Milestone,
    MilestoneStatus, Reputation, Resolution, Submission, User, UserRole,
};

use events::Events;
use types::*;
use validation::*;

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol, Vec};

// ── Storage keys ──────────────────────────────────────────────────────────────

const ADMIN: Symbol = symbol_short!("ADMIN");
const JOB_COUNT: Symbol = symbol_short!("JOB_CNT");
const DISPUTE_COUNT: Symbol = symbol_short!("DISP_CNT");
const VERIFIER_LIST: Symbol = symbol_short!("VER_LIST");
const PAUSED: Symbol = symbol_short!("PAUSED");
const TOKEN_ADDR: Symbol = symbol_short!("TKN_ADDR");

// ── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct ProofFlow;

#[contractimpl]
impl ProofFlow {
    // ══════════════════════════════════════════════════════════════════════════
    // INITIALIZATION
    // ══════════════════════════════════════════════════════════════════════════

    pub fn initialize(env: Env, admin: Address, token: Address) -> Result<(), Error> {
        if env.storage().instance().has(&ADMIN) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&TOKEN_ADDR, &token);
        env.storage().instance().set(&JOB_COUNT, &0u64);
        env.storage().instance().set(&DISPUTE_COUNT, &0u32);
        env.storage().instance().set(&PAUSED, &false);
        Ok(())
    }

    pub fn pause(env: Env, admin: Address) -> Result<(), Error> {
        require_admin(&env, &admin)?;
        env.storage().instance().set(&PAUSED, &true);
        Ok(())
    }

    pub fn unpause(env: Env, admin: Address) -> Result<(), Error> {
        require_admin(&env, &admin)?;
        env.storage().instance().set(&PAUSED, &false);
        Ok(())
    }

    pub fn get_admin(env: Env) -> Result<Address, Error> {
        env.storage().instance().get(&ADMIN).ok_or(Error::NotAuthorized)
    }

    pub fn transfer_admin(env: Env, admin: Address, new_admin: Address) -> Result<(), Error> {
        require_admin(&env, &admin)?;
        env.storage().instance().set(&ADMIN, &new_admin);
        Ok(())
    }

    // ══════════════════════════════════════════════════════════════════════════
    // USER MANAGEMENT
    // ══════════════════════════════════════════════════════════════════════════

    pub fn register_user(env: Env, address: Address, role: UserRole, name: String) -> Result<(), Error> {
        require_not_paused(&env)?;
        validate_non_empty_string(&name)?;

        // Prevent duplicate registration — address can only be registered once
        let key = user_key(&address);
        if env.storage().persistent().has(&key) {
            return Err(Error::UserAlreadyRegistered);
        }

        let user = User {
            address: address.clone(),
            org_id: env.current_contract_address(),
            has_org: false,
            role,
            name,
            registered_at: current_timestamp(&env),
        };

        env.storage().persistent().set(&key, &user);

        // Initialize reputation
        let rep = Reputation {
            address: address.clone(),
            completed_jobs: 0,
            successful_attestations: 0,
            disputes_involved: 0,
            disputes_won: 0,
            total_earned: 0,
            score: 0,
            updated_at: current_timestamp(&env),
        };
        let rep_key = reputation_key(&address);
        env.storage().persistent().set(&rep_key, &rep);

        Events::user_registered(&env, &address, &role);
        Ok(())
    }

    pub fn get_user(env: Env, address: Address) -> Result<User, Error> {
        let key = user_key(&address);
        env.storage().persistent().get(&key).ok_or(Error::NotFound)
    }

    // ══════════════════════════════════════════════════════════════════════════
    // VERIFIER MANAGEMENT
    // ══════════════════════════════════════════════════════════════════════════

    pub fn add_verifier(env: Env, admin: Address, verifier: Address) -> Result<(), Error> {
        require_admin(&env, &admin)?;

        let mut verifiers: Vec<Address> = env
            .storage()
            .instance()
            .get(&VERIFIER_LIST)
            .unwrap_or_else(|| Vec::new(&env));

        // Check duplicate
        for v in verifiers.iter() {
            if v == verifier {
                return Err(Error::VerifierAlreadyWhitelisted);
            }
        }

        verifiers.push_back(verifier.clone());
        env.storage().instance().set(&VERIFIER_LIST, &verifiers);
        Events::verifier_added(&env, &verifier, &admin);
        Ok(())
    }

    pub fn remove_verifier(env: Env, admin: Address, verifier: Address) -> Result<(), Error> {
        require_admin(&env, &admin)?;

        let mut verifiers: Vec<Address> = env
            .storage()
            .instance()
            .get(&VERIFIER_LIST)
            .ok_or(Error::VerifierNotWhitelisted)?;

        let mut found = false;
        let mut new_verifiers: Vec<Address> = Vec::new(&env);
        for v in verifiers.iter() {
            if v == verifier {
                found = true;
            } else {
                new_verifiers.push_back(v);
            }
        }

        if !found {
            return Err(Error::VerifierNotWhitelisted);
        }

        env.storage().instance().set(&VERIFIER_LIST, &new_verifiers);
        Events::verifier_removed(&env, &verifier, &admin);
        Ok(())
    }

    pub fn is_verifier(env: Env, address: Address) -> bool {
        let verifiers: Vec<Address> = env
            .storage()
            .instance()
            .get(&VERIFIER_LIST)
            .unwrap_or_else(|| Vec::new(&env));
        for v in verifiers.iter() {
            if v == address {
                return true;
            }
        }
        false
    }

    // ══════════════════════════════════════════════════════════════════════════
    // JOB MANAGEMENT
    // ══════════════════════════════════════════════════════════════════════════

    pub fn create_job(
        env: Env,
        client: Address,
        title: String,
        description: String,
        milestone_titles: Vec<String>,
        milestone_amounts: Vec<u128>,
        milestone_workers: Vec<Address>,
    ) -> Result<u64, Error> {
        require_not_paused(&env)?;
        validate_title(&title)?;
        validate_description(&description)?;

        if milestone_titles.len() != milestone_amounts.len() || milestone_titles.len() != milestone_workers.len() {
            return Err(Error::InvalidInput);
        }
        if milestone_titles.is_empty() {
            return Err(Error::JobNotEnoughMilestones);
        }

        // Check caller is registered as Client (or is the contract admin)
        let admin_addr: Option<Address> = env.storage().instance().get(&ADMIN);
        let is_contract_admin = admin_addr.as_ref() == Some(&client);
        if !is_contract_admin {
            let user = get_user_or_error(&env, &client)?;
            if user.role != UserRole::Client {
                return Err(Error::NotAuthorized);
            }
        }

        // Allocate job ID
        let job_id: u64 = env.storage().instance().get(&JOB_COUNT).unwrap_or(0);
        env.storage().instance().set(&JOB_COUNT, &(job_id + 1));

        let total_funded: u128 = milestone_amounts.iter().sum();
        let now = current_timestamp(&env);

        let job = Job {
            id: job_id,
            client: client.clone(),
            title: title.clone(),
            description,
            status: JobStatus::Draft,
            total_funded,
            milestone_count: milestone_titles.len(),
            created_at: now,
            updated_at: now,
        };

        let job_key = job_key(job_id);
        env.storage().persistent().set(&job_key, &job);

        // Create escrow
        let escrow = Escrow {
            job_id,
            total_funded: 0,
            total_released: 0,
            total_frozen: 0,
            status: EscrowStatus::Created,
        };
        let escrow_key = escrow_key(job_id);
        env.storage().persistent().set(&escrow_key, &escrow);

        // Create milestones
        for i in 0..milestone_titles.len() {
            let milestone = Milestone {
                job_id,
                index: i,
                title: milestone_titles.get(i).unwrap(),
                description: String::from_str(&env, ""),
                amount: milestone_amounts.get(i).unwrap(),
                status: MilestoneStatus::Pending,
                worker: milestone_workers.get(i).unwrap(),
                evidence_uri: String::from_str(&env, ""),
                has_evidence: false,
                submitted_at: 0,
                resolved_at: 0,
            };
            let ms_key = milestone_key(job_id, i);
            env.storage().persistent().set(&ms_key, &milestone);

            Events::milestone_created(&env, job_id, i, milestone.amount, &milestone.worker);
        }

        Events::job_created(&env, job_id, &client, total_funded);
        Events::escrow_created(&env, job_id);

        Ok(job_id)
    }

    pub fn fund_job(env: Env, client: Address, job_id: u64) -> Result<(), Error> {
        require_not_paused(&env)?;

        let mut job = get_job_or_error(&env, job_id)?;
        if job.client != client {
            return Err(Error::NotAuthorized);
        }
        if job.status != JobStatus::Draft {
            return Err(Error::JobNotDraft);
        }

        // Transfer tokens from client to contract
        let token_addr: Address = env.storage().instance().get(&TOKEN_ADDR).ok_or(Error::NotAuthorized)?;
        let token = soroban_sdk::token::Client::new(&env, &token_addr);
        token.transfer_from(
            &env.current_contract_address(),
            &client,
            &env.current_contract_address(),
            &(job.total_funded as i128),
        );

        // Update escrow
        let mut escrow = get_escrow_or_error(&env, job_id)?;
        escrow.total_funded = job.total_funded;
        escrow.status = EscrowStatus::Funded;
        let escrow_k = escrow_key(job_id);
        env.storage().persistent().set(&escrow_k, &escrow);

        // Update job
        job.status = JobStatus::Funded;
        job.updated_at = current_timestamp(&env);
        let job_k = job_key(job_id);
        env.storage().persistent().set(&job_k, &job);

        Events::job_funded(&env, job_id, job.total_funded);
        Events::escrow_funded(&env, job_id, job.total_funded);

        Ok(())
    }

    pub fn activate_job(env: Env, client: Address, job_id: u64) -> Result<(), Error> {
        require_not_paused(&env)?;

        let mut job = get_job_or_error(&env, job_id)?;
        if job.client != client {
            return Err(Error::NotAuthorized);
        }
        if job.status != JobStatus::Funded {
            return Err(Error::JobNotFunded);
        }

        job.status = JobStatus::Active;
        job.updated_at = current_timestamp(&env);
        let job_k = job_key(job_id);
        env.storage().persistent().set(&job_k, &job);

        Events::job_activated(&env, job_id);
        Ok(())
    }

    pub fn cancel_job(env: Env, client: Address, job_id: u64, reason: String) -> Result<(), Error> {
        require_not_paused(&env)?;

        let mut job = get_job_or_error(&env, job_id)?;
        if job.client != client {
            return Err(Error::NotAuthorized);
        }
        if job.status.is_terminal() {
            return Err(Error::JobCannotBeCancelled);
        }

        // Check no submissions yet
        for i in 0..job.milestone_count {
            let ms = get_milestone_or_error(&env, job_id, i)?;
            if ms.status == MilestoneStatus::Submitted || ms.status == MilestoneStatus::Approved {
                return Err(Error::JobCannotBeCancelled);
            }
        }

        // Refund escrow
        let mut escrow = get_escrow_or_error(&env, job_id)?;
        if escrow.total_funded > 0 {
            let token_addr: Address = env.storage().instance().get(&TOKEN_ADDR).ok_or(Error::NotAuthorized)?;
            let token = soroban_sdk::token::Client::new(&env, &token_addr);
            token.transfer(&env.current_contract_address(), &client, &(escrow.remaining() as i128));
            escrow.total_funded = 0;
            escrow.total_released = 0;
            escrow.total_frozen = 0;
            escrow.status = EscrowStatus::Created;
            let escrow_k = escrow_key(job_id);
            env.storage().persistent().set(&escrow_k, &escrow);
        }

        job.status = JobStatus::Cancelled;
        job.updated_at = current_timestamp(&env);
        let job_k = job_key(job_id);
        env.storage().persistent().set(&job_k, &job);

        Events::job_cancelled(&env, job_id);
        Ok(())
    }

    pub fn get_job(env: Env, job_id: u64) -> Result<Job, Error> {
        get_job_or_error(&env, job_id)
    }

    // ══════════════════════════════════════════════════════════════════════════
    // MILESTONE MANAGEMENT
    // ══════════════════════════════════════════════════════════════════════════

    pub fn submit_evidence(
        env: Env,
        worker: Address,
        job_id: u64,
        milestone_idx: u32,
        evidence_uri: String,
        notes: String,
    ) -> Result<(), Error> {
        require_not_paused(&env)?;
        validate_non_empty_string(&evidence_uri)?;

        let job = get_job_or_error(&env, job_id)?;
        if job.status != JobStatus::Active && job.status != JobStatus::InReview {
            return Err(Error::JobNotActive);
        }

        let mut ms = get_milestone_or_error(&env, job_id, milestone_idx)?;
        if ms.worker != worker {
            return Err(Error::NotAuthorized);
        }
        if !ms.status.can_submit() {
            return Err(Error::MilestoneCannotSubmit);
        }

        let now = current_timestamp(&env);

        // Create submission
        let submission = Submission {
            job_id,
            milestone_idx,
            worker: worker.clone(),
            evidence_uri: evidence_uri.clone(),
            notes,
            submitted_at: now,
        };
        let sub_key = submission_key(job_id, milestone_idx);
        env.storage().persistent().set(&sub_key, &submission);

        // Update milestone
        ms.status = MilestoneStatus::Submitted;
        ms.evidence_uri = evidence_uri.clone();
        ms.has_evidence = true;
        ms.submitted_at = now;
        let ms_key = milestone_key(job_id, milestone_idx);
        env.storage().persistent().set(&ms_key, &ms);

        // Transition job to InReview if first submission
        let mut job_mut = job;
        if job_mut.status == JobStatus::Active {
            job_mut.status = JobStatus::InReview;
            job_mut.updated_at = now;
            let job_k = job_key(job_id);
            env.storage().persistent().set(&job_k, &job_mut);
        }

        Events::milestone_submitted(&env, job_id, milestone_idx, &worker);
        Ok(())
    }

    pub fn approve_milestone(env: Env, verifier: Address, job_id: u64, milestone_idx: u32) -> Result<(), Error> {
        require_not_paused(&env)?;

        if !Self::is_verifier(env.clone(), verifier.clone()) {
            return Err(Error::NotWhitelistedVerifier);
        }

        let mut ms = get_milestone_or_error(&env, job_id, milestone_idx)?;
        if ms.status != MilestoneStatus::Submitted {
            return Err(Error::MilestoneNotSubmitted);
        }
        if ms.worker == verifier {
            return Err(Error::CannotAttestOwnWork);
        }

        let now = current_timestamp(&env);

        // Create attestation
        let attestation = Attestation {
            job_id,
            milestone_idx,
            verifier: verifier.clone(),
            outcome: AttestationOutcome::Approved,
            notes: String::from_str(&env, ""),
            created_at: now,
        };
        let att_key = attestation_key(job_id, milestone_idx);
        env.storage().persistent().set(&att_key, &attestation);

        // Update milestone
        ms.status = MilestoneStatus::Approved;
        ms.resolved_at = now;
        let ms_key = milestone_key(job_id, milestone_idx);
        env.storage().persistent().set(&ms_key, &ms);

        Events::milestone_approved(&env, job_id, milestone_idx, &verifier);
        Ok(())
    }

    pub fn reject_milestone(
        env: Env,
        verifier: Address,
        job_id: u64,
        milestone_idx: u32,
        reason: String,
    ) -> Result<(), Error> {
        require_not_paused(&env)?;

        if !Self::is_verifier(env.clone(), verifier.clone()) {
            return Err(Error::NotWhitelistedVerifier);
        }

        let mut ms = get_milestone_or_error(&env, job_id, milestone_idx)?;
        if ms.status != MilestoneStatus::Submitted {
            return Err(Error::MilestoneNotSubmitted);
        }

        let now = current_timestamp(&env);

        // Create attestation
        let attestation = Attestation {
            job_id,
            milestone_idx,
            verifier: verifier.clone(),
            outcome: AttestationOutcome::Rejected,
            notes: reason.clone(),
            created_at: now,
        };
        let att_key = attestation_key(job_id, milestone_idx);
        env.storage().persistent().set(&att_key, &attestation);

        // Update milestone
        ms.status = MilestoneStatus::Rejected;
        ms.resolved_at = now;
        ms.evidence_uri = String::from_str(&env, "");
        ms.has_evidence = false;
        let ms_key = milestone_key(job_id, milestone_idx);
        env.storage().persistent().set(&ms_key, &ms);

        Events::milestone_rejected(&env, job_id, milestone_idx, &verifier);
        Ok(())
    }

    pub fn get_milestone(env: Env, job_id: u64, index: u32) -> Result<Milestone, Error> {
        get_milestone_or_error(&env, job_id, index)
    }

    // ══════════════════════════════════════════════════════════════════════════
    // ESCROW & SETTLEMENT
    // ══════════════════════════════════════════════════════════════════════════

    pub fn release_milestone_escrow(env: Env, job_id: u64, milestone_idx: u32) -> Result<(), Error> {
        require_not_paused(&env)?;

        let ms = get_milestone_or_error(&env, job_id, milestone_idx)?;
        if ms.status != MilestoneStatus::Approved {
            return Err(Error::MilestoneNotApproved);
        }

        let mut escrow = get_escrow_or_error(&env, job_id)?;
        if escrow.status == EscrowStatus::Frozen {
            return Err(Error::EscrowFrozen);
        }
        if !escrow.can_release(ms.amount) {
            return Err(Error::EscrowOverRelease);
        }

        // Transfer tokens to worker
        let token_addr: Address = env.storage().instance().get(&TOKEN_ADDR).ok_or(Error::NotAuthorized)?;
        let token = soroban_sdk::token::Client::new(&env, &token_addr);
        token.transfer(&env.current_contract_address(), &ms.worker, &(ms.amount as i128));

        // Update escrow
        escrow.total_released += ms.amount;
        escrow.status = if escrow.remaining() == 0 {
            EscrowStatus::Completed
        } else {
            EscrowStatus::PartialRelease
        };
        let escrow_k = escrow_key(job_id);
        env.storage().persistent().set(&escrow_k, &escrow);

        // Update milestone
        let mut ms_mut = ms;
        ms_mut.status = MilestoneStatus::Released;
        let ms_key = milestone_key(job_id, milestone_idx);
        env.storage().persistent().set(&ms_key, &ms_mut);

        // Update reputation
        update_reputation(&env, &ms_mut.worker, true, 0, 0)?;

        Events::milestone_released(&env, job_id, milestone_idx, ms_mut.amount, &ms_mut.worker);
        Events::escrow_released(&env, job_id, milestone_idx, ms_mut.amount, &ms_mut.worker);

        // Check if all milestones are settled
        check_job_settlement(&env, job_id)?;

        Ok(())
    }

    pub fn get_escrow(env: Env, job_id: u64) -> Result<Escrow, Error> {
        get_escrow_or_error(&env, job_id)
    }

    // ══════════════════════════════════════════════════════════════════════════
    // DISPUTES
    // ══════════════════════════════════════════════════════════════════════════

    pub fn file_dispute(
        env: Env,
        caller: Address,
        job_id: u64,
        milestone_idx: u32,
        reason: String,
    ) -> Result<u32, Error> {
        require_not_paused(&env)?;
        validate_non_empty_string(&reason)?;

        let job = get_job_or_error(&env, job_id)?;
        if job.client != caller {
            // Worker can also file dispute
            let ms = get_milestone_or_error(&env, job_id, milestone_idx)?;
            if ms.worker != caller {
                return Err(Error::NotAuthorized);
            }
        }

        let ms = get_milestone_or_error(&env, job_id, milestone_idx)?;
        if ms.status == MilestoneStatus::Released {
            return Err(Error::CannotDisputeReleasedMilestone);
        }
        if ms.status == MilestoneStatus::Disputed {
            return Err(Error::MilestoneAlreadyDisputed);
        }

        let dispute_id: u32 = env.storage().instance().get(&DISPUTE_COUNT).unwrap_or(0);
        env.storage().instance().set(&DISPUTE_COUNT, &(dispute_id + 1));

        let now = current_timestamp(&env);

        let dispute = Dispute {
            job_id,
            milestone_idx,
            dispute_id,
            raised_by: caller.clone(),
            reason: reason.clone(),
            status: DisputeStatus::Filed,
            resolution: Resolution::UpholdClient,
            has_resolution: false,
            created_at: now,
            resolved_at: 0,
        };
        let disp_key = dispute_key(job_id, dispute_id);
        env.storage().persistent().set(&disp_key, &dispute);

        // Update milestone status
        let mut ms_mut = ms;
        ms_mut.status = MilestoneStatus::Disputed;
        let ms_key = milestone_key(job_id, milestone_idx);
        env.storage().persistent().set(&ms_key, &ms_mut);

        // Freeze escrow
        let mut escrow = get_escrow_or_error(&env, job_id)?;
        if escrow.status != EscrowStatus::Frozen {
            escrow.total_frozen += ms_mut.amount;
            escrow.status = EscrowStatus::Frozen;
            let escrow_k = escrow_key(job_id);
            env.storage().persistent().set(&escrow_k, &escrow);
        }

        // Update dispute status to UnderReview
        let mut disp_mut = dispute;
        disp_mut.status = DisputeStatus::UnderReview;
        let disp_k = dispute_key(job_id, dispute_id);
        env.storage().persistent().set(&disp_k, &disp_mut);

        // Update reputation
        update_reputation(&env, &caller, false, 0, 1)?;

        Events::dispute_filed(&env, job_id, milestone_idx, dispute_id, &caller);
        Events::escrow_frozen(&env, job_id, dispute_id);

        Ok(dispute_id)
    }

    pub fn resolve_dispute(
        env: Env,
        arbitrator: Address,
        job_id: u64,
        dispute_id: u32,
        resolution: Resolution,
    ) -> Result<(), Error> {
        require_not_paused(&env)?;

        // Only admin can assign arbitrators (or be the arbitrator)
        let user = get_user_or_error(&env, &arbitrator)?;
        if user.role != UserRole::Arbitrator && user.role != UserRole::Admin {
            return Err(Error::NotAuthorized);
        }

        let mut disp = get_dispute_or_error(&env, job_id, dispute_id)?;
        if disp.status == DisputeStatus::Resolved {
            return Err(Error::DisputeAlreadyResolved);
        }

        let now = current_timestamp(&env);
        let ms = get_milestone_or_error(&env, job_id, disp.milestone_idx)?;

        // Execute resolution
        let mut escrow = get_escrow_or_error(&env, job_id)?;

        match resolution {
            Resolution::UpholdWorker => {
                // Release to worker
                if escrow.can_release(ms.amount) {
                    let token_addr: Address = env.storage().instance().get(&TOKEN_ADDR).ok_or(Error::NotAuthorized)?;
                    let token = soroban_sdk::token::Client::new(&env, &token_addr);
                    token.transfer(&env.current_contract_address(), &ms.worker, &(ms.amount as i128));
                    escrow.total_released += ms.amount;
                }
            }
            Resolution::UpholdClient => {
                // Return to client
                let job = get_job_or_error(&env, job_id)?;
                if escrow.total_frozen >= ms.amount {
                    let token_addr: Address = env.storage().instance().get(&TOKEN_ADDR).ok_or(Error::NotAuthorized)?;
                    let token = soroban_sdk::token::Client::new(&env, &token_addr);
                    token.transfer(&env.current_contract_address(), &job.client, &(ms.amount as i128));
                }
            }
            Resolution::PartialSplit => {
                // Split 50/50
                let half = ms.amount / 2;
                let job = get_job_or_error(&env, job_id)?;
                let token_addr: Address = env.storage().instance().get(&TOKEN_ADDR).ok_or(Error::NotAuthorized)?;
                let token = soroban_sdk::token::Client::new(&env, &token_addr);

                if escrow.can_release(half) {
                    token.transfer(&env.current_contract_address(), &ms.worker, &(half as i128));
                    escrow.total_released += half;
                }
                if escrow.total_frozen >= ms.amount - half {
                    token.transfer(
                        &env.current_contract_address(),
                        &job.client,
                        &((ms.amount - half) as i128),
                    );
                }
            }
        }

        // Unfreeze
        escrow.total_frozen = escrow.total_frozen.saturating_sub(ms.amount);
        escrow.status = if escrow.remaining() == 0 {
            EscrowStatus::Completed
        } else if escrow.total_frozen == 0 {
            EscrowStatus::PartialRelease
        } else {
            EscrowStatus::Frozen
        };
        let escrow_k = escrow_key(job_id);
        env.storage().persistent().set(&escrow_k, &escrow);

        // Update dispute
        disp.status = DisputeStatus::Resolved;
        disp.resolution = resolution;
        disp.has_resolution = true;
        disp.resolved_at = now;
        let disp_k = dispute_key(job_id, dispute_id);
        env.storage().persistent().set(&disp_k, &disp);

        Events::dispute_resolved(&env, job_id, disp.milestone_idx, dispute_id, &resolution);
        Events::escrow_unfrozen(&env, job_id, dispute_id);

        Ok(())
    }

    pub fn get_dispute(env: Env, job_id: u64, dispute_id: u32) -> Result<Dispute, Error> {
        get_dispute_or_error(&env, job_id, dispute_id)
    }

    // ══════════════════════════════════════════════════════════════════════════
    // REPUTATION
    // ══════════════════════════════════════════════════════════════════════════

    pub fn get_reputation(env: Env, address: Address) -> Result<Reputation, Error> {
        let key = reputation_key(&address);
        env.storage().persistent().get(&key).ok_or(Error::ReputationNotFound)
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// INTERNAL HELPERS
// ══════════════════════════════════════════════════════════════════════════════

fn user_key(address: &Address) -> (Symbol, Address) {
    (symbol_short!("USR_"), address.clone())
}

fn job_key(job_id: u64) -> (Symbol, u64) {
    (symbol_short!("JOB_"), job_id)
}

fn escrow_key(job_id: u64) -> (Symbol, u64) {
    (symbol_short!("ESC_"), job_id)
}

fn milestone_key(job_id: u64, index: u32) -> (Symbol, u64, u32) {
    (symbol_short!("MS_"), job_id, index)
}

fn submission_key(job_id: u64, index: u32) -> (Symbol, u64, u32) {
    (symbol_short!("SUB_"), job_id, index)
}

fn attestation_key(job_id: u64, index: u32) -> (Symbol, u64, u32) {
    (symbol_short!("ATT_"), job_id, index)
}

fn dispute_key(job_id: u64, dispute_id: u32) -> (Symbol, u64, u32) {
    (symbol_short!("DSP_"), job_id, dispute_id)
}

fn reputation_key(address: &Address) -> (Symbol, Address) {
    (symbol_short!("REP_"), address.clone())
}

fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
    let admin: Address = env.storage().instance().get(&ADMIN).ok_or(Error::NotAuthorized)?;
    if *caller != admin {
        return Err(Error::NotAuthorized);
    }
    Ok(())
}

fn require_not_paused(env: &Env) -> Result<(), Error> {
    let paused: bool = env.storage().instance().get(&PAUSED).unwrap_or(false);
    if paused {
        return Err(Error::ContractPaused);
    }
    Ok(())
}

fn get_user_or_error(env: &Env, address: &Address) -> Result<User, Error> {
    let key = user_key(address);
    env.storage().persistent().get(&key).ok_or(Error::NotFound)
}

fn get_job_or_error(env: &Env, job_id: u64) -> Result<Job, Error> {
    let key = job_key(job_id);
    env.storage().persistent().get(&key).ok_or(Error::JobNotFound)
}

fn get_escrow_or_error(env: &Env, job_id: u64) -> Result<Escrow, Error> {
    let key = escrow_key(job_id);
    env.storage().persistent().get(&key).ok_or(Error::EscrowNotFound)
}

fn get_milestone_or_error(env: &Env, job_id: u64, index: u32) -> Result<Milestone, Error> {
    let key = milestone_key(job_id, index);
    env.storage().persistent().get(&key).ok_or(Error::MilestoneNotFound)
}

fn get_dispute_or_error(env: &Env, job_id: u64, dispute_id: u32) -> Result<Dispute, Error> {
    let key = dispute_key(job_id, dispute_id);
    env.storage().persistent().get(&key).ok_or(Error::DisputeNotFound)
}

fn update_reputation(
    env: &Env,
    address: &Address,
    job_completed: bool,
    attestations: u64,
    disputes: u64,
) -> Result<(), Error> {
    let key = reputation_key(address);
    let mut rep: Reputation = env.storage().persistent().get(&key).ok_or(Error::ReputationNotFound)?;

    let old_score = rep.score;

    if job_completed {
        rep.completed_jobs += 1;
    }
    rep.successful_attestations += attestations;
    rep.disputes_involved += disputes;
    rep.updated_at = current_timestamp(env);
    rep.score = rep.compute_score();

    env.storage().persistent().set(&key, &rep);

    Events::reputation_updated(env, address, old_score, rep.score);
    Ok(())
}

fn check_job_settlement(env: &Env, job_id: u64) -> Result<(), Error> {
    let job = get_job_or_error(env, job_id)?;
    let mut all_settled = true;

    for i in 0..job.milestone_count {
        let ms = get_milestone_or_error(env, job_id, i)?;
        if ms.status != MilestoneStatus::Released {
            all_settled = false;
            break;
        }
    }

    if all_settled {
        let mut job_mut = job;
        job_mut.status = JobStatus::Settled;
        job_mut.updated_at = current_timestamp(env);
        let job_k = job_key(job_id);
        env.storage().persistent().set(&job_k, &job_mut);
        Events::job_settled(env, job_id);
    }

    Ok(())
}

// ══════════════════════════════════════════════════════════════════════════════
// TESTS
// ══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::vec as soroban_vec;

    fn make_addr(env: &Env) -> Address {
        Address::generate(env)
    }

    fn init_client<'a>(env: &'a Env, admin: &Address, token: &Address) -> ProofFlowClient<'a> {
        let contract_id = env.register_contract(None, ProofFlow);
        let client = ProofFlowClient::new(env, &contract_id);
        client.initialize(admin, token);
        client
    }

    fn mk_client(client: &ProofFlowClient<'_>, name: &str) -> Address {
        let a = make_addr(&client.env);
        client.register_user(&a, &UserRole::Client, &String::from_str(&client.env, name));
        a
    }

    fn mk_worker(client: &ProofFlowClient<'_>, name: &str) -> Address {
        let a = make_addr(&client.env);
        client.register_user(&a, &UserRole::Worker, &String::from_str(&client.env, name));
        a
    }

    fn create_2ms(client: &ProofFlowClient<'_>, caller: &Address, worker: &Address) -> u64 {
        let env = &client.env;
        let titles = soroban_vec![env, String::from_str(env, "M1"), String::from_str(env, "M2")];
        let amounts = soroban_vec![env, 100u128, 200u128];
        let workers = soroban_vec![env, worker.clone(), worker.clone()];
        client.create_job(
            caller,
            &String::from_str(env, "Test Job"),
            &String::from_str(env, "Desc"),
            &titles,
            &amounts,
            &workers,
        )
    }

    // ── 1. Initialization ────────────────────────────────────────────────────

    #[test]
    fn init_sets_admin() {
        let env = Env::default();
        let admin = make_addr(&env);
        let token = make_addr(&env);
        let client = init_client(&env, &admin, &token);
        assert_eq!(client.get_admin(), admin);
    }

    #[test]
    fn init_no_double() {
        let env = Env::default();
        let a = make_addr(&env);
        let t = make_addr(&env);
        let client = init_client(&env, &a, &t);
        assert_eq!(client.try_initialize(&a, &t), Err(Ok(Error::AlreadyInitialized.into())));
    }

    #[test]
    fn pause_needs_admin() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let outsider = make_addr(&env);
        assert_eq!(client.try_pause(&outsider), Err(Ok(Error::NotAuthorized.into())));
    }

    #[test]
    fn unpause_needs_admin() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        client.pause(&admin);
        let outsider = make_addr(&env);
        assert_eq!(client.try_unpause(&outsider), Err(Ok(Error::NotAuthorized.into())));
    }

    #[test]
    fn transfer_admin_works() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let b = make_addr(&env);
        client.transfer_admin(&admin, &b);
        assert_eq!(client.get_admin(), b);
    }

    #[test]
    fn transfer_admin_needs_admin() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let outsider = make_addr(&env);
        let target = make_addr(&env);
        assert_eq!(
            client.try_transfer_admin(&outsider, &target),
            Err(Ok(Error::NotAuthorized.into()))
        );
    }

    // ── 2. User registration ─────────────────────────────────────────────────

    #[test]
    fn register_client() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let a = make_addr(&env);
        client.register_user(&a, &UserRole::Client, &String::from_str(&env, "C"));
        assert_eq!(client.get_user(&a).role, UserRole::Client);
    }

    #[test]
    fn register_worker() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let a = make_addr(&env);
        client.register_user(&a, &UserRole::Worker, &String::from_str(&env, "W"));
        assert_eq!(client.get_user(&a).role, UserRole::Worker);
    }

    #[test]
    fn register_verifier_role() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let a = make_addr(&env);
        client.register_user(&a, &UserRole::Verifier, &String::from_str(&env, "V"));
        assert_eq!(client.get_user(&a).role, UserRole::Verifier);
    }

    #[test]
    fn register_arbitrator() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let a = make_addr(&env);
        client.register_user(&a, &UserRole::Arbitrator, &String::from_str(&env, "A"));
        assert_eq!(client.get_user(&a).role, UserRole::Arbitrator);
    }

    #[test]
    fn register_empty_name_err() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let a = make_addr(&env);
        assert_eq!(
            client.try_register_user(&a, &UserRole::Client, &String::from_str(&env, "")),
            Err(Ok(Error::InvalidInput.into()))
        );
    }

    #[test]
    fn register_duplicate_err() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let a = make_addr(&env);
        client.register_user(&a, &UserRole::Client, &String::from_str(&env, "Alice"));
        // Second registration with same address must fail
        assert_eq!(
            client.try_register_user(&a, &UserRole::Worker, &String::from_str(&env, "Alice2")),
            Err(Ok(Error::UserAlreadyRegistered.into()))
        );
    }

    #[test]
    fn get_user_not_found() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let a = make_addr(&env);
        assert_eq!(client.try_get_user(&a), Err(Ok(Error::NotFound.into())));
    }

    // ── 3. Verifier management ───────────────────────────────────────────────

    #[test]
    fn add_remove_verifier() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let v = make_addr(&env);
        client.add_verifier(&admin, &v);
        assert!(client.is_verifier(&v));
        client.remove_verifier(&admin, &v);
        assert!(!client.is_verifier(&v));
    }

    #[test]
    fn add_verifier_dup_err() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let v = make_addr(&env);
        client.add_verifier(&admin, &v);
        assert_eq!(
            client.try_add_verifier(&admin, &v),
            Err(Ok(Error::VerifierAlreadyWhitelisted.into()))
        );
    }

    #[test]
    fn remove_verifier_not_found_err() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let v = make_addr(&env);
        assert_eq!(
            client.try_remove_verifier(&admin, &v),
            Err(Ok(Error::VerifierNotWhitelisted.into()))
        );
    }

    #[test]
    fn verifier_mgmt_needs_admin() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let outsider = make_addr(&env);
        let v = make_addr(&env);
        assert_eq!(
            client.try_add_verifier(&outsider, &v),
            Err(Ok(Error::NotAuthorized.into()))
        );
    }

    #[test]
    fn is_verifier_false_by_default() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let v = make_addr(&env);
        assert!(!client.is_verifier(&v));
    }

    // ── 4. Job creation ──────────────────────────────────────────────────────

    #[test]
    fn create_job_happy() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let c = mk_client(&client, "C");
        let w = mk_worker(&client, "W");
        let id = create_2ms(&client, &c, &w);
        let job = client.get_job(&id);
        assert_eq!(job.id, 0);
        assert_eq!(job.client, c);
        assert_eq!(job.status, JobStatus::Draft);
        assert_eq!(job.total_funded, 300);
        assert_eq!(job.milestone_count, 2);
    }

    #[test]
    fn create_job_empty_milestones_err() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let c = mk_client(&client, "C");
        let t = soroban_vec![&env];
        let a = soroban_vec![&env];
        let wv = soroban_vec![&env];
        assert_eq!(
            client.try_create_job(
                &c,
                &String::from_str(&env, "J"),
                &String::from_str(&env, "D"),
                &t,
                &a,
                &wv
            ),
            Err(Ok(Error::JobNotEnoughMilestones.into()))
        );
    }

    #[test]
    fn create_job_mismatched_err() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let c = mk_client(&client, "C");
        let w = mk_worker(&client, "W");
        let t = soroban_vec![&env, String::from_str(&env, "M1")];
        let a = soroban_vec![&env, 100u128, 200u128];
        let wv = soroban_vec![&env, w];
        assert_eq!(
            client.try_create_job(
                &c,
                &String::from_str(&env, "J"),
                &String::from_str(&env, "D"),
                &t,
                &a,
                &wv
            ),
            Err(Ok(Error::InvalidInput.into()))
        );
    }

    #[test]
    fn create_job_by_worker_err() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let w = mk_worker(&client, "W");
        let t = soroban_vec![&env, String::from_str(&env, "M1")];
        let a = soroban_vec![&env, 100u128];
        let wv = soroban_vec![&env, w.clone()];
        assert_eq!(
            client.try_create_job(
                &w,
                &String::from_str(&env, "J"),
                &String::from_str(&env, "D"),
                &t,
                &a,
                &wv
            ),
            Err(Ok(Error::NotAuthorized.into()))
        );
    }

    #[test]
    fn create_job_long_title_err() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let c = mk_client(&client, "C");
        let w = mk_worker(&client, "W");
        let long_title = String::from_str(&env, &"x".repeat(300));
        let t = soroban_vec![&env, String::from_str(&env, "M1")];
        let a = soroban_vec![&env, 100u128];
        let wv = soroban_vec![&env, w];
        assert_eq!(
            client.try_create_job(&c, &long_title, &String::from_str(&env, "D"), &t, &a, &wv),
            Err(Ok(Error::InvalidInput.into()))
        );
    }

    #[test]
    fn job_ids_sequential() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let c = mk_client(&client, "C");
        let w = mk_worker(&client, "W");
        assert_eq!(create_2ms(&client, &c, &w), 0);
        assert_eq!(create_2ms(&client, &c, &w), 1);
        assert_eq!(create_2ms(&client, &c, &w), 2);
    }

    #[test]
    fn get_job_not_found() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        assert_eq!(client.try_get_job(&999), Err(Ok(Error::JobNotFound.into())));
    }

    #[test]
    fn admin_can_create_jobs() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let w = mk_worker(&client, "W");
        let t = soroban_vec![&env, String::from_str(&env, "M1")];
        let a = soroban_vec![&env, 100u128];
        let wv = soroban_vec![&env, w];
        assert!(client
            .try_create_job(
                &admin,
                &String::from_str(&env, "J"),
                &String::from_str(&env, "D"),
                &t,
                &a,
                &wv
            )
            .is_ok());
    }

    // ── 5. Job cancel ────────────────────────────────────────────────────────

    #[test]
    fn cancel_job_draft() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let c = mk_client(&client, "C");
        let w = mk_worker(&client, "W");
        let id = create_2ms(&client, &c, &w);
        client.cancel_job(&c, &id, &String::from_str(&env, "Nope"));
        assert_eq!(client.get_job(&id).status, JobStatus::Cancelled);
    }

    #[test]
    fn cancel_job_wrong_client_err() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let c = mk_client(&client, "C");
        let o = mk_client(&client, "O");
        let w = mk_worker(&client, "W");
        let id = create_2ms(&client, &c, &w);
        assert_eq!(
            client.try_cancel_job(&o, &id, &String::from_str(&env, "r")),
            Err(Ok(Error::NotAuthorized.into()))
        );
    }

    #[test]
    fn cancel_nonexistent_err() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let c = mk_client(&client, "C");
        assert_eq!(
            client.try_cancel_job(&c, &999, &String::from_str(&env, "r")),
            Err(Ok(Error::JobNotFound.into()))
        );
    }

    // ── 6. Milestones ────────────────────────────────────────────────────────

    #[test]
    fn milestone_initial_state() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let c = mk_client(&client, "C");
        let w = mk_worker(&client, "W");
        let id = create_2ms(&client, &c, &w);
        let ms = client.get_milestone(&id, &0);
        assert_eq!(ms.status, MilestoneStatus::Pending);
        assert_eq!(ms.worker, w);
        assert_eq!(ms.amount, 100);
        assert_eq!(ms.job_id, id);
        assert_eq!(ms.index, 0);
    }

    #[test]
    fn milestone_second() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let c = mk_client(&client, "C");
        let w = mk_worker(&client, "W");
        let id = create_2ms(&client, &c, &w);
        let ms = client.get_milestone(&id, &1);
        assert_eq!(ms.amount, 200);
        assert_eq!(ms.index, 1);
    }

    #[test]
    fn milestone_bad_index() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let c = mk_client(&client, "C");
        let w = mk_worker(&client, "W");
        let id = create_2ms(&client, &c, &w);
        assert_eq!(
            client.try_get_milestone(&id, &5),
            Err(Ok(Error::MilestoneNotFound.into()))
        );
    }

    #[test]
    fn milestone_bad_job() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        assert_eq!(
            client.try_get_milestone(&999, &0),
            Err(Ok(Error::MilestoneNotFound.into()))
        );
    }

    // ── 7. Escrow ────────────────────────────────────────────────────────────

    #[test]
    fn escrow_created_with_job() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let c = mk_client(&client, "C");
        let w = mk_worker(&client, "W");
        let id = create_2ms(&client, &c, &w);
        let esc = client.get_escrow(&id);
        assert_eq!(esc.job_id, id);
        assert_eq!(esc.total_funded, 0);
        assert_eq!(esc.total_released, 0);
        assert_eq!(esc.total_frozen, 0);
        assert_eq!(esc.status, EscrowStatus::Created);
    }

    #[test]
    fn escrow_not_found() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        assert_eq!(client.try_get_escrow(&999), Err(Ok(Error::EscrowNotFound.into())));
    }

    #[test]
    fn escrow_remaining_invariant() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let c = mk_client(&client, "C");
        let w = mk_worker(&client, "W");
        let id = create_2ms(&client, &c, &w);
        let esc = client.get_escrow(&id);
        assert_eq!(
            esc.remaining(),
            esc.total_funded - esc.total_released - esc.total_frozen
        );
    }

    // ── 8. Reputation ────────────────────────────────────────────────────────

    #[test]
    fn rep_init_zero() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let w = mk_worker(&client, "W");
        let rep = client.get_reputation(&w);
        assert_eq!(rep.completed_jobs, 0);
        assert_eq!(rep.score, 0);
    }

    #[test]
    fn rep_not_found() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let a = make_addr(&env);
        assert_eq!(client.try_get_reputation(&a), Err(Ok(Error::ReputationNotFound.into())));
    }

    // ── 9. Pause ─────────────────────────────────────────────────────────────

    #[test]
    fn pause_blocks_register() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        client.pause(&admin);
        let a = make_addr(&env);
        assert_eq!(
            client.try_register_user(&a, &UserRole::Client, &String::from_str(&env, "X")),
            Err(Ok(Error::ContractPaused.into()))
        );
    }

    #[test]
    fn pause_blocks_job_create() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let c = mk_client(&client, "C");
        let w = mk_worker(&client, "W");
        client.pause(&admin);
        let t = soroban_vec![&env, String::from_str(&env, "M1")];
        let a = soroban_vec![&env, 100u128];
        let wv = soroban_vec![&env, w];
        assert_eq!(
            client.try_create_job(
                &c,
                &String::from_str(&env, "J"),
                &String::from_str(&env, "D"),
                &t,
                &a,
                &wv
            ),
            Err(Ok(Error::ContractPaused.into()))
        );
    }

    #[test]
    fn unpause_allows_ops() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        client.pause(&admin);
        client.unpause(&admin);
        let a = make_addr(&env);
        assert!(client
            .try_register_user(&a, &UserRole::Client, &String::from_str(&env, "X"))
            .is_ok());
    }

    // ── 10. Disputes ─────────────────────────────────────────────────────────

    #[test]
    fn file_dispute_bad_job() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let c = mk_client(&client, "C");
        assert_eq!(
            client.try_file_dispute(&c, &999, &0, &String::from_str(&env, "r")),
            Err(Ok(Error::JobNotFound.into()))
        );
    }

    #[test]
    fn get_dispute_not_found() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        assert_eq!(client.try_get_dispute(&999, &0), Err(Ok(Error::DisputeNotFound.into())));
    }

    // ── 11. Multi-milestone / multi-job ──────────────────────────────────────

    #[test]
    fn three_milestones_two_workers() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let c = mk_client(&client, "C");
        let w1 = mk_worker(&client, "W1");
        let w2 = mk_worker(&client, "W2");
        let titles = soroban_vec![
            &env,
            String::from_str(&env, "A"),
            String::from_str(&env, "B"),
            String::from_str(&env, "C")
        ];
        let amounts = soroban_vec![&env, 50u128, 150u128, 300u128];
        let workers = soroban_vec![&env, w1.clone(), w2.clone(), w1.clone()];
        let id = client.create_job(
            &c,
            &String::from_str(&env, "Big"),
            &String::from_str(&env, "D"),
            &titles,
            &amounts,
            &workers,
        );
        assert_eq!(client.get_job(&id).total_funded, 500);
        assert_eq!(client.get_milestone(&id, &0).worker, w1);
        assert_eq!(client.get_milestone(&id, &1).worker, w2);
        assert_eq!(client.get_milestone(&id, &2).worker, w1);
    }

    #[test]
    fn two_jobs_independent() {
        let env = Env::default();
        let admin = make_addr(&env);
        let client = init_client(&env, &admin, &make_addr(&env));
        let c1 = mk_client(&client, "C1");
        let c2 = mk_client(&client, "C2");
        let w = mk_worker(&client, "W");
        let id1 = create_2ms(&client, &c1, &w);
        let id2 = create_2ms(&client, &c2, &w);
        assert_ne!(id1, id2);
        assert_eq!(client.get_job(&id1).client, c1);
        assert_eq!(client.get_job(&id2).client, c2);
    }

    // ── 12. State transition matrix (unit tests on enums) ────────────────────

    #[test]
    fn job_status_terminal() {
        assert!(JobStatus::Settled.is_terminal());
        assert!(JobStatus::Cancelled.is_terminal());
        assert!(!JobStatus::Draft.is_terminal());
        assert!(!JobStatus::Funded.is_terminal());
        assert!(!JobStatus::Active.is_terminal());
        assert!(!JobStatus::InReview.is_terminal());
        assert!(!JobStatus::Disputed.is_terminal());
    }

    #[test]
    fn ms_can_submit() {
        assert!(MilestoneStatus::Pending.can_submit());
        assert!(MilestoneStatus::Rejected.can_submit());
        assert!(!MilestoneStatus::Submitted.can_submit());
        assert!(!MilestoneStatus::Approved.can_submit());
        assert!(!MilestoneStatus::Released.can_submit());
        assert!(!MilestoneStatus::Disputed.can_submit());
    }

    #[test]
    fn ms_can_settle() {
        assert!(MilestoneStatus::Approved.can_settle());
        assert!(!MilestoneStatus::Pending.can_settle());
        assert!(!MilestoneStatus::Submitted.can_settle());
        assert!(!MilestoneStatus::Rejected.can_settle());
        assert!(!MilestoneStatus::Released.can_settle());
        assert!(!MilestoneStatus::Disputed.can_settle());
    }

    // ── 13. Escrow unit tests ────────────────────────────────────────────────

    #[test]
    fn escrow_can_release() {
        let mut esc = Escrow {
            job_id: 0,
            total_funded: 1000,
            total_released: 0,
            total_frozen: 0,
            status: EscrowStatus::Funded,
        };
        assert!(esc.can_release(500));
        assert!(esc.can_release(1000));
        assert!(!esc.can_release(1001));
        esc.total_released = 800;
        assert!(esc.can_release(200));
        assert!(!esc.can_release(201));
        esc.status = EscrowStatus::Frozen;
        assert!(!esc.can_release(1));
    }

    #[test]
    fn escrow_remaining_table() {
        let check = |funded: u128, released: u128, frozen: u128, expected: u128| {
            let esc = Escrow {
                job_id: 0,
                total_funded: funded,
                total_released: released,
                total_frozen: frozen,
                status: EscrowStatus::Funded,
            };
            assert_eq!(esc.remaining(), expected);
        };
        check(1000, 0, 0, 1000);
        check(1000, 300, 0, 700);
        check(1000, 300, 200, 500);
        check(1000, 1000, 0, 0);
        check(0, 0, 0, 0);
    }

    // ── 14. Reputation formula (unit tests) ──────────────────────────────────

    #[test]
    fn rep_score_basic() {
        let env = Env::default();
        let rep = Reputation {
            address: make_addr(&env),
            completed_jobs: 5,
            successful_attestations: 3,
            disputes_involved: 2,
            disputes_won: 1,
            total_earned: 500,
            score: 0,
            updated_at: 0,
        };
        assert_eq!(rep.compute_score(), 62);
    }

    #[test]
    fn rep_score_no_disputes() {
        let env = Env::default();
        let rep = Reputation {
            address: make_addr(&env),
            completed_jobs: 10,
            successful_attestations: 0,
            disputes_involved: 0,
            disputes_won: 0,
            total_earned: 1000,
            score: 0,
            updated_at: 0,
        };
        assert_eq!(rep.compute_score(), 100);
    }

    #[test]
    fn rep_score_all_won() {
        let env = Env::default();
        let rep = Reputation {
            address: make_addr(&env),
            completed_jobs: 2,
            successful_attestations: 1,
            disputes_involved: 5,
            disputes_won: 5,
            total_earned: 200,
            score: 0,
            updated_at: 0,
        };
        assert_eq!(rep.compute_score(), 25);
    }

    #[test]
    fn rep_score_all_lost_saturates_zero() {
        let env = Env::default();
        let rep = Reputation {
            address: make_addr(&env),
            completed_jobs: 1,
            successful_attestations: 0,
            disputes_involved: 5,
            disputes_won: 0,
            total_earned: 100,
            score: 0,
            updated_at: 0,
        };
        assert_eq!(rep.compute_score(), 0);
    }

    #[test]
    fn rep_score_zero() {
        let env = Env::default();
        let rep = Reputation {
            address: make_addr(&env),
            completed_jobs: 0,
            successful_attestations: 0,
            disputes_involved: 0,
            disputes_won: 0,
            total_earned: 0,
            score: 0,
            updated_at: 0,
        };
        assert_eq!(rep.compute_score(), 0);
    }
}
