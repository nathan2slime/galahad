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
- Strong domain types for users, password credentials, and sessions
- Object-safe repository traits for persistence adapters
- Object-safe service contracts for password and session workflows
- Stable authentication error codes suitable for application-level i18n
- English fallback error messages for developer-facing output
- Facade crate with separate crates for core, Actix, and SeaORM integrations
- CI checks for formatting, linting, and tests

## Current Scope

Galahad is currently in early MVP development. The project includes core domain
types and contracts, while concrete framework and persistence adapters are being
built incrementally.

Implemented so far:

- `galahad-core` domain models
- `UserRepository`, `CredentialRepository`, and `SessionRepository` contracts
- `PasswordService` and `SessionService` contracts
- `AuthError` with stable localization codes

Planned for the MVP:

- SeaORM persistence adapter
- PostgreSQL support
- Argon2id password hashing
- Session token generation and hashing
- Actix Web routes and extractors
- HttpOnly cookie support

## Documentation

- [Repository](https://github.com/nathan2slime/galahad)
- [CI workflow](https://github.com/nathan2slime/galahad/actions/workflows/ci.yml)
- API documentation will be published after the first crate release.

## Crate Layout

- `galahad`: Public facade crate that re-exports the workspace crates.
- `galahad-core`: Core authentication domain, errors, repositories, and service contracts.
- `galahad-actix`: Planned Actix Web integration crate.
- `galahad-seaorm`: Planned SeaORM persistence integration crate.

## Example

Dependencies:

```toml
[dependencies]
galahad = { git = "https://github.com/nathan2slime/galahad" }
```

Code:

```rust
use std::time::{Duration, SystemTime};

use galahad::core::{Session, SessionId, User, UserId};

let user = User::new(UserId::from("user-1"), "user@example.com");
let expires_at = SystemTime::UNIX_EPOCH + Duration::from_secs(3600);

let session = Session::new(
    SessionId::from("session-1"),
    user.id.clone(),
    "hashed-session-token",
    expires_at,
);

assert!(session.is_active_at(SystemTime::UNIX_EPOCH));
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
