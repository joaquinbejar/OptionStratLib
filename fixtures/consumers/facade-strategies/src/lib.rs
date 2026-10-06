//! Consumer fixture for the `optionstratlib` facade built with only its
//! `strategies` capability (#535). The crate has no code of its own:
//! `tests/consumer.rs` builds and analyses a strategy through the facade the
//! way a downstream trading service would, and `expect.toml` lists the
//! packages its graph must and must not resolve.
