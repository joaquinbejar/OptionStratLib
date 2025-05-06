/*
Straddle Strategy

A straddle involves simultaneously buying a call and a put option with the same strike price and expiration date.
This strategy is used when a significant move in the underlying asset's price is expected, but the direction is uncertain.

Key characteristics:
- Unlimited profit potential
- High cost due to purchasing both a call and a put
- Profitable only with a large move in either direction
*/
use super::base::{
    BreakEvenable, Optimizable, Positionable, Strategable, Strategies, StrategyType, Validable,
};
use crate::chains::StrategyLegs;
use crate::chains::chain::OptionChain;
use crate::chains::utils::OptionDataGroup;
use crate::constants::{DARK_BLUE, ZERO};
use crate::error::position::{PositionError, PositionValidationErrorKind};
use crate::error::probability::ProbabilityError;
use crate::error::strategies::StrategyError;
use crate::error::{GreeksError, OperationErrorKind};
use crate::greeks::Greeks;
use crate::model::ProfitLossRange;
use crate::model::position::Position;
use crate::model::types::{OptionStyle, OptionType, Side};
use crate::model::utils::mean_and_std;
use crate::pnl::utils::{PnL, PnLCalculator};
use crate::pricing::payoff::Profit;
use crate::strategies::delta_neutral::DeltaNeutrality;
use crate::strategies::probabilities::core::ProbabilityAnalysis;
use crate::strategies::probabilities::utils::VolatilityAdjustment;
use crate::strategies::utils::{FindOptimalSide, OptimizationCriteria};
use crate::strategies::{StrategyBasics, StrategyConstructor};
use crate::visualization::model::{ChartPoint, ChartVerticalLine, LabelOffsetType};
use crate::visualization::utils::Graph;
use crate::{ExpirationDate, Options, Positive};
use chrono::Utc;
use num_traits::FromPrimitive;
use plotters::prelude::full_palette::ORANGE;
use plotters::prelude::{RED, ShapeStyle};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tracing::info;

/// A Long Straddle is an options trading strategy that involves simultaneously buying
/// a put and a call option with the same strike price and expiration date. This strategy
/// profits from high volatility, as it makes money when the underlying asset moves
/// significantly in either direction. While the maximum loss is limited to the total
/// premium paid, potential profit is unlimited. The strategy is most effective when
/// expecting a large price movement but uncertain about the direction.
///
/// Key characteristics:
/// - Buy 1 ATM Call
/// - Buy 1 ATM Put
/// - Same strike price
/// - Same expiration date
/// - Maximum loss: Total premium paid
/// - Maximum profit: Unlimited
/// - Break-even points: Strike price +/- total premium paid
/// - Ideal market forecast: High volatility, large price movement
///
const LONG_STRADDLE_DESCRIPTION: &str = "Long Straddle strategy involves simultaneously \
buying a put and a call option with identical strike prices and expiration dates. \
Profits from increased volatility and significant price movements in either direction. \
Maximum loss limited to premium paid with unlimited profit potential. Most effective \
when expecting large price movements but uncertain about direction.";

/// # LongStraddle
///
/// Represents a Long Straddle options trading strategy, which involves buying both a call and put option
/// with the same strike price and expiration date.
///
/// A Long Straddle is a volatility strategy that profits when the underlying asset price moves
/// significantly in either direction. It consists of buying a call option and buying a put option
/// with identical strike prices and expiration dates.
///
/// ## Fields
///
/// * `name` - The name of the strategy instance.
/// * `kind` - The type of strategy, which is `StrategyType::LongStraddle`.
/// * `description` - A description of this specific strategy instance.
/// * `break_even_points` - Vector of price points where the strategy breaks even (typically two points for a Long Straddle).
/// * `long_call` - The long call position component of the strategy.
/// * `long_put` - The long put position component of the strategy.
///
/// ## Profit/Loss Characteristics
///
/// * Maximum Loss: Limited to the total premium paid for both options (occurs when price at expiration equals the strike price).
/// * Maximum Profit: Theoretically unlimited to the upside, and limited to the downside by how far the underlying can fall (minus strike price and premiums paid).
/// * Break-even Points: Strike Price + Total Premium Paid (upper) and Strike Price - Total Premium Paid (lower).
///
/// ## Use Cases
///
/// Long Straddles are typically used when:
/// * Expecting significant price movement but uncertain about direction
/// * Before major market events (earnings announcements, FDA approvals, etc.)
/// * During periods of low implied volatility, anticipating increased volatility
///
/// ## Risk Management
///
/// * Time decay (theta) works against this strategy, as both options lose value over time
/// * Most effective when implemented with sufficient time to expiration
/// * Consider closing the position if implied volatility increases significantly without price movement
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LongStraddle {
    /// The name identifier for this strategy instance
    pub name: String,
    /// The strategy type classification, set to StrategyType::LongStraddle
    pub kind: StrategyType,
    /// Detailed description of this specific strategy instance
    pub description: String,
    /// The price points where profit/loss equals zero (typically two points for a straddle)
    pub break_even_points: Vec<Positive>,
    /// The purchased call option position component
    long_call: Position,
    /// The purchased put option position component
    long_put: Position,
}

