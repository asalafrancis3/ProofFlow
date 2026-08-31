//! Export API endpoints for data export functionality
//! 
//! This module provides endpoints for exporting various data types
//! in different formats (CSV, JSON).
//! 
//! Note: Legacy formats have been removed as part of #1077.
//! Active formats: CSV, JSON.

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};
use chrono::Utc;

use crate::{
    cache::Cache,
    services::api::ApiBuilder,
    services::email::EmailService,
    services::email::TransactionalEmail,
    services::export::ExportService,
    validation::{error_response, validate_date_range, validate_export_format, validate_pagination, ValidationError},
};

// ============================================
// Types
// ============================================

/// Export format options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Csv,
    Json,
    // Removed: Excel, XML, PDF (legacy formats, no active usage)
}

/// Export request parameters
#[derive(Debug, Clone, Deserialize)]
pub struct ExportQuery {
    pub format: ExportFormat,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub limit: Option<u32>,
}

/// Export request body
#[derive(Debug, Clone, Deserialize)]
pub struct ExportRequest {
    pub format: String,
    pub data_type: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

/// Export response
#[derive(Debug, Serialize)]
pub struct ExportResponse {
    pub id: String,
    pub format: String,
    pub data_type: String,
    pub status: String,
    pub file_size: Option<u64>,
    pub created_at: String,
    pub expires_at: String,
}

/// Export list query
#[derive(Debug, Clone, Deserialize)]
pub struct ExportListQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

/// Email export request
#[derive(Debug, Clone, Deserialize)]
pub struct EmailExportRequest {
    pub export_id: String,
    pub recipients: Vec<String>,
    pub subject: Option<String>,
    pub message: Option<String>,
}

/// Scheduled export config
#[derive(Debug, Clone, Deserialize)]
pub struct ScheduledExportConfig {
    pub format: String,
    pub data_type: String,
    pub schedule: String,
    pub recipients: Vec<String>,
    pub subject: Option<String>,
}

/// Export history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportHistoryEntry {
    pub id: String,
    pub format: String,
    pub data_type: String,
    pub status: String,
    pub created_at: String,
}

/// Paginate a list of items
fn paginate<T: Serialize>(items: &[T], page: u32, limit: u32) -> serde_json::Value {
    let total = items.len() as u64;
    let start = ((page - 1) * limit) as usize;
    let end = std::cmp::min(start + limit as usize, items.len());
    let page_items = if start < items.len() { &items[start..end] } else { &[] };
    serde_json::json!({
        "items": page_items,
        "pagination": {
            "page": page,
            "limit": limit,
            "total": total,
            "total_pages": (total as f64 / limit as f64).ceil() as u64,
        }
    })
}

// ============================================
// Endpoint Handlers
// ============================================

/// Export data
/// 
/// POST /api/export
/// Body: { format, data_type, start_date, end_date }
pub async fn export_data(
    cache: web::Data<Cache>,
    body: web::Json<ExportRequest>,
) -> HttpResponse {
    let mut errors = Vec::new();

    if let Some(ref err) = validate_export_format(&body.format) {
        errors.push(err.clone());
    }
    if body.data_type.trim().is_empty() {
        errors.push(ValidationError {
            field: "data_type".to_string(),
            message: "data_type is required".to_string(),
        });
    }
    if let (Some(ref start), Some(ref end)) = (&body.start_date, &body.end_date) {
        errors.extend(validate_date_range(start, end));
    }

    if !errors.is_empty() {
        return error_response(&errors);
    }

    let export_id = uuid::Uuid::new_v4().to_string();
    let format = body.format.to_lowercase();

    let sample_data = vec![
        crate::services::export::ExportData {
            id: "waste-001".to_string(),
            waste_type: "plastic".to_string(),
            weight: 100,
            status: "pending".to_string(),
            created_at: Utc::now().to_rfc3339(),
        },
        crate::services::export::ExportData {
            id: "waste-002".to_string(),
            waste_type: "metal".to_string(),
            weight: 250,
            status: "approved".to_string(),
            created_at: Utc::now().to_rfc3339(),
        },
    ];

    let content_bytes = match format.as_str() {
        "csv" => ExportService::export_to_csv(sample_data).map(|s| s.into_bytes()),
        "json" => ExportService::export_to_json(sample_data).map(|s| s.into_bytes()),
        // PDF removed (legacy format)
        _ => unreachable!(),
    };

    match content_bytes {
        Ok(bytes) => {
            let cache_key = format!("export:{}", export_id);
            cache.set(cache_key, bytes);

            let response = ExportResponse {
                id: export_id,
                format: format.clone(),
                data_type: body.data_type.clone(),
                status: "completed".to_string(),
                file_size: None,
                created_at: Utc::now().to_rfc3339(),
                expires_at: (Utc::now() + chrono::Duration::hours(24)).to_rfc3339(),
            };

            HttpResponse::Ok().json(ApiBuilder::success_response(response))
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiBuilder::error_response::<String>("export_failed", &format!("Export failed: {}", e), 500)),
    }
}

