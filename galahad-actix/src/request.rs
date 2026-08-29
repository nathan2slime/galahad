use serde::Deserialize;

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Deserialize)]
pub(crate) struct AuthRequest {
    pub(crate) email: String,
    pub(crate) password: String,
}