impl LongStraddle {
    /// Creates a new Long Straddle strategy.
    ///
    /// This constructor builds a Long Straddle by creating and adding both the long call and long put positions
    /// with the same strike price and expiration date.
    ///
    /// # Arguments
    /// * `underlying_symbol` - The ticker symbol of the underlying asset
    /// * `underlying_price` - The current market price of the underlying asset
    /// * `strike` - The strike price for both options (defaults to underlying_price if set to zero)
    /// * `expiration` - The expiration date for both options
    /// * `implied_volatility` - The implied volatility used for option pricing
    /// * `risk_free_rate` - The risk-free interest rate
    /// * `dividend_yield` - The dividend yield of the underlying asset
    /// * `quantity` - The number of contracts for each position
    /// * `premium_long_call` - The premium paid per contract for the call option
    /// * `premium_long_put` - The premium paid per contract for the put option
    /// * `open_fee_long_call` - Transaction fee for opening the call position
    /// * `close_fee_long_call` - Transaction fee for closing the call position
    /// * `open_fee_long_put` - Transaction fee for opening the put position
    /// * `close_fee_long_put` - Transaction fee for closing the put position
    ///
    /// # Returns
    /// A fully initialized Long Straddle strategy with calculated break-even points
    ///
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        mut strike: Positive,
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
        if strike == Positive::ZERO {
            strike = underlying_price;
        }

        let mut strategy = LongStraddle {
            name: "Long Straddle".to_string(),
            kind: StrategyType::LongStraddle,
            description: LONG_STRADDLE_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::default(),
            long_put: Position::default(),
        };

        let long_call_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            strike,
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
            .expect("Invalid long call");

        let long_put_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol,
            strike,
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
            .expect("Invalid long put");

        strategy
            .update_break_even_points()
            .expect("Unable to update break even points");
        strategy
    }
}

impl StrategyConstructor for LongStraddle {
    fn get_strategy(vec_options: &[Position]) -> Result<Self, StrategyError> {
        // Need exactly 2 options for a long straddle
        if vec_options.len() != 2 {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Long Straddle get_strategy".to_string(),
                    reason: "Must have exactly 2 options".to_string(),
                },
            ));
        }

        // Find call and put positions
        let mut call_position = None;
        let mut put_position = None;

        for option in vec_options {
            match option.option.option_style {
                OptionStyle::Call => call_position = Some(option),
                OptionStyle::Put => put_position = Some(option),
            }
        }

        // Validate we have both positions
        let (call_position, put_position) = match (call_position, put_position) {
            (Some(call), Some(put)) => (call, put),
            _ => {
                return Err(StrategyError::OperationError(
                    OperationErrorKind::InvalidParameters {
                        operation: "Long Straddle get_strategy".to_string(),
                        reason: "Must have one call and one put option".to_string(),
                    },
                ));
            }
        };

        // Validate strike prices match
        if call_position.option.strike_price != put_position.option.strike_price {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Long Straddle get_strategy".to_string(),
                    reason: "Options must have the same strike price".to_string(),
                },
            ));
        }

        // Validate both positions are long
        if call_position.option.side != Side::Long || put_position.option.side != Side::Long {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Long Straddle get_strategy".to_string(),
                    reason: "Both options must be long positions".to_string(),
                },
            ));
        }

        // Validate expiration dates match
        if call_position.option.expiration_date != put_position.option.expiration_date {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Long Straddle get_strategy".to_string(),
                    reason: "Options must have the same expiration date".to_string(),
                },
            ));
        }

        // Create positions
        let long_call = Position::new(
            call_position.option.clone(),
            call_position.premium,
            Utc::now(),
            call_position.open_fee,
            call_position.close_fee,
        );

        let long_put = Position::new(
            put_position.option.clone(),
            put_position.premium,
            Utc::now(),
            put_position.open_fee,
            put_position.close_fee,
        );

        // Create strategy
        let mut strategy = LongStraddle {
            name: "Long Straddle".to_string(),
            kind: StrategyType::LongStraddle,
            description: LONG_STRADDLE_DESCRIPTION.to_string(),
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

impl BreakEvenable for LongStraddle {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        let total_cost = self.total_cost()?;

        self.break_even_points.push(
            (self.long_put.option.strike_price - (total_cost / self.long_put.option.quantity))
                .round_to(2),
        );

        self.break_even_points.push(
            (self.long_call.option.strike_price + (total_cost / self.long_call.option.quantity))
                .round_to(2),
        );

        self.break_even_points.sort();
        Ok(())
    }
}

