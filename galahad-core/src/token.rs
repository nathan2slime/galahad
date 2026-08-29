use std::fmt;

use rand_core::RngCore;
use sha2::{Digest, Sha256};

const SESSION_TOKEN_BYTES: usize = 32;

/// An opaque session token intended to be returned to a client once.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SessionToken(String);

impl SessionToken {
    /// Creates a session token from its string representation.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the string representation of this token.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for SessionToken {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for SessionToken {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for SessionToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A hashed session token suitable for persistence and lookup.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SessionTokenHash(String);

impl SessionTokenHash {
    /// Creates a session token hash from its string representation.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the string representation of this token hash.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for SessionTokenHash {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for SessionTokenHash {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for SessionTokenHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Generates opaque session tokens.
pub trait SessionTokenGenerator: Send + Sync {
    /// Generates a new session token.
    fn generate(&self) -> SessionToken;
}

/// Hashes session tokens for persistence and lookup.
pub trait SessionTokenHasher: Send + Sync {
    /// Hashes a session token.
    fn hash_token(&self, token: &SessionToken) -> SessionTokenHash;
}

/// Session token generator backed by the operating system CSPRNG.
pub struct OsSessionTokenGenerator;

impl Default for OsSessionTokenGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl OsSessionTokenGenerator {
    /// Creates an OS-backed session token generator.
    pub const fn new() -> Self {
        Self
    }
}

impl SessionTokenGenerator for OsSessionTokenGenerator {
    fn generate(&self) -> SessionToken {
        let mut bytes = [0_u8; SESSION_TOKEN_BYTES];
        rand_core::OsRng.fill_bytes(&mut bytes);

        SessionToken::new(hex_encode(&bytes))
    }
}

/// SHA-256 session token hasher.
pub struct Sha256SessionTokenHasher;

impl Default for Sha256SessionTokenHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl Sha256SessionTokenHasher {
    /// Creates a SHA-256 session token hasher.
    pub const fn new() -> Self {
        Self
    }
}

impl SessionTokenHasher for Sha256SessionTokenHasher {
    fn hash_token(&self, token: &SessionToken) -> SessionTokenHash {
        let digest = Sha256::digest(token.as_str().as_bytes());

        SessionTokenHash::new(hex_encode(&digest))
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{
        OsSessionTokenGenerator, SessionToken, SessionTokenGenerator, SessionTokenHash,
        SessionTokenHasher, Sha256SessionTokenHasher,
    };

    #[test]
    fn session_token_preserves_and_displays_value() {
        let token = SessionToken::from("token-value");

        assert_eq!(token.as_str(), "token-value");
        assert_eq!(token.to_string(), "token-value");
    }

    #[test]
    fn session_token_hash_preserves_and_displays_value() {
        let token_hash = SessionTokenHash::from("hash-value");

        assert_eq!(token_hash.as_str(), "hash-value");
        assert_eq!(token_hash.to_string(), "hash-value");
    }

    #[test]
    fn generator_is_object_safe_and_usable() {
        let generator: Box<dyn SessionTokenGenerator> = Box::new(OsSessionTokenGenerator::new());

        let token = generator.generate();

        assert_eq!(token.as_str().len(), 64);
    }

    #[test]
    fn hasher_is_object_safe_and_usable() {
        let hasher: Box<dyn SessionTokenHasher> = Box::new(Sha256SessionTokenHasher::new());

        let token_hash = hasher.hash_token(&SessionToken::from("token-value"));

        assert_eq!(token_hash.as_str().len(), 64);
    }

    #[test]
    fn os_generator_returns_lowercase_hex_token() {
        let generator = OsSessionTokenGenerator::new();

        let token = generator.generate();

        assert_eq!(token.as_str().len(), 64);
        assert!(token
            .as_str()
            .chars()
            .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase()));
    }

    #[test]
    fn os_generator_does_not_repeat_tokens_in_sanity_check() {
        let generator = OsSessionTokenGenerator::new();
        let mut tokens = HashSet::new();

        for _ in 0..32 {
            assert!(tokens.insert(generator.generate()));
        }
    }

    #[test]
    fn sha256_hasher_is_deterministic() {
        let hasher = Sha256SessionTokenHasher::new();
        let token = SessionToken::from("token-value");

        assert_eq!(hasher.hash_token(&token), hasher.hash_token(&token));
    }

    #[test]
    fn sha256_hasher_returns_lowercase_hex_hash() {
        let hasher = Sha256SessionTokenHasher::new();

        let token_hash = hasher.hash_token(&SessionToken::from("token-value"));

        assert_eq!(token_hash.as_str().len(), 64);
        assert!(token_hash
            .as_str()
            .chars()
            .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase()));
    }

    #[test]
    fn sha256_hasher_does_not_return_plaintext_token() {
        let hasher = Sha256SessionTokenHasher::new();
        let token = SessionToken::from("token-value");

        let token_hash = hasher.hash_token(&token);

        assert_ne!(token_hash.as_str(), token.as_str());
        assert!(!token_hash.as_str().contains(token.as_str()));
    }

    #[test]
    fn sha256_hasher_distinguishes_different_tokens() {
        let hasher = Sha256SessionTokenHasher::new();

        let first_hash = hasher.hash_token(&SessionToken::from("first-token"));
        let second_hash = hasher.hash_token(&SessionToken::from("second-token"));

        assert_ne!(first_hash, second_hash);
    }
}
