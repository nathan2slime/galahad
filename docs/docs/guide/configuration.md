# Configuration

## Sessions

Customize the session cookie name and time-to-live with `Galahad::session()`.

```rust
use std::time::Duration;

use galahad::Galahad;

let session = Galahad::session()
    .cookie_name("my_session_cookie")
    .ttl(Duration::from_secs(60 * 60));
```

Pass the session config into the Actix builder:

```rust
let auth = Galahad::actix()
    .database(GalahadSeaOrm::new(db))
    .session(session)
    .build();
```

## JWTs

JWT support is optional. When configured, sign-in responses include an
`access_token`, and authenticated routes can read either the session cookie or a
Bearer token.

```rust
use std::time::Duration;

use galahad::{Galahad, GalahadJwtAlgorithm};

let jwt = Galahad::jwt(std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"))
    .algorithm(GalahadJwtAlgorithm::Hs512)
    .issuer("my-api")
    .audience("my-web-app")
    .ttl(Duration::from_secs(60 * 15))
    .leeway(30);
```

```rust
let auth = Galahad::actix()
    .database(GalahadSeaOrm::new(db))
    .jwt(jwt)
    .build();
```

Galahad currently supports HMAC JWT algorithms: `HS256`, `HS384`, and `HS512`.
JWT verification still resolves the persisted session token, so session
revocation and expiration continue to apply.

## Sign-Up Hooks

Extra sign-up fields are application-owned. Galahad accepts them as raw JSON and
passes them to an `after_action` hook after the auth user and password
credential are created.

```rust
use galahad::Galahad;

let sign_up = Galahad::sign_up().after_action(|context| async move {
    let name = context.fields.get("name");
    let company_id = context.fields.get("company_id");

    // Validate and persist application-owned profile fields here.
    Ok(())
});
```

```rust
let auth = Galahad::actix()
    .database(GalahadSeaOrm::new(db))
    .sign_up(sign_up)
    .build();
```

Galahad does not validate, migrate, or persist product-specific fields.
