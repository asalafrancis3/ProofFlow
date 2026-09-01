//! Stellar RPC / Horizon client with automatic retry and exponential backoff.
//!
//! Created as part of #907 — Extract Stellar RPC access into a single client module.
//!
//! # Design
//!
//! - One `StellarRpcClient` per application instance (created in `main.rs`, shared via `Arc`).
//! - All Stellar Horizon / Soroban RPC calls go through this module — no scattered `reqwest`
//!   calls in handlers.
//! - Retries are handled with full-jitter exponential backoff (see [`RetryConfig`]).
//! - Errors are mapped to the unified [`RpcError`] type.
//!
//! # Example
//!
//! ```ignore
//! let client = StellarRpcClient::new(StellarRpcConfig::from_env());
//! let account = client.get_account("GABC...").await?;
//! let ledger  = client.get_latest_ledger().await?;
//! ```

use std::time::Duration;

use anyhow::Context;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    #[error("HTTP error {status}: {message}")]
    Http { status: u16, message: String },

    #[error("Deserialization error: {0}")]
    Deserialize(#[from] serde_json::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("All {attempts} retry attempts exhausted: {last_error}")]
    RetriesExhausted { attempts: u32, last_error: String },

    #[error("Rate limited by Stellar node (429)")]
    RateLimited,
}

// ── Configuration ─────────────────────────────────────────────────────────────

/// Retry strategy for transient failures.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of attempts (initial + retries).
    pub max_attempts: u32,
    /// Base delay between retries.
    pub base_delay: Duration,
    /// Cap on exponential backoff.
    pub max_delay: Duration,
    /// HTTP status codes that trigger a retry.
    pub retryable_status: Vec<u16>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(5),
            retryable_status: vec![408, 429, 500, 502, 503, 504],
        }
    }
}

impl RetryConfig {
    /// Calculate delay for attempt `n` using full-jitter exponential backoff.
    pub(crate) fn delay_for(&self, attempt: u32) -> Duration {
        let exp = self.base_delay.as_millis() as f64 * (2_f64.powi(attempt as i32));
        let capped = exp.min(self.max_delay.as_millis() as f64);
        // Full jitter: random in [0, capped]
        let jitter = rand::random::<f64>() * capped;
        Duration::from_millis(jitter as u64)
    }
}

/// Connection and endpoint configuration for the Stellar client.
#[derive(Debug, Clone)]
pub struct StellarRpcConfig {
    /// Stellar Horizon base URL (e.g. `https://horizon-testnet.stellar.org`).
    pub horizon_url: String,
    /// Soroban RPC base URL (e.g. `https://soroban-testnet.stellar.org`).
    pub soroban_rpc_url: String,
    /// Network passphrase (`"Test SDF Network ; September 2015"` or mainnet).
    pub network_passphrase: String,
    /// Soroban contract ID to query.
    pub contract_id: String,
    /// Per-request timeout.
    pub request_timeout: Duration,
    /// Retry configuration.
    pub retry: RetryConfig,
}

