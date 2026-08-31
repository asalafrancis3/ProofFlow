//! Encryption verification service using shared cryptographic primitives
//! 
//! This service provides verification operations using the same cryptographic
//! primitives as the encryption service, ensuring consistency.

use crate::crypto::{
    verify as crypto_verify,
    create_hmac, verify_hmac,
    hash_password, verify_password,
    derive_key, generate_salt,
    EncryptedData, KeyMaterial, CryptoError,
};

const KEY_SIZE: usize = 32;

fn key_to_array(key: &KeyMaterial) -> Result<[u8; KEY_SIZE], CryptoError> {
    key.key.as_slice().try_into()
        .map_err(|_| CryptoError::InvalidKeySize)
}

/// Service for verifying encrypted data
pub struct VerificationService {
    // Configuration for verification
}

impl Default for VerificationService {
    fn default() -> Self {
        Self::new()
    }
}

impl VerificationService {
    /// Create a new verification service
    pub fn new() -> Self {
        Self {}
    }

    /// Verify encrypted data integrity
    pub fn verify_data(&self, data: &[u8], hmac: &[u8], key: &KeyMaterial) -> Result<bool, CryptoError> {
        let key_array = key_to_array(key)?;
        crypto_verify(data, hmac, &key_array)
    }

    /// Verify HMAC for data
    pub fn verify_hmac(&self, data: &[u8], hmac: &[u8], key: &KeyMaterial) -> Result<(), CryptoError> {
        let key_array = key_to_array(key)?;
        verify_hmac(data, hmac, &key_array)
    }

    /// Create HMAC for data (for verification purposes)
    pub fn create_hmac(&self, data: &[u8], key: &KeyMaterial) -> Result<Vec<u8>, CryptoError> {
        create_hmac(data, &key.key)
    }

    /// Hash a password
    pub fn hash_password(&self, password: &str) -> Result<String, CryptoError> {
        hash_password(password)
    }

    /// Verify a password against its hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, CryptoError> {
        verify_password(password, hash)
    }

    /// Derive a key from a password
    pub fn derive_key(&self, password: &str, salt: &[u8], output_len: usize) -> Result<Vec<u8>, CryptoError> {
        let salt_array: [u8; 16] = salt.try_into()
            .map_err(|_| CryptoError::InvalidKeySize)?;
        derive_key(password, &salt_array, output_len)
    }

    /// Generate a salt
    pub fn generate_salt() -> [u8; 16] {
        crate::crypto::primitives::generate_salt()
    }

    /// Tamper detection test
    pub fn detect_tamper(data: &[u8], hmac: &[u8], key: &KeyMaterial) -> Result<bool, CryptoError> {
        match Self::new().verify_hmac(data, hmac, key) {
            Ok(_) => Ok(false), // No tamper detected
            Err(CryptoError::IntegrityCheckFailed) => Ok(true), // Tamper detected
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::primitives::encrypt;

    #[test]
    fn test_verification_service_verify() {
        let service = VerificationService::new();
        let key = crate::crypto::primitives::generate_key();
        let key_material = KeyMaterial::new(key);
        let data = b"test data";
        
        let hmac = service.create_hmac(data, &key_material).unwrap();
        let result = service.verify_hmac(data, &hmac, &key_material);
        assert!(result.is_ok());
    }

    #[test]
    fn test_tamper_detection() {
        let service = VerificationService::new();
        let key = crate::crypto::primitives::generate_key();
        let key_material = KeyMaterial::new(key);
        let data = b"test data";
        
        let hmac = service.create_hmac(data, &key_material).unwrap();
        
        // Tamper with data
        let tampered_data = b"tampered data";
        let is_tampered = VerificationService::detect_tamper(tampered_data, &hmac, &key_material).unwrap();
        assert!(is_tampered);
    }

    #[test]
    fn test_password_hashing_verification() {
        let service = VerificationService::new();
        let password = "secure_password";
        
        let hash = service.hash_password(password).unwrap();
        let verified = service.verify_password(password, &hash).unwrap();
        assert!(verified);
    }
}
