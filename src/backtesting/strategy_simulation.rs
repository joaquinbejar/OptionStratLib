//! Simulation orchestration for the single-leg strategies.
//!
//! `LongCall`, `LongPut`, `ShortCall` and `ShortPut` are simulated the same
//! way: the leg is re-priced with Black-Scholes at every step of a random
//! walk, the exit policy is checked against the premium move, and the P&L
//! is taken either at the exit or at expiry. This module owns that loop
//! (backtesting composes strategies and simulation; neither may depend on
//! the other) and provides the `Simulate` implementations for the four
//! strategies. The per-strategy differences are captured by
//! [`SingleLegSimulation`]: the side of the leg and the fee adjustment
//! applied to every mark after the opening one.
//!
//! The progress bar drawn while the walks are evaluated stays with the
//! loop; M6-05 removes it from the library.

use crate::backtesting::results::{SimulationResult, SimulationStatsResult};
use crate::error::SimulationError;
use crate::model::decimal::{d_add, d_div, d_sub};
use crate::pnl::{PnL, PnLCalculator};
use crate::simulation::randomwalk::RandomWalk;
use crate::simulation::simulator::Simulator;
use crate::simulation::{ExitPolicy, PathEvaluator, Simulate, check_exit_policy};
use crate::strategies::base::Positionable;
use crate::strategies::{LongCall, LongPut, ShortCall, ShortPut, Strategies};
use crate::utils::Len;
use crate::{ExpirationDate, Options};
use indicatif::{ProgressBar, ProgressStyle};
use positive::Positive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::AddAssign;

/// Premium move, as a fraction of the opening premium, at which an exit
/// counts as a take-profit.
const TAKE_PROFIT_FRACTION: Decimal = dec!(0.5);
/// Premium move, as a fraction of the opening premium, at which an exit
/// counts as a stop-loss.
const STOP_LOSS_FRACTION: Decimal = dec!(-1.0);
/// Time to expiry, in days, used to mark the leg at expiration; a strictly
/// positive stand-in for zero so the closed form stays defined.
const EXPIRATION_MARK_DAYS: f64 = 0.001;

/// A strategy made of one option leg that can be marked along a price path.
///
/// Implemented for the four single-leg strategies. The leg is read through
/// [`Positionable::get_positions`], so the strategy needs no extra state;
/// the implementation only states the side and the fee adjustment.
pub trait SingleLegSimulation: PnLCalculator + Positionable {
    /// Whether the leg is long (bought) or short (sold). Drives the exit
    /// policy evaluation and the sign of the premium move.
    const IS_LONG: bool;

    /// Amount added to every premium mark after the opening one, in quote
    /// currency. Zero by default; `ShortPut` adds its fees.
    ///
    /// # Errors
    ///
    /// Returns [`SimulationError`] when the adjustment cannot be computed.
    fn mark_adjustment(&self) -> Result<Positive, SimulationError> {
        Ok(Positive::ZERO)
    }

    /// The single leg of the strategy.
    ///
    /// # Errors
    ///
    /// Returns [`SimulationError::InvalidParameters`] when the strategy
    /// does not hold exactly one position.
    fn simulated_option(&self) -> Result<&Options, SimulationError> {
        let positions = self.get_positions().map_err(|e| {
            SimulationError::invalid_parameters(&format!(
                "single-leg simulation cannot read the strategy positions: {e}"
            ))
        })?;
        match positions.as_slice() {
            [position] => Ok(&position.option),
            _ => Err(SimulationError::invalid_parameters(&format!(
                "single-leg simulation expects exactly one position, found {}",
                positions.len()
            ))),
        }
    }
}

impl SingleLegSimulation for LongCall {
    const IS_LONG: bool = true;
}

impl SingleLegSimulation for LongPut {
    const IS_LONG: bool = true;
}

impl SingleLegSimulation for ShortCall {
    const IS_LONG: bool = false;
}

impl SingleLegSimulation for ShortPut {
    const IS_LONG: bool = false;

    fn mark_adjustment(&self) -> Result<Positive, SimulationError> {
        Ok(self.get_fees()?)
    }
}

/// Divides without the default rounding of [`d_div`]; the average premium
/// has always been formed with the raw `Decimal` operator.
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

/// Evaluates one random walk for a single-leg strategy.
///
/// Holds the strategy and its opening premium (the Black-Scholes price of
/// the leg at construction), which every path is measured against.
#[derive(Debug)]
pub struct SingleLegPathEvaluator<'a, S> {
    strategy: &'a S,
    initial_premium: Decimal,
}

impl<'a, S> SingleLegPathEvaluator<'a, S>
where
    S: SingleLegSimulation,
{
    /// Prices the leg once to fix the opening premium.
    ///
    /// # Errors
    ///
    /// Returns the [`SimulationError`] raised when the strategy has no
    /// single leg or the closed-form price fails.
    pub fn new(strategy: &'a S) -> Result<Self, SimulationError> {
        let initial_premium = strategy
            .simulated_option()?
            .calculate_price_black_scholes()?
            .abs();
        Ok(Self {
            strategy,
            initial_premium,
        })
    }

    /// Opening premium every path is measured against, in quote currency.
    #[must_use]
    #[inline]
    pub fn initial_premium(&self) -> Decimal {
        self.initial_premium
    }
}

