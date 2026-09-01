//! Integration tests for cryptographic operations
//! 
//! These tests verify that the encryption and verification services
//! work together correctly and detect tampering.

use proofflow_backend::crypto::{
    encrypt, decrypt, verify,
    generate_key, create_hmac, verify_hmac,
    EncryptedData,
};
use proofflow_backend::services::{
    EncryptionService,
    VerificationService,
};

#[test]
fn test_encryption_verification_consistency() {
    // Test that encryption and verification use the same primitives
    let key = generate_key();
    let data = b"consistency test";
    
    // Encrypt
    let encrypted = encrypt(data, &key).unwrap();
    
    // Verify using encryption service
    let enc_service = EncryptionService::new();
    let key_material = crate::crypto::KeyMaterial::new(key);
    let verified = enc_service.verify(&encrypted.ciphertext, &encrypted.hmac, &key_material).unwrap();
    assert!(verified);
    
    // Verify using verification service
    let ver_service = VerificationService::new();
    let verified2 = ver_service.verify_data(&encrypted.ciphertext, &encrypted.hmac, &key_material).unwrap();
    assert!(verified2);
}

#[test]
fn test_tamper_detection_integration() {
    let key = generate_key();
    let data = b"important data";
    let key_material = crate::crypto::KeyMaterial::new(key);
    
    // Create HMAC
    let hmac = create_hmac(data, &key).unwrap();
    
    // Tamper with data
    let tampered = b"tampered data";
    
    // Verification should fail
    let result = verify_hmac(tampered, &hmac, &key);
    assert!(result.is_err());
    
    // Tamper detection should detect
    let service = VerificationService::new();
    let is_tampered = service.detect_tamper(tampered, &hmac, &key_material).unwrap();
    assert!(is_tampered);
}

#[test]
fn test_key_derivation_consistency() {
    let service = VerificationService::new();
    let password = "my_password";
    let salt = service.generate_salt();
    
    let derived = service.derive_key(password, &salt, 32).unwrap();
    assert_eq!(derived.len(), 32);
    
    // Same password and salt should produce same key
    let derived2 = service.derive_key(password, &salt, 32).unwrap();
    assert_eq!(derived, derived2);
}

#[test]
fn test_password_hashing_flow() {
    let service = VerificationService::new();
    let password = "my_secret_password";
    
    let hash = service.hash_password(password).unwrap();
    let verified = service.verify_password(password, &hash).unwrap();
    assert!(verified);
    
    // Wrong password should fail
    let wrong_password = "wrong_password";
    let wrong_verified = service.verify_password(wrong_password, &hash).unwrap();
    assert!(!wrong_verified);
}

#[test]
fn test_encryption_decryption_flow() {
    let service = EncryptionService::new();
    let key = EncryptionService::generate_key();
    let data = b"test data";
    
    let encrypted = service.encrypt(data, &key).unwrap();
    let decrypted = service.decrypt(&encrypted, &key).unwrap();
    
    assert_eq!(decrypted, data);
}

#[test]
fn test_hmac_verification_flow() {
    let key = generate_key();
    let data = b"test data";
    let key_material = crate::crypto::KeyMaterial::new(key);
    
    let service = VerificationService::new();
    let hmac = service.create_hmac(data, &key_material).unwrap();
    
    // Valid HMAC should verify
    let result = service.verify_hmac(data, &hmac, &key_material);
    assert!(result.is_ok());
    
    // Invalid HMAC should fail
    let invalid_hmac = vec![0u8; 32];
    let result = service.verify_hmac(data, &invalid_hmac, &key_material);
    assert!(result.is_err());
}

#[test]
fn test_algorithm_consistency() {
    use proofflow_backend::crypto::types::EncryptionParams;
    
    let params = crate::crypto::primitives::default_params();
    assert_eq!(params.algorithm, "AES-256-GCM");
    assert_eq!(params.key_size, 32);
    assert_eq!(params.nonce_size, 12);
}
