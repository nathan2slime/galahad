use crate::UserId;

/// A password credential stored for a user.
///
/// The value is expected to be a password hash. Hashing and validation are
/// intentionally handled outside the core domain model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PasswordCredential {
    pub user_id: UserId,
    pub password_hash: String,
}

impl PasswordCredential {
    /// Creates a password credential from an existing password hash.
    pub fn new(user_id: UserId, password_hash: impl Into<String>) -> Self {
        Self {
            user_id,
            password_hash: password_hash.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PasswordCredential;
    use crate::UserId;

    #[test]
    fn constructor_sets_user_and_hash() {
        let credential = PasswordCredential::new(UserId::from("user-1"), "hash");

        assert_eq!(credential.user_id, UserId::from("user-1"));
        assert_eq!(credential.password_hash, "hash");
    }
}
