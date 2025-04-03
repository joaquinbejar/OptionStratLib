/*
Strangle Strategy

A strangle involves simultaneously buying an out-of-the-money call and an out-of-the-money put option with the same expiration date.
This strategy is similar to a straddle but typically has a lower cost and requires a larger price move to become profitable.

Key characteristics:
- Unlimited profit potential
- Lower cost than a straddle
- Requires a larger price move to become profitable
*/
use super::base::{
    BreakEvenable, Optimizable, Positionable, Strategable, Strategies, StrategyBasics,
    StrategyType, Validable,
};
use crate::chains::StrategyLegs;
use crate::chains::chain::OptionChain;
use crate::chains::utils::OptionDataGroup;
use crate::constants::{DARK_BLUE, DARK_GREEN, ZERO};
use crate::error::position::{PositionError, PositionValidationErrorKind};
use crate::error::probability::ProbabilityError;
use crate::error::strategies::{ProfitLossErrorKind, StrategyError};
use crate::error::{GreeksError, OperationErrorKind};
use crate::greeks::Greeks;
use crate::model::ProfitLossRange;
use crate::model::position::Position;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use crate::model::utils::mean_and_std;
use crate::pnl::PnLCalculator;
use crate::pnl::utils::PnL;
use crate::pricing::payoff::Profit;
use crate::strategies::StrategyConstructor;
use crate::strategies::delta_neutral::DeltaNeutrality;
use crate::strategies::probabilities::core::ProbabilityAnalysis;
use crate::strategies::probabilities::utils::VolatilityAdjustment;
use crate::strategies::utils::{FindOptimalSide, OptimizationCriteria, calculate_price_range};
use crate::visualization::model::{ChartPoint, ChartVerticalLine, LabelOffsetType};
use crate::visualization::utils::Graph;
use crate::{Options, Positive};
use chrono::Utc;
use num_traits::{FromPrimitive, ToPrimitive};
use plotters::prelude::full_palette::ORANGE;
use plotters::prelude::{RED, ShapeStyle};
use rust_decimal::Decimal;
use std::error::Error;
use tracing::{debug, info, trace};

const SHORT_STRANGLE_DESCRIPTION: &str = "A short strangle involves selling an out-of-the-money call and an \
out-of-the-money put with the same expiration date. This strategy is used when low volatility \
is expected and the underlying asset's price is anticipated to remain stable.";

/// # Short Strangle Strategy
///
/// A Short Strangle is an options trading strategy that involves simultaneously selling an out-of-the-money call option
/// and an out-of-the-money put option with the same expiration date but different strike prices.
///
/// ## Strategy Overview
/// - **Outlook**: Neutral, expecting low volatility
/// - **Risk**: Unlimited in both directions (if the underlying asset moves significantly)
/// - **Reward**: Limited to the combined premium received from selling both options
/// - **Breakeven Points**: Upper strike + net premium received, Lower strike - net premium received
/// - **Margin Requirements**: Typically high due to undefined risk
///
/// ## Properties
/// The `ShortStrangle` struct encapsulates all components and characteristics of a short strangle options strategy.
///
/// ## Fields
/// * `name`: A descriptive name for this specific strategy instance
/// * `kind`: The type of strategy (ShortStrangle)
/// * `description`: A detailed description of the specific strategy implementation
/// * `break_even_points`: The price points at which the strategy breaks even (no profit/loss)
/// * `short_call`: The short call position component of the strategy
/// * `short_put`: The short put position component of the strategy
///
/// ## Maximum Profit/Loss
/// * **Max Profit**: Limited to the total premium received from selling both options
/// * **Max Loss**: Unlimited. Upside loss = stock price - call strike - premium received.
///   Downside loss = put strike - stock price - premium received.
///
/// ## Ideal Market Conditions
/// Best implemented during periods of high implied volatility that is expected to decrease,
/// and when the underlying asset is anticipated to trade within a narrow price range until expiration.
///
/// ## Example
/// A short strangle might be constructed by selling a call with a strike price of $110
/// and selling a put with a strike price of $90, when the underlying is trading at $100.
/// The premium collected might be $2 for the call and $2 for the put, for a total of $4.
/// Break-even points would be at $86 ($90 - $4) and $114 ($110 + $4).
#[derive(Clone, Debug)]
pub struct ShortStrangle {
    /// Name identifier for this specific strategy instance
    pub name: String,
    /// Identifies this as a ShortStrangle strategy type
    pub kind: StrategyType,
    /// Detailed description of this particular strategy implementation
    pub description: String,
    /// Price points at which the strategy neither makes nor loses money
    pub break_even_points: Vec<Positive>,
    /// The short call leg of the strategy (typically out-of-the-money)
    short_call: Position,
    /// The short put leg of the strategy (typically out-of-the-money)
    short_put: Position,
}

impl ShortStrangle {
    /// Creates a new Short Strangle options strategy.
    ///
    /// A Short Strangle is created by selling an out-of-the-money call option and an out-of-the-money put option with
    /// the same expiration date. This strategy is profitable when the underlying asset's price remains stable
    /// between the two strike prices, allowing both options to expire worthless.
    ///
    /// # Parameters
    ///
    /// * `underlying_symbol` - Symbol of the underlying asset (e.g., "AAPL", "SPY")
    /// * `underlying_price` - Current market price of the underlying asset
    /// * `call_strike` - Strike price for the short call option. If set to zero, defaults to 110% of underlying price.
    /// * `put_strike` - Strike price for the short put option. If set to zero, defaults to 90% of underlying price.
    /// * `expiration` - Expiration date for both options
    /// * `implied_volatility` - Current implied volatility for the options
    /// * `risk_free_rate` - Risk-free interest rate as a decimal
    /// * `dividend_yield` - Expected dividend yield of the underlying asset
    /// * `quantity` - Number of contracts to trade
    /// * `premium_short_call` - Premium received for selling the call option
    /// * `premium_short_put` - Premium received for selling the put option
    /// * `open_fee_short_call` - Transaction fee for opening the short call position
    /// * `close_fee_short_call` - Transaction fee for closing the short call position
    /// * `open_fee_short_put` - Transaction fee for opening the short put position
    /// * `close_fee_short_put` - Transaction fee for closing the short put position
    ///
    /// # Returns
    ///
    /// A new `ShortStrangle` instance with both positions initialized and break-even points calculated.
    ///
    /// # Default Values
    ///
    /// - If `call_strike` is zero, it will be set to 110% of the underlying price
    /// - If `put_strike` is zero, it will be set to 90% of the underlying price
    ///
    /// # Break-Even Points
    ///
    /// The strategy has two break-even points:
    /// - Lower break-even: Put strike minus the total premium received per contract
    /// - Upper break-even: Call strike plus the total premium received per contract
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        mut call_strike: Positive,
        mut put_strike: Positive,
        expiration: ExpirationDate,
        implied_volatility: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        quantity: Positive,
        premium_short_call: Positive,
        premium_short_put: Positive,
        open_fee_short_call: Positive,
        close_fee_short_call: Positive,
        open_fee_short_put: Positive,
        close_fee_short_put: Positive,
    ) -> Self {
        if call_strike == Positive::ZERO {
            call_strike = underlying_price * 1.1;
        }
        if put_strike == Positive::ZERO {
            put_strike = underlying_price * 0.9;
        }
        let mut strategy = ShortStrangle {
            name: "Short Strangle".to_string(),
            kind: StrategyType::ShortStrangle,
            description: SHORT_STRANGLE_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call: Position::default(),
            short_put: Position::default(),
        };

        let short_call_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            call_strike,
            expiration,
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
        );
        strategy
            .add_position(&short_call.clone())
            .expect("Invalid position");

        let short_put_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol,
            put_strike,
            expiration,
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
        );
        strategy
            .add_position(&short_put.clone())
            .expect("Invalid position");

        strategy
            .update_break_even_points()
            .expect("Unable to update break even points");
        strategy
    }
}

impl StrategyConstructor for ShortStrangle {
    fn get_strategy(vec_options: &[Position]) -> Result<Self, StrategyError> {
        // Need exactly 2 options for a short strangle
        if vec_options.len() != 2 {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Short Strangle get_strategy".to_string(),
                    reason: "Must have exactly 2 options".to_string(),
                },
            ));
        }

        // Sort options by option style to identify call and put
        let mut sorted_options = vec_options.to_vec();
        sorted_options.sort_by(|a, b| {
            a.option
                .strike_price
                .partial_cmp(&b.option.strike_price)
                .unwrap()
        });

        let put_option = &sorted_options[0]; // Put will be first
        let call_option = &sorted_options[1]; // Call will be second

        // Validate one option is call and other is put
        if call_option.option.option_style != OptionStyle::Call
            || put_option.option.option_style != OptionStyle::Put
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Short Strangle get_strategy".to_string(),
                    reason: "One option must be a call and one must be a put".to_string(),
                },
            ));
        }

        // Validate both options are Short
        if call_option.option.side != Side::Short || put_option.option.side != Side::Short {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Short Strangle get_strategy".to_string(),
                    reason: "Both options must be Short positions".to_string(),
                },
            ));
        }

        // Validate call strike is higher than put strike
        if call_option.option.strike_price <= put_option.option.strike_price {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Short Strangle get_strategy".to_string(),
                    reason: "Call strike must be higher than put strike".to_string(),
                },
            ));
        }

        // Validate expiration dates match
        if call_option.option.expiration_date != put_option.option.expiration_date {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Short Strangle get_strategy".to_string(),
                    reason: "Options must have the same expiration date".to_string(),
                },
            ));
        }

        // Create positions
        let short_call = Position::new(
            call_option.option.clone(),
            call_option.premium,
            Utc::now(),
            call_option.open_fee,
            call_option.close_fee,
        );

        let short_put = Position::new(
            put_option.option.clone(),
            put_option.premium,
            Utc::now(),
            put_option.open_fee,
            put_option.close_fee,
        );

        // Create strategy
        let mut strategy = ShortStrangle {
            name: "Short Strangle".to_string(),
            kind: StrategyType::ShortStrangle,
            description: SHORT_STRANGLE_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call,
            short_put,
        };

        // Validate and update break-even points
        strategy.validate();
        strategy.update_break_even_points()?;

        Ok(strategy)
    }
}

impl BreakEvenable for ShortStrangle {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        let total_premium = self.net_premium_received()?;

        self.break_even_points.push(
            (self.short_put.option.strike_price
                - (total_premium / self.short_put.option.quantity).to_dec())
            .round_to(2),
        );

        self.break_even_points.push(
            (self.short_call.option.strike_price
                + (total_premium / self.short_call.option.quantity))
                .round_to(2),
        );

        self.break_even_points.sort();
        Ok(())
    }
}

impl Positionable for ShortStrangle {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match (&position.option.option_style, &position.option.side) {
            (OptionStyle::Call, Side::Short) => {
                self.short_call = position.clone();
                Ok(())
            }
            (OptionStyle::Put, Side::Short) => {
                self.short_put = position.clone();
                Ok(())
            }
            _ => Err(PositionError::invalid_position_type(
                position.option.side,
                "Position side is Long, it is not valid for ShortStrangle".to_string(),
            )),
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.short_call, &self.short_put])
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
            (Side::Long, _, _) => Err(PositionError::invalid_position_type(
                *side,
                "Position side is Long, it is not valid for ShortStrangle".to_string(),
            )),
            (Side::Short, OptionStyle::Call, strike)
                if *strike == self.short_call.option.strike_price =>
            {
                Ok(vec![&mut self.short_call])
            }
            (Side::Short, OptionStyle::Put, strike)
                if *strike == self.short_put.option.strike_price =>
            {
                Ok(vec![&mut self.short_put])
            }
            _ => Err(PositionError::invalid_position_type(
                *side,
                "Strike not found in positions".to_string(),
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

        if position.option.side == Side::Long {
            return Err(PositionError::invalid_position_type(
                position.option.side,
                "Position side is Long, it is not valid for ShortStrangle".to_string(),
            ));
        }

        if position.option.strike_price != self.short_call.option.strike_price
            && position.option.strike_price != self.short_put.option.strike_price
        {
            return Err(PositionError::invalid_position_type(
                position.option.side,
                "Strike not found in positions".to_string(),
            ));
        }

        if position.option.option_style == OptionStyle::Call {
            self.short_call = position.clone();
        }

        if position.option.option_style == OptionStyle::Put {
            self.short_put = position.clone();
        }

        Ok(())
    }
}

impl Strategable for ShortStrangle {
    fn info(&self) -> Result<StrategyBasics, StrategyError> {
        Ok(StrategyBasics {
            name: self.name.clone(),
            kind: self.kind.clone(),
            description: self.description.clone(),
        })
    }
}

impl Strategies for ShortStrangle {
    fn get_underlying_price(&self) -> Positive {
        self.short_call.option.underlying_price
    }

    fn max_profit(&self) -> Result<Positive, StrategyError> {
        let max_profit = self.net_premium_received().unwrap().to_f64();
        if max_profit < ZERO {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxProfitError {
                    reason: "Max profit is negative".to_string(),
                },
            ))
        } else {
            Ok(max_profit.into())
        }
    }

    fn max_loss(&self) -> Result<Positive, StrategyError> {
        Ok(Positive::INFINITY)
    }

    fn profit_area(&self) -> Result<Decimal, StrategyError> {
        let max_profit = self.max_profit().unwrap_or(Positive::ZERO);
        if max_profit == Positive::ZERO {
            return Ok(Decimal::ZERO);
        }
        let strike_diff = self.short_call.option.strike_price - self.short_put.option.strike_price;
        let inner_square = strike_diff * max_profit;
        let break_even_diff = self.break_even_points[1] - self.break_even_points[0];
        let outer_square = break_even_diff * max_profit;
        let triangles = (outer_square - inner_square) / 2.0;
        let result =
            ((inner_square + triangles) / self.short_call.option.underlying_price).to_f64();
        Ok(Decimal::from_f64(result).unwrap())
    }

    fn profit_ratio(&self) -> Result<Decimal, StrategyError> {
        let break_even_diff = self.break_even_points[1] - self.break_even_points[0];
        let result = match self.max_profit() {
            Ok(max_profit) => max_profit.to_f64() / break_even_diff * 100.0,
            Err(_) => ZERO,
        };
        Ok(Decimal::from_f64(result).unwrap())
    }

    fn best_range_to_show(&self, step: Positive) -> Result<Vec<Positive>, StrategyError> {
        let max_profit = self.max_profit().unwrap_or(Positive::ZERO);
        let (first_option, last_option) = (self.break_even_points[0], self.break_even_points[1]);
        let start_price = first_option - max_profit.to_dec();
        let end_price = last_option + max_profit;
        Ok(calculate_price_range(start_price, end_price, step))
    }

    fn expiration_dates(&self) -> Result<Vec<ExpirationDate>, StrategyError> {
        let options = [
            self.short_call.option.expiration_date,
            self.short_put.option.expiration_date,
        ];
        Ok(options.to_vec())
    }

    fn set_expiration_date(
        &mut self,
        expiration_date: ExpirationDate,
    ) -> Result<(), StrategyError> {
        self.short_call.option.expiration_date = expiration_date;
        self.short_put.option.expiration_date = expiration_date;
        Ok(())
    }
}

