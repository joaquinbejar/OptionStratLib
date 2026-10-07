//! Simulation statistics as terminal tables.

use super::table::{Section, boxed_table, header_row, print, render};
use crate::error::GraphError;
use optionstratlib_backtest::backtesting::{
    SimulationResult, SimulationStats, SimulationStatsResult,
};
use optionstratlib_core::model::decimal::{d_div, d_sum_iter};
use prettytable::{Attr, Cell, Row, Table, color};
use rust_decimal::{Decimal, RoundingStrategy};

/// Banner above the summary tables.
const SUMMARY_BANNER: &str = "========== SIMULATION SUMMARY ==========";
/// Banner above the per-run table.
const INDIVIDUAL_RESULTS_BANNER: &str = "========== INDIVIDUAL SIMULATION RESULTS ==========";
/// Headers of the per-run table.
const INDIVIDUAL_RESULTS_HEADERS: [&str; 7] = [
    "Sim",
    "Max\nPremium",
    "Min\nPremium",
    "Avg\nPremium",
    "Final\nP&L",
    "Holding\nPeriod",
    "Exit\nReason",
];
/// Decimal places of a rounded figure.
const DISPLAY_DECIMALS: u32 = 2;
/// A count is shown as this many percent of the runs.
const PERCENT: f64 = 100.0;
/// Text of an average that cannot be represented as a `Decimal`.
const NO_VALUE: &str = "n/a";

/// Terminal tables for simulation statistics.
///
/// Implemented for
/// [`SimulationStatsResult`](optionstratlib_backtest::backtesting::SimulationStatsResult),
/// the statistics a `Simulate::simulate` run returns, and for
/// [`SimulationStats`](optionstratlib_backtest::backtesting::SimulationStats),
/// the running accumulator. The summary shows the run count, the trade
/// outcomes, the P&L figures, the average holding period and the exit
/// reasons; the individual results show one row per run. Money is written
/// with a `$` sign and `Decimal`'s `{:.2}`, which truncates to two decimal
/// places (as the tables always have); the average holding period is
/// rounded half-to-even, as its `f64` was. Exit reasons are listed in the
/// order of their text, so the output is deterministic.
///
/// In colour, headers are blue, and:
///
/// - in the [`SimulationStatsResult`] summary, every P&L figure but the
///   standard deviation is green when positive and red when negative;
/// - in the [`SimulationStats`] summary, the profitable-close count and
///   share and the maximum profit are always green, the loss-close count and
///   share and the maximum loss always red, and the total and average P&L
///   uncoloured;
/// - in the individual results of both, the final P&L is green when
///   positive and red when negative.
///
/// The P&L of a [`SimulationStatsResult`] run is its total P&L (realized
/// plus unrealized, `PnL::total_pnl`) and the P&L of a [`SimulationStats`]
/// run its realized leg, the figures each type accumulates.
///
/// This replaces the inherent `print_summary` and `print_individual_results`
/// methods those types had: import the trait and call the same names, now
/// returning a `Result`.
pub trait SimulationReport {
    /// The summary tables as plain text, without ANSI escapes.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError::Decimal`] when a derived figure (the total or
    /// the average P&L) leaves the representable `Decimal` range.
    fn render_summary(&self) -> Result<String, GraphError>;

    /// Writes the summary tables to stdout, coloured when stdout is a
    /// terminal.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError::Decimal`] as [`Self::render_summary`] does, and
    /// [`GraphError::Io`] when stdout cannot be written.
    fn print_summary(&self) -> Result<(), GraphError>;

    /// The per-run table as plain text, without ANSI escapes.
    #[must_use]
    fn render_individual_results(&self) -> String;

    /// Writes the per-run table to stdout, coloured when stdout is a
    /// terminal.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError::Io`] when stdout cannot be written.
    fn print_individual_results(&self) -> Result<(), GraphError>;
}

