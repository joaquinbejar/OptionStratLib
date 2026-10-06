//! Terminal tables for chain and simulation reports.
//!
//! The computational crates return structured data and never write to
//! stdout: an [`OptionChain`](optionstratlib_market::chains::OptionChain)
//! is market data, and
//! [`SimulationStats`](optionstratlib_backtest::backtesting::SimulationStats)
//! and
//! [`SimulationStatsResult`](optionstratlib_backtest::backtesting::SimulationStatsResult)
//! are run statistics. This module is the explicit adapter that turns them
//! into bordered terminal tables, so the terminal-table package is resolved
//! by the presentation crate alone (M6-05, #546).
//!
//! Each report comes in two forms:
//!
//! - `render_*` returns the tables as a `String`, in plain text without ANSI
//!   escapes, ready to log, snapshot or write anywhere.
//! - `print_*` writes the same tables to stdout, coloured when stdout is a
//!   terminal. It is the only place in the library that writes to stdout,
//!   and it does so only when called.
//!
//! | Report | Trait | Methods |
//! | --- | --- | --- |
//! | option chain | [`ChainReport`](crate::visualization::terminal::ChainReport) | `render_table`, `print_table` |
//! | simulation statistics | [`SimulationReport`](crate::visualization::terminal::SimulationReport) | `render_summary`, `print_summary`, `render_individual_results`, `print_individual_results` |
//!
//! ```rust
//! use optionstratlib_core::model::Positive;
//! use optionstratlib_market::chains::OptionChain;
//! use optionstratlib_visualization::visualization::terminal::ChainReport;
//!
//! let chain = OptionChain::new("SPY", Positive::HUNDRED, "2030-01-18".to_string(), None, None);
//! let table = chain.render_table();
//! assert!(table.contains("Strike"));
//! assert!(!table.contains('\u{1b}'));
//! ```

mod backtest;
mod chain;
mod table;

pub use backtest::SimulationReport;
pub use chain::ChainReport;
