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
    BasicAble, DeltaAdjustment, Strategable, Strategies, StrategyConstructor, Validable,
};
use crate::utils::Len;
use crate::{ExpirationDate, Options, Positive};
use chrono::Utc;
use pretty_simple_display::{DebugPretty, DisplaySimple};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::debug;
use utoipa::ToSchema;

pub(super) const LONG_CALL_DESCRIPTION: &str = "A Long Call is an options strategy where the trader buys a call option, acquiring the right (but not the obligation) to purchase the underlying asset at the strike price until expiration. \
    This strategy involves an upfront cost (the premium paid) and offers unlimited profit potential if the underlying asset's price increases significantly. \
    The breakeven point is the strike price plus the premium paid. Long calls are typically used to gain leveraged exposure to potential price increases with defined risk.";

/// Represents a Long Call strategy in options trading.
///
/// A Long Call is an options strategy where an investor purchases call options
/// with the expectation that the underlying asset's price will rise above the
/// strike price before expiration, allowing them to profit.
///
/// # Fields
/// * `name` - A unique identifier for this specific instance of the Long Call strategy.
/// * `kind` - The type of strategy, identified as a `LongCall` within the `StrategyType` enumeration.
/// * `description` - A detailed explanation or notes about this particular Long Call instance.
/// * `break_even_points` - A collection of price levels (as a vector of positive values) where the strategy reaches
///   its break-even — meaning no profit or loss occurs at these points.
/// * `long_call` - The position details representing the long call option, specifying the strike price,
///   premium, and quantity involved. This field is private within the module (`pub(super)` access level).
///
/// # Notes
/// This structure leverages the `Clone`, `Debug`, `Serialize`, and `Deserialize` traits for ease of duplication,
/// debugging, and storage/transfer as structured data.
///
#[derive(Clone, DebugPretty, DisplaySimple, Serialize, Deserialize, ToSchema)]
pub struct LongCall {
    /// Name identifier for this specific strategy instance
    pub name: String,
    /// Identifies this as a LongCall strategy type
    pub kind: StrategyType,
    /// Detailed description of this strategy instance
    pub description: String,
    /// Price points where the strategy neither makes nor loses money
    pub break_even_points: Vec<Positive>,
    /// The long call position
    pub(super) long_call: Position,
}

impl LongCall {
    /// Creates a new instance of a `LongCall` strategy with the provided parameters.
    ///
    /// The `new` function initializes a `LongCall` strategy for the given underlying symbol and options parameters.
    /// It sets up a long call position by creating an `Options` object and encapsulating it in a `Position` object,
    /// which includes fees and premiums associated with the long call position.
    ///
    /// # Parameters
    /// - `underlying_symbol`: The symbol of the underlying asset (e.g., a stock ticker symbol) as a `String`.
    /// - `long_call_strike`: The strike price of the long call option, represented as a `Positive` value.
    /// - `long_call_expiration`: The expiration date of the long call option, represented as an `ExpirationDate`.
    /// - `implied_volatility`: The implied volatility of the option, represented as a `Positive` value.
    /// - `quantity`: The quantity of options to include in the position, represented as a `Positive` value.
    /// - `underlying_price`: The current price of the underlying asset, represented as a `Positive` value.
    /// - `risk_free_rate`: The risk-free interest rate used for option pricing, as a `Decimal`.
    /// - `dividend_yield`: The yield of any dividends associated with the underlying asset, as a `Positive` value.
    /// - `premium_long_call`: The premium paid for the long call option, as a `Positive` value.
    /// - `open_fee_long_call`: The fee associated with opening the long call position, as a `Positive` value.
    /// - `close_fee_long_call`: The fee associated with closing the long call position, as a `Positive` value.
    ///
    /// # Returns
    /// An initialized instance of `LongCall` strategy configured with the provided parameters.
    ///
    /// # Panics
    /// - Panics if adding the long call position to the strategy fails.
    ///   This typically occurs if the created long call option is invalid.
    ///
    /// # Notes
    /// - The function relies on creating a default `LongCall` instance and then populating it with positions.
    /// - Uses the `Options` and `Position` structures to model and manage the long call position.
    /// - Assumes the current time (_via `Utc::now()`) when opening the long call position for tracking purposes.
    #[allow(clippy::too_many_arguments, dead_code)]
    pub fn new(
        underlying_symbol: String,
        long_call_strike: Positive,
        long_call_expiration: ExpirationDate,
        implied_volatility: Positive,
        quantity: Positive,
        underlying_price: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        premium_long_call: Positive,
        open_fee_long_call: Positive,
        close_fee_long_call: Positive,
    ) -> Self {
        let mut strategy = LongCall::default();

        let long_call_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol,
            long_call_strike,
            long_call_expiration,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        let long_call = Position::new(
            long_call_option,
            premium_long_call,
            Utc::now(),
            open_fee_long_call,
            close_fee_long_call,
            None,
            None,
        );
        strategy
            .add_position(&long_call.clone())
            .expect("Invalid long call option");

        strategy
    }
}

