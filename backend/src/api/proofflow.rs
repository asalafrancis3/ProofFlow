/// ProofFlow API routes.
///
/// Maps HTTP endpoints to ProofFlow contract operations.
/// Each route corresponds to either a contract read or a contract mutation.
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::services::domain::*;

// ── Health ────────────────────────────────────────────────────────────────────

pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "proofflow-api"
    }))
}

// ── User management ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct RegisterUserRequest {
    pub address: String,
    pub role: String,
    pub name: String,
}

pub async fn register_user(
    body: web::Json<RegisterUserRequest>,
) -> HttpResponse {
    // In production: validate role, check caller is admin, invoke contract
    let user = User {
        address: body.address.clone(),
        role: UserRole::from_str(&body.role).unwrap_or(UserRole::Client),
        name: body.name.clone(),
        registered_at: 0,
    };
    HttpResponse::Ok().json(ApiResponse::ok(user))
}

pub async fn get_user(path: web::Path<String>) -> HttpResponse {
    let address = path.into_inner();
    // In production: read from contract via adapter
    HttpResponse::Ok().json(ApiResponse::ok(serde_json::json!({
        "address": address,
        "role": "unknown",
        "name": "",
    })))
}

// ── Job management ────────────────────────────────────────────────────────────

pub async fn create_job(
    body: web::Json<CreateJobRequest>,
) -> HttpResponse {
    // In production: build contract TX, return signing instructions
    HttpResponse::Accepted().json(ApiResponse::ok(serde_json::json!({
        "status": "pending_signature",
        "message": "Transaction prepared, awaiting client signature",
        "job_title": body.title,
    })))
}

pub async fn get_job(path: web::Path<u64>) -> HttpResponse {
    let job_id = path.into_inner();
    // In production: read from contract via adapter
    HttpResponse::Ok().json(ApiResponse::ok(serde_json::json!({
        "id": job_id,
        "status": "unknown",
    })))
}

pub async fn list_jobs(
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let status_filter = query.get("status").map(|s| s.as_str()).unwrap_or("all");
    // In production: query Redis projections
    HttpResponse::Ok().json(ApiResponse::ok(serde_json::json!({
        "jobs": [],
        "filter": status_filter,
    })))
}

// ── Escrow ────────────────────────────────────────────────────────────────────

pub async fn fund_job(path: web::Path<u64>) -> HttpResponse {
    let job_id = path.into_inner();
    HttpResponse::Accepted().json(ApiResponse::ok(serde_json::json!({
        "status": "pending_signature",
        "message": "Fund escrow TX prepared",
        "job_id": job_id,
    })))
}

pub async fn get_escrow(path: web::Path<u64>) -> HttpResponse {
    let job_id = path.into_inner();
    HttpResponse::Ok().json(ApiResponse::ok(serde_json::json!({
        "job_id": job_id,
        "total_funded": 0,
        "total_released": 0,
        "status": "unknown",
    })))
}

// ── Milestones ────────────────────────────────────────────────────────────────

pub async fn submit_evidence(
    path: web::Path<(u64, u32)>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let (job_id, index) = path.into_inner();
    HttpResponse::Accepted().json(ApiResponse::ok(serde_json::json!({
        "status": "pending_signature",
        "message": "Evidence submission TX prepared",
        "job_id": job_id,
        "milestone_index": index,
    })))
}

pub async fn approve_milestone(
    path: web::Path<(u64, u32)>,
) -> HttpResponse {
    let (job_id, index) = path.into_inner();
    HttpResponse::Accepted().json(ApiResponse::ok(serde_json::json!({
        "status": "pending_signature",
        "message": "Milestone approval TX prepared",
        "job_id": job_id,
        "milestone_index": index,
    })))
}

pub async fn reject_milestone(
    path: web::Path<(u64, u32)>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let (job_id, index) = path.into_inner();
    HttpResponse::Accepted().json(ApiResponse::ok(serde_json::json!({
        "status": "pending_signature",
        "message": "Milestone rejection TX prepared",
        "job_id": job_id,
        "milestone_index": index,
    })))
}

// ── Disputes ──────────────────────────────────────────────────────────────────

