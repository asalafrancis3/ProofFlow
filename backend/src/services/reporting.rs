use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("Report error: {0}")]
    ServiceError(String),
    #[error("Invalid report: {0}")]
    InvalidReport(String),
    #[error("Report not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRequest {
    pub report_type: String, // "waste", "participants", "rewards"
    pub format: String,      // "pdf", "csv"
    pub filters: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: String,
    pub report_type: String,
    pub format: String,
    pub status: String, // "pending", "completed", "failed"
    pub created_at: String,
    pub file_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplate {
    pub name: String,
    pub description: String,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledReport {
    pub report_type: String,
    pub format: String,
    pub schedule: String, // "daily", "weekly", "monthly"
    pub email_recipients: Vec<String>,
}

#[async_trait::async_trait]
pub trait ReportService: Send + Sync {
    async fn generate_report(&self, request: ReportRequest) -> Result<Report, ReportError>;
    async fn get_report(&self, report_id: &str) -> Result<Report, ReportError>;
    async fn schedule_report(&self, scheduled: ScheduledReport) -> Result<String, ReportError>;
    async fn get_templates(&self) -> Result<Vec<ReportTemplate>, ReportError>;
    async fn cache_report(&self, report_id: &str, data: Vec<u8>) -> Result<(), ReportError>;
}

pub struct ReportingService {
    storage_path: String,
}

impl ReportingService {
    pub fn new(storage_path: String) -> Self {
        Self { storage_path }
    }

    fn validate_request(&self, request: &ReportRequest) -> Result<(), ReportError> {
        if request.report_type.is_empty() {
            return Err(ReportError::InvalidReport("Empty report_type".to_string()));
        }
        if request.format.is_empty() {
            return Err(ReportError::InvalidReport("Empty format".to_string()));
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl ReportService for ReportingService {
    async fn generate_report(&self, request: ReportRequest) -> Result<Report, ReportError> {
        self.validate_request(&request)?;

        let report_id = uuid::Uuid::new_v4().to_string();
        let format = request.format.clone();

        Ok(Report {
            id: report_id,
            report_type: request.report_type,
            format: format.clone(),
            status: "completed".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            file_url: Some(format!(
                "https://storage.example.com/{}.{}",
                uuid::Uuid::new_v4(),
                format
            )),
        })
    }

    async fn get_report(&self, report_id: &str) -> Result<Report, ReportError> {
        if report_id.is_empty() {
            return Err(ReportError::NotFound("Report not found".to_string()));
        }

        Ok(Report {
            id: report_id.to_string(),
            report_type: "waste".to_string(),
            format: "pdf".to_string(),
            status: "completed".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            file_url: Some(format!("https://storage.example.com/{}.pdf", report_id)),
        })
    }

    async fn schedule_report(&self, scheduled: ScheduledReport) -> Result<String, ReportError> {
        if scheduled.report_type.is_empty() {
            return Err(ReportError::InvalidReport("Empty report_type".to_string()));
        }
        if scheduled.email_recipients.is_empty() {
            return Err(ReportError::InvalidReport("No email recipients".to_string()));
        }

        Ok(uuid::Uuid::new_v4().to_string())
    }

    async fn get_templates(&self) -> Result<Vec<ReportTemplate>, ReportError> {
        Ok(vec![
            ReportTemplate {
                name: "waste_report".to_string(),
                description: "Waste collection and processing report".to_string(),
                fields: vec!["waste_type".to_string(), "quantity".to_string(), "date".to_string()],
            },
            ReportTemplate {
                name: "participants_report".to_string(),
                description: "Participant activity report".to_string(),
                fields: vec![
                    "participant_id".to_string(),
                    "role".to_string(),
                    "activity_count".to_string(),
                ],
            },
            ReportTemplate {
                name: "rewards_report".to_string(),
                description: "Rewards distribution report".to_string(),
                fields: vec![
                    "participant_id".to_string(),
                    "rewards_earned".to_string(),
                    "date".to_string(),
                ],
            },
        ])
    }

    async fn cache_report(&self, report_id: &str, data: Vec<u8>) -> Result<(), ReportError> {
        if report_id.is_empty() {
            return Err(ReportError::InvalidReport("Empty report_id".to_string()));
        }
        if data.is_empty() {
            return Err(ReportError::InvalidReport("Empty data".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_validation() {
        let service = ReportingService::new("/tmp".to_string());
        let valid = ReportRequest {
            report_type: "waste".to_string(),
            format: "pdf".to_string(),
            filters: HashMap::new(),
        };
        assert!(service.validate_request(&valid).is_ok());

        let invalid = ReportRequest {
            report_type: "".to_string(),
            format: "pdf".to_string(),
            filters: HashMap::new(),
        };
        assert!(service.validate_request(&invalid).is_err());
    }

    #[tokio::test]
    async fn test_generate_report() {
        let service = ReportingService::new("/tmp".to_string());
        let request = ReportRequest {
            report_type: "waste".to_string(),
            format: "pdf".to_string(),
            filters: HashMap::new(),
        };
        let result = service.generate_report(request).await;
        assert!(result.is_ok());
        let report = result.unwrap();
        assert_eq!(report.report_type, "waste");
        assert_eq!(report.status, "completed");
    }

    #[tokio::test]
    async fn test_get_report() {
        let service = ReportingService::new("/tmp".to_string());
        let result = service.get_report("report-123").await;
        assert!(result.is_ok());
        let report = result.unwrap();
        assert_eq!(report.id, "report-123");
    }

    #[tokio::test]
    async fn test_schedule_report() {
        let service = ReportingService::new("/tmp".to_string());
        let scheduled = ScheduledReport {
            report_type: "waste".to_string(),
            format: "pdf".to_string(),
            schedule: "daily".to_string(),
            email_recipients: vec!["test@example.com".to_string()],
        };
        let result = service.schedule_report(scheduled).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_templates() {
        let service = ReportingService::new("/tmp".to_string());
        let result = service.get_templates().await;
        assert!(result.is_ok());
        let templates = result.unwrap();
        assert_eq!(templates.len(), 3);
    }

    #[tokio::test]
    async fn test_cache_report() {
        let service = ReportingService::new("/tmp".to_string());
        let result = service.cache_report("report-123", vec![1, 2, 3]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cache_report_empty_data() {
        let service = ReportingService::new("/tmp".to_string());
        let result = service.cache_report("report-123", vec![]).await;
        assert!(result.is_err());
    }
}