impl SimulationReport for SimulationStatsResult {
    fn render_summary(&self) -> Result<String, GraphError> {
        Ok(render(&result_summary_sections(self)?))
    }

    fn print_summary(&self) -> Result<(), GraphError> {
        print(&result_summary_sections(self)?)
    }

    fn render_individual_results(&self) -> String {
        render(&individual_results_sections(&self.results, total_pnl_of))
    }

    fn print_individual_results(&self) -> Result<(), GraphError> {
        print(&individual_results_sections(&self.results, total_pnl_of))
    }
}

impl SimulationReport for SimulationStats {
    fn render_summary(&self) -> Result<String, GraphError> {
        Ok(render(&stats_summary_sections(self)?))
    }

    fn print_summary(&self) -> Result<(), GraphError> {
        print(&stats_summary_sections(self)?)
    }

    fn render_individual_results(&self) -> String {
        render(&individual_results_sections(
            self.results(),
            realized_pnl_of,
        ))
    }

    fn print_individual_results(&self) -> Result<(), GraphError> {
        print(&individual_results_sections(
            self.results(),
            realized_pnl_of,
        ))
    }
}

/// The P&L a [`SimulationStatsResult`] reports for a run: realized plus
/// unrealized, zero when neither is known.
#[must_use]
#[inline]
fn total_pnl_of(result: &SimulationResult) -> Decimal {
    result.pnl.total_pnl().unwrap_or_default()
}

/// The P&L a [`SimulationStats`] accumulates for a run: the realized leg,
/// zero when unknown.
#[must_use]
#[inline]
fn realized_pnl_of(result: &SimulationResult) -> Decimal {
    result.pnl.realized.unwrap_or_default()
}

/// `count` as a percentage of `total`, with two decimal places.
///
/// The same `f64` expression the tables have always printed, `NaN%` for
/// zero runs included.
#[must_use]
fn percentage(count: usize, total: usize) -> String {
    format!("{:.2}%", (count as f64 / total as f64) * PERCENT)
}

/// `value` with two decimal places, rounded half-to-even.
///
/// `Decimal`'s own `{:.2}` truncates; an `f64` written with `{:.2}` rounds
/// its exact binary value half-to-even. Rounding first keeps a figure that
/// was an `f64` printing exactly as it did, given a value converted with
/// `Decimal::from_f64_retain`.
#[must_use]
fn two_decimals(value: Decimal) -> String {
    format!(
        "{:.2}",
        value.round_dp_with_strategy(DISPLAY_DECIMALS, RoundingStrategy::MidpointNearestEven)
    )
}

/// A dollar amount with two decimal places.
#[must_use]
#[inline]
fn money(value: Decimal) -> String {
    format!("${value:.2}")
}

/// A dollar amount, green when positive and red when negative.
#[must_use]
fn pnl_cell(value: Decimal) -> Cell {
    let cell = Cell::new(&money(value));
    if value < Decimal::ZERO {
        cell.with_style(Attr::ForegroundColor(color::RED))
    } else if value > Decimal::ZERO {
        cell.with_style(Attr::ForegroundColor(color::GREEN))
    } else {
        cell
    }
}

/// A table whose header row is `headers`, in blue.
#[must_use]
fn titled_table(headers: &[&str]) -> Table {
    let mut table = boxed_table();
    table.set_titles(header_row(headers, color::BLUE));
    table
}

/// The run-count table.
#[must_use]
fn run_count_table(total_simulations: usize) -> Table {
    let mut table = titled_table(&["Metric", "Value"]);
    table.add_row(Row::new(vec![
        Cell::new("Total Simulations"),
        Cell::new(&total_simulations.to_string()),
    ]));
    table
}

/// The average-holding-period table, `period` in simulation steps.
#[must_use]
fn holding_period_table(period: String) -> Table {
    let mut table = titled_table(&["Metric", "Value"]);
    table.add_row(Row::new(vec![
        Cell::new("Average Holding Period"),
        Cell::new(&format!("{period} steps")),
    ]));
    table
}

