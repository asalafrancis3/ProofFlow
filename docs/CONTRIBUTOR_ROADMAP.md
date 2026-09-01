# Contributor Roadmap

This document identifies legitimate engineering tasks for ProofFlow contributors. Tasks are prioritized by impact and organized by difficulty.

---

## High-Priority: Contract & Backend Hardening

### 1. Persistent Indexer with Event Replay

**Problem:** The indexer currently decodes events but doesn't persist state projections. If the indexer restarts, all queryable state is lost.

**Expected behavior:**
- Events decode into domain objects and persist to a queryable store (Redis/Postgres)
- On restart, the indexer replays from the last checkpoint
- Projections are rebuilt from the event log

**Technical context:**
- `backend/src/indexer/processor.rs` — processes decoded events
- `backend/src/indexer/event_decoder.rs` — decodes on-chain events
- Storage adapter needs to support upsert operations for jobs, milestones, escrow, disputes, reputation

**Acceptance criteria:**
- Jobs, milestones, escrow, disputes, and reputation are queryable after indexing
- Restarting the indexer doesn't lose state
- Event replay produces identical projections

**Relevant files:**
- `backend/src/indexer/`
- `backend/src/services/domain.rs`
- `backend/src/contracts/adapter.rs`

**Difficulty:** Medium-Hard

---

### 2. Production Contract Adapter

**Problem:** The contract adapter (`backend/src/contracts/adapter.rs`) defines a trait but the production implementation may need hardening for error handling, retry logic, and network resilience.

**Expected behavior:**
- Adapter handles Stellar RPC failures gracefully
- Retries transient errors with exponential backoff
- Timeout handling for slow network conditions
- Proper error classification (transient vs permanent)

**Technical context:**
- `backend/src/contracts/adapter.rs` — trait definition
- `backend/src/rpc/client.rs` — Stellar RPC client

**Acceptance criteria:**
- Adapter handles network timeouts
- Retries are configurable
- Error types distinguish transient from permanent failures

**Relevant files:**
- `backend/src/contracts/adapter.rs`
- `backend/src/rpc/client.rs`
- `backend/src/services/error_model.rs`

**Difficulty:** Medium

---

### 3. Contract Property-Based Testing

**Problem:** The contract has 55 unit tests covering happy paths and some negative paths, but property-based testing could catch edge cases in state transitions and accounting invariants.

**Expected behavior:**
- Generate random sequences of contract operations
- Verify accounting invariants hold after every sequence
- Verify state machine transitions are always valid
- Verify no funds are created or destroyed

**Technical context:**
- `stellar-contract/src/lib.rs` — contract entry points
- `stellar-contract/src/types.rs` — domain types
- Use `proptest` or `quickcheck` for Rust

**Acceptance criteria:**
- Property tests run and pass
- At least 3 invariants are tested:
  1. Escrow: `total_funded == total_released + remaining`
  2. Job status: no invalid transitions
  3. Milestone: approved count ≤ total milestones

**Relevant files:**
- `stellar-contract/src/lib.rs`
- `stellar-contract/Cargo.toml`

**Difficulty:** Medium

---

### 4. E2E Workflow Tests

**Problem:** Frontend critical tests verify individual pages render, but don't test complete user workflows (create job → fund → submit milestone → approve → release).

**Expected behavior:**
- Test complete workflows across multiple pages
- Verify API calls are made correctly at each step
- Verify state transitions (job status changes, escrow updates)

**Technical context:**
- `frontend/src/__tests__/proofflow.critical.test.tsx` — existing critical tests
- `frontend/src/test/msw/proofflowHandlers.ts` — MSW handlers
- `frontend/src/test/proofflow-test-utils.tsx` — test utilities

**Acceptance criteria:**
- At least 3 workflow tests:
  1. Create job → verify it appears in job list
  2. Submit milestone → verify status changes
  3. Approve milestone → verify escrow release

**Relevant files:**
- `frontend/src/__tests__/`
- `frontend/src/test/msw/`

**Difficulty:** Medium

---

## Medium-Priority: Backend & API

### 5. WebSocket Event Updates

**Problem:** The backend has a WebSocket module (`backend/src/api/ws.rs`) but it may not be fully integrated with the indexer event stream.

**Expected behavior:**
- Clients subscribe to job/milestone/dispute updates
- Events from the indexer are pushed to connected clients
- Connection management handles disconnects gracefully

**Technical context:**
- `backend/src/api/ws.rs` — WebSocket handler
- `backend/src/indexer/processor.rs` — event source

