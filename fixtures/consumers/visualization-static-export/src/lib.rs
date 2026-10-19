//! Consumer fixture for `optionstratlib-visualization` used directly, with
//! only its `static_export` feature (#928). The crate has no code of its own:
//! `tests/consumer.rs` uses the component the way a downstream service would,
//! and `expect.toml` lists the packages its graph must and must not resolve.
