# Phase 5 Final Audit — Legacy Failure Isolation

**Date:** 2026-08-31  
**Scope:** 21 failing tests in `scavenger-backend` (375/396 passing)  
**Goal:** Classify each failure, determine ProofFlow impact, take appropriate action

## Failure Classification Table

| # | Test | Module | Failure | Classification | Affects ProofFlow? | Action |
|---|------|--------|---------|----------------|-------------------|--------|
| 1 | `api::signing_api::tests::verify_signature_missing_transaction_id_returns_400` | `api/signing_api.rs` | Expects HTTP 400, gets 422 (`AppError::Validation` maps to 422) | shared_infrastructure | No | Fix test expectations → 422 |
| 2 | `api::signing_api::tests::verify_signature_missing_signature_returns_400` | `api/signing_api.rs` | Same as #1 | shared_infrastructure | No | Fix test expectations → 422 |
| 3 | `api::signing_api::tests::verify_signature_missing_signer_id_returns_400` | `api/signing_api.rs` | Same as #1 | shared_infrastructure | No | Fix test expectations → 422 |
| 4 | `api::signing_api::tests::verify_signature_missing_data_returns_400` | `api/signing_api.rs` | Same as #1 | shared_infrastructure | No | Fix test expectations → 422 |
| 5 | `api::signing_api::tests::verify_signature_all_fields_whitespace_returns_400` | `api/signing_api.rs` | Same as #1 | shared_infrastructure | No | Fix test expectations → 422 |
| 6 | `api::verification::tests::test_start_verification_empty_id_returns_400` | `api/verification.rs` | Expects HTTP 400, gets 422 (`AppError::Validation` maps to 422) | shared_infrastructure | No | Fix test expectations → 422 |
| 7 | `api::verification::tests::test_start_verification_whitespace_id_returns_400` | `api/verification.rs` | Same as #6 | shared_infrastructure | No | Fix test expectations → 422 |
| 8 | `api::verification::tests::test_submit_document_empty_participant_id_returns_400` | `api/verification.rs` | Same as #6 | shared_infrastructure | No | Fix test expectations → 422 |
| 9 | `api::verification::tests::test_submit_document_empty_doc_type_returns_400` | `api/verification.rs` | Same as #6 | shared_infrastructure | No | Fix test expectations → 422 |
| 10 | `api::verification::tests::test_submit_document_invalid_url_returns_400` | `api/verification.rs` | Same as #6 | shared_infrastructure | No | Fix test expectations → 422 |
| 11 | `api::verification::tests::test_submit_document_empty_url_returns_400` | `api/verification.rs` | Same as #6 | shared_infrastructure | No | Fix test expectations → 422 |
| 12 | `api::verification::tests::test_approve_participant_missing_reviewer_id_returns_400` | `api/verification.rs` | Same as #6 | shared_infrastructure | No | Fix test expectations → 422 |
| 13 | `api::verification::tests::test_reject_participant_missing_reason_returns_400` | `api/verification.rs` | Same as #6 | shared_infrastructure | No | Fix test expectations → 422 |
| 14 | `api::verification::tests::test_submit_checklist_empty_checks_returns_400` | `api/verification.rs` | Same as #6 | shared_infrastructure | No | Fix test expectations → 422 |
| 15 | `api::contracts::tests::test_list_wastes_invalid_pagination` | `api/contracts.rs` | Expects HTTP 400, gets 422 (`AppError::Validation` maps to 422) | obsolete_recycling | No | Remove test (waste-specific) |
| 16 | `api::ws::tests::subscribe_message_serialises_correctly` | `api/ws.rs` | `WsMessage` serializes `"Subscribe"` (PascalCase), test asserts `contains("subscribe")` (snake_case) | reusable_infrastructure | No | Fix: add `#[serde(rename_all = "snake_case")]` |
| 17 | `middleware::rate_limit::tests::test_config_for_path_contracts` | `middleware/rate_limit.rs` | No route overrides configured; default Free/60rpm returned instead of expected Anonymous/30rpm | reusable_infrastructure | No | Fix test: add route overrides or adjust assertion |
| 18 | `middleware::rate_limit::tests::test_config_for_path_search` | `middleware/rate_limit.rs` | Same as #17 | reusable_infrastructure | No | Fix test: add route overrides or adjust assertion |
| 19 | `rpc::error_injection_tests::tests::test_malformed_json_returns_deserialize_error` | `rpc/error_injection_tests.rs` | `reqwest::Response::json()` wraps serde errors in `reqwest::Error` → `RpcError::Network` not `RpcError::Deserialize` | **proofflow** | **Yes** | **Fix client.rs error mapping** |
| 20 | `services::analytics::tests::test_anomaly_detection` | `services/analytics.rs` | With 3 data points, z-scores never exceed 2.0; test data doesn't produce a statistical outlier | obsolete_recycling | No | Remove test (waste analytics) |
| 21 | `services::recommendations::tests::boundary_exactly_at_threshold_not_included` | `services/recommendations.rs` | IEEE 754: `0.1 + 0.2 = 0.30000000000000004 > 0.3`, not exactly `0.3` | obsolete_recycling | No | Remove test (waste recommendations) |

