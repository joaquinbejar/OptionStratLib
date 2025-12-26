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
use positive::Positive;
use pretty_simple_display::{DebugPretty, DisplaySimple};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::debug;
use utoipa::ToSchema;

pub(super) const SHORT_CALL_DESCRIPTION: &str = "A Short Call (or Naked Call) is an options strategy where the trader sells a call option without owning the underlying stock. \
    This strategy generates immediate income through the premium received but carries unlimited risk if the stock price rises significantly. \
    The breakeven point is the strike price plus the premium received. Short calls are generally used when the trader has a bearish or neutral outlook on the underlying asset.";

/// Represents the details and structure of a Short Call options trading strategy.
///
/// A Short Call strategy involves selling a call option, which gives the buyer
/// the right to purchase the underlying asset at a specific strike price before
/// the expiration date. This strategy is generally employed when the trader
/// expects minimal movement or a decrease in the price of the underlying asset.
///
/// # Fields
///
/// * `name` - A unique name or identifier for this specific instance of the strategy.
/// * `kind` - Specifies that this instance is of the `ShortCall` strategy type.
/// * `description` - A detailed explanation providing more information about the strategy instance.
/// * `break_even_points` - A vector containing the price points where the strategy does not yield
///   any profit or loss. These points are represented as positive values.
/// * `short_call` - Represents the short call position in the strategy, which involves selling
///   a call option to generate premium income.
#[derive(Clone, DebugPretty, DisplaySimple, Serialize, Deserialize, ToSchema)]
pub struct ShortCall {
    /// Name identifier for this specific strategy instance
    pub name: String,
    /// Identifies this as a ShortCall strategy type
    pub kind: StrategyType,
    /// Detailed description of this strategy instance
    pub description: String,
    /// Price points where the strategy neither makes nor loses money
    pub break_even_points: Vec<Positive>,
    /// The short call option
    pub(super) short_call: Position,
}

impl ShortCall {
    /// Creates a new `ShortCall` strategy instance with the specified parameters.
    ///
    /// The `new` function initializes a short call option strategy by creating an associated
    /// option position and adding it to the strategy. This function is marked with
    /// `#[allow(clippy::too_many_arguments)]` because it takes several parameters required to
    /// define the short call options and associated financial metrics.
    ///
    /// # Parameters
    ///
    /// - `underlying_symbol` (`String`): The symbol of the underlying asset for the short call option.
    /// - `short_call_strike` (`Positive`): The strike price of the short call option.
    /// - `short_call_expiration` (`ExpirationDate`): The expiration date of the short call option.
    /// - `implied_volatility` (`Positive`): The implied volatility of the short call option.
    /// - `quantity` (`Positive`): The quantity of contracts for the short call option.
    /// - `underlying_price` (`Positive`): The current price of the underlying asset.
    /// - `risk_free_rate` (`Decimal`): The risk-free interest rate as a percentage.
    /// - `dividend_yield` (`Positive`): The dividend yield of the underlying asset as a percentage.
    /// - `premium_short_call` (`Positive`): Premium received for selling the short call option.
    /// - `open_fee_short_call` (`Positive`): Opening fee for the short call position.
    /// - `close_fee_short_call` (`Positive`): Closing fee for the short call position.
    ///
    /// # Returns
    ///
    /// Returns an initialized `ShortCall` strategy instance. The instance includes the short call
    /// option position with the specified parameters.
    ///
    /// # Panics
    ///
    /// This function will panic if the short call option created using the specified parameters
    /// fails to meet validity requirements during the `add_position` operation.
    ///
    #[allow(clippy::too_many_arguments, dead_code)]
    fn new(
        underlying_symbol: String,
        short_call_strike: Positive,
        short_call_expiration: ExpirationDate,
        implied_volatility: Positive,
        quantity: Positive,
        underlying_price: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        premium_short_call: Positive,
        open_fee_short_call: Positive,
        close_fee_short_call: Positive,
    ) -> Self {
        let mut strategy = ShortCall::default();

        let short_call_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol,
            short_call_strike,
            short_call_expiration,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        let short_call = Position::new(
            short_call_option,
            premium_short_call,
            Utc::now(),
            open_fee_short_call,
            close_fee_short_call,
            None,
            None,
        );
        strategy
            .add_position(&short_call.clone())
            .expect("Invalid short call option");

        strategy
    }
}

