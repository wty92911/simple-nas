// Utility modules - will be implemented in Task 1.3 (Security Infrastructure)
// pub mod crypto;       // Cryptographic utilities
// pub mod validation;   // Input validation utilities

use argon2::password_hash::{SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

use anyhow::Result;

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Password hashing failed: {}", e))?
        .to_string();
    Ok(password_hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| anyhow::anyhow!("Invalid password hash format: {}", e))?;
    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_success() {
        let password = "test_password_123";
        let result = hash_password(password);

        assert!(result.is_ok());
        let hash = result.unwrap();
        assert!(!hash.is_empty());
        // Argon2 hash should start with $argon2id$
        assert!(hash.starts_with("$argon2id$"));
    }

    #[test]
    fn test_hash_password_empty_string() {
        let password = "";
        let result = hash_password(password);

        assert!(result.is_ok());
        let hash = result.unwrap();
        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2id$"));
    }

    #[test]
    fn test_hash_password_long_string() {
        let password = "a".repeat(1000);
        let result = hash_password(&password);

        assert!(result.is_ok());
        let hash = result.unwrap();
        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2id$"));
    }

    #[test]
    fn test_hash_password_produces_different_hashes() {
        let password = "same_password";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();

        // Same password should produce different hashes due to different salts
        assert_ne!(hash1, hash2);
        assert!(hash1.starts_with("$argon2id$"));
        assert!(hash2.starts_with("$argon2id$"));
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "correct_password_123";
        let hash = hash_password(password).unwrap();

        let result = verify_password(password, &hash);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_password_incorrect() {
        let password = "correct_password";
        let wrong_password = "wrong_password";
        let hash = hash_password(password).unwrap();

        let result = verify_password(wrong_password, &hash);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_verify_password_empty_password() {
        let password = "";
        let hash = hash_password(password).unwrap();

        let result = verify_password(password, &hash);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_password_invalid_hash_format() {
        let password = "test_password";
        let invalid_hash = "invalid_hash_format";

        let result = verify_password(password, invalid_hash);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid password hash format")
        );
    }

    #[test]
    fn test_verify_password_empty_hash() {
        let password = "test_password";
        let empty_hash = "";

        let result = verify_password(password, empty_hash);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid password hash format")
        );
    }

    #[test]
    fn test_verify_password_malformed_argon2_hash() {
        let password = "test_password";
        let malformed_hash = "$argon2id$v=19$m=65536,t=2,p=1$"; // Missing salt and hash parts

        let result = verify_password(password, malformed_hash);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid password hash format")
        );
    }

    #[test]
    fn test_round_trip_password_verification() {
        let long_password = "very_long_password_".repeat(10);
        let test_cases = vec![
            "simple",
            "complex_password_with_123!@#",
            "unicode_测试_🔐",
            long_password.as_str(),
            "",
            "sp3ci@l!ch@r@ct3rs#$%^&*()",
        ];

        for password in test_cases {
            let hash = hash_password(password).unwrap();
            let verification = verify_password(password, &hash).unwrap();
            assert!(
                verification,
                "Round-trip verification failed for password: {password}"
            );

            // Also test that a wrong password fails
            let wrong_password = format!("{password}_wrong");
            let wrong_verification = verify_password(&wrong_password, &hash).unwrap();
            assert!(
                !wrong_verification,
                "Wrong password incorrectly verified for: {password}"
            );
        }
    }

    #[test]
    fn test_password_case_sensitivity() {
        let password = "CaseSensitive";
        let hash = hash_password(password).unwrap();

        // Correct case should verify
        assert!(verify_password(password, &hash).unwrap());

        // Different case should not verify
        assert!(!verify_password("casesensitive", &hash).unwrap());
        assert!(!verify_password("CASESENSITIVE", &hash).unwrap());
    }

    #[test]
    fn test_hash_format_consistency() {
        let password = "test_password";
        let hash = hash_password(password).unwrap();

        // Argon2id hash should have the expected format
        let parts: Vec<&str> = hash.split('$').collect();
        assert_eq!(parts.len(), 6); // ["", "argon2id", "v=19", "m=...,t=...,p=...", "salt", "hash"]
        assert_eq!(parts[1], "argon2id");
        assert_eq!(parts[2], "v=19"); // Argon2 version 19
        assert!(parts[3].contains("m=")); // Memory parameter
        assert!(parts[3].contains("t=")); // Time parameter
        assert!(parts[3].contains("p=")); // Parallelism parameter
        assert!(!parts[4].is_empty()); // Salt should not be empty
        assert!(!parts[5].is_empty()); // Hash should not be empty
    }
}