impl Validable for ShortStrangle {
    fn validate(&self) -> bool {
        self.short_call.validate()
            && self.short_put.validate()
            && self.short_call.option.strike_price > self.short_put.option.strike_price
    }
}

impl Optimizable for ShortStrangle {
    type Strategy = ShortStrangle;

    fn filter_combinations<'a>(
        &'a self,
        option_chain: &'a OptionChain,
        side: FindOptimalSide,
    ) -> impl Iterator<Item = OptionDataGroup<'a>> {
        let underlying_price = self.get_underlying_price();
        let strategy = self.clone();
        option_chain
            .get_double_iter()
            // Filter out invalid combinations based on FindOptimalSide
            .filter(move |(short_put, short_call)| {
                if side == FindOptimalSide::Center {
                    short_put.is_valid_optimal_side(underlying_price, &FindOptimalSide::Lower)
                        && short_call
                            .is_valid_optimal_side(underlying_price, &FindOptimalSide::Upper)
                } else {
                    short_put.is_valid_optimal_side(underlying_price, &side)
                        && short_call.is_valid_optimal_side(underlying_price, &side)
                }
            })
            .filter(move |(short_put, short_call)| short_put.strike_price < short_call.strike_price)
            // Filter out options with invalid bid/ask prices
            .filter(|(short_put, short_call)| {
                short_put.put_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && short_call.call_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
            })
            // Filter out options that don't meet strategy constraints
            .filter(move |(short_put, short_call)| {
                let legs = StrategyLegs::TwoLegs {
                    first: short_put,
                    second: short_call,
                };
                debug!("Legs: {:?}", legs);
                let strategy = strategy.create_strategy(option_chain, &legs);
                strategy.validate() && strategy.max_profit().is_ok() && strategy.max_loss().is_ok()
            })
            // Map to OptionDataGroup
            .map(move |(short_put, short_call)| OptionDataGroup::Two(short_put, short_call))
    }

    fn find_optimal(
        &mut self,
        option_chain: &OptionChain,
        side: FindOptimalSide,
        criteria: OptimizationCriteria,
    ) {
        let mut best_value = Decimal::MIN;
        let strategy_clone = self.clone();
        let options_iter = strategy_clone.filter_combinations(option_chain, side);

        for option_data_group in options_iter {
            // Unpack the OptionDataGroup into individual options
            let (short_put, short_call) = match option_data_group {
                OptionDataGroup::Two(first, second) => (first, second),
                _ => panic!("Invalid OptionDataGroup"),
            };

            let legs = StrategyLegs::TwoLegs {
                first: short_put,
                second: short_call,
            };
            let strategy = self.create_strategy(option_chain, &legs);
            // Calculate the current value based on the optimization criteria
            let current_value = match criteria {
                OptimizationCriteria::Ratio => strategy.profit_ratio().unwrap(),
                OptimizationCriteria::Area => strategy.profit_area().unwrap(),
            };

            if current_value > best_value {
                // Update the best value and replace the current strategy
                info!("Found better value: {}", current_value);
                best_value = current_value;
                *self = strategy.clone();
            }
        }
    }

    fn are_valid_prices(&self, legs: &StrategyLegs) -> bool {
        let (short_put, short_call) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };
        short_put.put_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
            && short_call.call_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
    }

    fn create_strategy(&self, chain: &OptionChain, legs: &StrategyLegs) -> Self::Strategy {
        let (put, call) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };
        if !call.validate() || !put.validate() {
            println!("Call: {}\nPut: {}", call, put);
            panic!("Invalid options");
        }
        let implied_volatility = call.implied_volatility.unwrap();
        assert!(implied_volatility <= Positive::ONE);

        ShortStrangle::new(
            chain.symbol.clone(),
            chain.underlying_price,
            call.strike_price,
            put.strike_price,
            self.short_call.option.expiration_date,
            implied_volatility,
            self.short_call.option.risk_free_rate,
            self.short_call.option.dividend_yield,
            self.short_call.option.quantity,
            call.call_bid.unwrap(),
            put.put_bid.unwrap(),
            self.short_call.open_fee,
            self.short_call.close_fee,
            self.short_put.open_fee,
            self.short_put.close_fee,
        )
    }
}

impl Profit for ShortStrangle {
    fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
        let price = Some(&price);
        trace!(
            "Price: {:?} Strike: {} Call: {:.2} Strike: {} Put: {:.2} Profit: {:.2}",
            price,
            self.short_call.option.strike_price,
            self.short_call.pnl_at_expiration(&price)?,
            self.short_put.option.strike_price,
            self.short_put.pnl_at_expiration(&price)?,
            self.short_call.pnl_at_expiration(&price)?
                + self.short_put.pnl_at_expiration(&price)?
        );
        Ok(
            self.short_call.pnl_at_expiration(&price)?
                + self.short_put.pnl_at_expiration(&price)?,
        )
    }
}

impl Graph for ShortStrangle {
    fn title(&self) -> String {
        let strategy_title = format!("{:?} Strategy: ", self.kind);
        let leg_titles: Vec<String> = [self.short_call.title(), self.short_put.title()]
            .iter()
            .map(|leg| leg.to_string())
            .collect();

        if leg_titles.is_empty() {
            strategy_title
        } else {
            format!("{}\n\t{}", strategy_title, leg_titles.join("\n\t"))
        }
    }

    fn get_x_values(&self) -> Vec<Positive> {
        self.best_range_to_show(Positive::from(1.0))
            .unwrap_or_else(|_| vec![self.short_call.option.strike_price])
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        let vertical_lines = vec![ChartVerticalLine {
            x_coordinate: self.short_call.option.underlying_price.to_f64(),
            y_range: (f64::NEG_INFINITY, f64::INFINITY),
            label: format!(
                "Current Price: {:.2}",
                self.short_call.option.underlying_price
            ),
            label_offset: (4.0, -1.0),
            line_color: ORANGE,
            label_color: ORANGE,
            line_style: ShapeStyle::from(&ORANGE).stroke_width(2),
            font_size: 18,
        }];

        vertical_lines
    }

    fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> {
        let mut points: Vec<ChartPoint<(f64, f64)>> = Vec::new();
        let max_profit = self.max_profit().unwrap_or(Positive::ZERO);

        let coordinates: (f64, f64) = (-3.0, 30.0);
        let font_size = 24;

        points.push(ChartPoint {
            coordinates: (self.break_even_points[0].to_f64(), 0.0),
            label: format!("Low Break Even\n\n{}", self.break_even_points[0]),
            label_offset: LabelOffsetType::Relative(coordinates.0, -coordinates.1),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size,
        });

        points.push(ChartPoint {
            coordinates: (self.break_even_points[1].to_f64(), 0.0),
            label: format!("High Break Even\n\n{}", self.break_even_points[1]),
            label_offset: LabelOffsetType::Relative(coordinates.0, coordinates.1),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size,
        });

        points.push(ChartPoint {
            coordinates: (
                self.short_call.option.strike_price.to_f64(),
                max_profit.to_f64(),
            ),
            label: format!(
                "Max Profit ${:.2} at {:.0}",
                max_profit, self.short_call.option.strike_price
            ),
            label_offset: LabelOffsetType::Relative(coordinates.0, -coordinates.1),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size,
        });

        points.push(ChartPoint {
            coordinates: (
                self.short_put.option.strike_price.to_f64(),
                max_profit.to_f64(),
            ),
            label: format!(
                "Max Profit ${:.2} at {:.0}",
                max_profit, self.short_put.option.strike_price
            ),
            label_offset: LabelOffsetType::Relative(coordinates.0, coordinates.1),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size,
        });

        points.push(ChartPoint {
            coordinates: (
                self.short_put.option.underlying_price.to_f64(),
                self.calculate_profit_at(self.short_put.option.underlying_price)
                    .unwrap()
                    .to_f64()
                    .unwrap(),
            ),
            label: format!(
                "${:.2}",
                self.calculate_profit_at(self.short_put.option.underlying_price)
                    .unwrap(),
            ),
            label_offset: LabelOffsetType::Relative(-coordinates.0, coordinates.1),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size,
        });

        points
    }
}

impl ProbabilityAnalysis for ShortStrangle {
    fn get_expiration(&self) -> Result<ExpirationDate, ProbabilityError> {
        let option = &self.short_call.option;
        Ok(option.expiration_date)
    }

    fn get_risk_free_rate(&self) -> Option<Decimal> {
        Some(self.short_call.option.risk_free_rate)
    }

    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let option = &self.short_call.option;
        let break_even_points = &self.get_break_even_points()?;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            option.implied_volatility,
            self.short_put.option.implied_volatility,
        ]);

        let mut profit_range = ProfitLossRange::new(
            Some(break_even_points[0]),
            Some(break_even_points[1]),
            Positive::ZERO,
        )?;

        profit_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        Ok(vec![profit_range])
    }

    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let option = &self.short_call.option;
        let break_even_points = self.get_break_even_points()?;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            option.implied_volatility,
            self.short_put.option.implied_volatility,
        ]);

        let mut lower_loss_range =
            ProfitLossRange::new(None, Some(break_even_points[0]), Positive::ZERO)?;

        lower_loss_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        let mut upper_loss_range =
            ProfitLossRange::new(Some(break_even_points[1]), None, Positive::ZERO)?;

        upper_loss_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        Ok(vec![lower_loss_range, upper_loss_range])
    }
}

impl Greeks for ShortStrangle {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.short_call.option, &self.short_put.option])
    }
}

impl DeltaNeutrality for ShortStrangle {}

impl PnLCalculator for ShortStrangle {
    fn calculate_pnl(
        &self,
        market_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        Ok(self
            .short_call
            .calculate_pnl(market_price, expiration_date, implied_volatility)?
            + self
                .short_put
                .calculate_pnl(market_price, expiration_date, implied_volatility)?)
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        Ok(self
            .short_call
            .calculate_pnl_at_expiration(underlying_price)?
            + self
                .short_put
                .calculate_pnl_at_expiration(underlying_price)?)
    }
}

const LONG_STRANGLE_DESCRIPTION: &str = "A long strangle involves buying an out-of-the-money call and an \
out-of-the-money put with the same expiration date. This strategy is used when high volatility \
is expected and a significant move in the underlying asset's price is anticipated, but the \
direction is uncertain.";

/// # LongStrangle
///
/// Represents a Long Strangle options trading strategy.
///
/// A Long Strangle strategy involves simultaneously buying an out-of-the-money call option
/// and an out-of-the-money put option on the same underlying asset with the same expiration date
/// but different strike prices. This strategy benefits from significant price movements in either
/// direction.
///
/// ## Fields
///
/// * `name` - Custom name identifier for this specific strategy instance
/// * `kind` - The type of strategy (always `StrategyType::LongStrangle` for this struct)
/// * `description` - Detailed description of this specific strategy instance
/// * `break_even_points` - Vector containing the price points where the strategy breaks even
///   (typically two points: call strike + call premium and put strike - put premium)
/// * `long_call` - The long call position component of the strategy
/// * `long_put` - The long put position component of the strategy
///
/// ## Risk Profile
///
/// * Maximum Loss: Limited to the total premium paid (call premium + put premium + fees)
/// * Maximum Profit: Theoretically unlimited on the upside; on the downside, limited to
///   the put strike price minus total premium paid
/// * Break-even Points: Upper break-even is call strike plus total premium paid;
///   lower break-even is put strike minus total premium paid
///
/// ## Typical Usage
///
/// Used when an investor expects significant price movement in the underlying asset
/// but is uncertain about the direction of the move. This strategy is often employed:
///
/// * Ahead of major market events with uncertain outcomes (earnings reports, regulatory decisions)
/// * During periods of expected high volatility
/// * When anticipating a break from a trading range
///
/// ## Example Scenario
///
/// If the underlying asset is trading at $100, a long strangle might involve:
/// * Buying a put with a strike price of $90 for a premium of $2
/// * Buying a call with a strike price of $110 for a premium of $2
///
/// Total cost: $4 per share ($400 per contract)
/// Break-even points: $86 and $114
///
/// ## Advantages and Disadvantages
///
/// ### Advantages
/// * Benefits from price movements in either direction
/// * Limited risk (maximum loss is the premium paid)
/// * No margin requirements (beyond the premium paid)
///
/// ### Disadvantages
/// * Requires significant price movement to be profitable
/// * Suffers from time decay (theta) as both options lose value over time
/// * Generally more expensive than directional strategies due to purchasing two options
#[derive(Clone, Debug)]
pub struct LongStrangle {
    /// Name identifier for this specific strategy instance
    pub name: String,
    /// Type of the strategy (always StrategyType::LongStrangle)
    pub kind: StrategyType,
    /// Detailed description of this strategy instance
    pub description: String,
    /// Price points where the strategy breaks even (typically two points)
    pub break_even_points: Vec<Positive>,
    /// The long call position component of the strategy
    long_call: Position,
    /// The long put position component of the strategy
    long_put: Position,
}

impl LongStrangle {
    /// ## Creation
    ///
    /// Creates a new Long Strangle strategy with two positions:
    /// 1. A long call option with strike typically above the current underlying price (OTM)
    /// 2. A long put option with strike typically below the current underlying price (OTM)
    ///
    /// If strike prices are not explicitly provided (passed as zero), the constructor will automatically set:
    /// - Call strike at 10% above the underlying price
    /// - Put strike at 10% below the underlying price
    ///
    /// ## Parameters
    ///
    /// * `underlying_symbol` - Symbol of the underlying asset
    /// * `underlying_price` - Current price of the underlying asset
    /// * `call_strike` - Strike price for the call option (will be set to 110% of underlying price if zero)
    /// * `put_strike` - Strike price for the put option (will be set to 90% of underlying price if zero)
    /// * `expiration` - Expiration date for both options
    /// * `implied_volatility` - Implied volatility for pricing models
    /// * `risk_free_rate` - Risk-free interest rate for pricing models
    /// * `dividend_yield` - Dividend yield for the underlying asset
    /// * `quantity` - Number of contracts to open for each position
    /// * `premium_long_call` - Premium paid for the call option
    /// * `premium_long_put` - Premium paid for the put option
    /// * `open_fee_long_call` - Fee for opening the call position
    /// * `close_fee_long_call` - Fee for closing the call position
    /// * `open_fee_long_put` - Fee for opening the put position
    /// * `close_fee_long_put` - Fee for closing the put position
    ///
    /// ## Returns
    ///
    /// Returns a fully initialized `LongStrangle` strategy with properly configured positions and calculated
    /// break-even points.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        mut call_strike: Positive,
        mut put_strike: Positive,
        expiration: ExpirationDate,
        implied_volatility: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        quantity: Positive,
        premium_long_call: Positive,
        premium_long_put: Positive,
        open_fee_long_call: Positive,
        close_fee_long_call: Positive,
        open_fee_long_put: Positive,
        close_fee_long_put: Positive,
    ) -> Self {
        if call_strike == Positive::ZERO {
            call_strike = underlying_price * 1.1;
        }
        if put_strike == Positive::ZERO {
            put_strike = underlying_price * 0.9;
        }
        let mut strategy = LongStrangle {
            name: "Long Strangle".to_string(),
            kind: StrategyType::LongStrangle,
            description: LONG_STRANGLE_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::default(),
            long_put: Position::default(),
        };

        let long_call_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            call_strike,
            expiration,
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
        );
        strategy
            .add_position(&long_call.clone())
            .expect("Invalid position");

        let long_put_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol,
            put_strike,
            expiration,
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
        );
        strategy
            .add_position(&long_put.clone())
            .expect("Invalid position");

        strategy
            .update_break_even_points()
            .expect("Unable to update break even points");

        strategy
    }
}

