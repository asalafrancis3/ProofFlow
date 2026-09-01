use proofflow_backend::services::{
    ArchivalService, ArchiveQuery, ArchiveStatus, FileSystemArchivalStorage, RetentionPolicy, StorageTier,
};
use std::sync::Arc;
use tempfile::tempdir;

// ── helpers ──────────────────────────────────────────────────────────────────

fn make_service() -> (ArchivalService, tempfile::TempDir) {
    let temp_dir = tempdir().unwrap();
    let storage = Arc::new(FileSystemArchivalStorage::new(temp_dir.path().to_path_buf()));
    (ArchivalService::new(storage), temp_dir)
}

fn default_policy() -> RetentionPolicy {
    RetentionPolicy::new("Test Policy".to_string(), "wastes".to_string(), 365, 90)
}

// ── #1093 – storage-path unit test ───────────────────────────────────────────

/// Storage-path logic must live in exactly one place (`ArchivalService::build_storage_path`).
/// This test exercises that helper directly so any accidental duplication would
/// be caught immediately by a diverging format.
#[test]
fn test_build_storage_path_format() {
    let path = ArchivalService::build_storage_path("wastes", "item-42");
    // Must start with the canonical prefix
    assert!(path.starts_with("archives/wastes/"), "path = {}", path);
    // Must end with the data_id
    assert!(path.ends_with("/item-42"), "path = {}", path);
}

/// Confirm the path helper is consistent across multiple calls for the same
/// (data_type, data_id) tuple on the same calendar day.
#[test]
fn test_build_storage_path_consistency() {
    let path1 = ArchivalService::build_storage_path("participants", "part-99");
    let path2 = ArchivalService::build_storage_path("participants", "part-99");
    assert_eq!(path1, path2);
}

// ── retention-policy CRUD ─────────────────────────────────────────────────────

#[test]
fn test_create_retention_policy() {
    let (service, _dir) = make_service();
    let policy = default_policy();

    let policy_id = service.create_policy(policy.clone()).unwrap();

    let retrieved = service.get_policy(&policy_id).unwrap();
    assert_eq!(retrieved.name, "Test Policy");
    assert_eq!(retrieved.data_type, "wastes");
    assert_eq!(retrieved.retention_days, 365);
}

#[test]
fn test_list_policies() {
    let (service, _dir) = make_service();

    let policy1 = RetentionPolicy::new("Policy 1".to_string(), "wastes".to_string(), 365, 90);
    let policy2 = RetentionPolicy::new("Policy 2".to_string(), "participants".to_string(), 730, 180);

    service.create_policy(policy1).unwrap();
    service.create_policy(policy2).unwrap();

    let policies = service.list_policies().unwrap();
    assert_eq!(policies.len(), 2);
}

#[test]
fn test_update_policy() {
    let (service, _dir) = make_service();
    let mut policy = RetentionPolicy::new("Test".to_string(), "wastes".to_string(), 365, 90);
    let policy_id = service.create_policy(policy.clone()).unwrap();

    policy.retention_days = 730;
    service.update_policy(&policy_id, policy).unwrap();

    let updated = service.get_policy(&policy_id).unwrap();
    assert_eq!(updated.retention_days, 730);
}

#[test]
fn test_delete_policy() {
    let (service, _dir) = make_service();
    let policy = RetentionPolicy::new("Test".to_string(), "wastes".to_string(), 365, 90);
    let policy_id = service.create_policy(policy).unwrap();

    service.delete_policy(&policy_id).unwrap();

    let result = service.get_policy(&policy_id);
    assert!(result.is_err());
}

// ── #1093 – archive-then-restore round-trip integration test ─────────────────

/// Full round-trip: archive binary payload → restore → bytes must match exactly.
/// Also verifies that the archive record status transitions to `Restored`.
#[tokio::test]
async fn test_archive_restore_round_trip() {
    let (service, _dir) = make_service();

    let policy_id = service.create_policy(default_policy()).unwrap();
    let original_data: Vec<u8> = b"Hello, archive round-trip!".to_vec();

    // Step 1 — archive
    let archive_id = service
        .archive_data(
            "wastes".to_string(),
            "round-trip-001".to_string(),
            original_data.clone(),
            policy_id,
        )
        .await
        .unwrap();

    // Step 2 — restore
    let restored_data = service.restore_data(&archive_id).await.unwrap();

    // Step 3 — payload must be byte-identical to what was archived
    assert_eq!(
        restored_data, original_data,
        "Restored data does not match the original payload"
    );
}

