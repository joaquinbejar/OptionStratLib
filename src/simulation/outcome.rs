//! Generic path evaluation contracts.
//!
//! A simulation produces price paths; what a path is worth depends on the
//! caller (a strategy, a hedging rule, a research fixture). This module
//! defines the contract between the two halves without naming either side:
//!
//! - [`PathEvaluator`] turns one [`crate::simulation::randomwalk::RandomWalk`] into an outcome of the
//!   caller's choosing.
//! - [`PathOutcome`] is the per-path record the library understands: a
//!   P&L, a holding period, an exit reason and the premium marks observed
//!   along the way. It is built from core types only.
//! - [`PathStatistics`] summarises a set of outcomes (mean, median, sample
//!   standard deviation, best, worst, win rate, average holding period).
//! - [`evaluate_paths`] runs an evaluator over every walk of a
//!   [`crate::simulation::simulator::Simulator`].
//!
//! Strategy-bound result types (the ones carrying `PnL` and risk metrics)
//! live in the backtesting layer, which adapts them to and from
//! [`PathOutcome`].

use crate::error::SimulationError;
use crate::model::decimal::{d_add, d_div, d_mul, d_sub, d_sum_iter};
use crate::simulation::ExitPolicy;
use crate::simulation::randomwalk::RandomWalk;
use crate::simulation::simulator::Simulator;
use crate::utils::Len;
use positive::Positive;
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::ops::AddAssign;

/// Outcome of evaluating one simulated price path.
///
/// Every field is a core type, so the record can be produced by any
/// evaluator and summarised by [`PathStatistics`] without knowing what was
/// traded along the path. Premium fields are the marks observed while the
/// position was open, in the quote currency of the instrument.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PathOutcome {
    /// Total P&L of the path, in quote currency. `None` when the path
    /// could not be evaluated (an empty walk yields no P&L); summaries
    /// count such a path as zero.
    pub pnl: Option<Decimal>,
    /// Number of steps the position was held before exit or expiry.
    pub holding_period: usize,
    /// Policy that closed the position (`Expiration` when none fired).
    pub exit_reason: ExitPolicy,
    /// Whether the take-profit threshold was reached at exit.
    pub hit_take_profit: bool,
    /// Whether the stop-loss threshold was reached at exit.
    pub hit_stop_loss: bool,
    /// Whether the position ran to expiry without an exit trigger.
    pub expired: bool,
    /// Highest premium mark observed along the path.
    pub max_premium: Decimal,
    /// Lowest premium mark observed along the path.
    pub min_premium: Decimal,
    /// Arithmetic mean of the premium marks observed along the path.
    pub avg_premium: Decimal,
    /// Premium mark at expiry; `None` when the position exited early.
    pub expiration_premium: Option<Decimal>,
}

/// Summary statistics over a set of [`PathOutcome`]s.
///
/// P&L figures are in quote currency; `win_rate` is a percentage
/// (`0..=100`); `average_holding_period` is in steps.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PathStatistics {
    /// Number of outcomes summarised.
    pub total_paths: usize,
    /// Outcomes with a strictly positive P&L.
    pub profitable_count: usize,
    /// Outcomes with a strictly negative P&L.
    pub loss_count: usize,
    /// Mean P&L.
    pub average_pnl: Decimal,
    /// Median P&L (mean of the two middle values for an even count).
    pub median_pnl: Decimal,
    /// Sample standard deviation of the P&L (`n - 1` denominator; zero for
    /// fewer than two outcomes).
    pub std_dev_pnl: Decimal,
    /// Highest P&L.
    pub best_pnl: Decimal,
    /// Lowest P&L.
    pub worst_pnl: Decimal,
    /// Percentage of outcomes with a strictly positive P&L.
    pub win_rate: Decimal,
    /// Mean holding period in steps.
    pub average_holding_period: Decimal,
}

