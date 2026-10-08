/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 18/12/24
******************************************************************************/

//! Strategy integration suites, through the component paths and without the
//! facade (#534). One test binary, so the suites link once.
//!
//! The optimisation suites (`optimal`, `optimal_center`) load a chain fixture
//! from `examples/Chains` at the workspace root; market's `io` is a
//! dev-dependency of this crate, so they always run here.

mod break_even_refresh_test;
mod contract_size_followups_test;
mod contract_size_test;
mod covered_leg_errors_test;
mod covered_mtm_test;
mod covered_sizing_test;
mod custom_break_even_refresh_test;
mod custom_test;
mod delta;
mod find_optimal_result_test;
mod find_optimal_success_test;
mod long_call_test;
mod long_put_test;
mod no_lower_break_even_test;
mod optimal;
mod optimal_center;
mod partial_cover_test;
mod pnl_quantity_test;
mod protective_put_test;
mod short_call_test;
mod short_put_test;
mod simple;
mod validated_builders_test;
