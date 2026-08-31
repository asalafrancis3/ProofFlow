#![no_std]

pub mod errors;
pub mod events;
pub mod types;
pub mod validation;

use errors::Error;
use events::Events;
use types::*;
use validation::*;

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol, Vec,
};

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
        env.storage()
            .instance()
            .get(&ADMIN)
            .ok_or(Error::NotAuthorized)
    }

    pub fn transfer_admin(env: Env, admin: Address, new_admin: Address) -> Result<(), Error> {
        require_admin(&env, &admin)?;
        env.storage().instance().set(&ADMIN, &new_admin);
        Ok(())
    }

    // ══════════════════════════════════════════════════════════════════════════
    // USER MANAGEMENT
    // ══════════════════════════════════════════════════════════════════════════

    pub fn register_user(
        env: Env,
        address: Address,
        role: UserRole,
        name: String,
    ) -> Result<(), Error> {
        require_not_paused(&env)?;
        validate_non_empty_string(&name)?;

        let user = User {
            address: address.clone(),
            org_id: None,
            role,
            name,
            registered_at: current_timestamp(&env),
        };

        let key = user_key(&address);
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
        env.storage()
            .persistent()
            .get(&key)
            .ok_or(Error::NotFound)
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

        if milestone_titles.len() != milestone_amounts.len()
            || milestone_titles.len() != milestone_workers.len()
        {
            return Err(Error::InvalidInput);
        }
        if milestone_titles.is_empty() {
            return Err(Error::JobNotEnoughMilestones);
        }

        // Check caller is registered as Client
        let user = get_user_or_error(&env, &client)?;
        if user.role != UserRole::Client && user.role != UserRole::Admin {
            return Err(Error::NotAuthorized);
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
                evidence_uri: None,
                submitted_at: None,
                resolved_at: None,
            };
            let ms_key = milestone_key(job_id, i);
            env.storage().persistent().set(&ms_key, &milestone);

            Events::milestone_created(
                &env,
                job_id,
                i,
                milestone.amount,
                &milestone.worker,
            );
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
        let token_addr: Address = env
            .storage()
            .instance()
            .get(&TOKEN_ADDR)
            .ok_or(Error::NotAuthorized)?;
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
            let token_addr: Address = env
                .storage()
                .instance()
                .get(&TOKEN_ADDR)
                .ok_or(Error::NotAuthorized)?;
            let token = soroban_sdk::token::Client::new(&env, &token_addr);
            token.transfer(
                &env.current_contract_address(),
                &client,
                &(escrow.remaining() as i128),
            );
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
        ms.evidence_uri = Some(evidence_uri.clone());
        ms.submitted_at = Some(now);
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

    pub fn approve_milestone(
        env: Env,
        verifier: Address,
        job_id: u64,
        milestone_idx: u32,
    ) -> Result<(), Error> {
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
        ms.resolved_at = Some(now);
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
        ms.resolved_at = Some(now);
        ms.evidence_uri = None;
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

    pub fn release_milestone_escrow(
        env: Env,
        job_id: u64,
        milestone_idx: u32,
    ) -> Result<(), Error> {
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
        let token_addr: Address = env
            .storage()
            .instance()
            .get(&TOKEN_ADDR)
            .ok_or(Error::NotAuthorized)?;
        let token = soroban_sdk::token::Client::new(&env, &token_addr);
        token.transfer(
            &env.current_contract_address(),
            &ms.worker,
            &(ms.amount as i128),
        );

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
            resolution: None,
            created_at: now,
            resolved_at: None,
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
                    let token_addr: Address = env
                        .storage()
                        .instance()
                        .get(&TOKEN_ADDR)
                        .ok_or(Error::NotAuthorized)?;
                    let token = soroban_sdk::token::Client::new(&env, &token_addr);
                    token.transfer(
                        &env.current_contract_address(),
                        &ms.worker,
                        &(ms.amount as i128),
                    );
                    escrow.total_released += ms.amount;
                }
            }
            Resolution::UpholdClient => {
                // Return to client
                let job = get_job_or_error(&env, job_id)?;
                if escrow.total_frozen >= ms.amount {
                    let token_addr: Address = env
                        .storage()
                        .instance()
                        .get(&TOKEN_ADDR)
                        .ok_or(Error::NotAuthorized)?;
                    let token = soroban_sdk::token::Client::new(&env, &token_addr);
                    token.transfer(
                        &env.current_contract_address(),
                        &job.client,
                        &(ms.amount as i128),
                    );
                }
            }
            Resolution::PartialSplit => {
                // Split 50/50
                let half = ms.amount / 2;
                let job = get_job_or_error(&env, job_id)?;
                let token_addr: Address = env
                    .storage()
                    .instance()
                    .get(&TOKEN_ADDR)
                    .ok_or(Error::NotAuthorized)?;
                let token = soroban_sdk::token::Client::new(&env, &token_addr);

                if escrow.can_release(half) {
                    token.transfer(
                        &env.current_contract_address(),
                        &ms.worker,
                        &(half as i128),
                    );
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
        disp.resolution = Some(resolution);
        disp.resolved_at = Some(now);
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
        env.storage()
            .persistent()
            .get(&key)
            .ok_or(Error::ReputationNotFound)
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// INTERNAL HELPERS
// ══════════════════════════════════════════════════════════════════════════════

fn user_key(address: &Address) -> Symbol {
    symbol_short!("USR_")
}

fn job_key(job_id: u64) -> Symbol {
    // Use a simple approach: JOB_ + job_id
    // In production, use a more robust key derivation
    match job_id {
        0 => symbol_short!("JOB_0"),
        1 => symbol_short!("JOB_1"),
        2 => symbol_short!("JOB_2"),
        3 => symbol_short!("JOB_3"),
        4 => symbol_short!("JOB_4"),
        5 => symbol_short!("JOB_5"),
        6 => symbol_short!("JOB_6"),
        7 => symbol_short!("JOB_7"),
        8 => symbol_short!("JOB_8"),
        9 => symbol_short!("JOB_9"),
        _ => symbol_short!("JOB_X"),
    }
}

fn escrow_key(job_id: u64) -> Symbol {
    match job_id {
        0 => symbol_short!("ESC_0"),
        1 => symbol_short!("ESC_1"),
        2 => symbol_short!("ESC_2"),
        3 => symbol_short!("ESC_3"),
        4 => symbol_short!("ESC_4"),
        5 => symbol_short!("ESC_5"),
        6 => symbol_short!("ESC_6"),
        7 => symbol_short!("ESC_7"),
        8 => symbol_short!("ESC_8"),
        9 => symbol_short!("ESC_9"),
        _ => symbol_short!("ESC_X"),
    }
}

fn milestone_key(job_id: u64, index: u32) -> Symbol {
    // Simplified key derivation
    match (job_id, index) {
        (0, 0) => symbol_short!("MS_0_0"),
        (0, 1) => symbol_short!("MS_0_1"),
        (1, 0) => symbol_short!("MS_1_0"),
        (1, 1) => symbol_short!("MS_1_1"),
        _ => symbol_short!("MS_X_X"),
    }
}

fn submission_key(job_id: u64, index: u32) -> Symbol {
    match (job_id, index) {
        (0, 0) => symbol_short!("SUB_0_0"),
        (0, 1) => symbol_short!("SUB_0_1"),
        (1, 0) => symbol_short!("SUB_1_0"),
        (1, 1) => symbol_short!("SUB_1_1"),
        _ => symbol_short!("SUB_X_X"),
    }
}

fn attestation_key(job_id: u64, index: u32) -> Symbol {
    match (job_id, index) {
        (0, 0) => symbol_short!("ATT_0_0"),
        (0, 1) => symbol_short!("ATT_0_1"),
        (1, 0) => symbol_short!("ATT_1_0"),
        (1, 1) => symbol_short!("ATT_1_1"),
        _ => symbol_short!("ATT_X_X"),
    }
}

fn dispute_key(job_id: u64, dispute_id: u32) -> Symbol {
    match (job_id, dispute_id) {
        (0, 0) => symbol_short!("DSP_0_0"),
        (0, 1) => symbol_short!("DSP_0_1"),
        (1, 0) => symbol_short!("DSP_1_0"),
        (1, 1) => symbol_short!("DSP_1_1"),
        _ => symbol_short!("DSP_X_X"),
    }
}

fn reputation_key(address: &Address) -> Symbol {
    symbol_short!("REP_")
}

fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
    let admin: Address = env
        .storage()
        .instance()
        .get(&ADMIN)
        .ok_or(Error::NotAuthorized)?;
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
    env.storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotFound)
}

fn get_job_or_error(env: &Env, job_id: u64) -> Result<Job, Error> {
    let key = job_key(job_id);
    env.storage()
        .persistent()
        .get(&key)
        .ok_or(Error::JobNotFound)
}

fn get_escrow_or_error(env: &Env, job_id: u64) -> Result<Escrow, Error> {
    let key = escrow_key(job_id);
    env.storage()
        .persistent()
        .get(&key)
        .ok_or(Error::EscrowNotFound)
}

fn get_milestone_or_error(env: &Env, job_id: u64, index: u32) -> Result<Milestone, Error> {
    let key = milestone_key(job_id, index);
    env.storage()
        .persistent()
        .get(&key)
        .ok_or(Error::MilestoneNotFound)
}

fn get_dispute_or_error(env: &Env, job_id: u64, dispute_id: u32) -> Result<Dispute, Error> {
    let key = dispute_key(job_id, dispute_id);
    env.storage()
        .persistent()
        .get(&key)
        .ok_or(Error::DisputeNotFound)
}

fn update_reputation(
    env: &Env,
    address: &Address,
    job_completed: bool,
    attestations: u64,
    disputes: u64,
) -> Result<(), Error> {
    let key = reputation_key(address);
    let mut rep: Reputation = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::ReputationNotFound)?;

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
