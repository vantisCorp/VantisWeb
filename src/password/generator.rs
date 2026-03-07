//! Password Generator for VantisWeb Password Manager
//! 
//! This module provides:
//! - Secure random password generation
//! - Password strength analysis
//! - Customizable generation options

use rand::Rng;
use super::{PasswordGeneratorOptions, PasswordStrength, StrengthLevel};

/// Password generator
pub struct PasswordGenerator {
    /// Uppercase letters
    uppercase: Vec<char>,
    /// Lowercase letters
    lowercase: Vec<char>,
    /// Numbers
    numbers: Vec<char>,
    /// Symbols
    symbols: Vec<char>,
    /// Similar characters to exclude
    similar_chars: Vec<char>,
}

impl PasswordGenerator {
    /// Create a new password generator
    pub fn new() -> Self {
        Self {
            uppercase: "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect(),
            lowercase: "abcdefghijklmnopqrstuvwxyz".chars().collect(),
            numbers: "0123456789".chars().collect(),
            symbols: "!@#$%^&*()_+-=[]{}|;:,.<>?".chars().collect(),
            similar_chars: "il1Lo0O".chars().collect(),
        }
    }

    /// Generate a new password
    pub async fn generate(&self, length: u8, options: PasswordGeneratorOptions) -> Result<String, String> {
        if length == 0 {
            return Err("Password length cannot be zero".to_string());
        }

        let mut charset = Vec::new();
        let mut required = Vec::new();

        if options.uppercase {
            charset.extend(self.uppercase.iter());
            required.push(self.uppercase[0]);
        }

        if options.lowercase {
            charset.extend(self.lowercase.iter());
            required.push(self.lowercase[0]);
        }

        if options.numbers {
            charset.extend(self.numbers.iter());
            required.push(self.numbers[0]);
        }

        if options.symbols {
            charset.extend(self.symbols.iter());
            required.push(self.symbols[0]);
        }

        if charset.is_empty() {
            return Err("At least one character type must be selected".to_string());
        }

        // Filter out similar characters if requested
        if options.exclude_similar {
            charset.retain(|c| !self.similar_chars.contains(c));
            required.retain(|c| !self.similar_chars.contains(c));
        }

        let mut password = String::new();
        let mut rng = rand::thread_rng();

        // Ensure at least one character from each selected type
        for &c in &required {
            let pos = rng.gen_range(0..charset.len());
            password.push(charset[pos]);
        }

        // Fill the rest randomly
        while password.len() < length as usize {
            let pos = rng.gen_range(0..charset.len());
            password.push(charset[pos]);
        }

        // Shuffle the password
        let mut chars: Vec<char> = password.chars().collect();
        for i in (1..chars.len()).rev() {
            let j = rng.gen_range(0..=i);
            chars.swap(i, j);
        }

        Ok(chars.into_iter().collect())
    }