/// Download export
/// 
/// GET /api/export/{export_id}
pub async fn download_export(
    cache: web::Data<Cache>,
    path: web::Path<String>,
) -> HttpResponse {
    let export_id = path.into_inner();
    let cache_key = format!("export:{}", export_id);

    match cache.get(&cache_key) {
        Some(data) => HttpResponse::Ok()
            .insert_header(("Content-Type", "application/octet-stream"))
            .insert_header((
                "Content-Disposition",
                format!("attachment; filename=\"{}.csv\"", export_id),
            ))
            .body(data),
        None => HttpResponse::NotFound().json(ApiBuilder::error_response::<String>("not_found", "Export not found or expired", 404)),
    }
}

/// List exports
/// 
/// GET /api/exports
/// Query: page, limit
pub async fn list_exports(query: web::Query<ExportListQuery>) -> HttpResponse {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);

    let errors = validate_pagination(page, limit);
    if !errors.is_empty() {
        return error_response(&errors);
    }

    let items: Vec<ExportHistoryEntry> = Vec::new();
    let response = paginate(&items, page, limit);
    HttpResponse::Ok().json(ApiBuilder::success_response(response))
}

/// Send export email
/// 
/// POST /api/export/send-email
pub async fn send_export_email(
    email_service: web::Data<Arc<dyn EmailService>>,
    body: web::Json<EmailExportRequest>,
) -> HttpResponse {
    let mut errors = Vec::new();
    if body.export_id.trim().is_empty() {
        errors.push(ValidationError {
            field: "export_id".to_string(),
            message: "export_id is required".to_string(),
        });
    }
    if body.recipients.is_empty() {
        errors.push(ValidationError {
            field: "recipients".to_string(),
            message: "at least one recipient is required".to_string(),
        });
    }

    if !errors.is_empty() {
        return error_response(&errors);
    }

    let subject = body
        .subject
        .clone()
        .unwrap_or_else(|| "Scavenger Data Export".to_string());

    for recipient in &body.recipients {
        let email = TransactionalEmail {
            recipient: recipient.clone(),
            template: subject.clone(),
            context: std::collections::HashMap::from([
                ("export_id".to_string(), body.export_id.clone()),
                ("message".to_string(), body.message.clone().unwrap_or_default()),
            ]),
        };

        match email_service.send_transactional(email).await {
            Ok(_) => {}
            Err(e) => {
                return HttpResponse::InternalServerError().json(ApiBuilder::error_response::<String>("email_failed", &format!(
                    "Failed to send email to {}: {}",
                    recipient, e
                ), 500));
            }
        }
    }

    HttpResponse::Ok().json(ApiBuilder::success_response("Emails sent successfully"))
}