/// How many times each distinct text occurs, in the order of the text.
///
/// Counts are group lengths of the sorted texts, so no counter is
/// incremented.
#[must_use]
fn count_by_text(texts: impl Iterator<Item = String>) -> Vec<(String, usize)> {
    let mut sorted: Vec<String> = texts.collect();
    sorted.sort();
    let mut counts = Vec::new();
    let mut rest = sorted.as_slice();
    while let Some(first) = rest.first() {
        let run = rest.partition_point(|text| text == first);
        counts.push((first.clone(), run));
        rest = rest.get(run..).unwrap_or_default();
    }
    counts
}

/// The exit-reason table, one row per reason in the order of its text.
#[must_use]
fn exit_reasons_table(mut reasons: Vec<(String, usize)>, total: usize) -> Table {
    reasons.sort();
    let mut table = titled_table(&["Exit Reason", "Count", "Percentage"]);
    for (reason, count) in reasons {
        table.add_row(Row::new(vec![
            Cell::new(&reason),
            Cell::new(&count.to_string()),
            Cell::new(&percentage(count, total)),
        ]));
    }
    table
}

/// The summary of a [`SimulationStatsResult`].
fn result_summary_sections(stats: &SimulationStatsResult) -> Result<Vec<Section>, GraphError> {
    let total = stats.total_simulations;
    let expired_count = stats.results.iter().filter(|r| r.expired).count();

    let mut outcomes = titled_table(&["Outcome", "Count", "Percentage"]);
    outcomes.add_row(Row::new(vec![
        Cell::new("Profitable Trades"),
        Cell::new(&stats.profitable_count.to_string()),
        Cell::new(&format!("{:.2}%", stats.win_rate)),
    ]));
    outcomes.add_row(Row::new(vec![
        Cell::new("Loss Trades"),
        Cell::new(&stats.loss_count.to_string()),
        Cell::new(&percentage(stats.loss_count, total)),
    ]));
    outcomes.add_row(Row::new(vec![
        Cell::new("Expired Trades"),
        Cell::new(&expired_count.to_string()),
        Cell::new(&percentage(expired_count, total)),
    ]));

    let total_pnl = d_sum_iter(
        stats.results.iter().filter_map(|r| r.pnl.total_pnl()),
        "visualization::terminal::total_pnl",
    )?;
    let mut pnl = titled_table(&["Metric", "Amount"]);
    pnl.add_row(Row::new(vec![Cell::new("Total P&L"), pnl_cell(total_pnl)]));
    pnl.add_row(Row::new(vec![
        Cell::new("Average P&L per Trade"),
        pnl_cell(stats.average_pnl),
    ]));
    pnl.add_row(Row::new(vec![
        Cell::new("Median P&L"),
        pnl_cell(stats.median_pnl),
    ]));
    pnl.add_row(Row::new(vec![
        Cell::new("Std Dev P&L"),
        Cell::new(&money(stats.std_dev_pnl)),
    ]));
    pnl.add_row(Row::new(vec![
        Cell::new("Maximum Profit"),
        pnl_cell(stats.best_pnl),
    ]));
    pnl.add_row(Row::new(vec![
        Cell::new("Maximum Loss"),
        pnl_cell(stats.worst_pnl),
    ]));

    let reasons = count_by_text(stats.results.iter().map(|r| r.exit_reason.to_string()));

    Ok(vec![
        Section::titled(SUMMARY_BANNER, run_count_table(total)),
        Section::titled("--- Trade Outcomes ---", outcomes),
        Section::titled("--- Profit/Loss Statistics ---", pnl),
        Section::titled(
            "--- Holding Period ---",
            holding_period_table(format!("{:.2}", stats.average_holding_period)),
        ),
        Section::titled("--- Exit Reasons ---", exit_reasons_table(reasons, total)),
    ])
}