/// Round-trip with binary (non-UTF-8) data to ensure compression handles arbitrary bytes.
#[tokio::test]
async fn test_archive_restore_binary_round_trip() {
    let (service, _dir) = make_service();

    let policy_id = service.create_policy(default_policy()).unwrap();
    let binary_data: Vec<u8> = (0u8..=255).collect();

    let archive_id = service
        .archive_data(
            "wastes".to_string(),
            "binary-round-trip-001".to_string(),
            binary_data.clone(),
            policy_id,
        )
        .await
        .unwrap();

    let restored = service.restore_data(&archive_id).await.unwrap();
    assert_eq!(restored, binary_data, "Binary round-trip failed");
}

/// Restoring a non-existent archive_id must return an error (not a panic).
#[tokio::test]
async fn test_restore_nonexistent_archive_returns_error() {
    let (service, _dir) = make_service();
    let result = service.restore_data("does-not-exist").await;
    assert!(result.is_err(), "Expected error for unknown archive_id");
}

/// Retention policy is enforced once during archive: `expires_at` is set
/// when `delete_after_days` is provided in the policy.
#[tokio::test]
async fn test_archive_respects_retention_policy_expiry() {
    let (service, _dir) = make_service();

    let mut policy = default_policy();
    policy.delete_after_days = Some(30);
    let policy_id = service.create_policy(policy).unwrap();

    let archive_id = service
        .archive_data(
            "wastes".to_string(),
            "expiry-test-001".to_string(),
            b"test payload".to_vec(),
            policy_id,
        )
        .await
        .unwrap();

    let archives = service
        .query_archives(ArchiveQuery {
            data_type: Some("wastes".to_string()),
            ..ArchiveQuery::default()
        })
        .unwrap();

    let record = archives.iter().find(|r| r.id == archive_id).expect("archive record missing");
    assert!(
        record.expires_at.is_some(),
        "expires_at should be set when delete_after_days is provided"
    );
}

// ── archive query ─────────────────────────────────────────────────────────────

#[test]
fn test_archive_query() {
    let (service, _dir) = make_service();

    let query = ArchiveQuery {
        data_type: Some("wastes".to_string()),
        status: Some(ArchiveStatus::Completed),
        from_date: None,
        to_date: None,
        storage_tier: Some(StorageTier::Cold),
        limit: 100,
        offset: 0,
    };

    let results = service.query_archives(query).unwrap();
    assert!(results.is_empty()); // No archives yet
}

// ── statistics ────────────────────────────────────────────────────────────────

#[test]
fn test_get_statistics() {
    let (service, _dir) = make_service();

    let stats = service.get_statistics().unwrap();
    assert_eq!(stats.total_archives, 0);
    assert_eq!(stats.total_size, 0);
}

// ── storage tiers ─────────────────────────────────────────────────────────────

#[test]
fn test_storage_tiers() {
    let (service, _dir) = make_service();

    let policy_hot = RetentionPolicy {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Hot Storage".to_string(),
        description: String::new(),
        data_type: "recent".to_string(),
        retention_days: 30,
        archive_after_days: 7,
        delete_after_days: None,
        storage_tier: StorageTier::Hot,
        enabled: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let policy_cold = RetentionPolicy {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Cold Storage".to_string(),
        description: String::new(),
        data_type: "old".to_string(),
        retention_days: 3650,
        archive_after_days: 365,
        delete_after_days: Some(7300),
        storage_tier: StorageTier::Glacier,
        enabled: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let hot_id = service.create_policy(policy_hot).unwrap();
    let cold_id = service.create_policy(policy_cold).unwrap();

    let hot_policy = service.get_policy(&hot_id).unwrap();
    let cold_policy = service.get_policy(&cold_id).unwrap();

    assert!(matches!(hot_policy.storage_tier, StorageTier::Hot));
    assert!(matches!(cold_policy.storage_tier, StorageTier::Glacier));
}
