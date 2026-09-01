use crate::services::VerificationService;
use crate::validation::{error_response, sanitize_string, validate_doc_type, validate_required, validate_url, ValidationError};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct StartVerificationRequest {
    pub participant_id: String,
}

#[derive(Deserialize)]
pub struct DocumentUploadRequest {
    pub participant_id: String,
    pub doc_type: String,
    pub url: String,
}

#[derive(Deserialize)]
pub struct ChecklistSubmitRequest {
    pub participant_id: String,
    pub checks: HashMap<String, bool>,
}

#[derive(Deserialize)]
pub struct ApprovalRequest {
    pub participant_id: String,
    pub reviewer_id: String,
}

#[derive(Deserialize)]
pub struct RejectionRequest {
    pub participant_id: String,
    pub reason: String,
    pub reviewer_id: String,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

pub async fn start_verification(
    req: web::Json<StartVerificationRequest>,
    service: web::Data<Arc<dyn VerificationService>>,
) -> HttpResponse {
    let mut errors = Vec::new();
    let participant_id = sanitize_string(&req.participant_id);

    if let Some(e) = validate_required(&participant_id, "participant_id") {
        errors.push(e);
    }

    if !errors.is_empty() {
        return error_response(&errors);
    }

    match service.start_verification(participant_id).await {
        Ok(verification) => HttpResponse::Ok().json(ApiResponse::success(verification)),
        Err(e) => HttpResponse::BadRequest().json(ApiResponse::<String>::error(e)),
    }
}

pub async fn submit_document(
    req: web::Json<DocumentUploadRequest>,
    service: web::Data<Arc<dyn VerificationService>>,
) -> HttpResponse {
    let mut errors = Vec::new();
    let participant_id = sanitize_string(&req.participant_id);
    let doc_type = sanitize_string(&req.doc_type);
    let url = sanitize_string(&req.url);

    if let Some(e) = validate_required(&participant_id, "participant_id") {
        errors.push(e);
    }
    if let Some(e) = validate_doc_type(&doc_type) {
        errors.push(e);
    }
    if let Some(e) = validate_url(&url, "url") {
        errors.push(e);
    }

    if !errors.is_empty() {
        return error_response(&errors);
    }

    match service.submit_document(participant_id, doc_type, url).await {
        Ok(document) => HttpResponse::Ok().json(ApiResponse::success(document)),
        Err(e) => HttpResponse::BadRequest().json(ApiResponse::<String>::error(e)),
    }
}

pub async fn verify_document(
    doc_id: web::Path<String>,
    service: web::Data<Arc<dyn VerificationService>>,
) -> HttpResponse {
    let doc_id = sanitize_string(doc_id.as_str());
    if doc_id.is_empty() {
        return error_response(&[ValidationError {
            field: "doc_id".to_string(),
            message: "doc_id is required".to_string(),
        }]);
    }
    match service.verify_document(doc_id).await {
        Ok(document) => HttpResponse::Ok().json(ApiResponse::success(document)),
        Err(e) => HttpResponse::NotFound().json(ApiResponse::<String>::error(e)),
    }
}

pub async fn get_verification_status(
    participant_id: web::Path<String>,
    service: web::Data<Arc<dyn VerificationService>>,
) -> HttpResponse {
    let participant_id = sanitize_string(participant_id.as_str());
    if participant_id.is_empty() {
        return error_response(&[ValidationError {
            field: "participant_id".to_string(),
            message: "participant_id is required".to_string(),
        }]);
    }
    match service.get_verification_status(participant_id).await {
        Ok(verification) => HttpResponse::Ok().json(ApiResponse::success(verification)),
        Err(e) => HttpResponse::NotFound().json(ApiResponse::<String>::error(e)),
    }
}

pub async fn submit_checklist(
    req: web::Json<ChecklistSubmitRequest>,
    service: web::Data<Arc<dyn VerificationService>>,
) -> HttpResponse {
    let mut errors = Vec::new();
    let participant_id = sanitize_string(&req.participant_id);

    if let Some(e) = validate_required(&participant_id, "participant_id") {
        errors.push(e);
    }
    if req.checks.is_empty() {
        errors.push(ValidationError {
            field: "checks".to_string(),
            message: "checks must contain at least one entry".to_string(),
        });
    }

    if !errors.is_empty() {
        return error_response(&errors);
    }

    match service.submit_checklist(participant_id, req.checks.clone()).await {
        Ok(checklist) => HttpResponse::Ok().json(ApiResponse::success(checklist)),
        Err(e) => HttpResponse::BadRequest().json(ApiResponse::<String>::error(e)),
    }
}

pub async fn get_pending_reviews(service: web::Data<Arc<dyn VerificationService>>) -> HttpResponse {
    match service.get_pending_reviews().await {
        Ok(reviews) => HttpResponse::Ok().json(ApiResponse::success(reviews)),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<String>::error(e)),
    }
}