/// The summary of a [`SimulationStats`] accumulator.
fn stats_summary_sections(stats: &SimulationStats) -> Result<Vec<Section>, GraphError> {
    let total = stats.total_simulations();

    let mut outcomes = titled_table(&["Outcome", "Count", "Percentage"]);
    if total > 0 {
        outcomes.add_row(Row::new(vec![
            Cell::new("Profitable Closes (50% reduction)"),
            Cell::new(&stats.profitable_closes().to_string())
                .with_style(Attr::ForegroundColor(color::GREEN)),
            Cell::new(&percentage(stats.profitable_closes(), total))
                .with_style(Attr::ForegroundColor(color::GREEN)),
        ]));
        outcomes.add_row(Row::new(vec![
            Cell::new("Loss Closes (100% increase)"),
            Cell::new(&stats.loss_closes().to_string())
                .with_style(Attr::ForegroundColor(color::RED)),
            Cell::new(&percentage(stats.loss_closes(), total))
                .with_style(Attr::ForegroundColor(color::RED)),
        ]));
        outcomes.add_row(Row::new(vec![
            Cell::new("Expired Trades"),
            Cell::new(&stats.expired_trades().to_string()),
            Cell::new(&percentage(stats.expired_trades(), total)),
        ]));
    }

    let mut pnl = titled_table(&["Metric", "Amount"]);
    pnl.add_row(Row::new(vec![
        Cell::new("Total P&L"),
        Cell::new(&money(stats.total_pnl())),
    ]));
    if total > 0 {
        let average = d_div(
            stats.total_pnl(),
            Decimal::from(total),
            "visualization::terminal::average_pnl",
        )?;
        pnl.add_row(Row::new(vec![
            Cell::new("Average P&L per Trade"),
            Cell::new(&money(average)),
        ]));
    }
    pnl.add_row(Row::new(vec![
        Cell::new("Maximum Profit"),
        Cell::new(&money(stats.max_profit())).with_style(Attr::ForegroundColor(color::GREEN)),
    ]));
    pnl.add_row(Row::new(vec![
        Cell::new("Maximum Loss"),
        Cell::new(&money(stats.max_loss())).with_style(Attr::ForegroundColor(color::RED)),
    ]));

    let reasons = stats
        .exit_reasons()
        .iter()
        .map(|(reason, count)| (reason.to_string(), *count))
        .collect();

    Ok(vec![
        Section::titled(SUMMARY_BANNER, run_count_table(total)),
        Section::titled("--- Trade Outcomes ---", outcomes),
        Section::titled("--- Profit/Loss Statistics ---", pnl),
        Section::titled(
            "--- Holding Period ---",
            holding_period_table(
                stats
                    .avg_holding_period()
                    .map_or_else(|| NO_VALUE.to_string(), two_decimals),
            ),
        ),
        Section::titled("--- Exit Reasons ---", exit_reasons_table(reasons, total)),
    ])
}