impl<S, X, Y> PathEvaluator<X, Y> for SingleLegPathEvaluator<'_, S>
where
    S: SingleLegSimulation,
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
    type Outcome = SimulationResult;

    fn evaluate_path(
        &self,
        random_walk: &RandomWalk<X, Y>,
        exit: &ExitPolicy,
    ) -> Result<SimulationResult, SimulationError> {
        let option = self.strategy.simulated_option()?;
        let initial_premium = self.initial_premium;
        let mark_adjustment = self.strategy.mark_adjustment()?.to_dec();

        let mut max_premium = initial_premium;
        let mut min_premium = initial_premium;
        let mut premium_sum = initial_premium;
        let mut premium_count: usize = 1;
        let mut hit_take_profit = false;
        let mut hit_stop_loss = false;
        let mut expired = false;
        let mut expiration_premium = None;
        let mut holding_period = 0;
        let mut exit_reason = ExitPolicy::Expiration;
        let mut final_pnl = None;

        // Iterate through the random walk
        for step in random_walk.get_steps().iter().skip(1) {
            let days_left = match step.x.days_left() {
                Ok(days) => days,
                Err(_) => {
                    expired = true;
                    break;
                }
            };

            // Calculate current option premium
            let mut current_option = option.clone();
            current_option.underlying_price = step.y.positive()?;
            current_option.expiration_date = ExpirationDate::Days(days_left);

            let current_premium = d_add(
                current_option.calculate_price_black_scholes()?.abs(),
                mark_adjustment,
                "backtesting::strategy_simulation/mark_adjustment",
            )?;
            let index = *step.x.index() as usize;

            // Track premium statistics
            max_premium = max_premium.max(current_premium);
            min_premium = min_premium.min(current_premium);
            premium_sum = d_add(
                premium_sum,
                current_premium,
                "backtesting::strategy_simulation/premium_sum",
            )?;
            premium_count = premium_count.checked_add(1).ok_or_else(|| {
                SimulationError::invalid_parameters(
                    "backtesting::strategy_simulation: premium count overflowed",
                )
            })?;
            holding_period = index;

            // Check exit policy
            if let Some(reason) = check_exit_policy(
                exit,
                initial_premium,
                current_premium,
                index,
                days_left,
                current_option.underlying_price,
                S::IS_LONG,
            ) {
                exit_reason = reason;

                // Check if it's take profit or stop loss.
                // Long: profit when current > initial; short: profit when
                // current < initial. A leg that opened at a zero premium has
                // no percentage to move by; the exit still fires, it just
                // does not classify as a take-profit or a stop-loss.
                let move_from_open = if S::IS_LONG {
                    d_sub(
                        current_premium,
                        initial_premium,
                        "backtesting::strategy_simulation/pnl_move",
                    )?
                } else {
                    d_sub(
                        initial_premium,
                        current_premium,
                        "backtesting::strategy_simulation/pnl_move",
                    )?
                };
                let pnl_percent = d_div(
                    move_from_open,
                    initial_premium,
                    "backtesting::strategy_simulation/pnl_percent",
                )
                .unwrap_or(Decimal::ZERO);
                if pnl_percent >= TAKE_PROFIT_FRACTION {
                    hit_take_profit = true;
                } else if pnl_percent <= STOP_LOSS_FRACTION {
                    hit_stop_loss = true;
                }

                // Exit triggered: the P&L is the premium move itself (paid
                // then sold for a long leg, received then bought back for a
                // short one). Recomputing the opening premium through
                // `calculate_pnl` would only introduce a discrepancy.
                let pnl = PnL {
                    realized: Some(move_from_open),
                    unrealized: None,
                    initial_costs: Positive::ZERO,
                    initial_income: Positive::ZERO,
                    date_time: chrono::Utc::now(),
                };
                final_pnl = Some(pnl);
                break;
            }
        }

        // If no exit triggered, calculate P&L at expiration
        if final_pnl.is_none()
            && let Some(last_step) = random_walk.last()
        {
            let final_price = last_step.y.positive()?;
            let pnl = self.strategy.calculate_pnl_at_expiration(&final_price)?;

            // Mark the leg at expiration (a very small time instead of zero)
            let mut exp_option = option.clone();
            exp_option.underlying_price = final_price;
            exp_option.expiration_date = ExpirationDate::Days(Positive::new(EXPIRATION_MARK_DAYS)?);
            expiration_premium = Some(exp_option.calculate_price_black_scholes()?.abs());

            expired = true;
            exit_reason = ExitPolicy::Expiration;
            final_pnl = Some(pnl);
            // `last()` returned a step, so the walk has at least one.
            holding_period = random_walk.len().checked_sub(1).ok_or_else(|| {
                SimulationError::invalid_parameters(
                    "backtesting::strategy_simulation: a walk with a last step has no length",
                )
            })?;
        }

        let pnl = final_pnl.unwrap_or_default();
        let avg_premium = div_unrounded(
            premium_sum,
            Decimal::from(premium_count),
            "backtesting::strategy_simulation/avg_premium",
        )?;

        Ok(SimulationResult {
            simulation_count: 1,
            risk_metrics: None,
            final_equity_percentiles: HashMap::new(),
            max_premium,
            min_premium,
            avg_premium,
            hit_take_profit,
            hit_stop_loss,
            expired,
            expiration_premium,
            pnl,
            holding_period,
            exit_reason,
        })
    }
}

