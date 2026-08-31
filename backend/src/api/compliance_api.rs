//! Compliance API - HTTP Layer Only
//!
//! This file handles only HTTP request/response mapping.
//! All business logic has been moved to the compliance/ module.

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

// Import domain logic from compliance module
use crate::compliance::{
    ComplianceService,
    ComplianceValidator,
    CheckRequest,
    ComplianceResult,
    ComplianceStatus,
    ComplianceError,
};

/// Compliance check endpoint - HTTP layer only
pub async fn check_compliance(req: web::Json<CheckRequest>) -> impl Responder {
    let service = ComplianceService::new();
    let validator = ComplianceValidator::new();

    // Validate the request
    if let Err(e) = validator.validate_check(&req) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "message": e.to_string()
        }));
    }

    // Delegate to service (business logic)
    match service.check_compliance(req.into_inner()).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => {
            // HTTP layer handles response mapping only
            match e {
                ComplianceError::InvalidAmount(msg) => {
                    HttpResponse::BadRequest().json(serde_json::json!({
                        "error": "Invalid request",
                        "message": msg
                    }))
                }
                ComplianceError::ValidationError(msg) => {
                    HttpResponse::BadRequest().json(serde_json::json!({
                        "error": "Validation error",
                        "message": msg
                    }))
                }
                ComplianceError::CheckFailed(msg) => {
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Compliance check failed",
                        "message": msg
                    }))
                }
            }
        }
    }
}

/// Get compliance status endpoint - HTTP layer only
pub async fn get_compliance_status(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();

    let service = ComplianceService::new();

    match service.get_status(id).await {
        Ok(status) => {
            HttpResponse::Ok().json(serde_json::json!({
                "status": format!("{:?}", status)
            }))
        }
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Not found",
            "message": e.to_string()
        })),
    }
}

/// Health check endpoint
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "compliance"
    }))
}

pub async fn list_checklists() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "checklists": [] }))
}

pub async fn create_checklist(body: web::Json<serde_json::Value>) -> impl Responder {
    HttpResponse::Created().json(serde_json::json!({ "id": uuid::Uuid::new_v4(), "data": body }))
}

pub async fn run_compliance_check(body: web::Json<serde_json::Value>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "status": "completed", "result": body }))
}

pub async fn list_compliance_alerts() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "alerts": [] }))
}

pub async fn create_alert_rule(body: web::Json<serde_json::Value>) -> impl Responder {
    HttpResponse::Created().json(serde_json::json!({ "id": uuid::Uuid::new_v4(), "rule": body }))
}

pub async fn list_alert_rules() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "rules": [] }))
}

pub async fn get_audit_trail() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "audit_trail": [] }))
}

pub async fn generate_compliance_report(body: web::Json<serde_json::Value>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "report_id": uuid::Uuid::new_v4(), "params": body }))
}

/// Route configuration
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/compliance")
            .route("/check", web::post().to(check_compliance))
            .route("/status/{id}", web::get().to(get_compliance_status))
            .route("/health", web::get().to(health_check))
    );
}