**Acceptance criteria:**
- WebSocket connection establishes successfully
- Events are pushed to subscribed clients
- Disconnect is handled cleanly

**Relevant files:**
- `backend/src/api/ws.rs`
- `backend/src/api/mod.rs`
- `backend/src/indexer/processor.rs`

**Difficulty:** Medium

---

### 6. Advanced Dispute Resolution Workflows

**Problem:** The current dispute flow is simple (file → resolve). More sophisticated workflows could include evidence submission, multi-round review, or escalation.

**Expected behavior:**
- Arbitrators can request additional evidence
- Disputes can be partially resolved (split funds)
- Dispute history is queryable

**Technical context:**
- `stellar-contract/src/lib.rs` — dispute entry points
- `backend/src/api/proofflow.rs` — dispute routes

**Acceptance criteria:**
- Dispute evidence submission endpoint
- Partial resolution (uphold worker, uphold client, split)
- Dispute history query endpoint

**Relevant files:**
- `stellar-contract/src/lib.rs`
- `stellar-contract/src/types.rs`
- `backend/src/api/proofflow.rs`

**Difficulty:** Medium-Hard

---

### 7. Worker Discovery and Search

**Problem:** Clients need to find workers for their jobs. Currently there's no search or discovery mechanism.

**Expected behavior:**
- Workers are searchable by role, reputation, availability
- Clients can filter workers by reputation score
- Worker profiles show completed jobs and ratings

**Technical context:**
- Worker data is on-chain (reputation, completed jobs)
- Backend can cache and index worker profiles

**Acceptance criteria:**
- `GET /api/v1/workers` endpoint with filtering
- Reputation-based ranking
- Filter by role, minimum reputation, availability

**Relevant files:**
- `backend/src/api/proofflow.rs`
- `backend/src/services/domain.rs`
- `frontend/src/pages/` (new worker discovery page)

**Difficulty:** Medium

---

## Lower-Priority: Frontend & Polish

### 8. Notification Architecture

**Problem:** Users have no way to know when milestones are submitted, disputes are filed, or payments are released without polling.

**Expected behavior:**
- In-app notifications for key events
- Notification preferences (email, push, in-app)
- Notification history

**Technical context:**
- Backend has notification infrastructure (`backend/src/services/`)
- WebSocket module can push notifications
- Frontend needs notification UI components

**Acceptance criteria:**
- Notifications appear for key events
- Users can mark notifications as read
- Notification preferences are configurable

**Relevant files:**
- `backend/src/services/`
- `backend/src/api/ws.rs`
- `frontend/src/components/` (new notification components)

**Difficulty:** Medium

---

### 9. Analytics Dashboard

**Problem:** The admin dashboard has basic stats but no analytics. Users and admins need insights into job completion rates, average settlement times, and dispute resolution patterns.

**Expected behavior:**
- Job completion rate over time
- Average milestone approval time
- Dispute resolution statistics
- Revenue/escrow flow visualization

**Technical context:**
- Data is available from indexed events
- Frontend has chart components (`components/analytics/`)

**Acceptance criteria:**
- At least 3 charts: job completion, approval time, dispute rate
- Date range filtering
- Export to CSV

**Relevant files:**
- `frontend/src/components/analytics/`
- `backend/src/api/proofflow.rs` (new analytics endpoints)

**Difficulty:** Medium

---

### 10. Accessibility Audit

**Problem:** The frontend uses Tailwind and Radix UI but may not meet WCAG 2.1 AA standards everywhere.

**Expected behavior:**
- All interactive elements are keyboard accessible
- Color contrast meets AA standards
- Screen reader support for key workflows
- Focus management in modals and dialogs

**Technical context:**
- Radix UI provides good accessibility primitives
- Tailwind needs manual accessibility classes

**Acceptance criteria:**
- Keyboard navigation works for all pages
- Screen reader announces key state changes
- Color contrast passes automated testing

**Relevant files:**
- `frontend/src/components/ui/`
- `frontend/src/pages/`

**Difficulty:** Low-Medium

---

## Getting Started

1. Check the [Engineering Principles](ENGINEERING_PRINCIPLES.md) for architectural context
2. Review the [Domain Model](DOMAIN_MODEL.md) for entity definitions
3. Look at existing tests for patterns
4. Start with a "good first issue" if available, or pick a task that matches your expertise

## Development Setup

```bash
# Contract tests
cargo test --manifest-path stellar-contract/Cargo.toml

# Backend tests
cargo test --manifest-path backend/Cargo.toml

# Frontend
cd frontend && pnpm install && pnpm exec vitest run
```