    /// Check password strength
    pub fn check_strength(&self, password: &str) -> PasswordStrength {
        let length = password.len();
        let mut score = 0;
        let mut suggestions = Vec::new();

        // Length scoring
        if length >= 8 {
            score += 20;
        }
        if length >= 12 {
            score += 20;
        }
        if length >= 16 {
            score += 10;
        } else {
            suggestions.push("Use a longer password (at least 12 characters)".to_string());
        }

        // Character variety
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_number = password.chars().any(|c| c.is_numeric());
        let has_symbol = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));

        if has_uppercase {
            score += 15;
        } else {
            suggestions.push("Add uppercase letters".to_string());
        }

        if has_lowercase {
            score += 15;
        } else {
            suggestions.push("Add lowercase letters".to_string());
        }

        if has_number {
            score += 15;
        } else {
            suggestions.push("Add numbers".to_string());
        }

        if has_symbol {
            score += 15;
        } else {
            suggestions.push("Add symbols".to_string());
        }

        // Check for common patterns
        if self.is_common_password(password) {
            score = 0;
            suggestions.push("Avoid using common passwords".to_string());
        }

        if self.has_repeating_chars(password) {
            score -= 10;
            suggestions.push("Avoid repeating characters".to_string());
        }

        if self.has_sequential_chars(password) {
            score -= 10;
            suggestions.push("Avoid sequential characters".to_string());
        }

        // Clamp score
        score = score.clamp(0, 100);

        let level = if score >= 90 {
            StrengthLevel::VeryStrong
        } else if score >= 70 {
            StrengthLevel::Strong
        } else if score >= 50 {
            StrengthLevel::Fair
        } else if score >= 30 {
            StrengthLevel::Weak
        } else {
            StrengthLevel::VeryWeak
        };

        let crack_time = self.estimate_crack_time(password);

        PasswordStrength {
            score,
            level,
            suggestions,
            crack_time,
        }
    }

    /// Check if password is common
    fn is_common_password(&self, password: &str) -> bool {
        let common = vec![
            "password", "123456", "12345678", "1234", "qwerty",
            "12345", "dragon", "pussy", "baseball", "football",
            "letmein", "monkey", "696969", "abc123", "mustang",
            "michael", "shadow", "master", "jennifer", "111111",
            "2000", "jordan", "superman", "harley", "1234567",
        ];

        let lower = password.to_lowercase();
        common.iter().any(|&c| lower == c)
    }

    /// Check for repeating characters
    fn has_repeating_chars(&self, password: &str) -> bool {
        let chars: Vec<char> = password.chars().collect();
        
        for i in 2..chars.len() {
            if chars[i] == chars[i - 1] && chars[i] == chars[i - 2] {
                return true;
            }
        }
        
        false
    }

    /// Check for sequential characters
    fn has_sequential_chars(&self, password: &str) -> bool {
        let chars: Vec<char> = password.chars().collect();
        
        // Check for sequential numbers
        for i in 2..chars.len() {
            if chars[i].is_numeric() && chars[i - 1].is_numeric() && chars[i - 2].is_numeric() {
                let a = chars[i].to_digit(10).unwrap();
                let b = chars[i - 1].to_digit(10).unwrap();
                let c = chars[i - 2].to_digit(10).unwrap();
                
                if a == b + 1 && b == c + 1 {
                    return true;
                }
                if a == b - 1 && b == c - 1 {
                    return true;
                }
            }
        }
        
        false
    }

    /// Estimate time to crack password
    fn estimate_crack_time(&self, password: &str) -> String {
        let charset_size = 26 + 26 + 10 + 30; // lowercase + uppercase + numbers + symbols
        let combinations = charset_size.pow(password.len() as u32);
        let attempts_per_second = 10_000_000_000; // 10 billion attempts per second
        
        let seconds = combinations as f64 / attempts_per_second as f64;
        
        if seconds < 60.0 {
            format!("Instantly")
        } else if seconds < 3600.0 {
            format!("{} minutes", (seconds / 60.0).ceil())
        } else if seconds < 86400.0 {
            format!("{} hours", (seconds / 3600.0).ceil())
        } else if seconds < 2592000.0 {
            format!("{} days", (seconds / 86400.0).ceil())
        } else if seconds < 31536000.0 {
            format!("{} months", (seconds / 2592000.0).ceil())
        } else if seconds < 315360000.0 {
            format!("{} years", (seconds / 31536000.0).ceil())
        } else if seconds < 3153600000.0 {
            format!("{} centuries", (seconds / 315360000.0).ceil())
        } else {
            format!("Centuries")
        }
    }

    /// Generate passphrase from words
    pub async fn generate_passphrase(&self, word_count: u8) -> Result<String, String> {
        if word_count == 0 {
            return Err("Word count cannot be zero".to_string());
        }

        let words = vec![
            "correct", "horse", "battery", "staple", "apple", "banana", "cherry",
            "dragon", "elephant", "forest", "garden", "house", "island", "jungle",
            "kingdom", "lemon", "mountain", "ocean", "planet", "queen", "river",
            "sunshine", "tiger", "umbrella", "village", "waterfall", "yellow",
            "zebra", "adventure", "butterfly", "crystal", "diamond", "emerald",
            "firefly", "galaxy", "harmony", "insight", "journey", "kangaroo",
            "lighthouse", "mystic", "nebula", "octopus", "phoenix", "quartz",
            "rainbow", "starlight", "triumph", "universe", "victory", "wonder",
            "xylophone", "yesterday", "zenith",
        ];

        let mut rng = rand::thread_rng();
        let mut passphrase = Vec::new();

        for _ in 0..word_count {
            let pos = rng.gen_range(0..words.len());
            passphrase.push(words[pos]);
        }

        Ok(passphrase.join("-"))
    }

    /// Generate PIN
    pub async fn generate_pin(&self, length: u8) -> Result<String, String> {
        if length == 0 || length > 16 {
            return Err("PIN length must be between 1 and 16".to_string());
        }

        let mut rng = rand::thread_rng();
        let pin: String = (0..length)
            .map(|_| rng.gen_range(0..10).to_string())
            .collect();

        Ok(pin)
    }
}

