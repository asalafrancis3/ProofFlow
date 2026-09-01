//! Integration tests for backend/src/api/compliance_api.rs  (#1117)
//!
//! Each handler is tested by:
//!   1. Spinning up an `actix_web::test::TestRequest` → `actix_web::App`
//!      instance (so routing + extraction run through the real actix-web stack).
//!   2. Asserting the HTTP status code.
//!   3. Deserialising the JSON body and asserting the shape / values.
//!
//! All tests are self-contained — no database, network, or filesystem access.
//!
//! ## Running
//! ```bash
//! cargo test --test compliance_api_tests
//! ```

use actix_web::{test, web, App};
use serde_json::Value;

use proofflow_backend::api::compliance_api::{
    create_alert_rule, create_checklist, generate_compliance_report, get_audit_trail, list_alert_rules,
    list_checklists, list_compliance_alerts, run_compliance_check,
};

// ── helpers ───────────────────────────────────────────────────────────────────

/// Deserialise the body of an actix-web `ServiceResponse` into a `serde_json::Value`.
async fn body_json(resp: actix_web::dev::ServiceResponse) -> Value {
    let body = test::read_body(resp).await;
    serde_json::from_slice(&body).expect("response body must be valid JSON")
}

// ── list_checklists ───────────────────────────────────────────────────────────

