use crate::{AuthError, ServiceResult};

/// Validates an email address using the domain's intentionally small rule set.
pub(crate) fn validate_email(email: &str) -> ServiceResult<()> {
    if email.is_empty() || email.chars().any(|character| character.is_whitespace()) {
        return Err(AuthError::InvalidEmail);
    }

    if email.chars().any(char::is_control) {
        return Err(AuthError::InvalidEmail);
    }

    let mut parts = email.split('@');
    let Some(local_part) = parts.next() else {
        return Err(AuthError::InvalidEmail);
    };
    let Some(domain_part) = parts.next() else {
        return Err(AuthError::InvalidEmail);
    };

    if parts.next().is_some() || local_part.is_empty() || domain_part.is_empty() {
        return Err(AuthError::InvalidEmail);
    }

    let mut domain_labels = domain_part.split('.');
    if domain_labels.clone().count() < 2
        || domain_labels.clone().any(str::is_empty)
        || domain_labels.any(|label| label.starts_with('-') || label.ends_with('-'))
    {
        return Err(AuthError::InvalidEmail);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_email;

    #[test]
    fn accepts_basic_email_addresses() {
        assert_eq!(validate_email("user@example.com"), Ok(()));
        assert_eq!(validate_email("first.last+tag@example.co.uk"), Ok(()));
    }

    #[test]
    fn rejects_email_without_single_at_sign() {
        assert_eq!(
            validate_email("user.example.com"),
            Err(crate::AuthError::InvalidEmail)
        );
        assert_eq!(
            validate_email("user@@example.com"),
            Err(crate::AuthError::InvalidEmail)
        );
    }

    #[test]
    fn rejects_empty_local_or_domain_part() {
        assert_eq!(
            validate_email("@example.com"),
            Err(crate::AuthError::InvalidEmail)
        );
        assert_eq!(validate_email("user@"), Err(crate::AuthError::InvalidEmail));
    }

    #[test]
    fn rejects_invalid_domain_labels() {
        assert_eq!(
            validate_email("user@example"),
            Err(crate::AuthError::InvalidEmail)
        );
        assert_eq!(
            validate_email("user@example..com"),
            Err(crate::AuthError::InvalidEmail)
        );
        assert_eq!(
            validate_email("user@-example.com"),
            Err(crate::AuthError::InvalidEmail)
        );
        assert_eq!(
            validate_email("user@example-.com"),
            Err(crate::AuthError::InvalidEmail)
        );
    }

    #[test]
    fn rejects_whitespace_and_control_characters() {
        assert_eq!(
            validate_email("user name@example.com"),
            Err(crate::AuthError::InvalidEmail)
        );
        assert_eq!(
            validate_email("user@example.com\n"),
            Err(crate::AuthError::InvalidEmail)
        );
    }
}
