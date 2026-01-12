use super::base::{BreakEvenable, Positionable, StrategyType};
use crate::backtesting::results::{SimulationResult, SimulationStatsResult};

use crate::chains::OptionChain;
use crate::error::strategies::ProfitLossErrorKind;
use crate::error::{
    GreeksError, PricingError, ProbabilityError, SimulationError, StrategyError,
    position::{PositionError, PositionValidationErrorKind},
    probability::ProfitLossRangeErrorKind,
};
use crate::greeks::Greeks;
use crate::model::{
    ProfitLossRange,
    position::Position,
    types::{OptionBasicType, OptionStyle, OptionType, Side},
};
use crate::pnl::{PnL, PnLCalculator};
use crate::pricing::payoff::Profit;
use crate::simulation::simulator::Simulator;
use crate::simulation::{ExitPolicy, Simulate};
use crate::strategies::base::Optimizable;
use crate::strategies::delta_neutral::DeltaNeutrality;
use crate::strategies::probabilities::core::ProbabilityAnalysis;
use crate::strategies::probabilities::utils::VolatilityAdjustment;
use crate::strategies::utils::OptimizationCriteria;
use crate::strategies::{
    BasicAble, DeltaAdjustment, FindOptimalSide, Strategable, Strategies, StrategyConstructor,
    Validable,
};
use crate::utils::Len;
use crate::{ExpirationDate, Options, test_strategy_traits};
use chrono::Utc;
use num_traits::FromPrimitive;
use positive::Positive;
use pretty_simple_display::{DebugPretty, DisplaySimple};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::debug;
use utoipa::ToSchema;

pub(super) const LONG_PUT_DESCRIPTION: &str = "A Long Put is an options strategy where the trader purchases a put option, gaining the right (but not the obligation) to sell the underlying asset at the strike price until expiration. \
    This strategy requires an initial investment (the premium paid) and provides downside protection or profit potential if the underlying asset's price decreases. \
    The breakeven point is the strike price minus the premium paid. Long puts are commonly used as insurance against price declines or to express a bearish outlook.";

/// Represents a Long Put options trading strategy.
///
/// A Long Put strategy is used when a trader expects the price of the underlying asset
/// to decrease significantly. It involves purchasing a put option with the anticipation
/// of profiting as the underlying asset's price falls below the strike price of the option.
///
/// # Fields
///
/// * `name` - A unique name identifier for this specific instance of the Long Put strategy.
/// * `kind` - The type of strategy, identified specifically as `StrategyType::LongPut`.
/// * `description` - A detailed description of this particular instance of the Long Put strategy,
///   providing additional context or information.
/// * `break_even_points` - A vector of price points (`Positive`) where the strategy neither gains
///   nor loses money based on the underlying asset's movement.
/// * `long_put` - Represents the specific long put position within the strategy, detailing
///   the option contract being used.
#[derive(Clone, DebugPretty, DisplaySimple, Serialize, Deserialize, ToSchema)]
pub struct LongPut {
    /// Name identifier for this specific strategy instance
    pub name: String,
    /// Identifies this as a LongPut strategy type
    pub kind: StrategyType,
    /// Detailed description of this strategy instance
    pub description: String,
    /// Price points where the strategy neither makes nor loses money
    pub break_even_points: Vec<Positive>,
    /// The long put position
    pub(super) long_put: Position,
}