/// Simulates a single-leg strategy across every walk of `sim`.
///
/// Prices the leg once for the opening premium, evaluates each walk with
/// [`SingleLegPathEvaluator`] (drawing a progress bar meanwhile) and
/// aggregates the results with [`SimulationStatsResult::from_results`].
///
/// # Errors
///
/// Returns a [`SimulationError`] when the strategy has no single leg, the
/// progress bar template is rejected, a Black-Scholes price or an
/// expiration P&L fails, a step cannot be read as a price, or the
/// aggregate statistics overflow.
#[must_use = "the simulation statistics are the only product of this call"]
#[tracing::instrument(level = "debug", skip(strategy, sim), fields(walks = sim.len()))]
pub fn simulate_single_leg<S, X, Y>(
    strategy: &S,
    sim: &Simulator<X, Y>,
    exit: ExitPolicy,
) -> Result<SimulationStatsResult, SimulationError>
where
    S: SingleLegSimulation,
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
    let evaluator = SingleLegPathEvaluator::new(strategy)?;
    let mut simulation_results = Vec::with_capacity(sim.len());

    // Create progress bar for simulations
    let walk_count = u64::try_from(sim.len()).map_err(|_| {
        SimulationError::invalid_parameters("simulator walk count does not fit a progress bar")
    })?;
    let progress_bar = ProgressBar::new(walk_count);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} simulations ({eta})",
            )
            .map_err(|e| {
                SimulationError::walk_error(&format!("Failed to set progress bar template: {e}"))
            })?
            .progress_chars("#>-"),
    );

    for random_walk in sim {
        simulation_results.push(evaluator.evaluate_path(random_walk, &exit)?);
        progress_bar.inc(1);
    }

    progress_bar.finish_with_message("Simulations completed!");

    SimulationStatsResult::from_results(simulation_results)
}

impl<X, Y> Simulate<X, Y> for LongCall
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
    fn simulate(
        &self,
        sim: &Simulator<X, Y>,
        exit: ExitPolicy,
    ) -> Result<SimulationStatsResult, SimulationError> {
        simulate_single_leg(self, sim, exit)
    }
}

impl<X, Y> Simulate<X, Y> for LongPut
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
    fn simulate(
        &self,
        sim: &Simulator<X, Y>,
        exit: ExitPolicy,
    ) -> Result<SimulationStatsResult, SimulationError> {
        simulate_single_leg(self, sim, exit)
    }
}

impl<X, Y> Simulate<X, Y> for ShortCall
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
    fn simulate(
        &self,
        sim: &Simulator<X, Y>,
        exit: ExitPolicy,
    ) -> Result<SimulationStatsResult, SimulationError> {
        simulate_single_leg(self, sim, exit)
    }
}

impl<X, Y> Simulate<X, Y> for ShortPut
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
    fn simulate(
        &self,
        sim: &Simulator<X, Y>,
        exit: ExitPolicy,
    ) -> Result<SimulationStatsResult, SimulationError> {
        simulate_single_leg(self, sim, exit)
    }
}

#[cfg(test)]
mod test_support {
    //! Fixtures shared by the four single-leg simulation test modules.
    //! Each strategy is built the way its constructor builds it: the
    //! default strategy plus one position with the test leg.
    use super::*;
    use crate::model::position::Position;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use crate::simulation::steps::Step;
    use crate::simulation::{WalkParams, WalkType, WalkTypeAble};
    use crate::utils::TimeFrame;
    use chrono::Utc;
    use positive::pos_or_panic;

    #[derive(Clone)]
    pub(super) struct TestWalker;
    impl WalkTypeAble<Positive, Positive> for TestWalker {}