impl StrategyConstructor for LongStrangle {
    fn get_strategy(vec_options: &[Position]) -> Result<Self, StrategyError> {
        // Need exactly 2 options for a long strangle
        if vec_options.len() != 2 {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Long Strangle get_strategy".to_string(),
                    reason: "Must have exactly 2 options".to_string(),
                },
            ));
        }

        // Sort options by option style to identify call and put
        let mut sorted_options = vec_options.to_vec();
        sorted_options.sort_by(|a, b| {
            a.option
                .strike_price
                .partial_cmp(&b.option.strike_price)
                .unwrap()
        });

        let put_option = &sorted_options[0]; // Put will be first
        let call_option = &sorted_options[1]; // Call will be second

        // Validate one option is call and other is put
        if call_option.option.option_style != OptionStyle::Call
            || put_option.option.option_style != OptionStyle::Put
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Long Strangle get_strategy".to_string(),
                    reason: "One option must be a call and one must be a put".to_string(),
                },
            ));
        }

        // Validate both options are long
        if call_option.option.side != Side::Long || put_option.option.side != Side::Long {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Long Strangle get_strategy".to_string(),
                    reason: "Both options must be long positions".to_string(),
                },
            ));
        }

        // Validate call strike is higher than put strike
        if call_option.option.strike_price <= put_option.option.strike_price {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Long Strangle get_strategy".to_string(),
                    reason: "Call strike must be higher than put strike".to_string(),
                },
            ));
        }

        // Validate expiration dates match
        if call_option.option.expiration_date != put_option.option.expiration_date {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Long Strangle get_strategy".to_string(),
                    reason: "Options must have the same expiration date".to_string(),
                },
            ));
        }

        // Create positions
        let long_call = Position::new(
            call_option.option.clone(),
            call_option.premium,
            Utc::now(),
            call_option.open_fee,
            call_option.close_fee,
        );

        let long_put = Position::new(
            put_option.option.clone(),
            put_option.premium,
            Utc::now(),
            put_option.open_fee,
            put_option.close_fee,
        );

        // Create strategy
        let mut strategy = LongStrangle {
            name: "Long Strangle".to_string(),
            kind: StrategyType::LongStrangle,
            description: LONG_STRANGLE_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call,
            long_put,
        };

        // Validate and update break-even points
        strategy.validate();
        strategy.update_break_even_points()?;

        Ok(strategy)
    }
}

impl BreakEvenable for LongStrangle {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        let total_premium = self.net_cost()?;

        self.break_even_points.push(
            (self.long_put.option.strike_price - (total_premium / self.long_put.option.quantity))
                .round_to(2),
        );

        self.break_even_points.push(
            (self.long_call.option.strike_price + (total_premium / self.long_call.option.quantity))
                .round_to(2),
        );

        self.break_even_points.sort();
        Ok(())
    }
}

impl Positionable for LongStrangle {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match (&position.option.option_style, &position.option.side) {
            (OptionStyle::Call, Side::Long) => {
                self.long_call = position.clone();
                Ok(())
            }
            (OptionStyle::Put, Side::Long) => {
                self.long_put = position.clone();
                Ok(())
            }
            _ => Err(PositionError::invalid_position_type(
                position.option.side,
                "Position side is Short, it is not valid for LongStrangle".to_string(),
            )),
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.long_call, &self.long_put])
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
            (Side::Short, _, _) => Err(PositionError::invalid_position_type(
                *side,
                "Position side is Short, it is not valid for LongStrangle".to_string(),
            )),
            (Side::Long, OptionStyle::Call, strike)
                if *strike == self.long_call.option.strike_price =>
            {
                Ok(vec![&mut self.long_call])
            }
            (Side::Long, OptionStyle::Put, strike)
                if *strike == self.long_put.option.strike_price =>
            {
                Ok(vec![&mut self.long_put])
            }
            _ => Err(PositionError::invalid_position_type(
                *side,
                "Strike not found in positions".to_string(),
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

        if position.option.side == Side::Short {
            return Err(PositionError::invalid_position_type(
                position.option.side,
                "Position side is Short, it is not valid for LongStrangle".to_string(),
            ));
        }

        if position.option.strike_price != self.long_call.option.strike_price
            && position.option.strike_price != self.long_put.option.strike_price
        {
            return Err(PositionError::invalid_position_type(
                position.option.side,
                "Strike not found in positions".to_string(),
            ));
        }

        if position.option.option_style == OptionStyle::Call {
            self.long_call = position.clone();
        }

        if position.option.option_style == OptionStyle::Put {
            self.long_put = position.clone();
        }

        Ok(())
    }
}

impl Strategable for LongStrangle {
    fn info(&self) -> Result<StrategyBasics, StrategyError> {
        Ok(StrategyBasics {
            name: self.name.clone(),
            kind: self.kind.clone(),
            description: self.description.clone(),
        })
    }
}

impl Strategies for LongStrangle {
    fn get_underlying_price(&self) -> Positive {
        self.long_call.option.underlying_price
    }

    fn max_profit(&self) -> Result<Positive, StrategyError> {
        Ok(Positive::INFINITY) // Theoretically unlimited
    }

    fn max_loss(&self) -> Result<Positive, StrategyError> {
        Ok(self.total_cost()?)
    }

    fn profit_area(&self) -> Result<Decimal, StrategyError> {
        let max_loss = self.max_loss().unwrap_or(Positive::ZERO);
        if max_loss == Positive::ZERO {
            return Ok(Decimal::MAX);
        }
        let strike_diff = self.long_call.option.strike_price - self.long_put.option.strike_price;
        let inner_square = strike_diff * max_loss;
        let break_even_diff = self.break_even_points[1] - self.break_even_points[0];
        let outer_square = break_even_diff * max_loss;
        let triangles = (outer_square - inner_square) / 2.0;
        let loss_area =
            ((inner_square + triangles) / self.long_call.option.underlying_price).to_f64();
        let result = 1.0 / loss_area; // Invert the value to get the profit area: the lower, the better
        Ok(Decimal::from_f64(result).unwrap())
    }

    fn profit_ratio(&self) -> Result<Decimal, StrategyError> {
        let max_loss = self.max_loss().unwrap_or(Positive::ZERO);
        if max_loss == Positive::ZERO {
            return Ok(Decimal::MAX);
        }
        let break_even_diff = self.break_even_points[1] - self.break_even_points[0];
        let ratio = max_loss / break_even_diff * 100.0;
        let result = 1.0 / ratio; // Invert the value to get the profit ratio: the lower, the better
        Ok(Decimal::from_f64(result).unwrap())
    }

    fn best_range_to_show(&self, step: Positive) -> Result<Vec<Positive>, StrategyError> {
        let (first_option, last_option) = (self.break_even_points[0], self.break_even_points[1]);
        debug!("First: {} Last: {}", first_option, last_option);
        assert!(first_option < last_option);
        let diff = last_option - first_option.to_dec();
        debug!(
            "First break even point: {} Last break even point: {}",
            first_option, last_option
        );
        let start_price = first_option - diff.to_dec();
        debug!("Start price: {}", start_price);
        let end_price = last_option + diff;
        debug!("End price: {}", end_price);
        Ok(calculate_price_range(start_price, end_price, step))
    }
}

impl Validable for LongStrangle {
    fn validate(&self) -> bool {
        self.long_call.validate()
            && self.long_put.validate()
            && self.long_call.option.strike_price > self.long_put.option.strike_price
    }
}

impl Optimizable for LongStrangle {
    type Strategy = LongStrangle;

    fn filter_combinations<'a>(
        &'a self,
        option_chain: &'a OptionChain,
        side: FindOptimalSide,
    ) -> impl Iterator<Item = OptionDataGroup<'a>> {
        let underlying_price = self.get_underlying_price();
        let strategy = self.clone();
        option_chain
            .get_double_iter()
            // Filter out invalid combinations based on FindOptimalSide
            .filter(move |(long_put, long_call)| {
                if side == FindOptimalSide::Center {
                    long_put.is_valid_optimal_side(underlying_price, &FindOptimalSide::Lower)
                        && long_call
                            .is_valid_optimal_side(underlying_price, &FindOptimalSide::Upper)
                } else {
                    long_put.is_valid_optimal_side(underlying_price, &side)
                        && long_call.is_valid_optimal_side(underlying_price, &side)
                }
            })
            .filter(move |(long_put, long_call)| long_put.strike_price < long_call.strike_price)
            // Filter out options with invalid bid/ask prices
            .filter(|(long_put, long_call)| {
                long_put.call_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && long_call.call_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
            })
            // Filter out options that don't meet strategy constraints
            .filter(move |(long_put, long_call)| {
                let legs = StrategyLegs::TwoLegs {
                    first: long_put,
                    second: long_call,
                };
                let strategy = strategy.create_strategy(option_chain, &legs);
                strategy.validate() && strategy.max_profit().is_ok() && strategy.max_loss().is_ok()
            })
            // Map to OptionDataGroup
            .map(move |(long_put, long_call)| OptionDataGroup::Two(long_put, long_call))
    }

    fn find_optimal(
        &mut self,
        option_chain: &OptionChain,
        side: FindOptimalSide,
        criteria: OptimizationCriteria,
    ) {
        let mut best_value = Decimal::MIN;
        let strategy_clone = self.clone();
        let options_iter = strategy_clone.filter_combinations(option_chain, side);

        for option_data_group in options_iter {
            // Unpack the OptionDataGroup into individual options
            let (long_put, long_call) = match option_data_group {
                OptionDataGroup::Two(first, second) => (first, second),
                _ => panic!("Invalid OptionDataGroup"),
            };

            let legs = StrategyLegs::TwoLegs {
                first: long_put,
                second: long_call,
            };
            let strategy = self.create_strategy(option_chain, &legs);
            // Calculate the current value based on the optimization criteria
            let current_value = match criteria {
                OptimizationCriteria::Ratio => strategy.profit_ratio().unwrap(),
                OptimizationCriteria::Area => strategy.profit_area().unwrap(),
            };

            if current_value > best_value {
                // Update the best value and replace the current strategy
                info!("Found better value: {}", current_value);
                best_value = current_value;
                *self = strategy.clone();
            }
        }
    }

    fn are_valid_prices(&self, legs: &StrategyLegs) -> bool {
        let (long_put, long_call) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };
        long_call.call_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
            && long_put.put_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
    }

    fn create_strategy(&self, chain: &OptionChain, legs: &StrategyLegs) -> Self::Strategy {
        let (put, call) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };
        let implied_volatility = call.implied_volatility.unwrap();
        assert!(implied_volatility <= Positive::ONE);
        LongStrangle::new(
            chain.symbol.clone(),
            chain.underlying_price,
            call.strike_price,
            put.strike_price,
            self.long_call.option.expiration_date,
            implied_volatility,
            self.long_call.option.risk_free_rate,
            self.long_call.option.dividend_yield,
            self.long_call.option.quantity,
            call.call_ask.unwrap(),
            put.put_ask.unwrap(),
            self.long_call.open_fee,
            self.long_call.close_fee,
            self.long_put.open_fee,
            self.long_put.close_fee,
        )
    }
}

impl Profit for LongStrangle {
    fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
        let price = Some(&price);
        Ok(self.long_call.pnl_at_expiration(&price)? + self.long_put.pnl_at_expiration(&price)?)
    }
}

impl Graph for LongStrangle {
    fn title(&self) -> String {
        let strategy_title = format!("{:?} Strategy: ", self.kind);
        let leg_titles: Vec<String> = [self.long_call.title(), self.long_put.title()]
            .iter()
            .map(|leg| leg.to_string())
            .collect();

        if leg_titles.is_empty() {
            strategy_title
        } else {
            format!("{}\n\t{}", strategy_title, leg_titles.join("\n\t"))
        }
    }

    fn get_x_values(&self) -> Vec<Positive> {
        self.best_range_to_show(Positive::from(1.0))
            .unwrap_or_else(|_| vec![self.long_call.option.strike_price])
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        let max_value = f64::INFINITY;
        let min_value = f64::NEG_INFINITY;

        let vertical_lines = vec![ChartVerticalLine {
            x_coordinate: self.long_call.option.underlying_price.to_f64(),
            y_range: (min_value, max_value),
            label: format!(
                "Current Price: {:.2}",
                self.long_call.option.underlying_price
            ),
            label_offset: (4.0, -50.0),
            line_color: ORANGE,
            label_color: ORANGE,
            line_style: ShapeStyle::from(&ORANGE).stroke_width(2),
            font_size: 18,
        }];

        vertical_lines
    }

    fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> {
        let mut points: Vec<ChartPoint<(f64, f64)>> = Vec::new();
        let max_loss = self.max_loss().unwrap_or(Positive::ZERO);
        let coordinates: (f64, f64) = (-3.0, 150.0);
        let font_size = 24;

        points.push(ChartPoint {
            coordinates: (self.break_even_points[0].to_f64(), 0.0),
            label: format!("Low Break Even\n\n{}", self.break_even_points[0]),
            label_offset: LabelOffsetType::Relative(coordinates.0, -coordinates.1),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size,
        });

        points.push(ChartPoint {
            coordinates: (self.break_even_points[1].to_f64(), 0.0),
            label: format!("High Break Even\n\n{}", self.break_even_points[1]),
            label_offset: LabelOffsetType::Relative(coordinates.0, coordinates.1),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size,
        });

        points.push(ChartPoint {
            coordinates: (
                self.long_call.option.strike_price.to_f64(),
                -max_loss.to_f64(),
            ),
            label: format!(
                "Max Loss high ${:.2} at {:.0}",
                max_loss, self.long_call.option.strike_price
            ),
            label_offset: LabelOffsetType::Relative(coordinates.0, -coordinates.1),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size,
        });

        points.push(ChartPoint {
            coordinates: (
                self.long_put.option.strike_price.to_f64(),
                -max_loss.to_f64(),
            ),
            label: format!(
                "Max Loss low ${:.2} at {:.0}",
                max_loss, self.long_put.option.strike_price
            ),
            label_offset: LabelOffsetType::Relative(coordinates.0, coordinates.1),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size,
        });

        points.push(ChartPoint {
            coordinates: (
                self.long_put.option.underlying_price.to_f64(),
                self.calculate_profit_at(self.long_put.option.underlying_price)
                    .unwrap()
                    .to_f64()
                    .unwrap(),
            ),
            label: format!(
                "${:.2}",
                self.calculate_profit_at(self.long_put.option.underlying_price)
                    .unwrap(),
            ),
            label_offset: LabelOffsetType::Relative(-coordinates.0, coordinates.1),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size,
        });

        points
    }
}

impl ProbabilityAnalysis for LongStrangle {
    fn get_expiration(&self) -> Result<ExpirationDate, ProbabilityError> {
        let option = &self.long_call.option;
        Ok(option.expiration_date)
    }