#[actix_web::test]
async fn list_checklists_returns_200() {
    let app = test::init_service(App::new().route("/checklists", web::get().to(list_checklists))).await;

    let req = test::TestRequest::get().uri("/checklists").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

#[actix_web::test]
async fn list_checklists_success_flag_is_true() {
    let app = test::init_service(App::new().route("/checklists", web::get().to(list_checklists))).await;

    let req = test::TestRequest::get().uri("/checklists").to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert_eq!(json["success"], true, "success flag must be true");
}

#[actix_web::test]
async fn list_checklists_data_is_empty_array() {
    let app = test::init_service(App::new().route("/checklists", web::get().to(list_checklists))).await;

    let req = test::TestRequest::get().uri("/checklists").to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert!(
        json["data"].is_array(),
        "data field must be an array, got: {:?}",
        json["data"]
    );
    assert_eq!(
        json["data"].as_array().unwrap().len(),
        0,
        "data array must be empty initially"
    );
}

#[actix_web::test]
async fn list_checklists_message_field_present() {
    let app = test::init_service(App::new().route("/checklists", web::get().to(list_checklists))).await;

    let req = test::TestRequest::get().uri("/checklists").to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert!(json["message"].is_string(), "message field must be a string");
    assert!(!json["message"].as_str().unwrap().is_empty(), "message must not be empty");
}

/// Authorization: any caller (no auth headers required) should get 200 back,
/// because this is a read endpoint behind infrastructure-level auth (not in the
/// handler itself).
#[actix_web::test]
async fn list_checklists_no_auth_header_still_returns_200() {
    let app = test::init_service(App::new().route("/checklists", web::get().to(list_checklists))).await;

    let req = test::TestRequest::get()
        .uri("/checklists")
        // deliberately omit Authorization header
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

/// Simulate a privileged caller by sending an Authorization header — handler
/// must still return 200 (the handler is role-agnostic; auth sits in middleware).
#[actix_web::test]
async fn list_checklists_with_admin_role_header_returns_200() {
    let app = test::init_service(App::new().route("/checklists", web::get().to(list_checklists))).await;

    let req = test::TestRequest::get()
        .uri("/checklists")
        .insert_header(("Authorization", "Bearer admin-token"))
        .insert_header(("X-User-Role", "admin"))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

#[actix_web::test]
async fn list_checklists_with_recycler_role_header_returns_200() {
    let app = test::init_service(App::new().route("/checklists", web::get().to(list_checklists))).await;

    let req = test::TestRequest::get()
        .uri("/checklists")
        .insert_header(("Authorization", "Bearer recycler-token"))
        .insert_header(("X-User-Role", "recycler"))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

// ── create_checklist ──────────────────────────────────────────────────────────

#[actix_web::test]
async fn create_checklist_with_name_and_description_returns_200() {
    let app =
        test::init_service(App::new().route("/checklists", web::post().to(create_checklist))).await;

    let body = serde_json::json!({
        "name": "GDPR Checklist",
        "description": "General Data Protection Regulation requirements"
    });

    let req = test::TestRequest::post()
        .uri("/checklists")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

#[actix_web::test]
async fn create_checklist_echoes_back_name() {
    let app =
        test::init_service(App::new().route("/checklists", web::post().to(create_checklist))).await;

    let body = serde_json::json!({
        "name": "GDPR Checklist",
        "description": "General Data Protection Regulation requirements"
    });

    let req = test::TestRequest::post()
        .uri("/checklists")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert_eq!(json["data"]["name"], "GDPR Checklist", "name must be echoed back");
}

#[actix_web::test]
async fn create_checklist_echoes_back_description() {
    let app =
        test::init_service(App::new().route("/checklists", web::post().to(create_checklist))).await;

    let body = serde_json::json!({
        "name": "ISO 27001",
        "description": "Information security management requirements"
    });

    let req = test::TestRequest::post()
        .uri("/checklists")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert_eq!(
        json["data"]["description"],
        "Information security management requirements",
        "description must be echoed back"
    );
}

#[actix_web::test]
async fn create_checklist_success_flag_is_true() {
    let app =
        test::init_service(App::new().route("/checklists", web::post().to(create_checklist))).await;

    let body = serde_json::json!({"name": "SOC 2"});
    let req = test::TestRequest::post()
        .uri("/checklists")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert_eq!(json["success"], true);
}

#[actix_web::test]
async fn create_checklist_without_description_returns_200() {
    let app =
        test::init_service(App::new().route("/checklists", web::post().to(create_checklist))).await;

    // description is Option<String> — sending without it must succeed
    let body = serde_json::json!({"name": "Minimal Checklist"});
    let req = test::TestRequest::post()
        .uri("/checklists")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

#[actix_web::test]
async fn create_checklist_without_description_echoes_name() {
    let app =
        test::init_service(App::new().route("/checklists", web::post().to(create_checklist))).await;

    let body = serde_json::json!({"name": "Minimal Checklist"});
    let req = test::TestRequest::post()
        .uri("/checklists")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert_eq!(json["data"]["name"], "Minimal Checklist");
}

/// Malformed body — missing required `name` field must yield 400.
#[actix_web::test]
async fn create_checklist_missing_name_returns_400() {
    let app =
        test::init_service(App::new().route("/checklists", web::post().to(create_checklist))).await;

    let body = serde_json::json!({"description": "No name provided"});
    let req = test::TestRequest::post()
        .uri("/checklists")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
}

/// Empty JSON body must yield 400 (cannot extract `ChecklistRequest`).
#[actix_web::test]
async fn create_checklist_empty_body_returns_400() {
    let app =
        test::init_service(App::new().route("/checklists", web::post().to(create_checklist))).await;

    let req = test::TestRequest::post()
        .uri("/checklists")
        .insert_header(("Content-Type", "application/json"))
        .set_payload("{}")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
}

/// Non-JSON content type must yield 400.
#[actix_web::test]
async fn create_checklist_non_json_content_type_returns_400() {
    let app =
        test::init_service(App::new().route("/checklists", web::post().to(create_checklist))).await;

    let req = test::TestRequest::post()
        .uri("/checklists")
        .insert_header(("Content-Type", "text/plain"))
        .set_payload("name=test")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn create_checklist_message_field_present() {
    let app =
        test::init_service(App::new().route("/checklists", web::post().to(create_checklist))).await;

    let body = serde_json::json!({"name": "Test"});
    let req = test::TestRequest::post()
        .uri("/checklists")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert!(json["message"].is_string(), "message field must be a string");
}

// ── run_compliance_check ──────────────────────────────────────────────────────

#[actix_web::test]
async fn run_compliance_check_returns_200() {
    let app = test::init_service(
        App::new().route("/compliance/check", web::post().to(run_compliance_check)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/check")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

#[actix_web::test]
async fn run_compliance_check_success_flag_is_true() {
    let app = test::init_service(
        App::new().route("/compliance/check", web::post().to(run_compliance_check)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/check")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert_eq!(json["success"], true);
}

#[actix_web::test]
async fn run_compliance_check_score_is_100() {
    let app = test::init_service(
        App::new().route("/compliance/check", web::post().to(run_compliance_check)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/check")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    let score = json["data"]["compliance_score"]
        .as_f64()
        .expect("compliance_score must be a number");
    assert!(
        (score - 100.0).abs() < f64::EPSILON,
        "compliance_score must be 100.0, got {score}"
    );
}

#[actix_web::test]
async fn run_compliance_check_data_has_required_fields() {
    let app = test::init_service(
        App::new().route("/compliance/check", web::post().to(run_compliance_check)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/check")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    let data = &json["data"];
    assert!(data["compliance_score"].is_number(), "compliance_score must be present");
    assert!(data["total_checks"].is_number(), "total_checks must be present");
    assert!(data["passed"].is_number(), "passed must be present");
    assert!(data["failed"].is_number(), "failed must be present");
}

#[actix_web::test]
async fn run_compliance_check_total_checks_is_zero() {
    let app = test::init_service(
        App::new().route("/compliance/check", web::post().to(run_compliance_check)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/check")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert_eq!(json["data"]["total_checks"], 0);
    assert_eq!(json["data"]["passed"], 0);
    assert_eq!(json["data"]["failed"], 0);
}

/// Authorization boundaries — different role headers must not change the result.
#[actix_web::test]
async fn run_compliance_check_with_admin_header_returns_200() {
    let app = test::init_service(
        App::new().route("/compliance/check", web::post().to(run_compliance_check)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/check")
        .insert_header(("Authorization", "Bearer admin-token"))
        .insert_header(("X-User-Role", "admin"))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

#[actix_web::test]
async fn run_compliance_check_with_manufacturer_header_returns_200() {
    let app = test::init_service(
        App::new().route("/compliance/check", web::post().to(run_compliance_check)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/check")
        .insert_header(("Authorization", "Bearer mfr-token"))
        .insert_header(("X-User-Role", "manufacturer"))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

// ── list_compliance_alerts ────────────────────────────────────────────────────

#[actix_web::test]
async fn list_compliance_alerts_returns_200() {
    let app = test::init_service(
        App::new().route("/compliance/alerts", web::get().to(list_compliance_alerts)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/compliance/alerts")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

#[actix_web::test]
async fn list_compliance_alerts_success_flag_is_true() {
    let app = test::init_service(
        App::new().route("/compliance/alerts", web::get().to(list_compliance_alerts)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/compliance/alerts")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert_eq!(json["success"], true);
}

#[actix_web::test]
async fn list_compliance_alerts_data_is_empty_array() {
    let app = test::init_service(
        App::new().route("/compliance/alerts", web::get().to(list_compliance_alerts)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/compliance/alerts")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert!(json["data"].is_array(), "data must be an array");
    assert_eq!(
        json["data"].as_array().unwrap().len(),
        0,
        "data must be empty initially"
    );
}

#[actix_web::test]
async fn list_compliance_alerts_message_field_present() {
    let app = test::init_service(
        App::new().route("/compliance/alerts", web::get().to(list_compliance_alerts)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/compliance/alerts")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert!(json["message"].is_string());
}

// ── create_alert_rule ─────────────────────────────────────────────────────────

fn full_alert_rule_body() -> serde_json::Value {
    serde_json::json!({
        "name": "High Error Rate",
        "description": "Triggers when error rate exceeds threshold",
        "severity": "critical",
        "metric": "error_rate",
        "operator": ">",
        "threshold": 0.05,
        "window_seconds": 300
    })
}

#[actix_web::test]
async fn create_alert_rule_returns_200() {
    let app = test::init_service(
        App::new().route("/compliance/alert-rules", web::post().to(create_alert_rule)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/alert-rules")
        .set_json(&full_alert_rule_body())
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

#[actix_web::test]
async fn create_alert_rule_success_flag_is_true() {
    let app = test::init_service(
        App::new().route("/compliance/alert-rules", web::post().to(create_alert_rule)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/alert-rules")
        .set_json(&full_alert_rule_body())
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert_eq!(json["success"], true);
}

#[actix_web::test]
async fn create_alert_rule_echoes_back_name() {
    let app = test::init_service(
        App::new().route("/compliance/alert-rules", web::post().to(create_alert_rule)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/alert-rules")
        .set_json(&full_alert_rule_body())
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert_eq!(json["data"]["name"], "High Error Rate");
}

#[actix_web::test]
async fn create_alert_rule_echoes_back_description() {
    let app = test::init_service(
        App::new().route("/compliance/alert-rules", web::post().to(create_alert_rule)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/alert-rules")
        .set_json(&full_alert_rule_body())
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert_eq!(
        json["data"]["description"],
        "Triggers when error rate exceeds threshold"
    );
}

#[actix_web::test]
async fn create_alert_rule_echoes_back_severity() {
    let app = test::init_service(
        App::new().route("/compliance/alert-rules", web::post().to(create_alert_rule)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/alert-rules")
        .set_json(&full_alert_rule_body())
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert_eq!(json["data"]["severity"], "critical");
}

#[actix_web::test]
async fn create_alert_rule_echoes_back_metric() {
    let app = test::init_service(
        App::new().route("/compliance/alert-rules", web::post().to(create_alert_rule)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/alert-rules")
        .set_json(&full_alert_rule_body())
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert_eq!(json["data"]["metric"], "error_rate");
}

#[actix_web::test]
async fn create_alert_rule_echoes_back_operator() {
    let app = test::init_service(
        App::new().route("/compliance/alert-rules", web::post().to(create_alert_rule)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/alert-rules")
        .set_json(&full_alert_rule_body())
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert_eq!(json["data"]["operator"], ">");
}

#[actix_web::test]
async fn create_alert_rule_echoes_back_threshold() {
    let app = test::init_service(
        App::new().route("/compliance/alert-rules", web::post().to(create_alert_rule)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/alert-rules")
        .set_json(&full_alert_rule_body())
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    let threshold = json["data"]["threshold"]
        .as_f64()
        .expect("threshold must be a number");
    assert!(
        (threshold - 0.05).abs() < 1e-9,
        "threshold must be echoed back as 0.05, got {threshold}"
    );
}

#[actix_web::test]
async fn create_alert_rule_echoes_back_window_seconds() {
    let app = test::init_service(
        App::new().route("/compliance/alert-rules", web::post().to(create_alert_rule)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/alert-rules")
        .set_json(&full_alert_rule_body())
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert_eq!(json["data"]["window_seconds"], 300_i64);
}

/// All fields must be echoed back in a single round-trip.
#[actix_web::test]
async fn create_alert_rule_echoes_back_all_fields() {
    let app = test::init_service(
        App::new().route("/compliance/alert-rules", web::post().to(create_alert_rule)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/alert-rules")
        .set_json(&full_alert_rule_body())
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    let data = &json["data"];
    assert!(data["name"].is_string(), "name must be present");
    assert!(data["description"].is_string(), "description must be present");
    assert!(data["severity"].is_string(), "severity must be present");
    assert!(data["metric"].is_string(), "metric must be present");
    assert!(data["operator"].is_string(), "operator must be present");
    assert!(data["threshold"].is_number(), "threshold must be present");
    assert!(data["window_seconds"].is_number(), "window_seconds must be present");
}

/// Malformed body — missing `name` must yield 400.
#[actix_web::test]
async fn create_alert_rule_missing_name_returns_400() {
    let app = test::init_service(
        App::new().route("/compliance/alert-rules", web::post().to(create_alert_rule)),
    )
    .await;

    let body = serde_json::json!({
        "description": "Missing name field",
        "severity": "warning",
        "metric": "latency",
        "operator": "<",
        "threshold": 200.0,
        "window_seconds": 60
    });

    let req = test::TestRequest::post()
        .uri("/compliance/alert-rules")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
}

/// Malformed body — missing `threshold` must yield 400.
#[actix_web::test]
async fn create_alert_rule_missing_threshold_returns_400() {
    let app = test::init_service(
        App::new().route("/compliance/alert-rules", web::post().to(create_alert_rule)),
    )
    .await;

    let body = serde_json::json!({
        "name": "Missing threshold",
        "description": "No threshold",
        "severity": "warning",
        "metric": "latency",
        "operator": "<",
        "window_seconds": 60
    });

    let req = test::TestRequest::post()
        .uri("/compliance/alert-rules")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
}

/// Malformed body — missing `window_seconds` must yield 400.
#[actix_web::test]
async fn create_alert_rule_missing_window_seconds_returns_400() {
    let app = test::init_service(
        App::new().route("/compliance/alert-rules", web::post().to(create_alert_rule)),
    )
    .await;

    let body = serde_json::json!({
        "name": "Missing window",
        "description": "No window",
        "severity": "warning",
        "metric": "latency",
        "operator": "<",
        "threshold": 100.0
    });

    let req = test::TestRequest::post()
        .uri("/compliance/alert-rules")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
}

/// Empty JSON body must yield 400.
#[actix_web::test]
async fn create_alert_rule_empty_body_returns_400() {
    let app = test::init_service(
        App::new().route("/compliance/alert-rules", web::post().to(create_alert_rule)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/alert-rules")
        .insert_header(("Content-Type", "application/json"))
        .set_payload("{}")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
}

/// Integer threshold should be coerced to f64 correctly.
#[actix_web::test]
async fn create_alert_rule_integer_threshold_is_accepted() {
    let app = test::init_service(
        App::new().route("/compliance/alert-rules", web::post().to(create_alert_rule)),
    )
    .await;

    let body = serde_json::json!({
        "name": "Integer Threshold Rule",
        "description": "Threshold as integer",
        "severity": "info",
        "metric": "cpu",
        "operator": ">=",
        "threshold": 80,        // integer, not float
        "window_seconds": 120
    });

    let req = test::TestRequest::post()
        .uri("/compliance/alert-rules")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    let threshold = json["data"]["threshold"].as_f64().unwrap();
    assert!((threshold - 80.0).abs() < f64::EPSILON);
}

/// Negative threshold is valid — just echoed back.
#[actix_web::test]
async fn create_alert_rule_negative_threshold_is_accepted() {
    let app = test::init_service(
        App::new().route("/compliance/alert-rules", web::post().to(create_alert_rule)),
    )
    .await;

    let body = serde_json::json!({
        "name": "Negative Threshold Rule",
        "description": "Temperature below zero",
        "severity": "low",
        "metric": "temperature",
        "operator": "<",
        "threshold": -10.0,
        "window_seconds": 60
    });

    let req = test::TestRequest::post()
        .uri("/compliance/alert-rules")
        .set_json(&body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    let threshold = json["data"]["threshold"].as_f64().unwrap();
    assert!((threshold - (-10.0)).abs() < f64::EPSILON);
}

// ── list_alert_rules ──────────────────────────────────────────────────────────

#[actix_web::test]
async fn list_alert_rules_returns_200() {
    let app = test::init_service(
        App::new().route("/compliance/alert-rules", web::get().to(list_alert_rules)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/compliance/alert-rules")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

#[actix_web::test]
async fn list_alert_rules_success_flag_is_true() {
    let app = test::init_service(
        App::new().route("/compliance/alert-rules", web::get().to(list_alert_rules)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/compliance/alert-rules")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert_eq!(json["success"], true);
}

#[actix_web::test]
async fn list_alert_rules_data_is_empty_array() {
    let app = test::init_service(
        App::new().route("/compliance/alert-rules", web::get().to(list_alert_rules)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/compliance/alert-rules")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert!(json["data"].is_array(), "data must be an array");
    assert_eq!(
        json["data"].as_array().unwrap().len(),
        0,
        "data must be empty initially"
    );
}

#[actix_web::test]
async fn list_alert_rules_message_field_present() {
    let app = test::init_service(
        App::new().route("/compliance/alert-rules", web::get().to(list_alert_rules)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/compliance/alert-rules")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert!(json["message"].is_string());
}

/// Authorization boundaries — collector role header is forwarded but handler
/// stays 200 (role enforcement is at the middleware layer).
#[actix_web::test]
async fn list_alert_rules_with_collector_role_header_returns_200() {
    let app = test::init_service(
        App::new().route("/compliance/alert-rules", web::get().to(list_alert_rules)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/compliance/alert-rules")
        .insert_header(("Authorization", "Bearer collector-token"))
        .insert_header(("X-User-Role", "collector"))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

// ── get_audit_trail ───────────────────────────────────────────────────────────

#[actix_web::test]
async fn get_audit_trail_with_valid_query_returns_200() {
    let app = test::init_service(
        App::new().route("/compliance/audit-trail", web::get().to(get_audit_trail)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/compliance/audit-trail?requirement_id=REQ-001&status=passed")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

#[actix_web::test]
async fn get_audit_trail_success_flag_is_true() {
    let app = test::init_service(
        App::new().route("/compliance/audit-trail", web::get().to(get_audit_trail)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/compliance/audit-trail?requirement_id=REQ-001&status=passed")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert_eq!(json["success"], true);
}

#[actix_web::test]
async fn get_audit_trail_data_is_array() {
    let app = test::init_service(
        App::new().route("/compliance/audit-trail", web::get().to(get_audit_trail)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/compliance/audit-trail?requirement_id=REQ-001&status=passed")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert!(json["data"].is_array(), "data must be an array");
    assert_eq!(
        json["data"].as_array().unwrap().len(),
        0,
        "data must be empty initially"
    );
}

#[actix_web::test]
async fn get_audit_trail_with_optional_message_param_returns_200() {
    let app = test::init_service(
        App::new().route("/compliance/audit-trail", web::get().to(get_audit_trail)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/compliance/audit-trail?requirement_id=REQ-002&status=failed&message=Verification+failed")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

/// Omitting the optional `message` param must still produce 200.
#[actix_web::test]
async fn get_audit_trail_without_optional_message_returns_200() {
    let app = test::init_service(
        App::new().route("/compliance/audit-trail", web::get().to(get_audit_trail)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/compliance/audit-trail?requirement_id=REQ-003&status=pending")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

/// Missing required `requirement_id` query param must yield 400.
#[actix_web::test]
async fn get_audit_trail_missing_requirement_id_returns_400() {
    let app = test::init_service(
        App::new().route("/compliance/audit-trail", web::get().to(get_audit_trail)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/compliance/audit-trail?status=passed")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
}

/// Missing required `status` query param must yield 400.
#[actix_web::test]
async fn get_audit_trail_missing_status_returns_400() {
    let app = test::init_service(
        App::new().route("/compliance/audit-trail", web::get().to(get_audit_trail)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/compliance/audit-trail?requirement_id=REQ-001")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
}

/// No query params at all must yield 400.
#[actix_web::test]
async fn get_audit_trail_no_query_params_returns_400() {
    let app = test::init_service(
        App::new().route("/compliance/audit-trail", web::get().to(get_audit_trail)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/compliance/audit-trail")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn get_audit_trail_message_field_present() {
    let app = test::init_service(
        App::new().route("/compliance/audit-trail", web::get().to(get_audit_trail)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/compliance/audit-trail?requirement_id=REQ-001&status=passed")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert!(json["message"].is_string());
}

// ── generate_compliance_report ────────────────────────────────────────────────

#[actix_web::test]
async fn generate_compliance_report_returns_200() {
    let app = test::init_service(
        App::new().route("/compliance/report", web::post().to(generate_compliance_report)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/report")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

#[actix_web::test]
async fn generate_compliance_report_success_flag_is_true() {
    let app = test::init_service(
        App::new().route("/compliance/report", web::post().to(generate_compliance_report)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/report")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert_eq!(json["success"], true);
}

#[actix_web::test]
async fn generate_compliance_report_has_timestamp() {
    let app = test::init_service(
        App::new().route("/compliance/report", web::post().to(generate_compliance_report)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/report")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    let generated_at = json["data"]["generated_at"]
        .as_str()
        .expect("generated_at must be a string timestamp");

    assert!(
        !generated_at.is_empty(),
        "generated_at timestamp must not be empty"
    );

    // Verify it parses as a valid RFC 3339 / ISO 8601 datetime.
    let parsed = chrono::DateTime::parse_from_rfc3339(generated_at);
    assert!(
        parsed.is_ok(),
        "generated_at must be a valid RFC 3339 timestamp, got: {generated_at}"
    );
}

#[actix_web::test]
async fn generate_compliance_report_score_is_100() {
    let app = test::init_service(
        App::new().route("/compliance/report", web::post().to(generate_compliance_report)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/report")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    let score = json["data"]["compliance_score"]
        .as_f64()
        .expect("compliance_score must be a number");
    assert!(
        (score - 100.0).abs() < f64::EPSILON,
        "compliance_score must be 100.0, got {score}"
    );
}

#[actix_web::test]
async fn generate_compliance_report_has_required_data_fields() {
    let app = test::init_service(
        App::new().route("/compliance/report", web::post().to(generate_compliance_report)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/report")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    let data = &json["data"];
    assert!(data["generated_at"].is_string(), "generated_at must be present");
    assert!(data["total_requirements"].is_number(), "total_requirements must be present");
    assert!(data["passed"].is_number(), "passed must be present");
    assert!(data["failed"].is_number(), "failed must be present");
    assert!(data["compliance_score"].is_number(), "compliance_score must be present");
}

#[actix_web::test]
async fn generate_compliance_report_counts_are_zero() {
    let app = test::init_service(
        App::new().route("/compliance/report", web::post().to(generate_compliance_report)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/report")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert_eq!(json["data"]["total_requirements"], 0);
    assert_eq!(json["data"]["passed"], 0);
    assert_eq!(json["data"]["failed"], 0);
}

#[actix_web::test]
async fn generate_compliance_report_message_field_present() {
    let app = test::init_service(
        App::new().route("/compliance/report", web::post().to(generate_compliance_report)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/report")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json = body_json(resp).await;

    assert!(json["message"].is_string());
}

/// Two consecutive calls must each produce a distinct timestamp (different
/// wall-clock instant or at minimum a well-formed RFC 3339 string on both).
#[actix_web::test]
async fn generate_compliance_report_timestamp_is_rfc3339() {
    let app = test::init_service(
        App::new().route("/compliance/report", web::post().to(generate_compliance_report)),
    )
    .await;

    for _ in 0..2 {
        let req = test::TestRequest::post()
            .uri("/compliance/report")
            .to_request();
        let resp = test::call_service(&app, req).await;
        let json = body_json(resp).await;

        let ts = json["data"]["generated_at"].as_str().unwrap();
        assert!(
            chrono::DateTime::parse_from_rfc3339(ts).is_ok(),
            "generated_at must be RFC 3339, got: {ts}"
        );
    }
}

/// Authorization: admin role must get a valid report.
#[actix_web::test]
async fn generate_compliance_report_with_admin_header_returns_200() {
    let app = test::init_service(
        App::new().route("/compliance/report", web::post().to(generate_compliance_report)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/compliance/report")
        .insert_header(("Authorization", "Bearer admin-token"))
        .insert_header(("X-User-Role", "admin"))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
}

// ── cross-cutting: response envelope shape ────────────────────────────────────
//
// Every endpoint must return an object with at minimum `success` (bool) and
// `message` (string) keys regardless of the request payload.

#[actix_web::test]
async fn all_get_endpoints_return_envelope_shape() {
    // list_checklists
    {
        let app =
            test::init_service(App::new().route("/checklists", web::get().to(list_checklists))).await;
        let req = test::TestRequest::get().uri("/checklists").to_request();
        let json = body_json(test::call_service(&app, req).await).await;
        assert!(json["success"].is_boolean(), "list_checklists: success must be boolean");
        assert!(json["message"].is_string(), "list_checklists: message must be string");
    }

    // list_compliance_alerts
    {
        let app = test::init_service(
            App::new().route("/alerts", web::get().to(list_compliance_alerts)),
        )
        .await;
        let req = test::TestRequest::get().uri("/alerts").to_request();
        let json = body_json(test::call_service(&app, req).await).await;
        assert!(
            json["success"].is_boolean(),
            "list_compliance_alerts: success must be boolean"
        );
        assert!(
            json["message"].is_string(),
            "list_compliance_alerts: message must be string"
        );
    }

    // list_alert_rules
    {
        let app = test::init_service(
            App::new().route("/alert-rules", web::get().to(list_alert_rules)),
        )
        .await;
        let req = test::TestRequest::get().uri("/alert-rules").to_request();
        let json = body_json(test::call_service(&app, req).await).await;
        assert!(json["success"].is_boolean(), "list_alert_rules: success must be boolean");
        assert!(json["message"].is_string(), "list_alert_rules: message must be string");
    }
}

#[actix_web::test]
async fn all_post_endpoints_return_envelope_shape() {
    // run_compliance_check
    {
        let app = test::init_service(
            App::new().route("/check", web::post().to(run_compliance_check)),
        )
        .await;
        let req = test::TestRequest::post().uri("/check").to_request();
        let json = body_json(test::call_service(&app, req).await).await;
        assert!(
            json["success"].is_boolean(),
            "run_compliance_check: success must be boolean"
        );
        assert!(
            json["message"].is_string(),
            "run_compliance_check: message must be string"
        );
    }

    // generate_compliance_report
    {
        let app = test::init_service(
            App::new().route("/report", web::post().to(generate_compliance_report)),
        )
        .await;
        let req = test::TestRequest::post().uri("/report").to_request();
        let json = body_json(test::call_service(&app, req).await).await;
        assert!(
            json["success"].is_boolean(),
            "generate_compliance_report: success must be boolean"
        );
        assert!(
            json["message"].is_string(),
            "generate_compliance_report: message must be string"
        );
    }
}
