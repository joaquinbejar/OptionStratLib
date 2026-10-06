/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 8/11/25
******************************************************************************/
use crate::backtesting::results::SimulationResult;
use crate::error::BacktestError;
use optionstratlib_core::model::decimal::d_add;
use optionstratlib_simulation::simulation::{ExitPolicy, PathOutcome, PathStatistics};
use prettytable::{Cell, Row, Table, format};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use tracing::info;

/// Running statistics over the runs of a strategy simulation.
///
/// Results are folded in one at a time through [`SimulationStats::update`]
/// (or [`SimulationStats::update_outcome`] for a bare [`PathOutcome`]). Each
/// result is projected onto its [`PathOutcome`] through the backtest
/// adapter, `From<&SimulationResult> for PathOutcome`, with one documented
/// exception: the P&L summed and compared here is the realized leg only (see
/// [`SimulationStats::update`]).
#[derive(Debug, Clone)]
pub struct SimulationStats {
    /// Total number of simulations run
    total_simulations: usize,
    /// Number of trades that closed with profit (50% premium reduction)
    profitable_closes: usize,
    /// Number of trades that closed with loss (100% premium increase)
    loss_closes: usize,
    /// Number of trades that expired without hitting exit conditions
    expired_trades: usize,
    /// Total profit/loss across all simulations in dollars
    total_pnl: Decimal,
    /// Maximum profit achieved in a single simulation in dollars
    max_profit: Decimal,
    /// Maximum loss incurred in a single simulation in dollars
    max_loss: Decimal,
    /// Average holding period in steps for closed trades
    avg_holding_period: f64,
    /// Distribution of exit policies that triggered exits
    exit_reasons: HashMap<ExitPolicy, usize>,
    /// Individual simulation results
    results: Vec<SimulationResult>,
    /// Every outcome folded in, in arrival order, for
    /// [`SimulationStats::statistics`]
    outcomes: Vec<PathOutcome>,
}

/// Adds one to a run counter, reporting the overflow instead of wrapping.
///
/// A `usize` counter cannot realistically reach its maximum here, but
/// [`SimulationStats::update`] promises to apply a result whole or not at
/// all, and that promise is only keepable if every arithmetic step in it can
/// be answered before the first field is written.
#[inline]
fn checked_increment(current: usize, counter: &'static str) -> Result<usize, BacktestError> {
    current
        .checked_add(1)
        .ok_or_else(|| BacktestError::counter_overflow(counter))
}

impl Default for SimulationStats {
    fn default() -> Self {
        Self::new()
    }
}

impl SimulationStats {
    /// Creates a new instance of `SimulationStats` with default values.
    ///
    /// # Returns
    ///
    /// A new `SimulationStats` instance with all counters set to zero.
    #[must_use]
    pub fn new() -> Self {
        Self {
            total_simulations: 0,
            profitable_closes: 0,
            loss_closes: 0,
            expired_trades: 0,
            total_pnl: dec!(0.0),
            max_profit: Decimal::MIN,
            max_loss: Decimal::MAX,
            avg_holding_period: 0.0,
            exit_reasons: HashMap::new(),
            results: Vec::new(),
            outcomes: Vec::new(),
        }
    }

    /// Updates statistics with results from a single simulation run.
    ///
    /// The counters are driven by the generic [`PathOutcome`] view of the
    /// result through [`SimulationStats::update_outcome`]; the result
    /// itself is then stored for [`SimulationStats::print_individual_results`].
    ///
    /// The view is the backtest adapter's `From<&SimulationResult> for
    /// PathOutcome` for every field but `pnl`, which is
    /// `result.pnl.realized` instead of the adapter's
    /// [`PnL::total_pnl`](optionstratlib_analytics::pnl::PnL::total_pnl). Every
    /// result the library builds has the two equal (an early exit carries no
    /// unrealized leg, an expiry carries a zero one), but a caller-built result
    /// with a non-zero unrealized leg would be summed differently, so this
    /// accumulator keeps the realized figure it has always reported while
    /// [`crate::backtesting::results::SimulationStatsResult`] reports the
    /// total.
    ///
    /// # Parameters
    ///
    /// * `result` - The simulation result containing all metrics
    ///
    /// # Errors
    ///
    /// Returns [`BacktestError::Decimal`] when the running P&L total leaves
    /// the representable `Decimal` range, and
    /// [`BacktestError::CounterOverflow`] when a run counter overflows.
    /// Skipping the addition instead would leave every later reader looking
    /// at a total that is quietly wrong, which is worse than the abort this
    /// replaces; the run is reported and the caller decides.
    ///
    /// The accumulator is left untouched when that happens. Every fallible
    /// step is resolved before the first field is written, so a rejected
    /// result cannot leave `total_simulations` counting a run whose P&L,
    /// outcome counters and stored result never landed, which would report
    /// every derived ratio against a denominator nobody can see.
    pub fn update(&mut self, result: SimulationResult) -> Result<(), BacktestError> {
        let outcome = PathOutcome {
            pnl: result.pnl.realized,
            ..PathOutcome::from(&result)
        };
        self.update_outcome(&outcome)?;
        // The counters have committed; storing the result cannot fail.
        self.results.push(result);
        Ok(())
    }