    fn get_risk_free_rate(&self) -> Option<Decimal> {
        Some(self.long_call.option.risk_free_rate)
    }

    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let option = &self.long_call.option;
        let break_even_points = &self.get_break_even_points()?;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            option.implied_volatility,
            self.long_put.option.implied_volatility,
        ]);

        let mut lower_profit_range =
            ProfitLossRange::new(None, Some(break_even_points[0]), Positive::ZERO)?;

        lower_profit_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        let mut upper_profit_range =
            ProfitLossRange::new(Some(break_even_points[1]), None, Positive::ZERO)?;

        upper_profit_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        Ok(vec![lower_profit_range, upper_profit_range])
    }

    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let option = &self.long_call.option;
        let break_even_points = &self.get_break_even_points()?;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            option.implied_volatility,
            self.long_call.option.implied_volatility,
        ]);

        let mut loss_range = ProfitLossRange::new(
            Some(break_even_points[0]),
            Some(break_even_points[1]),
            Positive::ZERO,
        )?;

        loss_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        Ok(vec![loss_range])
    }
}

impl Greeks for LongStrangle {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.long_call.option, &self.long_put.option])
    }
}

impl DeltaNeutrality for LongStrangle {}

impl PnLCalculator for LongStrangle {
    fn calculate_pnl(
        &self,
        market_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        Ok(self
            .long_call
            .calculate_pnl(market_price, expiration_date, implied_volatility)?
            + self
                .long_put
                .calculate_pnl(market_price, expiration_date, implied_volatility)?)
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        Ok(self
            .long_call
            .calculate_pnl_at_expiration(underlying_price)?
            + self
                .long_put
                .calculate_pnl_at_expiration(underlying_price)?)
    }
}

#[cfg(test)]
mod tests_short_strangle {
    use super::*;
    use crate::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
    use crate::{pos, spos};
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn setup() -> ShortStrangle {
        ShortStrangle::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(155.0),
            pos!(145.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.01),
            pos!(0.02),
            pos!(100.0),
            Positive::TWO,
            pos!(1.5),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
        )
    }

    fn wrong_setup() -> ShortStrangle {
        ShortStrangle::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(145.0),
            pos!(155.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.01),
            pos!(0.02),
            pos!(100.0),
            Positive::TWO,
            pos!(1.5),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
        )
    }

    #[test]

    fn test_new() {
        let strategy = setup();
        assert_eq!(strategy.name, "Short Strangle");
        assert_eq!(strategy.kind, StrategyType::ShortStrangle);
        assert_eq!(
            strategy.description,
            "A short strangle involves selling an out-of-the-money call and an \
out-of-the-money put with the same expiration date. This strategy is used when low volatility \
is expected and the underlying asset's price is anticipated to remain stable."
        );
    }

    #[test]

    fn test_validate() {
        let strategy = setup();
        let wrong_strategy = wrong_setup();
        assert!(strategy.validate());
        assert!(!wrong_strategy.validate());
    }

    #[test]

    fn test_get_break_even_points() {
        let strategy = setup();
        assert_eq!(strategy.get_break_even_points().unwrap()[0], 141.9);
    }

    #[test]

    fn test_calculate_profit_at() {
        let strategy = setup();
        let price = 150.0;
        assert_eq!(
            strategy
                .calculate_profit_at(pos!(price))
                .unwrap()
                .to_f64()
                .unwrap(),
            310.0
        );
    }

    #[test]

    fn test_max_profit() {
        let strategy = setup();
        assert_eq!(
            strategy.max_profit().unwrap_or(Positive::ZERO),
            strategy.net_premium_received().unwrap().to_f64()
        );
    }

    #[test]

    fn test_max_loss() {
        let strategy = setup();
        assert_eq!(
            strategy.max_loss().unwrap_or(Positive::ZERO),
            Positive::INFINITY
        );
    }

    #[test]

    fn test_total_cost() {
        let strategy = setup();
        assert_eq!(strategy.total_cost().unwrap(), 40.0);
    }

    #[test]

    fn test_net_premium_received() {
        let strategy = setup();
        assert_eq!(
            strategy.net_premium_received().unwrap().to_f64(),
            strategy.short_call.net_premium_received().unwrap()
                + strategy.short_put.net_premium_received().unwrap()
        );
    }

    #[test]

    fn test_fees() {
        let strategy = setup();
        let expected_fees = 40.0;
        assert_eq!(strategy.fees().unwrap().to_f64(), expected_fees);
    }

    #[test]

    fn test_area() {
        let strategy = setup();
        assert_eq!(
            strategy.profit_area().unwrap().to_f64().unwrap(),
            27.07333333333334
        );
    }

    #[test]

    fn test_graph_methods() {
        let strategy = setup();

        let vertical_lines = strategy.get_vertical_lines();
        assert_eq!(vertical_lines.len(), 1);
        assert_eq!(vertical_lines[0].label, "Current Price: 150");

        let data = strategy.get_x_values();
        let values = strategy.get_y_values();
        for (i, &price) in data.iter().enumerate() {
            assert_eq!(
                values[i],
                strategy
                    .calculate_profit_at(price)
                    .unwrap()
                    .to_f64()
                    .unwrap()
            );
        }

        let title = strategy.title();
        assert!(title.contains("ShortStrangle Strategy"));
        assert!(title.contains("Call"));
        assert!(title.contains("Put"));
    }

    #[test]

    fn test_add_leg() {
        let mut strategy = setup();
        let original_call = strategy.short_call.clone();
        let original_put = strategy.short_put.clone();

        // Test adding a new call leg
        strategy
            .add_position(&original_call.clone())
            .expect("Invalid position");
        assert_eq!(strategy.short_call, original_call);

        // Test adding a new put leg
        strategy
            .add_position(&original_put.clone())
            .expect("Invalid position");
        assert_eq!(strategy.short_put, original_put);
    }

    #[test]

    fn test_profit_ratio() {
        let strategy = setup();
        let break_even_diff = strategy.break_even_points[1] - strategy.break_even_points[0];
        let expected_ratio =
            strategy.max_profit().unwrap_or(Positive::ZERO) / break_even_diff * 100.0;
        assert_relative_eq!(
            strategy.profit_ratio().unwrap().to_f64().unwrap(),
            expected_ratio.to_f64(),
            epsilon = 0.0001
        );
    }

    #[test]

    fn test_best_ratio() {
        let mut strategy = setup();
        let option_chain = create_test_option_chain();

        strategy.best_ratio(&option_chain, FindOptimalSide::All);
        assert!(strategy.validate());
    }

    #[test]

    fn test_best_area() {
        let mut strategy = setup();
        let option_chain = create_test_option_chain();

        strategy.best_area(&option_chain, FindOptimalSide::All);
        assert!(strategy.validate());
    }

    #[test]

    fn test_is_valid_short_option() {
        let strategy = setup();
        let option_chain = create_test_option_chain();
        let option_data = option_chain.options.first().unwrap();
        let min_strike = option_chain.options.first().unwrap().strike_price;
        let max_strike = option_chain.options.last().unwrap().strike_price;

        // Test FindOptimalSide::Upper
        assert!(strategy.is_valid_short_option(option_data, &FindOptimalSide::Lower));

        // Test FindOptimalSide::Lower
        assert!(!strategy.is_valid_short_option(option_data, &FindOptimalSide::Upper));

        // Test FindOptimalSide::All
        assert!(strategy.is_valid_short_option(option_data, &FindOptimalSide::All));

        // Test FindOptimalSide::Range
        assert!(
            strategy.is_valid_short_option(
                option_data,
                &FindOptimalSide::Range(min_strike, max_strike)
            )
        );
    }

    #[test]

    fn test_create_strategy() {
        let strategy = setup();
        let chain = create_test_option_chain();
        let call_option = chain.atm_option_data().unwrap();
        let put_option = chain.options.last().unwrap();

        let legs = StrategyLegs::TwoLegs {
            first: call_option,
            second: put_option,
        };

        let new_strategy = strategy.create_strategy(&chain, &legs);
        assert!(new_strategy.validate());

        let call_option = chain.options.last().unwrap();
        let put_option = chain.atm_option_data().unwrap();

        let legs = StrategyLegs::TwoLegs {
            first: call_option,
            second: put_option,
        };

        let new_strategy = strategy.create_strategy(&chain, &legs);
        assert!(!new_strategy.validate());
    }

    #[test]

    fn test_get_points() {
        let strategy = setup();
        let points = strategy.get_points();

        assert_eq!(points.len(), 5);

        let break_even_points: Vec<f64> = points[0..2].iter().map(|p| p.coordinates.0).collect();
        assert!(break_even_points.contains(&strategy.break_even_points[0].to_f64()));
        assert!(break_even_points.contains(&strategy.break_even_points[1].to_f64()));
    }

    fn create_test_option_chain() -> OptionChain {
        let option_data_price_params = OptionDataPriceParams::new(
            pos!(1150.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            dec!(0.01),
            pos!(0.02),
            None,
        );
        let option_chain_build_params = OptionChainBuildParams::new(
            "AAPL".to_string(),
            spos!(1.0),
            10,
            pos!(10.0),
            dec!(0.00001),
            pos!(0.01),
            2,
            option_data_price_params,
        );
        OptionChain::build_chain(&option_chain_build_params)
    }
}

#[cfg(test)]
mod tests_long_strangle {
    use super::*;
    use crate::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
    use crate::{pos, spos};
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    #[test]

    fn test_long_strangle_new() {
        let underlying_symbol = "AAPL".to_string();
        let underlying_price = pos!(150.0);
        let call_strike = pos!(160.0);
        let put_strike = pos!(140.0);
        let expiration = ExpirationDate::default();
        let implied_volatility = pos!(0.25);
        let risk_free_rate = dec!(0.01);
        let dividend_yield = pos!(0.02);
        let quantity = pos!(10.0);
        let premium_long_call = pos!(5.0);
        let premium_long_put = pos!(5.0);
        let open_fee_long_call = pos!(0.5);
        let close_fee_long_call = pos!(0.5);
        let open_fee_long_put = pos!(0.5);
        let close_fee_long_put = pos!(0.5);

        let strategy = LongStrangle::new(
            underlying_symbol.clone(),
            underlying_price,
            call_strike,
            put_strike,
            expiration,
            implied_volatility,
            risk_free_rate,
            dividend_yield,
            quantity,
            premium_long_call,
            premium_long_put,
            open_fee_long_call,
            close_fee_long_call,
            open_fee_long_put,
            close_fee_long_put,
        );

        assert_eq!(strategy.name, "Long Strangle");
        assert_eq!(strategy.kind, StrategyType::LongStrangle);
        assert_eq!(strategy.description, LONG_STRANGLE_DESCRIPTION);

        let break_even_points = vec![128.0, 172.0];
        assert_eq!(strategy.break_even_points, break_even_points);
    }

    #[test]

    fn test_get_break_even_points() {
        let long_strangle = setup_long_strangle();
        assert_eq!(long_strangle.get_break_even_points().unwrap()[0], 128.0);
    }

    #[test]

    fn test_total_cost() {
        let long_strangle = setup_long_strangle();
        assert_eq!(
            long_strangle.total_cost().unwrap(),
            long_strangle.long_call.net_cost().unwrap()
                + long_strangle.long_put.net_cost().unwrap()
        );
    }

    #[test]

    fn test_calculate_profit_at() {
        let long_strangle = setup_long_strangle();
        let price = pos!(150.0);
        let expected_profit = long_strangle
            .long_call
            .pnl_at_expiration(&Some(&price))
            .unwrap()
            + long_strangle
                .long_put
                .pnl_at_expiration(&Some(&price))
                .unwrap();
        assert_eq!(
            long_strangle.calculate_profit_at(price).unwrap(),
            expected_profit
        );
    }

