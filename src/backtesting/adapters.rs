//! Adapters between the generic simulation contracts and the
//! strategy-bound result types.
//!
//! The simulation layer reports a run as [`crate::simulation::PathOutcome`]s and summarises
//! them as [`crate::simulation::PathStatistics`]. Backtesting keeps the richer
//! [`crate::backtesting::results::SimulationResult`] (carrying a [`crate::pnl::PnL`] and optional risk metrics) and
//! [`SimulationStatsResult`]; this module owns the conversion between the
//! two shapes so that neither simulation nor strategies has to know about
//! it. The aggregate figures of a [`SimulationStatsResult`] are always the
//! ones [`crate::simulation::PathStatistics::from_outcomes`] computes.

use crate::backtesting::results::{SimulationResult, SimulationStatsResult};
use crate::error::SimulationError;
use crate::pnl::PnL;
use crate::simulation::{PathOutcome, PathStatistics};
use chrono::Utc;
use positive::Positive;
use std::collections::HashMap;

impl From<PathOutcome> for SimulationResult {
    /// Lifts a generic outcome into the strategy-bound result.
    ///
    /// The path P&L becomes the realized leg of a [`PnL`] with no costs,
    /// no income and the current timestamp; `simulation_count` is one and
    /// the risk fields are empty, exactly as the strategy simulations have
    /// always filled them.
    fn from(outcome: PathOutcome) -> Self {
        Self {
            simulation_count: 1,
            risk_metrics: None,
            final_equity_percentiles: HashMap::new(),
            max_premium: outcome.max_premium,
            min_premium: outcome.min_premium,
            avg_premium: outcome.avg_premium,
            hit_take_profit: outcome.hit_take_profit,
            hit_stop_loss: outcome.hit_stop_loss,
            expired: outcome.expired,
            expiration_premium: outcome.expiration_premium,
            pnl: PnL {
                realized: outcome.pnl,
                unrealized: None,
                initial_costs: Positive::ZERO,
                initial_income: Positive::ZERO,
                date_time: Utc::now(),
            },
            holding_period: outcome.holding_period,
            exit_reason: outcome.exit_reason,
        }
    }
}

impl From<&SimulationResult> for PathOutcome {
    /// Projects a strategy-bound result onto the generic outcome.
    ///
    /// `pnl` is [`PnL::total_pnl`] (realized plus unrealized), which is the
    /// figure the aggregate statistics have always been computed from.
    fn from(result: &SimulationResult) -> Self {
        Self {
            pnl: result.pnl.total_pnl(),
            holding_period: result.holding_period,
            exit_reason: result.exit_reason.clone(),
            hit_take_profit: result.hit_take_profit,
            hit_stop_loss: result.hit_stop_loss,
            expired: result.expired,
            max_premium: result.max_premium,
            min_premium: result.min_premium,
            avg_premium: result.avg_premium,
            expiration_premium: result.expiration_premium,
        }
    }
}

impl SimulationStatsResult {
    /// Builds the aggregate from a set of per-run results.
    ///
    /// The statistics are [`PathStatistics::from_outcomes`] over the
    /// generic projection of each result; the results themselves are kept
    /// for the per-run report.
    ///
    /// # Errors
    ///
    /// Propagates the errors of [`PathStatistics::from_outcomes`].
    #[must_use = "the aggregate is the only product of this call"]
    pub fn from_results(results: Vec<SimulationResult>) -> Result<Self, SimulationError> {
        let outcomes: Vec<PathOutcome> = results.iter().map(PathOutcome::from).collect();
        let stats = PathStatistics::from_outcomes(&outcomes)?;
        Ok(Self::from_parts(results, stats))
    }

    /// Builds the aggregate from a generic run.
    ///
    /// Each outcome is lifted into a [`SimulationResult`] through
    /// `From<PathOutcome>`; the statistics are computed on the outcomes
    /// before the lift so a `None` P&L is counted the same way either
    /// entry point is used.
    ///
    /// # Errors
    ///
    /// Propagates the errors of [`PathStatistics::from_outcomes`].
    #[must_use = "the aggregate is the only product of this call"]
    pub fn from_outcomes(outcomes: Vec<PathOutcome>) -> Result<Self, SimulationError> {
        let stats = PathStatistics::from_outcomes(&outcomes)?;
        let results = outcomes.into_iter().map(SimulationResult::from).collect();
        Ok(Self::from_parts(results, stats))
    }