impl LongPut {
    /// Constructs a new instance of a `LongPut` strategy.
    ///
    /// This method initializes a `LongPut` strategy based on the given parameters,
    /// including details about the underlying asset, option specifications, and associated fees.
    ///
    /// # Parameters
    ///
    /// * `underlying_symbol` - Symbol of the underlying asset for the long put option.
    /// * `long_put_strike` - Strike price of the long put option. Must be a positive value.
    /// * `long_put_expiration` - Expiration date of the long put option.
    /// * `implied_volatility` - Implied volatility of the underlying asset. Must be a positive value.
    /// * `quantity` - Quantity of contracts. Must be a positive value.
    /// * `underlying_price` - Current price of the underlying asset. Must be a positive value.
    /// * `risk_free_rate` - Risk-free rate used for pricing the option (in decimal format).
    /// * `dividend_yield` - Dividend yield of the underlying asset. Must be a positive value.
    /// * `premium_long_put` - Premium cost of the long put option. Must be a positive value.
    /// * `open_fee_long_put` - Fee incurred when opening the long put position. Must be a positive value.
    /// * `close_fee_long_put` - Fee incurred when closing the long put position. Must be a positive value.
    ///
    /// # Returns
    ///
    /// A new instance of the `LongPut` strategy initialized with the provided parameters.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// * The `add_position` method fails, which could happen due to invalid configurations of the long put position.
    ///
    #[allow(clippy::too_many_arguments, dead_code)]
    fn new(
        underlying_symbol: String,
        long_put_strike: Positive,
        long_put_expiration: ExpirationDate,
        implied_volatility: Positive,
        quantity: Positive,
        underlying_price: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        premium_long_put: Positive,
        open_fee_long_put: Positive,
        close_fee_long_put: Positive,
    ) -> Self {
        let mut strategy = LongPut::default();

        let long_put_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol,
            long_put_strike,
            long_put_expiration,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Put,
            dividend_yield,
            None,
        );
        let long_put = Position::new(
            long_put_option,
            premium_long_put,
            Utc::now(),
            open_fee_long_put,
            close_fee_long_put,
            None,
            None,
        );
        strategy
            .add_position(&long_put)
            .expect("Invalid long put option");

        strategy
    }
}

impl BasicAble for LongPut {
    fn get_title(&self) -> String {
        let strategy_title = format!("{:?} Strategy: ", self.kind);
        let leg_titles: Vec<String> = [self.long_put.get_title()]
            .iter()
            .map(|leg| leg.to_string())
            .collect();

        if leg_titles.is_empty() {
            strategy_title
        } else {
            format!("{}\n\t{}", strategy_title, leg_titles.join("\n\t"))
        }
    }
    fn get_option_basic_type(&self) -> HashSet<OptionBasicType<'_>> {
        let mut hash_set = HashSet::new();
        let long_put = &self.long_put.option;

        hash_set.insert(OptionBasicType {
            option_style: &long_put.option_style,
            side: &long_put.side,
            strike_price: &long_put.strike_price,
            expiration_date: &long_put.expiration_date,
        });

        hash_set
    }
    fn get_implied_volatility(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let options = [(
            &self.long_put.option,
            &self.long_put.option.implied_volatility,
        )];

        options
            .into_iter()
            .map(|(option, iv)| {
                (
                    OptionBasicType {
                        option_style: &option.option_style,
                        side: &option.side,
                        strike_price: &option.strike_price,
                        expiration_date: &option.expiration_date,
                    },
                    iv,
                )
            })
            .collect()
    }
    fn get_quantity(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let options = [(&self.long_put.option, &self.long_put.option.quantity)];

        options
            .into_iter()
            .map(|(option, quantity)| {
                (
                    OptionBasicType {
                        option_style: &option.option_style,
                        side: &option.side,
                        strike_price: &option.strike_price,
                        expiration_date: &option.expiration_date,
                    },
                    quantity,
                )
            })
            .collect()
    }
    fn one_option(&self) -> &Options {
        self.long_put.one_option()
    }
    fn one_option_mut(&mut self) -> &mut Options {
        self.long_put.one_option_mut()
    }
    fn set_expiration_date(
        &mut self,
        expiration_date: ExpirationDate,
    ) -> Result<(), StrategyError> {
        self.long_put.option.expiration_date = expiration_date;
        Ok(())
    }
    fn set_underlying_price(&mut self, price: &Positive) -> Result<(), StrategyError> {
        self.long_put.option.underlying_price = *price;
        self.long_put.premium =
            Positive::new_decimal(self.long_put.option.calculate_price_black_scholes()?.abs())
                .unwrap_or(Positive::ZERO);
        Ok(())
    }
    fn set_implied_volatility(&mut self, volatility: &Positive) -> Result<(), StrategyError> {
        self.long_put.option.implied_volatility = *volatility;
        self.long_put.premium =
            Positive::new_decimal(self.long_put.option.calculate_price_black_scholes()?.abs())
                .unwrap_or(Positive::ZERO);
        Ok(())
    }
}

