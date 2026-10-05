//! Consumer fixture for the `optionstratlib` facade built with only its
//! `market` capability (#528). The crate has no code of its own:
//! `tests/consumer.rs` builds and reads option chains and series through the
//! facade the way a downstream market-data service would, and `expect.toml`
//! lists the packages its graph must and must not resolve.
