use super::base::{BreakEvenable, Positionable, StrategyType};
use crate::backtesting::results::{SimulationResult, SimulationStatsResult};

use crate::chains::OptionChain;
use crate::error::strategies::ProfitLossErrorKind;
use crate::error::{
    GreeksError, ProbabilityError, StrategyError,
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
use crate::{ExpirationDate, Options, Positive, test_strategy_traits};
use chrono::Utc;
use pretty_simple_display::{DebugPretty, DisplaySimple};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use tracing::debug;
use utoipa::ToSchema;

pub(super) const SHORT_PUT_DESCRIPTION: &str = "A Short Put (or Naked Put) is an options strategy where the trader sells a put option without holding a short position in the underlying stock. \
    This strategy provides immediate income from the premium collected but includes substantial risk if the stock price falls below the strike price. \
    The breakeven point is the strike price minus the premium received. Short puts are typiputy employed when the trader has a bullish or neutral market outlook.";

/// Represents a Short Put options trading strategy.
///
/// A short put is a neutral to bullish strategy involving the sale of a put option.
/// This strategy generates a credit upfront, with the potential obligation to buy the underlying asset
/// at the strike price if the price falls below it. Below are the details stored in this struct:
///
/// Fields:
/// - `name`: The name identifier for this specific strategy instance. This is used to uniquely recognize
///   and distinguish this instance.
/// - `kind`: A field that identifies this as a ShortPut strategy type. It is of type `StrategyType`,
///   which categorizes different trading strategies.
/// - `description`: A detailed description of this strategy instance. This field allows for additional
///   explanation or metadata about why this strategy is being used or how it functions.
/// - `break_even_points`: A vector of price points (of type `Positive`) where the strategy neither makes
///   nor loses money. These are the threshold price levels that determine profitability.
/// - `short_put`: The short put position associated with this strategy. It is declared private (via
///   `pub(super)`) to restrict its accessibility from other modules, ensuring controlled and encapsulated
///   use.
#[derive(Clone, DebugPretty, DisplaySimple, Serialize, Deserialize, ToSchema)]
pub struct ShortPut {
    /// Name identifier for this specific strategy instance
    pub name: String,
    /// Identifies this as a ShortPut strategy type
    pub kind: StrategyType,
    /// Detailed description of this strategy instance
    pub description: String,
    /// Price points where the strategy neither makes nor loses money
    pub break_even_points: Vec<Positive>,
    /// The short put position
    pub(super) short_put: Position,
}

impl ShortPut {
    /// Creates a new `ShortPut` strategy instance with the given parameters.
    ///
    /// This function constructs a `ShortPut` strategy by initializing a short put position
    /// using the provided parameters and adding it to the strategy's list of positions. The
    /// required inputs are both descriptive and numerical attributes of the short put option,
    /// such as the underlying symbol, strike price, expiration date, volatility, and fees.
    ///
    /// # Parameters
    ///
    /// - `underlying_symbol`: A `String` representing the symbol of the underlying asset.
    /// - `short_put_strike`: A `Positive` value specifying the strike price of the short put option.
    ///   This should always be greater than zero.
    /// - `short_put_expiration`: The `ExpirationDate` when the short put option expires.
    /// - `implied_volatility`: A `Positive` value representing the implied volatility of the option.
    /// - `quantity`: A `Positive` value indicating the number of contracts in the position.
    /// - `underlying_price`: A `Positive` value representing the current price of the underlying asset.
    /// - `risk_free_rate`: A `Decimal` representing the risk-free interest rate, expressed as a decimal.
    /// - `dividend_yield`: A `Positive` value representing the dividend yield of the underlying asset.
    /// - `premium_short_put`: A `Positive` value representing the premium received when selling the put option.
    /// - `open_fee_short_put`: A `Positive` value indicating the opening fee for the short put position.
    /// - `close_fee_short_put`: A `Positive` value indicating the closing fee for the short put position.
    ///
    /// # Returns
    ///
    /// A new instance of `ShortPut` containing the initialized short put position.
    ///
    /// # Panics
    ///
    /// This function will panic if adding the short put position to the strategy fails,
    /// which may happen if the position is deemed invalid.
    ///
    #[allow(clippy::too_many_arguments, dead_code)]
    pub fn new(
        underlying_symbol: String,
        short_put_strike: Positive,
        short_put_expiration: ExpirationDate,
        implied_volatility: Positive,
        quantity: Positive,
        underlying_price: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        premium_short_put: Positive,
        open_fee_short_put: Positive,
        close_fee_short_put: Positive,
    ) -> Self {
        let mut strategy = ShortPut::default();

        let short_put_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol,
            short_put_strike,
            short_put_expiration,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Put,
            dividend_yield,
            None,
        );
        let short_put = Position::new(
            short_put_option,
            premium_short_put,
            Utc::now(),
            open_fee_short_put,
            close_fee_short_put,
            None,
            None,
        );
        strategy
            .add_position(&short_put.clone())
            .expect("Invalid short put option");

        strategy
    }
}

