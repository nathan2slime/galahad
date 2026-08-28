# Galahad

Galahad is a Rust library designed to provide shared functionality and
integrations for applications built with Actix and SeaORM.

The project currently provides a workspace bootstrap with placeholder adapter
crates. Adapter implementations and external framework dependencies will be
added in later tasks.

## Crate layout

- `galahad`: Public facade crate that re-exports the workspace crates.
- `core`: `galahad-core`, shared core functionality.
- `actix`: `galahad-actix`, Actix integration.
- `seaorm`: `galahad-seaorm`, SeaORM integration.

## Development

Format, lint, and test the complete workspace with:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