    /// The test leg every fixture uses: ATM, 30 days, 20 % vol, 5.0
    /// premium, no fees.
    pub(super) fn test_leg(side: Side, style: OptionStyle) -> Position {
        let option = Options::new(
            OptionType::European,
            side,
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.20),
            Positive::ONE,
            Positive::HUNDRED,
            dec!(0.05),
            style,
            Positive::ZERO,
            None,
        );
        Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ZERO,
            Positive::ZERO,
            None,
            None,
        )
    }

    pub(super) fn create_test_long_call() -> LongCall {
        LongCall::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.20),
            Positive::ONE,
            Positive::HUNDRED,
            dec!(0.05),
            Positive::ZERO,
            pos_or_panic!(5.0),
            Positive::ZERO,
            Positive::ZERO,
        )
        .unwrap()
    }

    pub(super) fn create_test_long_put() -> LongPut {
        let mut strategy = LongPut::default();
        strategy
            .add_position(&test_leg(Side::Long, OptionStyle::Put))
            .unwrap();
        strategy
    }

    pub(super) fn create_test_short_call() -> ShortCall {
        let mut strategy = ShortCall::default();
        strategy
            .add_position(&test_leg(Side::Short, OptionStyle::Call))
            .unwrap();
        strategy
    }

    pub(super) fn create_test_short_put() -> ShortPut {
        let mut strategy = ShortPut::default();
        strategy
            .add_position(&test_leg(Side::Short, OptionStyle::Put))
            .unwrap();
        strategy
    }

    pub(super) fn create_walk_params(prices: Vec<Positive>) -> WalkParams<Positive, Positive> {
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
}

#[cfg(test)]
mod tests_single_leg_contract {
    use super::test_support::*;
    use super::*;
    use crate::simulation::generator_positive;
    use positive::pos_or_panic;
    use rust_decimal::MathematicalOps;

    #[test]
    fn test_single_leg_sides_and_adjustments() {
        let sides = [
            <LongCall as SingleLegSimulation>::IS_LONG,
            <LongPut as SingleLegSimulation>::IS_LONG,
            <ShortCall as SingleLegSimulation>::IS_LONG,
            <ShortPut as SingleLegSimulation>::IS_LONG,
        ];
        assert_eq!(sides, [true, true, false, false]);
        assert_eq!(
            create_test_long_call().mark_adjustment().unwrap(),
            Positive::ZERO
        );
        let short_put = create_test_short_put();
        assert_eq!(
            short_put.mark_adjustment().unwrap(),
            short_put.get_fees().unwrap()
        );
    }

    #[test]
    fn test_simulated_option_is_the_single_leg() {
        let strategy = create_test_long_call();
        let option = strategy.simulated_option().unwrap();
        assert_eq!(option.strike_price, Positive::HUNDRED);
        let evaluator = SingleLegPathEvaluator::new(&strategy).unwrap();
        assert_eq!(
            evaluator.initial_premium(),
            option.calculate_price_black_scholes().unwrap().abs()
        );
    }

    /// A `Positionable` with no legs (or with two) is not a single-leg
    /// strategy; the contract reports it instead of picking one.
    #[test]
    fn test_simulated_option_rejects_a_strategy_without_a_single_leg() {
        use crate::error::PricingError;
        use crate::model::position::Position;

        struct Legs(Vec<Position>);
        impl PnLCalculator for Legs {
            fn calculate_pnl(
                &self,
                _underlying_price: &Positive,
                _expiration_date: ExpirationDate,
                _implied_volatility: &Positive,
            ) -> Result<PnL, PricingError> {
                Ok(PnL::default())
            }
            fn calculate_pnl_at_expiration(
                &self,
                _underlying_price: &Positive,
            ) -> Result<PnL, PricingError> {
                Ok(PnL::default())
            }
        }
        impl Positionable for Legs {
            fn get_positions(&self) -> Result<Vec<&Position>, crate::error::PositionError> {
                Ok(self.0.iter().collect())
            }
        }
        impl SingleLegSimulation for Legs {
            const IS_LONG: bool = true;
        }

        assert!(matches!(
            Legs(Vec::new()).simulated_option(),
            Err(SimulationError::InvalidParameters { .. })
        ));
        let leg = create_test_long_call().get_positions().unwrap()[0].clone();
        assert!(matches!(
            Legs(vec![leg.clone(), leg]).simulated_option(),
            Err(SimulationError::InvalidParameters { .. })
        ));
        assert!(matches!(
            SingleLegPathEvaluator::new(&Legs(Vec::new())),
            Err(SimulationError::InvalidParameters { .. })
        ));
    }

    /// The aggregate of a run is `PathStatistics::from_outcomes` over the
    /// results: recompute it by hand from the per-run results.
    #[test]
    fn test_simulate_aggregate_matches_from_results() {
        let strategy = create_test_short_put();
        let walk_params = create_walk_params(vec![
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(110.0),
        ]);
        let simulator =
            Simulator::new("Test".to_string(), 3, &walk_params, generator_positive).unwrap();
        let stats = strategy
            .simulate(&simulator, ExitPolicy::Expiration)
            .unwrap();
        let recomputed = SimulationStatsResult::from_results(stats.results.clone()).unwrap();
        assert_eq!(stats.average_pnl, recomputed.average_pnl);
        assert_eq!(stats.median_pnl, recomputed.median_pnl);
        assert_eq!(stats.std_dev_pnl, recomputed.std_dev_pnl);
        assert_eq!(stats.win_rate, recomputed.win_rate);
        assert_eq!(
            stats.average_holding_period,
            recomputed.average_holding_period
        );
        assert_eq!(stats.std_dev_pnl, dec!(0.0).sqrt().unwrap());
    }
}

