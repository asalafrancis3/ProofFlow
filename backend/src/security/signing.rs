use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureScheme {
    pub algorithm: String,
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureRequest {
    pub transaction_id: String,
    pub data: Vec<u8>,
    pub signature: Vec<u8>,
    pub signer_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureValidation {
    pub valid: bool,
    pub signer_id: String,
    pub verified_at: chrono::DateTime<chrono::Utc>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSignatureSupport {
    pub transaction_id: String,
    pub required_signatures: u32,
    pub signatures: Vec<SignatureRequest>,
    pub status: SignatureStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignatureStatus {
    Pending,
    Partial { current: u32, required: u32 },
    Complete,
    Expired,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureRevocation {
    pub transaction_id: String,
    pub revoked_at: chrono::DateTime<chrono::Utc>,
    pub revoked_by: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureEvent {
    pub id: String,
    pub event_type: String,
    pub transaction_id: String,
    pub signer_id: Option<String>,
    pub details: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct TransactionSigningService {
    signatures: Arc<Mutex<HashMap<String, SignatureRequest>>>,
    multi_sig: Arc<Mutex<HashMap<String, MultiSignatureSupport>>>,
    revocations: Arc<Mutex<Vec<SignatureRevocation>>>,
    events: Arc<Mutex<Vec<SignatureEvent>>>,
    scheme: SignatureScheme,
}

impl TransactionSigningService {
    pub fn new(scheme: SignatureScheme) -> Self {
        Self {
            signatures: Arc::new(Mutex::new(HashMap::new())),
            multi_sig: Arc::new(Mutex::new(HashMap::new())),
            revocations: Arc::new(Mutex::new(Vec::new())),
            events: Arc::new(Mutex::new(Vec::new())),
            scheme,
        }
    }

    pub fn sign(&self, transaction_id: &str, signer_id: &str, data: &[u8]) -> SignatureRequest {
        let signature = self.create_signature(data);
        let request = SignatureRequest {
            transaction_id: transaction_id.to_string(),
            data: data.to_vec(),
            signature,
            signer_id: signer_id.to_string(),
            timestamp: Utc::now(),
        };
        self.signatures
            .lock()
            .unwrap()
            .insert(transaction_id.to_string(), request.clone());
        self.record_event(
            "sign",
            transaction_id,
            Some(signer_id),
            format!("signature created by {}", signer_id),
        );
        request
    }

    pub fn verify(&self, request: &SignatureRequest) -> SignatureValidation {
        match self.verify_signature(&request.data, &request.signature) {
            Ok(true) => SignatureValidation {
                valid: true,
                signer_id: request.signer_id.clone(),
                verified_at: Utc::now(),
                error: None,
            },
            Ok(false) => SignatureValidation {
                valid: false,
                signer_id: request.signer_id.clone(),
                verified_at: Utc::now(),
                error: Some("invalid signature".to_string()),
            },
            Err(e) => SignatureValidation {
                valid: false,
                signer_id: request.signer_id.clone(),
                verified_at: Utc::now(),
                error: Some(e),
            },
        }
    }

    pub fn create_multisig(&self, transaction_id: &str, required: u32) -> MultiSignatureSupport {
        let multi = MultiSignatureSupport {
            transaction_id: transaction_id.to_string(),
            required_signatures: required,
            signatures: Vec::new(),
            status: SignatureStatus::Pending,
        };
        self.multi_sig
            .lock()
            .unwrap()
            .insert(transaction_id.to_string(), multi.clone());
        self.record_event(
            "multisig_create",
            transaction_id,
            None,
            format!("multisig created requiring {} signatures", required),
        );
        multi
    }

    pub fn add_multisig_signature(
        &self,
        transaction_id: &str,
        request: SignatureRequest,
    ) -> Result<MultiSignatureSupport, String> {
        let signer_id = request.signer_id.clone();
        let mut multi = self
            .multi_sig
            .lock()
            .unwrap()
            .get(transaction_id)
            .cloned()
            .ok_or("multisig transaction not found")?;
        multi.signatures.push(request);
        let current = multi.signatures.len() as u32;
        multi.status = if current >= multi.required_signatures {
            SignatureStatus::Complete
        } else {
            SignatureStatus::Partial {
                current,
                required: multi.required_signatures,
            }
        };
        self.record_event(
            "multisig_add",
            transaction_id,
            Some(&signer_id),
            format!("signature added ({}/{})", current, multi.required_signatures),
        );
        self.multi_sig
            .lock()
            .unwrap()
            .insert(transaction_id.to_string(), multi.clone());
        Ok(multi)
    }

    pub fn revoke(&self, transaction_id: &str, revoked_by: &str, reason: &str) -> Result<SignatureRevocation, String> {
        self.signatures.lock().unwrap().remove(transaction_id);
        let revocation = SignatureRevocation {
            transaction_id: transaction_id.to_string(),
            revoked_at: Utc::now(),
            revoked_by: revoked_by.to_string(),
            reason: reason.to_string(),
        };
        self.revocations.lock().unwrap().push(revocation.clone());
        self.record_event(
            "revoke",
            transaction_id,
            Some(revoked_by),
            format!("revoked: {}", reason),
        );
        Ok(revocation)
    }

    pub fn get_signature(&self, transaction_id: &str) -> Option<SignatureRequest> {
        self.signatures.lock().unwrap().get(transaction_id).cloned()
    }

    pub fn get_multisig(&self, transaction_id: &str) -> Option<MultiSignatureSupport> {
        self.multi_sig.lock().unwrap().get(transaction_id).cloned()
    }

    pub fn get_events(&self, limit: usize) -> Vec<SignatureEvent> {
        let events = self.events.lock().unwrap();
        events.iter().rev().take(limit).cloned().collect()
    }

    pub fn get_revocations(&self) -> Vec<SignatureRevocation> {
        self.revocations.lock().unwrap().clone()
    }

    fn create_signature(&self, data: &[u8]) -> Vec<u8> {
        let mut sig = Vec::new();
        for (i, byte) in data.iter().enumerate() {
            sig.push(
                byte.wrapping_add(
                    (self
                        .scheme
                        .private_key
                        .get(i % self.scheme.private_key.len())
                        .copied()
                        .unwrap_or(0)) as u8,
                ),
            );
        }
        sig
    }

    fn verify_signature(&self, data: &[u8], signature: &[u8]) -> Result<bool, String> {
        let expected = self.create_signature(data);
        Ok(expected == signature)
    }

    fn record_event(&self, event_type: &str, transaction_id: &str, signer_id: Option<&str>, details: String) {
        let event = SignatureEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: event_type.to_string(),
            transaction_id: transaction_id.to_string(),
            signer_id: signer_id.map(|s| s.to_string()),
            details,
            timestamp: Utc::now(),
        };
        self.events.lock().unwrap().push(event);
    }
}

impl Default for TransactionSigningService {
    fn default() -> Self {
        let scheme = SignatureScheme {
            algorithm: "HMAC-SHA256".to_string(),
            public_key: vec![0x01, 0x02, 0x03, 0x04],
            private_key: vec![0x10, 0x20, 0x30, 0x40],
            created_at: Utc::now(),
        };
        Self::new(scheme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify() {
        let service = TransactionSigningService::default();
        let data = b"test transaction data";
        let request = service.sign("tx1", "signer1", data);
        let validation = service.verify(&request);
        assert!(validation.valid);
    }

    #[test]
    fn test_multisig_lifecycle() {
        let service = TransactionSigningService::default();
        let multi = service.create_multisig("tx1", 2);
        assert_eq!(multi.required_signatures, 2);
        let sig1 = service.sign("tx1", "signer1", b"data");
        let result = service.add_multisig_signature("tx1", sig1);
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.signatures.len(), 1);
    }

    #[test]
    fn test_revocation() {
        let service = TransactionSigningService::default();
        let _ = service.sign("tx1", "admin", b"data");
        let result = service.revoke("tx1", "admin", "compromised");
        assert!(result.is_ok());
        assert!(service.get_signature("tx1").is_none());
    }

    #[test]
    fn test_events_recorded() {
        let service = TransactionSigningService::default();
        let _ = service.sign("tx1", "s1", b"data");
        let events = service.get_events(10);
        assert!(!events.is_empty());
    }
}
