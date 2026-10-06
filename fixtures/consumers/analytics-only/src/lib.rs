//! Consumer fixture for `optionstratlib-analytics` without strategies or the
//! facade (#533). The crate has no code of its own: `tests/consumer.rs`
//! analyses positions and an option chain through analytics-owned APIs the
//! way a downstream risk service would, and `expect.toml` lists the packages
//! its graph must and must not resolve.