pub async fn approve_participant(
    req: web::Json<ApprovalRequest>,
    service: web::Data<Arc<dyn VerificationService>>,
) -> HttpResponse {
    let mut errors = Vec::new();
    let participant_id = sanitize_string(&req.participant_id);
    let reviewer_id = sanitize_string(&req.reviewer_id);

    if let Some(e) = validate_required(&participant_id, "participant_id") {
        errors.push(e);
    }
    if let Some(e) = validate_required(&reviewer_id, "reviewer_id") {
        errors.push(e);
    }

    if !errors.is_empty() {
        return error_response(&errors);
    }

    match service.approve_participant(participant_id.clone(), reviewer_id).await {
        Ok(_) => {
            let _ = service.send_approval_notification(participant_id).await;
            HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({"status": "approved"})))
        }
        Err(e) => HttpResponse::BadRequest().json(ApiResponse::<String>::error(e)),
    }
}

pub async fn reject_participant(
    req: web::Json<RejectionRequest>,
    service: web::Data<Arc<dyn VerificationService>>,
) -> HttpResponse {
    let mut errors = Vec::new();
    let participant_id = sanitize_string(&req.participant_id);
    let reason = sanitize_string(&req.reason);
    let reviewer_id = sanitize_string(&req.reviewer_id);

    if let Some(e) = validate_required(&participant_id, "participant_id") {
        errors.push(e);
    }
    if let Some(e) = validate_required(&reason, "reason") {
        errors.push(e);
    }
    if let Some(e) = validate_required(&reviewer_id, "reviewer_id") {
        errors.push(e);
    }

    if !errors.is_empty() {
        return error_response(&errors);
    }

    match service
        .reject_participant(participant_id.clone(), reason.clone(), reviewer_id)
        .await
    {
        Ok(_) => {
            let _ = service.send_rejection_notification(participant_id, reason).await;
            HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({"status": "rejected"})))
        }
        Err(e) => HttpResponse::BadRequest().json(ApiResponse::<String>::error(e)),
    }
}

