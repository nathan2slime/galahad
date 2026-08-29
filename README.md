<div align="center">
  <h1>Galahad</h1>
  <p>
    <strong>Galahad is a framework-agnostic authentication foundation for Rust applications</strong>
  </p>
  <p>

<!-- prettier-ignore-start -->

[![CI](https://github.com/nathan2slime/galahad/actions/workflows/ci.yml/badge.svg)](https://github.com/nathan2slime/galahad/actions/workflows/ci.yml)
![Rust](https://img.shields.io/badge/rust-stable-ab6000.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Status](https://img.shields.io/badge/status-MVP%20in%20progress-yellow.svg)

<!-- prettier-ignore-end -->

  </p>
</div>

## Features

- Framework-independent core authentication domain
- Strong domain types for users, password credentials, sessions, and session tokens
- Object-safe repository traits for persistence adapters
- Object-safe service contracts for password, session, and token workflows
- Argon2id password hashing and SHA-256 session token hashing
- Secure session token generation using the operating system CSPRNG
- Session expiration, revocation, and lookup primitives
- Stable internal and public-safe authentication error codes suitable for application-level i18n
- English fallback error messages for developer-facing output
- Facade crate with separate crates for core, Actix, and SeaORM integrations
- SeaORM persistence adapter with PostgreSQL support
- CI checks for formatting, linting, and tests

## Current Scope

Galahad is currently in early MVP development. The project includes core domain
types, concrete password/session services, and a SeaORM persistence adapter.
Actix Web integration is still planned.

Implemented so far:

- `galahad-core` domain models
- `UserRepository`, `CredentialRepository`, and `SessionRepository` contracts
- `PasswordService` and `SessionService` contracts
- Argon2id password hashing
- Email/password sign up and sign in services
- Email and minimum password validation
- User-enumeration-safe public error mapping
- Secure session token generation
- Session token hashing
- Session expiration policy, revocation, and lookup
- SeaORM entities, repositories, migrations, PostgreSQL support, transactions, and integration tests
- `AuthError` with stable internal and public-safe localization codes

Planned for the MVP:

- Actix Web routes and extractors
- HttpOnly cookie support
- Developer-experience facade builder

## Documentation

- [Repository](https://github.com/nathan2slime/galahad)
- [CI workflow](https://github.com/nathan2slime/galahad/actions/workflows/ci.yml)
- API documentation will be published after the first crate release.

## Crate Layout

- `galahad`: Public facade crate that re-exports the workspace crates.
- `galahad-core`: Core authentication domain, errors, repositories, services, and token utilities.
- `galahad-actix`: Planned Actix Web integration crate.
- `galahad-seaorm`: SeaORM persistence integration crate.

## Example

Dependencies:

```toml
[dependencies]
galahad = { git = "https://github.com/nathan2slime/galahad" }
```

Code:

```rust
use std::time::{Duration, SystemTime};

use galahad::core::{
    OsSessionTokenGenerator, Session, SessionExpirationPolicy, SessionId,
    SessionTokenGenerator, SessionTokenHasher, Sha256SessionTokenHasher, User, UserId,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let user = User::new(UserId::from("user-1"), "user@example.com");
    let now = SystemTime::UNIX_EPOCH;
    let expires_at = SessionExpirationPolicy::new(Duration::from_secs(3600))
        .expires_at(now)?;
    let token = OsSessionTokenGenerator::new().generate();
    let token_hash = Sha256SessionTokenHasher::new().hash_token(&token);

    let session = Session::new(
        SessionId::from("session-1"),
        user.id.clone(),
        token_hash.as_str(),
        expires_at,
    );

    assert!(session.is_active_at(now));

    Ok(())
}
```

## Development

Format, lint, and test the complete workspace with:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Roadmap

The first release targets email/password authentication with sessions,
PostgreSQL persistence through SeaORM, and Actix Web integration.

Not included in the initial MVP:

- OAuth
- RBAC
- Organizations
- MFA
- Passkeys
- Magic links
- API keys

## License

This project is licensed under the MIT license.
