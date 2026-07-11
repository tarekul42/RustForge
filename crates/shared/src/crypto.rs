use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

/// Default argon2 parameters aligned with OWASP 2023 recommendations.
const ARGON2_MEMORY: u32 = 19_456; // 19 MiB
const ARGON2_ITERATIONS: u32 = 2;
const ARGON2_PARALLELISM: u32 = 1;

/// Hash a password using argon2id with a random salt.
///
/// Returns the PHC-formatted encoded string.
///
/// This function performs CPU-intensive work and should be called
/// via `tokio::task::spawn_blocking` to avoid blocking the async runtime.
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let params = Params::new(ARGON2_MEMORY, ARGON2_ITERATIONS, ARGON2_PARALLELISM, None)?;

    let argon2 = Argon2::new(Algorithm::Argon2id, argon2::Version::V0x13, params);
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;

    Ok(hash.to_string())
}

/// Verify a password against a PHC-formatted argon2id hash string.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;

    let argon2 = Argon2::new(
        Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::default(),
    );
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Hash a session/OTP token using SHA-256 and return the hex-encoded digest.
///
/// Tokens are hashed before storage so the original value is never persisted.
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Generate cryptographically-secure random bytes.
pub fn generate_random_bytes(count: usize) -> Vec<u8> {
    use rand::RngCore;
    let mut buf = vec![0u8; count];
    OsRng.fill_bytes(&mut buf);
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify_round_trip() {
        let password = "correct-horse-battery-staple";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
    }

    #[test]
    fn wrong_password_fails() {
        let hash = hash_password("real-password").unwrap();
        assert!(!verify_password("wrong-password", &hash).unwrap());
    }

    #[test]
    fn hash_token_is_deterministic() {
        let token = "my-session-token";
        let a = hash_token(token);
        let b = hash_token(token);
        assert_eq!(a, b);
    }

    #[test]
    fn hash_token_differs_for_different_inputs() {
        let a = hash_token("token-a");
        let b = hash_token("token-b");
        assert_ne!(a, b);
    }

    #[test]
    fn random_bytes_has_correct_length() {
        let bytes = generate_random_bytes(32);
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn random_bytes_are_non_deterministic() {
        let a = generate_random_bytes(16);
        let b = generate_random_bytes(16);
        assert_ne!(a, b);
    }
}