impl Positionable for LongStraddle {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match position.option.option_style {
            OptionStyle::Call => {
                self.long_call = position.clone();
                Ok(())
            }
            OptionStyle::Put => {
                self.long_put = position.clone();
                Ok(())
            }
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
                "Position side is Short, it is not valid for LongStraddle".to_string(),
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
                "Position side is Short, it is not valid for LongStraddle".to_string(),
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

impl Strategable for LongStraddle {
    fn info(&self) -> Result<StrategyBasics, StrategyError> {
        Ok(StrategyBasics {
            name: self.name.clone(),
            kind: self.kind.clone(),
            description: self.description.clone(),
        })
    }
}

impl Strategies for LongStraddle {
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
        let strike_diff = self.break_even_points[1] - self.break_even_points[0];
        let cat = (strike_diff / 2.0_f64.sqrt()).to_f64();
        let loss_area = (cat.powf(2.0)) / (2.0 * 10.0_f64.powf(cat.log10().ceil()));
        let result = (1.0 / loss_area) * 10000.0; // Invert the value to get the profit area: the lower, the better
        Ok(Decimal::from_f64(result).unwrap())
    }

    fn profit_ratio(&self) -> Result<Decimal, StrategyError> {
        let break_even_diff = self.break_even_points[1] - self.break_even_points[0];
        let result = match self.max_loss() {
            Ok(max_loss) => ((break_even_diff / max_loss) * 100.0).to_f64(),
            Err(_) => ZERO,
        };
        Ok(Decimal::from_f64(result).unwrap())
    }
}

impl Validable for LongStraddle {
    fn validate(&self) -> bool {
        self.long_call.validate()
            && self.long_put.validate()
            && self.long_call.option.strike_price == self.long_put.option.strike_price
    }
}

impl Optimizable for LongStraddle {
    type Strategy = LongStraddle;

    fn filter_combinations<'a>(
        &'a self,
        option_chain: &'a OptionChain,
        side: FindOptimalSide,
    ) -> impl Iterator<Item = OptionDataGroup<'a>> {
        let underlying_price = self.get_underlying_price();
        let strategy = self.clone();
        option_chain
            .get_single_iter()
            // Filter out invalid combinations based on FindOptimalSide
            .filter(move |both| {
                if side == FindOptimalSide::Center {
                    let atm_strike = match option_chain.atm_strike() {
                        Ok(atm_strike) => atm_strike,
                        Err(_) => return false,
                    };
                    both.is_valid_optimal_side(
                        underlying_price,
                        &FindOptimalSide::Range(*atm_strike, *atm_strike),
                    )
                } else {
                    both.is_valid_optimal_side(underlying_price, &side)
                }
            })
            .filter(|both| {
                both.call_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && both.call_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
            })
            // Filter out options that don't meet strategy constraints
            .filter(move |both| {
                let legs = StrategyLegs::TwoLegs {
                    first: both,
                    second: both,
                };
                let strategy = strategy.create_strategy(option_chain, &legs);
                strategy.validate() && strategy.max_profit().is_ok() && strategy.max_loss().is_ok()
            })
            // Map to OptionDataGroup
            .map(OptionDataGroup::One)
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
            let both = match option_data_group {
                OptionDataGroup::One(first) => first,
                _ => panic!("Invalid OptionDataGroup"),
            };

