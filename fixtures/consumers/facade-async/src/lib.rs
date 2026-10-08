//! Consumer fixture for the `optionstratlib` facade built with only its
//! `async` feature (#552). The crate has no code of its own:
//! `tests/consumer.rs` drives the asynchronous market I/O the way a
//! downstream service would, and `expect.toml` lists the packages its graph
//! must and must not resolve.
//!
//! `async` is market I/O only: no strategy is there to build.
//!
//! ```compile_fail,E0433
//! use optionstratlib::strategies::LongCall;
//! ```
