use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Deserialize)]
pub(crate) struct AuthRequest {
    pub(crate) email: String,
    pub(crate) password: String,
    #[serde(flatten)]
    pub(crate) fields: BTreeMap<String, Value>,
}

impl AuthRequest {
    pub(crate) fn has_non_empty_field(&self, name: &str) -> bool {
        self.fields.get(name).is_some_and(is_non_empty_field)
    }
}

fn is_non_empty_field(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::String(value) => !value.trim().is_empty(),
        Value::Array(value) => !value.is_empty(),
        Value::Object(value) => !value.is_empty(),
        Value::Bool(_) | Value::Number(_) => true,
    }
}

#[cfg(test)]
mod tests {
    use super::AuthRequest;

    #[test]
    fn detects_non_empty_extra_field() {
        let request: AuthRequest = serde_json::from_str(
            r#"{"email":"user@example.com","password":"correct horse","name":"Ada"}"#,
        )
        .unwrap();

        assert!(request.has_non_empty_field("name"));
    }

    #[test]
    fn rejects_missing_or_empty_extra_field() {
        let request: AuthRequest = serde_json::from_str(
            r#"{"email":"user@example.com","password":"correct horse","name":"   "}"#,
        )
        .unwrap();

        assert!(!request.has_non_empty_field("name"));
        assert!(!request.has_non_empty_field("company"));
    }
}