    /// Folds one generic path outcome into the counters.
    ///
    /// This is the strategy-agnostic half of [`SimulationStats::update`]:
    /// it advances the run count, the outcome counters, the exit-reason
    /// distribution, the P&L total and extremes, and the average holding
    /// period. `outcome.pnl` is the figure summed and compared; a `None`
    /// counts as zero in the total and leaves the extremes untouched.
    ///
    /// # Errors
    ///
    /// Same contract as [`SimulationStats::update`]: the accumulator is
    /// left untouched when the P&L total overflows `Decimal` or a counter
    /// overflows.
    pub fn update_outcome(&mut self, outcome: &PathOutcome) -> Result<(), BacktestError> {
        let total_simulations = checked_increment(self.total_simulations, "total_simulations")?;
        let total_pnl = d_add(
            self.total_pnl,
            outcome.pnl.unwrap_or(dec!(0.0)),
            "backtesting::stats::total_pnl",
        )?;

        let mut profitable_closes = self.profitable_closes;
        let mut loss_closes = self.loss_closes;
        let mut expired_trades = self.expired_trades;
        if outcome.hit_take_profit {
            profitable_closes = checked_increment(profitable_closes, "profitable_closes")?;
        } else if outcome.hit_stop_loss {
            loss_closes = checked_increment(loss_closes, "loss_closes")?;
        } else if outcome.expired {
            expired_trades = checked_increment(expired_trades, "expired_trades")?;
        }

        let exit_reason_count = checked_increment(
            self.exit_reasons
                .get(&outcome.exit_reason)
                .copied()
                .unwrap_or(0),
            "exit_reasons",
        )?;

        let (max_profit, max_loss) = match outcome.pnl {
            Some(realized) => (self.max_profit.max(realized), self.max_loss.min(realized)),
            None => (self.max_profit, self.max_loss),
        };

        // `total_simulations` is at least one here, so the subtraction cannot
        // underflow and the division cannot be by zero.
        let total_holding = self.avg_holding_period * (total_simulations - 1) as f64;
        let avg_holding_period =
            (total_holding + outcome.holding_period as f64) / total_simulations as f64;

        // Every fallible step above has succeeded, so the writes below commit
        // the outcome as a whole.
        self.total_simulations = total_simulations;
        self.total_pnl = total_pnl;
        self.profitable_closes = profitable_closes;
        self.loss_closes = loss_closes;
        self.expired_trades = expired_trades;
        self.exit_reasons
            .insert(outcome.exit_reason.clone(), exit_reason_count);
        self.max_profit = max_profit;
        self.max_loss = max_loss;
        self.avg_holding_period = avg_holding_period;
        self.outcomes.push(outcome.clone());
        Ok(())
    }

    /// Aggregate statistics over every outcome folded in so far.
    ///
    /// The figures are [`PathStatistics::from_outcomes`] over the outcomes
    /// [`SimulationStats::update`] and [`SimulationStats::update_outcome`]
    /// accepted, so they follow the same formulas as
    /// [`crate::backtesting::results::SimulationStatsResult`]. The P&L of a
    /// result folded in through `update` is its realized leg (see
    /// [`SimulationStats::update`]), which equals the `PnL::total_pnl` the
    /// aggregate uses for every result the library builds.
    /// [`SimulationStats::print_summary`] does not read these figures.
    ///
    /// # Errors
    ///
    /// Returns [`BacktestError::Simulation`] carrying the error of
    /// [`PathStatistics::from_outcomes`], for example when the variance of a
    /// P&L set far outside any traded size overflows `Decimal`.
    #[must_use = "the statistics are the only product of this call"]
    pub fn statistics(&self) -> Result<PathStatistics, BacktestError> {
        Ok(PathStatistics::from_outcomes(&self.outcomes)?)
    }

