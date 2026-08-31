# Routes

Galahad registers Actix routes under `/auth`.

## Sign Up

```http
POST /auth/sign-up
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "correct horse battery staple",
  "name": "Ada Lovelace",
  "company_id": "company-1"
}
```

```json
{
  "id": "generated-user-id",
  "email": "user@example.com"
}
```

Additional fields are passed to the configured sign-up hook.

## Sign In

```http
POST /auth/sign-in
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "correct horse battery staple"
}
```

The response sets a session cookie and returns the authenticated session.

```http
Set-Cookie: galahad_session=<token>; HttpOnly; SameSite=Lax; Path=/
```

```json
{
  "user": {
    "id": "generated-user-id",
    "email": "user@example.com"
  },
  "session": {
    "id": "session-id",
    "expires_at_unix_seconds": 1728000000
  },
  "access_token": "jwt-token"
}
```

`access_token` is present only when JWT support is configured.

## Current Session

```http
GET /auth/session
Cookie: galahad_session=<token>
```

JWT-enabled applications can also use a Bearer token.

```http
GET /auth/session
Authorization: Bearer <jwt-token>
```

The response shape matches sign in.

## Sign Out

```http
POST /auth/sign-out
Cookie: galahad_session=<token>
```

```http
204 No Content
Set-Cookie: galahad_session=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0
```

## Route Protection

Use `AuthenticatedUser` for routes that require a valid session, and
`OptionalUser` for routes that can behave differently when a user is signed in.

```rust
use actix_web::HttpResponse;
use galahad::actix::{AuthenticatedUser, OptionalUser};

async fn account(user: AuthenticatedUser) -> HttpResponse {
    HttpResponse::Ok().body(user.0.email)
}

async fn home(user: OptionalUser) -> HttpResponse {
    match user.0 {
        Some(user) => HttpResponse::Ok().body(format!("Signed in as {}", user.email)),
        None => HttpResponse::Ok().body("Signed out"),
    }
}
```
