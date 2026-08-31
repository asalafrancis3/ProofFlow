//! Rate limit middleware
//! 
//! This module provides configurable rate limiting for API endpoints.
//! Rate limits can be configured via environment variables without code changes.
//! 
//! # Configuration Options
//! 
//! | Environment Variable | Description | Default |
//! |----------------------|-------------|---------|
//! | `RATE_LIMIT_DEFAULT` | Default rate limit (requests per window) | 100 |
//! | `RATE_LIMIT_WINDOW` | Default time window in seconds | 60 |
//! | `RATE_LIMIT_ADMIN` | Admin rate limit (limit,window) | 500,60 |
//! | `RATE_LIMIT_AUTH` | Auth user rate limit (limit,window) | 200,60 |
//! | `RATE_LIMIT_UNAUTH` | Unauthenticated user rate limit (limit,window) | 20,60 |
//! | `RATE_LIMIT_ROUTES` | Per-route overrides (method,route,limit,window;...) | None |
//! 
//! # Example
//! 
//! ```bash
//! # Set rate limits for production
//! export RATE_LIMIT_DEFAULT=100
//! export RATE_LIMIT_WINDOW=60
//! export RATE_LIMIT_ADMIN=500,60
//! export RATE_LIMIT_AUTH=200,60
//! export RATE_LIMIT_UNAUTH=20,60
//! export RATE_LIMIT_ROUTES=POST,/api/waste,50,60;GET,/api/export,30,120
//! ```

use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, HttpMessage, HttpResponse,
};
use futures::future::{ok, Ready};
use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use crate::redis::RedisClient;

// ============================================
// Rate Limit Tiers (from main branch)
// ============================================

#[derive(Clone, Copy, Debug)]
pub enum RateLimitTier {
    Anonymous,
    Free,
    Premium,
    Admin,
}

