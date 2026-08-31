# Forensic Audit — Module Classification

## Executive Summary

The Scavenger codebase contains **~106K LOC** across 4 major components. The audit classifies every module as KEEP, ADAPT, REWRITE, REMOVE, FIX, or INVESTIGATE.

### Classification Distribution

| Classification | Backend | Frontend | Indexer | Contract | Total |
|---------------|---------|----------|---------|----------|-------|
| **KEEP** | ~60% | ~25% | ~60% | ~45% | ~47% |
| **ADAPT** | ~15% | ~10% | ~30% | ~30% | ~19% |
| **REWRITE** | ~10% | ~5% | 0% | ~5% | ~6% |
| **REMOVE** | ~15% | ~55% | 0% | ~20% | ~25% |
| **FIX** | ~5% | ~5% | ~10% | 0% | ~4% |

### Key Finding

The codebase splits cleanly into:
1. **Generic infrastructure** (~47%) — reusable as-is
2. **Domain-specific logic** (~25%) — to be removed
3. **Adaptable patterns** (~19%) — infrastructure to rewrite with new domain
4. **New work needed** (~6%) — contract rewrite, new pages

---

## Backend (`backend/src/`) — 90 files, 21,818 LOC

### KEEP — Generic Infrastructure (54 files)

| Module | File(s) | Why Keep |
|--------|---------|----------|
| Server framework | `main.rs`, `lib.rs` | Actix-Web server, DI, middleware stack |
| Middleware | `middleware/*.rs` (6 files) | RequestId, Validation, RateLimit, CSRF, Idempotency |
| API envelope | `services/api.rs` | `ApiResponse<T>` pattern |
| Archival | `services/archival.rs`, `services/archival_storage.rs` | Generic data archival |
| Notification | `services/notification_delivery.rs`, `services/email.rs` | Generic notification + email |
| Encryption | `services/encryption.rs`, `services/encryption_verification.rs` | AES-GCM encryption |
| Storage | `services/storage.rs` | Generic storage service |
| Search | `services/search.rs`, `search/*.rs` | Elasticsearch integration |
| Export | `services/export.rs` | CSV/PDF export |
| Reporting | `services/reporting.rs` | Generic report generation |
| Multichain | `services/multichain.rs` | Multi-chain blockchain support |
| Geospatial | `services/geospatial.rs` | Location services |
| Signing | `services/signing.rs` | Transaction signing |
| Webhook | `services/webhook.rs` | Webhook delivery |
| Contract upgrades | `services/contract_upgrades.rs` | Upgrade management |
| Crypto | `crypto/*.rs` | HMAC, AES, primitives |
| Cache | `cache/*.rs` | Redis caching |
| Config | `config/*.rs` | Environment config |
| Errors | `errors/*.rs` | Error types |
| Validation | `validation/*.rs` | Input validation |
| Redis | `redis/*.rs` | Redis client |
| RPC | `rpc/*.rs` | RPC client |
| Security | `security/*.rs` | Auth, JWT |
| Compliance infra | `compliance/mod.rs`, `compliance/validator.rs` | Compliance framework |

### ADAPT — Reusable Patterns, New Domain (14 files)

| Module | File(s) | Action |
|--------|---------|--------|
| Analytics | `services/analytics.rs` | Keep infrastructure, rewrite metrics for new domain |
| Audit | `services/audit.rs` | Keep infrastructure, adapt event types |
| NFT | `services/nft.rs` | REMOVE (recycling-specific) |
| ML classification | `services/ml_classification.rs` | REMOVE (recycling-specific) |
| Recommendations | `services/recommendations.rs` | REMOVE (recycling-specific) |
| API endpoints | `api/*.rs` (13 files) | Keep framework, rewrite recycling-specific handlers |
| Compliance logic | `compliance/reporting.rs`, `compliance/monitoring.rs` | Adapt for new compliance model |

### REMOVE — Recycling-Specific (6 files)

| Module | File(s) | Why Remove |
|--------|---------|-----------|
| NFT service | `services/nft.rs` | Waste recycling certificates |
| ML classification | `services/ml_classification.rs` | Waste type classification |
| Recommendations | `services/recommendations.rs` | Recycling recommendations |
| Compliance monitoring | `services/monitoring.rs` | Recycling compliance monitoring |
| Contract upgrades (domain) | Parts of `services/contract_upgrades.rs` | Recycling-specific upgrade logic |

### FIX — Pre-existing Errors (already fixed)

The 39 pre-existing compilation errors have been fixed. All were in generic infrastructure modules (middleware Transform API, missing module declarations, broken re-exports, type mismatches).

---

## Frontend (`frontend/src/`) — 436 files, 58,320 LOC

### KEEP — Generic Infrastructure (~110 files)