#[cfg(test)]
mod tests_simulate_long_call {
    use super::test_support::*;
    use super::*;
    use crate::simulation::generator_positive;
    use positive::pos_or_panic;

    #[test]
    fn test_simulate_profit_percent_exit() {
        let strategy = create_test_long_call();
        let prices = vec![
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(110.0),
            pos_or_panic!(115.0),
        ];
        let walk_params = create_walk_params(prices);
        let Ok(simulator) = Simulator::new("Test".to_string(), 1, &walk_params, generator_positive)
        else {
            panic!("simulator setup failed");
        };
        let results = strategy.simulate(&simulator, ExitPolicy::ProfitPercent(dec!(0.5)));
        assert!(results.is_ok());
        let stats = results.unwrap();
        assert_eq!(stats.total_simulations, 1);
    }

    #[test]
    fn test_simulate_expiration_exit() {
        let strategy = create_test_long_call();
        let prices = vec![
            Positive::HUNDRED,
            pos_or_panic!(101.0),
            pos_or_panic!(102.0),
        ];
        let walk_params = create_walk_params(prices);
        let Ok(simulator) = Simulator::new("Test".to_string(), 1, &walk_params, generator_positive)
        else {
            panic!("simulator setup failed");
        };
        let results = strategy.simulate(&simulator, ExitPolicy::Expiration);
        assert!(results.is_ok(), "Simulate failed: {:?}", results.err());
        let stats = results.unwrap();
        assert_eq!(stats.total_simulations, 1);
    }

    #[test]
    fn test_simulate_stats_aggregation() {
        let strategy = create_test_long_call();
        let prices = vec![
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(110.0),
        ];
        let walk_params = create_walk_params(prices);
        let Ok(simulator) = Simulator::new("Test".to_string(), 3, &walk_params, generator_positive)
        else {
            panic!("simulator setup failed");
        };
        let results = strategy.simulate(&simulator, ExitPolicy::Expiration);
        assert!(results.is_ok(), "Simulate failed: {:?}", results.err());
        let stats = results.unwrap();
        assert!(stats.total_simulations >= 1);
        assert_eq!(stats.total_simulations, stats.results.len());
        assert!(stats.win_rate >= dec!(0.0) && stats.win_rate <= dec!(100.0));
    }
}

#[cfg(test)]
mod tests_simulate_long_put {
    use super::test_support::*;
    use super::*;
    use crate::simulation::generator_positive;
    use positive::pos_or_panic;

    #[test]
    fn test_simulate_profit_percent_exit() {
        let strategy = create_test_long_put();
        let prices = vec![
            Positive::HUNDRED,
            pos_or_panic!(95.0),
            pos_or_panic!(90.0),
            pos_or_panic!(85.0),
        ];
        let walk_params = create_walk_params(prices);
        let Ok(simulator) = Simulator::new("Test".to_string(), 1, &walk_params, generator_positive)
        else {
            panic!("simulator setup failed");
        };
        let results = strategy.simulate(&simulator, ExitPolicy::ProfitPercent(dec!(0.5)));
        assert!(results.is_ok());
        let stats = results.unwrap();
        assert_eq!(stats.total_simulations, 1);
    }

    #[test]
    fn test_simulate_expiration_exit() {
        let strategy = create_test_long_put();
        let prices = vec![Positive::HUNDRED, pos_or_panic!(99.0), pos_or_panic!(98.0)];
        let walk_params = create_walk_params(prices);
        let Ok(simulator) = Simulator::new("Test".to_string(), 1, &walk_params, generator_positive)
        else {
            panic!("simulator setup failed");
        };
        let results = strategy.simulate(&simulator, ExitPolicy::Expiration);
        assert!(results.is_ok(), "Simulate failed: {:?}", results.err());
        let stats = results.unwrap();
        assert_eq!(stats.total_simulations, 1);
    }

    #[test]
    fn test_simulate_stats_aggregation() {
        let strategy = create_test_long_put();
        let prices = vec![Positive::HUNDRED, pos_or_panic!(95.0), pos_or_panic!(90.0)];
        let walk_params = create_walk_params(prices);
        let Ok(simulator) = Simulator::new("Test".to_string(), 3, &walk_params, generator_positive)
        else {
            panic!("simulator setup failed");
        };
        let results = strategy.simulate(&simulator, ExitPolicy::Expiration);
        assert!(results.is_ok(), "Simulate failed: {:?}", results.err());
        let stats = results.unwrap();
        assert!(stats.total_simulations >= 1);
        assert_eq!(stats.total_simulations, stats.results.len());
        assert!(stats.win_rate >= dec!(0.0) && stats.win_rate <= dec!(100.0));
    }
}

#[cfg(test)]
mod tests_simulate_short_call {
    use super::test_support::*;
    use super::*;
    use crate::simulation::generator_positive;
    use positive::pos_or_panic;

