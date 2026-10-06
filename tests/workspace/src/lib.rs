//! Cross-component integration tests of the OptionStratLib workspace
//! (ADR-0004 section 8, #534).
//!
//! A test lives here when it checks two component crates against each other,
//! so it belongs to neither alone: the strategies' aggregate Greeks against
//! the per-leg Greeks of pricing, for instance. A test of one component
//! through the lower crates it already depends on stays in that component's
//! `tests/`; a test of the facade paths, prelude or features stays in the
//! facade. The library itself is empty; every module is `#[cfg(test)]`.

/// Strategy aggregate Greeks equal the signed sum of the per-leg pricing
/// Greeks (strategies against pricing, #428).
#[cfg(test)]
mod greeks_side_sign_test;