    fn setup_long_strangle() -> LongStrangle {
        LongStrangle::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(160.0),
            pos!(140.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.25),
            dec!(0.01),
            pos!(0.02),
            pos!(10.0),
            pos!(5.0),
            pos!(5.0),
            pos!(0.5),
            pos!(0.5),
            pos!(0.5),
            pos!(0.5),
        )
    }

    fn wrong_setup_long_strangle() -> LongStrangle {
        // Setup with put strike higher than call strike
        LongStrangle::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(140.0), // Call strike lower than put
            pos!(160.0), // Put strike higher than call
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.25),
            dec!(0.01),
            pos!(0.02),
            pos!(10.0),
            pos!(5.0),
            pos!(5.0),
            pos!(0.5),
            pos!(0.5),
            pos!(0.5),
            pos!(0.5),
        )
    }

    #[test]

    fn test_new() {
        let strategy = setup_long_strangle();
        assert_eq!(strategy.name, "Long Strangle");
        assert_eq!(strategy.kind, StrategyType::LongStrangle);
        assert_eq!(strategy.description, LONG_STRANGLE_DESCRIPTION);
    }

    #[test]

    fn test_validate() {
        let strategy = setup_long_strangle();
        let wrong_strategy = wrong_setup_long_strangle();
        assert!(strategy.validate());
        assert!(!wrong_strategy.validate());
    }

    #[test]

    fn test_max_profit() {
        let strategy = setup_long_strangle();
        assert_eq!(
            strategy.max_profit().unwrap_or(Positive::ZERO),
            Positive::INFINITY
        );
    }

    #[test]

    fn test_max_loss() {
        let strategy = setup_long_strangle();
        assert_eq!(
            strategy.max_loss().unwrap_or(Positive::ZERO),
            strategy.total_cost().unwrap()
        );
    }

    #[test]

    fn test_fees() {
        let strategy = setup_long_strangle();
        let expected_fees = 20.0; // 0.5 * 4 fees * 10 qty
        assert_eq!(strategy.fees().unwrap().to_f64(), expected_fees);
    }

    #[test]

    fn test_net_premium_received() {
        let strategy = setup_long_strangle();
        assert_eq!(strategy.net_premium_received().unwrap().to_f64(), 0.0);
    }

    #[test]

    fn test_profit_area() {
        let strategy = setup_long_strangle();
        let area = strategy.profit_area().unwrap();
        assert!(area > Decimal::ZERO);
    }

    #[test]

    fn test_profit_ratio() {
        let strategy = setup_long_strangle();
        let expected_ratio = 0.003666666666666666;
        assert_eq!(
            strategy.profit_ratio().unwrap().to_f64().unwrap(),
            expected_ratio
        );
    }

    #[test]

    fn test_add_leg() {
        let mut strategy = setup_long_strangle();
        let original_call = strategy.long_call.clone();
        let original_put = strategy.long_put.clone();

        strategy
            .add_position(&original_call.clone())
            .expect("Invalid position");
        assert_eq!(strategy.long_call, original_call);

        strategy
            .add_position(&original_put.clone())
            .expect("Invalid position");
        assert_eq!(strategy.long_put, original_put);
    }

    #[test]

    fn test_graph_methods() {
        let strategy = setup_long_strangle();

        // Test vertical lines
        let vertical_lines = strategy.get_vertical_lines();
        assert_eq!(vertical_lines.len(), 1);
        assert_eq!(vertical_lines[0].label, "Current Price: 150");

        // Test values calculation
        let data = strategy.get_x_values();
        let values = strategy.get_y_values();
        for (i, &price) in data.iter().enumerate() {
            assert_eq!(
                values[i],
                strategy
                    .calculate_profit_at(price)
                    .unwrap()
                    .to_f64()
                    .unwrap()
            );
        }

        // Test title
        let title = strategy.title();
        assert!(title.contains("LongStrangle Strategy"));
        assert!(title.contains("Call"));
        assert!(title.contains("Put"));
    }

    #[test]

    fn test_best_ratio() {
        let mut strategy = setup_long_strangle();
        let option_chain = create_test_option_chain();

        strategy.best_ratio(&option_chain, FindOptimalSide::Upper);
        assert!(strategy.validate());
    }

    #[test]

    fn test_best_area() {
        let mut strategy = setup_long_strangle();
        let option_chain = create_test_option_chain();

        strategy.best_area(&option_chain, FindOptimalSide::Upper);
        assert!(strategy.validate());
    }

    #[test]

    fn test_best_range_to_show() {
        let strategy = setup_long_strangle();
        let step = pos!(1.0);

        let range = strategy.best_range_to_show(step).unwrap();
        assert!(!range.is_empty());
        assert!(range[0] <= strategy.break_even_points[0]);
        assert!(*range.last().unwrap() >= strategy.break_even_points[1]);
    }

    #[test]

    fn test_is_valid_long_option() {
        let strategy = setup_long_strangle();
        let option_chain = create_test_option_chain();
        let option_data = option_chain.options.last().unwrap();
        let min_strike = option_chain.options.first().unwrap().strike_price;
        let max_strike = option_chain.options.last().unwrap().strike_price;

        assert!(strategy.is_valid_long_option(option_data, &FindOptimalSide::Upper));
        assert!(!strategy.is_valid_long_option(option_data, &FindOptimalSide::Lower));
        assert!(strategy.is_valid_long_option(option_data, &FindOptimalSide::All));
        assert!(
            strategy
                .is_valid_long_option(option_data, &FindOptimalSide::Range(min_strike, max_strike))
        );
    }

    #[test]

    fn test_are_valid_prices() {
        let strategy = setup_long_strangle();
        let option_chain = create_test_option_chain();
        let call_option = option_chain.atm_option_data().unwrap();
        let put_option = option_chain.options.last().unwrap();

        let legs = StrategyLegs::TwoLegs {
            first: call_option,
            second: put_option,
        };

        assert!(strategy.are_valid_prices(&legs));

        let legs = StrategyLegs::TwoLegs {
            first: option_chain.options.first().unwrap(),
            second: put_option,
        };
        assert!(!strategy.are_valid_prices(&legs));
    }

    #[test]

    fn test_create_strategy() {
        let strategy = setup_long_strangle();
        let chain = create_test_option_chain();
        let call_option = chain.atm_option_data().unwrap();
        let put_option = chain.options.last().unwrap();
        let legs = StrategyLegs::TwoLegs {
            first: call_option,
            second: put_option,
        };
        let new_strategy = strategy.create_strategy(&chain, &legs);
        assert!(new_strategy.validate());
        let call_option = chain.options.last().unwrap();
        let put_option = chain.atm_option_data().unwrap();

        let legs = StrategyLegs::TwoLegs {
            first: call_option,
            second: put_option,
        };
        let new_strategy = strategy.create_strategy(&chain, &legs);
        assert!(!new_strategy.validate());
    }

    #[test]

    fn test_get_points() {
        let strategy = setup_long_strangle();
        let points = strategy.get_points();

        // Should have 5 points: 2 break-even, 2 max loss, 1 current price
        assert_eq!(points.len(), 5);

        let break_even_points: Vec<f64> = points[0..2].iter().map(|p| p.coordinates.0).collect();
        assert!(break_even_points.contains(&strategy.break_even_points[0].to_f64()));
        assert!(break_even_points.contains(&strategy.break_even_points[1].to_f64()));
    }

    fn create_test_option_chain() -> OptionChain {
        let option_data_price_params = OptionDataPriceParams::new(
            pos!(150.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.65),
            dec!(0.01),
            pos!(0.02),
            None,
        );
        let option_chain_build_params = OptionChainBuildParams::new(
            "AAPL".to_string(),
            spos!(1.0),
            10,
            pos!(5.0),
            dec!(0.00001),
            pos!(0.01),
            2,
            option_data_price_params,
        );
        OptionChain::build_chain(&option_chain_build_params)
    }
}

#[cfg(test)]
mod tests_short_strangle_probability {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use crate::strategies::probabilities::utils::PriceTrend;
    use rust_decimal_macros::dec;

    /// Helper function that creates a basic short strangle for testing purposes
    /// Returns a ShortStrangle instance with predefined test values
    fn create_test() -> ShortStrangle {
        ShortStrangle::new(
            "TEST".to_string(),
            pos!(100.0),                      // underlying_price
            pos!(110.0),                      // call_strike
            pos!(90.0),                       // put_strike
            ExpirationDate::Days(pos!(30.0)), // expiration
            pos!(0.2),                        // implied_volatility
            dec!(0.05),                       // risk_free_rate
            Positive::ZERO,                   // dividend_yield
            pos!(1.0),                        // quantity
            Positive::TWO,                    // premium_short_call
            Positive::TWO,                    // premium_short_put
            Positive::ZERO,                   // open_fee_short_call
            Positive::ZERO,                   // close_fee_short_call
            Positive::ZERO,                   // open_fee_short_put
            Positive::ZERO,                   // close_fee_short_put
        )
    }

    #[test]

    fn test_probability_of_profit_basic() {
        let strangle = create_test();
        let result = strangle.probability_of_profit(None, None);

        assert!(result.is_ok(), "Probability calculation should succeed");
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO, "Probability should be positive");
        assert!(prob <= pos!(1.0), "Probability should not exceed 1.0");
    }

    #[test]

    fn test_probability_of_profit_with_volatility_adjustment() {
        let strangle = create_test();
        let vol_adj = VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.05),
        };

        let result = strangle.probability_of_profit(Some(vol_adj), None);

        assert!(
            result.is_ok(),
            "Probability calculation with volatility adjustment should succeed"
        );
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO, "Probability should be positive");
        assert!(prob <= pos!(1.0), "Probability should not exceed 1.0");
    }

    #[test]

    fn test_probability_of_profit_with_trend() {
        let strangle = create_test();
        let trend = PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        };

        let result = strangle.probability_of_profit(None, Some(trend));

        assert!(
            result.is_ok(),
            "Probability calculation with trend should succeed"
        );
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO, "Probability should be positive");
        assert!(prob <= pos!(1.0), "Probability should not exceed 1.0");
    }

    #[test]

    fn test_probability_of_profit_with_downward_trend() {
        let strangle = create_test();
        let trend = PriceTrend {
            drift_rate: -0.1,
            confidence: 0.90,
        };

        let result = strangle.probability_of_profit(None, Some(trend));

        assert!(
            result.is_ok(),
            "Probability calculation with downward trend should succeed"
        );
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO, "Probability should be positive");
        assert!(prob <= pos!(1.0), "Probability should not exceed 1.0");
    }

    #[test]

    fn test_get_reference_price() {
        let strangle = create_test();
        let result = strangle.get_underlying_price();

        assert_eq!(
            result,
            pos!(100.0),
            "Reference price should match underlying price"
        );
    }

    #[test]

    fn test_get_expiration() {
        let strangle = create_test();
        let result = strangle.get_expiration();

        assert!(result.is_ok(), "Expiration retrieval should succeed");
        match result.unwrap() {
            ExpirationDate::Days(days) => assert_eq!(days, 30.0),
            _ => panic!("Expected ExpirationDate::Days"),
        }
    }

    #[test]

    fn test_get_profit_ranges() {
        let strangle = create_test();
        let result = strangle.get_profit_ranges();

        assert!(result.is_ok(), "Profit ranges calculation should succeed");
        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1, "Should have exactly one profit range");

        let range = &ranges[0];
        assert!(range.lower_bound.is_some(), "Lower bound should be defined");
        assert!(range.upper_bound.is_some(), "Upper bound should be defined");
        assert!(
            range.probability > Positive::ZERO,
            "Probability should be positive"
        );
    }
}

#[cfg(test)]
mod tests_short_strangle_probability_bis {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use crate::strategies::probabilities::utils::PriceTrend;
    use rust_decimal_macros::dec;

    fn create_test() -> ShortStrangle {
        ShortStrangle::new(
            "TEST".to_string(),
            pos!(100.0),                      // underlying_price
            pos!(110.0),                      // call_strike
            pos!(90.0),                       // put_strike
            ExpirationDate::Days(pos!(30.0)), // expiration
            pos!(0.2),                        // implied_volatility
            dec!(0.05),                       // risk_free_rate
            Positive::ZERO,                   // dividend_yield
            pos!(1.0),                        // quantity
            Positive::TWO,                    // premium_short_call
            Positive::TWO,                    // premium_short_put
            Positive::ZERO,                   // open_fee_short_call
            Positive::ZERO,                   // close_fee_short_call
            Positive::ZERO,                   // open_fee_short_put
            Positive::ZERO,                   // close_fee_short_put
        )
    }

    #[test]

    fn test_get_expiration() {
        let strangle = create_test();
        let result = strangle.get_expiration();
        assert!(result.is_ok());
        match result.unwrap() {
            ExpirationDate::Days(days) => assert_eq!(days, 30.0),
            _ => panic!("Expected ExpirationDate::Days"),
        }
    }

    #[test]

    fn test_get_risk_free_rate() {
        let strangle = create_test();
        assert_eq!(strangle.get_risk_free_rate(), Some(dec!(0.05)));
    }

    #[test]

    fn test_get_profit_ranges() {
        let strangle = create_test();
        let result = strangle.get_profit_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1); // Short strangle has one profit range

        let range = &ranges[0];
        assert!(range.lower_bound.is_some());
        assert!(range.upper_bound.is_some());
    }

    #[test]

    fn test_get_loss_ranges() {
        let strangle = create_test();
        let result = strangle.get_loss_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 2); // Short strangle has two loss ranges

        // Verify ranges have correct bounds
        assert!(ranges[0].lower_bound.is_none()); // First loss range extends to negative infinity
        assert!(ranges[1].upper_bound.is_none()); // Second loss range extends to positive infinity
    }

    #[test]

    fn test_probability_of_profit() {
        let strangle = create_test();
        let result = strangle.probability_of_profit(None, None);

        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]

    fn test_probability_with_volatility_adjustment() {
        let strangle = create_test();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.1),
        });

        let result = strangle.probability_of_profit(vol_adj, None);
        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]

    fn test_probability_with_trend() {
        let strangle = create_test();
        let trend = Some(PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        });

        let result = strangle.probability_of_profit(None, trend);
        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]

    fn test_analyze_probabilities() {
        let strangle = create_test();
        let result = strangle.analyze_probabilities(None, None);

        assert!(result.is_ok());
        let analysis = result.unwrap();

        assert!(analysis.probability_of_profit > Positive::ZERO);
        assert!(analysis.probability_of_max_profit >= Positive::ZERO);
        assert!(analysis.probability_of_max_loss >= Positive::ZERO);
        assert!(analysis.expected_value > Positive::ZERO);
        assert!(!analysis.break_even_points.is_empty());
        assert!(analysis.risk_reward_ratio > Positive::ZERO);
    }

    #[test]

    fn test_calculate_extreme_probabilities() {
        let strangle = create_test();
        let result = strangle.calculate_extreme_probabilities(None, None);

        assert!(result.is_ok());
        let (max_profit_prob, max_loss_prob) = result.unwrap();
        assert!(max_profit_prob >= Positive::ZERO);
        assert!(max_loss_prob >= Positive::ZERO);
        assert!(max_profit_prob + max_loss_prob <= pos!(1.0));
    }
}

#[cfg(test)]
mod tests_long_strangle_probability {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use crate::strategies::probabilities::utils::PriceTrend;
    use rust_decimal_macros::dec;

    fn create_test_long_strangle() -> LongStrangle {
        LongStrangle::new(
            "TEST".to_string(),
            pos!(100.0),                      // underlying_price
            pos!(110.0),                      // call_strike
            pos!(90.0),                       // put_strike
            ExpirationDate::Days(pos!(30.0)), // expiration
            pos!(0.2),                        // implied_volatility
            dec!(0.05),                       // risk_free_rate
            Positive::ZERO,                   // dividend_yield
            pos!(1.0),                        // quantity
            Positive::TWO,                    // premium_long_call
            Positive::TWO,                    // premium_long_put
            Positive::ZERO,                   // open_fee_long_call
            Positive::ZERO,                   // close_fee_long_call
            Positive::ZERO,                   // open_fee_long_put
            Positive::ZERO,                   // close_fee_long_put
        )
    }

    #[test]

    fn test_get_expiration() {
        let strangle = create_test_long_strangle();
        let result = strangle.get_expiration();
        assert!(result.is_ok());
        match result.unwrap() {
            ExpirationDate::Days(days) => assert_eq!(days, 30.0),
            _ => panic!("Expected ExpirationDate::Days"),
        }
    }

    #[test]

    fn test_get_risk_free_rate() {
        let strangle = create_test_long_strangle();
        assert_eq!(strangle.get_risk_free_rate(), Some(dec!(0.05)));
    }

    #[test]

    fn test_get_profit_ranges() {
        let strangle = create_test_long_strangle();
        let result = strangle.get_profit_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 2); // Long strangle has two profit ranges

        // Verify ranges have correct bounds
        assert!(ranges[0].upper_bound.is_some());
        assert!(ranges[1].lower_bound.is_some());
    }

    #[test]

    fn test_get_loss_ranges() {
        let strangle = create_test_long_strangle();
        let result = strangle.get_loss_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1); // Long strangle has one loss range
        assert!(ranges[0].lower_bound.is_some());
        assert!(ranges[0].upper_bound.is_some());
    }

    #[test]

    fn test_probability_of_profit() {
        let strangle = create_test_long_strangle();
        let result = strangle.probability_of_profit(None, None);

        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]

    fn test_probability_with_volatility_adjustment() {
        let strangle = create_test_long_strangle();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.1),
        });

        let result = strangle.probability_of_profit(vol_adj, None);
        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]

    fn test_probability_with_trend() {
        let strangle = create_test_long_strangle();
        let trend = Some(PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        });

        let result = strangle.probability_of_profit(None, trend);
        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]

    fn test_expected_value_calculation() {
        let strangle = create_test_long_strangle();
        let result = strangle.expected_value(None, None);

        assert!(result.is_ok());
        let ev = result.unwrap();
        assert!(
            ev >= Positive::ZERO,
            "Expected value should be non-negative"
        );

        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.1),
        });
        let result_with_vol = strangle.expected_value(vol_adj, None);
        assert!(result_with_vol.is_ok());
        assert!(result_with_vol.unwrap() >= Positive::ZERO);
    }

    #[test]

    fn test_calculate_extreme_probabilities() {
        let strangle = create_test_long_strangle();
        let result = strangle.calculate_extreme_probabilities(None, None);

        assert!(result.is_ok());
        let (max_profit_prob, max_loss_prob) = result.unwrap();
        assert!(max_profit_prob >= Positive::ZERO);
        assert!(max_loss_prob >= Positive::ZERO);
        assert!(max_profit_prob + max_loss_prob <= pos!(1.0));
    }
}