    #[test]
    fn test_simulate_profit_percent_exit() {
        let strategy = create_test_short_call();
        let prices = vec![
            Positive::HUNDRED,
            pos_or_panic!(95.0),
            pos_or_panic!(90.0),
            pos_or_panic!(85.0),
        ];
        let walk_params = create_walk_params(prices);
        let Ok(simulator) = Simulator::new("Test".to_string(), 1, &walk_params, generator_positive)
        else {
            panic!("simulator setup failed");
        };
        let results = strategy.simulate(&simulator, ExitPolicy::ProfitPercent(dec!(0.5)));
        assert!(results.is_ok());
        let stats = results.unwrap();
        assert_eq!(stats.total_simulations, 1);
    }

    #[test]
    fn test_simulate_expiration_exit() {
        let strategy = create_test_short_call();
        let prices = vec![Positive::HUNDRED, pos_or_panic!(99.0), pos_or_panic!(98.0)];
        let walk_params = create_walk_params(prices);
        let Ok(simulator) = Simulator::new("Test".to_string(), 1, &walk_params, generator_positive)
        else {
            panic!("simulator setup failed");
        };
        let results = strategy.simulate(&simulator, ExitPolicy::Expiration);
        assert!(results.is_ok(), "Simulate failed: {:?}", results.err());
        let stats = results.unwrap();
        assert_eq!(stats.total_simulations, 1);
    }

    #[test]
    fn test_simulate_stats_aggregation() {
        let strategy = create_test_short_call();
        let prices = vec![Positive::HUNDRED, pos_or_panic!(95.0), pos_or_panic!(90.0)];
        let walk_params = create_walk_params(prices);
        let Ok(simulator) = Simulator::new("Test".to_string(), 3, &walk_params, generator_positive)
        else {
            panic!("simulator setup failed");
        };
        let results = strategy.simulate(&simulator, ExitPolicy::Expiration);
        assert!(results.is_ok(), "Simulate failed: {:?}", results.err());
        let stats = results.unwrap();
        assert!(stats.total_simulations >= 1);
        assert_eq!(stats.total_simulations, stats.results.len());
        assert!(stats.win_rate >= dec!(0.0) && stats.win_rate <= dec!(100.0));
    }
}

#[cfg(test)]
mod tests_simulate_short_put {
    use super::test_support::*;
    use super::*;
    use crate::simulation::generator_positive;
    use positive::pos_or_panic;

    #[test]
    fn test_simulate_profit_percent_exit() {
        let strategy = create_test_short_put();
        let prices = vec![
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(110.0),
            pos_or_panic!(115.0),
        ];

        let walk_params = create_walk_params(prices);
        let Ok(simulator) = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        ) else {
            panic!("simulator setup failed");
        };

        let exit_policy = ExitPolicy::ProfitPercent(dec!(0.5));
        let results = strategy.simulate(&simulator, exit_policy);

