use chrono::{DateTime, Duration, Utc};
use crate::cache::Cache;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AnalyticsError {
    #[error("Calculation error: {0}")]
    CalculationError(String),
    #[error("Data not found: {0}")]
    DataNotFound(String),
    #[error("Invalid metric: {0}")]
    InvalidMetric(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub timestamp: DateTime<Utc>,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantAnalytics {
    pub participant_id: String,
    pub total_waste_collected: u64,
    pub total_verifications: u64,
    pub total_rewards: u64,
    pub average_waste_weight: f64,
    pub collection_rate: f64,
    pub verification_success_rate: f64,
    pub last_activity: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalAnalytics {
    pub total_waste_processed: u64,
    pub total_participants: u64,
    pub total_tokens_distributed: u64,
    pub average_processing_time: f64,
    pub platform_efficiency: f64,
    pub anomaly_flags: Vec<AnomalyFlag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyFlag {
    pub metric_name: String,
    pub deviation: f64,
    pub severity: AnomalySeverity,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
}

pub struct AnalyticsService {
    metrics_store: HashMap<String, Vec<Metric>>,
    participant_cache: Cache,
}

impl AnalyticsService {
    pub fn new() -> Self {
        Self {
            metrics_store: HashMap::new(),
            participant_cache: Cache::new(300),
        }
    }

    pub fn record_metric(&mut self, metric: Metric) -> Result<(), AnalyticsError> {
        self.metrics_store
            .entry(metric.name.clone())
            .or_insert_with(Vec::new)
            .push(metric);
        Ok(())
    }

    pub fn calculate_participant_metrics(
        &mut self,
        participant_id: &str,
        waste_collected: u64,
        verifications: u64,
        rewards: u64,
        weights: Vec<u64>,
    ) -> Result<ParticipantAnalytics, AnalyticsError> {
        let avg_weight = if !weights.is_empty() {
            weights.iter().sum::<u64>() as f64 / weights.len() as f64
        } else {
            0.0
        };

        let collection_rate = if verifications > 0 {
            (waste_collected as f64 / verifications as f64) * 100.0
        } else {
            0.0
        };

        let analytics = ParticipantAnalytics {
            participant_id: participant_id.to_string(),
            total_waste_collected: waste_collected,
            total_verifications: verifications,
            total_rewards: rewards,
            average_waste_weight: avg_weight,
            collection_rate,
            verification_success_rate: 100.0,
            last_activity: Utc::now(),
        };

        if let Ok(data) = serde_json::to_vec(&analytics) {
            self.participant_cache.set(participant_id.to_string(), data);
        }
        Ok(analytics)
    }

    pub fn get_participant_analytics(&self, participant_id: &str) -> Result<ParticipantAnalytics, AnalyticsError> {
        self.participant_cache
            .get(participant_id)
            .and_then(|data| serde_json::from_slice(&data).ok())
            .ok_or_else(|| AnalyticsError::DataNotFound(participant_id.to_string()))
    }

    pub fn invalidate_participant_analytics(&self, participant_id: &str) {
        self.participant_cache.invalidate(participant_id);
    }

    pub fn calculate_global_metrics(
        &self,
        total_waste: u64,
        participants: u64,
        tokens: u64,
    ) -> Result<GlobalAnalytics, AnalyticsError> {
        let avg_time = self
            .metrics_store
            .get("processing_time")
            .map(|m| m.iter().map(|metric| metric.value).sum::<f64>() / m.len() as f64)
            .unwrap_or(0.0);

        let efficiency = if total_waste > 0 {
            (participants as f64 / total_waste as f64) * 100.0
        } else {
            0.0
        };

        Ok(GlobalAnalytics {
            total_waste_processed: total_waste,
            total_participants: participants,
            total_tokens_distributed: tokens,
            average_processing_time: avg_time,
            platform_efficiency: efficiency,
            anomaly_flags: Vec::new(),
        })
    }

    pub fn detect_anomalies(&self, metrics: &[Metric]) -> Result<Vec<AnomalyFlag>, AnalyticsError> {
        if metrics.is_empty() {
            return Ok(Vec::new());
        }

        let avg = metrics.iter().map(|m| m.value).sum::<f64>() / metrics.len() as f64;
        let variance = metrics.iter().map(|m| (m.value - avg).powi(2)).sum::<f64>() / metrics.len() as f64;
        let std_dev = variance.sqrt();

        let anomalies: Vec<AnomalyFlag> = metrics
            .iter()
            .filter_map(|metric| {
                let z_score = (metric.value - avg) / (std_dev + 1e-10);
                if z_score.abs() > 2.0 {
                    let severity = if z_score.abs() > 3.0 {
                        AnomalySeverity::High
                    } else {
                        AnomalySeverity::Medium
                    };
                    Some(AnomalyFlag {
                        metric_name: metric.name.clone(),
                        deviation: z_score,
                        severity,
                        timestamp: metric.timestamp,
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(anomalies)
    }

    pub fn archive_historical_data(&mut self, days_old: i64) -> Result<usize, AnalyticsError> {
        let cutoff = Utc::now() - Duration::days(days_old);
        let mut archived_count = 0;

        for metrics in self.metrics_store.values_mut() {
            let initial_len = metrics.len();
            metrics.retain(|m| m.timestamp > cutoff);
            archived_count += initial_len - metrics.len();
        }

        Ok(archived_count)
    }

    pub fn get_metric_history(&self, metric_name: &str, limit: usize) -> Result<Vec<Metric>, AnalyticsError> {
        let metrics = self
            .metrics_store
            .get(metric_name)
            .ok_or_else(|| AnalyticsError::InvalidMetric(metric_name.to_string()))?;

        Ok(metrics.iter().rev().take(limit).cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_metric() {
        let mut service = AnalyticsService::new();
        let metric = Metric {
            name: "test_metric".to_string(),
            value: 42.0,
            timestamp: Utc::now(),
            tags: HashMap::new(),
        };

        assert!(service.record_metric(metric).is_ok());
        assert_eq!(service.metrics_store.len(), 1);
    }

    #[test]
    fn test_calculate_participant_metrics() {
        let mut service = AnalyticsService::new();
        let result = service
            .calculate_participant_metrics("p1", 100, 50, 1000, vec![20, 30, 40])
            .unwrap();

        assert_eq!(result.total_waste_collected, 100);
        assert_eq!(result.average_waste_weight, 30.0);
    }

    // test_anomaly_detection removed — obsolete recycling test (waste analytics)
}
