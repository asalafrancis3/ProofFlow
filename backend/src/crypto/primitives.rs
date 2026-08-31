//! Core cryptographic primitives
//! 
//! This module contains the shared encryption, decryption, and verification
//! primitives used throughout the backend.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use rand::RngCore;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::errors::CryptoError;
use super::types::{EncryptionParams, KeyMaterial, EncryptedData};

// ============================================
// Constants
// ============================================

/// AES-256-GCM key size in bytes
pub const KEY_SIZE: usize = 32;

/// AES-256-GCM nonce size in bytes
pub const NONCE_SIZE: usize = 12;

/// HMAC-SHA256 tag size in bytes
pub const HMAC_SIZE: usize = 32;

/// Argon2 salt size in bytes
pub const SALT_SIZE: usize = 16;

/// Default encryption algorithm
pub const DEFAULT_ALGORITHM: &str = "AES-256-GCM";

/// Default hash algorithm
pub const DEFAULT_HASH_ALGORITHM: &str = "Argon2id";

// ============================================
// Encryption Functions
// ============================================

/// Generate a cryptographically secure random key
pub fn generate_key() -> [u8; KEY_SIZE] {
    let mut key = [0u8; KEY_SIZE];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

/// Generate a cryptographically secure random IV/nonce
pub fn generate_iv() -> [u8; NONCE_SIZE] {
    let mut iv = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut iv);
    iv
}

/// Generate a cryptographically secure random salt
pub fn generate_salt() -> [u8; SALT_SIZE] {
    let mut salt = [0u8; SALT_SIZE];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

/// Encrypt data using AES-256-GCM
pub fn encrypt(data: &[u8], key: &[u8; KEY_SIZE]) -> Result<EncryptedData, CryptoError> {
    if key.len() != KEY_SIZE {
        return Err(CryptoError::InvalidKeySize);
    }

    // Generate IV
    let iv = generate_iv();
    let nonce = Nonce::from_slice(&iv);

    // Create cipher
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| CryptoError::EncryptionFailed)?;

    // Encrypt
    let ciphertext = cipher.encrypt(nonce, data)
        .map_err(|_| CryptoError::EncryptionFailed)?;

    // Create HMAC for integrity
    let hmac = create_hmac(&ciphertext, key)?;

    Ok(EncryptedData {
        ciphertext,
        iv: iv.to_vec(),
        hmac,
        algorithm: DEFAULT_ALGORITHM.to_string(),
    })
}

/// Decrypt data using AES-256-GCM
pub fn decrypt(encrypted: &EncryptedData, key: &[u8; KEY_SIZE]) -> Result<Vec<u8>, CryptoError> {
    if key.len() != KEY_SIZE {
        return Err(CryptoError::InvalidKeySize);
    }

    // Verify HMAC first
    verify_hmac(&encrypted.ciphertext, &encrypted.hmac, key)?;

    // Create cipher
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| CryptoError::DecryptionFailed)?;

    // Decrypt
    let nonce = Nonce::from_slice(&encrypted.iv);
    cipher.decrypt(nonce, encrypted.ciphertext.as_ref())
        .map_err(|_| CryptoError::DecryptionFailed)
}

/// Verify encrypted data integrity using HMAC
pub fn verify(data: &[u8], hmac: &[u8], key: &[u8; KEY_SIZE]) -> Result<bool, CryptoError> {
    if key.len() != KEY_SIZE {
        return Err(CryptoError::InvalidKeySize);
    }

    verify_hmac(data, hmac, key)?;
    Ok(true)
}

/// Create HMAC-SHA256 tag for data
pub fn create_hmac(data: &[u8], key: &[u8]) -> Result<Vec<u8>, CryptoError> {
    type HmacSha256 = Hmac<Sha256>;
    
    let mut mac = <HmacSha256 as hmac::Mac>::new_from_slice(key)
        .map_err(|_| CryptoError::HmacCreationFailed)?;
    
    mac.update(data);
    Ok(mac.finalize().into_bytes().to_vec())
}

