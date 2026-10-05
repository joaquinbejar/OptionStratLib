//! Consumer fixture for the `optionstratlib` facade built with only its
//! `pricing` capability (#528). The crate has no code of its own:
//! `tests/consumer.rs` prices, reads Greeks and recovers implied volatility
//! through the facade the way a downstream pricing service would, and
//! `expect.toml` lists the packages its graph must and must not resolve.