#[cfg(test)]
mod tests_short_strangle_delta {
    use super::*;
    use crate::greeks::Greeks;
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::strategies::strangle::Positive;
    use crate::strategies::strangle::ShortStrangle;
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn get_strategy(call_strike: Positive, put_strike: Positive) -> ShortStrangle {
        let underlying_price = pos!(7138.5);
        ShortStrangle::new(
            "CL".to_string(),
            underlying_price, // underlying_price
            call_strike,      // call_strike 7450 (delta -0.415981)
            put_strike,       // put_strike 7050 (delta 0.417810)
            ExpirationDate::Days(pos!(45.0)),
            pos!(0.3745),   // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(84.2),     // premium_short_call
            pos!(353.2),    // premium_short_put
            pos!(7.01),     // open_fee_short_call
            pos!(7.01),     // close_fee_short_call
            pos!(7.01),     // open_fee_short_put
            pos!(7.01),     // close_fee_short_put
        )
    }

    #[test]

    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos!(7450.0), pos!(7250.0));
        let size = dec!(0.086108);
        let delta = pos!(0.171500192678);
        let k = pos!(7250.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        match &binding[1] {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Put);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        let short_call = strategy.short_call.option.clone();
        let mut short_put = strategy.short_put.option.clone();
        short_put.quantity = short_put.quantity - delta.to_dec();
        assert_decimal_eq!(
            short_call.delta().unwrap(),
            -short_put.delta().unwrap(),
            DELTA_THRESHOLD
        );
    }

    #[test]

    fn create_test_increasing_adjustments() {
        let strike = pos!(7050.0);
        let strategy = get_strategy(pos!(7150.0), strike);
        let net_delta = dec!(-0.1221);
        let call_qty = pos!(0.22624889861);
        let put_qty = pos!(0.2924052685);

        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            net_delta,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        assert_eq!(binding.len(), 3);
        for suggestion in binding.iter() {
            match suggestion {
                DeltaAdjustment::SellOptions {
                    quantity,
                    strike,
                    option_style,
                    side,
                } => {
                    assert_pos_relative_eq!(*quantity, call_qty, Positive(DELTA_THRESHOLD));
                    assert_pos_relative_eq!(*strike, pos!(7150.0), Positive(DELTA_THRESHOLD));
                    assert_eq!(*option_style, OptionStyle::Call);
                    assert_eq!(*side, Side::Short);
                }
                DeltaAdjustment::BuyOptions {
                    quantity,
                    strike,
                    option_style,
                    side,
                } => {
                    assert_pos_relative_eq!(*quantity, put_qty, Positive(DELTA_THRESHOLD));
                    assert_pos_relative_eq!(*strike, pos!(7050.0), Positive(DELTA_THRESHOLD));
                    assert_eq!(*option_style, OptionStyle::Put);
                    assert_eq!(*side, Side::Short);
                }
                DeltaAdjustment::SameSize(adj1, adj2) => {
                    // Get quantities from both adjustments
                    let qty1 = match **adj1 {
                        DeltaAdjustment::BuyOptions { quantity, .. } => quantity,
                        DeltaAdjustment::SellOptions { quantity, .. } => quantity,
                        _ => panic!("Invalid adjustment type in SameSize"),
                    };

                    let qty2 = match **adj2 {
                        DeltaAdjustment::BuyOptions { quantity, .. } => quantity,
                        DeltaAdjustment::SellOptions { quantity, .. } => quantity,
                        _ => panic!("Invalid adjustment type in SameSize"),
                    };

                    // Assert that quantities are equal
                    assert_eq!(
                        qty1, qty2,
                        "Adjustments in SameSize must have equal quantities"
                    );
                }
                _ => panic!("Invalid suggestion"),
            }
        }

        let mut option = strategy.short_put.option.clone();
        option.quantity = put_qty;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, -net_delta, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]

    fn create_test_no_adjustments() {
        let strategy = get_strategy(pos!(7445.5), pos!(7050.0));

        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
        assert!(strategy.is_delta_neutral());
        let suggestion = strategy.delta_adjustments().unwrap();
        assert_eq!(suggestion[0], DeltaAdjustment::NoAdjustmentNeeded);
    }
}

#[cfg(test)]
mod tests_long_strangle_delta {
    use super::*;
    use crate::greeks::Greeks;
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::strategies::strangle::{LongStrangle, Positive};
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn get_strategy(call_strike: Positive, put_strike: Positive) -> LongStrangle {
        let underlying_price = pos!(7138.5);
        LongStrangle::new(
            "CL".to_string(),
            underlying_price, // underlying_price
            call_strike,      // call_strike 7450 (delta -0.415981)
            put_strike,       // put_strike 7050 (delta 0.417810)
            ExpirationDate::Days(pos!(45.0)),
            pos!(0.3745),   // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(84.2),     // premium_short_call
            pos!(353.2),    // premium_short_put
            pos!(7.01),     // open_fee_short_call
            pos!(7.01),     // close_fee_short_call
            pos!(7.01),     // open_fee_short_put
            pos!(7.01),     // close_fee_short_put
        )
    }

    #[test]

