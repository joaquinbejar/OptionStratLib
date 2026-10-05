/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 18/12/24
******************************************************************************/
mod custom_test;
mod delta;
mod graph_test;
mod long_call_test;
mod long_put_test;
mod no_lower_break_even_test;
// Every optimisation suite loads a chain fixture from disk.
#[cfg(feature = "io")]
mod optimal;
#[cfg(feature = "io")]
mod optimal_center;
mod protective_put_test;
mod short_call_test;
mod short_put_test;
mod simple;
