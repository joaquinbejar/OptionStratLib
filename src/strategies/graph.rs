//! Former home of the strategy chart construction.
//!
//! The `Graph` implementations for every concrete strategy, and the
//! `impl_graph_for_payoff_strategy!` macro that generates them, now live in
//! `visualization::strategies` (#505, ADR-0001 D2): `Graph` is a
//! visualization trait and strategies must not depend on visualization.
//! The macro is `#[macro_export]`, so its path
//! (`optionstratlib::impl_graph_for_payoff_strategy!`) is unchanged, and
//! every strategy still implements `Graph`. This module is kept so the
//! `strategies::graph` path stays valid until the 0.22.0 breaking batch.
