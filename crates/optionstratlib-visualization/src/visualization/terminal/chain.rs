//! The option chain as terminal tables.

use super::table::{Section, boxed_table, header_row, print, render};
use crate::error::GraphError;
use optionstratlib_market::chains::{OPTION_CHAIN_TABLE_HEADERS, OptionChain};
use prettytable::{Attr, Cell, Row, color};
use rust_decimal::Decimal;

/// Strikes on a multiple of this many points are highlighted.
const HIGHLIGHTED_STRIKE_MULTIPLE: u32 = 25;

/// Headers of the table naming the chain.
const CHAIN_HEADER_COLUMNS: [&str; 3] = ["Symbol", "Underlying Price", "Expiration Date"];

/// Terminal tables for an option chain.
///
/// The chain renders as two box-drawn tables: one naming the symbol, the
/// underlying price and the expiration date, then one row per strike with
/// the columns of
/// [`OPTION_CHAIN_TABLE_HEADERS`](optionstratlib_market::chains::OPTION_CHAIN_TABLE_HEADERS)
/// and the cells of
/// [`OptionData::table_cells`](optionstratlib_market::chains::OptionData::table_cells).
/// In colour, the headers are green, the chain's name magenta, and the rows
/// of strikes on a multiple of 25 points yellow.
///
/// This replaces `OptionChain::show`; `OptionChain`'s own `Display` is the
/// dependency-free plain-text table.
pub trait ChainReport {
    /// The chain's tables as plain text, without ANSI escapes.
    #[must_use]
    fn render_table(&self) -> String;

    /// Writes the chain's tables to stdout, coloured when stdout is a
    /// terminal.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError::Io`] when stdout cannot be written.
    fn print_table(&self) -> Result<(), GraphError>;
}

impl ChainReport for OptionChain {
    fn render_table(&self) -> String {
        render(&chain_sections(self))
    }

    fn print_table(&self) -> Result<(), GraphError> {
        print(&chain_sections(self))
    }
}

/// The header table and the strike table of `chain`.
#[must_use]
fn chain_sections(chain: &OptionChain) -> [Section; 2] {
    let mut header = boxed_table();
    header.add_row(header_row(&CHAIN_HEADER_COLUMNS, color::GREEN));
    header.add_row(Row::new(
        [
            chain.symbol.clone(),
            chain.underlying_price.to_string(),
            chain.get_expiration_date(),
        ]
        .iter()
        .map(|value| Cell::new(value).with_style(Attr::ForegroundColor(color::MAGENTA)))
        .collect(),
    ));

    let highlight = Decimal::from(HIGHLIGHTED_STRIKE_MULTIPLE);
    let mut strikes = boxed_table();
    strikes.add_row(header_row(&OPTION_CHAIN_TABLE_HEADERS, color::GREEN));
    for option in &chain.options {
        let highlighted = option.strike_price.is_multiple_of_dec(highlight);
        strikes.add_row(Row::new(
            option
                .table_cells()
                .iter()
                .map(|value| {
                    let cell = Cell::new(value);
                    if highlighted {
                        cell.with_style(Attr::ForegroundColor(color::YELLOW))
                    } else {
                        cell
                    }
                })
                .collect(),
        ));
    }

    [Section::untitled(header), Section::untitled(strikes)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use optionstratlib_core::model::Positive;
    use optionstratlib_core::pos_or_panic;
    use optionstratlib_market::chains::OptionData;
    use rust_decimal_macros::dec;

    fn option(strike: Positive) -> OptionData {
        OptionData {
            strike_price: strike,
            call_bid: Some(pos_or_panic!(5.12345)),
            call_ask: Some(pos_or_panic!(5.5)),
            call_middle: Some(pos_or_panic!(5.31)),
            put_bid: Some(pos_or_panic!(1.2)),
            put_ask: None,
            put_middle: None,
            implied_volatility: pos_or_panic!(0.2),
            delta_call: Some(dec!(0.55)),
            delta_put: Some(dec!(-0.45)),
            gamma: Some(dec!(0.0123)),
            volume: Some(pos_or_panic!(250.0)),
            open_interest: Some(1000),
            ..OptionData::default()
        }
    }

    fn chain() -> OptionChain {
        let mut chain = OptionChain::new(
            "TEST",
            Positive::HUNDRED,
            "2030-01-18".to_string(),
            None,
            None,
        );
        chain.options.insert(option(pos_or_panic!(95.0)));
        chain.options.insert(option(Positive::HUNDRED));
        chain
    }

    #[test]
    fn test_terminal_chain_render_table_matches_snapshot() {
        assert_eq!(
            chain().render_table(),
            include_str!("snapshots/chain_table.txt")
        );
    }

    #[test]
    fn test_terminal_chain_render_table_has_no_ansi_escapes() {
        assert!(!chain().render_table().contains('\u{1b}'));
    }

    #[test]
    fn test_terminal_chain_render_table_empty_chain_has_headers_only() {
        let empty = OptionChain::new("EMPTY", Positive::ONE, "2030-01-18".to_string(), None, None);
        let rendered = empty.render_table();
        assert!(rendered.contains("EMPTY"));
        assert!(rendered.contains("Strike"));
        assert!(!rendered.contains("1000"));
    }

    #[test]
    fn test_terminal_chain_print_table_writes_without_error() {
        assert!(chain().print_table().is_ok());
    }
}
