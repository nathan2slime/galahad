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
    pub(crate) fn into_parts(self) -> (String, String, BTreeMap<String, Value>) {
        (self.email, self.password, self.fields)
    }
}

#[cfg(test)]
mod tests {
    use super::AuthRequest;

    #[test]
    fn keeps_extra_fields_for_application_hooks() {
        let request: AuthRequest = serde_json::from_str(
            r#"{"email":"user@example.com","password":"correct horse","name":"Ada"}"#,
        )
        .unwrap();

        assert_eq!(
            request.fields.get("name").and_then(|value| value.as_str()),
            Some("Ada")
        );
    }
}