            let legs = StrategyLegs::TwoLegs {
                first: both,
                second: both,
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

    fn create_strategy(&self, chain: &OptionChain, legs: &StrategyLegs) -> Self::Strategy {
        let (call, put) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };
        let implied_volatility = call.implied_volatility.unwrap();
        assert!(implied_volatility <= Positive::ONE);
        LongStraddle::new(
            chain.symbol.clone(),
            chain.underlying_price,
            call.strike_price,
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

impl Profit for LongStraddle {
    fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
        let price = Some(&price);
        Ok(self.long_call.pnl_at_expiration(&price)? + self.long_put.pnl_at_expiration(&price)?)
    }
}

impl Graph for LongStraddle {
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

        points.push(ChartPoint {
            coordinates: (self.break_even_points[0].to_f64(), 0.0),
            label: format!("Low Break Even {}", self.break_even_points[0]),
            label_offset: LabelOffsetType::Relative(10.0, -10.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (self.break_even_points[1].to_f64(), 0.0),
            label: format!("High Break Even {}", self.break_even_points[1]),
            label_offset: LabelOffsetType::Relative(-60.0, -10.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (
                self.long_call.option.strike_price.to_f64(),
                -max_loss.to_f64(),
            ),
            label: format!(
                "Max Loss {:.2} at {:.0}",
                max_loss, self.long_call.option.strike_price
            ),
            label_offset: LabelOffsetType::Relative(0.0, -20.0),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        points.push(self.get_point_at_price(self.long_call.option.underlying_price));

        points
    }
}

impl ProbabilityAnalysis for LongStraddle {
    fn get_expiration(&self) -> Result<ExpirationDate, ProbabilityError> {
        let option = &self.long_call.option;
        Ok(option.expiration_date)
    }

    fn get_risk_free_rate(&self) -> Option<Decimal> {
        Some(self.long_call.option.risk_free_rate)
    }

    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let option = &self.long_call.option;
        let break_even_points = self.get_break_even_points()?;

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

impl Greeks for LongStraddle {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.long_call.option, &self.long_put.option])
    }
}

impl DeltaNeutrality for LongStraddle {}

impl PnLCalculator for LongStraddle {
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
mod tests_long_straddle {
    use super::*;
    use crate::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
    use crate::{pos, spos};
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn setup_long_straddle() -> LongStraddle {
        LongStraddle::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(150.0),
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

    fn test_long_straddle_new() {
        let underlying_symbol = "AAPL".to_string();
        let underlying_price = pos!(150.0);
        let call_strike = pos!(160.0);
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

        let strategy = LongStraddle::new(
            underlying_symbol.clone(),
            underlying_price,
            call_strike,
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

        assert_eq!(strategy.name, "Long Straddle");
        assert_eq!(strategy.kind, StrategyType::LongStraddle);
        assert_eq!(strategy.description, LONG_STRADDLE_DESCRIPTION);

        let break_even_points = vec![148.0, 172.0];
        assert_eq!(strategy.break_even_points, break_even_points);
    }

    #[test]

    fn test_get_break_even_points() {
        let long_straddle = setup_long_straddle();
        assert_eq!(long_straddle.get_break_even_points().unwrap()[0], 138.0);
    }

    #[test]

    fn test_total_cost() {
        let long_straddle = setup_long_straddle();
        assert_eq!(
            long_straddle.total_cost().unwrap(),
            long_straddle.long_call.net_cost().unwrap()
                + long_straddle.long_put.net_cost().unwrap()
        );
    }

    #[test]

    fn test_calculate_profit_at() {
        let long_straddle = setup_long_straddle();
        let price = pos!(150.0);
        let expected_profit = long_straddle
            .long_call
            .pnl_at_expiration(&Some(&price))
            .unwrap()
            + long_straddle
                .long_put
                .pnl_at_expiration(&Some(&price))
                .unwrap();
        assert_eq!(
            long_straddle.calculate_profit_at(price).unwrap(),
            expected_profit
        );
    }

    #[test]

    fn test_new() {
        let strategy = setup_long_straddle();
        assert_eq!(strategy.name, "Long Straddle");
        assert_eq!(strategy.kind, StrategyType::LongStraddle);
        assert_eq!(strategy.description, LONG_STRADDLE_DESCRIPTION);
    }

    #[test]

    fn test_validate() {
        let strategy = setup_long_straddle();
        assert!(strategy.validate());
    }

    #[test]

    fn test_max_profit() {
        let strategy = setup_long_straddle();
        assert_eq!(
            strategy.max_profit().unwrap_or(Positive::ZERO),
            Positive::INFINITY
        );
    }

    #[test]

    fn test_max_loss() {
        let strategy = setup_long_straddle();
        assert_eq!(
            strategy.max_loss().unwrap_or(Positive::ZERO),
            strategy.total_cost().unwrap()
        );
    }

    #[test]

    fn test_fees() {
        let strategy = setup_long_straddle();
        let expected_fees = 20.0; // 0.5 * 4 fees * 10 qty
        assert_eq!(strategy.fees().unwrap().to_f64(), expected_fees);
    }

    #[test]

    fn test_net_premium_received() {
        let strategy = setup_long_straddle();
        assert_eq!(strategy.net_premium_received().unwrap().to_f64(), 0.0);
    }

    #[test]

    fn test_profit_area() {
        let strategy = setup_long_straddle();
        let area = strategy.profit_area();
        assert!(area.unwrap().to_f64().unwrap() > 0.0);
    }

    #[test]

    fn test_profit_ratio() {
        let strategy = setup_long_straddle();
        assert_eq!(strategy.profit_ratio().unwrap().to_f64().unwrap(), 20.0);
    }

    #[test]

    fn test_add_leg() {
        let mut strategy = setup_long_straddle();
        let original_call = strategy.long_call.clone();
        let original_put = strategy.long_put.clone();

        strategy
            .add_position(&original_call.clone())
            .expect("Invalid call");
        assert_eq!(strategy.long_call, original_call);

        strategy
            .add_position(&original_put.clone())
            .expect("Invalid put");
        assert_eq!(strategy.long_put, original_put);
    }

    #[test]

    fn test_graph_methods() {
        let strategy = setup_long_straddle();

        // Test vertical lines
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

        // Test title
        let title = strategy.title();
        assert!(title.contains("LongStraddle Strategy"));
        assert!(title.contains("Call"));
        assert!(title.contains("Put"));
    }

    #[test]

    fn test_best_ratio() {
        let mut strategy = setup_long_straddle();
        let option_chain = create_test_option_chain();

        strategy.best_ratio(&option_chain, FindOptimalSide::Upper);
        assert!(strategy.validate());
    }

    #[test]

    fn test_best_area() {
        let mut strategy = setup_long_straddle();
        let option_chain = create_test_option_chain();

        strategy.best_area(&option_chain, FindOptimalSide::Upper);
        assert!(strategy.validate());
    }

    #[test]

    fn test_best_range_to_show() {
        let strategy = setup_long_straddle();
        let step = pos!(1.0);

        let range = strategy.best_range_to_show(step).unwrap();
        assert!(!range.is_empty());
        assert!(range[0] <= strategy.break_even_points[0]);
        assert!(*range.last().unwrap() >= strategy.break_even_points[1]);
    }

    #[test]

    fn test_is_valid_long_option() {
        let strategy = setup_long_straddle();
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
        let strategy = setup_long_straddle();
        let option_chain = create_test_option_chain();
        let call_option = option_chain.atm_option_data().unwrap();
        let put_option = call_option;

        let legs = StrategyLegs::TwoLegs {
            first: call_option,
            second: put_option,
        };
        assert!(strategy.are_valid_prices(&legs));

        let mut invalid_call = call_option.clone();
        invalid_call.call_ask = Some(Positive::ZERO);

        let legs = StrategyLegs::TwoLegs {
            first: &invalid_call,
            second: put_option,
        };
        assert!(!strategy.are_valid_prices(&legs));
    }

    #[test]

    fn test_create_strategy() {
        let strategy = setup_long_straddle();
        let chain = create_test_option_chain();
        let call_option = chain.atm_option_data().unwrap();
        let put_option = call_option;
        let legs = StrategyLegs::TwoLegs {
            first: put_option,
            second: call_option,
        };
        let new_strategy = strategy.create_strategy(&chain, &legs);
        assert!(new_strategy.validate());
    }

    #[test]

    fn test_get_points() {
        let strategy = setup_long_straddle();
        let points = strategy.get_points();

        // Should have 4 points: 2 break-even, 1 max loss, 1 current price
        assert_eq!(points.len(), 4);

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
            spos!(5.0),
            dec!(-0.2),
            dec!(0.1),
            pos!(0.01),
            2,
            option_data_price_params,
        );
        OptionChain::build_chain(&option_chain_build_params)
    }
}

#[cfg(test)]
mod tests_long_straddle_probability {
    use super::*;
    use crate::model::ExpirationDate;
    use crate::pos;
    use crate::strategies::probabilities::utils::PriceTrend;
    use rust_decimal_macros::dec;

    fn create_test_long_straddle() -> LongStraddle {
        LongStraddle::new(
            "TEST".to_string(),
            pos!(100.0),                      // underlying_price
            pos!(110.0),                      // strike
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
        let straddle = create_test_long_straddle();
        let result = straddle.get_expiration();
        assert!(result.is_ok());
        match result.unwrap() {
            ExpirationDate::Days(days) => assert_eq!(days, 30.0),
            _ => panic!("Expected ExpirationDate::Days"),
        }
    }

    #[test]

    fn test_get_risk_free_rate() {
        let straddle = create_test_long_straddle();
        assert_eq!(straddle.get_risk_free_rate(), Some(dec!(0.05)));
    }

    #[test]

    fn test_get_profit_ranges() {
        let straddle = create_test_long_straddle();
        let result = straddle.get_profit_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 2); // Long Straddle has two profit ranges

        // Verify ranges have correct bounds
        assert!(ranges[0].upper_bound.is_some());
        assert!(ranges[1].lower_bound.is_some());
    }

    #[test]

    fn test_get_loss_ranges() {
        let straddle = create_test_long_straddle();
        let result = straddle.get_loss_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1); // Long Straddle has one loss range
        assert!(ranges[0].lower_bound.is_some());
        assert!(ranges[0].upper_bound.is_some());
    }

    #[test]

    fn test_probability_of_profit() {
        let straddle = create_test_long_straddle();
        let result = straddle.probability_of_profit(None, None);

        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]

    fn test_probability_with_volatility_adjustment() {
        let straddle = create_test_long_straddle();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.1),
        });

        let result = straddle.probability_of_profit(vol_adj, None);
        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]

    fn test_probability_with_trend() {
        let straddle = create_test_long_straddle();
        let trend = Some(PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        });

        let result = straddle.probability_of_profit(None, trend);
        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]