impl BasicAble for LongCall {
    fn get_title(&self) -> String {
        let strategy_title = format!("{:?} Strategy: ", self.kind);
        let leg_titles: Vec<String> = [self.long_call.get_title()]
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
        let long_call = &self.long_call.option;

        hash_set.insert(OptionBasicType {
            option_style: &long_call.option_style,
            side: &long_call.side,
            strike_price: &long_call.strike_price,
            expiration_date: &long_call.expiration_date,
        });

        hash_set
    }
    fn get_implied_volatility(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let options = [(
            &self.long_call.option,
            &self.long_call.option.implied_volatility,
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
        let options = [(&self.long_call.option, &self.long_call.option.quantity)];

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
        self.long_call.one_option()
    }
    fn one_option_mut(&mut self) -> &mut Options {
        self.long_call.one_option_mut()
    }
    fn set_expiration_date(
        &mut self,
        expiration_date: ExpirationDate,
    ) -> Result<(), StrategyError> {
        self.long_call.option.expiration_date = expiration_date;
        Ok(())
    }
    fn set_underlying_price(&mut self, price: &Positive) -> Result<(), StrategyError> {
        self.long_call.option.underlying_price = *price;
        self.long_call.premium =
            Positive::from(self.long_call.option.calculate_price_black_scholes()?.abs());
        Ok(())
    }
    fn set_implied_volatility(&mut self, volatility: &Positive) -> Result<(), StrategyError> {
        self.long_call.option.implied_volatility = *volatility;
        self.long_call.premium =
            Positive(self.long_call.option.calculate_price_black_scholes()?.abs());
        Ok(())
    }
}

impl Validable for LongCall {
    fn validate(&self) -> bool {
        if !self.long_call.validate() {
            debug!("Long call is invalid");
            return false;
        }
        true
    }
}

impl BreakEvenable for LongCall {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        self.break_even_points.push(
            (self.long_call.option.strike_price
                + self.get_net_cost()? / self.long_call.option.quantity)
                .round_to(2),
        );

        Ok(())
    }
}

impl Strategies for LongCall {
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        let profit = self.calculate_profit_at(&self.long_call.option.strike_price)?;
        if profit >= Decimal::ZERO {
            Ok(profit.into())
        } else {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxProfitError {
                    reason: "Net premium received is negative".to_string(),
                },
            ))
        }
    }
    fn get_max_loss(&self) -> Result<Positive, StrategyError> {
        let loss = self.calculate_profit_at(&self.long_call.option.strike_price)?;
        if loss <= Decimal::ZERO {
            Ok(loss.abs().into())
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
        let base = self.long_call.option.strike_price - self.break_even_points[0];
        Ok((high * base / 200.0).into())
    }
    fn get_profit_ratio(&self) -> Result<Decimal, StrategyError> {
        let max_profit = self.get_max_profit().unwrap_or(Positive::ZERO);
        let max_loss = self.get_max_loss().unwrap_or(Positive::ZERO);
        match (max_profit, max_loss) {
            (value, _) if value == Positive::ZERO => Ok(Decimal::ZERO),
            (_, value) if value == Positive::ZERO => Ok(Decimal::MAX),
            _ => Ok((max_profit / max_loss * 100.0).into()),
        }
    }
}

impl Profit for LongCall {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, PricingError> {
        let price = Some(price);
        self.long_call.pnl_at_expiration(&price)
    }
}

impl Positionable for LongCall {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match (position.option.option_style, position.option.side) {
            (OptionStyle::Call, Side::Long) => {
                self.long_call = position.clone();
                Ok(())
            }
            _ => Err(PositionError::invalid_position_style(
                position.option.option_style,
                "Position is a Put or Long, it is not valid for LongCall".to_string(),
            )),
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.long_call])
    }

    /// Gets mutable positions matching the specified criteria from the strategy.
    ///
    /// # Arguments
    /// * `option_style` - The style of the option (Put/Call)
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
            (Side::Long, OptionStyle::Call, strike)
                if *strike == self.long_call.option.strike_price =>
            {
                Ok(vec![&mut self.long_call])
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
            (Side::Long, OptionStyle::Call, strike)
                if *strike == self.long_call.option.strike_price =>
            {
                self.long_call = position.clone();
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

impl StrategyConstructor for LongCall {
    fn get_strategy(_vec_positions: &[Position]) -> Result<Self, StrategyError> {
        todo!()
    }
}

impl Optimizable for LongCall {
    type Strategy = Self;

    fn find_optimal(
        &mut self,
        _option_chain: &OptionChain,
        _side: crate::strategies::FindOptimalSide,
        _criteria: OptimizationCriteria,
    ) {
        todo!()
    }
}

impl ProbabilityAnalysis for LongCall {
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        // Long call is profitable when price rises above break-even
        let break_even = self.break_even_points.first().ok_or_else(|| {
            ProbabilityError::RangeError(ProfitLossRangeErrorKind::InvalidBreakEvenPoints {
                reason: "No break-even points found for long call".to_string(),
            })
        })?;

        let option = &self.long_call.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let mut profit_range = ProfitLossRange::new(Some(*break_even), None, Positive::ZERO)?;

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
        // Long call has losses when price stays below break-even
        let break_even = self.break_even_points.first().ok_or_else(|| {
            ProbabilityError::RangeError(ProfitLossRangeErrorKind::InvalidBreakEvenPoints {
                reason: "No break-even points found for long call".to_string(),
            })
        })?;

        let option = &self.long_call.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let mut loss_range = ProfitLossRange::new(None, Some(*break_even), Positive::ZERO)?;

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

impl Greeks for LongCall {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.long_call.option])
    }
}

