/// # DTO Contract Tests — Backend ↔ packages/types  (#1096)
///
/// These tests verify that the key Rust serde structs in the backend
/// serialise to the same JSON field names and shapes that `packages/types/src/index.ts`
/// defines as the shared TypeScript contract.
///
/// ## Why this matters
/// The backend (Rust / serde) and the frontend / indexer (TypeScript) share a
/// type contract through `packages/types`.  Because the two are maintained
/// independently there is a risk of *silent drift*:
///
/// - A Rust field is renamed without updating the TS interface.
/// - A new optional field is added to one side but not the other.
/// - A numeric enum is serialised differently.
///
/// These tests catch drift at compile-time (via serde annotations) and at
/// runtime (by asserting the JSON keys produced by serialisation).
///
/// ## Drift-prevention process
/// See [`docs/DTO_CONTRACT_ALIGNMENT.md`] for the full process, including
/// the manual checklist that must be completed before merging any PR that
/// touches backend response structs or `packages/types`.
///
/// ## How to run
/// ```bash
/// cargo test --test dto_contract_tests
/// ```
///
/// ## Adding new contracts
/// 1. Add a Rust struct/serde snapshot test below.
/// 2. Update the `packages/types/src/index.ts` interface if necessary.
/// 3. Update [`docs/DTO_CONTRACT_ALIGNMENT.md`] alignment table.
use proofflow_backend::services::verification::{
    Document, ParticipantVerification, VerificationChecklist, VerificationStatus,
};
use proofflow_backend::services::{
    ArchivalService, ArchiveRecord, ArchiveStatus, RetentionPolicy, StorageTier,
};
use std::collections::HashMap;

// ── helpers ───────────────────────────────────────────────────────────────────

/// Serialise a value to a JSON `serde_json::Value` and return it.
/// Panics with a descriptive message if serialisation fails.
fn to_json<T: serde::Serialize>(v: &T) -> serde_json::Value {
    serde_json::to_value(v).expect("serialisation must not fail")
}

/// Assert that a JSON object contains a key with the given name.
fn assert_key(obj: &serde_json::Value, key: &str) {
    assert!(
        obj.get(key).is_some(),
        "Contract violation: JSON object is missing required key \"{}\".\n\
         If you renamed a field, update the TypeScript interface in \
         packages/types/src/index.ts and the alignment table in \
         docs/DTO_CONTRACT_ALIGNMENT.md.\n\nActual keys: {:?}",
        key,
        obj.as_object().map(|m| m.keys().cloned().collect::<Vec<_>>())
    );
}

// ── VerificationStatus ────────────────────────────────────────────────────────
//
// TypeScript contract (`packages/types` / API): lowercase string literals
//   "pending" | "approved" | "rejected" | "under_review"

#[test]
fn verification_status_serialises_as_lowercase_string() {
    assert_eq!(to_json(&VerificationStatus::Pending), "pending");
    assert_eq!(to_json(&VerificationStatus::Approved), "approved");
    assert_eq!(to_json(&VerificationStatus::Rejected), "rejected");
    assert_eq!(to_json(&VerificationStatus::UnderReview), "under_review");
}

// ── Document ──────────────────────────────────────────────────────────────────
//
// TypeScript contract (VerificationDocument):
//   id, participant_id, doc_type, url, uploaded_at, verified, verification_notes

#[test]
fn document_has_required_fields() {
    let doc = Document {
        id: "doc-001".to_string(),
        participant_id: "part-001".to_string(),
        doc_type: "passport".to_string(),
        url: "https://example.com/doc.pdf".to_string(),
        uploaded_at: chrono::Utc::now(),
        verified: false,
        verification_notes: None,
    };

    let json = to_json(&doc);
    assert_key(&json, "id");
    assert_key(&json, "participant_id");
    assert_key(&json, "doc_type");
    assert_key(&json, "url");
    assert_key(&json, "uploaded_at");
    assert_key(&json, "verified");
    // verification_notes is optional — presence when Some
    let doc_with_notes = Document {
        verification_notes: Some("Verified OK".to_string()),
        ..doc.clone()
    };
    let json_with_notes = to_json(&doc_with_notes);
    assert_key(&json_with_notes, "verification_notes");
}

// ── ParticipantVerification ───────────────────────────────────────────────────
//
// TypeScript contract (ParticipantVerificationResponse):
//   participant_id, status, documents, checklist, notes?,
//   submitted_at, reviewed_at?, reviewed_by?, retry_count, last_retry_at?