impl BasicAble for ShortCall {
    fn get_title(&self) -> String {
        let strategy_title = format!("{:?} Strategy: ", self.kind);
        let leg_titles: Vec<String> = [self.short_call.get_title()]
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
        let short_call = &self.short_call.option;

        hash_set.insert(OptionBasicType {
            option_style: &short_call.option_style,
            side: &short_call.side,
            strike_price: &short_call.strike_price,
            expiration_date: &short_call.expiration_date,
        });

        hash_set
    }
    fn get_implied_volatility(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let options = [(
            &self.short_call.option,
            &self.short_call.option.implied_volatility,
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
        let options = [(&self.short_call.option, &self.short_call.option.quantity)];

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
        self.short_call.one_option()
    }
    fn one_option_mut(&mut self) -> &mut Options {
        self.short_call.one_option_mut()
    }
    fn set_expiration_date(
        &mut self,
        expiration_date: ExpirationDate,
    ) -> Result<(), StrategyError> {
        self.short_call.option.expiration_date = expiration_date;
        Ok(())
    }
    fn set_underlying_price(&mut self, price: &Positive) -> Result<(), StrategyError> {
        self.short_call.option.underlying_price = *price;
        self.short_call.premium = Positive::from(
            self.short_call
                .option
                .calculate_price_black_scholes()?
                .abs(),
        );
        Ok(())
    }
    fn set_implied_volatility(&mut self, volatility: &Positive) -> Result<(), StrategyError> {
        self.short_call.option.implied_volatility = *volatility;
        self.short_call.premium = Positive(
            self.short_call
                .option
                .calculate_price_black_scholes()?
                .abs(),
        );
        Ok(())
    }
}

impl Validable for ShortCall {
    fn validate(&self) -> bool {
        if !self.short_call.validate() {
            debug!("Long call is invalid");
            return false;
        }
        true
    }
}

impl BreakEvenable for ShortCall {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        // For a short call, net_cost() from Position returns (fees - premium_received).
        // Break-even = strike + (premium_received - fees) / quantity
        // So, break-even = strike - (fees - premium_received) / quantity
        // Which is strike - (net_cost_from_position / quantity)
        self.break_even_points.push(
            (self.short_call.option.strike_price
                - self.short_call.net_cost()? / self.short_call.option.quantity)
                .round_to(2),
        );

        Ok(())
    }
}

impl Strategies for ShortCall {
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        let profit = self.calculate_profit_at(&self.short_call.option.strike_price)?;
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
        // Max loss for a short call is theoretically unlimited.
        Err(StrategyError::ProfitLossError(
            ProfitLossErrorKind::MaxLossError {
                reason: "Maximum loss is unlimited for a short call.".to_string(),
            },
        ))
    }
    fn get_profit_area(&self) -> Result<Decimal, StrategyError> {
        let high = self.get_max_profit().unwrap_or(Positive::ZERO);
        let base = self.short_call.option.strike_price - self.break_even_points[0];
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

impl Profit for ShortCall {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, PricingError> {
        let price = Some(price);
        self.short_call.pnl_at_expiration(&price)
    }
}

impl Positionable for ShortCall {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match (position.option.option_style, position.option.side) {
            (OptionStyle::Call, Side::Short) => {
                self.short_call = position.clone();
                Ok(())
            }
            _ => Err(PositionError::invalid_position_style(
                position.option.option_style,
                "Position is a Put or Long, it is not valid for ShortCall".to_string(),
            )),
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.short_call])
    }

    /// Gets mutable positions matching the specified criteria from the strategy.
    ///
    /// # Arguments
    /// * `option_style` - The style of the option (Put/Call)
    /// * `side` - The side of the position (Long/Short)
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
            (Side::Short, OptionStyle::Call, strike)
                if *strike == self.short_call.option.strike_price =>
            {
                Ok(vec![&mut self.short_call])
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
            (Side::Short, OptionStyle::Call, strike)
                if *strike == self.short_call.option.strike_price =>
            {
                self.short_call = position.clone();
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

impl StrategyConstructor for ShortCall {
    fn get_strategy(_vec_positions: &[Position]) -> Result<Self, StrategyError> {
        todo!()
    }
}

impl Optimizable for ShortCall {
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

impl ProbabilityAnalysis for ShortCall {
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        // Short call is profitable when price stays below break-even
        let break_even = self.break_even_points.first().ok_or_else(|| {
            ProbabilityError::RangeError(ProfitLossRangeErrorKind::InvalidBreakEvenPoints {
                reason: "No break-even points found for short call".to_string(),
            })
        })?;

        let option = &self.short_call.option;
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
        // Short call has losses when price rises above break-even
        let break_even = self.break_even_points.first().ok_or_else(|| {
            ProbabilityError::RangeError(ProfitLossRangeErrorKind::InvalidBreakEvenPoints {
                reason: "No break-even points found for short call".to_string(),
            })
        })?;

        let option = &self.short_call.option;
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

impl Greeks for ShortCall {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.short_call.option])
    }
}

impl DeltaNeutrality for ShortCall {}

impl PnLCalculator for ShortCall {
    fn calculate_pnl(
        &self,
        market_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, PricingError> {
        self.short_call
            .calculate_pnl(market_price, expiration_date, implied_volatility)
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<PnL, PricingError> {
        self.short_call
            .calculate_pnl_at_expiration(underlying_price)
    }

    fn adjustments_pnl(&self, _adjustment: &DeltaAdjustment) -> Result<PnL, PricingError> {
        // Single-leg strategies like ShortCall don't typically require delta adjustments
        // as they are directional strategies. Delta adjustments are more relevant for
        // complex multi-leg strategies aiming for delta neutrality.
        Err("Delta adjustments are not applicable to single-leg ShortCall strategy".into())
    }
}

impl<X, Y> Simulate<X, Y> for ShortCall
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
        let initial_premium = self
            .short_call
            .option
            .calculate_price_black_scholes()?
            .abs();

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
                let mut current_option = self.short_call.option.clone();
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
                    false, // is_long = false for Short Call
                ) {
                    exit_reason = reason;

                    // Check if it's take profit or stop loss
                    // For short: profit when current < initial, loss when current > initial
                    let pnl_percent = (initial_premium - current_premium) / initial_premium;
                    if pnl_percent >= dec!(0.5) {
                        hit_take_profit = true;
                    } else if pnl_percent <= dec!(-1.0) {
                        hit_stop_loss = true;
                    }

                    // Exit triggered - calculate P&L directly from premiums
                    // For short call: P&L = initial_premium - current_premium (we received initial, buy back at current)
                    // We use direct premium calculation instead of calculate_pnl() to avoid discrepancies
                    // from recalculating the initial premium
                    let pnl = PnL {
                        realized: Some(initial_premium - current_premium),
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
                let mut exp_option = self.short_call.option.clone();
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

impl Strategable for ShortCall {}

test_strategy_traits!(ShortCall, test_short_call_implementations);

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

    fn create_test_short_call() -> ShortCall {
        ShortCall::new(
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
        let strategy = create_test_short_call();
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
        let strategy = create_test_short_call();
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
        let strategy = create_test_short_call();
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