/// Divides without the default rounding of [`d_div`].
///
/// The strategy simulations have always formed their averages and ratios
/// with the raw `Decimal` operator, which keeps the full quotient; this
/// helper reports the overflow that operator would abort on and changes
/// nothing else, so the summaries stay bit-for-bit what they were.
#[inline]
fn div_unrounded(lhs: Decimal, rhs: Decimal, op: &'static str) -> Result<Decimal, SimulationError> {
    if rhs.is_zero() {
        return Err(SimulationError::invalid_parameters(&format!(
            "{op}: division by zero"
        )));
    }
    lhs.checked_div(rhs).ok_or_else(|| {
        SimulationError::invalid_parameters(&format!("{op}: {lhs} / {rhs} overflowed"))
    })
}

impl PathStatistics {
    /// Summarises a set of outcomes.
    ///
    /// An outcome without a P&L counts as zero. The median averages the
    /// two middle values for an even count; the standard deviation uses
    /// the sample (`n - 1`) denominator and is zero for fewer than two
    /// outcomes. An empty set yields all-zero statistics.
    ///
    /// # Errors
    ///
    /// Returns [`SimulationError::Decimal`] when a sum, difference,
    /// product, or the median and variance quotients (which go through
    /// `d_div`) leave the representable `Decimal` range, and
    /// [`SimulationError::InvalidParameters`] when one of the three
    /// unrounded quotients (mean, win rate, average holding period) does or
    /// the holding-period total overflows `usize`.
    #[must_use = "the statistics are the only product of this call"]
    pub fn from_outcomes(outcomes: &[PathOutcome]) -> Result<Self, SimulationError> {
        let total_paths = outcomes.len();
        let mut pnl_values: Vec<Decimal> = outcomes
            .iter()
            .map(|outcome| outcome.pnl.unwrap_or(dec!(0.0)))
            .collect();
        pnl_values.sort();

        let profitable_count = pnl_values.iter().filter(|&&pnl| pnl > dec!(0.0)).count();
        let loss_count = pnl_values.iter().filter(|&&pnl| pnl < dec!(0.0)).count();

        let sum_pnl = d_sum_iter(
            pnl_values.iter().copied(),
            "PathStatistics::from_outcomes/sum_pnl",
        )?;
        let average_pnl = if total_paths > 0 {
            div_unrounded(
                sum_pnl,
                Decimal::from(total_paths),
                "PathStatistics::from_outcomes/average_pnl",
            )?
        } else {
            dec!(0.0)
        };

        let median_pnl = if pnl_values.is_empty() {
            dec!(0.0)
        } else {
            let mid = pnl_values.len() / 2;
            let at = |i: usize| {
                pnl_values.get(i).copied().ok_or_else(|| {
                    SimulationError::invalid_parameters(
                        "PathStatistics::from_outcomes: median index out of range",
                    )
                })
            };
            if pnl_values.len().is_multiple_of(2) {
                let lower = mid.checked_sub(1).ok_or_else(|| {
                    SimulationError::invalid_parameters(
                        "PathStatistics::from_outcomes: even sample count with no lower median",
                    )
                })?;
                d_div(
                    d_add(at(lower)?, at(mid)?, "PathStatistics::from_outcomes/median")?,
                    dec!(2.0),
                    "PathStatistics::from_outcomes/median",
                )?
            } else {
                at(mid)?
            }
        };

        let variance = if total_paths > 1 {
            let mut sum_squared_diff = Decimal::ZERO;
            for &pnl in &pnl_values {
                let diff = d_sub(
                    pnl,
                    average_pnl,
                    "PathStatistics::from_outcomes/variance_diff",
                )?;
                sum_squared_diff = d_add(
                    sum_squared_diff,
                    d_mul(diff, diff, "PathStatistics::from_outcomes/variance_square")?,
                    "PathStatistics::from_outcomes/variance_sum",
                )?;
            }
            d_div(
                sum_squared_diff,
                Decimal::from(total_paths - 1),
                "PathStatistics::from_outcomes/variance",
            )?
        } else {
            dec!(0.0)
        };
        let std_dev_pnl = variance.sqrt().unwrap_or(dec!(0.0));

        let best_pnl = pnl_values.last().copied().unwrap_or(dec!(0.0));
        let worst_pnl = pnl_values.first().copied().unwrap_or(dec!(0.0));

        let win_rate = if total_paths > 0 {
            d_mul(
                div_unrounded(
                    Decimal::from(profitable_count),
                    Decimal::from(total_paths),
                    "PathStatistics::from_outcomes/win_rate",
                )?,
                dec!(100.0),
                "PathStatistics::from_outcomes/win_rate",
            )?
        } else {
            dec!(0.0)
        };

        let average_holding_period = if total_paths > 0 {
            let sum_holding = outcomes
                .iter()
                .try_fold(0usize, |acc, outcome| {
                    acc.checked_add(outcome.holding_period)
                })
                .ok_or_else(|| {
                    SimulationError::invalid_parameters(
                        "PathStatistics::from_outcomes: holding period total overflowed",
                    )
                })?;
            div_unrounded(
                Decimal::from(sum_holding),
                Decimal::from(total_paths),
                "PathStatistics::from_outcomes/average_holding_period",
            )?
        } else {
            dec!(0.0)
        };

        Ok(Self {
            total_paths,
            profitable_count,
            loss_count,
            average_pnl,
            median_pnl,
            std_dev_pnl,
            best_pnl,
            worst_pnl,
            win_rate,
            average_holding_period,
        })
    }
}

