//! Simulation statistics as terminal tables.

use super::table::{Section, boxed_table, header_row, print, render};
use crate::error::GraphError;
use optionstratlib_backtest::backtesting::{
    SimulationResult, SimulationStats, SimulationStatsResult,
};
use optionstratlib_core::model::decimal::d_sum_iter;
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
/// places (as the tables always have). The average holding period of a
/// [`SimulationStats`] summary is rounded half-up to two decimal places
/// (#691); a [`SimulationStatsResult`] summary writes its own with `{:.2}`.
/// Exit reasons are listed in the order of their text, so the output is
/// deterministic.
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
/// The P&L of a run is its total P&L (realized plus unrealized,
/// `PnL::total_pnl`) for both types, the figure each accumulates (#691).
/// The [`SimulationStats`] summary takes its average P&L, extremes and
/// average holding period from `SimulationStats::statistics`, so a run
/// without a P&L counts as zero there, as in the [`SimulationStatsResult`]
/// summary.
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
    /// the average P&L) leaves the representable `Decimal` range, and
    /// [`GraphError::Backtest`] when the statistics of a [`SimulationStats`]
    /// accumulator cannot be computed (its P&L total or variance leaves the
    /// `Decimal` range).
    fn render_summary(&self) -> Result<String, GraphError>;

    /// Writes the summary tables to stdout, coloured when stdout is a
    /// terminal.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::render_summary`], and
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
        render(&individual_results_sections(self.results(), total_pnl_of))
    }

    fn print_individual_results(&self) -> Result<(), GraphError> {
        print(&individual_results_sections(self.results(), total_pnl_of))
    }
}

/// The P&L both report types show for a run: realized plus unrealized,
/// zero when neither is known.
#[must_use]
#[inline]
fn total_pnl_of(result: &SimulationResult) -> Decimal {
    result.pnl.total_pnl().unwrap_or_default()
}

/// `count` as a percentage of `total`, with two decimal places.
///
/// The same `f64` expression the tables have always printed, `NaN%` for
/// zero runs included.
#[must_use]
fn percentage(count: usize, total: usize) -> String {
    format!("{:.2}%", (count as f64 / total as f64) * PERCENT)
}