    fn test_expected_value_calculation() {
        let straddle = create_test_long_straddle();
        let result = straddle.expected_value(None, None);

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
        let result_with_vol = straddle.expected_value(vol_adj, None);
        assert!(result_with_vol.is_ok());
        assert!(result_with_vol.unwrap() >= Positive::ZERO);
    }

    #[test]

    fn test_calculate_extreme_probabilities() {
        let straddle = create_test_long_straddle();
        let result = straddle.calculate_extreme_probabilities(None, None);

        assert!(result.is_ok());
        let (max_profit_prob, max_loss_prob) = result.unwrap();
        assert!(max_profit_prob >= Positive::ZERO);
        assert!(max_loss_prob >= Positive::ZERO);
        assert!(max_profit_prob + max_loss_prob <= pos!(1.0));
    }
}

#[cfg(test)]
mod tests_long_straddle_delta {
    use super::*;
    use crate::greeks::Greeks;
    use crate::model::types::OptionStyle;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::strategies::long_straddle::{LongStraddle, Positive};
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn get_strategy(strike: Positive) -> LongStraddle {
        let underlying_price = pos!(7138.5);
        LongStraddle::new(
            "CL".to_string(),
            underlying_price, // underlying_price
            strike,           // call_strike 7450
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

    fn create_test_short_straddle_reducing_adjustments() {
        let strike = pos!(7450.0);
        let strategy = get_strategy(strike);
        let size = dec!(-0.168);
        let delta = pos!(0.4039537995372765);
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

    fn create_test_short_straddle_increasing_adjustments() {
        let strategy = get_strategy(pos!(7150.0));
        let size = dec!(0.079961694);
        let delta = pos!(0.17382253382440663);
        let k = pos!(7150.0);
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
                assert_eq!(*side, Side::Long);
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

    fn create_test_short_straddle_no_adjustments() {
        let strategy = get_strategy(pos!(7245.0));

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
mod tests_long_straddle_delta_size {
    use crate::greeks::Greeks;
    use crate::model::types::OptionStyle;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::strategies::long_straddle::{LongStraddle, Positive};
    use crate::{ExpirationDate, Side, assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::str::FromStr;

    fn get_strategy(strike: Positive) -> LongStraddle {
        let underlying_price = pos!(7138.5);
        LongStraddle::new(
            "CL".to_string(),
            underlying_price, // underlying_price
            strike,           // call_strike 7450
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

    fn create_test_short_straddle_reducing_adjustments() {
        let strike = pos!(7450.0);
        let strategy = get_strategy(strike);
        let size = dec!(-0.3360);
        let delta = pos!(0.807_907_599_074_553);
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

    fn create_test_short_straddle_increasing_adjustments() {
        let strategy = get_strategy(pos!(7150.0));
        let size = dec!(0.1599);
        let delta =
            Positive::new_decimal(Decimal::from_str("0.3476450676488132").unwrap()).unwrap();
        let k = pos!(7150.0);
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
                assert_eq!(*side, Side::Long);
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

    fn create_test_short_straddle_no_adjustments() {
        let strategy = get_strategy(pos!(7245.0));

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
mod tests_straddle_position_management {
    use super::*;
    use crate::error::position::PositionValidationErrorKind;
    use crate::model::types::{OptionStyle, Side};
    use crate::pos;
    use rust_decimal_macros::dec;
    use tracing::error;

    fn create_test_long_straddle() -> LongStraddle {
        LongStraddle::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(110.0), // strike
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
    fn test_long_straddle_get_position() {
        let mut straddle = create_test_long_straddle();

        // Test getting long call position
        let call_position = straddle.get_position(&OptionStyle::Call, &Side::Long, &pos!(110.0));
        assert!(call_position.is_ok());
        let positions = call_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(110.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Long);

        // Test getting long put position
        let put_position = straddle.get_position(&OptionStyle::Put, &Side::Long, &pos!(110.0));
        assert!(put_position.is_ok());
        let positions = put_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(110.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Put);
        assert_eq!(positions[0].option.side, Side::Long);

        // Test getting non-existent position
        let invalid_position = straddle.get_position(&OptionStyle::Call, &Side::Long, &pos!(100.0));
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
                error!("Unexpected error: {:?}", invalid_position);
                panic!()
            }
        }
    }

    #[test]
    fn test_long_straddle_modify_position() {
        let mut straddle = create_test_long_straddle();

        // Modify long call position
        let mut modified_call = straddle.long_call.clone();
        modified_call.option.quantity = pos!(2.0);
        let result = straddle.modify_position(&modified_call);
        assert!(result.is_ok());
        assert_eq!(straddle.long_call.option.quantity, pos!(2.0));

        // Modify long put position
        let mut modified_put = straddle.long_put.clone();
        modified_put.option.quantity = pos!(2.0);
        let result = straddle.modify_position(&modified_put);
        assert!(result.is_ok());
        assert_eq!(straddle.long_put.option.quantity, pos!(2.0));

        // Test modifying with invalid position
        let mut invalid_position = straddle.long_call.clone();
        invalid_position.option.strike_price = pos!(95.0);
        let result = straddle.modify_position(&invalid_position);
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
mod tests_adjust_option_position {
    use super::*;
    use crate::model::types::{OptionStyle, Side};
    use crate::pos;
    use rust_decimal_macros::dec;

    fn create_test_long_straddle() -> LongStraddle {
        LongStraddle::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(110.0), // strike
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
    fn test_adjust_existing_call_position_long() {
        let mut strategy = create_test_long_straddle();
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
    fn test_adjust_existing_put_position_long() {
        let mut strategy = create_test_long_straddle();
        let initial_quantity = strategy.long_put.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos!(110.0),
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
    fn test_adjust_nonexistent_position_long() {
        let mut strategy = create_test_long_straddle();

        // Try to adjust a non-existent long call position
        let result = strategy.adjust_option_position(
            Decimal::ONE,
            &pos!(100.0),
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
                    "Position side is Short, it is not valid for LongStraddle"
                );
            }
            _ => panic!("Expected PositionError::ValidationError"),
        }
    }

    #[test]
    fn test_adjust_with_invalid_strike_long() {
        let mut strategy = create_test_long_straddle();

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
    fn test_zero_quantity_adjustment_long() {
        let mut strategy = create_test_long_straddle();
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
mod tests_long_strategy_constructor {
    use super::*;
    use crate::model::utils::create_sample_position;
    use crate::pos;

    #[test]
    fn test_get_strategy_valid() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(100.0),
                pos!(1.0),
                pos!(100.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Put,
                Side::Long,
                pos!(100.0),
                pos!(1.0),
                pos!(100.0),
                pos!(0.2),
            ),
        ];

        let result = LongStraddle::get_strategy(&options);
        assert!(result.is_ok());

        let strategy = result.unwrap();
        assert_eq!(strategy.long_call.option.strike_price, pos!(100.0));
        assert_eq!(strategy.long_put.option.strike_price, pos!(100.0));
    }

    #[test]
    fn test_get_strategy_wrong_number_of_options() {
        let options = vec![create_sample_position(
            OptionStyle::Call,
            Side::Long,
            pos!(100.0),
            pos!(1.0),
            pos!(100.0),
            pos!(0.2),
        )];

        let result = LongStraddle::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Long Straddle get_strategy" && reason == "Must have exactly 2 options"
        ));
    }

    #[test]
    fn test_get_strategy_missing_put_option() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(100.0),
                pos!(1.0),
                pos!(100.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(100.0),
                pos!(1.0),
                pos!(100.0),
                pos!(0.2),
            ),
        ];

        let result = LongStraddle::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Long Straddle get_strategy" && reason == "Must have one call and one put option"
        ));
    }

    #[test]
    fn test_get_strategy_different_strikes() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(100.0),
                pos!(1.0),
                pos!(100.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Put,
                Side::Long,
                pos!(100.0),
                pos!(1.0),
                pos!(95.0),
                pos!(0.2),
            ),
        ];

        let result = LongStraddle::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Long Straddle get_strategy" && reason == "Options must have the same strike price"
        ));
    }

    #[test]
    fn test_get_strategy_wrong_sides() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(100.0),
                pos!(1.0),
                pos!(100.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Put,
                Side::Short,
                pos!(100.0),
                pos!(1.0),
                pos!(100.0),
                pos!(0.2),
            ),
        ];

        let result = LongStraddle::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Long Straddle get_strategy" && reason == "Both options must be long positions"
        ));
    }

    #[test]
    fn test_get_strategy_different_expiration_dates() {
        let mut option1 = create_sample_position(
            OptionStyle::Call,
            Side::Long,
            pos!(100.0),
            pos!(1.0),
            pos!(100.0),
            pos!(0.2),
        );
        let mut option2 = create_sample_position(
            OptionStyle::Put,
            Side::Long,
            pos!(100.0),
            pos!(1.0),
            pos!(100.0),
            pos!(0.2),
        );

        option1.option.expiration_date = ExpirationDate::Days(pos!(30.0));
        option2.option.expiration_date = ExpirationDate::Days(pos!(60.0));

        let options = vec![option1, option2];
        let result = LongStraddle::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Long Straddle get_strategy" && reason == "Options must have the same expiration date"
        ));
    }
}

