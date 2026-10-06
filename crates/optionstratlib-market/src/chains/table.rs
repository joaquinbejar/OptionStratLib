//! The tabular view of an option chain.
//!
//! An [`OptionChain`] reads as a table: one row per strike, with the call
//! and put quotes, the implied volatility, the deltas, the gamma, the volume
//! and the open interest. This module owns that view as data, the column
//! headers in [`OPTION_CHAIN_TABLE_HEADERS`] and the formatted cells of a row
//! in [`OptionData::table_cells`], and renders it as dependency-free plain
//! text through `Display`.
//!
//! Terminal rendering (box-drawing borders, colours, writing to stdout) is
//! not market data and lives with the presentation layer: the
//! `optionstratlib-visualization` crate draws the same rows as a terminal
//! table (`visualization::terminal`), so a headless consumer of this crate
//! resolves no terminal-table package (M6-05, #546).

use crate::chains::utils::{default_empty_string, empty_string_round_to_3};
use crate::chains::{OptionChain, OptionData};
use rust_decimal::Decimal;
use std::fmt;

/// Number of columns of the option-chain table.
pub const OPTION_CHAIN_TABLE_COLUMNS: usize = 13;

/// Column headers of the option-chain table, in the order of
/// [`OptionData::table_cells`].
///
/// `Gamma` is the gamma per 100 points of the underlying (gamma × 100), as
/// the table has always shown it; `Vol.` is the traded volume and `OI` the
/// open interest.
pub const OPTION_CHAIN_TABLE_HEADERS: [&str; OPTION_CHAIN_TABLE_COLUMNS] = [
    "Strike", "Call Bid", "Call Ask", "Call Mid", "Put Bid", "Put Ask", "Put Mid", "IV", "C-Delta",
    "P-Delta", "Gamma", "Vol.", "OI",
];

/// Gamma is shown per 100 points of the underlying.
const GAMMA_DISPLAY_SCALE: Decimal = Decimal::ONE_HUNDRED;

/// Separator between two columns of the plain-text rendering.
const COLUMN_SEPARATOR: &str = "  ";

impl OptionData {
    /// The formatted cells of this option's row in the option-chain table,
    /// in [`OPTION_CHAIN_TABLE_HEADERS`] order.
    ///
    /// Quotes are rounded to three decimal places and an absent quote is an
    /// empty cell. The implied volatility and the deltas are written with
    /// three decimal places and the gamma, scaled by 100, with four; an
    /// absent delta or gamma reads as zero. The volume and the open interest
    /// are written as stored and an absent one is an empty cell. A gamma
    /// whose scaled value leaves the `Decimal` range is an empty cell rather
    /// than an abort.
    #[must_use]
    pub fn table_cells(&self) -> [String; OPTION_CHAIN_TABLE_COLUMNS] {
        let gamma = self
            .gamma
            .unwrap_or(Decimal::ZERO)
            .checked_mul(GAMMA_DISPLAY_SCALE)
            .map_or_else(String::new, |scaled| format!("{scaled:.4}"));
        [
            self.strike_price.to_string(),
            empty_string_round_to_3(self.call_bid),
            empty_string_round_to_3(self.call_ask),
            empty_string_round_to_3(self.call_middle),
            empty_string_round_to_3(self.put_bid),
            empty_string_round_to_3(self.put_ask),
            empty_string_round_to_3(self.put_middle),
            format!("{:.3}", self.implied_volatility),
            format!("{:.3}", self.delta_call.unwrap_or(Decimal::ZERO)),
            format!("{:.3}", self.delta_put.unwrap_or(Decimal::ZERO)),
            gamma,
            default_empty_string(self.volume),
            default_empty_string(self.open_interest),
        ]
    }
}

/// Writes one row of left-aligned cells padded to `widths`, without
/// trailing whitespace.
fn write_row<S: AsRef<str>>(
    f: &mut fmt::Formatter<'_>,
    cells: &[S],
    widths: &[usize; OPTION_CHAIN_TABLE_COLUMNS],
) -> fmt::Result {
    let mut line = String::new();
    for (index, (cell, width)) in cells.iter().zip(widths.iter()).enumerate() {
        if index > 0 {
            line.push_str(COLUMN_SEPARATOR);
        }
        line.push_str(&format!("{:<width$}", cell.as_ref()));
    }
    writeln!(f, "{}", line.trim_end())
}

