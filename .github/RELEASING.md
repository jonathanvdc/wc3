# Releasing the workspace

The crates `wc3-derive`, `wc3`, `bevy-mpq`, and `bevy-wc3` share a release
version. Update their package versions, internal dependency versions, and `Cargo.lock` together.
CI runs `cargo publish --workspace --dry-run --locked` to build the packaged
crates together, including dependencies that are not yet on crates.io.

Push a matching `vX.Y.Z` tag to publish the workspace in dependency order.
Each crate must have a crates.io trusted publisher configured for this GitHub
repository and `.github/workflows/publish.yml`; the workflow uses OIDC rather
than a stored registry token. The Bevy crates require Rust 1.95 or newer.