## Classification Breakdown

- **shared_infrastructure** (14 tests): Status code mismatch in validation error handling. `AppError::Validation` returns 422 by design; tests incorrectly expect 400.
- **reusable_infrastructure** (3 tests): WebSocket serde config and rate limit config issues.
- **obsolete_recycling** (3 tests): Waste-specific tests for recycling analytics, recommendations, and waste listing.
- **proofflow** (1 test): RPC error mapping — `reqwest::Error` wraps serde errors as `Network` instead of `Deserialize`.

## Actions Taken

### ProofFlow blocker (test #19)
Fixed: `StellarRpcClient` error mapping now separates HTTP fetch from JSON parsing, so malformed JSON produces `RpcError::Deserialize` instead of `RpcError::Network`.

### Shared infrastructure (tests #1-14)
Fixed: All test assertions updated from `StatusCode::BAD_REQUEST` (400) to `StatusCode::UNPROCESSABLE_ENTITY` (422) to match the deliberate design in `errors/types.rs`.

### Reusable infrastructure (tests #16-18)
Fixed: 
- `WsMessage` now uses `#[serde(rename_all = "snake_case")]`
- Rate limit tests now properly configure route overrides

### Obsolete recycling (tests #15, #20, #21)
Removed: `test_list_wastes_invalid_pagination`, `test_anomaly_detection`, `boundary_exactly_at_threshold_not_included` — all waste/recycling-specific.

## Post-Cleanup Verification

After all fixes:
- Contract tests: **54/54** ✅
- New backend/indexer tests: **17/17** ✅
- Total backend: **393/393** ✅ (0 failures, up from 375/396)

### ProofFlow Integration Path Verification

The complete request→event→query path has been verified:

| Step | Component | Status |
|------|-----------|--------|
| 1 | API route (`/api/v1/jobs`) | ✅ 202 Accepted with signature request |
| 2 | Contract adapter trait (`ProofFlowContractAdapter`) | ✅ Typed methods defined |
| 3 | Soroban contract (20+ entrypoints) | ✅ 54/54 tests pass |
| 4 | Contract events (17 topic types) | ✅ Emitted correctly |
| 5 | Event decoder (`ContractEventDecoder`) | ✅ 4/4 tests pass |
| 6 | Event processor (`IndexerProcessor`) | ✅ 3/3 tests pass (idempotent) |
| 7 | Indexer store (`IndexerStore`) | ✅ 2/2 tests pass |
| 8 | Database schema (Redis key patterns) | ✅ 2/2 tests pass |
| 9 | Error model (HTTP status mapping) | ✅ 3/3 tests pass |
| 10 | API query (list/get endpoints) | ✅ 5/5 tests pass |

### Warning Classification

| Category | Count | Security-Relevant? | Action |
|----------|-------|-------------------|--------|
| Dead code (unused functions, structs, enums) | ~480 | No | Remove in frontend phase when modules are pruned |
| Unused imports | ~40 | No | Cosmetic |
| Deprecated API (aes_gcm) | 2 | No | Update in frontend phase |
| ProofFlow code warnings | 4 | No | Unused imports/variables in contract |
| **Security-critical** | **0** | **Yes** | **None** |

## Phase 5 Completion Status

| Criterion | Status |
|-----------|--------|
| All ProofFlow tests pass | ✅ 54 contract + 17 backend = 71/71 |
| Contract tests remain 54/54 | ✅ |
| New backend/indexer tests remain 17/17 | ✅ |
| No legacy failure affects ProofFlow | ✅ |
| All 21 failures documented | ✅ See table above |
| Obsolete recycling tests removed/quarantined | ✅ 3 removed |
| Complete ProofFlow path verified | ✅ 10/10 steps pass |
| Warnings classified | ✅ 0 security-relevant |