    /// Prints a formatted summary of the simulation statistics.
    pub fn print_summary(&self) {
        info!("========== SIMULATION SUMMARY ==========");

        // General Info Table
        let mut info_table = Table::new();
        info_table.set_format(*format::consts::FORMAT_BOX_CHARS);
        info_table.add_row(Row::new(vec![
            Cell::new("Metric").style_spec("Fb"),
            Cell::new("Value").style_spec("Fb"),
        ]));
        info_table.add_row(Row::new(vec![
            Cell::new("Total Simulations"),
            Cell::new(&self.total_simulations.to_string()),
        ]));
        info_table.printstd();

        // Trade Outcomes Table
        info!("--- Trade Outcomes ---");
        let mut outcomes_table = Table::new();
        outcomes_table.set_format(*format::consts::FORMAT_BOX_CHARS);
        outcomes_table.add_row(Row::new(vec![
            Cell::new("Outcome").style_spec("Fb"),
            Cell::new("Count").style_spec("Fb"),
            Cell::new("Percentage").style_spec("Fb"),
        ]));

        if self.total_simulations > 0 {
            let win_rate = (self.profitable_closes as f64 / self.total_simulations as f64) * 100.0;
            let loss_rate = (self.loss_closes as f64 / self.total_simulations as f64) * 100.0;
            let expired_rate = (self.expired_trades as f64 / self.total_simulations as f64) * 100.0;

            outcomes_table.add_row(Row::new(vec![
                Cell::new("Profitable Closes (50% reduction)"),
                Cell::new(&self.profitable_closes.to_string()).style_spec("Fg"),
                Cell::new(&format!("{:.2}%", win_rate)).style_spec("Fg"),
            ]));
            outcomes_table.add_row(Row::new(vec![
                Cell::new("Loss Closes (100% increase)"),
                Cell::new(&self.loss_closes.to_string()).style_spec("Fr"),
                Cell::new(&format!("{:.2}%", loss_rate)).style_spec("Fr"),
            ]));
            outcomes_table.add_row(Row::new(vec![
                Cell::new("Expired Trades"),
                Cell::new(&self.expired_trades.to_string()),
                Cell::new(&format!("{:.2}%", expired_rate)),
            ]));
        }
        outcomes_table.printstd();

        // P&L Statistics Table
        info!("--- Profit/Loss Statistics ---");
        let mut pnl_table = Table::new();
        pnl_table.set_format(*format::consts::FORMAT_BOX_CHARS);
        pnl_table.add_row(Row::new(vec![
            Cell::new("Metric").style_spec("Fb"),
            Cell::new("Amount").style_spec("Fb"),
        ]));

        pnl_table.add_row(Row::new(vec![
            Cell::new("Total P&L"),
            Cell::new(&format!("${:.2}", self.total_pnl)),
        ]));

        if self.total_simulations > 0 {
            let avg_pnl = self.total_pnl / Decimal::from(self.total_simulations);
            pnl_table.add_row(Row::new(vec![
                Cell::new("Average P&L per Trade"),
                Cell::new(&format!("${:.2}", avg_pnl)),
            ]));
        }

        pnl_table.add_row(Row::new(vec![
            Cell::new("Maximum Profit"),
            Cell::new(&format!("${:.2}", self.max_profit)).style_spec("Fg"),
        ]));
        pnl_table.add_row(Row::new(vec![
            Cell::new("Maximum Loss"),
            Cell::new(&format!("${:.2}", self.max_loss)).style_spec("Fr"),
        ]));
        pnl_table.printstd();

        // Holding Period Table
        info!("--- Holding Period ---");
        let mut holding_table = Table::new();
        holding_table.set_format(*format::consts::FORMAT_BOX_CHARS);
        holding_table.add_row(Row::new(vec![
            Cell::new("Metric").style_spec("Fb"),
            Cell::new("Value").style_spec("Fb"),
        ]));
        holding_table.add_row(Row::new(vec![
            Cell::new("Average Holding Period"),
            Cell::new(&format!("{:.2} steps", self.avg_holding_period)),
        ]));
        holding_table.printstd();

        // Exit Reasons Table
        info!("--- Exit Reasons ---");
        let mut exit_table = Table::new();
        exit_table.set_format(*format::consts::FORMAT_BOX_CHARS);
        exit_table.add_row(Row::new(vec![
            Cell::new("Exit Reason").style_spec("Fb"),
            Cell::new("Count").style_spec("Fb"),
            Cell::new("Percentage").style_spec("Fb"),
        ]));

        for (reason, count) in &self.exit_reasons {
            let percentage = (*count as f64 / self.total_simulations as f64) * 100.0;
            exit_table.add_row(Row::new(vec![
                Cell::new(&reason.to_string()),
                Cell::new(&count.to_string()),
                Cell::new(&format!("{:.2}%", percentage)),
            ]));
        }
        exit_table.printstd();

        info!("==================================================");
    }

    /// Prints detailed results for each individual simulation in a table format.
    pub fn print_individual_results(&self) {
        info!("========== INDIVIDUAL SIMULATION RESULTS ==========");

        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_BOX_CHARS);

        // Add header
        table.add_row(Row::new(vec![
            Cell::new("Sim"),
            Cell::new("Max\nPremium"),
            Cell::new("Min\nPremium"),
            Cell::new("Avg\nPremium"),
            Cell::new("Final\nP&L"),
            Cell::new("Holding\nPeriod"),
            Cell::new("Exit\nReason"),
        ]));

        // Add data rows
        for result in &self.results {
            table.add_row(Row::new(vec![
                Cell::new(&result.simulation_count.to_string()),
                Cell::new(&format!("${:.2}", result.max_premium)),
                Cell::new(&format!("${:.2}", result.min_premium)),
                Cell::new(&format!("${:.2}", result.avg_premium)),
                Cell::new(&format!("${:.2}", result.pnl.realized.unwrap_or(dec!(0.0)))),
                Cell::new(&result.holding_period.to_string()),
                Cell::new(&result.exit_reason.to_string()),
            ]));
        }

        table.printstd();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtesting::results::SimulationStatsResult;
    use optionstratlib_analytics::pnl::PnL;

    use chrono::Utc;
    use optionstratlib_core::pos_or_panic;
    use std::collections::HashMap;

    /// A rejected result must leave the accumulator exactly as it was: the
    /// P&L that overflows arrives after `total_simulations` has a reason to
    /// advance, and advancing it alone would report every later ratio against
    /// a run that contributed nothing.
    #[test]
    fn test_update_rejects_a_result_without_partially_mutating() {
        let mut stats = SimulationStats::new();
        stats
            .update(create_test_result(
                Decimal::MAX,
                5,
                true,
                false,
                false,
                ExitPolicy::Expiration,
            ))
            .expect("the first result fits");
        let before = stats.clone();

        let overflowing =
            create_test_result(dec!(1.0), 7, false, true, false, ExitPolicy::Expiration);
        assert!(stats.update(overflowing).is_err());

        assert_eq!(stats.total_simulations, before.total_simulations);
        assert_eq!(stats.total_pnl, before.total_pnl);
        assert_eq!(stats.profitable_closes, before.profitable_closes);
        assert_eq!(stats.loss_closes, before.loss_closes);
        assert_eq!(stats.expired_trades, before.expired_trades);
        assert_eq!(stats.exit_reasons, before.exit_reasons);
        assert_eq!(stats.max_profit, before.max_profit);
        assert_eq!(stats.max_loss, before.max_loss);
        assert_eq!(stats.results.len(), before.results.len());
        assert!((stats.avg_holding_period - before.avg_holding_period).abs() < f64::EPSILON);
    }