| Directory | Files | Why Keep |
|-----------|-------|----------|
| `components/ui/` | ~30 files | Base UI primitives (Button, Card, Modal, etc.) |
| `components/layout/` | ~8 files | AppShell, Sidebar, navigation |
| `components/form/` | ~5 files | Form components |
| `components/admin/` | ~8 files | Admin layout and routing |
| `components/charts/` | ~6 files | Chart components (reusable) |
| `context/` | 3 files | Auth, Theme, Wallet contexts |
| `store/` | ~6 files | Zustand stores (auth, UI, wallet) |
| `hooks/` | ~15 files | Generic hooks (useAuth, useWallet, etc.) |
| `config/` | ~4 files | Environment config |
| `styles/` | ~5 files | Global CSS |
| `lib/` | ~15 files | Generic utilities |
| `pages/LoginPage.tsx` | 1 | Auth page |
| `pages/NotFoundPage.tsx` | 1 | 404 page |
| `pages/OfflinePage.tsx` | 1 | Offline fallback |
| `pages/ProfilePage.tsx` | 1 | User profile |
| `pages/SearchResultsPage.tsx` | 1 | Search results |
| `pages/SettingsPage.tsx` | 1 | Settings |
| `pages/PerformanceMonitoringPage.tsx` | 1 | DevOps |
| `pages/PlatformHealthDashboardPage.tsx` | 1 | Health dashboard |
| `pages/FeatureFlagsPage.tsx` | 1 | Feature flags |

### ADAPT — Reusable Structure, New Content (~45 files)

| Directory | Files | Action |
|-----------|-------|--------|
| `api/` | ~5 files | Keep client architecture, rewrite domain types |
| `types/` | ~3 files | Keep structure, new type definitions |
| `i18n/` | ~10 files | Keep infrastructure, update translation content |
| `design-system/` | ~8 files | Keep tokens, remove recycling-specific components |
| `pages/NotificationsPage.tsx` | 1 | Adapt notification types |
| `pages/GovernancePage.tsx` | 1 | Adapt for protocol governance |
| `pages/RewardsPage.tsx` | 1 | Adapt for settlement history |
| `components/modals/` | ~5 files | Keep generic modals, remove recycling modals |

### REMOVE — Recycling-Specific (~240 files)

**Pages to remove (~55 files):**
All `Waste*` pages, `Recycler*`, `Collector*`, `Manufacturer*`, `Incentive*`, `Community*`, `Gamification*`, `Achievement*`, `Leaderboard*`, `Forum*`, `Events*`, `Blockchain*`, `SmartBin*`, `SmartContract*`, `StellarContract*`, `Tracking*`, `Map*`, `Wallet*`, `Faucet*`, `Donation*`, `Impact*`, `Environmental*`, `Recycling*`, `Comparison*`, `Monitoring*`, `Reduction*`, `Stream*`, `Exchange*`, `Request*`, `Offers*`, `Import*`, `Form*`, `Management*`, `Stats*`, `Collection*`, `Advanced*`, `Register*`

**Components to remove (~30 files):**
`community/`, `map/`, `qr/`, `wizard/`, `WasteLocationFields.tsx`, `WasteSubmissionForm.tsx`, `OfflineIndicator/`, `OfflineStateBanner/`

**Features to remove (~15 files):**
`features/gamification/`, `features/importWaste/`

**Stories to remove (~20 files):**
All `stories/` content

**Hooks to remove (~20 files):**
Recycling-specific hooks

**Lib to remove (~15 files):**
Recycling-specific utilities

---

## Indexer (`indexer/src/`) — 55 files, 4,372 LOC

### KEEP — Generic Infrastructure (30 files)

| Module | File(s) | Why Keep |
|--------|---------|----------|
| Entry point | `index.ts` | Startup orchestration |
| Core loop | `indexer.ts` | Polling, batch processing, reorg detection |
| Config | `config/index.ts` | Environment config |
| API server | `api/server.ts` | HTTP server, SSE, routing |
| DB client | `db/client.ts` | PostgreSQL pool |
| Migrations | `db/migrate.ts` | Migration runner |
| Stellar | `stellar/streamer.ts` | Soroban RPC event fetcher |
| Pipeline infra | `pipeline/index.ts`, `pipeline/parse.ts` (structure) | Pipeline framework |
| Dispatcher | `handlers/dispatcher.ts` | Event routing |
| Services | `services/alertService.ts`, `services/healthService.ts`, `services/replayService.ts` | Generic services |
| Controllers | All 5 controllers | Generic HTTP controllers |
| Monitoring | `monitoring/metrics.ts` | Metrics collection |
| Jobs | `jobs/index.ts`, `jobs/job-queue.ts` | Job queue |
| Sync | `sync/syncStatus.ts` | Sync status, reorg detection |
| Errors | `errors/*.rs` (2 files) | Error classes |
| Utils | `utils/*.ts` (2 files) | Logger, utilities |
| Cache | `cache/*.ts` (3 files) | Redis cache |
| Analytics | `analytics/*.ts` (2 files) | Analytics service |
| Validation | `validation/index.ts` | Input validation |
| Rate limit | `rate-limit/*.ts` (2 files) | Rate limiter |
| Queries index | `queries/index.ts` | Re-export |

### ADAPT — New Domain Required (15 files)