    fn create_test_reducing_adjustments() {
        let strike = pos!(7450.0);
        let strategy = get_strategy(strike, pos!(7250.0));
        let size = dec!(-0.0861);
        let delta = pos!(0.2070008842036104);
        let k = pos!(7450.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        info!("{:?}", strategy.delta_neutrality().unwrap());
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        let suggestion = binding.first().unwrap();
        info!("{:?}", suggestion);
        match suggestion {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Long);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.long_call.option.clone();
        option.quantity = delta;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, -size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]

    fn create_test_increasing_adjustments() {
        let strike = pos!(7050.0);
        let strategy = get_strategy(pos!(7150.0), strike);
        let size = dec!(0.1221);
        let delta = pos!(0.29240526858778937);
        let k = pos!(7050.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        match binding[1] {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(quantity, delta, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(strike, k, Positive(DELTA_THRESHOLD));
                assert_eq!(option_style, OptionStyle::Put);
                assert_eq!(side, Side::Long);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.long_put.option.clone();
        option.quantity = delta;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, -size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]

    fn create_test_no_adjustments() {
        let strategy = get_strategy(pos!(7445.5), pos!(7050.0));

        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
        assert!(strategy.is_delta_neutral());
        let suggestion = strategy.delta_adjustments().unwrap();
        assert_eq!(suggestion[0], DeltaAdjustment::NoAdjustmentNeeded);
    }
}

#[cfg(test)]
mod tests_short_strangle_delta_size {
    use super::*;
    use crate::greeks::Greeks;
    use crate::model::types::{Action, ExpirationDate, OptionStyle};
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::strategies::strangle::Positive;
    use crate::strategies::strangle::ShortStrangle;
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn get_strategy(call_strike: Positive, put_strike: Positive) -> ShortStrangle {
        let underlying_price = pos!(7138.5);
        ShortStrangle::new(
            "CL".to_string(),
            underlying_price, // underlying_price
            call_strike,      // call_strike 7450 (delta -0.415981)
            put_strike,       // put_strike 7050 (delta 0.417810)
            ExpirationDate::Days(pos!(45.0)),
            pos!(0.3745),   // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(2.0),      // quantity
            pos!(84.2),     // premium_short_call
            pos!(353.2),    // premium_short_put
            pos!(7.01),     // open_fee_short_call
            pos!(7.01),     // close_fee_short_call
            pos!(7.01),     // open_fee_short_put
            pos!(7.01),     // close_fee_short_put
        )
    }

    #[test]

    fn create_test_apply_adjustments() {
        let strategy = get_strategy(pos!(7450.0), pos!(7250.0));
        let binding = strategy.delta_adjustments().unwrap();
        for suggestion in binding.iter() {
            match suggestion {
                DeltaAdjustment::BuyOptions {
                    quantity,
                    strike,
                    option_style,
                    side,
                } => {
                    assert_pos_relative_eq!(
                        *quantity,
                        pos!(0.41400176840722),
                        Positive(DELTA_THRESHOLD)
                    );
                    assert_pos_relative_eq!(*strike, pos!(7450.0), Positive(DELTA_THRESHOLD));
                    assert_eq!(*option_style, OptionStyle::Call);
                    assert_eq!(*side, Side::Short);

                    let mut temp_strategy = strategy.clone();
                    let result = temp_strategy.adjust_option_position(
                        quantity.to_dec(),
                        strike,
                        option_style,
                        side,
                    );
                    assert!(result.is_ok());
                    assert!(temp_strategy.is_delta_neutral());
                }
                DeltaAdjustment::SellOptions {
                    quantity,
                    strike,
                    option_style,
                    side,
                } => {
                    assert_pos_relative_eq!(
                        *quantity,
                        pos!(0.3430003853562),
                        Positive(DELTA_THRESHOLD)
                    );
                    assert_pos_relative_eq!(*strike, pos!(7250.0), Positive(DELTA_THRESHOLD));
                    assert_eq!(*option_style, OptionStyle::Put);
                    assert_eq!(*side, Side::Short);

                    let mut temp_strategy = strategy.clone();
                    let result = temp_strategy.adjust_option_position(
                        -quantity.to_dec(),
                        strike,
                        option_style,
                        side,
                    );
                    assert!(result.is_ok());
                    assert!(temp_strategy.is_delta_neutral());
                }
                DeltaAdjustment::SameSize(first, second) => {
                    let call_short_qty = match **first {
                        DeltaAdjustment::BuyOptions {
                            quantity,
                            strike,
                            option_style,
                            side,
                        } => {
                            assert_pos_relative_eq!(
                                quantity,
                                pos!(0.1875856830735),
                                Positive(DELTA_THRESHOLD)
                            );
                            assert_pos_relative_eq!(
                                strike,
                                pos!(7450.0),
                                Positive(DELTA_THRESHOLD)
                            );
                            assert_eq!(option_style, OptionStyle::Call);
                            assert_eq!(side, Side::Short);
                            quantity
                        }
                        _ => panic!("Invalid first adjustment"),
                    };
                    let put_short_qty = match **second {
                        DeltaAdjustment::SellOptions {
                            quantity,
                            strike,
                            option_style,
                            side,
                        } => {
                            assert_pos_relative_eq!(
                                quantity,
                                pos!(0.187585683073),
                                Positive(DELTA_THRESHOLD)
                            );
                            assert_pos_relative_eq!(
                                strike,
                                pos!(7250.0),
                                Positive(DELTA_THRESHOLD)
                            );
                            assert_eq!(option_style, OptionStyle::Put);
                            assert_eq!(side, Side::Short);
                            quantity
                        }
                        _ => {
                            panic!("Invalid suggestion")
                        }
                    };

                    let mut temp_strategy = strategy.clone();
                    let result = temp_strategy.adjust_option_position(
                        call_short_qty.to_dec(),
                        &pos!(7450.0),
                        &OptionStyle::Call,
                        &Side::Short,
                    );
                    assert!(result.is_ok());
                    let result = temp_strategy.adjust_option_position(
                        -put_short_qty.to_dec(),
                        &pos!(7250.0),
                        &OptionStyle::Put,
                        &Side::Short,
                    );
                    assert!(result.is_ok());
                    assert!(temp_strategy.is_delta_neutral());
                }
                _ => panic!("Invalid suggestion"),
            }
        }
    }

    #[test]

    fn create_test_apply_delta_adjustments() {
        let mut strategy = get_strategy(pos!(7450.0), pos!(7250.0));
        let result = strategy.apply_delta_adjustments(Some(Action::Buy));
        assert!(result.is_ok());
        assert!(strategy.is_delta_neutral());

        let mut strategy = get_strategy(pos!(7450.0), pos!(7250.0));
        let result = strategy.apply_delta_adjustments(Some(Action::Sell));
        assert!(result.is_ok());
        assert!(strategy.is_delta_neutral());

        let mut strategy = get_strategy(pos!(7450.0), pos!(7250.0));
        let result = strategy.apply_delta_adjustments(None);
        assert!(result.is_ok());
        assert!(strategy.is_delta_neutral());
    }

    #[test]

    fn create_test_reducing_adjustments() {
        let strike = pos!(7450.0);
        let mut strategy = get_strategy(strike, pos!(7250.0));
        let size_call = dec!(0.1722170236);
        let delta_call = pos!(0.4140017684072);
        let k_call = pos!(7450.0);

        // let size_put = dec!(-0.1722170236744605883028172966);
        let delta_put = pos!(0.343_000_385_356_289_6);
        let k_put = pos!(7250.0);

        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size_call,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        let first_suggestion = binding.first().unwrap();
        match first_suggestion {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta_call, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k_call, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut short_call = strategy.short_call.option.clone();
        short_call.quantity += delta_call;
        let short_put = strategy.short_put.option.clone();
        assert_decimal_eq!(
            short_call.delta().unwrap(),
            -short_put.delta().unwrap(),
            DELTA_THRESHOLD
        );

        let result = strategy.adjust_option_position(
            -delta_put.to_dec(),
            &k_put,
            &OptionStyle::Put,
            &Side::Short,
        );
        assert!(result.is_ok());
        assert!(strategy.is_delta_neutral());
    }

    #[test]

    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(pos!(7150.0), pos!(7050.0));
        let size = dec!(-0.24434);
        let delta = pos!(0.5848105371755788);
        let k = pos!(7050.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        match &binding[1] {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Put);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.short_put.option.clone();
        option.quantity = delta;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, -size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]

    fn create_test_no_adjustments() {
        let strategy = get_strategy(pos!(7450.7), pos!(7045.0));

        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
        assert!(strategy.is_delta_neutral());
        let suggestion = strategy.delta_adjustments().unwrap();
        assert_eq!(suggestion[0], DeltaAdjustment::NoAdjustmentNeeded);
    }
}

#[cfg(test)]
mod tests_long_strangle_delta_size {
    use super::*;
    use crate::greeks::Greeks;
    use crate::model::types::{Action, ExpirationDate, OptionStyle};
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::strategies::strangle::{LongStrangle, Positive};
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn get_strategy(call_strike: Positive, put_strike: Positive) -> LongStrangle {
        let underlying_price = pos!(7138.5);
        LongStrangle::new(
            "CL".to_string(),
            underlying_price, // underlying_price
            call_strike,      // call_strike 7450 (delta -0.415981)
            put_strike,       // put_strike 7050 (delta 0.417810)
            ExpirationDate::Days(pos!(45.0)),
            pos!(0.3745),   // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(2.0),      // quantity
            pos!(84.2),     // premium_short_call
            pos!(353.2),    // premium_short_put
            pos!(7.01),     // open_fee_short_call
            pos!(7.01),     // close_fee_short_call
            pos!(7.01),     // open_fee_short_put
            pos!(7.01),     // close_fee_short_put
        )
    }

    #[test]

    fn create_test_apply_adjustments() {
        let strategy = get_strategy(pos!(7450.0), pos!(7250.0));
        let binding = strategy.delta_adjustments().unwrap();
        for suggestion in binding.iter() {
            match suggestion {
                DeltaAdjustment::BuyOptions {
                    quantity,
                    strike,
                    option_style,
                    side,
                } => {
                    assert_pos_relative_eq!(
                        *quantity,
                        pos!(0.41400176840722),
                        Positive(DELTA_THRESHOLD)
                    );
                    assert_pos_relative_eq!(*strike, pos!(7450.0), Positive(DELTA_THRESHOLD));
                    assert_eq!(*option_style, OptionStyle::Call);
                    assert_eq!(*side, Side::Long);

                    let mut temp_strategy = strategy.clone();
                    let result = temp_strategy.adjust_option_position(
                        quantity.to_dec(),
                        strike,
                        option_style,
                        side,
                    );
                    assert!(result.is_ok());
                    assert!(temp_strategy.is_delta_neutral());
                }
                DeltaAdjustment::SellOptions {
                    quantity,
                    strike,
                    option_style,
                    side,
                } => {
                    assert_pos_relative_eq!(
                        *quantity,
                        pos!(0.3430003853562),
                        Positive(DELTA_THRESHOLD)
                    );
                    assert_pos_relative_eq!(*strike, pos!(7250.0), Positive(DELTA_THRESHOLD));
                    assert_eq!(*option_style, OptionStyle::Put);
                    assert_eq!(*side, Side::Long);

                    let mut temp_strategy = strategy.clone();
                    let result = temp_strategy.adjust_option_position(
                        -quantity.to_dec(),
                        strike,
                        option_style,
                        side,
                    );
                    assert!(result.is_ok());
                    assert!(temp_strategy.is_delta_neutral());
                }
                DeltaAdjustment::SameSize(first, second) => {
                    let call_short_qty = match **first {
                        DeltaAdjustment::BuyOptions {
                            quantity,
                            strike,
                            option_style,
                            side,
                        } => {
                            assert_pos_relative_eq!(
                                quantity,
                                pos!(0.1875856830735),
                                Positive(DELTA_THRESHOLD)
                            );
                            assert_pos_relative_eq!(
                                strike,
                                pos!(7450.0),
                                Positive(DELTA_THRESHOLD)
                            );
                            assert_eq!(option_style, OptionStyle::Call);
                            assert_eq!(side, Side::Long);
                            quantity
                        }
                        _ => panic!("Invalid first adjustment"),
                    };
                    let put_short_qty = match **second {
                        DeltaAdjustment::SellOptions {
                            quantity,
                            strike,
                            option_style,
                            side,
                        } => {
                            assert_pos_relative_eq!(
                                quantity,
                                pos!(0.187585683073),
                                Positive(DELTA_THRESHOLD)
                            );
                            assert_pos_relative_eq!(
                                strike,
                                pos!(7250.0),
                                Positive(DELTA_THRESHOLD)
                            );
                            assert_eq!(option_style, OptionStyle::Put);
                            assert_eq!(side, Side::Long);
                            quantity
                        }
                        _ => {
                            panic!("Invalid suggestion")
                        }
                    };

                    let mut temp_strategy = strategy.clone();
                    let result = temp_strategy.adjust_option_position(
                        call_short_qty.to_dec(),
                        &pos!(7450.0),
                        &OptionStyle::Call,
                        &Side::Long,
                    );
                    assert!(result.is_ok());
                    let result = temp_strategy.adjust_option_position(
                        -put_short_qty.to_dec(),
                        &pos!(7250.0),
                        &OptionStyle::Put,
                        &Side::Long,
                    );
                    assert!(result.is_ok());
                    assert!(temp_strategy.is_delta_neutral());
                }
                _ => panic!("Invalid suggestion"),
            }
        }
    }

    #[test]

    fn create_test_apply_delta_adjustments() {
        let mut strategy = get_strategy(pos!(7450.0), pos!(7250.0));
        let result = strategy.apply_delta_adjustments(Some(Action::Buy));
        assert!(result.is_ok());
        assert!(strategy.is_delta_neutral());

        let mut strategy = get_strategy(pos!(7450.0), pos!(7250.0));
        let result = strategy.apply_delta_adjustments(Some(Action::Sell));
        assert!(result.is_ok());
        assert!(strategy.is_delta_neutral());

        let mut strategy = get_strategy(pos!(7450.0), pos!(7250.0));
        let result = strategy.apply_delta_adjustments(None);
        assert!(result.is_ok());
        assert!(strategy.is_delta_neutral());
    }

    #[test]

    fn create_test_reducing_adjustments() {
        let strike = pos!(7450.0);
        let strategy = get_strategy(strike, pos!(7250.0));
        let size = dec!(-0.17221);
        let delta = pos!(0.4140017684072208);
        let k = pos!(7450.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        let suggestion = binding.first().unwrap();
        match suggestion {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Long);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.long_call.option.clone();
        option.quantity = delta;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, -size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]

    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(pos!(7150.0), pos!(7050.0));
        let size = dec!(0.244340);
        let delta = pos!(0.452497797238);

        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        let first_suggestion = binding.first().unwrap();

        match first_suggestion {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, pos!(7150.0), Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Long);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.long_call.option.clone();
        option.quantity = delta;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta - strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]

    fn create_test_no_adjustments() {
        let strategy = get_strategy(pos!(7445.5), pos!(7050.0));

        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
        assert!(strategy.is_delta_neutral());
        let suggestion = strategy.delta_adjustments().unwrap();
        assert_eq!(suggestion[0], DeltaAdjustment::NoAdjustmentNeeded);
    }
}

#[cfg(test)]
mod tests_strangle_position_management {
    use super::*;
    use crate::error::position::PositionValidationErrorKind;
    use crate::model::types::{ExpirationDate, OptionStyle, Side};
    use crate::pos;
    use rust_decimal_macros::dec;

    fn create_test_short_strangle() -> ShortStrangle {
        ShortStrangle::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(110.0), // call_strike
            pos!(90.0),  // put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(2.0),      // premium_short_call
            pos!(2.0),      // premium_short_put
            pos!(0.1),      // open_fee_short_call
            pos!(0.1),      // close_fee_short_call
            pos!(0.1),      // open_fee_short_put
            pos!(0.1),      // close_fee_short_put
        )
    }

    fn create_test_long_strangle() -> LongStrangle {
        LongStrangle::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(110.0), // call_strike
            pos!(90.0),  // put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(2.0),      // premium_long_call
            pos!(2.0),      // premium_long_put
            pos!(0.1),      // open_fee_long_call
            pos!(0.1),      // close_fee_long_call
            pos!(0.1),      // open_fee_long_put
            pos!(0.1),      // close_fee_long_put
        )
    }

    #[test]
    fn test_short_strangle_get_position() {
        let mut strangle = create_test_short_strangle();

        // Test getting short call position
        let call_position = strangle.get_position(&OptionStyle::Call, &Side::Short, &pos!(110.0));
        assert!(call_position.is_ok());
        let positions = call_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(110.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Short);

        // Test getting short put position
        let put_position = strangle.get_position(&OptionStyle::Put, &Side::Short, &pos!(90.0));
        assert!(put_position.is_ok());
        let positions = put_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(90.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Put);
        assert_eq!(positions[0].option.side, Side::Short);

        // Test getting non-existent position
        let invalid_position =
            strangle.get_position(&OptionStyle::Call, &Side::Short, &pos!(100.0));
        assert!(invalid_position.is_err());
        match invalid_position {
            Err(PositionError::ValidationError(
                PositionValidationErrorKind::IncompatibleSide {
                    position_side: _,
                    reason,
                },
            )) => {
                assert_eq!(reason, "Strike not found in positions");
            }
            _ => {
                panic!()
            }
        }
    }

    #[test]
    fn test_short_strangle_modify_position() {
        let mut strangle = create_test_short_strangle();

        // Modify short call position
        let mut modified_call = strangle.short_call.clone();
        modified_call.option.quantity = pos!(2.0);
        let result = strangle.modify_position(&modified_call);
        assert!(result.is_ok());
        assert_eq!(strangle.short_call.option.quantity, pos!(2.0));

        // Modify short put position
        let mut modified_put = strangle.short_put.clone();
        modified_put.option.quantity = pos!(2.0);
        let result = strangle.modify_position(&modified_put);
        assert!(result.is_ok());
        assert_eq!(strangle.short_put.option.quantity, pos!(2.0));

        // Test modifying with invalid position
        let mut invalid_position = strangle.short_call.clone();
        invalid_position.option.strike_price = pos!(95.0);
        let result = strangle.modify_position(&invalid_position);
        assert!(result.is_err());
        match result {
            Err(PositionError::ValidationError(kind)) => match kind {
                PositionValidationErrorKind::IncompatibleSide {
                    position_side: _,
                    reason,
                } => {
                    assert_eq!(reason, "Strike not found in positions");
                }
                _ => panic!("Expected ValidationError::InvalidPosition"),
            },
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_long_strangle_get_position() {
        let mut strangle = create_test_long_strangle();

        // Test getting long call position
        let call_position = strangle.get_position(&OptionStyle::Call, &Side::Long, &pos!(110.0));
        assert!(call_position.is_ok());
        let positions = call_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(110.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Long);

        // Test getting long put position
        let put_position = strangle.get_position(&OptionStyle::Put, &Side::Long, &pos!(90.0));
        assert!(put_position.is_ok());
        let positions = put_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(90.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Put);
        assert_eq!(positions[0].option.side, Side::Long);

        // Test getting non-existent position
        let invalid_position = strangle.get_position(&OptionStyle::Call, &Side::Long, &pos!(100.0));
        assert!(invalid_position.is_err());
        match invalid_position {
            Err(PositionError::ValidationError(
                PositionValidationErrorKind::IncompatibleSide {
                    position_side: _,
                    reason,
                },
            )) => {
                assert_eq!(reason, "Strike not found in positions");
            }
            _ => {
                panic!()
            }
        }
    }

    #[test]
    fn test_long_strangle_modify_position() {
        let mut strangle = create_test_long_strangle();

        // Modify long call position
        let mut modified_call = strangle.long_call.clone();
        modified_call.option.quantity = pos!(2.0);
        let result = strangle.modify_position(&modified_call);
        assert!(result.is_ok());
        assert_eq!(strangle.long_call.option.quantity, pos!(2.0));

        // Modify long put position
        let mut modified_put = strangle.long_put.clone();
        modified_put.option.quantity = pos!(2.0);
        let result = strangle.modify_position(&modified_put);
        assert!(result.is_ok());
        assert_eq!(strangle.long_put.option.quantity, pos!(2.0));

        // Test modifying with invalid position
        let mut invalid_position = strangle.long_call.clone();
        invalid_position.option.strike_price = pos!(95.0);
        let result = strangle.modify_position(&invalid_position);
        assert!(result.is_err());
        match result {
            Err(PositionError::ValidationError(kind)) => match kind {
                PositionValidationErrorKind::IncompatibleSide {
                    position_side: _,
                    reason,
                } => {
                    assert_eq!(reason, "Strike not found in positions");
                }
                _ => panic!("Expected ValidationError::InvalidPosition"),
            },
            _ => panic!("Expected ValidationError"),
        }
    }
}

#[cfg(test)]
mod tests_adjust_option_position_short {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle, Side};
    use crate::pos;
    use rust_decimal_macros::dec;

    // Helper function to create a test strategy
    fn create_test_strategy() -> ShortStrangle {
        ShortStrangle::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(110.0), // call_strike
            pos!(90.0),  // put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(2.0),      // premium_short_call
            pos!(2.0),      // premium_short_put
            pos!(0.1),      // open_fee_short_call
            pos!(0.1),      // close_fee_short_call
            pos!(0.1),      // open_fee_short_put
            pos!(0.1),      // close_fee_short_put
        )
    }

    #[test]
    fn test_adjust_existing_call_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.short_call.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos!(110.0),
            &OptionStyle::Call,
            &Side::Short,
        );

        assert!(result.is_ok());
        assert_eq!(
            strategy.short_call.option.quantity,
            initial_quantity + adjustment
        );
    }

    #[test]
    fn test_adjust_existing_put_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.short_put.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos!(90.0),
            &OptionStyle::Put,
            &Side::Short,
        );

        assert!(result.is_ok());
        assert_eq!(
            strategy.short_put.option.quantity,
            initial_quantity + adjustment
        );
    }

    #[test]
    fn test_adjust_nonexistent_position() {
        let mut strategy = create_test_strategy();

        // Try to adjust a non-existent long call position
        let result = strategy.adjust_option_position(
            Decimal::ONE,
            &pos!(110.0),
            &OptionStyle::Call,
            &Side::Long,
        );

        assert!(result.is_err());
        match result.unwrap_err().downcast_ref::<PositionError>() {
            Some(PositionError::ValidationError(
                PositionValidationErrorKind::IncompatibleSide {
                    position_side: _,
                    reason,
                },
            )) => {
                assert_eq!(
                    reason,
                    "Position side is Long, it is not valid for ShortStrangle"
                );
            }
            _ => panic!("Expected PositionError::ValidationError"),
        }
    }

    #[test]
    fn test_adjust_with_invalid_strike() {
        let mut strategy = create_test_strategy();

        // Try to adjust position with wrong strike price
        let result = strategy.adjust_option_position(
            Decimal::ONE,
            &pos!(100.0), // Invalid strike price
            &OptionStyle::Call,
            &Side::Short,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_zero_quantity_adjustment() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.short_call.option.quantity;

        let result = strategy.adjust_option_position(
            Decimal::ZERO,
            &pos!(110.0),
            &OptionStyle::Call,
            &Side::Short,
        );

        assert!(result.is_ok());
        assert_eq!(strategy.short_call.option.quantity, initial_quantity);
    }
}

#[cfg(test)]
mod tests_adjust_option_position_long {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle, Side};
    use crate::pos;
    use rust_decimal_macros::dec;

    // Helper function to create a test strategy
    fn create_test_strategy() -> LongStrangle {
        LongStrangle::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(110.0), // call_strike
            pos!(90.0),  // put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(2.0),      // premium_short_call
            pos!(2.0),      // premium_short_put
            pos!(0.1),      // open_fee_short_call
            pos!(0.1),      // close_fee_short_call
            pos!(0.1),      // open_fee_short_put
            pos!(0.1),      // close_fee_short_put
        )
    }

    #[test]
    fn test_adjust_existing_call_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.long_call.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos!(110.0),
            &OptionStyle::Call,
            &Side::Long,
        );

        assert!(result.is_ok());
        assert_eq!(
            strategy.long_call.option.quantity,
            initial_quantity + adjustment
        );
    }

    #[test]
    fn test_adjust_existing_put_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.long_put.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos!(90.0),
            &OptionStyle::Put,
            &Side::Long,
        );

        assert!(result.is_ok());
        assert_eq!(
            strategy.long_put.option.quantity,
            initial_quantity + adjustment
        );
    }

    #[test]
    fn test_adjust_nonexistent_position() {
        let mut strategy = create_test_strategy();

        // Try to adjust a non-existent long call position
        let result = strategy.adjust_option_position(
            Decimal::ONE,
            &pos!(110.0),
            &OptionStyle::Call,
            &Side::Short,
        );

        assert!(result.is_err());
        match result.unwrap_err().downcast_ref::<PositionError>() {
            Some(PositionError::ValidationError(
                PositionValidationErrorKind::IncompatibleSide {
                    position_side: _,
                    reason,
                },
            )) => {
                assert_eq!(
                    reason,
                    "Position side is Short, it is not valid for LongStrangle"
                );
            }
            _ => panic!("Expected PositionError::ValidationError"),
        }
    }

    #[test]
    fn test_adjust_with_invalid_strike() {
        let mut strategy = create_test_strategy();

        // Try to adjust position with wrong strike price
        let result = strategy.adjust_option_position(
            Decimal::ONE,
            &pos!(100.0), // Invalid strike price
            &OptionStyle::Call,
            &Side::Short,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_zero_quantity_adjustment() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.long_call.option.quantity;

        let result = strategy.adjust_option_position(
            Decimal::ZERO,
            &pos!(110.0),
            &OptionStyle::Call,
            &Side::Long,
        );

        assert!(result.is_ok());
        assert_eq!(strategy.long_call.option.quantity, initial_quantity);
    }
}