/// Verify HMAC-SHA256 tag for data
pub fn verify_hmac(data: &[u8], hmac: &[u8], key: &[u8]) -> Result<(), CryptoError> {
    type HmacSha256 = Hmac<Sha256>;
    
    let mut mac = <HmacSha256 as hmac::Mac>::new_from_slice(key)
        .map_err(|_| CryptoError::HmacVerificationFailed)?;
    
    mac.update(data);
    
    mac.verify_slice(hmac)
        .map_err(|_| CryptoError::IntegrityCheckFailed)
}

/// Sign data using HMAC (alias for create_hmac)
pub fn sign(data: &[u8], key: &[u8]) -> Result<Vec<u8>, CryptoError> {
    create_hmac(data, key)
}

/// Derive a key from a password using Argon2id
pub fn derive_key(
    password: &str,
    salt: &[u8; SALT_SIZE],
    output_len: usize,
) -> Result<Vec<u8>, CryptoError> {
    let salt_str = SaltString::encode_b64(salt)
        .map_err(|_| CryptoError::KeyDerivationFailed)?;
    
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt_str)
        .map_err(|_| CryptoError::KeyDerivationFailed)?;
    
    // Extract hash bytes (simplified - in production use proper key derivation)
    let hash_str = password_hash.to_string();
    let bytes = hash_str.as_bytes();
    
    // Take first output_len bytes
    let len = std::cmp::min(output_len, bytes.len());
    Ok(bytes[..len].to_vec())
}

/// Hash a password using Argon2id
pub fn hash_password(password: &str) -> Result<String, CryptoError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    argon2.hash_password(password.as_bytes(), &salt)
        .map_err(|_| CryptoError::PasswordHashFailed)
        .map(|hash| hash.to_string())
}

/// Verify a password against its hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, CryptoError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|_| CryptoError::PasswordVerificationFailed)?;
    
    let argon2 = Argon2::default();
    
    Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

/// Get default encryption parameters
pub fn default_params() -> EncryptionParams {
    EncryptionParams {
        algorithm: DEFAULT_ALGORITHM.to_string(),
        key_size: KEY_SIZE,
        nonce_size: NONCE_SIZE,
        iterations: 100_000,
        memory_cost: 1024 * 64, // 64 MB
        parallelism: 4,
    }
}

/// Create a new key material struct
pub fn key_material(key: [u8; KEY_SIZE]) -> KeyMaterial {
    KeyMaterial::new(key)
}

// ============================================
// Test Helpers
// ============================================

#[cfg(test)]
pub mod test_helpers {
    use super::*;

    /// Generate a test key
    pub fn test_key() -> [u8; KEY_SIZE] {
        generate_key()
    }

    /// Create test encrypted data
    pub fn test_encrypted_data() -> EncryptedData {
        let key = test_key();
        let data = b"test data";
        encrypt(data, &key).unwrap()
    }

    /// Verify a decryption result
    pub fn verify_decryption(result: Result<Vec<u8>, CryptoError>, expected: &[u8]) -> bool {
        match result {
            Ok(decrypted) => decrypted == expected,
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = generate_key();
        let data = b"Hello, World!";
        
        let encrypted = encrypt(data, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();
        
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_hmac_verification() {
        let key = generate_key();
        let data = b"test data";
        
        let hmac = create_hmac(data, &key).unwrap();
        let result = verify_hmac(data, &hmac, &key);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hmac_verification_failure() {
        let key = generate_key();
        let data = b"test data";
        let tampered = b"tampered data";
        
        let hmac = create_hmac(data, &key).unwrap();
        let result = verify_hmac(tampered, &hmac, &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_password_hashing() {
        let password = "my_secure_password";
        let hash = hash_password(password).unwrap();
        let verified = verify_password(password, &hash).unwrap();
        assert!(verified);
    }

    #[test]
    fn test_key_derivation() {
        let password = "my_password";
        let salt = generate_salt();
        let derived = derive_key(password, &salt, 32).unwrap();
        assert_eq!(derived.len(), 32);
    }
}
