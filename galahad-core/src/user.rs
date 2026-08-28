use std::fmt;

/// The identifier of a user.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct UserId(String);

impl UserId {
    /// Creates a user identifier from its string representation.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the string representation of this identifier.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for UserId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for UserId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A user known to the authentication domain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct User {
    pub id: UserId,
    pub email: String,
}

impl User {
    /// Creates a user with the provided identifier and email address.
    pub fn new(id: UserId, email: impl Into<String>) -> Self {
        Self {
            id,
            email: email.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{User, UserId};

    #[test]
    fn user_id_preserves_and_displays_value() {
        let from_string = UserId::from(String::from("user-1"));
        let from_str = UserId::from("user-1");

        assert_eq!(from_string, from_str);
        assert_eq!(from_str.as_str(), "user-1");
        assert_eq!(from_str.to_string(), "user-1");
    }

    #[test]
    fn user_constructor_sets_fields() {
        let user = User::new(UserId::from("user-1"), "user@example.com");

        assert_eq!(user.id.as_str(), "user-1");
        assert_eq!(user.email, "user@example.com");
    }
}