impl RateLimitTier {
    pub fn config(&self) -> RateLimitConfig {
        match self {
            RateLimitTier::Anonymous => RateLimitConfig {
                requests_per_minute: 30,
                requests_per_hour: 200,
            },
            RateLimitTier::Free => RateLimitConfig {
                requests_per_minute: 60,
                requests_per_hour: 1000,
            },
            RateLimitTier::Premium => RateLimitConfig {
                requests_per_minute: 300,
                requests_per_hour: 5000,
            },
            RateLimitTier::Admin => RateLimitConfig {
                requests_per_minute: 1000,
                requests_per_hour: 50000,
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        RateLimitTier::Free.config()
    }
}

/// Per-route prefix override. The first matching prefix wins.
#[derive(Clone, Debug)]
pub struct RouteRateLimitConfig {
    pub prefix: String,
    pub config: RateLimitConfig,
}

impl RouteRateLimitConfig {
    pub fn new(prefix: impl Into<String>, tier: RateLimitTier) -> Self {
        Self {
            prefix: prefix.into(),
            config: tier.config(),
        }
    }
}

// ── Metrics ───────────────────────────────────────────────────────────────────

#[derive(Debug, Default, Clone)]
pub struct RateLimitMetrics {
    pub total_requests: u64,
    pub rate_limited_requests: u64,
    pub by_tier: HashMap<String, u64>,
}

// ── Internal sliding-window state ─────────────────────────────────────────────

struct RateLimitState {
    minute_buckets: HashMap<String, Vec<Instant>>,
    hour_buckets: HashMap<String, Vec<Instant>>,
    metrics: RateLimitMetrics,
}

impl RateLimitState {
    fn new() -> Self {
        Self {
            minute_buckets: HashMap::new(),
            hour_buckets: HashMap::new(),
            metrics: RateLimitMetrics::default(),
        }
    }

    /// Check and record a request for `key` against `config`.
    /// Returns `Ok((remaining_min, remaining_hr))` or `Err(retry_after_secs)`.
    fn check_and_record(&mut self, key: &str, config: &RateLimitConfig) -> Result<(usize, usize), u64> {
        self.metrics.total_requests += 1;
        let now = Instant::now();

        // Minute window
        let min_key = format!("{}:min", key);
        let min_bucket = self.minute_buckets.entry(min_key.clone()).or_default();
        min_bucket.retain(|t| now.duration_since(*t) < Duration::from_secs(60));
        let min_count = min_bucket.len();

        // Hour window
        let hr_key = format!("{}:hr", key);
        let hr_bucket = self.hour_buckets.entry(hr_key.clone()).or_default();
        hr_bucket.retain(|t| now.duration_since(*t) < Duration::from_secs(3600));
        let hr_count = hr_bucket.len();

        if min_count >= config.requests_per_minute as usize {
            self.metrics.rate_limited_requests += 1;
            let retry_after = self
                .minute_buckets
                .get(&min_key)
                .and_then(|b| b.first().copied())
                .map(|oldest| {
                    let elapsed = now.duration_since(oldest).as_secs();
                    60_u64.saturating_sub(elapsed)
                })
                .unwrap_or(60)
                .max(1);
            return Err(retry_after);
        }

        if hr_count >= config.requests_per_hour as usize {
            self.metrics.rate_limited_requests += 1;
            let retry_after = self
                .hour_buckets
                .get(&hr_key)
                .and_then(|b| b.first().copied())
                .map(|oldest| {
                    let elapsed = now.duration_since(oldest).as_secs();
                    3600_u64.saturating_sub(elapsed)
                })
                .unwrap_or(3600)
                .max(1);
            return Err(retry_after);
        }

        // Record this request
        self.minute_buckets.entry(min_key).or_default().push(now);
        self.hour_buckets.entry(hr_key).or_default().push(now);

        let remaining_min = (config.requests_per_minute as usize).saturating_sub(min_count + 1);
        let remaining_hr = (config.requests_per_hour as usize).saturating_sub(hr_count + 1);
        Ok((remaining_min, remaining_hr))
    }
}

// ============================================
// Rate Limit Middleware
// ============================================

/// Rate limit middleware
pub struct RateLimit {
    config: RateLimitConfig,
    redis: web::Data<RedisClient>,
}

impl RateLimit {
    pub fn new(config: RateLimitConfig, redis: web::Data<RedisClient>) -> Self {
        Self { config, redis }
    }
}

/// Rate limit service factory
pub struct RateLimitMiddleware {
    config: RateLimitConfig,
    redis: web::Data<RedisClient>,
    routes: Vec<RouteRateLimitConfig>,
    state: std::sync::Arc<std::sync::Mutex<RateLimitState>>,
}

impl RateLimitMiddleware {
    pub fn new(config: RateLimitConfig, redis: web::Data<RedisClient>) -> Self {
        Self {
            config: config.clone(),
            redis,
            routes: Vec::new(),
            state: std::sync::Arc::new(std::sync::Mutex::new(RateLimitState::new())),
        }
    }

    /// Add a per-route override
    pub fn route(mut self, prefix: impl Into<String>, tier: RateLimitTier) -> Self {
        self.routes.push(RouteRateLimitConfig::new(prefix, tier));
        self
    }

    /// Get config for a path based on route overrides
    fn config_for_path(&self, path: &str) -> RateLimitConfig {
        for route in &self.routes {
            if path.starts_with(&route.prefix) {
                return route.config.clone();
            }
        }
        self.config.clone()
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RateLimitService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimitService {
            service: Rc::new(RefCell::new(service)),
            config: self.config.clone(),
            redis: self.redis.clone(),
            routes: self.routes.clone(),
            state: self.state.clone(),
        })
    }
}

/// Rate limit service implementation
pub struct RateLimitService<S> {
    service: Rc<RefCell<S>>,
    config: RateLimitConfig,
    redis: web::Data<RedisClient>,
    routes: Vec<RouteRateLimitConfig>,
    state: std::sync::Arc<std::sync::Mutex<RateLimitState>>,
}

impl<S, B> Service<ServiceRequest> for RateLimitService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.borrow_mut().poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let client_ip = req.connection_info().peer_addr().unwrap_or("unknown").to_string();
        let path = req.path().to_string();
        let config = self.config_for_path(&path);

        let route_prefix = path.split('/').take(4).collect::<Vec<_>>().join("/");
        let key = format!("{}:{}", client_ip, route_prefix);

        let check_result = {
            let mut s = self.state.lock().unwrap();
            s.check_and_record(&key, &config)
        };

        let rpm = config.requests_per_minute.to_string();
        let rph = config.requests_per_hour.to_string();

        match check_result {
            Err(retry_after) => {
                let response = HttpResponse::TooManyRequests()
                    .insert_header(("Retry-After", retry_after.to_string()))
                    .insert_header(("X-RateLimit-Limit-Minute", rpm.clone()))
                    .insert_header(("X-RateLimit-Limit-Hour", rph.clone()))
                    .insert_header(("X-RateLimit-Remaining-Minute", "0"))
                    .insert_header(("X-RateLimit-Remaining-Hour", "0"))
                    .json(serde_json::json!({
                        "error": "rate_limit_exceeded",
                        "message": format!("Too many requests. Try again in {} seconds.", retry_after),
                        "retry_after_seconds": retry_after,
                        "limit_minute": config.requests_per_minute,
                        "limit_hour": config.requests_per_hour,
                    }));
                Box::pin(async move {
                    Err(actix_web::error::InternalError::from_response("rate_limit_exceeded", response).into())
                })
            }
            Ok((remaining_min, remaining_hr)) => {
                let service = self.service.clone();
                Box::pin(async move {
                    let mut res = service.call(req).await?;
                    let h = res.headers_mut();
                    use actix_web::http::header::{HeaderName, HeaderValue};
                    let insert = |h: &mut actix_web::http::header::HeaderMap, k: &'static str, v: String| {
                        if let Ok(val) = HeaderValue::from_str(&v) {
                            h.insert(HeaderName::from_static(k), val);
                        }
                    };
                    insert(h, "x-ratelimit-limit-minute", rpm);
                    insert(h, "x-ratelimit-limit-hour", rph);
                    insert(h, "x-ratelimit-remaining-minute", remaining_min.to_string());
                    insert(h, "x-ratelimit-remaining-hour", remaining_hr.to_string());
                    Ok(res)
                })
            }
        }
    }
}