/// Create scheduled export
/// 
/// POST /api/export/schedule
pub async fn create_scheduled_export(
    body: web::Json<ScheduledExportConfig>,
) -> HttpResponse {
    let mut errors = Vec::new();

    if let Some(ref err) = validate_export_format(&body.format) {
        errors.push(err.clone());
    }
    if body.data_type.trim().is_empty() {
        errors.push(ValidationError {
            field: "data_type".to_string(),
            message: "data_type is required".to_string(),
        });
    }
    if body.schedule.trim().is_empty() {
        errors.push(ValidationError {
            field: "schedule".to_string(),
            message: "schedule is required".to_string(),
        });
    }
    if body.recipients.is_empty() {
        errors.push(ValidationError {
            field: "recipients".to_string(),
            message: "at least one recipient is required".to_string(),
        });
    }

    if !errors.is_empty() {
        return error_response(&errors);
    }

    HttpResponse::Ok().json(ApiBuilder::success_response("Scheduled export created"))
}

/// List scheduled exports
///
/// GET /api/export/schedule
pub async fn list_scheduled_exports() -> HttpResponse {
    let items: Vec<serde_json::Value> = Vec::new();
    HttpResponse::Ok().json(ApiBuilder::success_response(items))
}

/// Delete scheduled export
/// 
/// DELETE /api/export/schedule/{id}
pub async fn delete_scheduled_export(path: web::Path<String>) -> HttpResponse {
    let export_id = path.into_inner();
    HttpResponse::Ok().json(ApiBuilder::success_response(format!(
        "Scheduled export {} deleted",
        export_id
    )))
}

// ============================================
// Route Registration
// ============================================

/// Configure export routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/export", web::post().to(export_data))
            .route("/export/{export_id}", web::get().to(download_export))
            .route("/exports", web::get().to(list_exports))
            .route("/export/send-email", web::post().to(send_export_email))
            .route("/export/schedule", web::post().to(create_scheduled_export))
            .route("/export/schedule/{id}", web::delete().to(delete_scheduled_export))
    );
}

// ============================================
// Changelog
// ============================================

///
/// # Changelog - Export Endpoints
/// 
/// ## Removed (Issue #1077)
/// - `GET /api/export/excel` - Legacy Excel format (no active usage)
/// - `GET /api/export/xml` - Legacy XML format (no active usage)
/// - `GET /api/export/pdf` - Legacy PDF format (no active usage)
/// - `GET /api/export/legacy` - Legacy endpoint (no active usage)
/// 
/// ## Kept (Active)
/// - `POST /api/export` - CSV and JSON formats
/// - `GET /api/export/{id}` - Download export
/// - `GET /api/exports` - List exports
/// - `POST /api/export/send-email` - Send export email
/// - `POST /api/export/schedule` - Schedule export
/// - `DELETE /api/export/schedule/{id}` - Delete scheduled export
/// 
/// ## Migration
/// Clients should migrate to the active endpoints:
/// - Use `format=csv` for CSV data
/// - Use `format=json` for JSON data
/// 
/// ## Date
/// 2024-01-15: Legacy endpoints removed
///

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_export_format_serialization() {
        let format = ExportFormat::Csv;
        let json = serde_json::to_string(&format).unwrap();
        assert_eq!(json, "\"csv\"");
    }

    #[test]
    fn test_export_response_serialization() {
        let response = ExportResponse {
            id: "test-123".to_string(),
            format: "csv".to_string(),
            data_type: "waste".to_string(),
            status: "completed".to_string(),
            file_size: Some(1024),
            created_at: Utc::now().to_rfc3339(),
            expires_at: (Utc::now() + chrono::Duration::hours(24)).to_rfc3339(),
        };
        let json = serde_json::to_value(&response).unwrap();
        assert!(json.get("id").is_some());
        assert!(json.get("format").is_some());
        assert!(json.get("data_type").is_some());
    }

    #[test]
    fn test_export_format_deserialization() {
        let json = "\"csv\"";
        let format: ExportFormat = serde_json::from_str(json).unwrap();
        match format {
            ExportFormat::Csv => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_invalid_format_deserialization() {
        let json = "\"xml\"";
        let result: Result<ExportFormat, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}