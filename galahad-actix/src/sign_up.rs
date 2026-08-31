use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use galahad_core::{AuthError, User};
use serde_json::Value;

/// Context passed to sign-up hooks after the auth user is created.
pub struct SignUpContext {
    pub user: User,
    pub fields: BTreeMap<String, Value>,
}

/// A boxed future returned by sign-up hooks.
pub type AfterSignUpFuture = Pin<Box<dyn Future<Output = Result<(), AuthError>> + Send>>;

/// Hook executed after a successful sign-up.
pub trait AfterSignUp: Send + Sync {
    fn after_sign_up(&self, context: SignUpContext) -> AfterSignUpFuture;
}

impl<F, Fut> AfterSignUp for F
where
    F: Fn(SignUpContext) -> Fut + Send + Sync,
    Fut: Future<Output = Result<(), AuthError>> + Send + 'static,
{
    fn after_sign_up(&self, context: SignUpContext) -> AfterSignUpFuture {
        Box::pin(self(context))
    }
}