impl Validable for LongPut {
    fn validate(&self) -> bool {
        if !self.long_put.validate() {
            debug!("Long call is invalid");
            return false;
        }
        true
    }
}

impl BreakEvenable for LongPut {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        self.break_even_points.push(
            (self.long_put.option.strike_price
                + self.get_net_cost()? / self.long_put.option.quantity)
                .round_to(2),
        );

        Ok(())
    }
}

impl Strategies for LongPut {
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        let profit = self.calculate_profit_at(&self.long_put.option.strike_price)?;
        if profit >= Decimal::ZERO {
            Ok(Positive::new_decimal(profit)?)
        } else {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxProfitError {
                    reason: "Net premium received is negative".to_string(),
                },
            ))
        }
    }
    fn get_max_loss(&self) -> Result<Positive, StrategyError> {
        let loss = self.calculate_profit_at(&self.long_put.option.strike_price)?;
        if loss <= Decimal::ZERO {
            Ok(Positive::new_decimal(loss.abs()).unwrap_or(Positive::ZERO))
        } else {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxLossError {
                    reason: "Max loss is negative".to_string(),
                },
            ))
        }
    }
    fn get_profit_area(&self) -> Result<Decimal, StrategyError> {
        let high = self.get_max_profit().unwrap_or(Positive::ZERO);
        let base = self.long_put.option.strike_price - self.break_even_points[0];
        Ok(Decimal::from_f64(high.to_f64() * base.to_f64() / 200.0).unwrap_or(Decimal::ZERO))
    }
    fn get_profit_ratio(&self) -> Result<Decimal, StrategyError> {
        let max_profit = self.get_max_profit().unwrap_or(Positive::ZERO);
        let max_loss = self.get_max_loss().unwrap_or(Positive::ZERO);
        match (max_profit, max_loss) {
            (value, _) if value == Positive::ZERO => Ok(Decimal::ZERO),
            (_, value) if value == Positive::ZERO => Ok(Decimal::MAX),
            _ => Ok(
                Decimal::from_f64(max_profit.to_f64() / max_loss.to_f64() * 100.0)
                    .unwrap_or(Decimal::ZERO),
            ),
        }
    }
}

impl Profit for LongPut {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, PricingError> {
        let price = Some(price);
        self.long_put.pnl_at_expiration(&price)
    }
}

impl Positionable for LongPut {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match (position.option.option_style, position.option.side) {
            (OptionStyle::Put, Side::Long) => {
                self.long_put = position.clone();
                Ok(())
            }
            _ => Err(PositionError::invalid_position_style(
                position.option.option_style,
                "Position is a Put or Long, it is not valid for LongPut".to_string(),
            )),
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.long_put])
    }

    /// Gets mutable positions matching the specified criteria from the strategy.
    ///
    /// # Arguments
    /// * `option_style` - The style of the option (Put/Put)
    /// * `side` - The side of the position (Long/Long)
    /// * `strike` - The strike price of the option
    ///
    /// # Returns
    /// * `Ok(Vec<&mut Position>)` - A vector containing mutable references to matching positions
    /// * `Err(PositionError)` - If there was an error retrieving positions
    fn get_position(
        &mut self,
        option_style: &OptionStyle,
        side: &Side,
        strike: &Positive,
    ) -> Result<Vec<&mut Position>, PositionError> {
        match (side, option_style, strike) {
            (Side::Long, OptionStyle::Put, strike)
                if *strike == self.long_put.option.strike_price =>
            {
                Ok(vec![&mut self.long_put])
            }
            _ => Err(PositionError::invalid_position_type(
                *side,
                "Position not found".to_string(),
            )),
        }
    }

    /// Modifies an existing position in the strategy.
    ///
    /// # Arguments
    /// * `position` - The new position data to update
    ///
    /// # Returns
    /// * `Ok(())` if position was successfully modified
    /// * `Err(PositionError)` if position was not found or validation failed
    fn modify_position(&mut self, position: &Position) -> Result<(), PositionError> {
        if !position.validate() {
            return Err(PositionError::ValidationError(
                PositionValidationErrorKind::InvalidPosition {
                    reason: "Invalid position data".to_string(),
                },
            ));
        }

        match (
            &position.option.side,
            &position.option.option_style,
            &position.option.strike_price,
        ) {
            (Side::Long, OptionStyle::Put, strike)
                if *strike == self.long_put.option.strike_price =>
            {
                self.long_put = position.clone();
            }
            _ => {
                return Err(PositionError::invalid_position_type(
                    position.option.side,
                    "Position not found".to_string(),
                ));
            }
        }

        Ok(())
    }
}