/// One row per run of `results`, with `pnl_of` as each run's final P&L.
#[must_use]
fn individual_results_sections(
    results: &[SimulationResult],
    pnl_of: fn(&SimulationResult) -> Decimal,
) -> [Section; 1] {
    let mut table = titled_table(&INDIVIDUAL_RESULTS_HEADERS);
    for result in results {
        table.add_row(Row::new(vec![
            Cell::new(&result.simulation_count.to_string()),
            Cell::new(&money(result.max_premium)),
            Cell::new(&money(result.min_premium)),
            Cell::new(&money(result.avg_premium)),
            pnl_cell(pnl_of(result)),
            Cell::new(&result.holding_period.to_string()),
            Cell::new(&result.exit_reason.to_string()),
        ]));
    }
    [Section::titled(INDIVIDUAL_RESULTS_BANNER, table)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use optionstratlib_simulation::simulation::ExitPolicy;
    use rust_decimal_macros::dec;

    fn result(
        sim: usize,
        pnl: Decimal,
        holding_period: usize,
        expired: bool,
        exit_reason: ExitPolicy,
    ) -> SimulationResult {
        let mut result = SimulationResult {
            simulation_count: sim,
            max_premium: dec!(100.0),
            min_premium: dec!(50.0),
            avg_premium: dec!(75.0),
            hit_take_profit: pnl > Decimal::ZERO && !expired,
            hit_stop_loss: pnl < Decimal::ZERO && !expired,
            expired,
            expiration_premium: if expired { Some(dec!(50.0)) } else { None },
            holding_period,
            exit_reason,
            ..SimulationResult::default()
        };
        result.pnl.realized = Some(pnl);
        result
    }

    fn results() -> Vec<SimulationResult> {
        vec![
            result(
                1,
                dec!(100.0),
                10,
                false,
                ExitPolicy::ProfitPercent(dec!(0.5)),
            ),
            result(
                2,
                dec!(-50.0),
                15,
                false,
                ExitPolicy::LossPercent(dec!(1.0)),
            ),
            result(3, dec!(25.0), 20, true, ExitPolicy::Expiration),
        ]
    }

    fn stats_result() -> SimulationStatsResult {
        SimulationStatsResult {
            results: results(),
            total_simulations: 3,
            profitable_count: 2,
            loss_count: 1,
            average_pnl: dec!(25.0),
            median_pnl: dec!(25.0),
            std_dev_pnl: dec!(61.24),
            best_pnl: dec!(100.0),
            worst_pnl: dec!(-50.0),
            win_rate: dec!(66.67),
            average_holding_period: dec!(15.0),
        }
    }

    fn stats() -> SimulationStats {
        let mut stats = SimulationStats::new();
        for result in results() {
            stats.update(result).unwrap();
        }
        stats
    }

    #[test]
    fn test_terminal_backtest_summary_matches_snapshot() {
        assert_eq!(
            stats_result().render_summary().unwrap(),
            include_str!("snapshots/backtest_summary.txt")
        );
    }

    #[test]
    fn test_terminal_backtest_individual_results_match_snapshot() {
        assert_eq!(
            stats_result().render_individual_results(),
            include_str!("snapshots/backtest_individual_results.txt")
        );
    }

    #[test]
    fn test_terminal_simulation_summary_matches_snapshot() {
        assert_eq!(
            stats().render_summary().unwrap(),
            include_str!("snapshots/simulation_summary.txt")
        );
    }

    #[test]
    fn test_terminal_simulation_individual_results_match_snapshot() {
        assert_eq!(
            stats().render_individual_results(),
            include_str!("snapshots/simulation_individual_results.txt")
        );
    }

    /// A summary of zero runs prints what it always printed, `NaN%`
    /// included: the percentages are the same `f64` expression as before.
    #[test]
    fn test_terminal_backtest_empty_summary_keeps_its_percentages() {
        let empty = SimulationStatsResult::default();
        let rendered = empty.render_summary().unwrap();
        assert!(rendered.contains("NaN%"));
        assert!(!rendered.contains('\u{1b}'));
    }

    #[test]
    fn test_terminal_simulation_empty_summary_omits_outcome_rows() {
        let rendered = SimulationStats::new().render_summary().unwrap();
        assert!(!rendered.contains("Profitable Closes"));
        assert!(!rendered.contains("Average P&L per Trade"));
        assert!(rendered.contains("Total Simulations"));
    }

    #[test]
    fn test_terminal_backtest_summary_total_pnl_overflow_is_decimal_error() {
        let mut stats = stats_result();
        stats.results = vec![
            result(1, Decimal::MAX, 1, false, ExitPolicy::Expiration),
            result(2, Decimal::MAX, 1, false, ExitPolicy::Expiration),
        ];
        assert!(matches!(
            stats.render_summary(),
            Err(GraphError::Decimal(_))
        ));
        assert!(matches!(stats.print_summary(), Err(GraphError::Decimal(_))));
    }

    #[test]
    fn test_terminal_backtest_exit_reasons_are_sorted() {
        let rendered = stats_result().render_summary().unwrap();
        let reasons = rendered.split("--- Exit Reasons ---").nth(1).unwrap();
        let expiration = reasons.find("Hold to Expiration").unwrap();
        let profit = reasons.find("Profit Target").unwrap();
        let stop = reasons.find("Stop Loss").unwrap();
        assert!(expiration < profit && profit < stop);
    }

    /// Each type shows the P&L it accumulates: the total for
    /// `SimulationStatsResult`, the realized leg for `SimulationStats`.
    #[test]
    fn test_terminal_individual_results_show_each_type_pnl() {
        let mut run = result(1, dec!(10.0), 5, false, ExitPolicy::Expiration);
        run.pnl.unrealized = Some(dec!(2.5));

        let mut stats_result = stats_result();
        stats_result.results = vec![run.clone()];
        assert!(stats_result.render_individual_results().contains("$12.50"));

        let mut stats = SimulationStats::new();
        stats.update(run).unwrap();
        let rendered = stats.render_individual_results();
        assert!(rendered.contains("$10.00"));
        assert!(!rendered.contains("$12.50"));
    }

    #[test]
    fn test_terminal_reports_print_without_error() {
        assert!(stats_result().print_summary().is_ok());
        assert!(stats_result().print_individual_results().is_ok());
        assert!(stats().print_summary().is_ok());
        assert!(stats().print_individual_results().is_ok());
    }

    #[test]
    fn test_terminal_count_by_text_groups_sorted() {
        let counts = count_by_text(["b", "a", "b", "c", "b"].into_iter().map(String::from));
        assert_eq!(
            counts,
            vec![
                ("a".to_string(), 1),
                ("b".to_string(), 3),
                ("c".to_string(), 1)
            ]
        );
        assert!(count_by_text(std::iter::empty()).is_empty());
    }

    /// Money keeps `Decimal`'s `{:.2}`, which truncates, as the tables
    /// always printed it.
    #[test]
    fn test_terminal_money_truncates_to_two_decimals() {
        assert_eq!(money(dec!(1.239)), "$1.23");
        assert_eq!(money(dec!(-50.0)), "$-50.00");
    }

    #[test]
    fn test_terminal_percentage_matches_the_f64_expression() {
        assert_eq!(percentage(1, 3), "33.33%");
        assert_eq!(percentage(2, 3), "66.67%");
        assert_eq!(percentage(0, 0), "NaN%");
    }

    /// `two_decimals` over `Decimal::from_f64_retain` prints exactly what
    /// `{:.2}` prints for the `f64`, so the average holding period reads as
    /// it did when the table formatted the `f64` field: the running averages
    /// of up to 400 runs of up to 60 steps, exact binary ties (`x.125`,
    /// `x.375`, ...) and values just either side of a decimal tie.
    #[test]
    fn test_terminal_two_decimals_matches_f64_formatting() {
        let check = |value: f64| {
            let decimal = Decimal::from_f64_retain(value).unwrap();
            assert_eq!(two_decimals(decimal), format!("{value:.2}"), "{value:?}");
        };
        for runs in 1..=400_u32 {
            for total in (0..=runs * 60).step_by(7) {
                check(f64::from(total) / f64::from(runs));
            }
        }
        let mut average = 0.0_f64;
        for run in 1..=400_u32 {
            let holding = f64::from((run * 37) % 61);
            average = (average * f64::from(run - 1) + holding) / f64::from(run);
            check(average);
        }
        for whole in 0..200_u32 {
            for eighth in 0..8_u32 {
                check(f64::from(whole) + f64::from(eighth) / 8.0);
            }
            for cents in 0..100_u32 {
                let tie = f64::from(whole) + (f64::from(cents) + 0.5) / 100.0;
                check(tie);
                check(f64::from_bits(tie.to_bits() + 1));
                check(f64::from_bits(tie.to_bits() - 1));
            }
        }
    }
}
