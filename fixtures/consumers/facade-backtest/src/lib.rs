//! Consumer fixture for the `optionstratlib` facade built with only its
//! `backtest` capability (#541). The crate has no code of its own:
//! `tests/consumer.rs` uses the facade the way a downstream service would,
//! and `expect.toml` lists the packages its graph must and must not resolve.