| Module | File(s) | Action |
|--------|---------|--------|
| Types | `types.ts`, `types/index.ts` | New domain types |
| Constants | `constants.ts` | New event types |
| Event handlers | `handlers/eventHandlers.ts` | New event handlers |
| Event service | `services/eventService.ts` | Adapt to new events |
| Participant service | `services/participantService.ts` | Adapt to new actors |
| Participant controller | `controllers/participantController.ts` | Adapt endpoints |
| Monitoring alerts | `monitoring/alerts.ts` | New alert thresholds |
| Job types | `jobs/jobTypes.ts` | New job types |
| Event queries | `queries/eventQueries.ts` | New event SQL |
| Participant queries | `queries/participantQueries.ts` | New actor SQL |
| Alert queries | `queries/alertQueries.ts` | New alert SQL |
| Optimized queries | `queries/optimizedQueries.ts` | New optimized SQL |
| Search queries | `queries/search.ts` | New search SQL |
| Pipeline types | `pipeline/types.ts` | New pipeline types |
| Pipeline transform | `pipeline/transform.ts` | New transformations |
| Pipeline store | `pipeline/store.ts` | New storage |
| Migration 001 | `db/migrations/001_initial_schema_up.sql` | New schema |

---

## Smart Contract (`stellar-contract/src/`) — 36 files, 21,808 LOC

### KEEP — Generic Infrastructure (17 files)

| Module | File(s) | Why Keep |
|--------|---------|----------|
| Re-export modules | `admin.rs`, `participant_mgmt.rs`, `waste_mgmt.rs`, `incentive_mgmt.rs`, `transfer_mgmt.rs` | Domain re-export pattern |
| Event builder | `event_builder.rs` | Generic emit1/emit2/emit3 utilities |
| Validation | `validation.rs` | Generic validators (amounts, strings, timestamps) |
| Storage utils | `storage_utils.rs` | TTL bumping utilities |
| Analytics | `analytics.rs` | Generic analytics types |
| Verification | `verification.rs` | Generic verification state machine |
| Upgrade | `upgrade.rs` | Generic upgrade proposals |
| Versioning | `versioning.rs` | API versioning |
| Explorer | `explorer.rs` | Blockchain explorer types |
| Type utils | `type_utils.rs` | Packed flags, compressed coords |
| ZKP | `zkp.rs` | SHA-256 commitment scheme |
| Key rotation | `key_rotation.rs` | Versioned key management |
| Audit log | `audit_log.rs` | Audit logging |

### ADAPT — Pattern Reusable, Domain Rewritten (10 files)

| Module | File(s) | Action |
|--------|---------|--------|
| Types | `types.rs` | Keep pattern, new domain structs |
| Errors | `errors.rs` | Keep pattern, new error variants |
| Events | `events.rs` | Keep pattern, new event topics |
| Waste modules | `waste.rs`, `waste_storage.rs` | Rewrite as job/submission modules |
| Participant modules | `participant.rs`, `participant_storage.rs` | Rewrite as actor modules |
| Incentive modules | `incentive.rs`, `incentive_storage.rs` | Rewrite as escrow modules |

### REWRITE — Core Contract (1 file)

| Module | File(s) | Action |
|--------|---------|--------|
| `lib.rs` | 1 (~8000 lines) | Complete rewrite — new entrypoints, new domain logic |

### REMOVE — Dead Code (7 files)

| Module | File(s) | Why Remove |
|--------|---------|-----------|
| Storage optimizer | `storage_optimizer.rs` | Not called from lib.rs |
| Query optimizer | `query_optimizer.rs` | Not called from lib.rs |
| Batch optimizer | `batch_optimizer.rs` | Not called from lib.rs |
| Benchmark regression | `benchmark_regression.rs` | Not called from lib.rs |
| Search | `search.rs` | Not called from lib.rs |
| Contract analytics | `contract_analytics.rs` | Debug-only, not compiled |
| Test files | `test_*.rs` (3 files) | Recycling-specific tests |

---

## Infrastructure (non-source)

### KEEP — All Infrastructure

| Component | Files | Why Keep |
|-----------|-------|----------|
| Docker | 6 Dockerfiles, 6 compose files | Generic container setup |
| CI/CD | 12 GitHub Actions workflows | Generic CI pipelines |
| K8s | `k8s/` | Kubernetes manifests |
| Terraform | `terraform/` | Infrastructure as code |
| Scripts | `scripts/` | Build/deploy scripts |
| Config | `.editorconfig`, `.gitignore`, etc. | Project config |
| Shared packages | `packages/shared/`, `packages/types/` | Shared TypeScript types |
| SDK | `packages/scavenger-sdk/` | Stellar SDK wrapper |
| Integration tests | `integration-tests/` | Test infrastructure |
| Security tests | `security-tests/` | Security test infra |
| Performance | `performance/` | Performance test infra |
| Documentation | `docs/` | Documentation framework |

### REMOVE — Duplicate

| Component | Files | Why Remove |
|-----------|-------|-----------|
| Duplicate frontend | `Scavenger/frontend/` | Upstream P0 issue — duplicated frontend |
