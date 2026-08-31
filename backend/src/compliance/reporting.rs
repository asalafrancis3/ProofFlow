use super::checklist::ComplianceChecklist;
use super::monitor::ComplianceMonitor;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRequest {
    pub checklist_id: String,
    pub period_start: chrono::DateTime<chrono::Utc>,
    pub period_end: chrono::DateTime<chrono::Utc>,
    pub include_details: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total_requirements: u64,
    pub passed: u64,
    pub failed: u64,
    pub skipped: u64,
    pub compliance_score: f64,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

pub struct ComplianceReportingService {
    monitor: ComplianceMonitor,
}

impl ComplianceReportingService {
    pub fn new(monitor: ComplianceMonitor) -> Self {
        Self { monitor }
    }

    pub fn generate_report(&mut self, checklist: &ComplianceChecklist) -> ReportSummary {
        let report = self.monitor.evaluate_checklist(checklist);
        ReportSummary {
            total_requirements: report.summary.total_checks,
            passed: report.summary.passed_checks,
            failed: report.summary.failed_checks,
            skipped: 0,
            compliance_score: report.summary.compliance_score,
            generated_at: Utc::now(),
        }
    }

    pub fn generate_detailed_report(&mut self, checklist: &ComplianceChecklist) -> (ReportSummary, Vec<super::monitor::CheckResult>) {
        let report = self.monitor.evaluate_checklist(checklist);
        (
            ReportSummary {
                total_requirements: report.summary.total_checks,
                passed: report.summary.passed_checks,
                failed: report.summary.failed_checks,
                skipped: 0,
                compliance_score: report.summary.compliance_score,
                generated_at: Utc::now(),
            },
            report.results,
        )
    }

    pub fn get_latest_report_summary(&self) -> Option<ReportSummary> {
        self.monitor.get_latest_report().map(|r| ReportSummary {
            total_requirements: r.summary.total_checks,
            passed: r.summary.passed_checks,
            failed: r.summary.failed_checks,
            skipped: 0,
            compliance_score: r.summary.compliance_score,
            generated_at: r.generated_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compliance::checklist::{ComplianceChecklist, ComplianceRequirement};
    use crate::compliance::monitor::ComplianceMonitor;

    fn make_req(id: &str, mandatory: bool, check_fn: Option<&str>) -> ComplianceRequirement {
        ComplianceRequirement {
            id: id.to_string(),
            category: "Security".to_string(),
            description: format!("Req {}", id),
            framework: "SOC2".to_string(),
            mandatory,
            check_function: check_fn.map(|s| s.to_string()),
        }
    }

    #[test]
    fn generate_report_empty_checklist() {
        let monitor = ComplianceMonitor::new();
        let mut svc = ComplianceReportingService::new(monitor);
        let cl = ComplianceChecklist::new("cl-1".to_string());

        let summary = svc.generate_report(&cl);
        assert_eq!(summary.total_requirements, 0);
        assert_eq!(summary.passed, 0);
        assert_eq!(summary.failed, 0);
        assert_eq!(summary.skipped, 0);
        assert_eq!(summary.compliance_score, 100.0);
    }

    #[test]
    fn generate_report_all_passing() {
        let monitor = ComplianceMonitor::new();
        let mut svc = ComplianceReportingService::new(monitor);
        let mut cl = ComplianceChecklist::new("cl-1".to_string());
        cl.add_requirement(make_req("r1", true, Some("data_encrypted")));
        cl.add_requirement(make_req("r2", true, Some("audit_logging_enabled")));

        let summary = svc.generate_report(&cl);
        assert_eq!(summary.total_requirements, 2);
        assert_eq!(summary.passed, 2);
        assert_eq!(summary.failed, 0);
        assert_eq!(summary.compliance_score, 100.0);
    }

    #[test]
    fn generate_report_with_skipped_checks() {
        let monitor = ComplianceMonitor::new();
        let mut svc = ComplianceReportingService::new(monitor);
        let mut cl = ComplianceChecklist::new("cl-1".to_string());
        cl.add_requirement(make_req("r1", true, Some("data_encrypted"))); // pass
        cl.add_requirement(make_req("r2", true, None)); // skipped

        let summary = svc.generate_report(&cl);
        // skipped checks excluded from total
        assert_eq!(summary.total_requirements, 1);
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.compliance_score, 100.0);
    }

    #[test]
    fn generate_report_score_independently_computed() {
        // 1 pass out of 1 counted check => 100%
        // If we had 1 pass + 1 fail, score would be 50%
        // Current impl only has Pass and Skipped, so verify 100% for single pass
        let monitor = ComplianceMonitor::new();
        let mut svc = ComplianceReportingService::new(monitor);
        let mut cl = ComplianceChecklist::new("cl-1".to_string());
        cl.add_requirement(make_req("r1", true, Some("access_control_configured")));

        let summary = svc.generate_report(&cl);
        assert_eq!(summary.compliance_score, 100.0);
        assert_eq!(summary.total_requirements, 1);
        assert_eq!(summary.passed, 1);
    }

    #[test]
    fn generate_detailed_report_returns_check_results() {
        let monitor = ComplianceMonitor::new();
        let mut svc = ComplianceReportingService::new(monitor);
        let mut cl = ComplianceChecklist::new("cl-1".to_string());
        cl.add_requirement(make_req("r1", true, Some("data_encrypted")));
        cl.add_requirement(make_req("r2", true, None));

        let (summary, results) = svc.generate_detailed_report(&cl);
        assert_eq!(summary.total_requirements, 1); // skipped excluded
        assert_eq!(results.len(), 2); // all results returned including skipped
        assert_eq!(results[0].status, crate::compliance::monitor::CheckStatus::Pass);
        assert_eq!(results[1].status, crate::compliance::monitor::CheckStatus::Skipped);
    }

    #[test]
    fn get_latest_report_summary_none_before_first_report() {
        let monitor = ComplianceMonitor::new();
        let mut svc = ComplianceReportingService::new(monitor);
        assert!(svc.get_latest_report_summary().is_none());
    }

    #[test]
    fn get_latest_report_summary_returns_most_recent() {
        let monitor = ComplianceMonitor::new();
        let mut svc = ComplianceReportingService::new(monitor);

        let cl1 = ComplianceChecklist::new("cl-1".to_string());
        let cl2 = ComplianceChecklist::new("cl-2".to_string());

        svc.generate_report(&cl1);
        svc.generate_report(&cl2);

        let latest = svc.get_latest_report_summary().unwrap();
        // The summary reflects the last evaluation
        assert_eq!(latest.compliance_score, 100.0);
    }

    #[test]
    fn report_summary_has_generated_at_timestamp() {
        let monitor = ComplianceMonitor::new();
        let mut svc = ComplianceReportingService::new(monitor);
        let cl = ComplianceChecklist::new("cl-1".to_string());

        let before = Utc::now();
        let summary = svc.generate_report(&cl);
        let after = Utc::now();

        assert!(summary.generated_at >= before);
        assert!(summary.generated_at <= after);
    }

    #[test]
    fn report_request_struct_stores_fields() {
        let req = ReportRequest {
            checklist_id: "cl-1".to_string(),
            period_start: Utc::now(),
            period_end: Utc::now(),
            include_details: true,
        };
        assert_eq!(req.checklist_id, "cl-1");
        assert!(req.include_details);
    }

    #[test]
    fn generate_report_can_be_called_multiple_times() {
        let monitor = ComplianceMonitor::new();
        let mut svc = ComplianceReportingService::new(monitor);
        let cl = ComplianceChecklist::new("cl-1".to_string());

        let s1 = svc.generate_report(&cl);
        let s2 = svc.generate_report(&cl);

        // Both should produce the same score
        assert_eq!(s1.compliance_score, s2.compliance_score);
        assert_eq!(s1.total_requirements, s2.total_requirements);
    }
}