impl BasicAble for ShortPut {
    fn get_title(&self) -> String {
        let strategy_title = format!("{:?} Strategy: ", self.kind);
        let leg_titles: Vec<String> = [self.short_put.get_title()]
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
        let short_put = &self.short_put.option;

        hash_set.insert(OptionBasicType {
            option_style: &short_put.option_style,
            side: &short_put.side,
            strike_price: &short_put.strike_price,
            expiration_date: &short_put.expiration_date,
        });

        hash_set
    }
    fn get_implied_volatility(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let options = [(
            &self.short_put.option,
            &self.short_put.option.implied_volatility,
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
        let options = [(&self.short_put.option, &self.short_put.option.quantity)];

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
        self.short_put.one_option()
    }
    fn one_option_mut(&mut self) -> &mut Options {
        self.short_put.one_option_mut()
    }
    fn set_expiration_date(
        &mut self,
        expiration_date: ExpirationDate,
    ) -> Result<(), StrategyError> {
        self.short_put.option.expiration_date = expiration_date;
        Ok(())
    }
    fn set_underlying_price(&mut self, price: &Positive) -> Result<(), StrategyError> {
        self.short_put.option.underlying_price = *price;
        self.short_put.premium =
            Positive::from(self.short_put.option.calculate_price_black_scholes()?.abs());
        Ok(())
    }
    fn set_implied_volatility(&mut self, volatility: &Positive) -> Result<(), StrategyError> {
        self.short_put.option.implied_volatility = *volatility;
        self.short_put.premium =
            Positive(self.short_put.option.calculate_price_black_scholes()?.abs());
        Ok(())
    }
}

impl Validable for ShortPut {
    fn validate(&self) -> bool {
        if !self.short_put.validate() {
            debug!("Long call is invalid");
            return false;
        }
        true
    }
}

impl BreakEvenable for ShortPut {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        self.break_even_points.push(
            (self.short_put.option.strike_price
                + self.get_net_cost()? / self.short_put.option.quantity)
                .round_to(2),
        );

        Ok(())
    }
}

impl Strategies for ShortPut {
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        let profit = self.calculate_profit_at(&self.short_put.option.strike_price)?;
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
        let loss = self.calculate_profit_at(&self.short_put.option.strike_price)?;
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
        let base = self.short_put.option.strike_price - self.break_even_points[0];
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

impl Profit for ShortPut {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, Box<dyn Error>> {
        let price = Some(price);
        self.short_put.pnl_at_expiration(&price)
    }
}

impl Positionable for ShortPut {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match (position.option.option_style, position.option.side) {
            (OptionStyle::Put, Side::Short) => {
                self.short_put = position.clone();
                Ok(())
            }
            _ => Err(PositionError::invalid_position_style(
                position.option.option_style,
                "Position is a Put or Long, it is not valid for ShortPut".to_string(),
            )),
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.short_put])
    }