/// `value` with two decimal places, rounded half-up (#691 decision 2).
///
/// `Decimal`'s own `{:.2}` truncates, so the value is rounded first: a tie
/// goes away from zero, which for the non-negative holding periods this
/// formats is up.
#[must_use]
fn two_decimals(value: Decimal) -> String {
    format!(
        "{:.2}",
        value.round_dp_with_strategy(DISPLAY_DECIMALS, RoundingStrategy::MidpointAwayFromZero)
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
///
/// Every P&L and holding-period figure comes from
/// `SimulationStats::statistics` (#691 decision 2) and the total from
/// `SimulationStats::total_pnl`, both over the same outcomes.
fn stats_summary_sections(stats: &SimulationStats) -> Result<Vec<Section>, GraphError> {
    let total = stats.total_simulations();
    let statistics = stats.statistics()?;

    let mut outcomes = titled_table(&["Outcome", "Count", "Percentage"]);
    if total > 0 {
        outcomes.add_row(Row::new(vec![
            Cell::new("Take-Profit Closes"),
            Cell::new(&stats.profitable_closes().to_string())
                .with_style(Attr::ForegroundColor(color::GREEN)),
            Cell::new(&percentage(stats.profitable_closes(), total))
                .with_style(Attr::ForegroundColor(color::GREEN)),
        ]));
        outcomes.add_row(Row::new(vec![
            Cell::new("Stop-Loss Closes"),
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
        Cell::new(&money(stats.total_pnl()?)),
    ]));
    if total > 0 {
        pnl.add_row(Row::new(vec![
            Cell::new("Average P&L per Trade"),
            Cell::new(&money(statistics.average_pnl)),
        ]));
    }
    pnl.add_row(Row::new(vec![
        Cell::new("Maximum Profit"),
        Cell::new(&money(statistics.best_pnl)).with_style(Attr::ForegroundColor(color::GREEN)),
    ]));
    pnl.add_row(Row::new(vec![
        Cell::new("Maximum Loss"),
        Cell::new(&money(statistics.worst_pnl)).with_style(Attr::ForegroundColor(color::RED)),
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
            holding_period_table(two_decimals(statistics.average_holding_period)),
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

    /// Both types show the total P&L of a run. #691 decision 1 re-baselines
    /// the `SimulationStats` side, which showed the realized leg alone
    /// (`$10.00`) and now shows the total, as it accumulates it.
    #[test]
    fn test_terminal_individual_results_show_total_pnl() {
        let mut run = result(1, dec!(10.0), 5, false, ExitPolicy::Expiration);
        run.pnl.unrealized = Some(dec!(2.5));

        let mut stats_result = stats_result();
        stats_result.results = vec![run.clone()];
        assert!(stats_result.render_individual_results().contains("$12.50"));

        let mut stats = SimulationStats::new();
        stats.update(run).unwrap();
        let rendered = stats.render_individual_results();
        assert!(rendered.contains("$12.50"));
        assert!(!rendered.contains("$10.00"));
        let summary = stats.render_summary().unwrap();
        assert!(summary.contains("$12.50"));
        assert!(!summary.contains("$10.00"));
    }

    /// #691 decision 2: the extremes come from `PathStatistics`, which reads a
    /// run without a P&L as zero. The running extremes used to skip it and
    /// print the `Decimal::MIN` / `Decimal::MAX` sentinels.
    #[test]
    fn test_terminal_simulation_summary_reads_missing_pnl_as_zero() {
        let mut run = result(1, dec!(10.0), 5, false, ExitPolicy::Expiration);
        run.pnl.realized = None;
        let mut stats = SimulationStats::new();
        stats.update(run).unwrap();

        let rendered = stats.render_summary().unwrap();
        let maximum_profit = rendered
            .lines()
            .find(|line| line.contains("Maximum Profit"))
            .unwrap();
        let maximum_loss = rendered
            .lines()
            .find(|line| line.contains("Maximum Loss"))
            .unwrap();
        assert!(maximum_profit.contains("$0.00"), "{maximum_profit}");
        assert!(maximum_loss.contains("$0.00"), "{maximum_loss}");
        assert!(!rendered.contains(&Decimal::MAX.to_string()));
    }

    /// An accumulator whose statistics cannot be computed is reported, not
    /// printed with made-up figures.
    #[test]
    fn test_terminal_simulation_summary_statistics_overflow_is_backtest_error() {
        let mut stats = SimulationStats::new();
        stats
            .update(result(1, Decimal::MAX, 1, false, ExitPolicy::Expiration))
            .unwrap();
        stats
            .update(result(2, Decimal::MAX, 1, false, ExitPolicy::Expiration))
            .unwrap();
        assert!(matches!(
            stats.render_summary(),
            Err(GraphError::Backtest(_))
        ));
        assert!(matches!(
            stats.print_summary(),
            Err(GraphError::Backtest(_))
        ));
    }

    /// The average holding period of a `SimulationStats` summary is the
    /// `PathStatistics` mean rounded half-up to two places: 157 steps over 3
    /// runs is 52.333…, 313 over 6 is 52.1666… and prints 52.17, where
    /// `Decimal`'s own `{:.2}` would truncate it to 52.16.
    #[test]
    fn test_terminal_simulation_summary_rounds_holding_period_half_up() {
        let mut stats = SimulationStats::new();
        for (sim, holding) in [50, 50, 50, 50, 50, 63].into_iter().enumerate() {
            stats
                .update(result(
                    sim,
                    dec!(1.0),
                    holding,
                    false,
                    ExitPolicy::Expiration,
                ))
                .unwrap();
        }
        let rendered = stats.render_summary().unwrap();
        assert!(rendered.contains("52.17 steps"), "{rendered}");
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

    /// `two_decimals` rounds half-up, ties included, instead of truncating
    /// or rounding half-to-even.
    #[test]
    fn test_terminal_two_decimals_rounds_half_up() {
        assert_eq!(two_decimals(dec!(52.1666)), "52.17");
        assert_eq!(two_decimals(dec!(52.164)), "52.16");
        assert_eq!(two_decimals(dec!(2.125)), "2.13");
        assert_eq!(two_decimals(dec!(2.135)), "2.14");
        assert_eq!(two_decimals(dec!(15)), "15.00");
        assert_eq!(two_decimals(Decimal::ZERO), "0.00");
    }
}
