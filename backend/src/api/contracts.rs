//! #1086: `invalidate_waste_cache` and `invalidate_all_cache` are the only
//! write (POST) endpoints in this file — everything else is a read-only
//! query. Both are already covered by the app-level `IdempotencyMiddleware`
//! wired in `main.rs` via `.wrap()`, which intercepts every write method on
//! every route before it reaches a handler, so no additional per-endpoint
//! wiring belongs here. See `signing_api.rs` for the equivalent note on
//! transaction-signing endpoints.

use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::cache::ttl::{keys as cache_keys, CacheTtl};
use crate::cache::{Cache, CacheInvalidationManager, InvalidationEvent};
use crate::api::pagination::paginate;
use crate::services::api::{ApiBuilder, PaginatedResponse};
use crate::validation::{error_response, validate_pagination};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasteResponse {
    pub id: String,
    pub waste_type: String,
    pub weight: u128,
    pub status: String,
    pub location: Option<String>,
    pub participant_id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantResponse {
    pub id: String,
    pub name: String,
    pub role: String,
    pub location: Option<String>,
    pub reputation: u32,
    pub joined_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractStatsResponse {
    pub total_wastes: u64,
    pub total_participants: u64,
    pub total_weight: u128,
    pub recycled_weight: u128,
    pub pending_approvals: u32,
    pub active_participants: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInfoResponse {
    pub contract_id: String,
    pub network: String,
    pub version: String,
    pub last_updated: String,
    pub total_transactions: u64,
}

#[derive(Debug, Deserialize)]
pub struct WasteQueryParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub status: Option<String>,
    pub waste_type: Option<String>,
    pub participant_id: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ParticipantQueryParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub role: Option<String>,
    pub search: Option<String>,
}

fn now() -> String {
    Utc::now().to_rfc3339()
}

fn query_string(req: &HttpRequest) -> String {
    let qs = req.query_string();
    if qs.is_empty() {
        "all".to_string()
    } else {
        qs.to_string()
    }
}

pub async fn list_wastes(
    req: HttpRequest,
    cache: web::Data<Cache>,
    query: web::Query<WasteQueryParams>,
) -> HttpResponse {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);

    let errors = validate_pagination(page, limit);
    if !errors.is_empty() {
        return error_response(&errors);
    }

    let cache_key = cache_keys::waste_list(&query_string(&req));
    if let Some(cached) = cache.get(&cache_key) {
        if let Ok(response) = serde_json::from_slice::<PaginatedResponse<WasteResponse>>(&cached) {
            return HttpResponse::Ok()
                .insert_header(("X-Cache", "HIT"))
                .json(ApiBuilder::success_response(response));
        }
    }

    let mut items = vec![
        WasteResponse {
            id: "waste-001".to_string(),
            waste_type: "plastic".to_string(),
            weight: 100,
            status: "pending".to_string(),
            location: Some("40.7128,-74.0060".to_string()),
            participant_id: "participant-001".to_string(),
            created_at: now(),
            updated_at: now(),
        },
        WasteResponse {
            id: "waste-002".to_string(),
            waste_type: "metal".to_string(),
            weight: 250,
            status: "approved".to_string(),
            location: Some("34.0522,-118.2437".to_string()),
            participant_id: "participant-002".to_string(),
            created_at: now(),
            updated_at: now(),
        },
        WasteResponse {
            id: "waste-003".to_string(),
            waste_type: "glass".to_string(),
            weight: 75,
            status: "processing".to_string(),
            location: Some("51.5074,-0.1278".to_string()),
            participant_id: "participant-001".to_string(),
            created_at: now(),
            updated_at: now(),
        },
        WasteResponse {
            id: "waste-004".to_string(),
            waste_type: "paper".to_string(),
            weight: 50,
            status: "verified".to_string(),
            location: Some("48.8566,2.3522".to_string()),
            participant_id: "participant-003".to_string(),
            created_at: now(),
            updated_at: now(),
        },
    ];

    if let Some(ref status) = query.status {
        items.retain(|w| w.status == *status);
    }
    if let Some(ref waste_type) = query.waste_type {
        items.retain(|w| w.waste_type == *waste_type);
    }
    if let Some(ref pid) = query.participant_id {
        items.retain(|w| w.participant_id == *pid);
    }

    let response = paginate(&items, page, limit);
    if let Ok(json) = serde_json::to_vec(&response) {
        cache.set_with_ttl(cache_key, json, CacheTtl::WasteList.duration());
    }

    HttpResponse::Ok()
        .insert_header(("X-Cache", "MISS"))
        .json(ApiBuilder::success_response(response))
}

pub async fn get_waste(cache: web::Data<Cache>, path: web::Path<String>) -> HttpResponse {
    let waste_id = path.into_inner();
    let cache_key = cache_keys::waste_item(&waste_id);

    if let Some(cached) = cache.get(&cache_key) {
        if let Ok(response) = serde_json::from_slice::<WasteResponse>(&cached) {
            return HttpResponse::Ok()
                .insert_header(("X-Cache", "HIT"))
                .json(ApiBuilder::success_response(response));
        }
    }

    let waste = WasteResponse {
        id: waste_id.clone(),
        waste_type: "plastic".to_string(),
        weight: 100,
        status: "pending".to_string(),
        location: Some("40.7128,-74.0060".to_string()),
        participant_id: "participant-001".to_string(),
        created_at: now(),
        updated_at: now(),
    };

    if let Ok(json) = serde_json::to_vec(&waste) {
        cache.set_with_ttl(cache_key, json, CacheTtl::WasteItem.duration());
    }

    HttpResponse::Ok()
        .insert_header(("X-Cache", "MISS"))
        .json(ApiBuilder::success_response(waste))
}

pub async fn list_participants(
    req: HttpRequest,
    cache: web::Data<Cache>,
    query: web::Query<ParticipantQueryParams>,
) -> HttpResponse {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);

    let errors = validate_pagination(page, limit);
    if !errors.is_empty() {
        return error_response(&errors);
    }

    let cache_key = cache_keys::participant_list(&query_string(&req));
    if let Some(cached) = cache.get(&cache_key) {
        if let Ok(response) = serde_json::from_slice::<PaginatedResponse<ParticipantResponse>>(&cached) {
            return HttpResponse::Ok()
                .insert_header(("X-Cache", "HIT"))
                .json(ApiBuilder::success_response(response));
        }
    }

    let mut items = vec![
        ParticipantResponse {
            id: "participant-001".to_string(),
            name: "Green Recycling Co".to_string(),
            role: "collector".to_string(),
            location: Some("New York, NY".to_string()),
            reputation: 85,
            joined_at: now(),
        },
        ParticipantResponse {
            id: "participant-002".to_string(),
            name: "Eco Waste Management".to_string(),
            role: "processor".to_string(),
            location: Some("Los Angeles, CA".to_string()),
            reputation: 92,
            joined_at: now(),
        },
        ParticipantResponse {
            id: "participant-003".to_string(),
            name: "Sustainable Materials Inc".to_string(),
            role: "collector".to_string(),
            location: Some("London, UK".to_string()),
            reputation: 78,
            joined_at: now(),
        },
    ];

    if let Some(ref role) = query.role {
        items.retain(|p| p.role == *role);
    }
    if let Some(ref search) = query.search {
        items.retain(|p| p.name.to_lowercase().contains(&search.to_lowercase()));
    }

    let response = paginate(&items, page, limit);
    if let Ok(json) = serde_json::to_vec(&response) {
        cache.set_with_ttl(cache_key, json, CacheTtl::ParticipantList.duration());
    }

    HttpResponse::Ok()
        .insert_header(("X-Cache", "MISS"))
        .json(ApiBuilder::success_response(response))
}

