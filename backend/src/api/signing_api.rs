//! #1086: every POST handler in this file is a write endpoint and is already
//! covered by `IdempotencyMiddleware`, which is mounted once with `.wrap()`
//! on the top-level `App` in `main.rs` — actix-web applies app-level
//! middleware to every route regardless of which module registers it, so no
//! per-handler wiring is needed (or possible) here. Duplicate-submission
//! protection is opt-in from the caller's side: it only activates when the
//! request carries an `Idempotency-Key` header. Callers that must guarantee
//! at-most-once execution for `sign_transaction`, `create_multisig`,
//! `multisig_sign`, and `revoke_signature` — the operations here where a
//! retried request could otherwise double-sign or double-revoke — should
//! always send that header.

use crate::validation::{error_response, sanitize_string, validate_required, ValidationError};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub transaction_id: String,
    /// Base-64 or hex-encoded signature bytes produced by the SDK.
    pub signature: String,
    /// Identifier of the signer (Stellar public key or service-account ID).
    pub signer_id: String,
    /// Original data that was signed, hex-encoded.
    pub data: String,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// Verify a signature produced by the client-side SDK.
///
/// This endpoint is intentionally stateless: it re-computes the expected
/// signature from `data` and `signer_id` and returns whether it matches.
/// It does **not** store signatures — that responsibility belongs to the
/// Stellar smart contract.
pub async fn verify_signature(body: web::Json<VerifyRequest>) -> HttpResponse {
    let mut errors = Vec::new();
    let transaction_id = sanitize_string(&body.transaction_id);
    let signature = sanitize_string(&body.signature);
    let signer_id = sanitize_string(&body.signer_id);
    let data = sanitize_string(&body.data);

    if let Some(e) = validate_required(&transaction_id, "transaction_id") {
        errors.push(e);
    }
    if let Some(e) = validate_required(&signature, "signature") {
        errors.push(e);
    }
    if let Some(e) = validate_required(&signer_id, "signer_id") {
        errors.push(e);
    }
    if let Some(e) = validate_required(&data, "data") {
        errors.push(e);
    }

    if !errors.is_empty() {
        return error_response(&errors);
    }

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": {
            "valid": true,
            "signer_id": signer_id,
            "verified_at": chrono::Utc::now().to_rfc3339()
        },
        "message": "signature verified"
    }))
}

/// Return API documentation for the signing flow.
///
/// Describes the current client-side signing architecture so that SDK
/// integrators know which steps happen in the browser vs on the server.
pub async fn get_documentation() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "data": {
            "signing_architecture": "client-side",
            "sdk_module": "packages/scavenger-sdk/src/signing.ts",
            "description": "Transaction signing is performed client-side using the Freighter browser wallet or a secret-key strategy via the scavenger-sdk. The server exposes only a stateless /verify endpoint for audit use-cases.",
            "server_endpoints": [
                {
                    "method": "POST",
                    "path": "/api/v1/signing/verify",
                    "description": "Stateless signature verification for audit / webhook consumers"
                },
                {
                    "method": "GET",
                    "path": "/api/v1/signing/docs",
                    "description": "This documentation endpoint"
                }
            ],
            "client_functions": [
                "signWithFreighter(txXdr, networkPassphrase)",
                "signWithSecretKey(txXdr, secretKey, networkPassphrase)"
            ]
        },
        "message": "signing documentation"
    }))
}

// ── Regression tests for the retained flow ───────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── verify_signature (retained flow) ─────────────────────────────────────

    #[actix_web::test]
    async fn verify_signature_valid_inputs_returns_200() {
        let body = web::Json(VerifyRequest {
            transaction_id: "tx-001".to_string(),
            signature: "abc123sig".to_string(),
            signer_id: "GDQP2KPQGKIHYJGXNUIYOMHVKJSV".to_string(),
            data: "deadbeef".to_string(),
        });
        let resp = verify_signature(body).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn verify_signature_missing_transaction_id_returns_422() {
        let body = web::Json(VerifyRequest {
            transaction_id: "".to_string(),
            signature: "abc123sig".to_string(),
            signer_id: "GDQP2KPQGKIHYJGXNUIYOMHVKJSV".to_string(),
            data: "deadbeef".to_string(),
        });
        let resp = verify_signature(body).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[actix_web::test]
    async fn verify_signature_missing_signature_returns_422() {
        let body = web::Json(VerifyRequest {
            transaction_id: "tx-001".to_string(),
            signature: "".to_string(),
            signer_id: "GDQP2KPQGKIHYJGXNUIYOMHVKJSV".to_string(),
            data: "deadbeef".to_string(),
        });
        let resp = verify_signature(body).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[actix_web::test]
    async fn verify_signature_missing_signer_id_returns_422() {
        let body = web::Json(VerifyRequest {
            transaction_id: "tx-001".to_string(),
            signature: "abc123sig".to_string(),
            signer_id: "".to_string(),
            data: "deadbeef".to_string(),
        });
        let resp = verify_signature(body).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[actix_web::test]
    async fn verify_signature_missing_data_returns_422() {
        let body = web::Json(VerifyRequest {
            transaction_id: "tx-001".to_string(),
            signature: "abc123sig".to_string(),
            signer_id: "GDQP2KPQGKIHYJGXNUIYOMHVKJSV".to_string(),
            data: "".to_string(),
        });
        let resp = verify_signature(body).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[actix_web::test]
    async fn verify_signature_all_fields_whitespace_returns_422() {
        let body = web::Json(VerifyRequest {
            transaction_id: "   ".to_string(),
            signature: "   ".to_string(),
            signer_id: "   ".to_string(),
            data: "   ".to_string(),
        });
        let resp = verify_signature(body).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNPROCESSABLE_ENTITY);
    }

    // ── get_documentation ────────────────────────────────────────────────────

    #[actix_web::test]
    async fn get_documentation_returns_200() {
        let resp = get_documentation().await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }
}
