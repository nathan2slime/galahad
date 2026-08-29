use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct AuthRequest {
    pub(crate) email: String,
    pub(crate) password: String,
}