pub async fn file_dispute(
    body: web::Json<FileDisputeRequest>,
) -> HttpResponse {
    HttpResponse::Accepted().json(ApiResponse::ok(serde_json::json!({
        "status": "pending_signature",
        "message": "Dispute filing TX prepared",
        "job_id": body.job_id,
    })))
}

pub async fn get_dispute(
    path: web::Path<(u64, u32)>,
) -> HttpResponse {
    let (job_id, dispute_id) = path.into_inner();
    HttpResponse::Ok().json(ApiResponse::ok(serde_json::json!({
        "job_id": job_id,
        "dispute_id": dispute_id,
        "status": "unknown",
    })))
}

pub async fn resolve_dispute(
    body: web::Json<ResolveDisputeRequest>,
) -> HttpResponse {
    HttpResponse::Accepted().json(ApiResponse::ok(serde_json::json!({
        "status": "pending_signature",
        "message": "Dispute resolution TX prepared",
        "job_id": body.job_id,
        "dispute_id": body.dispute_id,
    })))
}

// ── Reputation ────────────────────────────────────────────────────────────────

pub async fn get_reputation(path: web::Path<String>) -> HttpResponse {
    let address = path.into_inner();
    HttpResponse::Ok().json(ApiResponse::ok(serde_json::json!({
        "address": address,
        "completed_jobs": 0,
        "score": 0,
    })))
}

// ── Verifiers ────────────────────────────────────────────────────────────────

pub async fn list_verifiers() -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse::ok(serde_json::json!({
        "verifiers": [],
    })))
}

// ── Route registration ────────────────────────────────────────────────────────

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            // Health
            .route("/health", web::get().to(health_check))
            // Users
            .route("/users", web::post().to(register_user))
            .route("/users/{address}", web::get().to(get_user))
            // Jobs
            .route("/jobs", web::post().to(create_job))
            .route("/jobs", web::get().to(list_jobs))
            .route("/jobs/{job_id}", web::get().to(get_job))
            // Escrow
            .route("/jobs/{job_id}/fund", web::post().to(fund_job))
            .route("/jobs/{job_id}/escrow", web::get().to(get_escrow))
            // Milestones
            .route(
                "/jobs/{job_id}/milestones/{index}/evidence",
                web::post().to(submit_evidence),
            )
            .route(
                "/jobs/{job_id}/milestones/{index}/approve",
                web::post().to(approve_milestone),
            )
            .route(
                "/jobs/{job_id}/milestones/{index}/reject",
                web::post().to(reject_milestone),
            )
            // Disputes
            .route("/disputes", web::post().to(file_dispute))
            .route("/disputes/{job_id}/{dispute_id}", web::get().to(get_dispute))
            .route("/disputes/resolve", web::post().to(resolve_dispute))
            // Reputation
            .route("/reputation/{address}", web::get().to(get_reputation))
            // Verifiers
            .route("/verifiers", web::get().to(list_verifiers))
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};

    fn app() -> App<
        impl actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Config = (),
            Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
            Error = actix_web::Error,
            InitError = (),
        >,
    > {
        App::new().configure(configure)
    }

    #[actix_web::test]
    async fn health_check_returns_ok() {
        let app = test::init_service(app()).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/health")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn list_jobs_returns_empty() {
        let app = test::init_service(app()).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/jobs")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn get_user_returns_json() {
        let app = test::init_service(app()).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/users/GABC123")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["success"].as_bool().unwrap());
    }

    #[actix_web::test]
    async fn create_job_returns_accepted() {
        let app = test::init_service(app()).await;
        let body = serde_json::json!({
            "client": "GABC",
            "title": "Build website",
            "description": "A website",
            "milestone_titles": ["Design", "Code"],
            "milestone_amounts": [5000, 5000],
            "milestone_workers": ["GDEF", "GDEF"],
        });
        let req = test::TestRequest::post()
            .uri("/api/v1/jobs")
            .set_json(&body)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 202);
    }

    #[actix_web::test]
    async fn list_verifiers_returns_empty() {
        let app = test::init_service(app()).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/verifiers")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }
}
