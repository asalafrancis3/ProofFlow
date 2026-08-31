# Upstream Baseline

## Source Repository

| Field | Value |
|-------|-------|
| Upstream | `Xoulomon/Scavenger` |
| License | MIT (Copyright 2026 Scavngr Team) |
| Description | Decentralized recycling platform on Stellar/Soroban |

## Fork Repositories

| Fork | URL | Purpose |
|------|-----|---------|
| Yebom3220/Scavenger | `https://github.com/Yebom3220/Scavenger.git` | Primary development fork |
| florence2peter/Scavenger | `https://github.com/florence2peter/Scavenger.git` | Contributor fork (Wave PRs) |

## Baseline Commit

| Field | Value |
|-------|-------|
| Commit | `835350a` |
| Date | 2026-08-30 09:53:59 +0100 |
| Message | Merge pull request #1193 from MJ-RWA/refactor/extract-indexer-magic-constants |
| Branch | `upstream/main` |

## Transformation Starting Point

| Field | Value |
|-------|-------|
| Starting commit | `bae3aaa` (1 commit ahead of upstream `835350a`) |
| Starting date | 2026-08-31 |
| Working branch | `test/1133-nft-service-unit-tests` |
| Divergence | 1 commit (NFT service test addition) |

## Codebase Size at Baseline

| Component | Files | Lines |
|-----------|-------|-------|
| `backend/src/` | 90 Rust files | 21,818 LOC |
| `frontend/src/` | 436 TS/TSX files | 58,320 LOC |
| `indexer/src/` | 52 TS files | 4,372 LOC |
| `stellar-contract/src/` | 36 Rust files | 21,808 LOC |
| **Total** | **614 source files** | **~106,318 LOC** |

## Upstream Sync Policy

**Locked during architectural transformation.**

- Do not blindly merge upstream into a heavily transformed branch.
- Periodically inspect upstream for important security or infrastructure changes.
- Selectively port only changes that remain relevant to the new project.
- After transformation is complete, review upstream for any missed improvements.

## Pre-existing Issues at Baseline

- 39 backend compilation errors (pre-existing in upstream `main`)
- Duplicated frontend directory (`Scavenger/frontend/` vs primary `frontend/`)
- Known type-safety gaps in search/filter modules
- Missing Serde derives on some enums
- Various unused imports/warnings across backend
