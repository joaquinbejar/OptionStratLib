//! Consumer fixture for `optionstratlib-market` (#537, ADR-0003). The crate
//! has no code of its own: `tests/consumer.rs` uses the market crate the
//! way a downstream market-data service would, and `expect.toml` lists the
//! packages its graph must and must not resolve.
