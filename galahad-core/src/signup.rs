use std::sync::Arc;

use crate::{
    BoxServiceFuture, CredentialRepository, PasswordCredential, PasswordService, ServiceResult,
    SignUpInput, User, UserId, UserRepository,
};

/// Generates identifiers for users created by a sign-up operation.
pub trait UserIdGenerator: Send + Sync {
    /// Generates a new user identifier.
    fn generate(&self) -> UserId;
}

impl<F> UserIdGenerator for F
where
    F: Fn() -> UserId + Send + Sync,
{
    fn generate(&self) -> UserId {
        self()
    }
}

/// Signs users up with an email address and password.
pub struct EmailPasswordSignUpService {
    user_repository: Arc<dyn UserRepository>,
    credential_repository: Arc<dyn CredentialRepository>,
    password_service: Arc<dyn PasswordService>,
    user_id_generator: Arc<dyn UserIdGenerator>,
}

impl EmailPasswordSignUpService {
    /// Creates a sign-up service from repository, password, and identifier dependencies.
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        credential_repository: Arc<dyn CredentialRepository>,
        password_service: Arc<dyn PasswordService>,
        user_id_generator: Arc<dyn UserIdGenerator>,
    ) -> Self {
        Self {
            user_repository,
            credential_repository,
            password_service,
            user_id_generator,
        }
    }

    /// Creates a user and stores its hashed password credential.
    pub fn sign_up<'a>(
        &'a self,
        input: &'a SignUpInput,
    ) -> BoxServiceFuture<'a, ServiceResult<User>> {
        Box::pin(async move {
            crate::email::validate_email(&input.email)?;

            if self
                .user_repository
                .find_by_email(&input.email)
                .await?
                .is_some()
            {
                return Err(crate::AuthError::UserAlreadyExists);
            }

            let user = User::new(self.user_id_generator.generate(), input.email.clone());
            let password_hash = self.password_service.hash_password(&input.password).await?;

            self.user_repository.save(&user).await?;

            let credential = PasswordCredential::new(user.id.clone(), password_hash);
            self.credential_repository.save(&credential).await?;

            Ok(user)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::future::Future;
    use std::sync::{Arc, Mutex};
    use std::task::{Context, Poll, Waker};

    use super::{EmailPasswordSignUpService, UserIdGenerator};
    use crate::{
        BoxRepositoryFuture, BoxServiceFuture, CredentialRepository, PasswordCredential,
        PasswordService, RepositoryResult, ServiceResult, SignUpInput, User, UserId,
        UserRepository,
    };

    #[derive(Default)]
    struct InMemoryRepositories {
        users: Mutex<HashMap<UserId, User>>,
        credentials: Mutex<HashMap<UserId, PasswordCredential>>,
    }

    impl UserRepository for InMemoryRepositories {
        fn find_by_id<'a>(
            &'a self,
            id: &'a UserId,
        ) -> BoxRepositoryFuture<'a, RepositoryResult<Option<User>>> {
            let user = self.users.lock().unwrap().get(id).cloned();
            Box::pin(std::future::ready(Ok(user)))
        }

        fn find_by_email<'a>(
            &'a self,
            email: &'a str,
        ) -> BoxRepositoryFuture<'a, RepositoryResult<Option<User>>> {
            let user = self
                .users
                .lock()
                .unwrap()
                .values()
                .find(|user| user.email == email)
                .cloned();
            Box::pin(std::future::ready(Ok(user)))
        }

        fn save<'a>(&'a self, user: &'a User) -> BoxRepositoryFuture<'a, RepositoryResult<()>> {
            self.users
                .lock()
                .unwrap()
                .insert(user.id.clone(), user.clone());
            Box::pin(std::future::ready(Ok(())))
        }
    }

    impl CredentialRepository for InMemoryRepositories {
        fn find_by_user_id<'a>(
            &'a self,
            user_id: &'a UserId,
        ) -> BoxRepositoryFuture<'a, RepositoryResult<Option<PasswordCredential>>> {
            let credential = self.credentials.lock().unwrap().get(user_id).cloned();
            Box::pin(std::future::ready(Ok(credential)))
        }

        fn save<'a>(
            &'a self,
            credential: &'a PasswordCredential,
        ) -> BoxRepositoryFuture<'a, RepositoryResult<()>> {
            self.credentials
                .lock()
                .unwrap()
                .insert(credential.user_id.clone(), credential.clone());
            Box::pin(std::future::ready(Ok(())))
        }
    }

    struct FakePasswordService;

    impl PasswordService for FakePasswordService {
        fn hash_password<'a>(
            &'a self,
            _password: &'a str,
        ) -> BoxServiceFuture<'a, ServiceResult<String>> {
            Box::pin(std::future::ready(Ok(String::from("fake-hash"))))
        }

        fn verify_password<'a>(
            &'a self,
            _password: &'a str,
            password_hash: &'a str,
        ) -> BoxServiceFuture<'a, ServiceResult<bool>> {
            Box::pin(std::future::ready(Ok(password_hash == "fake-hash")))
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

    fn service(repositories: &Arc<InMemoryRepositories>) -> EmailPasswordSignUpService {
        let id_generator: Arc<dyn UserIdGenerator> = Arc::new(|| UserId::from("user-1"));

        EmailPasswordSignUpService::new(
            repositories.clone(),
            repositories.clone(),
            Arc::new(FakePasswordService),
            id_generator,
        )
    }

    #[test]
    fn successful_sign_up_stores_user_and_hashed_credential() {
        let repositories = Arc::new(InMemoryRepositories::default());
        let service = service(&repositories);
        let input = SignUpInput::new("user@example.com", "correct horse");

        let user = block_on(service.sign_up(&input)).unwrap();

        assert_eq!(user, User::new(UserId::from("user-1"), "user@example.com"));
        assert_eq!(
            repositories.users.lock().unwrap().get(&user.id),
            Some(&user)
        );
        assert_eq!(
            repositories
                .credentials
                .lock()
                .unwrap()
                .get(&user.id)
                .map(|credential| credential.password_hash.as_str()),
            Some("fake-hash")
        );
    }

    #[test]
    fn successful_sign_up_does_not_store_plaintext_password() {
        let repositories = Arc::new(InMemoryRepositories::default());
        let service = service(&repositories);
        let input = SignUpInput::new("user@example.com", "correct horse");

        let user = block_on(service.sign_up(&input)).unwrap();
        let credential = repositories
            .credentials
            .lock()
            .unwrap()
            .get(&user.id)
            .cloned()
            .unwrap();

        assert_ne!(credential.password_hash, input.password);
        assert!(!credential.password_hash.contains(&input.password));
    }

    #[test]
    fn duplicate_email_is_rejected_without_overwriting_existing_data() {
        let repositories = Arc::new(InMemoryRepositories::default());
        let existing_user = User::new(UserId::from("existing-user"), "user@example.com");
        let existing_credential = PasswordCredential::new(existing_user.id.clone(), "old-hash");
        repositories
            .users
            .lock()
            .unwrap()
            .insert(existing_user.id.clone(), existing_user.clone());
        repositories
            .credentials
            .lock()
            .unwrap()
            .insert(existing_user.id.clone(), existing_credential.clone());

        let service = service(&repositories);
        let result =
            block_on(service.sign_up(&SignUpInput::new("user@example.com", "new password")));

        assert_eq!(result, Err(crate::AuthError::UserAlreadyExists));
        assert_eq!(
            repositories.users.lock().unwrap().get(&existing_user.id),
            Some(&existing_user)
        );
        assert_eq!(
            repositories
                .credentials
                .lock()
                .unwrap()
                .get(&existing_user.id),
            Some(&existing_credential)
        );
        assert_eq!(repositories.users.lock().unwrap().len(), 1);
        assert_eq!(repositories.credentials.lock().unwrap().len(), 1);
    }

    #[test]
    fn invalid_email_is_rejected_without_saving_data() {
        let repositories = Arc::new(InMemoryRepositories::default());
        let service = service(&repositories);
        let input = SignUpInput::new("not-an-email", "correct horse");

        let result = block_on(service.sign_up(&input));

        assert_eq!(result, Err(crate::AuthError::InvalidEmail));
        assert!(repositories.users.lock().unwrap().is_empty());
        assert!(repositories.credentials.lock().unwrap().is_empty());
    }
}
