# Phase 6 — Frontend Migration: ProofFlow

**Date:** 2026-08-31  
**Scope:** Full frontend transformation from Scavngr recycling to ProofFlow verification protocol

---

## 1. Forensic Audit Summary

| Category | Files | Action |
|----------|-------|--------|
| Design system primitives (ui/) | 28 | KEEP — Button, Card, Dialog, Input, Select, Modal, Badge, Checkbox, Switch, EmptyState, ErrorState, LoadingState, StatCard, SearchBar, ThemeToggle, AddressDisplay, TransactionConfirmDialog |
| Charts | 6 | KEEP — Recharts wrappers |
| Layout (AppShell) | 1 | REWRITE — Strip to ProofFlow routes |
| Error boundaries | 2 | KEEP |
| Pages to KEEP | 14 | NotFoundPage, ApiPlaygroundPage, FeatureFlagsPage, MessagingPage, OfflinePage, OfflineSettings, PerformanceMonitoringPage, PerformanceSLAsPage, PlatformHealthDashboardPage, QRCodePage, TestReportsPage |
| Pages to ADAPT | 8 | LoginPage, HomePage, SettingsPage, AdminDashboardPage, ParticipantRegistrationPage, ParticipantSearchPage, NotificationCenterPage, SearchResultsPage |
| Pages to REWRITE | 10 | LandingPage, ProfilePage, AnalyticsPage, GamificationPage, GovernancePage, RecyclerDashboard, RewardsPage, CommunityPage, VerificationPage, WasteVerificationDashboardPage |
| Pages to REMOVE | 23 | BatchUploadPage, CharityDonationsPage, CollectorDashboardPage, ComplianceReportsPage, EnvironmentalImpactDashboardPage, ImpactCalculatorPage, IncentivesMarketplacePage, IncentivesPage, ManufacturerDashboardPage, MaterialTransferPage, PredictiveAnalyticsPage, RecyclingGuidePage, RewardTrackingPage, RoutePlannerPage, SubscriptionsPage, SupplyChainTrackerPage, WasteCertificationPage, WasteComparisonPage, WasteHistoryPage, WasteListPage, WasteMapPage, WasteMarketplacePage, WasteStatisticsPage |
| Components to REMOVE | ~30 | analytics/, map/, qr/, ApiPlayground/, community/, admin/ tabs, WasteComparison, WasteFilterUI, WasteJourneyTimeline |
| Hooks to REMOVE | ~15 | useMapData, useDonateToCharity, useCharityDonations, useGamification, useImpactCalculator, usePredictiveAnalytics, useRoutePlanner, useScanHistory, useMessaging, useSubscriptions, useFirebase, usePerformanceMonitoring, useAnalyticsExport |
| Lib to REMOVE | ~15 | gamification, impactCalculator, governance, contributorRecognition, healthMonitoring, performanceSLAs, webVitals, bundleMonitor, analytics, analyticsService, comparisonHistory, conflictResolution, searchFilters, wasteFilterManager, pdfExporter |

---

## 2. Information Architecture

### ProofFlow Navigation Structure

```
/ (public)
├── / how-it-works
├── / features
├── / protocol
└── / login

/authenticated
├── /dashboard                    — Overview (role-based)
├── /jobs                         — Job listing
│   ├── /jobs/new                 — Create job
│   ├── /jobs/:id                 — Job detail
│   └── /jobs/:id/milestones/:idx — Milestone detail
├── /evidence                     — Evidence submission
├── /verification                 — Verification queue (verifiers)
├── /escrow                       — Escrow activity
├── /disputes                     — Dispute management
├── /reputation/:address          — Reputation profile
├── /activity                     — Transaction history
├── /settings                     — User settings
└── /admin                        — Admin dashboard
```

### ProofFlow Roles

| Role | Description | Primary Pages |
|------|-------------|---------------|
| Client | Posts jobs, funds escrow, reviews evidence, approves/rejects | Dashboard, Jobs, Escrow |
| Worker | Discovers jobs, submits evidence, receives payment | Dashboard, Jobs, Evidence |
| Verifier | Reviews evidence, approves/rejects milestones | Dashboard, Verification |
| Admin | Manages users, disputes, system config | Admin Dashboard |

---