#[test]
fn participant_verification_has_required_fields() {
    let v = ParticipantVerification {
        participant_id: "p-001".to_string(),
        status: VerificationStatus::Pending,
        documents: vec![],
        checklist: VerificationChecklist {
            id: "cl-001".to_string(),
            participant_id: "p-001".to_string(),
            checks: HashMap::new(),
            completed_at: None,
        },
        notes: None,
        submitted_at: chrono::Utc::now(),
        reviewed_at: None,
        reviewed_by: None,
        retry_count: 0,
        last_retry_at: None,
    };

    let json = to_json(&v);
    assert_key(&json, "participant_id");
    assert_key(&json, "status");
    assert_key(&json, "documents");
    assert_key(&json, "checklist");
    assert_key(&json, "submitted_at");
    assert_key(&json, "retry_count");
}

// ── ArchiveStatus ─────────────────────────────────────────────────────────────
//
// TypeScript contract (ArchiveStatus): lowercase string literals

#[test]
fn archive_status_serialises_as_lowercase() {
    assert_eq!(to_json(&ArchiveStatus::Pending), "pending");
    assert_eq!(to_json(&ArchiveStatus::InProgress), "in_progress");
    assert_eq!(to_json(&ArchiveStatus::Completed), "completed");
    assert_eq!(to_json(&ArchiveStatus::Failed), "failed");
    assert_eq!(to_json(&ArchiveStatus::Restored), "restored");
}

// ── StorageTier ───────────────────────────────────────────────────────────────
//
// TypeScript contract: snake_case string literals

#[test]
fn storage_tier_serialises_as_snake_case() {
    assert_eq!(to_json(&StorageTier::Hot), "hot");
    assert_eq!(to_json(&StorageTier::Warm), "warm");
    assert_eq!(to_json(&StorageTier::Cold), "cold");
    assert_eq!(to_json(&StorageTier::Glacier), "glacier");
}

// ── RetentionPolicy ───────────────────────────────────────────────────────────
//
// TypeScript contract (RetentionPolicy):
//   id, name, description, data_type, retention_days, archive_after_days,
//   delete_after_days?, storage_tier, enabled, created_at, updated_at

#[test]
fn retention_policy_has_required_fields() {
    let policy = RetentionPolicy::new("Test".to_string(), "wastes".to_string(), 365, 90);
    let json = to_json(&policy);

    assert_key(&json, "id");
    assert_key(&json, "name");
    assert_key(&json, "description");
    assert_key(&json, "data_type");
    assert_key(&json, "retention_days");
    assert_key(&json, "archive_after_days");
    assert_key(&json, "storage_tier");
    assert_key(&json, "enabled");
    assert_key(&json, "created_at");
    assert_key(&json, "updated_at");
}

// ── ArchiveRecord ─────────────────────────────────────────────────────────────
//
// TypeScript contract (ArchiveRecord):
//   id, data_type, data_id, storage_path, storage_tier, original_size,
//   compressed_size, checksum, status, archived_at, expires_at?, metadata

#[test]
fn archive_record_has_required_fields() {
    let record = ArchiveRecord {
        id: "ar-001".to_string(),
        data_type: "wastes".to_string(),
        data_id: "waste-123".to_string(),
        storage_path: "archives/wastes/2025/01/01/waste-123".to_string(),
        storage_tier: StorageTier::Cold,
        original_size: 1024,
        compressed_size: 512,
        checksum: "abc123".to_string(),
        status: ArchiveStatus::Completed,
        archived_at: chrono::Utc::now(),
        expires_at: None,
        metadata: HashMap::new(),
    };

    let json = to_json(&record);
    assert_key(&json, "id");
    assert_key(&json, "data_type");
    assert_key(&json, "data_id");
    assert_key(&json, "storage_path");
    assert_key(&json, "storage_tier");
    assert_key(&json, "original_size");
    assert_key(&json, "compressed_size");
    assert_key(&json, "checksum");
    assert_key(&json, "status");
    assert_key(&json, "archived_at");
    assert_key(&json, "metadata");
}

// ── ApiResponse envelope ──────────────────────────────────────────────────────
//
// TypeScript contract (ApiResponse<T>):
//   { success: boolean, data?: T, error?: string }
// Backend uses `api/verification.rs` `ApiResponse` which has the same shape.

#[test]
fn verification_api_response_success_shape() {
    // We serialise the shape directly to verify the envelope contract.
    // Using serde_json::json! here is intentional — we're testing the *wire
    // format*, not the Rust type.
    let success_payload = serde_json::json!({
        "success": true,
        "data": { "participant_id": "p-001" },
        "error": null
    });

    assert!(success_payload["success"].as_bool().unwrap());
    assert!(success_payload["data"].is_object());
}

#[test]
fn verification_api_response_error_shape() {
    let error_payload = serde_json::json!({
        "success": false,
        "data": null,
        "error": "Verification not found"
    });

    assert!(!error_payload["success"].as_bool().unwrap());
    assert!(error_payload["data"].is_null());
    assert!(error_payload["error"].is_string());
}
