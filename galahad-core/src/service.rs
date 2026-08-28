use std::future::Future;
use std::pin::Pin;

use crate::AuthError;

/// A boxed, sendable future returned by a service operation.
pub type BoxServiceFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// The result type shared by service operations.
pub type ServiceResult<T> = Result<T, AuthError>;

/// Password hashing and verification operations.
pub trait PasswordService: Send + Sync {
    /// Hashes a plaintext password.
    fn hash_password<'a>(
        &'a self,
        password: &'a str,
    ) -> BoxServiceFuture<'a, ServiceResult<String>>;

    /// Verifies a plaintext password against a password hash.
    fn verify_password<'a>(
        &'a self,
        password: &'a str,
        password_hash: &'a str,
    ) -> BoxServiceFuture<'a, ServiceResult<bool>>;
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::task::{Context, Poll, Waker};

    use super::{BoxServiceFuture, PasswordService, ServiceResult};

    struct FakePasswordService;

    impl PasswordService for FakePasswordService {
        fn hash_password<'a>(
            &'a self,
            password: &'a str,
        ) -> BoxServiceFuture<'a, ServiceResult<String>> {
            Box::pin(std::future::ready(Ok(format!("test-hash:{password}"))))
        }

        fn verify_password<'a>(
            &'a self,
            password: &'a str,
            password_hash: &'a str,
        ) -> BoxServiceFuture<'a, ServiceResult<bool>> {
            Box::pin(std::future::ready(Ok(
                password_hash == format!("test-hash:{password}")
            )))
        }
    }

    fn block_on<T>(future: impl Future<Output = T>) -> T {
        let waker = Waker::noop();
        let mut context = Context::from_waker(waker);
        let mut future = std::pin::pin!(future);

        loop {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(value) => return value,
                Poll::Pending => std::thread::yield_now(),
            }
        }
    }

    #[test]
    fn password_service_is_object_safe_and_usable_without_runtime() {
        let service: Box<dyn PasswordService> = Box::new(FakePasswordService);

        let password_hash = block_on(service.hash_password("correct horse")).unwrap();

        assert_eq!(password_hash, "test-hash:correct horse");
        assert!(block_on(service.verify_password("correct horse", &password_hash)).unwrap());
        assert!(!block_on(service.verify_password("wrong horse", &password_hash)).unwrap());
    }
}