impl Default for PasswordGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_generator() {
        let generator = PasswordGenerator::new();
        assert_eq!(generator.uppercase.len(), 26);
        assert_eq!(generator.lowercase.len(), 26);
        assert_eq!(generator.numbers.len(), 10);
    }

    #[tokio::test]
    async fn test_generate_password() {
        let generator = PasswordGenerator::new();
        let options = PasswordGeneratorOptions::default();
        let password = generator.generate(16, options).await.unwrap();
        
        assert_eq!(password.len(), 16);
    }

    #[tokio::test]
    async fn test_generate_password_custom() {
        let generator = PasswordGenerator::new();
        let options = PasswordGeneratorOptions {
            uppercase: true,
            lowercase: true,
            numbers: true,
            symbols: false,
            exclude_similar: true,
        };
        let password = generator.generate(12, options).await.unwrap();
        
        assert_eq!(password.len(), 12);
        assert!(!password.contains('!'));
        assert!(!password.contains('@'));
    }

    #[test]
    fn test_check_strength_weak() {
        let generator = PasswordGenerator::new();
        let result = generator.check_strength("password");
        
        assert!(result.score < 50);
        assert_eq!(result.level, StrengthLevel::Weak);
    }

    #[test]
    fn test_check_strength_strong() {
        let generator = PasswordGenerator::new();
        let result = generator.check_strength("P@ssw0rd!123456");
        
        assert!(result.score > 80);
        assert_eq!(result.level, StrengthLevel::VeryStrong);
    }

    #[test]
    fn test_common_password() {
        let generator = PasswordGenerator::new();
        assert!(generator.is_common_password("password"));
        assert!(generator.is_common_password("123456"));
        assert!(!generator.is_common_password("xK9#mP2$vL8!"));
    }

    #[test]
    fn test_repeating_chars() {
        let generator = PasswordGenerator::new();
        assert!(generator.has_repeating_chars("aaa"));
        assert!(!generator.has_repeating_chars("abc"));
    }

    #[test]
    fn test_sequential_chars() {
        let generator = PasswordGenerator::new();
        assert!(generator.has_sequential_chars("123"));
        assert!(!generator.has_sequential_chars("135"));
    }

    #[tokio::test]
    async fn test_generate_passphrase() {
        let generator = PasswordGenerator::new();
        let passphrase = generator.generate_passphrase(4).await.unwrap();
        
        let words: Vec<&str> = passphrase.split('-').collect();
        assert_eq!(words.len(), 4);
    }

    #[tokio::test]
    async fn test_generate_pin() {
        let generator = PasswordGenerator::new();
        let pin = generator.generate_pin(6).await.unwrap();
        
        assert_eq!(pin.len(), 6);
        assert!(pin.chars().all(|c| c.is_numeric()));
    }
}