impl StellarRpcConfig {
    /// Build config from environment variables with sensible defaults.
    pub fn from_env() -> Self {
        Self {
            horizon_url: std::env::var("STELLAR_HORIZON_URL")
                .unwrap_or_else(|_| "https://horizon-testnet.stellar.org".to_string()),
            soroban_rpc_url: std::env::var("SOROBAN_RPC_URL")
                .unwrap_or_else(|_| "https://soroban-testnet.stellar.org".to_string()),
            network_passphrase: std::env::var("STELLAR_NETWORK_PASSPHRASE")
                .unwrap_or_else(|_| "Test SDF Network ; September 2015".to_string()),
            contract_id: std::env::var("CONTRACT_ID")
                .unwrap_or_else(|_| "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4".to_string()),
            request_timeout: Duration::from_secs(
                std::env::var("STELLAR_REQUEST_TIMEOUT_SECS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(10),
            ),
            retry: RetryConfig::default(),
        }
    }
}

// ── Response types ────────────────────────────────────────────────────────────

/// Stellar account from Horizon `/accounts/{id}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarAccount {
    pub id: String,
    pub sequence: String,
    pub balances: Vec<Balance>,
    pub thresholds: Thresholds,
    pub flags: AccountFlags,
    pub last_modified_ledger: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub balance: String,
    pub asset_type: String,
    pub asset_code: Option<String>,
    pub asset_issuer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thresholds {
    pub low_threshold: u8,
    pub med_threshold: u8,
    pub high_threshold: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountFlags {
    pub auth_required: bool,
    pub auth_revocable: bool,
    pub auth_immutable: bool,
}

/// Latest ledger information from Horizon `/ledgers?order=desc&limit=1`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestLedger {
    pub sequence: u64,
    pub closed_at: String,
    pub transaction_count: u64,
    pub operation_count: u64,
    pub successful_transaction_count: Option<u64>,
    pub failed_transaction_count: Option<u64>,
}

/// Contract data entry from Soroban RPC `getContractData`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractDataEntry {
    pub key: String,
    pub value: serde_json::Value,
    pub ledger: u64,
    pub ledger_expiration: Option<u64>,
}

/// Soroban RPC `getLedgerEntries` response envelope.
#[derive(Debug, Clone, Deserialize)]
struct SorobanRpcResponse<T> {
    pub id: u32,
    pub jsonrpc: String,
    pub result: Option<T>,
    pub error: Option<SorobanRpcError>,
}

#[derive(Debug, Clone, Deserialize)]
struct SorobanRpcError {
    pub code: i64,
    pub message: String,
}

/// Transaction submission result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    pub hash: String,
    pub successful: bool,
    pub ledger: Option<u64>,
    pub envelope_xdr: String,
    pub result_xdr: Option<String>,
}

// ── Horizon paginated envelope ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct HorizonEmbedded<T> {
    pub records: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct HorizonPage<T> {
    #[serde(rename = "_embedded")]
    pub embedded: HorizonEmbedded<T>,
}

// ── Client ────────────────────────────────────────────────────────────────────

/// Shared Stellar/Soroban RPC client.
///
/// Wrap in `Arc` and register as `web::Data` in the actix-web app to share
/// across request handlers.
#[derive(Clone)]
pub struct StellarRpcClient {
    http: Client,
    config: StellarRpcConfig,
}

impl StellarRpcClient {
    /// Create a new client from config.
    pub fn new(config: StellarRpcConfig) -> Result<Self, anyhow::Error> {
        let http = Client::builder()
            .timeout(config.request_timeout)
            .user_agent("proofflow-backend/1.0")
            .build()
            .context("Failed to build HTTP client")?;

        info!(
            horizon = %config.horizon_url,
            rpc = %config.soroban_rpc_url,
            contract = %config.contract_id,
            "Stellar RPC client initialised"
        );

        Ok(Self { http, config })
    }

    // ── Internal retry machinery ──────────────────────────────────────────────

