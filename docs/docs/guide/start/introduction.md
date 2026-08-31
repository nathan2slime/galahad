# Introduction

Galahad is a focused authentication library for Rust applications. It provides
email/password authentication, Argon2id password hashing, secure sessions,
Actix Web routes and extractors, optional JWT bearer tokens, optional OpenAPI
docs, and SeaORM persistence.

## Scope

Galahad owns authentication concerns:

- User sign up and sign in
- Password hashing and verification
- Session creation, lookup, expiration, revocation, and logout
- Session cookies
- Optional JWT issuance and verification
- Authentication routes and extractors

Your application owns product concerns:

- Authorization and permissions
- Roles and teams
- Organizations and tenants
- Profiles and onboarding fields
- Billing and subscriptions
- Product-specific workflows

## Crates

- `galahad`: Facade crate that re-exports Galahad integrations.
- `galahad-actix`: Actix Web routes, extractors, cookies, and JWT support.
- `galahad-seaorm`: SeaORM repositories and migrations.
- `galahad-core`: Authentication domain types, traits, services, and errors.
