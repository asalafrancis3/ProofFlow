# ProofFlow Architecture

## Overview

ProofFlow is a decentralized verification and milestone settlement protocol on Stellar/Soroban. It enables structured job contracts with escrow, evidence-based milestone approval, independent verification, dispute arbitration, and on-chain reputation.

## System Layers

```
┌─────────────────────────────────────────────────────────────────┐
│                       Frontend (React/Vite)                      │
│  Pages: Dashboard, Jobs, Create Job, Job Detail, Verification,  │
│         Reputation, Activity, Settings, Admin                    │
│  Infrastructure: React Query, Wallet Integration, Auth, PWA     │
└──────────────────────────┬──────────────────────────────────────┘
                           │ REST API
┌──────────────────────────▼──────────────────────────────────────┐
│                     Backend (Actix-Web)                          │
│  Routes: 17 REST endpoints (proofflow.rs)                       │
│  Services: Domain model, Error hierarchy, Contract adapter      │
│  Middleware: Rate limiting, Idempotency, CORS, Auth             │
└──────────────────────────┬──────────────────────────────────────┘
                           │ Contract calls / Event subscription
┌──────────────────────────▼──────────────────────────────────────┐
│               Indexer (Event → State)                            │
│  Decoder: 21 event types with typed payloads                    │
│  Processor: Deterministic state transitions                     │
│  Persistence: Redis/Postgres projections                        │
└──────────────────────────┬──────────────────────────────────────┘
                           │ Soroban SDK
┌──────────────────────────▼──────────────────────────────────────┐
│              Stellar Contract (Soroban/Rust)                     │
│  Modules: types, errors, events, validation, storage, lib       │
│  Entry points: 14 mutations, 7 queries                          │
│  Storage: Deterministic composite keys                          │
│  Events: 21 typed event symbols                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Domain Model

### Entities

| Entity | Description |
|--------|-------------|
| **User** | Registered participant with a role (Client, Worker, Verifier, Arbitrator, Admin) |
| **Job** | Work definition with title, description, milestones, and funding status |
| **Milestone** | Discrete unit of work within a job, with amount, worker, and status |
| **Escrow** | Per-job fund tracking (funded, released, frozen amounts) |
| **Dispute** | Disagreement on a milestone, with resolution workflow |
| **Reputation** | On-chain scoring based on completed jobs, attestations, disputes |

### State Machines

**Job**: `Draft → Funded → Active → InReview → Settled` (or `Cancelled`/`Disputed`)

**Milestone**: `Pending → Submitted → Approved → Released` (or `Rejected`/`Disputed`)

**Escrow**: `Created → Funded → PartialRelease → Completed` (or `Frozen`)

**Dispute**: `Filed → UnderReview → Resolved`

### Roles

| Role | Permissions |
|------|------------|
| **Client** | Create jobs, fund escrow, approve milestones |
| **Worker** | Submit milestone evidence |
| **Verifier** | Attest to milestone completion |
| **Arbitrator** | Resolve disputes |
| **Admin** | Register users, manage system |

## Contract Architecture

### Storage Keys

All storage uses deterministic composite keys:

```rust
// User storage
user_key(address) → (Symbol, Address)

// Job storage
job_key(job_id) → (Symbol, u64)

// Milestone storage
milestone_key(job_id, idx) → (Symbol, u64, u32)

// Escrow storage
escrow_key(job_id) → (Symbol, u64)

// Dispute storage
dispute_key(job_id, dispute_id) → (Symbol, u64, u32)

// Reputation storage
reputation_key(address) → (Symbol, Address)
```

### Event Architecture

21 event types with ≤9-character symbols:

| Symbol | Event |
|--------|-------|
| `JOB_CR` | JobCreated |
| `JOB_FND` | JobFunded |
| `JOB_ACT` | JobActivated |
| `JOB_CNC` | JobCancelled |
| `JOB_STL` | JobSettled |
| `MS_CR` | MilestoneCreated |
| `MS_SUB` | MilestoneSubmitted |
| `MS_APR` | MilestoneApproved |
| `MS_REJ` | MilestoneRejected |
| `MS_RLS` | MilestoneReleased |
| `ESC_CR` | EscrowCreated |
| `ESC_FND` | EscrowFunded |
| `ESC_RLS` | EscrowReleased |
| `ESC_FRZ` | EscrowFrozen |
| `ESC_UNF` | EscrowUnfrozen |
| `DISP_FL` | DisputeFiled |
| `DISP_RS` | DisputeResolved |
| `REP_UPD` | ReputationUpdated |
| `USR_REG` | UserRegistered |
| `VER_ADD` | VerifierAdded |
| `VER_REM` | VerifierRemoved |

### Authorization Model

- `create_job`: Contract admin OR registered Client role
- `fund_job`: Job client only
- `submit_milestone`: Assigned worker only
- `approve_milestone`: Job client OR registered Verifier
- `release_payment`: Contract admin only
- `file_dispute`: Assigned worker OR job client
- `resolve_dispute`: Registered Arbitrator only

## Backend Architecture

### API Layer

17 REST routes organized by domain:

| Domain | Routes |
|--------|--------|
| User | `POST /register`, `GET /user/:addr` |
| Job | `POST /jobs`, `GET /jobs`, `GET /jobs/:id`, `PATCH /jobs/:id/activate`, `PATCH /jobs/:id/cancel` |
| Milestone | `GET /jobs/:id/milestones`, `POST /jobs/:id/milestones/:idx/submit`, `POST /jobs/:id/milestones/:idx/approve`, `POST /jobs/:id/milestones/:idx/reject` |
| Escrow | `GET /escrow/:job_id`, `POST /escrow/:job_id/fund`, `POST /escrow/:job_id/release` |
| Dispute | `POST /disputes`, `GET /disputes/:id`, `POST /disputes/:id/resolve` |
| Reputation | `GET /reputation/:addr` |
| System | `GET /health` |

### Error Architecture

Structured error hierarchy with HTTP status code mapping:

```
ServiceError
├── Contract (RpcError, ContractError)
├── Validation (MissingField, InvalidFormat, OutOfRange)
├── Auth (Unauthorized, Forbidden, AdminRequired)
├── Not Found (UserNotFound, JobNotFound, ...)
├── Conflict (AlreadyRegistered, AlreadySubmitted, ...)
├── State (WrongJobStatus, WrongMilestoneStatus, ...)
└── Infrastructure (DatabaseError, Timeout, RateLimited)
```

## Frontend Architecture

### Pages

| Page | Purpose |
|------|---------|
| LandingPage | Public marketing page |
| LoginPage | Wallet connect + role selection |
| DashboardPage | Role-aware overview with stats |
| JobsPage | Filterable job list |
| CreateJobPage | Multi-milestone job creation |
| JobDetailPage | Job overview + milestone management |
| VerificationPage | Review queue for verifiers |
| ReputationPage | User reputation display |
| ActivityPage | Activity feed |
| SettingsPage | User settings |
| AdminDashboardPage | Admin overview |

### Data Flow

```
Page → useQuery/useMutation → API Client → REST → Backend → Contract/Adapter
```

React Query handles caching, invalidation, and optimistic updates.

## Testing

| Layer | Framework | Count |
|-------|-----------|-------|
| Contract unit tests | `cargo test` | 55 |
| Backend unit tests | `cargo test` | 393+ |
| Frontend critical tests | Vitest | 11 |
| API contract alignment | Manual verification | 16 types |
