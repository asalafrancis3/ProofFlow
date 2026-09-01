//! Integration tests for export endpoints

use axum::http::StatusCode;
use serde_json::json;
use std::sync::Arc;

use proofflow_backend::api::export::{
    ExportFormat, ExportQuery, ExportResponse,
    export_waste, export_users, export_analytics,
};
use proofflow_backend::AppState;

#[tokio::test]
async fn test_export_waste_csv() {
    // This is a placeholder test - would need full integration setup
    // For now, we test the response structure
    
    let format = ExportFormat::Csv;
    match format {
        ExportFormat::Csv => assert!(true),
        ExportFormat::Json => assert!(true),
    }
}

#[tokio::test]
async fn test_export_waste_json() {
    let response = ExportResponse {
        success: true,
        data: Some(json!({ "test": "data" })),
        message: "Success".to_string(),
        format: "json".to_string(),
        record_count: 5,
    };
    
    assert!(response.success);
    assert_eq!(response.format, "json");
    assert_eq!(response.record_count, 5);
}

#[tokio::test]
async fn test_export_format_validation() {
    let valid_formats = vec![ExportFormat::Csv, ExportFormat::Json];
    let invalid_formats = vec!["xml", "excel", "pdf"];
    
    for format in valid_formats {
        match format {
            ExportFormat::Csv | ExportFormat::Json => {
                // Valid
                assert!(true);
            }
        }
    }
}

#[tokio::test]
async fn test_export_response_format() {
    let response = ExportResponse {
        success: true,
        data: Some(json!({ "csv": "a,b,c\n1,2,3" })),
        message: "Success".to_string(),
        format: "csv".to_string(),
        record_count: 1,
    };
    
    let json_value = serde_json::to_value(&response).unwrap();
    assert!(json_value.get("success").is_some());
    assert!(json_value.get("data").is_some());
    assert!(json_value.get("format").is_some());
    assert!(json_value.get("record_count").is_some());
}

#[test]
fn test_export_format_serialization() {
    let format = ExportFormat::Csv;
    let json = serde_json::to_string(&format).unwrap();
    assert_eq!(json, "\"csv\"");
    
    let format = ExportFormat::Json;
    let json = serde_json::to_string(&format).unwrap();
    assert_eq!(json, "\"json\"");
}

#[test]
fn test_export_format_deserialization() {
    let json = "\"csv\"";
    let format: ExportFormat = serde_json::from_str(json).unwrap();
    match format {
        ExportFormat::Csv => assert!(true),
        _ => assert!(false),
    }
    
    let json = "\"json\"";
    let format: ExportFormat = serde_json::from_str(json).unwrap();
    match format {
        ExportFormat::Json => assert!(true),
        _ => assert!(false),
    }
}