#[cfg(test)]
mod tests_long_straddle_pnl {
    use super::*;
    use crate::model::utils::create_sample_position;
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn create_test_long_straddle() -> Result<LongStraddle, StrategyError> {
        let long_call = create_sample_position(
            OptionStyle::Call,
            Side::Long,
            pos!(100.0), // Underlying price
            pos!(1.0),   // Quantity
            pos!(100.0), // Strike price (ATM)
            pos!(0.2),   // Implied volatility
        );

        let long_put = create_sample_position(
            OptionStyle::Put,
            Side::Long,
            pos!(100.0), // Same underlying price
            pos!(1.0),   // Quantity
            pos!(100.0), // Same strike price
            pos!(0.2),   // Implied volatility
        );

        LongStraddle::get_strategy(&vec![long_call, long_put])
    }

    #[test]
    fn test_calculate_pnl_at_strike() {
        let straddle = create_test_long_straddle().unwrap();
        let market_price = pos!(100.0); // At strike price
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.2);

        let result = straddle.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Both options ATM, should be near max profit
        assert_pos_relative_eq!(pnl.initial_income, pos!(0.0), pos!(1e-6)); // Premium from both options
        assert_pos_relative_eq!(pnl.initial_costs, pos!(12.0), pos!(1e-6)); // Total fees
    }

    #[test]
    fn test_calculate_pnl_below_strike() {
        let straddle = create_test_long_straddle().unwrap();
        let market_price = pos!(100.0); // Below strike
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.2);

        let result = straddle.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Put ITM, call OTM
        assert!(pnl.unrealized.unwrap() < dec!(0.0)); // Should be a loss
    }

    #[test]
    fn test_calculate_pnl_above_strike() {
        let straddle = create_test_long_straddle().unwrap();
        let market_price = pos!(110.0); // Above strike
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.2);

        let result = straddle.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Call ITM, put OTM
        assert!(pnl.unrealized.unwrap() > dec!(0.0)); // Should be a loss
    }

    #[test]
    fn test_calculate_pnl_with_higher_volatility() {
        let straddle = create_test_long_straddle().unwrap();
        let market_price = pos!(100.0);
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.4); // Higher volatility

        let result = straddle.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Higher volatility should result in larger losses
        assert!(pnl.unrealized.unwrap() < dec!(3.0));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_max_profit() {
        let straddle = create_test_long_straddle().unwrap();
        let underlying_price = pos!(100.0); // At strike price

        let result = straddle.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // At strike price at expiration, both options expire worthless
        // Max profit is the net premium received minus fees
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(-12.0), dec!(1e-6)); // Premium received - costs
        assert_eq!(pnl.initial_income, pos!(0.0));
        assert_eq!(pnl.initial_costs, pos!(12.0));
    }
}