        assert!(results.is_ok());
        let stats = results.unwrap();
        assert_eq!(stats.total_simulations, 1);
        assert!(!stats.results.is_empty());
    }

    #[test]
    fn test_simulate_loss_percent_exit() {
        let strategy = create_test_short_put();
        let prices = vec![
            Positive::HUNDRED,
            pos_or_panic!(95.0),
            pos_or_panic!(90.0),
            pos_or_panic!(85.0),
        ];

        let walk_params = create_walk_params(prices);
        let Ok(simulator) = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        ) else {
            panic!("simulator setup failed");
        };

        let exit_policy = ExitPolicy::LossPercent(dec!(1.0));
        let results = strategy.simulate(&simulator, exit_policy);

        assert!(results.is_ok());
        let stats = results.unwrap();
        assert_eq!(stats.total_simulations, 1);
        assert!(!stats.results.is_empty());
    }

    #[test]
    fn test_simulate_expiration_exit() {
        let strategy = create_test_short_put();
        let prices = vec![
            Positive::HUNDRED,
            pos_or_panic!(101.0),
            pos_or_panic!(102.0),
            pos_or_panic!(103.0),
        ];

        let walk_params = create_walk_params(prices);
        let Ok(simulator) = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        ) else {
            panic!("simulator setup failed");
        };

        let exit_policy = ExitPolicy::Expiration;
        let results = strategy.simulate(&simulator, exit_policy);

        assert!(results.is_ok(), "Simulate failed: {:?}", results.err());
        let stats = results.unwrap();
        assert_eq!(stats.total_simulations, 1);
        assert!(!stats.results.is_empty());
    }

    #[test]
    fn test_simulate_or_exit_policy() {
        let strategy = create_test_short_put();
        let prices = vec![
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(110.0),
            pos_or_panic!(115.0),
        ];

        let walk_params = create_walk_params(prices);
        let Ok(simulator) = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        ) else {
            panic!("simulator setup failed");
        };

        let exit_policy = ExitPolicy::Or(vec![
            ExitPolicy::ProfitPercent(dec!(0.5)),
            ExitPolicy::UnderlyingAbove(pos_or_panic!(112.0)),
        ]);
        let results = strategy.simulate(&simulator, exit_policy);

        assert!(results.is_ok());
        let stats = results.unwrap();
        assert_eq!(stats.total_simulations, 1);
        assert!(!stats.results.is_empty());
    }

    #[test]
    fn test_simulate_stats_aggregation() {
        let strategy = create_test_short_put();
        let prices = vec![
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(110.0),
        ];

        let walk_params = create_walk_params(prices);
        let Ok(simulator) = Simulator::new(
            "Test Simulator".to_string(),
            5,
            &walk_params,
            generator_positive,
        ) else {
            panic!("simulator setup failed");
        };

        let exit_policy = ExitPolicy::Expiration;
        let results = strategy.simulate(&simulator, exit_policy);

        assert!(results.is_ok(), "Simulate failed: {:?}", results.err());
        let stats = results.unwrap();

        assert!(stats.total_simulations >= 1);
        assert_eq!(stats.total_simulations, stats.results.len());
        assert!(stats.win_rate >= dec!(0.0) && stats.win_rate <= dec!(100.0));
        assert!(stats.average_holding_period >= dec!(0.0));
    }

    #[test]
    fn test_simulate_time_steps_exit() {
        let strategy = create_test_short_put();
        let prices = vec![
            Positive::HUNDRED,
            pos_or_panic!(101.0),
            pos_or_panic!(102.0),
            pos_or_panic!(103.0),
            pos_or_panic!(104.0),
        ];

        let walk_params = create_walk_params(prices);
        let Ok(simulator) = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        ) else {
            panic!("simulator setup failed");
        };

        let exit_policy = ExitPolicy::TimeSteps(2);
        let results = strategy.simulate(&simulator, exit_policy);

        assert!(results.is_ok());
        let stats = results.unwrap();
        assert_eq!(stats.total_simulations, 1);
        assert!(!stats.results.is_empty());
    }

    #[test]
    fn test_simulate_underlying_below_exit() {
        let strategy = create_test_short_put();
        let prices = vec![Positive::HUNDRED, pos_or_panic!(95.0), pos_or_panic!(90.0)];

        let walk_params = create_walk_params(prices);
        let Ok(simulator) = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        ) else {
            panic!("simulator setup failed");
        };

        let exit_policy = ExitPolicy::UnderlyingBelow(pos_or_panic!(92.0));
        let results = strategy.simulate(&simulator, exit_policy);

        assert!(results.is_ok());
        let stats = results.unwrap();
        assert_eq!(stats.total_simulations, 1);
    }

    #[test]
    fn test_simulate_and_exit_policy() {
        let strategy = create_test_short_put();
        let prices = vec![
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(110.0),
            pos_or_panic!(115.0),
        ];

        let walk_params = create_walk_params(prices);
        let Ok(simulator) = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        ) else {
            panic!("simulator setup failed");
        };

        let exit_policy = ExitPolicy::And(vec![
            ExitPolicy::TimeSteps(2),
            ExitPolicy::UnderlyingAbove(pos_or_panic!(105.0)),
        ]);
        let results = strategy.simulate(&simulator, exit_policy);

        assert!(results.is_ok());
        let stats = results.unwrap();
        assert_eq!(stats.total_simulations, 1);
    }

    #[test]
    fn test_simulate_multiple_simulations() {
        let strategy = create_test_short_put();
        let prices = vec![
            Positive::HUNDRED,
            pos_or_panic!(102.0),
            pos_or_panic!(104.0),
        ];

        let walk_params = create_walk_params(prices);
        let Ok(simulator) = Simulator::new(
            "Test Simulator".to_string(),
            10,
            &walk_params,
            generator_positive,
        ) else {
            panic!("simulator setup failed");
        };

        let exit_policy = ExitPolicy::Expiration;
        let results = strategy.simulate(&simulator, exit_policy);

        assert!(results.is_ok());
        let stats = results.unwrap();
        assert_eq!(stats.total_simulations, 10);
        assert_eq!(stats.results.len(), 10);
        assert!(stats.profitable_count + stats.loss_count <= 10);
    }

    #[test]
    fn test_simulate_stats_calculations() {
        let strategy = create_test_short_put();
        let prices = vec![
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(110.0),
        ];

        let walk_params = create_walk_params(prices);
        let Ok(simulator) = Simulator::new(
            "Test Simulator".to_string(),
            3,
            &walk_params,
            generator_positive,
        ) else {
            panic!("simulator setup failed");
        };

        let exit_policy = ExitPolicy::Expiration;
        let results = strategy.simulate(&simulator, exit_policy);

        assert!(results.is_ok());
        let stats = results.unwrap();

        // Verify stats are calculated
        assert!(stats.average_pnl.abs() >= dec!(0.0));
        assert!(stats.median_pnl.abs() >= dec!(0.0));
        assert!(stats.std_dev_pnl >= dec!(0.0));
        assert!(stats.best_pnl >= stats.worst_pnl);
    }

    #[test]
    fn test_simulate_expiration_path() {
        let strategy = create_test_short_put();
        // Price stays stable, should expire
        let prices = vec![
            Positive::HUNDRED,
            pos_or_panic!(100.5),
            pos_or_panic!(101.0),
            pos_or_panic!(100.8),
        ];

        let walk_params = create_walk_params(prices);
        let Ok(simulator) = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        ) else {
            panic!("simulator setup failed");
        };

        let exit_policy = ExitPolicy::Expiration;
        let results = strategy.simulate(&simulator, exit_policy);

        assert!(results.is_ok());
        let stats = results.unwrap();
        assert_eq!(stats.total_simulations, 1);
        assert!(!stats.results.is_empty());

        // Verify expiration was triggered
        let result = &stats.results[0];
        assert!(result.expired);
        assert!(result.expiration_premium.is_some());
    }

    #[test]
    fn test_simulate_profit_target_hit() {
        let strategy = create_test_short_put();
        // Price rises significantly to trigger profit
        let prices = vec![
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            pos_or_panic!(120.0),
        ];

        let walk_params = create_walk_params(prices);
        let Ok(simulator) = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        ) else {
            panic!("simulator setup failed");
        };

        let exit_policy = ExitPolicy::ProfitPercent(dec!(0.5));
        let results = strategy.simulate(&simulator, exit_policy);

        assert!(results.is_ok());
        let stats = results.unwrap();
        assert_eq!(stats.total_simulations, 1);

        let result = &stats.results[0];
        assert!(result.hit_take_profit || result.expired);
    }

    #[test]
    fn test_simulate_stop_loss_hit() {
        let strategy = create_test_short_put();
        // Price drops significantly to trigger stop loss
        let prices = vec![Positive::HUNDRED, pos_or_panic!(85.0), pos_or_panic!(70.0)];

        let walk_params = create_walk_params(prices);
        let Ok(simulator) = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        ) else {
            panic!("simulator setup failed");
        };

        let exit_policy = ExitPolicy::LossPercent(dec!(1.0));
        let results = strategy.simulate(&simulator, exit_policy);

        assert!(results.is_ok());
        let stats = results.unwrap();
        assert_eq!(stats.total_simulations, 1);

        let result = &stats.results[0];
        assert!(result.hit_stop_loss || result.expired);
    }

    #[test]
    fn test_simulate_premium_tracking() {
        let strategy = create_test_short_put();
        let prices = vec![
            Positive::HUNDRED,
            pos_or_panic!(95.0),
            pos_or_panic!(105.0),
            Positive::HUNDRED,
        ];

        let walk_params = create_walk_params(prices);
        let Ok(simulator) = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        ) else {
            panic!("simulator setup failed");
        };

        let exit_policy = ExitPolicy::Expiration;
        let results = strategy.simulate(&simulator, exit_policy);

        assert!(results.is_ok());
        let stats = results.unwrap();

        let result = &stats.results[0];
        // Verify premium tracking
        assert!(result.max_premium >= result.min_premium);
        assert!(result.avg_premium >= result.min_premium);
        assert!(result.avg_premium <= result.max_premium);
    }

    #[test]
    fn test_simulate_holding_period() {
        let strategy = create_test_short_put();
        let prices = vec![
            Positive::HUNDRED,
            pos_or_panic!(101.0),
            pos_or_panic!(102.0),
        ];
        let price_count = prices.len();

        let walk_params = create_walk_params(prices);
        let Ok(simulator) = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        ) else {
            panic!("simulator setup failed");
        };

        let exit_policy = ExitPolicy::TimeSteps(1);
        let results = strategy.simulate(&simulator, exit_policy);

        assert!(results.is_ok());
        let stats = results.unwrap();

        let result = &stats.results[0];
        assert!(result.holding_period > 0);
        assert!(result.holding_period <= price_count);
    }

    #[test]
    fn test_simulate_mixed_results() {
        let strategy = create_test_short_put();
        let prices = vec![Positive::HUNDRED, pos_or_panic!(102.0), pos_or_panic!(98.0)];

        let walk_params = create_walk_params(prices);
        let Ok(simulator) = Simulator::new(
            "Test Simulator".to_string(),
            5,
            &walk_params,
            generator_positive,
        ) else {
            panic!("simulator setup failed");
        };

        let exit_policy = ExitPolicy::Or(vec![
            ExitPolicy::ProfitPercent(dec!(0.5)),
            ExitPolicy::LossPercent(dec!(1.0)),
        ]);
        let results = strategy.simulate(&simulator, exit_policy);

        assert!(results.is_ok());
        let stats = results.unwrap();

        // Verify we have results
        assert_eq!(stats.total_simulations, 5);
        assert_eq!(stats.results.len(), 5);

        // Verify win rate is calculated
        assert!(stats.win_rate >= dec!(0.0));
        assert!(stats.win_rate <= dec!(100.0));
    }
}