pub async fn retry_verification(
    participant_id: web::Path<String>,
    service: web::Data<Arc<dyn VerificationService>>,
) -> HttpResponse {
    let participant_id = sanitize_string(participant_id.as_str());
    if participant_id.is_empty() {
        return error_response(&[ValidationError {
            field: "participant_id".to_string(),
            message: "participant_id is required".to_string(),
        }]);
    }
    match service.retry_verification(participant_id).await {
        Ok(verification) => HttpResponse::Ok().json(ApiResponse::success(verification)),
        Err(e) => HttpResponse::BadRequest().json(ApiResponse::<String>::error(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::verification::{Document, ParticipantVerification, VerificationChecklist, VerificationStatus};
    use async_trait::async_trait;

    struct MockVerificationService;

    #[async_trait]
    impl VerificationService for MockVerificationService {
        async fn start_verification(&self, participant_id: String) -> Result<ParticipantVerification, String> {
            Ok(ParticipantVerification {
                participant_id: participant_id.clone(),
                status: VerificationStatus::Pending,
                documents: vec![],
                checklist: VerificationChecklist {
                    id: "cl-001".to_string(),
                    participant_id,
                    checks: HashMap::new(),
                    completed_at: None,
                },
                notes: None,
                submitted_at: chrono::Utc::now(),
                reviewed_at: None,
                reviewed_by: None,
                retry_count: 0,
                last_retry_at: None,
            })
        }

        async fn get_verification_status(&self, participant_id: String) -> Result<ParticipantVerification, String> {
            Ok(ParticipantVerification {
                participant_id: participant_id.clone(),
                status: VerificationStatus::Pending,
                documents: vec![],
                checklist: VerificationChecklist {
                    id: "cl-001".to_string(),
                    participant_id,
                    checks: HashMap::new(),
                    completed_at: None,
                },
                notes: None,
                submitted_at: chrono::Utc::now(),
                reviewed_at: None,
                reviewed_by: None,
                retry_count: 0,
                last_retry_at: None,
            })
        }

        async fn submit_document(
            &self,
            participant_id: String,
            doc_type: String,
            url: String,
        ) -> Result<Document, String> {
            Ok(Document {
                id: "doc-001".to_string(),
                participant_id,
                doc_type,
                url,
                uploaded_at: chrono::Utc::now(),
                verified: false,
                verification_notes: None,
            })
        }

        async fn verify_document(&self, _doc_id: String) -> Result<Document, String> {
            Err("not found".to_string())
        }

        async fn submit_checklist(
            &self,
            participant_id: String,
            checks: HashMap<String, bool>,
        ) -> Result<VerificationChecklist, String> {
            Ok(VerificationChecklist {
                id: "cl-001".to_string(),
                participant_id,
                checks,
                completed_at: Some(chrono::Utc::now()),
            })
        }

        async fn create_review_queue_item(&self, _participant_id: String) -> Result<String, String> {
            Ok("queue-item-001".to_string())
        }

        async fn get_pending_reviews(&self) -> Result<Vec<ParticipantVerification>, String> {
            Ok(vec![])
        }

        async fn approve_participant(&self, _participant_id: String, _reviewer_id: String) -> Result<(), String> {
            Ok(())
        }

        async fn reject_participant(
            &self,
            _participant_id: String,
            _reason: String,
            _reviewer_id: String,
        ) -> Result<(), String> {
            Ok(())
        }

        async fn retry_verification(&self, participant_id: String) -> Result<ParticipantVerification, String> {
            Ok(ParticipantVerification {
                participant_id: participant_id.clone(),
                status: VerificationStatus::Pending,
                documents: vec![],
                checklist: VerificationChecklist {
                    id: "cl-001".to_string(),
                    participant_id,
                    checks: HashMap::new(),
                    completed_at: None,
                },
                notes: None,
                submitted_at: chrono::Utc::now(),
                reviewed_at: None,
                reviewed_by: None,
                retry_count: 1,
                last_retry_at: Some(chrono::Utc::now()),
            })
        }

        async fn send_approval_notification(&self, _participant_id: String) -> Result<(), String> {
            Ok(())
        }

        async fn send_rejection_notification(&self, _participant_id: String, _reason: String) -> Result<(), String> {
            Ok(())
        }
    }

    fn mock_service() -> web::Data<Arc<dyn VerificationService>> {
        web::Data::new(Arc::new(MockVerificationService) as Arc<dyn VerificationService>)
    }

    // ── start_verification ─────────────────────────────────────────────────

    #[actix_web::test]
    async fn test_start_verification_valid() {
        let svc = mock_service();
        let req = web::Json(StartVerificationRequest {
            participant_id: "participant-001".to_string(),
        });
        let resp = start_verification(req, svc).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_start_verification_empty_id_returns_422() {
        let svc = mock_service();
        let req = web::Json(StartVerificationRequest {
            participant_id: "".to_string(),
        });
        let resp = start_verification(req, svc).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[actix_web::test]
    async fn test_start_verification_whitespace_id_returns_422() {
        let svc = mock_service();
        let req = web::Json(StartVerificationRequest {
            participant_id: "   ".to_string(),
        });
        let resp = start_verification(req, svc).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNPROCESSABLE_ENTITY);
    }

    // ── submit_document ────────────────────────────────────────────────────

    #[actix_web::test]
    async fn test_submit_document_valid() {
        let svc = mock_service();
        let req = web::Json(DocumentUploadRequest {
            participant_id: "participant-001".to_string(),
            doc_type: "passport".to_string(),
            url: "https://example.com/doc.pdf".to_string(),
        });
        let resp = submit_document(req, svc).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_submit_document_empty_participant_id_returns_422() {
        let svc = mock_service();
        let req = web::Json(DocumentUploadRequest {
            participant_id: "".to_string(),
            doc_type: "passport".to_string(),
            url: "https://example.com/doc.pdf".to_string(),
        });
        let resp = submit_document(req, svc).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[actix_web::test]
    async fn test_submit_document_empty_doc_type_returns_422() {
        let svc = mock_service();
        let req = web::Json(DocumentUploadRequest {
            participant_id: "participant-001".to_string(),
            doc_type: "".to_string(),
            url: "https://example.com/doc.pdf".to_string(),
        });
        let resp = submit_document(req, svc).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[actix_web::test]
    async fn test_submit_document_invalid_url_returns_422() {
        let svc = mock_service();
        let req = web::Json(DocumentUploadRequest {
            participant_id: "participant-001".to_string(),
            doc_type: "passport".to_string(),
            url: "not-a-url".to_string(),
        });
        let resp = submit_document(req, svc).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[actix_web::test]
    async fn test_submit_document_empty_url_returns_422() {
        let svc = mock_service();
        let req = web::Json(DocumentUploadRequest {
            participant_id: "participant-001".to_string(),
            doc_type: "passport".to_string(),
            url: "".to_string(),
        });
        let resp = submit_document(req, svc).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNPROCESSABLE_ENTITY);
    }

    // ── approve / reject ───────────────────────────────────────────────────

    #[actix_web::test]
    async fn test_approve_participant_valid() {
        let svc = mock_service();
        let req = web::Json(ApprovalRequest {
            participant_id: "participant-001".to_string(),
            reviewer_id: "reviewer-001".to_string(),
        });
        let resp = approve_participant(req, svc).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_approve_participant_missing_reviewer_id_returns_422() {
        let svc = mock_service();
        let req = web::Json(ApprovalRequest {
            participant_id: "participant-001".to_string(),
            reviewer_id: "".to_string(),
        });
        let resp = approve_participant(req, svc).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[actix_web::test]
    async fn test_reject_participant_valid() {
        let svc = mock_service();
        let req = web::Json(RejectionRequest {
            participant_id: "participant-001".to_string(),
            reason: "Insufficient documentation".to_string(),
            reviewer_id: "reviewer-001".to_string(),
        });
        let resp = reject_participant(req, svc).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_reject_participant_missing_reason_returns_422() {
        let svc = mock_service();
        let req = web::Json(RejectionRequest {
            participant_id: "participant-001".to_string(),
            reason: "".to_string(),
            reviewer_id: "reviewer-001".to_string(),
        });
        let resp = reject_participant(req, svc).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNPROCESSABLE_ENTITY);
    }

    // ── submit_checklist ───────────────────────────────────────────────────

    #[actix_web::test]
    async fn test_submit_checklist_valid() {
        let svc = mock_service();
        let mut checks = HashMap::new();
        checks.insert("identity_verified".to_string(), true);
        let req = web::Json(ChecklistSubmitRequest {
            participant_id: "participant-001".to_string(),
            checks,
        });
        let resp = submit_checklist(req, svc).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_submit_checklist_empty_checks_returns_422() {
        let svc = mock_service();
        let req = web::Json(ChecklistSubmitRequest {
            participant_id: "participant-001".to_string(),
            checks: HashMap::new(),
        });
        let resp = submit_checklist(req, svc).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNPROCESSABLE_ENTITY);
    }

    // ── URL/doc_type helper unit tests ────────────────────────────────────

    #[test]
    fn test_validate_url_http() {
        assert!(validate_url("http://example.com/doc.pdf", "url").is_none());
    }

    #[test]
    fn test_validate_url_https() {
        assert!(validate_url("https://example.com/doc.pdf", "url").is_none());
    }

    #[test]
    fn test_validate_url_empty() {
        assert!(validate_url("", "url").is_some());
    }

    #[test]
    fn test_validate_url_no_scheme() {
        assert!(validate_url("example.com/doc.pdf", "url").is_some());
    }

    #[test]
    fn test_validate_url_ftp_scheme_rejected() {
        assert!(validate_url("ftp://example.com/doc.pdf", "url").is_some());
    }

    #[test]
    fn test_validate_doc_type_valid() {
        assert!(validate_doc_type("passport").is_none());
        assert!(validate_doc_type("national_id").is_none());
    }

    #[test]
    fn test_validate_doc_type_empty() {
        assert!(validate_doc_type("").is_some());
    }

    #[test]
    fn test_validate_doc_type_too_long() {
        assert!(validate_doc_type(&"a".repeat(65)).is_some());
        assert!(validate_doc_type(&"a".repeat(64)).is_none());
    }
}