impl StrategyConstructor for LongPut {
    fn get_strategy(_vec_positions: &[Position]) -> Result<Self, StrategyError> {
        todo!()
    }
}

impl Optimizable for LongPut {
    type Strategy = Self;

    fn find_optimal(
        &mut self,
        _option_chain: &OptionChain,
        _side: FindOptimalSide,
        _criteria: OptimizationCriteria,
    ) {
        todo!()
    }
}

impl ProbabilityAnalysis for LongPut {
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        // Long put is profitable when price falls below break-even
        let break_even = self.break_even_points.first().ok_or_else(|| {
            ProbabilityError::RangeError(ProfitLossRangeErrorKind::InvalidBreakEvenPoints {
                reason: "No break-even points found for long put".to_string(),
            })
        })?;

        let option = &self.long_put.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let mut profit_range = ProfitLossRange::new(None, Some(*break_even), Positive::ZERO)?;

        profit_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: option.implied_volatility,
                std_dev_adjustment: Positive::ZERO,
            }),
            None,
            expiration_date,
            Some(risk_free_rate),
        )?;

        Ok(vec![profit_range])
    }

    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        // Long put has losses when price stays above break-even
        let break_even = self.break_even_points.first().ok_or_else(|| {
            ProbabilityError::RangeError(ProfitLossRangeErrorKind::InvalidBreakEvenPoints {
                reason: "No break-even points found for long put".to_string(),
            })
        })?;

        let option = &self.long_put.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let mut loss_range = ProfitLossRange::new(Some(*break_even), None, Positive::ZERO)?;

        loss_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: option.implied_volatility,
                std_dev_adjustment: Positive::ZERO,
            }),
            None,
            expiration_date,
            Some(risk_free_rate),
        )?;

        Ok(vec![loss_range])
    }
}

impl Greeks for LongPut {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.long_put.option])
    }
}

impl DeltaNeutrality for LongPut {}