impl DeltaNeutrality for LongCall {}

impl PnLCalculator for LongCall {
    fn calculate_pnl(
        &self,
        market_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, PricingError> {
        self.long_call
            .calculate_pnl(market_price, expiration_date, implied_volatility)
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<PnL, PricingError> {
        self.long_call.calculate_pnl_at_expiration(underlying_price)
    }

    fn adjustments_pnl(&self, _adjustment: &DeltaAdjustment) -> Result<PnL, PricingError> {
        // Single-leg strategies like LongCall don't typically require delta adjustments
        // as they are directional strategies. Delta adjustments are more relevant for
        // complex multi-leg strategies aiming for delta neutrality.
        Err("Delta adjustments are not applicable to single-leg LongCall strategy".into())
    }
}

impl<X, Y> Simulate<X, Y> for LongCall
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
        let initial_premium = self.long_call.option.calculate_price_black_scholes()?.abs();

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
                let mut current_option = self.long_call.option.clone();
                current_option.underlying_price = step.y.positive();
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
                    true, // is_long = true for Long Call
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
                    // For long call: P&L = current_premium - initial_premium (we paid initial, sell at current)
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
                let final_price = last_step.y.positive();
                let pnl = self.calculate_pnl_at_expiration(&final_price)?;

                // Calculate expiration premium
                let mut exp_option = self.long_call.option.clone();
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

impl Strategable for LongCall {}

#[cfg(test)]
mod tests_simulate {
    use super::*;
    use crate::chains::generator_positive;

    use crate::simulation::simulator::Simulator;
    use crate::simulation::steps::Step;
    use crate::simulation::{Simulate, WalkParams, WalkType, WalkTypeAble};
    use crate::utils::TimeFrame;
    use rust_decimal_macros::dec;

    struct TestWalker;
    impl WalkTypeAble<Positive, Positive> for TestWalker {}

    fn create_test_long_call() -> LongCall {
        LongCall::new(
            "TEST".to_string(),
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.20),
            pos!(1.0),
            pos!(100.0),
            dec!(0.05),
            pos!(0.0),
            pos!(5.0),
            pos!(0.0),
            pos!(0.0),
        )
    }

    fn create_walk_params(prices: Vec<Positive>) -> WalkParams<Positive, Positive> {
        let init_step = Step::new(
            Positive::ONE,
            TimeFrame::Day,
            ExpirationDate::Days(pos!(30.0)),
            pos!(100.0),
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
        let strategy = create_test_long_call();
        let prices = vec![pos!(100.0), pos!(105.0), pos!(110.0), pos!(115.0)];
        let walk_params = create_walk_params(prices);
        let simulator = Simulator::new("Test".to_string(), 1, &walk_params, generator_positive);
        let results = strategy.simulate(&simulator, ExitPolicy::ProfitPercent(dec!(0.5)));
        assert!(results.is_ok());
        let stats = results.unwrap();
        assert_eq!(stats.total_simulations, 1);
    }

    #[test]
    fn test_simulate_expiration_exit() {
        let strategy = create_test_long_call();
        let prices = vec![pos!(100.0), pos!(101.0), pos!(102.0)];
        let walk_params = create_walk_params(prices);
        let simulator = Simulator::new("Test".to_string(), 1, &walk_params, generator_positive);
        let results = strategy.simulate(&simulator, ExitPolicy::Expiration);
        assert!(results.is_ok(), "Simulate failed: {:?}", results.err());
        let stats = results.unwrap();
        assert_eq!(stats.total_simulations, 1);
    }

    #[test]
    fn test_simulate_stats_aggregation() {
        let strategy = create_test_long_call();
        let prices = vec![pos!(100.0), pos!(105.0), pos!(110.0)];
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