/// Evaluates one simulated price path.
///
/// Implementors decide what a path is worth; the simulation layer only
/// generates the paths and, through [`evaluate_paths`], drives the
/// evaluator over each of them. `Outcome` is usually [`PathOutcome`] so
/// the result can be summarised by [`PathStatistics`], but any type is
/// admissible for evaluators with their own reporting.
///
/// # Type Parameters
///
/// * `X` - Time-step type of the walk.
/// * `Y` - Price type of the walk.
pub trait PathEvaluator<X, Y>
where
    X: Copy + TryInto<Positive> + AddAssign + Display,
    Y: TryInto<Positive> + Display + Clone,
{
    /// What one evaluated path produces.
    type Outcome;

    /// Evaluates `walk` under `exit`.
    ///
    /// # Errors
    ///
    /// Implementation-defined; typically a pricing failure or an invalid
    /// step along the walk.
    fn evaluate_path(
        &self,
        walk: &RandomWalk<X, Y>,
        exit: &ExitPolicy,
    ) -> Result<Self::Outcome, SimulationError>;
}

/// Runs `evaluator` over every walk in `sim`, in order.
///
/// The result has one entry per walk. The first evaluation failure aborts
/// the run; partial results are not returned.
///
/// # Errors
///
/// Propagates the first error returned by
/// [`PathEvaluator::evaluate_path`].
#[must_use = "the outcomes are the only product of this call"]
pub fn evaluate_paths<X, Y, E>(
    evaluator: &E,
    sim: &Simulator<X, Y>,
    exit: &ExitPolicy,
) -> Result<Vec<E::Outcome>, SimulationError>
where
    X: Copy + TryInto<Positive> + AddAssign + Display,
    Y: TryInto<Positive> + Display + Clone,
    E: PathEvaluator<X, Y>,
{
    let mut outcomes = Vec::with_capacity(sim.len());
    for walk in sim {
        outcomes.push(evaluator.evaluate_path(walk, exit)?);
    }
    Ok(outcomes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExpirationDate;
    use crate::simulation::steps::Step;
    use crate::simulation::{WalkParams, WalkType, WalkTypeAble, generator_positive};
    use crate::utils::TimeFrame;
    use positive::pos_or_panic;

    fn outcome(
        pnl: Option<Decimal>,
        holding_period: usize,
        exit_reason: ExitPolicy,
    ) -> PathOutcome {
        PathOutcome {
            pnl,
            holding_period,
            exit_reason,
            hit_take_profit: false,
            hit_stop_loss: false,
            expired: false,
            max_premium: dec!(10.0),
            min_premium: dec!(5.0),
            avg_premium: dec!(7.5),
            expiration_premium: None,
        }
    }

    #[test]
    fn test_path_statistics_empty_set_all_zero() {
        let stats = PathStatistics::from_outcomes(&[]).unwrap();
        assert_eq!(stats, PathStatistics::default());
    }

    #[test]
    fn test_path_statistics_single_outcome_zero_std_dev() {
        let stats =
            PathStatistics::from_outcomes(&[outcome(Some(dec!(12.5)), 4, ExitPolicy::Expiration)])
                .unwrap();
        assert_eq!(stats.total_paths, 1);
        assert_eq!(stats.profitable_count, 1);
        assert_eq!(stats.loss_count, 0);
        assert_eq!(stats.average_pnl, dec!(12.5));
        assert_eq!(stats.median_pnl, dec!(12.5));
        assert_eq!(stats.std_dev_pnl, dec!(0.0));
        assert_eq!(stats.best_pnl, dec!(12.5));
        assert_eq!(stats.worst_pnl, dec!(12.5));
        assert_eq!(stats.win_rate, dec!(100.0));
        assert_eq!(stats.average_holding_period, dec!(4));
    }

    /// Hand-built set: P&L {50, -100, 25, 0}, holding {10, 20, 15, 5}.
    /// Sorted P&L: [-100, 0, 25, 50]; mean -6.25; median (0 + 25) / 2 =
    /// 12.5; sample variance (8789.0625 + 39.0625 + 976.5625 + 3164.0625)
    /// / 3 = 4322.9166...; win rate 2 / 4 = 50 %.
    #[test]
    fn test_path_statistics_hand_built_set_exact() {
        let outcomes = [
            outcome(Some(dec!(50.0)), 10, ExitPolicy::ProfitPercent(dec!(0.5))),
            outcome(Some(dec!(-100.0)), 20, ExitPolicy::LossPercent(dec!(1.0))),
            outcome(Some(dec!(25.0)), 15, ExitPolicy::Expiration),
            outcome(None, 5, ExitPolicy::Expiration),
        ];
        let stats = PathStatistics::from_outcomes(&outcomes).unwrap();

        assert_eq!(stats.total_paths, 4);
        assert_eq!(stats.profitable_count, 2);
        assert_eq!(stats.loss_count, 1);
        assert_eq!(stats.average_pnl, dec!(-6.25));
        assert_eq!(stats.median_pnl, dec!(12.5));
        assert_eq!(stats.best_pnl, dec!(50.0));
        assert_eq!(stats.worst_pnl, dec!(-100.0));
        assert_eq!(stats.win_rate, dec!(50.0));
        assert_eq!(stats.average_holding_period, dec!(12.5));

        let variance = d_div(dec!(12968.75), dec!(3), "test/variance").unwrap();
        assert_eq!(stats.std_dev_pnl, variance.sqrt().unwrap());
    }

    #[test]
    fn test_path_statistics_odd_count_median_is_middle_value() {
        let outcomes = [
            outcome(Some(dec!(3.0)), 1, ExitPolicy::Expiration),
            outcome(Some(dec!(-1.0)), 1, ExitPolicy::Expiration),
            outcome(Some(dec!(7.0)), 1, ExitPolicy::Expiration),
        ];
        let stats = PathStatistics::from_outcomes(&outcomes).unwrap();
        assert_eq!(stats.median_pnl, dec!(3.0));
        assert_eq!(stats.average_pnl, dec!(3.0));
        assert_eq!(stats.std_dev_pnl, dec!(4.0));
    }

    #[test]
    fn test_path_statistics_sum_overflow_is_reported() {
        let outcomes = [
            outcome(Some(Decimal::MAX), 1, ExitPolicy::Expiration),
            outcome(Some(Decimal::MAX), 1, ExitPolicy::Expiration),
        ];
        assert!(matches!(
            PathStatistics::from_outcomes(&outcomes),
            Err(SimulationError::Decimal(_))
        ));
    }

    #[test]
    fn test_div_unrounded_reports_division_by_zero() {
        assert!(matches!(
            div_unrounded(dec!(1), Decimal::ZERO, "test"),
            Err(SimulationError::InvalidParameters { .. })
        ));
    }

    /// A simulation-only evaluator: P&L is the move from the first to the
    /// last price, holding period the number of steps taken.
    struct LastMinusFirst;

    impl PathEvaluator<Positive, Positive> for LastMinusFirst {
        type Outcome = PathOutcome;

        fn evaluate_path(
            &self,
            walk: &RandomWalk<Positive, Positive>,
            _exit: &ExitPolicy,
        ) -> Result<PathOutcome, SimulationError> {
            let steps = walk.get_steps();
            let first = steps
                .first()
                .ok_or_else(|| SimulationError::invalid_parameters("empty walk"))?;
            let last = steps
                .last()
                .ok_or_else(|| SimulationError::invalid_parameters("empty walk"))?;
            let pnl = d_sub(
                last.y.positive()?.to_dec(),
                first.y.positive()?.to_dec(),
                "test",
            )?;
            Ok(PathOutcome {
                pnl: Some(pnl),
                holding_period: steps.len() - 1,
                exit_reason: ExitPolicy::Expiration,
                expired: true,
                ..PathOutcome::default()
            })
        }
    }

    #[derive(Clone)]
    struct TestWalker;
    impl WalkTypeAble<Positive, Positive> for TestWalker {}

    fn historical_params(prices: Vec<Positive>) -> WalkParams<Positive, Positive> {
        let init_step = Step::new(
            Positive::ONE,
            TimeFrame::Day,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            Positive::HUNDRED,
        );
        WalkParams {
            size: prices.len(),
            init_step,
            walker: Box::new(TestWalker),
            walk_type: WalkType::Historical {
                timeframe: TimeFrame::Day,
                prices,
                symbol: Some("TEST".to_string()),
            },
        }
    }

    #[test]
    fn test_evaluate_paths_simulation_only_fixture_summarises() {
        let params = historical_params(vec![
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(110.0),
        ]);
        let sim = Simulator::new("fixture".to_string(), 3, &params, generator_positive).unwrap();

        let outcomes = evaluate_paths(&LastMinusFirst, &sim, &ExitPolicy::Expiration).unwrap();
        assert_eq!(outcomes.len(), 3);
        for outcome in &outcomes {
            assert_eq!(outcome.pnl, Some(dec!(10.0)));
            assert_eq!(outcome.holding_period, 2);
            assert!(outcome.expired);
        }

        let stats = PathStatistics::from_outcomes(&outcomes).unwrap();
        assert_eq!(stats.total_paths, 3);
        assert_eq!(stats.profitable_count, 3);
        assert_eq!(stats.average_pnl, dec!(10.0));
        assert_eq!(stats.median_pnl, dec!(10.0));
        assert_eq!(stats.std_dev_pnl, dec!(0.0));
        assert_eq!(stats.win_rate, dec!(100.0));
        assert_eq!(stats.average_holding_period, dec!(2));
    }

    struct AlwaysFails;

    impl PathEvaluator<Positive, Positive> for AlwaysFails {
        type Outcome = ();

        fn evaluate_path(
            &self,
            _walk: &RandomWalk<Positive, Positive>,
            _exit: &ExitPolicy,
        ) -> Result<(), SimulationError> {
            Err(SimulationError::invalid_parameters("fixture failure"))
        }
    }

    #[test]
    fn test_evaluate_paths_propagates_first_error() {
        let params = historical_params(vec![Positive::HUNDRED, pos_or_panic!(101.0)]);
        let sim = Simulator::new("fixture".to_string(), 2, &params, generator_positive).unwrap();
        assert!(matches!(
            evaluate_paths(&AlwaysFails, &sim, &ExitPolicy::Expiration),
            Err(SimulationError::InvalidParameters { .. })
        ));
    }
}
