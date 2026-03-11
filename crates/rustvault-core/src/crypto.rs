//! Cryptographic utilities — password hashing and JWT encode/decode.

use argon2::password_hash::{SaltString, rand_core::OsRng};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use sha2::{Digest, Sha256};

use crate::error::CoreError;
use crate::models::session::AccessTokenClaims;

// ── Password Hashing (Argon2id) ──────────────────────────────

/// Hash a plain-text password with Argon2id.
pub fn hash_password(password: &str) -> Result<String, CoreError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| CoreError::Internal(format!("password hashing failed: {e}")))
}

/// Verify a plain-text password against an Argon2id hash.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, CoreError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| CoreError::Internal(format!("invalid password hash: {e}")))?;

    match Argon2::default().verify_password(password.as_bytes(), &parsed) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(CoreError::Internal(format!(
            "password verification failed: {e}"
        ))),
    }
}

// ── JWT ──────────────────────────────────────────────────────

/// Encode an access token JWT.
pub fn encode_access_token(
    user_id: &str,
    role: &str,
    secret: &str,
    ttl_secs: u64,
) -> Result<String, CoreError> {
    let now = time::OffsetDateTime::now_utc().unix_timestamp();

    let claims = AccessTokenClaims {
        sub: user_id.to_string(),
        role: role.to_string(),
        iat: now,
        exp: now + ttl_secs as i64,
    };

    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| CoreError::Internal(format!("JWT encoding failed: {e}")))
}

/// Decode and validate an access token JWT.
///
/// Tries the primary secret first; if that fails and an old secret is provided,
/// retries with the old secret (graceful key rotation).
pub fn decode_access_token(
    token: &str,
    secret: &str,
    old_secret: Option<&str>,
) -> Result<AccessTokenClaims, CoreError> {
    let validation = Validation::default();

    // Try primary secret
    match jsonwebtoken::decode::<AccessTokenClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    ) {
        Ok(data) => Ok(data.claims),
        Err(e) => {
            // If there's an old secret, try that before failing
            if let Some(old) = old_secret {
                if let Ok(data) = jsonwebtoken::decode::<AccessTokenClaims>(
                    token,
                    &DecodingKey::from_secret(old.as_bytes()),
                    &validation,
                ) {
                    return Ok(data.claims);
                }
            }
            // Map specific JWT errors
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => Err(CoreError::TokenExpired),
                _ => Err(CoreError::TokenInvalid(e.to_string())),
            }
        }
    }
}

// ── Refresh Token Hashing ────────────────────────────────────

/// Generate a cryptographically random refresh token (hex-encoded).
pub fn generate_refresh_token() -> String {
    let mut bytes = [0u8; 32];
    rand::fill(&mut bytes);

    hex::encode(bytes)
}

/// SHA-256 hash a refresh token for storage.
pub fn hash_refresh_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());

    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_hash_and_verify() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn jwt_roundtrip() {
        let secret = "test-secret-key-that-is-long-enough-for-hmac";
        let token = encode_access_token("user-123", "admin", secret, 3600).unwrap();
        let claims = decode_access_token(&token, secret, None).unwrap();
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.role, "admin");
    }

    #[test]
    fn jwt_old_secret_rotation() {
        let old_secret = "old-secret-key-that-is-long-enough-for-hmac";
        let new_secret = "new-secret-key-that-is-long-enough-for-hmac";

        // Token signed with old secret
        let token = encode_access_token("user-456", "member", old_secret, 3600).unwrap();

        // Should fail with new secret alone
        assert!(decode_access_token(&token, new_secret, None).is_err());

        // Should succeed when old secret is provided as fallback
        let claims = decode_access_token(&token, new_secret, Some(old_secret)).unwrap();
        assert_eq!(claims.sub, "user-456");
    }

    #[test]
    fn refresh_token_hash_deterministic() {
        let token = "test-refresh-token";
        let hash1 = hash_refresh_token(token);
        let hash2 = hash_refresh_token(token);
        assert_eq!(hash1, hash2);
    }
}
