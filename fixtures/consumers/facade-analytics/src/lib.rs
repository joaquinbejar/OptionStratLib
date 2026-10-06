//! Consumer fixture for the `optionstratlib` facade built with only its
//! `analytics` capability (#535). The crate has no code of its own:
//! `tests/consumer.rs` analyses a position and an option chain through the
//! facade the way a downstream risk service would, and `expect.toml` lists
//! the packages its graph must and must not resolve, strategies among the
//! absent ones.
