//! Unit tests for security modules

#[cfg(test)]
mod tests {
    #[test]
    fn test_crypto_engine_encrypt_decrypt() {
        // Test encryption and decryption
        let crypto = vanisweb::security::crypto::CryptoEngine::new().unwrap();
        
        let plaintext = b"Hello, VantisWeb!";
        let encrypted = crypto.encrypt(plaintext).unwrap();
        let decrypted = crypto.decrypt(&encrypted).unwrap();
        
        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_crypto_engine_hash() {
        // Test hash generation
        let crypto = vanisweb::security::crypto::CryptoEngine::new().unwrap();
        
        let data = b"Hello, VantisWeb!";
        let hash1 = crypto.hash(data).unwrap();
        let hash2 = crypto.hash(data).unwrap();
        
        assert_eq!(hash1, hash2);
        assert!(!hash1.is_empty());
    }

    #[test]
    fn test_digital_immune_system_scan() {
        // Test threat scanning
        let immune_system = vanisweb::security::immune_system::DigitalImmuneSystem::new().unwrap();
        
        // In production: This would test actual scanning
        let threats = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(immune_system.scan("/tmp/test"))
            .unwrap();
        
        assert_eq!(threats, 0); // No threats in test
    }

    #[test]
    fn test_sandbox_isolate() {
        // Test process isolation
        let mut sandbox = vanisweb::security::sandbox::Sandbox::new().unwrap();
        
        sandbox.initialize().unwrap();
        sandbox.isolate("test_process_123".to_string()).unwrap();
        
        assert!(sandbox.is_active());
        assert_eq!(sandbox.get_isolated_processes().len(), 1);
    }
}