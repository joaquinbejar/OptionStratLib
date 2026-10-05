//! Consumer fixture for `optionstratlib-core` plus `optionstratlib-pricing`
//! (#527). The crate has no code of its own: `tests/consumer.rs` drives the
//! two components the way a downstream pricing service would, and
//! `expect.toml` lists the packages its graph must and must not resolve.