impl PnLCalculator for LongPut {
    fn calculate_pnl(
        &self,
        market_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, PricingError> {
        self.long_put
            .calculate_pnl(market_price, expiration_date, implied_volatility)
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<PnL, PricingError> {
        self.long_put.calculate_pnl_at_expiration(underlying_price)
    }

    fn adjustments_pnl(&self, _adjustment: &DeltaAdjustment) -> Result<PnL, PricingError> {
        // Single-leg strategies like LongPut don't typically require delta adjustments
        // as they are directional strategies. Delta adjustments are more relevant for
        // complex multi-leg strategies aiming for delta neutrality.
        Err("Delta adjustments are not applicable to single-leg LongPut strategy".into())
    }
}

impl<X, Y> Simulate<X, Y> for LongPut
where
    X: Copy + Into<Positive> + std::ops::AddAssign + std::fmt::Display,
    Y: Into<Positive> + std::fmt::Display + Clone,
{
    /// Simulates the Short Put strategy across multiple price paths.
    ///
    /// This method evaluates the strategy's performance by:
    /// 1. Iterating through each random walk in the simulator
    /// 2. Calculating the option premium at each step using Black-Scholes
    /// 3. Checking if the exit policy is triggered
    /// 4. Computing final P&L based on exit conditions or expiration
    ///
    /// # Parameters
    ///
    /// * `sim` - The simulator containing multiple random walks
    /// * `exit` - The exit policy defining when to close the position
    ///
    /// # Returns
    ///
    /// `Ok(Vec<PnL>)` - A vector of P&L results for each simulation
    /// `Err(Box<dyn Error>)` - If any calculation fails
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Black-Scholes pricing fails
    /// - P&L calculation fails
    /// - Invalid option parameters
    fn simulate(
        &self,
        sim: &Simulator<X, Y>,
        exit: ExitPolicy,
    ) -> Result<SimulationStatsResult, SimulationError> {
        use indicatif::{ProgressBar, ProgressStyle};
        use rust_decimal::MathematicalOps;
        use rust_decimal_macros::dec;

        let mut simulation_results = Vec::with_capacity(sim.len());
        let initial_premium = self.long_put.option.calculate_price_black_scholes()?.abs();

        // Create progress bar for simulations
        let progress_bar = ProgressBar::new(sim.len() as u64);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} simulations ({eta})",
                )
                .expect("Failed to set progress bar template")
                .progress_chars("#>-"),
        );