/// The option chain as a plain-text table.
///
/// The first line names the symbol, the underlying price and the expiration
/// date; then come the column headers of [`OPTION_CHAIN_TABLE_HEADERS`] and
/// one line per strike with [`OptionData::table_cells`], each column
/// left-aligned to its widest cell. The output holds no box-drawing
/// characters and no ANSI escapes; the `optionstratlib-visualization` crate
/// renders the same rows as a bordered terminal table.
impl fmt::Display for OptionChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Symbol: {}  Underlying Price: {}  Expiration Date: {}",
            self.symbol, self.underlying_price, self.expiration_date
        )?;

        let rows: Vec<[String; OPTION_CHAIN_TABLE_COLUMNS]> =
            self.options.iter().map(OptionData::table_cells).collect();
        let mut widths = OPTION_CHAIN_TABLE_HEADERS.map(str::len);
        for row in &rows {
            for (width, cell) in widths.iter_mut().zip(row.iter()) {
                *width = (*width).max(cell.chars().count());
            }
        }

        write_row(f, &OPTION_CHAIN_TABLE_HEADERS, &widths)?;
        for row in &rows {
            write_row(f, row, &widths)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use optionstratlib_core::model::Positive;
    use optionstratlib_core::pos_or_panic;
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

    #[test]
    fn test_option_data_table_cells_formats_every_column() {
        let cells = option(Positive::HUNDRED).table_cells();
        assert_eq!(
            cells,
            [
                "100", "5.123", "5.5", "5.31", "1.2", "", "", "0.200", "0.550", "-0.450", "1.2300",
                "250", "1000",
            ]
            .map(String::from)
        );
    }

    #[test]
    fn test_option_data_table_cells_absent_greeks_read_as_zero() {
        let data = OptionData {
            strike_price: Positive::HUNDRED,
            implied_volatility: pos_or_panic!(0.2),
            ..OptionData::default()
        };
        let cells = data.table_cells();
        assert_eq!(cells[8], "0.000");
        assert_eq!(cells[9], "0.000");
        assert_eq!(cells[10], "0.0000");
        assert_eq!(cells[11], "");
        assert_eq!(cells[12], "");
    }

    #[test]
    fn test_option_data_table_cells_unrepresentable_gamma_is_empty() {
        let data = OptionData {
            strike_price: Positive::HUNDRED,
            gamma: Some(Decimal::MAX),
            ..OptionData::default()
        };
        assert_eq!(data.table_cells()[10], "");
    }

    #[test]
    fn test_option_chain_display_renders_plain_text_table() {
        let mut chain = OptionChain::new(
            "TEST",
            Positive::HUNDRED,
            "2030-01-18".to_string(),
            None,
            None,
        );
        chain.options.insert(option(pos_or_panic!(95.0)));
        chain.options.insert(option(Positive::HUNDRED));

        let expected = "\
Symbol: TEST  Underlying Price: 100  Expiration Date: 2030-01-18
Strike  Call Bid  Call Ask  Call Mid  Put Bid  Put Ask  Put Mid  IV     C-Delta  P-Delta  Gamma   Vol.  OI
95      5.123     5.5       5.31      1.2                        0.200  0.550    -0.450   1.2300  250   1000
100     5.123     5.5       5.31      1.2                        0.200  0.550    -0.450   1.2300  250   1000
";
        assert_eq!(chain.to_string(), expected);
    }

    #[test]
    fn test_option_chain_display_empty_chain_has_headers_only() {
        let chain = OptionChain::new("EMPTY", Positive::ONE, "2030-01-18".to_string(), None, None);
        let rendered = chain.to_string();
        assert_eq!(rendered.lines().count(), 2);
        assert!(!rendered.contains('\u{1b}'));
    }
}
