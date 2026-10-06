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

mod custom_test;
mod delta;
mod long_call_test;
mod long_put_test;
mod no_lower_break_even_test;
mod optimal;
mod optimal_center;
mod protective_put_test;
mod short_call_test;
mod short_put_test;
mod simple;
