use galahad_actix::GalahadActixOpenApi;

/// Entry point for Galahad's OpenAPI integration API.
pub struct GalahadOpenApi;

impl GalahadOpenApi {
    /// Starts injecting Galahad's Actix endpoint documentation into an OpenAPI document.
    pub fn actix(openapi: utoipa::openapi::OpenApi) -> GalahadActixOpenApiBuilder {
        GalahadActixOpenApiBuilder {
            openapi,
            session_cookie_name: None,
        }
    }
}

/// Builder for injecting Galahad Actix documentation into an OpenAPI document.
pub struct GalahadActixOpenApiBuilder {
    openapi: utoipa::openapi::OpenApi,
    session_cookie_name: Option<String>,
}

impl GalahadActixOpenApiBuilder {
    /// Sets the session cookie name documented by the OpenAPI security scheme.
    pub fn session_cookie_name(mut self, name: impl Into<String>) -> Self {
        self.session_cookie_name = Some(name.into());
        self
    }

    /// Injects Galahad's Actix endpoint documentation and returns the OpenAPI document.
    pub fn build(mut self) -> utoipa::openapi::OpenApi {
        self.openapi
            .merge(GalahadActixOpenApi::openapi_for_session_cookie_name(
                self.session_cookie_name,
            ));
        self.openapi
    }
}

#[cfg(test)]
mod tests {
    use super::GalahadOpenApi;
    use utoipa::OpenApi;

    #[derive(OpenApi)]
    #[openapi(info(title = "Application API", version = "1.0.0"))]
    struct ApplicationOpenApi;

    #[test]
    fn openapi_builder_injects_galahad_actix_paths() {
        let openapi = GalahadOpenApi::actix(ApplicationOpenApi::openapi()).build();
        let json = openapi.to_json().unwrap();

        assert!(json.contains("Application API"));
        assert!(json.contains("/auth/sign-up"));
        assert!(json.contains("/auth/sign-in"));
        assert!(json.contains("/auth/sign-out"));
        assert!(json.contains("/auth/session"));
    }

    #[test]
    fn openapi_builder_documents_custom_session_cookie_name() {
        let openapi = GalahadOpenApi::actix(ApplicationOpenApi::openapi())
            .session_cookie_name("app_session")
            .build();
        let json = openapi.to_json().unwrap();

        assert!(json.contains("app_session"));
    }
}