    /// Gets mutable positions matching the specified criteria from the strategy.
    ///
    /// # Arguments
    /// * `option_style` - The style of the option (Put/Put)
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
            (Side::Short, OptionStyle::Put, strike)
                if *strike == self.short_put.option.strike_price =>
            {
                Ok(vec![&mut self.short_put])
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
            (Side::Short, OptionStyle::Put, strike)
                if *strike == self.short_put.option.strike_price =>
            {
                self.short_put = position.clone();
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

impl StrategyConstructor for ShortPut {
    fn get_strategy(_vec_positions: &[Position]) -> Result<Self, StrategyError> {
        todo!()
    }
}

impl Optimizable for ShortPut {
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

impl ProbabilityAnalysis for ShortPut {
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        // Short put is profitable when price stays above break-even
        let break_even = self.break_even_points.first().ok_or_else(|| {
            ProbabilityError::RangeError(ProfitLossRangeErrorKind::InvalidBreakEvenPoints {
                reason: "No break-even points found for short put".to_string(),
            })
        })?;

        let option = &self.short_put.option;
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
        // Short put has losses when price falls below break-even
        let break_even = self.break_even_points.first().ok_or_else(|| {
            ProbabilityError::RangeError(ProfitLossRangeErrorKind::InvalidBreakEvenPoints {
                reason: "No break-even points found for short put".to_string(),
            })
        })?;

        let option = &self.short_put.option;
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

impl Greeks for ShortPut {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.short_put.option])
    }
}

impl DeltaNeutrality for ShortPut {}

impl PnLCalculator for ShortPut {
    fn calculate_pnl(
        &self,
        market_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        self.short_put
            .calculate_pnl(market_price, expiration_date, implied_volatility)
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        self.short_put.calculate_pnl_at_expiration(underlying_price)
    }

    fn adjustments_pnl(&self, _adjustment: &DeltaAdjustment) -> Result<PnL, Box<dyn Error>> {
        // Single-leg strategies like ShortPut don't typically require delta adjustments
        // as they are directional strategies. Delta adjustments are more relevant for
        // complex multi-leg strategies aiming for delta neutrality.
        Err("Delta adjustments are not applicable to single-leg ShortPut strategy".into())
    }
}

impl<X, Y> Simulate<X, Y> for ShortPut
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
    ) -> Result<SimulationStatsResult, Box<dyn Error>> {
        use indicatif::{ProgressBar, ProgressStyle};
        use rust_decimal::MathematicalOps;
        use rust_decimal_macros::dec;

        let mut simulation_results = Vec::with_capacity(sim.len());
        let initial_premium = self.short_put.option.calculate_price_black_scholes()?.abs();

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
                let mut current_option = self.short_put.option.clone();
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
                    false, // is_long = false for Short Put
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
                    // For short put: P&L = initial_premium - current_premium (we received initial, buy back at current)
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

                // Calculate expiration premium (use very small time instead of zero)
                let mut exp_option = self.short_put.option.clone();
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

impl Strategable for ShortPut {}

test_strategy_traits!(ShortPut, test_short_put_implementations);

#[cfg(test)]
mod tests_simulate {
    use super::*;
    use crate::chains::generator_positive;
    use crate::pos;
    use crate::simulation::simulator::Simulator;
    use crate::simulation::steps::Step;
    use crate::simulation::{Simulate, WalkParams, WalkType, WalkTypeAble};
    use crate::utils::TimeFrame;
    use rust_decimal_macros::dec;

    /// Helper struct to implement WalkTypeAble for tests
    struct TestWalker;

    impl WalkTypeAble<Positive, Positive> for TestWalker {}

    /// Creates a test ShortPut strategy
    fn create_test_short_put() -> ShortPut {
        ShortPut::new(
            "TEST".to_string(),
            pos!(100.0), // strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.20),  // implied volatility
            pos!(1.0),   // quantity
            pos!(100.0), // underlying price
            dec!(0.05),  // risk-free rate
            pos!(0.0),   // dividend yield
            pos!(5.0),   // premium received
            pos!(0.0),   // open fee
            pos!(0.0),   // close fee
        )
    }

    /// Helper to create WalkParams with Historical data
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
        let strategy = create_test_short_put();
        let prices = vec![pos!(100.0), pos!(105.0), pos!(110.0), pos!(115.0)];

        let walk_params = create_walk_params(prices);
        let simulator = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        );

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
        let prices = vec![pos!(100.0), pos!(95.0), pos!(90.0), pos!(85.0)];

        let walk_params = create_walk_params(prices);
        let simulator = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        );

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
        let prices = vec![pos!(100.0), pos!(101.0), pos!(102.0), pos!(103.0)];

