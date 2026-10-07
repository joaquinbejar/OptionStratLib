/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 8/11/25
******************************************************************************/
use crate::backtesting::results::SimulationResult;
use crate::error::BacktestError;
use optionstratlib_core::model::decimal::d_sum_iter;
use optionstratlib_simulation::simulation::{ExitPolicy, PathOutcome, PathStatistics};
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Running statistics over the runs of a strategy simulation.
///
/// Results are folded in one at a time through [`SimulationStats::update`]
/// (or [`SimulationStats::update_outcome`] for a bare [`PathOutcome`]). Each
/// result is projected onto its [`PathOutcome`] through the backtest
/// adapter, `From<&SimulationResult> for PathOutcome`, so the P&L of a run
/// is [`PnL::total_pnl`](optionstratlib_analytics::pnl::PnL::total_pnl), the
/// same figure [`crate::backtesting::results::SimulationStatsResult`]
/// reports.
///
/// The accumulator counts how each run closed (take profit, stop loss,
/// expiry) and by which exit policy. Every P&L and holding-period figure
/// (mean, median, standard deviation, best, worst, average holding period)
/// comes from [`SimulationStats::statistics`], that is from
/// [`PathStatistics::from_outcomes`] over the stored outcomes; no second,
/// running copy of them is kept.
///
/// # Memory
///
/// Every outcome folded in is stored, and every result folded in through
/// `update` is stored as well, so memory grows linearly with the number of
/// runs. The outcomes cannot be rebuilt from the results on demand: an
/// outcome folded in through `update_outcome` has no result behind it.
#[derive(Debug, Clone)]
pub struct SimulationStats {
    /// Number of runs that closed on their take-profit condition
    profitable_closes: usize,
    /// Number of runs that closed on their stop-loss condition
    loss_closes: usize,
    /// Number of runs that expired without hitting another exit condition
    expired_trades: usize,
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
            profitable_closes: 0,
            loss_closes: 0,
            expired_trades: 0,
            exit_reasons: HashMap::new(),
            results: Vec::new(),
            outcomes: Vec::new(),
        }
    }

    /// Updates statistics with results from a single simulation run.
    ///
    /// The counters are driven by the backtest adapter's `From<&SimulationResult>
    /// for PathOutcome` view of the result, whose P&L is
    /// [`PnL::total_pnl`](optionstratlib_analytics::pnl::PnL::total_pnl)
    /// (realized plus unrealized); the result itself is then stored and read
    /// back through [`SimulationStats::results`]. A result whose P&L reports
    /// a non-zero unrealized leg therefore counts with that leg included, as
    /// in [`crate::backtesting::results::SimulationStatsResult`].
    ///
    /// # Parameters
    ///
    /// * `result` - The simulation result containing all metrics
    ///
    /// # Errors
    ///
    /// Returns [`BacktestError::CounterOverflow`] when a run counter
    /// overflows. The accumulator is left untouched when that happens: every
    /// fallible step is resolved before the first field is written, so a
    /// rejected result cannot leave one counter advanced for a run the
    /// others never saw.
    pub fn update(&mut self, result: SimulationResult) -> Result<(), BacktestError> {
        self.fold(PathOutcome::from(&result))?;
        // The counters have committed; storing the result cannot fail.
        self.results.push(result);
        Ok(())
    }

    /// Folds one generic path outcome into the counters.
    ///
    /// This is the strategy-agnostic half of [`SimulationStats::update`]: it
    /// advances the outcome counters and the exit-reason distribution, and
    /// stores the outcome for [`SimulationStats::statistics`].
    ///
    /// # Errors
    ///
    /// Same contract as [`SimulationStats::update`]: the accumulator is
    /// left untouched when a counter overflows.
    pub fn update_outcome(&mut self, outcome: &PathOutcome) -> Result<(), BacktestError> {
        self.fold(outcome.clone())
    }

    /// Folds an owned outcome: the shared body of `update` and
    /// `update_outcome`, taking the outcome by value so `update` needs no
    /// copy of the projection it just built.
    fn fold(&mut self, outcome: PathOutcome) -> Result<(), BacktestError> {
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

        // Every fallible step above has succeeded, so the writes below commit
        // the outcome as a whole.
        self.profitable_closes = profitable_closes;
        self.loss_closes = loss_closes;
        self.expired_trades = expired_trades;
        self.exit_reasons
            .insert(outcome.exit_reason.clone(), exit_reason_count);
        self.outcomes.push(outcome);
        Ok(())
    }

    /// Aggregate statistics over every outcome folded in so far.
    ///
    /// The figures are [`PathStatistics::from_outcomes`] over the outcomes
    /// [`SimulationStats::update`] and [`SimulationStats::update_outcome`]
    /// accepted, so they follow the same formulas, and for results the same
    /// P&L, as [`crate::backtesting::results::SimulationStatsResult`]. An
    /// outcome without a P&L counts as zero, in the mean and in the best and
    /// worst figures alike.
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

    /// Number of outcomes folded in.
    #[must_use]
    #[inline]
    pub fn total_simulations(&self) -> usize {
        self.outcomes.len()
    }

    /// Number of runs that closed on their take-profit condition.
    #[must_use]
    #[inline]
    pub fn profitable_closes(&self) -> usize {
        self.profitable_closes
    }

    /// Number of runs that closed on their stop-loss condition.
    #[must_use]
    #[inline]
    pub fn loss_closes(&self) -> usize {
        self.loss_closes
    }

    /// Number of runs that reached expiration without another exit.
    #[must_use]
    #[inline]
    pub fn expired_trades(&self) -> usize {
        self.expired_trades
    }

    /// Sum of the P&L of every run, in the strategy's currency.
    ///
    /// Summed over the stored outcomes, so it is the same P&L the
    /// [`SimulationStats::statistics`] mean is taken over; a run without a
    /// P&L counts as zero.
    ///
    /// # Errors
    ///
    /// Returns [`BacktestError::Decimal`] when the sum leaves the
    /// representable `Decimal` range.
    pub fn total_pnl(&self) -> Result<Decimal, BacktestError> {
        Ok(d_sum_iter(
            self.outcomes
                .iter()
                .map(|outcome| outcome.pnl.unwrap_or_default()),
            "backtesting::stats::total_pnl",
        )?)
    }

    /// How many runs each exit policy closed.
    #[must_use]
    #[inline]
    pub fn exit_reasons(&self) -> &HashMap<ExitPolicy, usize> {
        &self.exit_reasons
    }

    /// The results folded in through [`SimulationStats::update`], in
    /// arrival order. Outcomes folded in through
    /// [`SimulationStats::update_outcome`] alone are not stored.
    #[must_use]
    #[inline]
    pub fn results(&self) -> &[SimulationResult] {
        &self.results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtesting::results::SimulationStatsResult;
    use optionstratlib_analytics::pnl::PnL;
    use rust_decimal_macros::dec;

    use chrono::Utc;
    use optionstratlib_core::pos_or_panic;
    use std::collections::HashMap;

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

    fn take_profit(pnl: Decimal, holding_period: usize) -> SimulationResult {
        create_test_result(
            pnl,
            holding_period,
            true,
            false,
            false,
            ExitPolicy::ProfitPercent(dec!(0.5)),
        )
    }

    fn stop_loss(pnl: Decimal, holding_period: usize) -> SimulationResult {
        create_test_result(
            pnl,
            holding_period,
            false,
            true,
            false,
            ExitPolicy::LossPercent(dec!(1.0)),
        )
    }

    fn statistics_of(stats: &SimulationStats) -> PathStatistics {
        match stats.statistics() {
            Ok(statistics) => statistics,
            Err(error) => panic!("the outcomes fit: {error}"),
        }
    }

    fn total_pnl_of(stats: &SimulationStats) -> Decimal {
        match stats.total_pnl() {
            Ok(total) => total,
            Err(error) => panic!("the total fits: {error}"),
        }
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
        assert_eq!(stats.profitable_closes, before.profitable_closes);
        assert_eq!(stats.loss_closes, before.loss_closes);
        assert_eq!(stats.expired_trades, before.expired_trades);
        assert_eq!(stats.exit_reasons, before.exit_reasons);
        assert_eq!(stats.results.len(), before.results.len());
        assert_eq!(stats.outcomes, before.outcomes);
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

        assert_eq!(stats.total_simulations(), 2);
        assert_eq!(stats.profitable_closes, 1);
        assert_eq!(stats.expired_trades, 1);
        assert_eq!(total_pnl_of(&stats), dec!(40.0));
        let statistics = statistics_of(&stats);
        assert_eq!(statistics.best_pnl, dec!(40.0));
        // #691 decision 2: the extremes come from `PathStatistics`, which
        // counts the outcome without a P&L as zero. The running worst used to
        // skip it and read 40.
        assert_eq!(statistics.worst_pnl, Decimal::ZERO);
        assert_eq!(statistics.average_holding_period, dec!(5));
        assert_eq!(stats.exit_reasons.len(), 2);
        assert!(stats.results.is_empty());
    }

    #[test]
    fn test_new_creates_default_stats() {
        let stats = SimulationStats::new();

        assert_eq!(stats.total_simulations(), 0);
        assert_eq!(stats.profitable_closes, 0);
        assert_eq!(stats.loss_closes, 0);
        assert_eq!(stats.expired_trades, 0);
        assert_eq!(total_pnl_of(&stats), Decimal::ZERO);
        assert!(stats.exit_reasons.is_empty());
        assert!(stats.results.is_empty());
        assert!(stats.outcomes.is_empty());
    }

    #[test]
    fn test_default_trait() {
        let stats = SimulationStats::default();

        assert_eq!(stats.total_simulations(), 0);
        assert_eq!(total_pnl_of(&stats), Decimal::ZERO);
    }

    #[test]
    fn test_update_with_profitable_trade() {
        let mut stats = SimulationStats::new();
        stats.update(take_profit(dec!(50.0), 10)).unwrap();

        assert_eq!(stats.total_simulations(), 1);
        assert_eq!(stats.profitable_closes, 1);
        assert_eq!(stats.loss_closes, 0);
        assert_eq!(stats.expired_trades, 0);
        assert_eq!(total_pnl_of(&stats), dec!(50.0));
        let statistics = statistics_of(&stats);
        assert_eq!(statistics.best_pnl, dec!(50.0));
        assert_eq!(statistics.average_holding_period, dec!(10));
        assert_eq!(stats.results.len(), 1);
    }

    #[test]
    fn test_update_with_loss_trade() {
        let mut stats = SimulationStats::new();
        stats.update(stop_loss(dec!(-100.0), 15)).unwrap();

        assert_eq!(stats.total_simulations(), 1);
        assert_eq!(stats.profitable_closes, 0);
        assert_eq!(stats.loss_closes, 1);
        assert_eq!(stats.expired_trades, 0);
        assert_eq!(total_pnl_of(&stats), dec!(-100.0));
        let statistics = statistics_of(&stats);
        assert_eq!(statistics.worst_pnl, dec!(-100.0));
        assert_eq!(statistics.average_holding_period, dec!(15));
    }

    #[test]
    fn test_update_with_expired_trade() {
        let mut stats = SimulationStats::new();
        let result = create_test_result(dec!(25.0), 20, false, false, true, ExitPolicy::Expiration);

        stats.update(result).unwrap();

        assert_eq!(stats.total_simulations(), 1);
        assert_eq!(stats.profitable_closes, 0);
        assert_eq!(stats.loss_closes, 0);
        assert_eq!(stats.expired_trades, 1);
        assert_eq!(total_pnl_of(&stats), dec!(25.0));
        assert_eq!(statistics_of(&stats).average_holding_period, dec!(20));
    }

    #[test]
    fn test_update_multiple_trades() {
        let mut stats = SimulationStats::new();
        stats.update(take_profit(dec!(50.0), 10)).unwrap();
        stats.update(stop_loss(dec!(-100.0), 20)).unwrap();
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

        assert_eq!(stats.total_simulations(), 3);
        assert_eq!(stats.profitable_closes, 1);
        assert_eq!(stats.loss_closes, 1);
        assert_eq!(stats.expired_trades, 1);
        assert_eq!(total_pnl_of(&stats), dec!(-25.0)); // 50 - 100 + 25
        let statistics = statistics_of(&stats);
        assert_eq!(statistics.best_pnl, dec!(50.0));
        assert_eq!(statistics.worst_pnl, dec!(-100.0));
        assert_eq!(statistics.average_holding_period, dec!(15)); // (10 + 20 + 15) / 3
        assert_eq!(stats.results.len(), 3);
    }

    #[test]
    fn test_update_tracks_exit_reasons() {
        let mut stats = SimulationStats::new();
        stats.update(take_profit(dec!(50.0), 10)).unwrap();
        stats.update(take_profit(dec!(50.0), 10)).unwrap();
        stats.update(stop_loss(dec!(-100.0), 20)).unwrap();

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

    /// A result with no P&L at all counts as zero everywhere. #691 decision
    /// 2 re-baselines the extremes: they come from `PathStatistics`, which
    /// reads the missing P&L as zero, instead of running values that skipped
    /// it and stayed at the `Decimal::MIN` / `Decimal::MAX` sentinels.
    #[test]
    fn test_update_with_none_pnl() {
        let mut stats = SimulationStats::new();
        let mut result = take_profit(dec!(50.0), 10);
        result.pnl.realized = None;

        stats.update(result).unwrap();

        assert_eq!(stats.total_simulations(), 1);
        assert_eq!(total_pnl_of(&stats), Decimal::ZERO);
        let statistics = statistics_of(&stats);
        assert_eq!(statistics.best_pnl, Decimal::ZERO);
        assert_eq!(statistics.worst_pnl, Decimal::ZERO);
    }

    #[test]
    fn test_average_holding_period_calculation() {
        let mut stats = SimulationStats::new();

        stats.update(take_profit(dec!(50.0), 10)).unwrap();
        assert_eq!(statistics_of(&stats).average_holding_period, dec!(10));

        stats.update(take_profit(dec!(50.0), 20)).unwrap();
        assert_eq!(statistics_of(&stats).average_holding_period, dec!(15)); // (10 + 20) / 2

        stats.update(take_profit(dec!(50.0), 30)).unwrap();
        assert_eq!(statistics_of(&stats).average_holding_period, dec!(20)); // (10 + 20 + 30) / 3
    }

    #[test]
    fn test_best_pnl_updates_correctly() {
        let mut stats = SimulationStats::new();

        stats.update(take_profit(dec!(50.0), 10)).unwrap();
        assert_eq!(statistics_of(&stats).best_pnl, dec!(50.0));

        stats.update(take_profit(dec!(100.0), 10)).unwrap();
        assert_eq!(statistics_of(&stats).best_pnl, dec!(100.0));

        // Lower profit should not update the best
        stats.update(take_profit(dec!(75.0), 10)).unwrap();
        assert_eq!(statistics_of(&stats).best_pnl, dec!(100.0));
    }

    #[test]
    fn test_worst_pnl_updates_correctly() {
        let mut stats = SimulationStats::new();

        stats.update(stop_loss(dec!(-50.0), 10)).unwrap();
        assert_eq!(statistics_of(&stats).worst_pnl, dec!(-50.0));

        stats.update(stop_loss(dec!(-100.0), 10)).unwrap();
        assert_eq!(statistics_of(&stats).worst_pnl, dec!(-100.0));

        // Smaller loss should not update the worst
        stats.update(stop_loss(dec!(-75.0), 10)).unwrap();
        assert_eq!(statistics_of(&stats).worst_pnl, dec!(-100.0));
    }

    #[test]
    fn test_clone_trait() {
        let mut stats = SimulationStats::new();
        stats.update(take_profit(dec!(50.0), 10)).unwrap();

        let cloned = stats.clone();

        assert_eq!(cloned.total_simulations(), stats.total_simulations());
        assert_eq!(cloned.profitable_closes, stats.profitable_closes);
        assert_eq!(cloned.outcomes, stats.outcomes);
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

        stats.update(take_profit(dec!(50.0), 10)).unwrap();
        assert_eq!(total_pnl_of(&stats), dec!(50.0));

        stats.update(take_profit(dec!(30.0), 10)).unwrap();
        assert_eq!(total_pnl_of(&stats), dec!(80.0));

        stats.update(stop_loss(dec!(-20.0), 10)).unwrap();
        assert_eq!(total_pnl_of(&stats), dec!(60.0));
    }

    /// No running total is kept (#691 decision 2), so a P&L set whose sum
    /// leaves the `Decimal` range is accepted and the overflow is reported
    /// where the sum is taken, with the backtest call-site label, and by
    /// `statistics`, whose mean needs the same sum.
    #[test]
    fn test_total_pnl_overflow_is_reported_when_read() {
        let mut stats = SimulationStats::new();
        assert!(
            stats
                .update(create_test_result(
                    Decimal::MAX,
                    5,
                    true,
                    false,
                    false,
                    ExitPolicy::Expiration,
                ))
                .is_ok()
        );
        assert!(stats.update(stop_loss(dec!(1.0), 7)).is_ok());
        assert_eq!(stats.total_simulations(), 2);

        match stats.total_pnl() {
            Err(BacktestError::Decimal(error)) => {
                assert!(error.to_string().contains("backtesting::stats::total_pnl"));
            }
            other => panic!("expected a Decimal overflow, got {other:?}"),
        }
        assert!(matches!(
            stats.statistics(),
            Err(BacktestError::Simulation(_))
        ));
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
        let cases: [(&str, Saturate, &PathOutcome); 4] = [
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

    /// `update` reports the counter overflow too, and stores neither the
    /// outcome nor the result.
    #[test]
    fn test_update_counter_overflow_stores_no_result() {
        let mut stats = SimulationStats::new();
        stats.profitable_closes = usize::MAX;
        let before = stats.clone();
        assert!(matches!(
            stats.update(take_profit(dec!(50.0), 10)),
            Err(BacktestError::CounterOverflow {
                counter: "profitable_closes"
            })
        ));
        assert_untouched(&stats, &before);
        assert_eq!(stats.total_simulations(), 0);
    }

    /// Shapes of the P&L the library builds: an early exit (realized, no
    /// unrealized leg) and an expiry (realized plus a zero unrealized leg,
    /// as `Position::calculate_pnl_at_expiration` returns it).
    fn library_shaped_results() -> Vec<SimulationResult> {
        let mut expiry =
            create_test_result(dec!(25.0), 15, false, false, true, ExitPolicy::Expiration);
        expiry.pnl.unrealized = Some(Decimal::ZERO);
        vec![
            take_profit(dec!(50.0), 10),
            stop_loss(dec!(-100.0), 20),
            expiry,
        ]
    }

    /// Asserts that the accumulator over `results` reports the
    /// `SimulationStatsResult` aggregate over the same results.
    fn assert_agrees_with_simulation_stats_result(results: Vec<SimulationResult>) {
        let stats = match accumulate(&results) {
            Ok(stats) => stats,
            Err(error) => panic!("the results fit: {error}"),
        };
        let aggregate = match SimulationStatsResult::from_results(results) {
            Ok(aggregate) => aggregate,
            Err(error) => panic!("the results fit: {error}"),
        };
        let statistics = statistics_of(&stats);

        assert_eq!(stats.total_simulations(), aggregate.total_simulations);
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
    }

    /// On results the library builds, the statistics are exactly the
    /// `PathStatistics` aggregate `SimulationStatsResult` reports.
    #[test]
    fn test_update_agrees_with_simulation_stats_result() {
        let results = library_shaped_results();
        assert_agrees_with_simulation_stats_result(results.clone());

        let stats = match accumulate(&results) {
            Ok(stats) => stats,
            Err(error) => panic!("the results fit: {error}"),
        };
        let statistics = statistics_of(&stats);
        assert_eq!(stats.total_simulations(), 3);
        assert_eq!(total_pnl_of(&stats), dec!(-25.0));
        assert_eq!(statistics.average_pnl, dec!(-25.0) / dec!(3));
        assert_eq!(statistics.best_pnl, dec!(50.0));
        assert_eq!(statistics.worst_pnl, dec!(-100.0));
        assert_eq!(statistics.average_holding_period, dec!(15));
        assert_eq!(statistics.profitable_count, 2);
        assert_eq!(statistics.loss_count, 1);
        assert_eq!(statistics.median_pnl, dec!(25.0));
        assert_eq!(statistics.win_rate, dec!(2) / dec!(3) * dec!(100.0));
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
        assert_eq!(total_pnl_of(&stats), Decimal::ZERO);
        assert!(matches!(
            stats.statistics(),
            Err(BacktestError::Simulation(_))
        ));
    }

    /// A result whose P&L reports a non-zero unrealized leg counts with that
    /// leg, as in `SimulationStatsResult`. #691 decision 1 re-baselines this
    /// case: the accumulator used to sum the realized leg alone (10 here)
    /// while the aggregate summed the total (12.5).
    #[test]
    fn test_update_sums_total_pnl_with_unrealized_leg() {
        let mut result = take_profit(dec!(10.0), 4);
        result.pnl.unrealized = Some(dec!(2.5));
        let mut unrealized_only = take_profit(dec!(0.0), 6);
        unrealized_only.pnl.realized = None;
        unrealized_only.pnl.unrealized = Some(dec!(-3.0));
        let results = vec![result, unrealized_only];

        assert_agrees_with_simulation_stats_result(results.clone());

        let stats = match accumulate(&results) {
            Ok(stats) => stats,
            Err(error) => panic!("the results fit: {error}"),
        };
        assert_eq!(total_pnl_of(&stats), dec!(9.5));
        let statistics = statistics_of(&stats);
        assert_eq!(statistics.best_pnl, dec!(12.5));
        assert_eq!(statistics.worst_pnl, dec!(-3.0));
    }

    /// `update` folds exactly the adapter's projection: it leaves the
    /// accumulator as `update_outcome` on that projection does, apart from
    /// the stored result.
    #[test]
    fn test_update_matches_update_outcome_on_the_adapter_projection() {
        let mut via_result = SimulationStats::new();
        let mut via_outcome = SimulationStats::new();
        let mut results = library_shaped_results();
        let mut unrealized = take_profit(dec!(10.0), 4);
        unrealized.pnl.unrealized = Some(dec!(2.5));
        results.push(unrealized);
        for result in results {
            let outcome = PathOutcome::from(&result);
            assert!(via_outcome.update_outcome(&outcome).is_ok());
            assert!(via_result.update(result).is_ok());
        }
        assert_eq!(via_result.profitable_closes, via_outcome.profitable_closes);
        assert_eq!(via_result.loss_closes, via_outcome.loss_closes);
        assert_eq!(via_result.expired_trades, via_outcome.expired_trades);
        assert_eq!(via_result.exit_reasons, via_outcome.exit_reasons);
        assert_eq!(via_result.outcomes, via_outcome.outcomes);
        assert_eq!(via_result.results.len(), 4);
        assert!(via_outcome.results.is_empty());
    }

    #[test]
    fn test_results_vector_grows() {
        let mut stats = SimulationStats::new();

        for i in 0..10 {
            stats.update(take_profit(dec!(50.0), i)).unwrap();
        }

        assert_eq!(stats.results.len(), 10);
        assert_eq!(stats.total_simulations(), 10);
    }

    /// The read accessors report exactly the folded counters and totals, so
    /// a renderer outside this crate sees what `update` accumulated.
    #[test]
    fn test_accessors_report_the_folded_state() {
        let empty = SimulationStats::new();
        assert_eq!(empty.total_simulations(), 0);
        assert!(empty.results().is_empty());
        assert!(empty.exit_reasons().is_empty());

        let mut stats = SimulationStats::new();
        stats.update(take_profit(dec!(40.0), 4)).unwrap();
        stats.update(stop_loss(dec!(-10.0), 8)).unwrap();
        stats
            .update(create_test_result(
                dec!(5.0),
                12,
                false,
                false,
                true,
                ExitPolicy::Expiration,
            ))
            .unwrap();

        assert_eq!(stats.total_simulations(), 3);
        assert_eq!(stats.profitable_closes(), 1);
        assert_eq!(stats.loss_closes(), 1);
        assert_eq!(stats.expired_trades(), 1);
        assert_eq!(total_pnl_of(&stats), dec!(35.0));
        assert_eq!(stats.exit_reasons().len(), 3);
        assert_eq!(stats.exit_reasons().get(&ExitPolicy::Expiration), Some(&1));
        assert_eq!(stats.results().len(), 3);
    }
}
