use super::base::{BreakEvenable, Positionable, StrategyType};
use crate::chains::OptionChain;
use crate::error::strategies::ProfitLossErrorKind;
use crate::error::{
    GreeksError, ProbabilityError, StrategyError,
    position::{PositionError, PositionValidationErrorKind},
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
use crate::strategies::probabilities::utils::{PriceTrend, VolatilityAdjustment};
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
    fn expected_value(
        &self,
        _volatility_adj: Option<VolatilityAdjustment>,
        _trend: Option<PriceTrend>,
    ) -> Result<Positive, ProbabilityError> {
        todo!()
    }

    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        todo!()
    }

    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        todo!()
    }
}

impl Greeks for ShortPut {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        todo!()
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
        todo!()
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
    ) -> Result<Vec<PnL>, Box<dyn Error>> {
        let mut results = Vec::with_capacity(sim.len());
        let initial_premium = self.short_put.option.calculate_price_black_scholes()?.abs();
        let implied_volatility = self.short_put.option.implied_volatility;

        for random_walk in sim.into_iter() {
            // Iterate through the random walk
            for step in random_walk.get_steps().iter().skip(1) {
                let days_left = match step.x.days_left() {
                    Ok(days) => days,
                    Err(_) => break, // Expiration reached
                };

                // Calculate current option premium
                let mut current_option = self.short_put.option.clone();
                current_option.underlying_price = step.y.positive();
                current_option.expiration_date = ExpirationDate::Days(days_left);

                let current_premium = current_option.calculate_price_black_scholes()?.abs();
                let index = *step.x.index() as usize;

                // Check exit policy
                if crate::simulation::check_exit_policy(
                    &exit,
                    initial_premium,
                    current_premium,
                    index,
                    days_left,
                    current_option.underlying_price,
                )
                .is_some()
                {
                    // Exit triggered - calculate P&L
                    let pnl = self.calculate_pnl(
                        &current_option.underlying_price,
                        ExpirationDate::Days(days_left),
                        &implied_volatility,
                    )?;
                    results.push(pnl);
                    break;
                }
            }

            // If no exit triggered, calculate P&L at expiration
            if results.len() < sim.into_iter().count()
                && let Some(last_step) = random_walk.last()
            {
                let final_price = last_step.y.positive();
                let pnl = self.calculate_pnl_at_expiration(&final_price)?;
                results.push(pnl);
            }
        }

        Ok(results)
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
    fn test_simulate_profit_target_reached() {
        // Create a short put at strike 100
        let strategy = create_test_short_put();

        // Historical prices that move up (favorable for short put)
        let prices = vec![
            pos!(100.0),
            pos!(102.0),
            pos!(104.0),
            pos!(106.0),
            pos!(108.0),
            pos!(110.0),
        ];

        let walk_params = create_walk_params(prices);
        let simulator = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        );

        // Exit policy: 50% profit target
        let exit_policy = ExitPolicy::ProfitPercent(dec!(0.5));

        let results = strategy.simulate(&simulator, exit_policy);
        assert!(results.is_ok());

        let pnl_vec = results.unwrap();
        assert_eq!(pnl_vec.len(), 1);

        // Should have positive P&L since price moved up
        let pnl = &pnl_vec[0];
        let total_pnl = pnl.realized.unwrap_or(dec!(0.0)) + pnl.unrealized.unwrap_or(dec!(0.0));
        assert!(
            total_pnl > dec!(0.0),
            "Expected positive P&L but got {}",
            total_pnl
        );
    }

    #[test]
    fn test_simulate_with_price_movement() {
        // Create a short put at strike 100
        let strategy = create_test_short_put();

        // Historical prices that move down (unfavorable for short put)
        let prices = vec![
            pos!(100.0),
            pos!(95.0),
            pos!(90.0),
            pos!(85.0),
            pos!(80.0),
            pos!(75.0),
        ];

        let walk_params = create_walk_params(prices);
        let simulator = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        );

        // Exit policy: 50% profit or 200% loss
        let exit_policy = ExitPolicy::profit_or_loss(dec!(0.5), dec!(2.0));

        let results = strategy.simulate(&simulator, exit_policy);
        assert!(results.is_ok());

        let pnl_vec = results.unwrap();
        assert_eq!(pnl_vec.len(), 1);

        // Verify we got a result (either profit, loss, or expiration)
        let pnl = &pnl_vec[0];
        let total_pnl = pnl.realized.unwrap_or(dec!(0.0)) + pnl.unrealized.unwrap_or(dec!(0.0));
        // With significant price drop, P&L should be less than initial premium
        assert!(
            total_pnl < dec!(5.0),
            "Expected P&L < 5.0 but got {}",
            total_pnl
        );
    }

    #[test]
    fn test_simulate_expiration_otm() {
        // Create a short put at strike 100
        let strategy = create_test_short_put();

        // Historical prices that stay above strike (OTM at expiration)
        let prices = vec![
            pos!(100.0),
            pos!(101.0),
            pos!(102.0),
            pos!(103.0),
            pos!(104.0),
            pos!(105.0),
        ];

        let walk_params = create_walk_params(prices);
        let simulator = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        );

        // Exit policy that won't trigger (very wide range)
        let exit_policy = ExitPolicy::profit_or_loss(dec!(0.9), dec!(5.0));

        let results = strategy.simulate(&simulator, exit_policy);
        assert!(results.is_ok());

        let pnl_vec = results.unwrap();
        assert_eq!(pnl_vec.len(), 1);

        // Should have positive P&L (keep full premium) since expired OTM
        let pnl = &pnl_vec[0];
        let total_pnl = pnl.realized.unwrap_or(dec!(0.0)) + pnl.unrealized.unwrap_or(dec!(0.0));
        assert!(
            total_pnl > dec!(0.0),
            "Expected positive P&L but got {}",
            total_pnl
        );
    }

    #[test]
    fn test_simulate_expiration_itm() {
        // Create a short put at strike 100
        let strategy = create_test_short_put();

        // Historical prices that end below strike (ITM at expiration)
        let prices = vec![
            pos!(100.0),
            pos!(99.0),
            pos!(98.0),
            pos!(97.0),
            pos!(96.0),
            pos!(95.0),
        ];

        let walk_params = create_walk_params(prices);
        let simulator = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        );

        // Exit policy that won't trigger
        let exit_policy = ExitPolicy::profit_or_loss(dec!(0.9), dec!(5.0));

        let results = strategy.simulate(&simulator, exit_policy);
        assert!(results.is_ok());

        let pnl_vec = results.unwrap();
        assert_eq!(pnl_vec.len(), 1);

        // Should have negative or small positive P&L since expired ITM
        let pnl = &pnl_vec[0];
        let total_pnl = pnl.realized.unwrap_or(dec!(0.0)) + pnl.unrealized.unwrap_or(dec!(0.0));
        // At strike 100, final price 95, intrinsic value = 5
        // Premium received = 5, so P&L should be close to 0 or slightly negative
        assert!(
            total_pnl <= dec!(1.0),
            "Expected P&L <= 1.0 but got {}",
            total_pnl
        );
    }

    #[test]
    fn test_simulate_multiple_simulations() {
        // Create a short put at strike 100
        let strategy = create_test_short_put();

        // Historical prices for multiple simulations
        let prices = vec![pos!(100.0), pos!(105.0), pos!(110.0), pos!(115.0)];

        let walk_params = create_walk_params(prices);
        // Create simulator with 3 simulations
        let simulator = Simulator::new(
            "Test Simulator".to_string(),
            3,
            &walk_params,
            generator_positive,
        );

        let exit_policy = ExitPolicy::ProfitPercent(dec!(0.5));

        let results = strategy.simulate(&simulator, exit_policy);
        assert!(results.is_ok());

        let pnl_vec = results.unwrap();
        // Should have results for each simulation
        assert!(
            pnl_vec.len() >= 3,
            "Expected at least 3 results but got {}",
            pnl_vec.len()
        );

        // All should be profitable since price moves up
        for pnl in &pnl_vec {
            let total_pnl = pnl.realized.unwrap_or(dec!(0.0)) + pnl.unrealized.unwrap_or(dec!(0.0));
            assert!(
                total_pnl > dec!(0.0),
                "Expected positive P&L but got {}",
                total_pnl
            );
        }
    }

    #[test]
    fn test_simulate_with_or_exit_policy() {
        // Create a short put at strike 100
        let strategy = create_test_short_put();

        // Prices that should trigger profit target
        let prices = vec![pos!(100.0), pos!(110.0), pos!(120.0)];

        let walk_params = create_walk_params(prices);
        let simulator = Simulator::new(
            "Test Simulator".to_string(),
            1,
            &walk_params,
            generator_positive,
        );

        // OR exit policy: profit target OR stop loss
        let exit_policy = ExitPolicy::profit_or_loss(dec!(0.5), dec!(1.0));

        let results = strategy.simulate(&simulator, exit_policy);
        assert!(results.is_ok());

        let pnl_vec = results.unwrap();
        assert_eq!(pnl_vec.len(), 1);
        let total_pnl =
            pnl_vec[0].realized.unwrap_or(dec!(0.0)) + pnl_vec[0].unrealized.unwrap_or(dec!(0.0));
        assert!(
            total_pnl > dec!(0.0),
            "Expected positive P&L but got {}",
            total_pnl
        );
    }
}