#[cfg(test)]
mod tests_strategy_constructor {
    use super::*;
    use crate::model::utils::create_sample_position;
    use crate::{OptionStyle, Side, pos};

    mod long_strangle_tests {
        use super::*;

        #[test]
        fn test_valid_long_strangle() {
            let options = vec![
                create_sample_position(
                    OptionStyle::Call,
                    Side::Long,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(110.0),
                    pos!(0.2),
                ),
                create_sample_position(
                    OptionStyle::Put,
                    Side::Long,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(90.0),
                    pos!(0.2),
                ),
            ];

            let result = LongStrangle::get_strategy(&options);
            assert!(result.is_ok());

            let strategy = result.unwrap();
            assert_eq!(strategy.long_call.option.strike_price, pos!(110.0));
            assert_eq!(strategy.long_put.option.strike_price, pos!(90.0));
        }

        #[test]
        fn test_wrong_number_of_options() {
            let options = vec![create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(100.0),
                pos!(1.0),
                pos!(110.0),
                pos!(0.2),
            )];

            let result = LongStrangle::get_strategy(&options);
            assert!(matches!(
                result,
                Err(StrategyError::OperationError(
                    OperationErrorKind::InvalidParameters { .. }
                ))
            ));
        }

        #[test]
        fn test_wrong_option_styles() {
            let options = vec![
                create_sample_position(
                    OptionStyle::Call,
                    Side::Long,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(110.0),
                    pos!(0.2),
                ),
                create_sample_position(
                    OptionStyle::Call,
                    Side::Long,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(90.0),
                    pos!(0.2),
                ),
            ];

            let result = LongStrangle::get_strategy(&options);
            assert!(matches!(
                result,
                Err(StrategyError::OperationError(
                    OperationErrorKind::InvalidParameters { .. }
                ))
            ));
        }

        #[test]
        fn test_wrong_sides() {
            let options = vec![
                create_sample_position(
                    OptionStyle::Call,
                    Side::Short,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(110.0),
                    pos!(0.2),
                ),
                create_sample_position(
                    OptionStyle::Put,
                    Side::Long,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(90.0),
                    pos!(0.2),
                ),
            ];

            let result = LongStrangle::get_strategy(&options);
            assert!(matches!(
                result,
                Err(StrategyError::OperationError(
                    OperationErrorKind::InvalidParameters { .. }
                ))
            ));
        }

        #[test]
        fn test_invalid_strikes() {
            let options = vec![
                create_sample_position(
                    OptionStyle::Call,
                    Side::Long,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(90.0),
                    pos!(0.2),
                ),
                create_sample_position(
                    OptionStyle::Put,
                    Side::Long,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(110.0),
                    pos!(0.2),
                ),
            ];

            let result = LongStrangle::get_strategy(&options);
            assert!(matches!(
                result,
                Err(StrategyError::OperationError(
                    OperationErrorKind::InvalidParameters { .. }
                ))
            ));
        }
    }

    mod short_strangle_tests {
        use super::*;

        #[test]
        fn test_valid_short_strangle() {
            let options = vec![
                create_sample_position(
                    OptionStyle::Call,
                    Side::Short,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(110.0),
                    pos!(0.2),
                ),
                create_sample_position(
                    OptionStyle::Put,
                    Side::Short,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(90.0),
                    pos!(0.2),
                ),
            ];

            let result = ShortStrangle::get_strategy(&options);
            assert!(result.is_ok());

            let strategy = result.unwrap();
            assert_eq!(strategy.short_call.option.strike_price, pos!(110.0));
            assert_eq!(strategy.short_put.option.strike_price, pos!(90.0));
        }

        #[test]
        fn test_wrong_number_of_options() {
            let options = vec![create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(100.0),
                pos!(1.0),
                pos!(110.0),
                pos!(0.2),
            )];

            let result = ShortStrangle::get_strategy(&options);
            assert!(matches!(
                result,
                Err(StrategyError::OperationError(
                    OperationErrorKind::InvalidParameters { .. }
                ))
            ));
        }

        #[test]
        fn test_wrong_option_styles() {
            let options = vec![
                create_sample_position(
                    OptionStyle::Call,
                    Side::Short,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(110.0),
                    pos!(0.2),
                ),
                create_sample_position(
                    OptionStyle::Call,
                    Side::Short,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(90.0),
                    pos!(0.2),
                ),
            ];

            let result = ShortStrangle::get_strategy(&options);
            assert!(matches!(
                result,
                Err(StrategyError::OperationError(
                    OperationErrorKind::InvalidParameters { .. }
                ))
            ));
        }

        #[test]
        fn test_wrong_sides() {
            let options = vec![
                create_sample_position(
                    OptionStyle::Call,
                    Side::Short,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(110.0),
                    pos!(0.2),
                ),
                create_sample_position(
                    OptionStyle::Put,
                    Side::Long,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(90.0),
                    pos!(0.2),
                ),
            ];

            let result = ShortStrangle::get_strategy(&options);
            assert!(matches!(
                result,
                Err(StrategyError::OperationError(
                    OperationErrorKind::InvalidParameters { .. }
                ))
            ));
        }

        #[test]
        fn test_invalid_strikes() {
            let options = vec![
                create_sample_position(
                    OptionStyle::Call,
                    Side::Short,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(90.0),
                    pos!(0.2),
                ),
                create_sample_position(
                    OptionStyle::Put,
                    Side::Short,
                    pos!(100.0),
                    pos!(1.0),
                    pos!(110.0),
                    pos!(0.2),
                ),
            ];

            let result = ShortStrangle::get_strategy(&options);
            assert!(matches!(
                result,
                Err(StrategyError::OperationError(
                    OperationErrorKind::InvalidParameters { .. }
                ))
            ));
        }
    }
}

#[cfg(test)]
mod tests_long_strangle_pnl {
    use super::*;
    use crate::model::utils::create_sample_position;
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn create_test_strangle() -> Result<LongStrangle, StrategyError> {
        // Create long call position
        let long_call = create_sample_position(
            OptionStyle::Call,
            Side::Long,
            pos!(100.0), // Underlying price
            pos!(1.0),   // Quantity
            pos!(105.0), // Strike price
            pos!(0.2),   // Implied volatility
        );

        // Create long put position
        let long_put = create_sample_position(
            OptionStyle::Put,
            Side::Long,
            pos!(100.0), // Same underlying price
            pos!(1.0),   // Quantity
            pos!(95.0),  // Strike price
            pos!(0.2),   // Implied volatility
        );

        LongStrangle::get_strategy(&vec![long_call, long_put])
    }

    #[test]
    fn test_calculate_pnl_at_money() {
        let strangle = create_test_strangle().unwrap();
        let market_price = pos!(100.0);
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.3);

        let result = strangle.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // At the money, both options should have time value but no intrinsic value
        // Initial cost is 2 * (premium + fees) = 2 * (5.0 + 1.0) = 12.0
        assert_pos_relative_eq!(pnl.initial_costs, pos!(12.0), pos!(1e-6));
        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(0.746072), dec!(1e-6));
        assert_eq!(pnl.initial_income, pos!(0.0));
        // Unrealized loss should be less than full premium paid (time value remains)
        assert!(pnl.unrealized.unwrap() > dec!(-12.0));
    }

    #[test]
    fn test_calculate_pnl_above_call_strike() {
        let strangle = create_test_strangle().unwrap();
        let market_price = pos!(110.0); // Above call strike
        let expiration_date = ExpirationDate::Days(pos!(30.0));
        let implied_volatility = pos!(0.2);

        let result = strangle.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Call should be in the money by 5.0 (110 - 105)
        // Put should still have some time value
        assert!(pnl.unrealized.unwrap() > dec!(-7.0)); // Better than max loss
    }

    #[test]
    fn test_calculate_pnl_below_put_strike() {
        let strangle = create_test_strangle().unwrap();
        let market_price = pos!(90.0); // Below put strike
        let expiration_date = ExpirationDate::Days(pos!(30.0));
        let implied_volatility = pos!(0.2);

        let result = strangle.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Put should be in the money by 5.0 (95 - 90)
        // Call should still have some time value
        assert!(pnl.unrealized.unwrap() > dec!(-7.0)); // Better than max loss
    }

    #[test]
    fn test_calculate_pnl_at_expiration_max_loss() {
        let strangle = create_test_strangle().unwrap();
        let underlying_price = pos!(100.0); // At the money

        let result = strangle.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // At expiration, both options expire worthless
        // Max loss is the total premium paid plus fees
        assert_eq!(pnl.realized.unwrap(), dec!(-12.0));
        assert_eq!(pnl.initial_costs, pos!(12.0));
        assert_eq!(pnl.initial_income, pos!(0.0));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_call_profit() {
        let strangle = create_test_strangle().unwrap();
        let underlying_price = pos!(115.0); // Well above call strike

        let result = strangle.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // Call profit: 115 - 105 = 10
        // Put expires worthless
        // Total: 10 - initial costs (12)
        assert_eq!(pnl.realized.unwrap(), dec!(-2.0));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_put_profit() {
        let strangle = create_test_strangle().unwrap();
        let underlying_price = pos!(85.0); // Well below put strike

        let result = strangle.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // Put profit: 95 - 85 = 10
        // Call expires worthless
        // Total: 10 - initial costs (12)
        assert_eq!(pnl.realized.unwrap(), dec!(-2.0));
    }

    #[test]
    fn test_calculate_pnl_with_higher_volatility() {
        let strangle = create_test_strangle().unwrap();
        let market_price = pos!(100.0);
        let expiration_date = ExpirationDate::Days(pos!(30.0));
        let implied_volatility = pos!(0.4); // Higher volatility

        let result = strangle.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // With higher volatility, options should be worth more
        // Loss should be less than with lower volatility
        assert!(pnl.unrealized.unwrap() > dec!(-12.0));
    }
}

#[cfg(test)]
mod tests_short_strangle_pnl {
    use super::*;
    use crate::model::utils::create_sample_position;
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn create_test_strangle() -> Result<ShortStrangle, StrategyError> {
        // Create short call position
        let short_call = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(100.0), // Underlying price
            pos!(1.0),   // Quantity
            pos!(105.0), // Strike price
            pos!(0.2),   // Implied volatility
        );

        // Create short put position
        let short_put = create_sample_position(
            OptionStyle::Put,
            Side::Short,
            pos!(100.0), // Same underlying price
            pos!(1.0),   // Quantity
            pos!(95.0),  // Strike price
            pos!(0.2),   // Implied volatility
        );

        ShortStrangle::get_strategy(&vec![short_call, short_put])
    }

    #[test]
    fn test_calculate_pnl_at_money() {
        let strangle = create_test_strangle().unwrap();
        let market_price = pos!(100.0);
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.3);

        let result = strangle.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // At the money, both options should have time value but no intrinsic value
        // Initial cost is 2 * fees = 2 * (1.0) = 2.0
        // Initial income is 2 * premium = 2 * 5.0 = 10.0
        assert_pos_relative_eq!(pnl.initial_costs, pos!(2.0), pos!(1e-6));
        assert_pos_relative_eq!(pnl.initial_income, pos!(10.0), pos!(1e-6));
        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(-0.746072), dec!(1e-6));
        // Unrealized loss should be less than max potential loss
        assert!(pnl.unrealized.unwrap() > dec!(-100.0)); // Using a large number as max theoretical loss is unlimited
    }

    #[test]
    fn test_calculate_pnl_above_call_strike() {
        let strangle = create_test_strangle().unwrap();
        let market_price = pos!(110.0); // Above call strike
        let expiration_date = ExpirationDate::Days(pos!(30.0));
        let implied_volatility = pos!(0.2);

        let result = strangle.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Call is ITM against us by 5.0 (110 - 105)
        // Put still has some time value
        assert!(pnl.unrealized.unwrap() < dec!(0.0)); // Should be losing money
        assert!(pnl.unrealized.unwrap() > dec!(-10.0)); // But not maximum theoretical loss
    }

    #[test]
    fn test_calculate_pnl_below_put_strike() {
        let strangle = create_test_strangle().unwrap();
        let market_price = pos!(90.0); // Below put strike
        let expiration_date = ExpirationDate::Days(pos!(30.0));
        let implied_volatility = pos!(0.2);

        let result = strangle.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Put is ITM against us by 5.0 (95 - 90)
        // Call still has some time value
        assert!(pnl.unrealized.unwrap() < dec!(0.0)); // Should be losing money
        assert!(pnl.unrealized.unwrap() > dec!(-10.0)); // But not maximum theoretical loss
    }

    #[test]
    fn test_calculate_pnl_at_expiration_max_profit() {
        let strangle = create_test_strangle().unwrap();
        let underlying_price = pos!(100.0); // At the money

        let result = strangle.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // At expiration, both options expire worthless
        // Max profit is the total premium received minus fees
        // Premium received = 10.0 (2 * 5.0)
        // Fees = 2.0 (2 * 1.0)
        assert_eq!(pnl.realized.unwrap(), dec!(8.0)); // 10.0 - 2.0
        assert_eq!(pnl.initial_costs, pos!(2.0));
        assert_eq!(pnl.initial_income, pos!(10.0));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_call_loss() {
        let strangle = create_test_strangle().unwrap();
        let underlying_price = pos!(115.0); // Well above call strike

        let result = strangle.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // Call loss: -(115 - 105) = -10
        // Put expires worthless
        // Plus initial income (10.0) minus costs (2.0)
        assert_eq!(pnl.realized.unwrap(), dec!(-2.0));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_put_loss() {
        let strangle = create_test_strangle().unwrap();
        let underlying_price = pos!(85.0); // Well below put strike

        let result = strangle.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // Put loss: -(95 - 85) = -10
        // Call expires worthless
        // Plus initial income (10.0) minus costs (2.0)
        assert_eq!(pnl.realized.unwrap(), dec!(-2.0));
    }

    #[test]
    fn test_calculate_pnl_with_higher_volatility() {
        let strangle = create_test_strangle().unwrap();
        let market_price = pos!(100.0);
        let expiration_date = ExpirationDate::Days(pos!(30.0));
        let implied_volatility = pos!(0.4); // Higher volatility

        let result = strangle.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // With higher volatility, options should be worth more
        // This is bad for short options - should be larger loss than with lower volatility
        assert!(pnl.unrealized.unwrap() < dec!(0.0));
        // But still not at maximum theoretical loss
        assert!(pnl.unrealized.unwrap() > dec!(-100.0));
    }
}