    /// Helper function to create a test SimulationResult
    fn create_test_result(
        pnl_value: Decimal,
        holding_period: usize,
        hit_take_profit: bool,
        hit_stop_loss: bool,
        expired: bool,
        exit_reason: ExitPolicy,
    ) -> SimulationResult {
        SimulationResult {
            simulation_count: 1,
            risk_metrics: None,
            final_equity_percentiles: HashMap::new(),
            max_premium: dec!(100.0),
            min_premium: dec!(50.0),
            avg_premium: dec!(75.0),
            hit_take_profit,
            hit_stop_loss,
            expired,
            expiration_premium: if expired { Some(dec!(50.0)) } else { None },
            pnl: PnL::new(
                Some(pnl_value),
                None,
                pos_or_panic!(10.0),
                pos_or_panic!(5.0),
                Utc::now(),
            ),
            holding_period,
            exit_reason,
        }
    }

    #[test]
    fn test_update_outcome_drives_counters_without_storing_a_result() {
        let mut stats = SimulationStats::new();
        let outcome = PathOutcome {
            pnl: Some(dec!(40.0)),
            holding_period: 8,
            exit_reason: ExitPolicy::ProfitPercent(dec!(0.5)),
            hit_take_profit: true,
            ..PathOutcome::default()
        };
        stats.update_outcome(&outcome).unwrap();
        stats
            .update_outcome(&PathOutcome {
                pnl: None,
                holding_period: 2,
                exit_reason: ExitPolicy::Expiration,
                expired: true,
                ..PathOutcome::default()
            })
            .unwrap();

        assert_eq!(stats.total_simulations, 2);
        assert_eq!(stats.profitable_closes, 1);
        assert_eq!(stats.expired_trades, 1);
        assert_eq!(stats.total_pnl, dec!(40.0));
        assert_eq!(stats.max_profit, dec!(40.0));
        assert_eq!(stats.max_loss, dec!(40.0));
        assert_eq!(stats.avg_holding_period, 5.0);
        assert_eq!(stats.exit_reasons.len(), 2);
        assert!(stats.results.is_empty());
    }

    #[test]
    fn test_new_creates_default_stats() {
        let stats = SimulationStats::new();

        assert_eq!(stats.total_simulations, 0);
        assert_eq!(stats.profitable_closes, 0);
        assert_eq!(stats.loss_closes, 0);
        assert_eq!(stats.expired_trades, 0);
        assert_eq!(stats.total_pnl, dec!(0.0));
        assert_eq!(stats.max_profit, Decimal::MIN);
        assert_eq!(stats.max_loss, Decimal::MAX);
        assert_eq!(stats.avg_holding_period, 0.0);
        assert!(stats.exit_reasons.is_empty());
        assert!(stats.results.is_empty());
    }

    #[test]
    fn test_default_trait() {
        let stats = SimulationStats::default();

        assert_eq!(stats.total_simulations, 0);
        assert_eq!(stats.total_pnl, dec!(0.0));
    }

    #[test]
    fn test_update_with_profitable_trade() {
        let mut stats = SimulationStats::new();
        let result = create_test_result(
            dec!(50.0),
            10,
            true,
            false,
            false,
            ExitPolicy::ProfitPercent(dec!(0.5)),
        );

        stats.update(result).unwrap();

        assert_eq!(stats.total_simulations, 1);
        assert_eq!(stats.profitable_closes, 1);
        assert_eq!(stats.loss_closes, 0);
        assert_eq!(stats.expired_trades, 0);
        assert_eq!(stats.total_pnl, dec!(50.0));
        assert_eq!(stats.max_profit, dec!(50.0));
        assert_eq!(stats.avg_holding_period, 10.0);
        assert_eq!(stats.results.len(), 1);
    }

    #[test]
    fn test_update_with_loss_trade() {
        let mut stats = SimulationStats::new();
        let result = create_test_result(
            dec!(-100.0),
            15,
            false,
            true,
            false,
            ExitPolicy::LossPercent(dec!(1.0)),
        );

        stats.update(result).unwrap();

        assert_eq!(stats.total_simulations, 1);
        assert_eq!(stats.profitable_closes, 0);
        assert_eq!(stats.loss_closes, 1);
        assert_eq!(stats.expired_trades, 0);
        assert_eq!(stats.total_pnl, dec!(-100.0));
        assert_eq!(stats.max_loss, dec!(-100.0));
        assert_eq!(stats.avg_holding_period, 15.0);
    }

    #[test]
    fn test_update_with_expired_trade() {
        let mut stats = SimulationStats::new();
        let result = create_test_result(dec!(25.0), 20, false, false, true, ExitPolicy::Expiration);

        stats.update(result).unwrap();

        assert_eq!(stats.total_simulations, 1);
        assert_eq!(stats.profitable_closes, 0);
        assert_eq!(stats.loss_closes, 0);
        assert_eq!(stats.expired_trades, 1);
        assert_eq!(stats.total_pnl, dec!(25.0));
        assert_eq!(stats.avg_holding_period, 20.0);
    }

