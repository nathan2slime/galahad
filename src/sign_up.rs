/// Sign-up configuration shared by high-level integrations.
#[derive(Clone, Debug, Default)]
pub struct GalahadSignUp {
    pub(crate) required_fields: Vec<String>,
}

impl GalahadSignUp {
    /// Requires an additional non-empty field in sign-up requests.
    pub fn required_field(mut self, name: impl Into<String>) -> Self {
        self.required_fields.push(name.into());
        self
    }

    /// Requires additional non-empty fields in sign-up requests.
    pub fn required_fields<I, S>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.required_fields
            .extend(fields.into_iter().map(Into::into));
        self
    }
}
