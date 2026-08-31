use std::future::Future;
use std::sync::Arc;

pub use galahad_actix::SignUpContext as GalahadSignUpContext;

/// Sign-up configuration shared by high-level integrations.
#[derive(Clone, Default)]
pub struct GalahadSignUp {
    pub(crate) after_action: Option<Arc<dyn galahad_actix::AfterSignUp>>,
}

impl GalahadSignUp {
    /// Runs a hook after Galahad creates the auth user and password credential.
    pub fn after_action<F, Fut>(mut self, hook: F) -> Self
    where
        F: Fn(GalahadSignUpContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), galahad_core::AuthError>> + Send + 'static,
    {
        self.after_action = Some(Arc::new(hook));
        self
    }
}
