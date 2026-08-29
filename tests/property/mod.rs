//! Property-based tests for OptionStratLib
//!
//! This module contains property-based tests using proptest to verify
//! mathematical invariants and bounds across a wide range of inputs.

mod curves_panic_freedom_test;
mod greeks_bounds_test;
mod panic_freedom_test;
mod pricing_panic_freedom_test;
mod put_call_parity_test;
mod quote_invariants_test;