    #[test]
    fn test_update_multiple_trades() {
        let mut stats = SimulationStats::new();

        // Add profitable trade
        stats
            .update(create_test_result(
                dec!(50.0),
                10,
                true,
                false,
                false,
                ExitPolicy::ProfitPercent(dec!(0.5)),
            ))
            .unwrap();

        // Add loss trade
        stats
            .update(create_test_result(
                dec!(-100.0),
                20,
                false,
                true,
                false,
                ExitPolicy::LossPercent(dec!(1.0)),
            ))
            .unwrap();

        // Add expired trade
        stats
            .update(create_test_result(
                dec!(25.0),
                15,
                false,
                false,
                true,
                ExitPolicy::Expiration,
            ))
            .unwrap();

        assert_eq!(stats.total_simulations, 3);
        assert_eq!(stats.profitable_closes, 1);
        assert_eq!(stats.loss_closes, 1);
        assert_eq!(stats.expired_trades, 1);
        assert_eq!(stats.total_pnl, dec!(-25.0)); // 50 - 100 + 25
        assert_eq!(stats.max_profit, dec!(50.0));
        assert_eq!(stats.max_loss, dec!(-100.0));
        assert_eq!(stats.avg_holding_period, 15.0); // (10 + 20 + 15) / 3
        assert_eq!(stats.results.len(), 3);
    }

    #[test]
    fn test_update_tracks_exit_reasons() {
        let mut stats = SimulationStats::new();

        stats
            .update(create_test_result(
                dec!(50.0),
                10,
                true,
                false,
                false,
                ExitPolicy::ProfitPercent(dec!(0.5)),
            ))
            .unwrap();

        stats
            .update(create_test_result(
                dec!(50.0),
                10,
                true,
                false,
                false,
                ExitPolicy::ProfitPercent(dec!(0.5)),
            ))
            .unwrap();

        stats
            .update(create_test_result(
                dec!(-100.0),
                20,
                false,
                true,
                false,
                ExitPolicy::LossPercent(dec!(1.0)),
            ))
            .unwrap();

        assert_eq!(stats.exit_reasons.len(), 2);
        assert_eq!(
            *stats
                .exit_reasons
                .get(&ExitPolicy::ProfitPercent(dec!(0.5)))
                .unwrap(),
            2
        );
        assert_eq!(
            *stats
                .exit_reasons
                .get(&ExitPolicy::LossPercent(dec!(1.0)))
                .unwrap(),
            1
        );
    }

    #[test]
    fn test_update_with_none_pnl() {
        let mut stats = SimulationStats::new();
        let mut result = create_test_result(
            dec!(50.0),
            10,
            true,
            false,
            false,
            ExitPolicy::ProfitPercent(dec!(0.5)),
        );

        // Set realized to None
        result.pnl.realized = None;

        stats.update(result).unwrap();

        assert_eq!(stats.total_simulations, 1);
        assert_eq!(stats.total_pnl, dec!(0.0)); // Should use 0.0 when None
        assert_eq!(stats.max_profit, Decimal::MIN); // Should not update
        assert_eq!(stats.max_loss, Decimal::MAX); // Should not update
    }

    #[test]
    fn test_avg_holding_period_calculation() {
        let mut stats = SimulationStats::new();

        // First trade: 10 steps
        stats
            .update(create_test_result(
                dec!(50.0),
                10,
                true,
                false,
                false,
                ExitPolicy::ProfitPercent(dec!(0.5)),
            ))
            .unwrap();
        assert_eq!(stats.avg_holding_period, 10.0);

        // Second trade: 20 steps
        stats
            .update(create_test_result(
                dec!(50.0),
                20,
                true,
                false,
                false,
                ExitPolicy::ProfitPercent(dec!(0.5)),
            ))
            .unwrap();
        assert_eq!(stats.avg_holding_period, 15.0); // (10 + 20) / 2

        // Third trade: 30 steps
        stats
            .update(create_test_result(
                dec!(50.0),
                30,
                true,
                false,
                false,
                ExitPolicy::ProfitPercent(dec!(0.5)),
            ))
            .unwrap();
        assert_eq!(stats.avg_holding_period, 20.0); // (10 + 20 + 30) / 3
    }

    #[test]
    fn test_max_profit_updates_correctly() {
        let mut stats = SimulationStats::new();

        stats
            .update(create_test_result(
                dec!(50.0),
                10,
                true,
                false,
                false,
                ExitPolicy::ProfitPercent(dec!(0.5)),
            ))
            .unwrap();
        assert_eq!(stats.max_profit, dec!(50.0));

        stats
            .update(create_test_result(
                dec!(100.0),
                10,
                true,
                false,
                false,
                ExitPolicy::ProfitPercent(dec!(0.5)),
            ))
            .unwrap();
        assert_eq!(stats.max_profit, dec!(100.0));

        // Lower profit should not update max
        stats
            .update(create_test_result(
                dec!(75.0),
                10,
                true,
                false,
                false,
                ExitPolicy::ProfitPercent(dec!(0.5)),
            ))
            .unwrap();
        assert_eq!(stats.max_profit, dec!(100.0));
    }

    #[test]
    fn test_max_loss_updates_correctly() {
        let mut stats = SimulationStats::new();

        stats
            .update(create_test_result(
                dec!(-50.0),
                10,
                false,
                true,
                false,
                ExitPolicy::LossPercent(dec!(1.0)),
            ))
            .unwrap();
        assert_eq!(stats.max_loss, dec!(-50.0));

        stats
            .update(create_test_result(
                dec!(-100.0),
                10,
                false,
                true,
                false,
                ExitPolicy::LossPercent(dec!(1.0)),
            ))
            .unwrap();
        assert_eq!(stats.max_loss, dec!(-100.0));

        // Smaller loss should not update max_loss
        stats
            .update(create_test_result(
                dec!(-75.0),
                10,
                false,
                true,
                false,
                ExitPolicy::LossPercent(dec!(1.0)),
            ))
            .unwrap();
        assert_eq!(stats.max_loss, dec!(-100.0));
    }

