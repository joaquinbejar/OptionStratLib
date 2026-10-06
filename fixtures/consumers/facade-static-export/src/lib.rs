//! Consumer fixture for the `optionstratlib` facade built with only its
//! `static_export` feature (#544). The crate
//! has no code of its own: `tests/consumer.rs` uses the facade the way a
//! downstream service would, and `expect.toml` lists the packages its graph
//! must and must not resolve.
