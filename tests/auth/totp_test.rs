//! Unit tests for TOTP module

use vantisweb::auth::totp::{TOTPSecret, HMACAlgorithm};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totp_secret_generation() {
        let secret = TOTPSecret::generate();
        assert!(!secret.secret.is_empty());
        assert_eq!(secret.digits, 6);
        assert_eq!(secret.time_step, 30);
    }

    #[test]
    fn test_totp_current_code() {
        let secret = TOTPSecret::generate();
        let code = secret.current_code();
        assert!(code.is_ok());
        assert_eq!(code.unwrap().len(), 6);
    }

    #[test]
    fn test_totp_verify_valid_code() {
        let secret = TOTPSecret::generate();
        let code = secret.current_code().unwrap();
        assert!(secret.verify(&code).unwrap());
    }

    #[test]
    fn test_totp_verify_invalid_code() {
        let secret = TOTPSecret::generate();
        assert!(!secret.verify("000000").unwrap());
    }

    #[test]
    fn test_totp_qr_uri() {
        let secret = TOTPSecret::generate();
        let uri = secret.to_qr_uri("user@example.com", "VantisWeb");
        assert!(uri.contains("otpauth://totp"));
        assert!(uri.contains("VantisWeb"));
        assert!(uri.contains("user@example.com"));
    }

    #[test]
    fn test_totp_algorithm_sha1() {
        let secret = TOTPSecret::generate();
        assert_eq!(secret.algorithm, HMACAlgorithm::SHA1);
    }

    #[test]
    fn test_totp_custom_digits() {
        let secret = TOTPSecret::with_options(HMACAlgorithm::SHA256, 8, 30);
        assert_eq!(secret.digits, 8);
        let code = secret.current_code().unwrap();
        assert_eq!(code.len(), 8);
    }
}