    #[test]
    fn test_print_summary_does_not_panic() {
        let mut stats = SimulationStats::new();

        // Test with empty stats
        stats.print_summary();

        // Test with some data
        stats
            .update(create_test_result(
                dec!(50.0),
                10,
                true,
                false,
                false,
                ExitPolicy::ProfitPercent(dec!(0.5)),
            ))
            .unwrap();
        stats.print_summary();
    }

    #[test]
    fn test_print_individual_results_does_not_panic() {
        let mut stats = SimulationStats::new();

        // Test with empty results
        stats.print_individual_results();

        // Test with some results
        stats
            .update(create_test_result(
                dec!(50.0),
                10,
                true,
                false,
                false,
                ExitPolicy::ProfitPercent(dec!(0.5)),
            ))
            .unwrap();
        stats.print_individual_results();
    }

    #[test]
    fn test_clone_trait() {
        let mut stats = SimulationStats::new();
        stats
            .update(create_test_result(
                dec!(50.0),
                10,
                true,
                false,
                false,
                ExitPolicy::ProfitPercent(dec!(0.5)),
            ))
            .unwrap();

        let cloned = stats.clone();

        assert_eq!(cloned.total_simulations, stats.total_simulations);
        assert_eq!(cloned.profitable_closes, stats.profitable_closes);
        assert_eq!(cloned.total_pnl, stats.total_pnl);
        assert_eq!(cloned.results.len(), stats.results.len());
    }

    #[test]
    fn test_update_with_complex_exit_policy() {
        let mut stats = SimulationStats::new();

        let complex_exit = ExitPolicy::Or(vec![
            ExitPolicy::ProfitPercent(dec!(0.5)),
            ExitPolicy::Expiration,
        ]);

        stats
            .update(create_test_result(
                dec!(50.0),
                10,
                true,
                false,
                false,
                complex_exit.clone(),
            ))
            .unwrap();

        assert_eq!(stats.exit_reasons.len(), 1);
        assert_eq!(*stats.exit_reasons.get(&complex_exit).unwrap(), 1);
    }

    #[test]
    fn test_total_pnl_accumulation() {
        let mut stats = SimulationStats::new();

        stats
            .update(create_test_result(
                dec!(50.0),
                10,
                true,
                false,
                false,
                ExitPolicy::ProfitPercent(dec!(0.5)),
            ))
            .unwrap();
        assert_eq!(stats.total_pnl, dec!(50.0));

        stats
            .update(create_test_result(
                dec!(30.0),
                10,
                true,
                false,
                false,
                ExitPolicy::ProfitPercent(dec!(0.5)),
            ))
            .unwrap();
        assert_eq!(stats.total_pnl, dec!(80.0));

        stats
            .update(create_test_result(
                dec!(-20.0),
                10,
                false,
                true,
                false,
                ExitPolicy::LossPercent(dec!(1.0)),
            ))
            .unwrap();
        assert_eq!(stats.total_pnl, dec!(60.0));
    }

    /// Folds `results` into a fresh accumulator, reporting the first rejection.
    fn accumulate(results: &[SimulationResult]) -> Result<SimulationStats, BacktestError> {
        let mut stats = SimulationStats::new();
        for result in results {
            stats.update(result.clone())?;
        }
        Ok(stats)
    }

    /// Asserts that `stats` is exactly `before`, field by field.
    fn assert_untouched(stats: &SimulationStats, before: &SimulationStats) {
        assert_eq!(stats.total_simulations, before.total_simulations);
        assert_eq!(stats.total_pnl, before.total_pnl);
        assert_eq!(stats.profitable_closes, before.profitable_closes);
        assert_eq!(stats.loss_closes, before.loss_closes);
        assert_eq!(stats.expired_trades, before.expired_trades);
        assert_eq!(stats.exit_reasons, before.exit_reasons);
        assert_eq!(stats.max_profit, before.max_profit);
        assert_eq!(stats.max_loss, before.max_loss);
        assert_eq!(stats.results.len(), before.results.len());
        assert_eq!(stats.outcomes, before.outcomes);
        assert_eq!(
            stats.avg_holding_period.to_bits(),
            before.avg_holding_period.to_bits()
        );
    }

    /// The P&L total overflowing is a `Decimal` failure, reported through
    /// the backtest's own error with the backtest call-site label.
    #[test]
    fn test_update_pnl_overflow_reports_backtest_decimal_error() {
        let mut stats = SimulationStats::new();
        let first = create_test_result(Decimal::MAX, 5, true, false, false, ExitPolicy::Expiration);
        assert!(stats.update(first).is_ok());
        let before = stats.clone();

        let overflowing =
            create_test_result(dec!(1.0), 7, false, true, false, ExitPolicy::Expiration);
        match stats.update(overflowing) {
            Err(BacktestError::Decimal(error)) => {
                assert!(error.to_string().contains("backtesting::stats::total_pnl"));
            }
            other => panic!("expected a Decimal overflow, got {other:?}"),
        }
        assert_untouched(&stats, &before);
    }