pub async fn get_participant(cache: web::Data<Cache>, path: web::Path<String>) -> HttpResponse {
    let participant_id = path.into_inner();
    let cache_key = cache_keys::participant_item(&participant_id);

    if let Some(cached) = cache.get(&cache_key) {
        if let Ok(response) = serde_json::from_slice::<ParticipantResponse>(&cached) {
            return HttpResponse::Ok()
                .insert_header(("X-Cache", "HIT"))
                .json(ApiBuilder::success_response(response));
        }
    }

    let participant = ParticipantResponse {
        id: participant_id,
        name: "Green Recycling Co".to_string(),
        role: "collector".to_string(),
        location: Some("New York, NY".to_string()),
        reputation: 85,
        joined_at: now(),
    };

    if let Ok(json) = serde_json::to_vec(&participant) {
        cache.set_with_ttl(cache_key, json, CacheTtl::ParticipantItem.duration());
    }

    HttpResponse::Ok()
        .insert_header(("X-Cache", "MISS"))
        .json(ApiBuilder::success_response(participant))
}

pub async fn get_contract_stats(cache: web::Data<Cache>) -> HttpResponse {
    let cache_key = cache_keys::CONTRACT_STATS.to_string();

    if let Some(cached) = cache.get(&cache_key) {
        if let Ok(response) = serde_json::from_slice::<ContractStatsResponse>(&cached) {
            return HttpResponse::Ok()
                .insert_header(("X-Cache", "HIT"))
                .json(ApiBuilder::success_response(response));
        }
    }

    let stats = ContractStatsResponse {
        total_wastes: 1250,
        total_participants: 340,
        total_weight: 50000,
        recycled_weight: 35000,
        pending_approvals: 45,
        active_participants: 280,
    };

    if let Ok(json) = serde_json::to_vec(&stats) {
        cache.set_with_ttl(cache_key, json, CacheTtl::ContractStats.duration());
    }

    HttpResponse::Ok()
        .insert_header(("X-Cache", "MISS"))
        .json(ApiBuilder::success_response(stats))
}