    /// Pairs already-computed statistics with the results they describe.
    #[must_use]
    fn from_parts(results: Vec<SimulationResult>, stats: PathStatistics) -> Self {
        Self {
            results,
            total_simulations: stats.total_paths,
            profitable_count: stats.profitable_count,
            loss_count: stats.loss_count,
            average_pnl: stats.average_pnl,
            median_pnl: stats.median_pnl,
            std_dev_pnl: stats.std_dev_pnl,
            best_pnl: stats.best_pnl,
            worst_pnl: stats.worst_pnl,
            win_rate: stats.win_rate,
            average_holding_period: stats.average_holding_period,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::ExitPolicy;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    fn outcome(pnl: Option<Decimal>, holding_period: usize) -> PathOutcome {
        PathOutcome {
            pnl,
            holding_period,
            exit_reason: ExitPolicy::ProfitPercent(dec!(0.5)),
            hit_take_profit: pnl.is_some_and(|p| p > Decimal::ZERO),
            hit_stop_loss: pnl.is_some_and(|p| p < Decimal::ZERO),
            expired: false,
            max_premium: dec!(9.0),
            min_premium: dec!(4.0),
            avg_premium: dec!(6.0),
            expiration_premium: Some(dec!(4.5)),
        }
    }

    #[test]
    fn test_path_outcome_round_trips_through_simulation_result() {
        let original = outcome(Some(dec!(-12.0)), 7);
        let result = SimulationResult::from(original.clone());
        assert_eq!(result.simulation_count, 1);
        assert!(result.risk_metrics.is_none());
        assert!(result.final_equity_percentiles.is_empty());
        assert_eq!(result.pnl.realized, Some(dec!(-12.0)));
        assert_eq!(result.pnl.unrealized, None);
        assert_eq!(result.pnl.initial_costs, Positive::ZERO);
        assert_eq!(result.pnl.initial_income, Positive::ZERO);
        assert_eq!(PathOutcome::from(&result), original);
    }

    #[test]
    fn test_path_outcome_from_result_uses_total_pnl() {
        let mut result = SimulationResult::from(outcome(Some(dec!(10.0)), 1));
        result.pnl.unrealized = Some(dec!(2.5));
        assert_eq!(PathOutcome::from(&result).pnl, Some(dec!(12.5)));
    }

    #[test]
    fn test_from_outcomes_matches_path_statistics() {
        let outcomes = vec![
            outcome(Some(dec!(50.0)), 10),
            outcome(Some(dec!(-100.0)), 20),
            outcome(Some(dec!(25.0)), 15),
            outcome(None, 5),
        ];
        let expected = PathStatistics::from_outcomes(&outcomes).unwrap();
        let stats = SimulationStatsResult::from_outcomes(outcomes).unwrap();

        assert_eq!(stats.results.len(), 4);
        assert_eq!(stats.total_simulations, expected.total_paths);
        assert_eq!(stats.profitable_count, expected.profitable_count);
        assert_eq!(stats.loss_count, expected.loss_count);
        assert_eq!(stats.average_pnl, expected.average_pnl);
        assert_eq!(stats.median_pnl, expected.median_pnl);
        assert_eq!(stats.std_dev_pnl, expected.std_dev_pnl);
        assert_eq!(stats.best_pnl, expected.best_pnl);
        assert_eq!(stats.worst_pnl, expected.worst_pnl);
        assert_eq!(stats.win_rate, expected.win_rate);
        assert_eq!(
            stats.average_holding_period,
            expected.average_holding_period
        );
        assert_eq!(stats.average_pnl, dec!(-6.25));
        assert_eq!(stats.median_pnl, dec!(12.5));
        assert_eq!(stats.win_rate, dec!(50.0));
    }

    #[test]
    fn test_from_results_and_from_outcomes_agree() {
        let outcomes = vec![
            outcome(Some(dec!(3.0)), 1),
            outcome(Some(dec!(-1.0)), 3),
            outcome(Some(dec!(7.0)), 5),
        ];
        let via_outcomes = SimulationStatsResult::from_outcomes(outcomes.clone()).unwrap();
        let results: Vec<SimulationResult> =
            outcomes.into_iter().map(SimulationResult::from).collect();
        let via_results = SimulationStatsResult::from_results(results).unwrap();

        assert_eq!(via_results.total_simulations, 3);
        assert_eq!(via_results.average_pnl, via_outcomes.average_pnl);
        assert_eq!(via_results.median_pnl, dec!(3.0));
        assert_eq!(via_results.std_dev_pnl, dec!(4.0));
        assert_eq!(via_results.best_pnl, dec!(7.0));
        assert_eq!(via_results.worst_pnl, dec!(-1.0));
        assert_eq!(via_results.average_holding_period, dec!(3));
    }

    #[test]
    fn test_from_outcomes_empty_run_is_all_zero() {
        let stats = SimulationStatsResult::from_outcomes(Vec::new()).unwrap();
        assert!(stats.results.is_empty());
        assert_eq!(stats.total_simulations, 0);
        assert_eq!(stats.average_pnl, Decimal::ZERO);
        assert_eq!(stats.win_rate, Decimal::ZERO);
    }

    #[test]
    fn test_from_outcomes_reports_overflow() {
        let outcomes = vec![
            outcome(Some(Decimal::MAX), 1),
            outcome(Some(Decimal::MAX), 1),
        ];
        assert!(matches!(
            SimulationStatsResult::from_outcomes(outcomes),
            Err(SimulationError::Decimal(_))
        ));
    }
}
