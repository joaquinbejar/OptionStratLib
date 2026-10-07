//! Consumer fixture for the `optionstratlib` facade built with every
//! capability and without `schema` (#549). The crate has no code of its own:
//! `expect.toml` lists the packages its graph must and must not resolve, and
//! the doctests below show that no component derives `utoipa::ToSchema`
//! without the feature: each one names a type of a different component and
//! fails to compile because it does not implement the trait. The same types
//! implement it in `headless-full` (the default, which has `schema`).
//!
//! The normal graph resolves no `utoipa` at all (`expect.toml`): since
//! `expiration_date` 0.4.1 (#628) the foundational crates bring it only behind
//! their own `utoipa` feature, which only `schema` turns on. The doctests'
//! `utoipa` is a dev-dependency.
//!
//! ```compile_fail,E0277
//! fn derives_schema<T: utoipa::ToSchema>() {}
//! derives_schema::<optionstratlib::model::option::ExoticParams>();
//! ```
//!
//! ```compile_fail,E0277
//! fn derives_schema<T: utoipa::ToSchema>() {}
//! derives_schema::<optionstratlib::geometrics::MergeOperation>();
//! ```
//!
//! ```compile_fail,E0277
//! fn derives_schema<T: utoipa::ToSchema>() {}
//! derives_schema::<optionstratlib::greeks::Greek>();
//! ```
//!
//! ```compile_fail,E0277
//! fn derives_schema<T: utoipa::ToSchema>() {}
//! derives_schema::<optionstratlib::simulation::ExitPolicy>();
//! ```
//!
//! ```compile_fail,E0277
//! fn derives_schema<T: utoipa::ToSchema>() {}
//! derives_schema::<optionstratlib::chains::OptionsInStrike>();
//! ```
//!
//! ```compile_fail,E0277
//! fn derives_schema<T: utoipa::ToSchema>() {}
//! derives_schema::<optionstratlib::pnl::DeltaAdjustment>();
//! ```
//!
//! ```compile_fail,E0277
//! fn derives_schema<T: utoipa::ToSchema>() {}
//! derives_schema::<optionstratlib::strategies::LongCall>();
//! ```
//!
//! ```compile_fail,E0277
//! fn derives_schema<T: utoipa::ToSchema>() {}
//! derives_schema::<optionstratlib::backtesting::results::SimulationResult>();
//! ```