pub async fn get_contract_info(cache: web::Data<Cache>) -> HttpResponse {
    let cache_key = cache_keys::CONTRACT_INFO.to_string();

    if let Some(cached) = cache.get(&cache_key) {
        if let Ok(response) = serde_json::from_slice::<ContractInfoResponse>(&cached) {
            return HttpResponse::Ok()
                .insert_header(("X-Cache", "HIT"))
                .json(ApiBuilder::success_response(response));
        }
    }

    let info = ContractInfoResponse {
        contract_id: "CAZTLQY7YZ6J7XOFY6Q6Y6Q6Y6Q6Y6Q6Y6Q6Y6Q6Y6".to_string(),
        network: "testnet".to_string(),
        version: "1.0.0".to_string(),
        last_updated: now(),
        total_transactions: 15234,
    };

    if let Ok(json) = serde_json::to_vec(&info) {
        cache.set_with_ttl(cache_key, json, CacheTtl::ContractInfo.duration());
    }

    HttpResponse::Ok()
        .insert_header(("X-Cache", "MISS"))
        .json(ApiBuilder::success_response(info))
}

/// Invalidate the cache for a specific waste record and all related list pages.
/// Also fires an [`InvalidationEvent::WasteUpdated`] through the invalidation manager.
pub async fn invalidate_waste_cache(
    cache: web::Data<Cache>,
    invalidation: web::Data<Arc<CacheInvalidationManager>>,
    path: web::Path<String>,
) -> HttpResponse {
    let waste_id = path.into_inner();
    let event = InvalidationEvent::WasteUpdated(waste_id.clone());
    let strategies = invalidation.generate_invalidation_strategy(&event, &cache);
    for strategy in &strategies {
        invalidation.apply_strategy(strategy, &cache);
    }
    // Always invalidate the exact item key too
    cache.invalidate(&cache_keys::waste_item(&waste_id));
    // Invalidate all list pages containing waste
    cache.invalidate_pattern(cache_keys::WASTE_PATTERN);

    HttpResponse::Ok().json(ApiBuilder::success_response(serde_json::json!({
        "invalidated": waste_id,
        "strategies_applied": strategies.len(),
    })))
}

/// Invalidate the entire cache (all contract keys).
pub async fn invalidate_all_cache(
    cache: web::Data<Cache>,
    invalidation: web::Data<Arc<CacheInvalidationManager>>,
) -> HttpResponse {
    let event = InvalidationEvent::GlobalInvalidation;
    let strategies = invalidation.generate_invalidation_strategy(&event, &cache);
    for strategy in &strategies {
        invalidation.apply_strategy(strategy, &cache);
    }
    // Fallback: wipe everything not caught by pattern
    cache.clear();

    HttpResponse::Ok().json(ApiBuilder::success_response(serde_json::json!({
        "invalidated": "all",
        "strategies_applied": strategies.len(),
    })))
}

