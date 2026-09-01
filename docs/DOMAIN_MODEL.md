# Domain Model — ProofFlow

> Decentralized verification and milestone settlement for field work on Stellar/Soroban.

## Product Vision

ProofFlow is a protocol for creating funded work agreements where:
- **Clients** define jobs with funded milestones
- **Workers** perform work and submit evidence
- **Verifiers** attest to completion
- **Escrow** releases programmatically upon verification
- **Reputation** evolves from completed/verified work
- **Disputes** can freeze or redirect settlement

The key differentiator: **verified work → programmable settlement** — not another freelance marketplace, but a trust and settlement protocol.

---

## Actors

| Actor | Description | Can Do |
|-------|-------------|--------|
| **Client** | Creates and funds jobs | Create job, fund escrow, approve/reject milestones, initiate dispute |
| **Worker** | Performs work, submits evidence | Accept job, submit evidence, view settlements |
| **Verifier** | Reviews evidence, attests to completion | Review evidence, approve/reject work, view verification history |
| **Arbitrator** | Resolves disputes | Review dispute, issue resolution, redirect settlement |
| **Admin** | Protocol governance | Update parameters, manage verifier whitelist |

---

## Core Entities

### Organization
A group of users under a shared identity.

```
Organization {
  id:            Address
  name:          String
  created_at:    u64
  members:       Vec<Address>
}
```

### User (Participant)
A single actor in the system.

```
User {
  address:       Address
  org_id:        Option<Address>
  role:          UserRole  // Client | Worker | Verifier | Arbitrator | Admin
  name:          String
  reputation:    Reputation
  registered_at: u64
}
```

### Job
A funded work agreement between a client and one or more workers.

```
Job {
  id:            u64
  client:        Address
  title:         String
  description:   String
  status:        JobStatus
  total_funded:  u128
  created_at:    u64
  updated_at:    u64
}
```

### Milestone
A discrete unit of work within a job, with its own funding and settlement path.

```
Milestone {
  job_id:        u64
  index:         u32
  title:         String
  description:   String
  amount:        u128
  status:        MilestoneStatus
  worker:        Address
  evidence_uri:  Option<String>
  submitted_at:  Option<u64>
  resolved_at:   Option<u64>
}
```

### Escrow
Funds held on-chain for a job's milestones.

```
Escrow {
  job_id:         u64
  total_funded:   u128
  total_released: u128
  total_frozen:   u128
  status:         EscrowStatus
}
```

### Submission
A worker's evidence of work completion for a specific milestone.

```
Submission {
  job_id:        u64
  milestone_idx: u32
  worker:        Address
  evidence_uri:  String
  notes:         String
  submitted_at:  u64
}
```

### Attestation
A verifier's assessment of a submission.

```
Attestation {
  job_id:        u64
  milestone_idx: u32
  verifier:      Address
  outcome:       AttestationOutcome  // Approved | Rejected
  notes:         String
  created_at:    u64
}
```

### Dispute
A challenge raised by a client or worker against a milestone outcome.

```
Dispute {
  job_id:        u64
  milestone_idx: u32
  dispute_id:    u32
  raised_by:     Address
  reason:        String
  status:        DisputeStatus
  resolution:    Option<Resolution>
  created_at:    u64
  resolved_at:   Option<u64>
}
```

### Reputation
A user's on-chain trust score derived from completed work.

```
Reputation {
  address:               Address
  completed_jobs:        u64
  successful_attestations: u64
  disputes_involved:     u64
  disputes_won:          u64
  total_earned:          u128
  score:                 u64  // computed from above
  updated_at:            u64
}
```

---

## State Machines

### Job Lifecycle

```
                    ┌──────────┐
                    │  Draft   │
                    └────┬─────┘
                         │ fund()
                    ┌────▼─────┐
                    │  Funded  │
                    └────┬─────┘
                         │ activate()
                    ┌────▼─────┐
                    │  Active  │◄──────────────┐
                    └────┬─────┘               │
                         │ submit_evidence()    │
                    ┌────▼─────┐               │
                    │ InReview │───────────────┘
                    └────┬─────┘  reject()
                         │ approve()
                    ┌────▼─────┐
                    │ Settled  │
                    └──────────┘

  Any state → Cancelled (client only, before first submission)
  Any state → Disputed (by either party)
```

### Milestone Lifecycle

```
    ┌─────────┐
    │ Pending │
    └────┬────┘
         │ submit()
    ┌────▼──────┐
    │ Submitted │
    └────┬──────┘
         │ approve()          │ reject()
    ┌────▼─────┐         ┌───▼────────┐
    │ Approved │         │  Rejected  │──→ resubmit() → Submitted
    └────┬─────┘         └────────────┘
         │ settle()
    ┌────▼───────┐
    │  Released  │
    └────────────┘

    Any non-Released state → Disputed
```