    /// Execute an async closure with retry/backoff. The closure receives the
    /// `reqwest::Client` and should return `Result<T, RpcError>`.
    async fn with_retry<F, Fut, T>(&self, label: &str, f: F) -> Result<T, RpcError>
    where
        F: Fn(Client) -> Fut,
        Fut: std::future::Future<Output = Result<T, RpcError>>,
    {
        let mut last_error = String::new();
        for attempt in 0..self.config.retry.max_attempts {
            if attempt > 0 {
                let delay = self.config.retry.delay_for(attempt - 1);
                warn!(
                    label,
                    attempt,
                    delay_ms = delay.as_millis(),
                    "Retrying Stellar RPC call"
                );
                tokio::time::sleep(delay).await;
            }

            match f(self.http.clone()).await {
                Ok(result) => {
                    if attempt > 0 {
                        info!(label, attempt, "Stellar RPC call succeeded after retry");
                    }
                    return Ok(result);
                }
                Err(RpcError::Http { status, message }) => {
                    last_error = format!("HTTP {status}: {message}");
                    if self.config.retry.retryable_status.contains(&status) {
                        if status == 429 {
                            return Err(RpcError::RateLimited);
                        }
                        debug!(label, attempt, status, "Transient HTTP error, will retry");
                        continue;
                    }
                    // Non-retryable HTTP error — fail fast
                    return Err(RpcError::Http { status, message });
                }
                Err(RpcError::Network(ref inner)) if inner.is_timeout() || inner.is_connect() => {
                    last_error = format!("network error (transient)");
                    debug!(label, attempt, "Transient network error, will retry");
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        Err(RpcError::RetriesExhausted {
            attempts: self.config.retry.max_attempts,
            last_error,
        })
    }

    // ── Horizon endpoints ─────────────────────────────────────────────────────

    /// Fetch an account by Stellar address.
    pub async fn get_account(&self, account_id: &str) -> Result<StellarAccount, RpcError> {
        let url = format!("{}/accounts/{}", self.config.horizon_url, account_id);
        self.with_retry("get_account", |client| {
            let url = url.clone();
            async move {
                let res = client.get(&url).send().await?;
                let status = res.status();
                if !status.is_success() {
                    let body = res.text().await.unwrap_or_default();
                    return Err(RpcError::Http {
                        status: status.as_u16(),
                        message: body,
                    });
                }
                let text = res.text().await?;
                serde_json::from_str::<StellarAccount>(&text).map_err(RpcError::Deserialize)
            }
        })
        .await
    }

    /// Fetch the most recently closed ledger.
    pub async fn get_latest_ledger(&self) -> Result<LatestLedger, RpcError> {
        let url = format!("{}/ledgers?order=desc&limit=1", self.config.horizon_url);
        self.with_retry("get_latest_ledger", |client| {
            let url = url.clone();
            async move {
                let res = client.get(&url).send().await?;
                let status = res.status();
                if !status.is_success() {
                    let body = res.text().await.unwrap_or_default();
                    return Err(RpcError::Http {
                        status: status.as_u16(),
                        message: body,
                    });
                }
                let page = res.json::<HorizonPage<LatestLedger>>().await?;
                page.embedded.records.into_iter().next().ok_or_else(|| RpcError::Http {
                    status: 404,
                    message: "No ledger records returned".to_string(),
                })
            }
        })
        .await
    }

    /// Submit a signed XDR transaction envelope to Horizon.
    pub async fn submit_transaction(&self, xdr: &str) -> Result<TransactionResult, RpcError> {
        let url = format!("{}/transactions", self.config.horizon_url);
        let body = format!("tx={}", urlencoding::encode(xdr));
        self.with_retry("submit_transaction", |client| {
            let url = url.clone();
            let body = body.clone();
            async move {
                let res = client
                    .post(&url)
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .body(body)
                    .send()
                    .await?;
                let status = res.status();
                if !status.is_success() {
                    let body = res.text().await.unwrap_or_default();
                    return Err(RpcError::Http {
                        status: status.as_u16(),
                        message: body,
                    });
                }
                Ok(res.json::<TransactionResult>().await?)
            }
        })
        .await
    }

    // ── Soroban RPC endpoints ─────────────────────────────────────────────────

    /// Call a Soroban JSON-RPC method.
    async fn soroban_call<P, R>(&self, method: &str, params: P) -> Result<R, RpcError>
    where
        P: Serialize + Clone + Send + 'static,
        R: for<'de> Deserialize<'de> + 'static,
    {
        let url = self.config.soroban_rpc_url.clone();
        let method = method.to_string();
        self.with_retry("soroban_rpc", move |client| {
            let url = url.clone();
            let method = method.clone();
            let params = params.clone();
            async move {
                let payload = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": method,
                    "params": params,
                });
                let res = client.post(&url).json(&payload).send().await?;
                let status = res.status();
                if !status.is_success() {
                    let body = res.text().await.unwrap_or_default();
                    return Err(RpcError::Http {
                        status: status.as_u16(),
                        message: body,
                    });
                }
                let envelope: SorobanRpcResponse<R> = res.json().await?;
                if let Some(err) = envelope.error {
                    return Err(RpcError::Http {
                        status: 200,
                        message: format!("Soroban RPC error {}: {}", err.code, err.message),
                    });
                }
                envelope.result.ok_or_else(|| RpcError::Http {
                    status: 200,
                    message: "Soroban RPC returned null result".to_string(),
                })
            }
        })
        .await
    }

    /// Fetch contract data entries by key XDR strings.
    pub async fn get_ledger_entries(&self, keys: Vec<String>) -> Result<Vec<ContractDataEntry>, RpcError> {
        #[derive(Serialize, Clone)]
        struct Params {
            keys: Vec<String>,
        }

        #[derive(Deserialize)]
        struct Result_ {
            entries: Option<Vec<ContractDataEntry>>,
        }

        let result: Result_ = self.soroban_call("getLedgerEntries", Params { keys }).await?;
        Ok(result.entries.unwrap_or_default())
    }

    /// Simulate a contract call (read-only, no fee).
    pub async fn simulate_transaction(&self, xdr: &str) -> Result<serde_json::Value, RpcError> {
        #[derive(Serialize, Clone)]
        struct Params {
            transaction: String,
        }

        self.soroban_call::<_, serde_json::Value>(
            "simulateTransaction",
            Params {
                transaction: xdr.to_string(),
            },
        )
        .await
    }

    /// Expose config for diagnostic/health endpoints.
    pub fn horizon_url(&self) -> &str {
        &self.config.horizon_url
    }

    pub fn contract_id(&self) -> &str {
        &self.config.contract_id
    }

    pub fn network_passphrase(&self) -> &str {
        &self.config.network_passphrase
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn test_config() -> StellarRpcConfig {
        StellarRpcConfig {
            horizon_url: "https://horizon-testnet.stellar.org".to_string(),
            soroban_rpc_url: "https://soroban-testnet.stellar.org".to_string(),
            network_passphrase: "Test SDF Network ; September 2015".to_string(),
            contract_id: "CTEST".to_string(),
            request_timeout: Duration::from_secs(5),
            retry: RetryConfig {
                max_attempts: 2,
                base_delay: Duration::from_millis(10),
                max_delay: Duration::from_millis(100),
                retryable_status: vec![429, 500, 503],
            },
        }
    }

    #[test]
    fn test_client_builds_ok() {
        let config = test_config();
        let client = StellarRpcClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_retry_config_delay() {
        let config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_millis(1000),
            retryable_status: vec![500],
        };
        // Delay should be in range [0, max_delay]
        for attempt in 0..5 {
            let d = config.delay_for(attempt);
            assert!(d <= config.max_delay, "delay {:?} exceeded cap", d);
        }
    }

    #[test]
    fn test_horizon_url_accessor() {
        let config = test_config();
        let client = StellarRpcClient::new(config).unwrap();
        assert_eq!(client.horizon_url(), "https://horizon-testnet.stellar.org");
    }

    #[test]
    fn test_contract_id_accessor() {
        let config = test_config();
        let client = StellarRpcClient::new(config).unwrap();
        assert_eq!(client.contract_id(), "CTEST");
    }

    #[test]
    fn test_config_from_env_defaults() {
        // With no env vars set, from_env() should return well-formed defaults
        let config = StellarRpcConfig::from_env();
        assert!(!config.horizon_url.is_empty());
        assert!(!config.soroban_rpc_url.is_empty());
        assert!(!config.network_passphrase.is_empty());
        assert!(config.retry.max_attempts > 0);
    }

    #[test]
    fn test_rpc_error_display() {
        let e = RpcError::Http {
            status: 404,
            message: "not found".to_string(),
        };
        assert!(e.to_string().contains("404"));

        let e2 = RpcError::RetriesExhausted {
            attempts: 3,
            last_error: "timeout".to_string(),
        };
        assert!(e2.to_string().contains("3"));

        let e3 = RpcError::RateLimited;
        assert!(e3.to_string().contains("429"));
    }
}