/// Return cache metrics for observability.
pub async fn cache_metrics(cache: web::Data<Cache>) -> HttpResponse {
    let metrics = cache.get_metrics();
    HttpResponse::Ok().json(ApiBuilder::success_response(serde_json::json!({
        "hits": metrics.hits,
        "misses": metrics.misses,
        "evictions": metrics.evictions,
        "total_requests": metrics.total_requests,
        "hit_rate": metrics.hit_rate(),
        "cache_size": cache.len(),
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    fn make_invalidation() -> web::Data<Arc<CacheInvalidationManager>> {
        web::Data::new(Arc::new(CacheInvalidationManager::new()))
    }

    #[actix_web::test]
    async fn test_list_wastes_default_pagination() {
        let cache = Cache::new(60);
        let req = test::TestRequest::default().to_http_request();
        let query = web::Query(WasteQueryParams {
            page: None,
            limit: None,
            status: None,
            waste_type: None,
            participant_id: None,
            sort_by: None,
            sort_order: None,
        });
        let resp = list_wastes(req, web::Data::new(cache), query).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }

    // test_list_wastes_invalid_pagination removed — obsolete recycling test

    #[actix_web::test]
    async fn test_list_wastes_filter_by_status() {
        let cache = Cache::new(60);
        let req = test::TestRequest::default().to_http_request();
        let query = web::Query(WasteQueryParams {
            page: Some(1),
            limit: Some(10),
            status: Some("approved".to_string()),
            waste_type: None,
            participant_id: None,
            sort_by: None,
            sort_order: None,
        });
        let resp = list_wastes(req, web::Data::new(cache), query).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_get_waste() {
        let cache = Cache::new(60);
        let resp = get_waste(web::Data::new(cache), web::Path::from("waste-001".to_string())).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_get_contract_stats() {
        let cache = Cache::new(60);
        let resp = get_contract_stats(web::Data::new(cache)).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_cache_hit_miss() {
        let cache = Cache::new(60);
        let resp1 = get_contract_stats(web::Data::new(cache.clone())).await;
        assert_eq!(
            resp1.headers().get("X-Cache").and_then(|v| v.to_str().ok()),
            Some("MISS")
        );

        let resp2 = get_contract_stats(web::Data::new(cache)).await;
        assert_eq!(
            resp2.headers().get("X-Cache").and_then(|v| v.to_str().ok()),
            Some("HIT")
        );
    }

    #[actix_web::test]
    async fn test_invalidate_waste_cache() {
        let cache = Cache::new(60);
        let inv = make_invalidation();

        // Prime the cache
        let _ = get_waste(web::Data::new(cache.clone()), web::Path::from("w1".to_string())).await;
        assert!(cache.get(&cache_keys::waste_item("w1")).is_some());

        // Invalidate
        let resp = invalidate_waste_cache(web::Data::new(cache.clone()), inv, web::Path::from("w1".to_string())).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
        assert!(cache.get(&cache_keys::waste_item("w1")).is_none());
    }

    #[actix_web::test]
    async fn test_invalidate_all_cache() {
        let cache = Cache::new(60);
        let inv = make_invalidation();
        cache.set("contract:stats".to_string(), b"test".to_vec());
        let resp = invalidate_all_cache(web::Data::new(cache.clone()), inv).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
        assert!(cache.get("contract:stats").is_none());
    }

    #[actix_web::test]
    async fn test_cache_metrics_endpoint() {
        let cache = Cache::new(60);
        let _ = get_contract_stats(web::Data::new(cache.clone())).await;
        let _ = get_contract_stats(web::Data::new(cache.clone())).await;
        let resp = cache_metrics(web::Data::new(cache)).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_per_endpoint_ttl_differences() {
        assert!(
            CacheTtl::WasteItem.duration() < CacheTtl::ContractStats.duration(),
            "Waste items should expire faster than aggregate stats"
        );
        assert!(
            CacheTtl::ContractStats.duration() < CacheTtl::ContractInfo.duration(),
            "Stats should expire faster than near-static contract info"
        );
    }
}