### Escrow Lifecycle

```
    ┌──────────┐
    │ Created  │
    └────┬─────┘
         │ fund()
    ┌────▼─────┐
    │  Funded  │◄──────────────────┐
    └────┬─────┘                   │
         │ release_milestone()     │
    ┌────▼────────────┐            │
    │ PartialRelease  │────────────┘
    └────┬────────────┘  (more milestones to release)
         │ release_final()
    ┌────▼──────────┐
    │  Completed    │
    └───────────────┘

    Any non-Completed state → Frozen (by dispute)
    Frozen → Released/Returned (after resolution)
```

### Dispute Lifecycle

```
    ┌──────────┐
    │  Filed   │
    └────┬─────┘
         │ assign_arbitrator()
    ┌────▼──────────┐
    │  UnderReview  │
    └────┬──────────┘
         │ resolve()
    ┌────▼──────────┐
    │   Resolved    │
    └───────────────┘

    Resolution types:
    - UpholdWorker: release escrow to worker
    - UpholdClient: return escrow to client
    - PartialSplit: split escrow between parties
```

---

## Authorization Rules

| Action | Who Can Do It |
|--------|--------------|
| Create job | Client |
| Fund escrow | Client (job creator) |
| Cancel job | Client (before first submission) |
| Submit evidence | Worker (assigned to milestone) |
| Approve milestone | Verifier (whitelisted) |
| Reject milestone | Verifier (whitelisted) |
| Initiate dispute | Client or Worker |
| Resolve dispute | Arbitrator (assigned) |
| Update config | Admin |
| Add/remove verifier | Admin |
| Update reputation | System (automatic on settlement) |

---

## Economic Model

- Clients fund escrow in native Stellar tokens (XLM or custom token)
- Milestone amounts sum to job total funding
- On milestone approval, escrow releases to worker
- On dispute freeze, escrow holds until resolution
- No platform fees in base protocol (can be added via admin config)
- Reputation score is computed, not earned through token staking

---

## Event Taxonomy

```
JOB_CREATED          { job_id, client, title, total_funded }
JOB_FUNDED           { job_id, amount }
JOB_ACTIVATED        { job_id }
JOB_CANCELLED        { job_id, reason }
JOB_SETTLED          { job_id }

MILESTONE_CREATED    { job_id, index, title, amount, worker }
MILESTONE_SUBMITTED  { job_id, index, worker, evidence_uri }
MILESTONE_APPROVED   { job_id, index, verifier }
MILESTONE_REJECTED   { job_id, index, verifier, reason }
MILESTONE_RELEASED   { job_id, index, amount, worker }

ESCROW_CREATED       { job_id }
ESCROW_FUNDED        { job_id, amount }
ESCROW_RELEASED      { job_id, milestone_idx, amount, recipient }
ESCROW_FROZEN        { job_id, dispute_id }
ESCROW_UNFROZEN      { job_id, dispute_id }

DISPUTE_FILED        { job_id, milestone_idx, dispute_id, raised_by, reason }
DISPUTE_RESOLVED     { job_id, milestone_idx, dispute_id, resolution }

REPUTATION_UPDATED   { address, old_score, new_score, trigger }

VERIFIER_ADDED       { address, added_by }
VERIFIER_REMOVED     { address, removed_by }
```

---

## Contract Invariants

1. **No double settlement**: A milestone cannot be released twice. Releasing an already-released milestone returns `MilestoneAlreadyReleased` error.
2. **No over-release**: Total escrow releases cannot exceed total funded. `Escrow::can_release()` checks sufficient balance.
3. **Authorization**: Only authorized parties can modify state. Admin-only ops require `require_admin()`. Role-based access enforced on job creation, evidence submission, milestone approval, and dispute resolution.
4. **State transitions**: Invalid state transitions are rejected. Every mutation checks current status before allowing transition.
5. **Dispute isolation**: A disputed milestone's escrow is frozen independently. Only the disputed milestone's amount is frozen.
6. **Settlement idempotency**: Releasing an already-released milestone returns an error (not a no-op). The contract rejects duplicate releases with `MilestoneAlreadyReleased`.
7. **Verifier authorization**: Only whitelisted verifiers can attest. `NotWhitelistedVerifier` error returned for unauthorized verifiers.
8. **Escrow accounting**: `total_funded == total_released + total_frozen + remaining`. `Escrow::remaining()` uses `saturating_sub` to prevent underflow.
9. **Reputation integrity**: Reputation updates only occur after valid settlement. Score changes are tracked with old/new values in events.
10. **Duplicate registration prevention**: Each address can only be registered once. `register_user` returns `UserAlreadyRegistered` if the address already exists.
11. **Input validation**: All string inputs are validated for non-empty, length limits (title: 1-256, description: max 4096). Array lengths must match milestone count.