## 3. API Integration Matrix

| UI Operation | API Endpoint | Backend Service | Contract Operation | State/Event |
|-------------|-------------|-----------------|-------------------|-------------|
| Create job | POST /api/v1/jobs | ContractAdapter.build_create_job_tx | create_job | JobCreated |
| Fund job | POST /api/v1/jobs/:id/fund | ContractAdapter.build_fund_job_tx | fund_job | JobFunded |
| Activate job | POST /api/v1/jobs/:id/activate | ContractAdapter.build_activate_job_tx | activate_job | JobActivated |
| Cancel job | POST /api/v1/jobs/:id/cancel | ContractAdapter.build_cancel_job_tx | cancel_job | JobCancelled |
| Get job | GET /api/v1/jobs/:id | IndexerStore | — | — |
| List jobs | GET /api/v1/jobs | IndexerStore | — | — |
| Submit evidence | POST /api/v1/jobs/:id/milestones/:idx/evidence | ContractAdapter.build_submit_evidence_tx | submit_evidence | MilestoneSubmitted |
| Approve milestone | POST /api/v1/jobs/:id/milestones/:idx/approve | ContractAdapter.build_approve_milestone_tx | approve_milestone | MilestoneApproved |
| Reject milestone | POST /api/v1/jobs/:id/milestones/:idx/reject | ContractAdapter.build_reject_milestone_tx | reject_milestone | MilestoneRejected |
| Release escrow | POST /api/v1/jobs/:id/release | ContractAdapter.build_release_escrow_tx | release_escrow | EscrowReleased |
| File dispute | POST /api/v1/disputes | ContractAdapter.build_file_dispute_tx | file_dispute | DisputeFiled |
| Resolve dispute | POST /api/v1/disputes/resolve | ContractAdapter.build_resolve_dispute_tx | resolve_dispute | DisputeResolved |
| Get user | GET /api/v1/users/:address | IndexerStore | — | — |
| Register user | POST /api/v1/users | ContractAdapter | register_user | UserRegistered |
| Get reputation | GET /api/v1/reputation/:address | IndexerStore | — | — |
| List verifiers | GET /api/v1/verifiers | IndexerStore | — | — |
| Health check | GET /api/v1/health | — | — | — |

---

## 4. Core Pages Implemented

### Public
- **LandingPage** — Hero, how-it-works (5 steps), live stats, CTA
- **LoginPage** — Wallet connect with role selection

### Authenticated
- **Dashboard** — Role-based overview with stat cards
- **JobsPage** — Job listing with filters
- **CreateJobPage** — Multi-step job creation
- **JobDetailPage** — Job info, milestones, escrow, actions
- **VerificationPage** — Evidence review queue
- **ReputationPage** — Public reputation profile
- **ActivityPage** — Transaction history
- **SettingsPage** — User settings

---

## 5. Future Contributor Work (NOT implemented in MVP)

| Feature | Rationale | Priority |
|---------|-----------|----------|
| Multi-verifier consensus | Complex voting logic, single verifier sufficient for MVP | Medium |
| Reputation decay | Time-based score reduction, not needed at launch | Low |
| Advanced dispute evidence | File uploads, document comparison, expert review | Medium |
| Notification system | Firebase integration deferred, backend notifications exist | High |
| Real-time WebSocket updates | WebSocket client exists but not wired to ProofFlow events | Medium |
| i18n / RTL support | Infrastructure exists but English-only MVP is fine | Low |
| Advanced analytics | Charts exist but ProofFlow-specific analytics deferred | Low |
| Gamification / badges | Generic system exists, ProofFlow gamification deferred | Low |
| Governance / voting | DAO governance deferred to post-MVP | Low |
| CSV/PDF export | Infrastructure exists, not MVP-critical | Low |
| Batch operations | Bulk actions deferred, single-item operations sufficient | Low |
| Onboarding tutorial | Joyride-based, deferred to post-MVP | Low |
| Storybook stories | Existing stories need updating for ProofFlow components | Medium |
| E2E tests | Playwright infrastructure exists, MVP E2E deferred | Medium |
| Mutation testing | Stryker config exists, deferred to post-MVP | Low |
| Lighthouse CI | Config exists, deferred to post-MVP | Low |