    /// Each run counter reports its own name when it cannot advance, and the
    /// rejected outcome leaves every field as it was.
    #[test]
    fn test_update_outcome_counter_overflow_reports_backtest_error() {
        let take_profit = PathOutcome {
            pnl: Some(dec!(1.0)),
            exit_reason: ExitPolicy::ProfitPercent(dec!(0.5)),
            hit_take_profit: true,
            ..PathOutcome::default()
        };
        let stop_loss = PathOutcome {
            pnl: Some(dec!(-1.0)),
            exit_reason: ExitPolicy::LossPercent(dec!(1.0)),
            hit_stop_loss: true,
            ..PathOutcome::default()
        };
        let expired = PathOutcome {
            pnl: Some(dec!(0.5)),
            exit_reason: ExitPolicy::Expiration,
            expired: true,
            ..PathOutcome::default()
        };

        /// Puts one counter at the top of its range.
        type Saturate = fn(&mut SimulationStats);
        let cases: [(&str, Saturate, &PathOutcome); 5] = [
            (
                "total_simulations",
                |stats| stats.total_simulations = usize::MAX,
                &take_profit,
            ),
            (
                "profitable_closes",
                |stats| stats.profitable_closes = usize::MAX,
                &take_profit,
            ),
            (
                "loss_closes",
                |stats| stats.loss_closes = usize::MAX,
                &stop_loss,
            ),
            (
                "expired_trades",
                |stats| stats.expired_trades = usize::MAX,
                &expired,
            ),
            (
                "exit_reasons",
                |stats| {
                    stats
                        .exit_reasons
                        .insert(ExitPolicy::Expiration, usize::MAX);
                },
                &expired,
            ),
        ];

        for (counter, saturate, outcome) in cases {
            let mut stats = SimulationStats::new();
            saturate(&mut stats);
            let before = stats.clone();
            match stats.update_outcome(outcome) {
                Err(BacktestError::CounterOverflow { counter: reported }) => {
                    assert_eq!(reported, counter);
                }
                other => panic!("expected `{counter}` to overflow, got {other:?}"),
            }
            assert_untouched(&stats, &before);
        }
    }

    /// `update` reports the counter overflow too, and stores no result.
    #[test]
    fn test_update_counter_overflow_stores_no_result() {
        let mut stats = SimulationStats::new();
        stats.total_simulations = usize::MAX;
        let before = stats.clone();
        let result = create_test_result(
            dec!(50.0),
            10,
            true,
            false,
            false,
            ExitPolicy::ProfitPercent(dec!(0.5)),
        );
        assert!(matches!(
            stats.update(result),
            Err(BacktestError::CounterOverflow {
                counter: "total_simulations"
            })
        ));
        assert_untouched(&stats, &before);
    }

    /// Shapes of the P&L the library builds: an early exit (realized, no
    /// unrealized leg) and an expiry (realized plus a zero unrealized leg,
    /// as `Position::calculate_pnl_at_expiration` returns it). On these the
    /// realized figure and `PnL::total_pnl` coincide.
    fn library_shaped_results() -> Vec<SimulationResult> {
        let mut expiry =
            create_test_result(dec!(25.0), 15, false, false, true, ExitPolicy::Expiration);
        expiry.pnl.unrealized = Some(Decimal::ZERO);
        vec![
            create_test_result(
                dec!(50.0),
                10,
                true,
                false,
                false,
                ExitPolicy::ProfitPercent(dec!(0.5)),
            ),
            create_test_result(
                dec!(-100.0),
                20,
                false,
                true,
                false,
                ExitPolicy::LossPercent(dec!(1.0)),
            ),
            expiry,
        ]
    }

    /// On results the library builds, the running figures agree with the
    /// `PathStatistics` aggregate `SimulationStatsResult` reports.
    #[test]
    fn test_update_agrees_with_simulation_stats_result() {
        let results = library_shaped_results();
        for result in &results {
            assert_eq!(result.pnl.total_pnl(), result.pnl.realized);
        }
        let stats = match accumulate(&results) {
            Ok(stats) => stats,
            Err(error) => panic!("the results fit: {error}"),
        };
        let aggregate = match SimulationStatsResult::from_results(results) {
            Ok(aggregate) => aggregate,
            Err(error) => panic!("the results fit: {error}"),
        };

        assert_eq!(stats.total_simulations, aggregate.total_simulations);
        assert_eq!(
            stats.total_pnl / Decimal::from(stats.total_simulations),
            aggregate.average_pnl
        );
        assert_eq!(stats.max_profit, aggregate.best_pnl);
        assert_eq!(stats.max_loss, aggregate.worst_pnl);
        assert_eq!(
            Decimal::from_f64_retain(stats.avg_holding_period),
            Some(aggregate.average_holding_period)
        );

        assert_eq!(stats.total_simulations, 3);
        assert_eq!(stats.total_pnl, dec!(-25.0));
        assert_eq!(aggregate.average_pnl, dec!(-25.0) / dec!(3));
        assert_eq!(aggregate.best_pnl, dec!(50.0));
        assert_eq!(aggregate.worst_pnl, dec!(-100.0));
        assert_eq!(aggregate.average_holding_period, dec!(15));
        assert_eq!(stats.avg_holding_period, 15.0);

        match stats.statistics() {
            Ok(statistics) => {
                assert_eq!(statistics.total_paths, aggregate.total_simulations);
                assert_eq!(statistics.profitable_count, aggregate.profitable_count);
                assert_eq!(statistics.loss_count, aggregate.loss_count);
                assert_eq!(statistics.average_pnl, aggregate.average_pnl);
                assert_eq!(statistics.median_pnl, aggregate.median_pnl);
                assert_eq!(statistics.std_dev_pnl, aggregate.std_dev_pnl);
                assert_eq!(statistics.best_pnl, aggregate.best_pnl);
                assert_eq!(statistics.worst_pnl, aggregate.worst_pnl);
                assert_eq!(statistics.win_rate, aggregate.win_rate);
                assert_eq!(
                    statistics.average_holding_period,
                    aggregate.average_holding_period
                );
                assert_eq!(statistics.profitable_count, 2);
                assert_eq!(statistics.loss_count, 1);
                assert_eq!(statistics.median_pnl, dec!(25.0));
                assert_eq!(statistics.win_rate, dec!(2) / dec!(3) * dec!(100.0));
            }
            Err(error) => panic!("the results fit: {error}"),
        }
    }