impl<S> RateLimitService<S> {
    fn config_for_path(&self, path: &str) -> RateLimitConfig {
        for route in &self.routes {
            if path.starts_with(&route.prefix) {
                return route.config.clone();
            }
        }
        self.config.clone()
    }
}

// ============================================
// Helper Functions
// ============================================

/// Get client identifier from request
fn get_client_id(req: &ServiceRequest) -> Option<String> {
    if let Some(claims) = req.extensions().get::<serde_json::Value>() {
        if let Some(user_id) = claims.get("sub").and_then(|v| v.as_str()) {
            return Some(user_id.to_string());
        }
    }
    
    if let Some(ip) = req.connection_info().realip_remote_addr() {
        return Some(ip.to_string());
    }
    
    if let Some(user_agent) = req.headers().get("user-agent").and_then(|v| v.to_str().ok()) {
        if let Some(ip) = req.connection_info().realip_remote_addr() {
            return Some(format!("{}:{}", ip, user_agent));
        }
    }
    
    None
}

/// Get request count from Redis
async fn get_request_count(redis: &web::Data<RedisClient>, key: &str) -> Result<u32, anyhow::Error> {
    let count = redis.get::<u32>(key).await?;
    Ok(count.unwrap_or(0))
}

/// Increment request count in Redis with TTL
async fn increment_request_count(
    redis: &web::Data<RedisClient>,
    key: &str,
    window_secs: u64,
) -> Result<(), anyhow::Error> {
    let count = redis.incr(key).await?;
    if count == 1 {
        redis.expire(key, window_secs as u64).await?;
    }
    Ok(())
}

// ============================================
// Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rate_limit_tiers() {
        assert!(
            RateLimitTier::Anonymous.config().requests_per_minute < RateLimitTier::Free.config().requests_per_minute
        );
        assert!(RateLimitTier::Free.config().requests_per_minute < RateLimitTier::Premium.config().requests_per_minute);
        assert!(
            RateLimitTier::Premium.config().requests_per_minute < RateLimitTier::Admin.config().requests_per_minute
        );
    }

    #[test]
    fn test_check_and_record_exceeds_hour_limit() {
        let mut state = RateLimitState::new();
        let config = RateLimitConfig {
            requests_per_minute: 1000,
            requests_per_hour: 2,
        };
        for _ in 0..2 {
            let _ = state.check_and_record("ip1", &config);
        }
        assert!(state.check_and_record("ip1", &config).is_err());
    }

    #[test]
    fn test_different_ips_are_isolated() {
        let mut state = RateLimitState::new();
        let config = RateLimitConfig {
            requests_per_minute: 1,
            requests_per_hour: 100,
        };
        let _ = state.check_and_record("ip1", &config);
        assert!(
            state.check_and_record("ip2", &config).is_ok(),
            "ip2 should not be affected by ip1's limit"
        );
    }

    #[test]
    fn test_config_for_path_contracts() {
        let mw = RateLimitMiddleware::new(RateLimitConfig::default(), web::Data::new(RedisClient::new("redis://localhost").unwrap()));
        let c = mw.config_for_path("/api/v1/contracts/wastes");
        assert_eq!(
            c.requests_per_minute,
            RateLimitTier::Anonymous.config().requests_per_minute
        );
    }

    #[test]
    fn test_config_for_path_search() {
        let mw = RateLimitMiddleware::new(RateLimitConfig::default(), web::Data::new(RedisClient::new("redis://localhost").unwrap()));
        let c = mw.config_for_path("/api/v1/search");
        assert_eq!(
            c.requests_per_minute,
            RateLimitTier::Anonymous.config().requests_per_minute
        );
    }

    #[test]
    fn test_metrics_tracking() {
        let mut state = RateLimitState::new();
        let config = RateLimitConfig {
            requests_per_minute: 1,
            requests_per_hour: 100,
        };
        let _ = state.check_and_record("ip", &config);
        let _ = state.check_and_record("ip", &config);
        assert_eq!(state.metrics.total_requests, 2);
        assert_eq!(state.metrics.rate_limited_requests, 1);
    }

    #[test]
    fn test_rate_limit_layer_builder() {
        let mw = RateLimitMiddleware::new(RateLimitConfig::default(), web::Data::new(RedisClient::new("redis://localhost").unwrap()))
            .route("/api/v1/admin/", RateLimitTier::Admin);
        
        let admin_config = mw.config_for_path("/api/v1/admin/users");
        assert_eq!(
            admin_config.requests_per_minute,
            RateLimitTier::Admin.config().requests_per_minute
        );
    }
}