        for random_walk in sim.into_iter() {
            let mut max_premium = initial_premium;
            let mut min_premium = initial_premium;
            let mut premium_sum = initial_premium;
            let mut premium_count = 1;
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
                let mut current_option = self.long_put.option.clone();
                current_option.underlying_price = step.y.positive()?;
                current_option.expiration_date = ExpirationDate::Days(days_left);

                let current_premium = current_option.calculate_price_black_scholes()?.abs();
                let index = *step.x.index() as usize;

                // Track premium statistics
                max_premium = max_premium.max(current_premium);
                min_premium = min_premium.min(current_premium);
                premium_sum += current_premium;
                premium_count += 1;
                holding_period = index;

                // Check exit policy
                if let Some(reason) = crate::simulation::check_exit_policy(
                    &exit,
                    initial_premium,
                    current_premium,
                    index,
                    days_left,
                    current_option.underlying_price,
                    true, // is_long = true for Long Put
                ) {
                    exit_reason = reason;

                    // Check if it's take profit or stop loss
                    // For long: profit when current > initial, loss when current < initial
                    let pnl_percent = (current_premium - initial_premium) / initial_premium;
                    if pnl_percent >= dec!(0.5) {
                        hit_take_profit = true;
                    } else if pnl_percent <= dec!(-1.0) {
                        hit_stop_loss = true;
                    }

                    // Exit triggered - calculate P&L directly from premiums
                    // For long put: P&L = current_premium - initial_premium (we paid initial, sell at current)
                    // We use direct premium calculation instead of calculate_pnl() to avoid discrepancies
                    // from recalculating the initial premium
                    let pnl = PnL {
                        realized: Some(current_premium - initial_premium),
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
                let pnl = self.calculate_pnl_at_expiration(&final_price)?;

                // Calculate expiration premium
                let mut exp_option = self.long_put.option.clone();
                exp_option.underlying_price = final_price;
                exp_option.expiration_date = ExpirationDate::Days(Positive::new(0.001).unwrap());
                expiration_premium = Some(exp_option.calculate_price_black_scholes()?.abs());

                expired = true;
                exit_reason = ExitPolicy::Expiration;
                final_pnl = Some(pnl);
                holding_period = random_walk.get_steps().len() - 1;
            }

            let pnl = final_pnl.unwrap_or_default();
            let avg_premium = premium_sum / Decimal::from(premium_count);

            simulation_results.push(SimulationResult {
                simulation_count: 1,
                risk_metrics: None,
                final_equity_percentiles: std::collections::HashMap::new(),
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
            });

            // Update progress bar
            progress_bar.inc(1);
        }

        // Finish progress bar
        progress_bar.finish_with_message("Simulations completed!");

        // Calculate aggregate statistics
        let total_simulations = simulation_results.len();
        let mut pnl_values: Vec<Decimal> = simulation_results
            .iter()
            .map(|r| r.pnl.total_pnl().unwrap_or(dec!(0.0)))
            .collect();

        pnl_values.sort();

        let profitable_count = pnl_values.iter().filter(|&&pnl| pnl > dec!(0.0)).count();
        let loss_count = pnl_values.iter().filter(|&&pnl| pnl < dec!(0.0)).count();

        let sum_pnl: Decimal = pnl_values.iter().sum();
        let average_pnl = if total_simulations > 0 {
            sum_pnl / Decimal::from(total_simulations)
        } else {
            dec!(0.0)
        };

        let median_pnl = if !pnl_values.is_empty() {
            let mid = pnl_values.len() / 2;
            if pnl_values.len().is_multiple_of(2) {
                (pnl_values[mid - 1] + pnl_values[mid]) / dec!(2.0)
            } else {
                pnl_values[mid]
            }
        } else {
            dec!(0.0)
        };

        let variance = if total_simulations > 1 {
            let sum_squared_diff: Decimal = pnl_values
                .iter()
                .map(|&pnl| (pnl - average_pnl).powi(2))
                .sum();
            sum_squared_diff / Decimal::from(total_simulations - 1)
        } else {
            dec!(0.0)
        };

        let std_dev_pnl = variance.sqrt().unwrap_or(dec!(0.0));

        let best_pnl = pnl_values.last().copied().unwrap_or(dec!(0.0));
        let worst_pnl = pnl_values.first().copied().unwrap_or(dec!(0.0));

        let win_rate = if total_simulations > 0 {
            Decimal::from(profitable_count) / Decimal::from(total_simulations) * dec!(100.0)
        } else {
            dec!(0.0)
        };

        let average_holding_period = if total_simulations > 0 {
            let sum_holding: usize = simulation_results.iter().map(|r| r.holding_period).sum();
            Decimal::from(sum_holding) / Decimal::from(total_simulations)
        } else {
            dec!(0.0)
        };

        Ok(SimulationStatsResult {
            results: simulation_results,
            total_simulations,
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

impl Strategable for LongPut {}

test_strategy_traits!(LongPut, test_long_put_implementations);

#[cfg(test)]
mod tests_simulate {
    use super::*;
    use crate::chains::generator_positive;
    use positive::pos_or_panic;

    use crate::simulation::simulator::Simulator;
    use crate::simulation::steps::Step;
    use crate::simulation::{Simulate, WalkParams, WalkType, WalkTypeAble};
    use crate::utils::TimeFrame;
    use rust_decimal_macros::dec;

    struct TestWalker;
    impl WalkTypeAble<Positive, Positive> for TestWalker {}

    fn create_test_long_put() -> LongPut {
        LongPut::new(
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
    }

    fn create_walk_params(prices: Vec<Positive>) -> WalkParams<Positive, Positive> {
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
    fn test_simulate_profit_percent_exit() {
        let strategy = create_test_long_put();
        let prices = vec![
            Positive::HUNDRED,
            pos_or_panic!(95.0),
            pos_or_panic!(90.0),
            pos_or_panic!(85.0),
        ];
        let walk_params = create_walk_params(prices);
        let simulator = Simulator::new("Test".to_string(), 1, &walk_params, generator_positive);
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
        let simulator = Simulator::new("Test".to_string(), 1, &walk_params, generator_positive);
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
        let simulator = Simulator::new("Test".to_string(), 3, &walk_params, generator_positive);
        let results = strategy.simulate(&simulator, ExitPolicy::Expiration);
        assert!(results.is_ok(), "Simulate failed: {:?}", results.err());
        let stats = results.unwrap();
        assert!(stats.total_simulations >= 1);
        assert_eq!(stats.total_simulations, stats.results.len());
        assert!(stats.win_rate >= dec!(0.0) && stats.win_rate <= dec!(100.0));
    }
}