    /// No outcome yet: the statistics are the all-zero empty summary.
    #[test]
    fn test_statistics_empty_is_all_zero() {
        match SimulationStats::new().statistics() {
            Ok(statistics) => assert_eq!(statistics, PathStatistics::default()),
            Err(error) => panic!("an empty set has statistics: {error}"),
        }
    }

    /// `update_outcome` feeds the statistics as `update` does, and a P&L
    /// set whose variance leaves the `Decimal` range is reported as the
    /// simulation error `PathStatistics::from_outcomes` raises.
    #[test]
    fn test_statistics_variance_overflow_reports_simulation_error() {
        let mut stats = SimulationStats::new();
        for pnl in [dec!(1e20), dec!(-1e20)] {
            let outcome = PathOutcome {
                pnl: Some(pnl),
                ..PathOutcome::default()
            };
            assert!(stats.update_outcome(&outcome).is_ok());
        }
        assert_eq!(stats.total_pnl, Decimal::ZERO);
        assert!(matches!(
            stats.statistics(),
            Err(BacktestError::Simulation(_))
        ));
    }

    /// A caller-built result with a non-zero unrealized leg is where the two
    /// projections part: this accumulator sums the realized leg, the
    /// aggregate sums `PnL::total_pnl`. Pinned on both sides so that moving
    /// either is a visible decision.
    #[test]
    fn test_update_sums_realized_where_total_pnl_differs() {
        let mut result = create_test_result(
            dec!(10.0),
            4,
            true,
            false,
            false,
            ExitPolicy::ProfitPercent(dec!(0.5)),
        );
        result.pnl.unrealized = Some(dec!(2.5));
        let results = vec![result];

        let stats = match accumulate(&results) {
            Ok(stats) => stats,
            Err(error) => panic!("the result fits: {error}"),
        };
        let aggregate = match SimulationStatsResult::from_results(results) {
            Ok(aggregate) => aggregate,
            Err(error) => panic!("the result fits: {error}"),
        };

        assert_eq!(stats.total_pnl, dec!(10.0));
        assert_eq!(stats.max_profit, dec!(10.0));
        assert_eq!(stats.max_loss, dec!(10.0));
        assert_eq!(aggregate.average_pnl, dec!(12.5));
        assert_eq!(aggregate.best_pnl, dec!(12.5));
        match stats.statistics() {
            Ok(statistics) => {
                assert_eq!(statistics.average_pnl, dec!(10.0));
                assert_eq!(statistics.best_pnl, dec!(10.0));
            }
            Err(error) => panic!("the result fits: {error}"),
        }
    }

    /// Apart from the P&L, `update` folds exactly the adapter's projection:
    /// the counters it drives match `update_outcome` on that projection.
    #[test]
    fn test_update_matches_update_outcome_on_the_adapter_projection() {
        let mut via_result = SimulationStats::new();
        let mut via_outcome = SimulationStats::new();
        for result in library_shaped_results() {
            let outcome = PathOutcome::from(&result);
            assert!(via_outcome.update_outcome(&outcome).is_ok());
            assert!(via_result.update(result).is_ok());
        }
        assert_eq!(via_result.total_simulations, via_outcome.total_simulations);
        assert_eq!(via_result.total_pnl, via_outcome.total_pnl);
        assert_eq!(via_result.profitable_closes, via_outcome.profitable_closes);
        assert_eq!(via_result.loss_closes, via_outcome.loss_closes);
        assert_eq!(via_result.expired_trades, via_outcome.expired_trades);
        assert_eq!(via_result.exit_reasons, via_outcome.exit_reasons);
        assert_eq!(via_result.max_profit, via_outcome.max_profit);
        assert_eq!(via_result.max_loss, via_outcome.max_loss);
        assert_eq!(
            via_result.avg_holding_period.to_bits(),
            via_outcome.avg_holding_period.to_bits()
        );
        assert_eq!(via_result.outcomes, via_outcome.outcomes);
        assert_eq!(via_result.results.len(), 3);
        assert!(via_outcome.results.is_empty());
    }

    #[test]
    fn test_results_vector_grows() {
        let mut stats = SimulationStats::new();

        for i in 0..10 {
            stats
                .update(create_test_result(
                    dec!(50.0),
                    i,
                    true,
                    false,
                    false,
                    ExitPolicy::ProfitPercent(dec!(0.5)),
                ))
                .unwrap();
        }

        assert_eq!(stats.results.len(), 10);
        assert_eq!(stats.total_simulations, 10);
    }
}