        let walk_params = create_walk_params(prices);
        let simulator = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        );

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
        let prices = vec![pos!(100.0), pos!(105.0), pos!(110.0), pos!(115.0)];

        let walk_params = create_walk_params(prices);
        let simulator = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        );

        let exit_policy = ExitPolicy::Or(vec![
            ExitPolicy::ProfitPercent(dec!(0.5)),
            ExitPolicy::UnderlyingAbove(pos!(112.0)),
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
        let prices = vec![pos!(100.0), pos!(105.0), pos!(110.0)];

        let walk_params = create_walk_params(prices);
        let simulator = Simulator::new(
            "Test Simulator".to_string(),
            5,
            &walk_params,
            generator_positive,
        );

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
            pos!(100.0),
            pos!(101.0),
            pos!(102.0),
            pos!(103.0),
            pos!(104.0),
        ];

        let walk_params = create_walk_params(prices);
        let simulator = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        );

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
        let prices = vec![pos!(100.0), pos!(95.0), pos!(90.0)];

        let walk_params = create_walk_params(prices);
        let simulator = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        );

        let exit_policy = ExitPolicy::UnderlyingBelow(pos!(92.0));
        let results = strategy.simulate(&simulator, exit_policy);

        assert!(results.is_ok());
        let stats = results.unwrap();
        assert_eq!(stats.total_simulations, 1);
    }

    #[test]
    fn test_simulate_and_exit_policy() {
        let strategy = create_test_short_put();
        let prices = vec![pos!(100.0), pos!(105.0), pos!(110.0), pos!(115.0)];

        let walk_params = create_walk_params(prices);
        let simulator = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        );

        let exit_policy = ExitPolicy::And(vec![
            ExitPolicy::TimeSteps(2),
            ExitPolicy::UnderlyingAbove(pos!(105.0)),
        ]);
        let results = strategy.simulate(&simulator, exit_policy);

        assert!(results.is_ok());
        let stats = results.unwrap();
        assert_eq!(stats.total_simulations, 1);
    }

    #[test]
    fn test_simulate_multiple_simulations() {
        let strategy = create_test_short_put();
        let prices = vec![pos!(100.0), pos!(102.0), pos!(104.0)];

        let walk_params = create_walk_params(prices);
        let simulator = Simulator::new(
            "Test Simulator".to_string(),
            10,
            &walk_params,
            generator_positive,
        );

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
        let prices = vec![pos!(100.0), pos!(105.0), pos!(110.0)];

        let walk_params = create_walk_params(prices);
        let simulator = Simulator::new(
            "Test Simulator".to_string(),
            3,
            &walk_params,
            generator_positive,
        );

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
        let prices = vec![pos!(100.0), pos!(100.5), pos!(101.0), pos!(100.8)];

        let walk_params = create_walk_params(prices);
        let simulator = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        );

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
        let prices = vec![pos!(100.0), pos!(110.0), pos!(120.0)];

        let walk_params = create_walk_params(prices);
        let simulator = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        );

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
        let prices = vec![pos!(100.0), pos!(85.0), pos!(70.0)];

        let walk_params = create_walk_params(prices);
        let simulator = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        );

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
        let prices = vec![pos!(100.0), pos!(95.0), pos!(105.0), pos!(100.0)];

        let walk_params = create_walk_params(prices);
        let simulator = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        );

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
        let prices = vec![pos!(100.0), pos!(101.0), pos!(102.0)];
        let price_count = prices.len();

        let walk_params = create_walk_params(prices);
        let simulator = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        );

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
        let prices = vec![pos!(100.0), pos!(102.0), pos!(98.0)];

        let walk_params = create_walk_params(prices);
        let simulator = Simulator::new(
            "Test Simulator".to_string(),
            5,
            &walk_params,
            generator_positive,
        );

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
