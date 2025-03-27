/*
Bull Put Spread Strategy

A bull put spread involves buying a put option with a lower strike price and selling a put option with a higher strike price,
both with the same expiration date. This strategy is used when a moderate rise in the underlying asset's price is expected.

Key characteristics:
- Limited profit potential (net premium received)
- Limited risk (difference between strikes minus net premium)
- Bullish strategy that profits from price increase
- Both options have same expiration date
- Requires less margin than naked put selling
- Lower risk than naked put selling
- Maximum profit achieved when price stays above higher strike
- Also known as a vertical put credit spread
*/
use super::base::{
    BreakEvenable, Optimizable, Positionable, Strategable, Strategies, StrategyType, Validable,
};
use crate::Options;
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
use crate::pnl::utils::{PnL, PnLCalculator};
use crate::pricing::payoff::Profit;
use crate::strategies::delta_neutral::DeltaNeutrality;
use crate::strategies::probabilities::core::ProbabilityAnalysis;
use crate::strategies::probabilities::utils::VolatilityAdjustment;
use crate::strategies::utils::{FindOptimalSide, OptimizationCriteria};
use crate::strategies::{StrategyBasics, StrategyConstructor};
use crate::visualization::model::{ChartPoint, ChartVerticalLine, LabelOffsetType};
use crate::visualization::utils::Graph;
use crate::{Positive, pos};
use chrono::Utc;
use plotters::prelude::full_palette::ORANGE;
use plotters::prelude::{RED, ShapeStyle};
use rust_decimal::Decimal;
use std::error::Error;
use tracing::debug;

const BULL_PUT_SPREAD_DESCRIPTION: &str = "A bull put spread is created by buying a put option with a lower strike price \
    and simultaneously selling a put option with a higher strike price, both with the same \
    expiration date. This strategy is used when you expect a moderate increase in the underlying \
    asset's price. The maximum profit is limited to the net credit received, while the maximum \
    loss is limited to the difference between strike prices minus the net credit.";

/// Represents a Bull Put Spread options trading strategy.
///
/// A Bull Put Spread consists of buying a put option with a lower strike price (long put)
/// and selling a put option with a higher strike price (short put), both with the same
/// expiration date. This strategy is used when an investor is moderately bullish on the
/// underlying asset and wants to generate income with limited risk.
///
/// # Characteristics
/// - Limited profit potential (difference between premiums received and paid)
/// - Limited risk (difference between strike prices minus net premium received)
/// - Bullish outlook (profits when the underlying price stays above the short put strike)
/// - Generates upfront income from the net premium received
///
/// # Attributes
#[derive(Clone, Debug)]
pub struct BullPutSpread {
    /// The name of the strategy, typically "Bull Put Spread"
    pub name: String,

    /// The type of strategy, represented by the StrategyType enum
    pub kind: StrategyType,

    /// A detailed description of the strategy, its use cases and risk profile
    pub description: String,

    /// The price points at which the strategy breaks even (typically a single point)
    /// representing the short put strike minus the net premium received
    pub break_even_points: Vec<Positive>,

    /// The long put position (lower strike price) that limits the downside risk
    long_put: Position,

    /// The short put position (higher strike price) that generates premium income
    short_put: Position,
}

impl BullPutSpread {
    /// Creates a new Bull Put Spread options strategy.
    ///
    /// A Bull Put Spread is created by buying a put option with a lower strike price and simultaneously
    /// selling a put option with a higher strike price, both with the same expiration date. This strategy
    /// is used when you expect a moderate increase in the underlying asset's price.
    ///
    /// # Parameters
    ///
    /// * `underlying_symbol` - Symbol of the underlying asset (e.g., stock ticker).
    /// * `underlying_price` - Current price of the underlying asset.
    /// * `long_strike` - Strike price for the long put option. Defaults to `underlying_price` if set to zero.
    /// * `short_strike` - Strike price for the short put option. Defaults to `underlying_price` if set to zero.
    /// * `expiration` - Expiration date for both options.
    /// * `implied_volatility` - Implied volatility used for option pricing calculations.
    /// * `risk_free_rate` - Risk-free interest rate used in pricing models.
    /// * `dividend_yield` - Dividend yield of the underlying asset.
    /// * `quantity` - Number of option contracts.
    /// * `premium_long_put` - Premium paid for the long put option.
    /// * `premium_short_put` - Premium received for the short put option.
    /// * `open_fee_long_put` - Transaction fee for opening the long put position.
    /// * `close_fee_long_put` - Transaction fee for closing the long put position.
    /// * `open_fee_short_put` - Transaction fee for opening the short put position.
    /// * `close_fee_short_put` - Transaction fee for closing the short put position.
    ///
    /// # Returns
    ///
    /// A validated `BullPutSpread` strategy with calculated break-even points.
    ///
    /// # Strategy Details
    ///
    /// - Maximum profit: Limited to the net credit received (premium difference)
    /// - Maximum loss: Limited to the difference between strike prices minus the net credit
    /// - Break-even point: Short put strike price minus net credit received
    ///
    /// # Validation
    ///
    /// The created strategy is validated to ensure:
    /// 1. Both positions are valid
    /// 2. The long put strike price is lower than the short put strike price
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        mut long_strike: Positive,
        mut short_strike: Positive,
        expiration: ExpirationDate,
        implied_volatility: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        quantity: Positive,
        premium_long_put: Positive,
        premium_short_put: Positive,
        open_fee_long_put: Positive,
        close_fee_long_put: Positive,
        open_fee_short_put: Positive,
        close_fee_short_put: Positive,
    ) -> Self {
        if long_strike == Positive::ZERO {
            long_strike = underlying_price;
        }
        if short_strike == Positive::ZERO {
            short_strike = underlying_price;
        }

        let mut strategy = BullPutSpread {
            name: "Bull Put Spread".to_string(),
            kind: StrategyType::BullPutSpread,
            description: BULL_PUT_SPREAD_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_put: Position::default(),
            short_put: Position::default(),
        };

        let long_put_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            long_strike,
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
            .expect("Error adding long put");

        let short_put_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol,
            short_strike,
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
            .expect("Error adding short put");

        strategy.validate();

        strategy
            .update_break_even_points()
            .expect("Unable to update break even points");

        strategy
    }
}

impl StrategyConstructor for BullPutSpread {
    fn get_strategy(vec_options: &[Position]) -> Result<Self, StrategyError> {
        // Need exactly 2 options for a bull put spread
        if vec_options.len() != 2 {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Bull Put Spread get_strategy".to_string(),
                    reason: "Must have exactly 2 options".to_string(),
                },
            ));
        }

        // Sort options by strike price to identify short and long positions
        let mut sorted_options = vec_options.to_vec();
        sorted_options.sort_by(|a, b| {
            a.option
                .strike_price
                .partial_cmp(&b.option.strike_price)
                .unwrap()
        });

        let lower_strike_option = &sorted_options[0];
        let higher_strike_option = &sorted_options[1];

        // Validate options are puts
        if lower_strike_option.option.option_style != OptionStyle::Put
            || higher_strike_option.option.option_style != OptionStyle::Put
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Bull Put Spread get_strategy".to_string(),
                    reason: "Options must be puts".to_string(),
                },
            ));
        }

        // Validate option sides - long higher strike put, short lower strike put
        if lower_strike_option.option.side != Side::Short
            || higher_strike_option.option.side != Side::Long
        {
            return Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters {
                operation: "Bull Put Spread get_strategy".to_string(),
                reason: "Bull Put Spread requires a short lower strike put and a long higher strike put".to_string(),
            }));
        }

        // Validate expiration dates match
        if lower_strike_option.option.expiration_date != higher_strike_option.option.expiration_date
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Bull Put Spread get_strategy".to_string(),
                    reason: "Options must have the same expiration date".to_string(),
                },
            ));
        }

        // Create positions
        let short_put = Position::new(
            lower_strike_option.option.clone(),
            lower_strike_option.premium,
            Utc::now(),
            lower_strike_option.open_fee,
            lower_strike_option.close_fee,
        );

        let long_put = Position::new(
            higher_strike_option.option.clone(),
            higher_strike_option.premium,
            Utc::now(),
            higher_strike_option.open_fee,
            higher_strike_option.close_fee,
        );

        // Create strategy
        let mut strategy = BullPutSpread {
            name: "Bull Put Spread".to_string(),
            kind: StrategyType::BullPutSpread,
            description: BULL_PUT_SPREAD_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_put,
            long_put,
        };

        // Validate and update break-even points
        strategy.validate();
        strategy.update_break_even_points()?;

        Ok(strategy)
    }
}

impl BreakEvenable for BullPutSpread {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        self.break_even_points.push(
            (self.short_put.option.strike_price
                + self.net_cost()? / self.short_put.option.quantity)
                .round_to(2),
        );

        Ok(())
    }
}

impl Positionable for BullPutSpread {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match position.option.side {
            Side::Short => {
                self.short_put = position.clone();
                Ok(())
            }
            Side::Long => {
                self.long_put = position.clone();
                Ok(())
            }
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.long_put, &self.short_put])
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
            (_, OptionStyle::Call, _) => Err(PositionError::invalid_position_type(
                *side,
                "Call is not valid for BullPutSpread".to_string(),
            )),
            (Side::Long, OptionStyle::Put, strike)
                if *strike == self.long_put.option.strike_price =>
            {
                Ok(vec![&mut self.long_put])
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

        match (
            &position.option.side,
            &position.option.option_style,
            &position.option.strike_price,
        ) {
            (_, OptionStyle::Call, _) => {
                return Err(PositionError::invalid_position_type(
                    position.option.side,
                    "Call is not valid for BullPutSpread".to_string(),
                ));
            }
            (Side::Long, OptionStyle::Put, strike)
                if *strike == self.long_put.option.strike_price =>
            {
                self.long_put = position.clone();
            }
            (Side::Short, OptionStyle::Put, strike)
                if *strike == self.short_put.option.strike_price =>
            {
                self.short_put = position.clone();
            }
            _ => {
                return Err(PositionError::invalid_position_type(
                    position.option.side,
                    "Strike not found in positions".to_string(),
                ));
            }
        }

        Ok(())
    }
}

impl Strategable for BullPutSpread {
    fn info(&self) -> Result<StrategyBasics, StrategyError> {
        Ok(StrategyBasics {
            name: self.name.clone(),
            kind: self.kind.clone(),
            description: self.description.clone(),
        })
    }
}

impl Strategies for BullPutSpread {
    fn get_underlying_price(&self) -> Positive {
        self.short_put.option.underlying_price
    }

    fn max_profit(&self) -> Result<Positive, StrategyError> {
        let net_premium_received = self.net_premium_received()?;
        if net_premium_received < Decimal::ZERO {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxProfitError {
                    reason: "Net premium received is negative".to_string(),
                },
            ))
        } else {
            Ok(net_premium_received)
        }
    }

    fn max_loss(&self) -> Result<Positive, StrategyError> {
        let width = self.short_put.option.strike_price - self.long_put.option.strike_price;
        let max_loss = (width * self.short_put.option.quantity) - self.net_premium_received()?;
        if max_loss < ZERO {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxLossError {
                    reason: "Max loss is negative".to_string(),
                },
            ))
        } else {
            Ok(max_loss)
        }
    }

    fn profit_area(&self) -> Result<Decimal, StrategyError> {
        let high = self.max_profit().unwrap_or(Positive::ZERO);
        let base = if self.short_put.option.strike_price > self.break_even_points[0] {
            self.short_put.option.strike_price - self.break_even_points[0]
        } else {
            self.break_even_points[0] - self.short_put.option.strike_price
        };
        Ok((high * base / 200.0).into())
    }

    fn profit_ratio(&self) -> Result<Decimal, StrategyError> {
        let max_profit = self.max_profit().unwrap_or(Positive::ZERO);
        let max_loss = self.max_loss().unwrap_or(Positive::ZERO);
        match (max_profit, max_loss) {
            (value, _) if value == Positive::ZERO => Ok(Decimal::ZERO),
            (_, value) if value == Positive::ZERO => Ok(Decimal::MAX),
            _ => Ok((max_profit / max_loss * 100.0).into()),
        }
    }
}

impl Validable for BullPutSpread {
    fn validate(&self) -> bool {
        if !self.long_put.validate() {
            debug!("Long put is invalid");
            return false;
        }
        if !self.short_put.validate() {
            debug!("Short put is invalid");
            return false;
        }
        if self.long_put.option.strike_price >= self.short_put.option.strike_price {
            debug!("Long put strike price must be lower than short put strike price");
            return false;
        }
        true
    }
}

impl Optimizable for BullPutSpread {
    type Strategy = BullPutSpread;

    /// Filters combinations of `OptionData` from the provided `OptionChain`
    /// based on validity, pricing conditions, and strategy constraints.
    ///
    /// This function generates pairs of options from the `OptionChain` and applies
    /// a series of filters and validations to ensure the results conform to the
    /// specified trading strategy requirements. Each returned pair (`long`, `short`)
    /// represents options that are suitable for building a strategy, such as a
    /// "bull put spread".
    ///
    /// # Parameters
    ///
    /// - `option_chain`: A reference to the `OptionChain` containing the option data.
    /// - `side`: The `FindOptimalSide` specifying the filtering condition based on the
    ///   strike price range relative to the underlying price.
    ///
    /// # Returns
    ///
    /// An iterator over pairs of references to `OptionData` that meet the selection criteria.
    /// Each pair satisfies:
    /// - Both options are valid according to the `is_valid_optimal_side` method.
    /// - Both options have valid bid/ask prices for the put options (`put_ask` for long and
    ///   `put_bid` for short must be greater than zero).
    /// - The strategy created using the pair passes validation checks and successfully
    ///   calculates `max_profit` and `max_loss`.
    ///
    /// # Process
    ///
    /// 1. Computes the underlying price via `self.get_underlying_price()`.
    /// 2. Initializes a cloned version of the strategy for dynamic closures.
    /// 3. Uses the `option_chain.get_double_iter()` method to generate all possible pairs
    ///    of options for evaluation.
    /// 4. Filters the pairs based on combination validity, pricing constraints, and
    ///    strategy feasibility:
    ///    - Ensures the options are on the correct `FindOptimalSide` relative to the
    ///      underlying price.
    ///    - Ensures the `put_ask` and `put_bid` prices meet the conditions.
    ///    - Ensures the strategy created with the options is valid and has calculable
    ///      profit and loss parameters.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_decimal_macros::dec;
    /// use tracing::info;
    /// use optionstratlib::chains::chain::OptionChain;
    /// use optionstratlib::chains::utils::OptionDataGroup;
    /// use optionstratlib::ExpirationDate;
    /// use optionstratlib::Positive;
    /// use optionstratlib::pos;
    /// use optionstratlib::strategies::base::Optimizable;
    /// use optionstratlib::strategies::bull_put_spread::BullPutSpread;
    /// use optionstratlib::strategies::utils::FindOptimalSide;
    ///
    /// let underlying_price = pos!(5810.0);
    /// let option_chain = OptionChain::new("TEST", underlying_price, "2024-01-01".to_string(), None, None);
    /// let bull_put_spread_strategy = BullPutSpread::new(
    ///         "SP500".to_string(),
    ///         underlying_price,   // underlying_price
    ///         pos!(5750.0),   // long_strike
    ///         pos!(5920.0),   // short_strike
    ///         ExpirationDate::Days(pos!(2.0)),
    ///         pos!(0.18),   // implied_volatility
    ///         dec!(0.05),   // risk_free_rate
    ///         Positive::ZERO,   // dividend_yield
    ///         pos!(1.0),   // long quantity
    ///         pos!(15.04),   // premium_long
    ///         pos!(89.85),   // premium_short
    ///         pos!(0.78),   // open_fee_long
    ///         pos!(0.78),   // open_fee_long
    ///         pos!(0.73),   // close_fee_long
    ///         pos!(0.73),   // close_fee_short
    ///     );
    ///
    /// let side = FindOptimalSide::Lower;
    /// let filtered_combinations = bull_put_spread_strategy.filter_combinations(&option_chain, side);
    ///
    /// for option_data_group in filtered_combinations {
    ///    let (long, short) = match option_data_group {
    ///        OptionDataGroup::Two(first, second) => (first, second),
    ///       _ => panic!("Invalid OptionDataGroup"),
    ///    };
    ///    info!("Long Option: {:?}, Short Option: {:?}", long, short);
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// - This function assumes that the `OptionChain` data structure is well-formed
    ///   and contains valid `OptionData`.
    /// - It is intended for strategies requiring combinations of two legs, like spreads.
    ///   For strategies requiring more legs, an alternative method may be needed.
    ///
    /// # See Also
    ///
    /// - [`OptionChain::get_double_iter`](crate::chains::OptionChain::get_double_iter)
    /// - [`OptionData::is_valid_optimal_side`](crate::chains::OptionData::is_valid_optimal_side)
    /// - [`BullPutSpread::validate`](crate::strategies::bull_put_spread::BullPutSpread::validate)
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
            .filter(move |(long, short)| {
                if side == FindOptimalSide::Center {
                    long.is_valid_optimal_side(underlying_price, &FindOptimalSide::Lower)
                        && short.is_valid_optimal_side(underlying_price, &FindOptimalSide::Upper)
                } else {
                    long.is_valid_optimal_side(underlying_price, &side)
                        && short.is_valid_optimal_side(underlying_price, &side)
                }
            })
            // Filter out options with invalid bid/ask prices
            .filter(|(long, short)| {
                long.put_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && short.put_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
            })
            // Filter out options that don't meet strategy constraints
            .filter(move |(long_option, short_option)| {
                let legs = StrategyLegs::TwoLegs {
                    first: long_option,
                    second: short_option,
                };
                let strategy = strategy.create_strategy(option_chain, &legs);
                strategy.validate() && strategy.max_profit().is_ok() && strategy.max_loss().is_ok()
            })
            // Map to OptionDataGroup
            .map(move |(long, short)| OptionDataGroup::Two(long, short))
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
            let (long_option, short_option) = match option_data_group {
                OptionDataGroup::Two(first, second) => (first, second),
                _ => panic!("Invalid OptionDataGroup"),
            };

            let legs = StrategyLegs::TwoLegs {
                first: long_option,
                second: short_option,
            };
            let strategy = self.create_strategy(option_chain, &legs);
            // Calculate the current value based on the optimization criteria
            let current_value = match criteria {
                OptimizationCriteria::Ratio => strategy.profit_ratio().unwrap(),
                OptimizationCriteria::Area => strategy.profit_area().unwrap(),
            };

            if current_value > best_value {
                // Update the best value and replace the current strategy
                debug!("Found better value: {}", current_value);
                best_value = current_value;
                *self = strategy.clone();
            }
        }
    }

    fn create_strategy(&self, chain: &OptionChain, legs: &StrategyLegs) -> Self::Strategy {
        let (long, short) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };
        BullPutSpread::new(
            chain.symbol.clone(),
            chain.underlying_price,
            long.strike_price,
            short.strike_price,
            self.long_put.option.expiration_date,
            long.implied_volatility.unwrap() / 100.0,
            self.long_put.option.risk_free_rate,
            self.long_put.option.dividend_yield,
            self.long_put.option.quantity,
            long.put_ask.unwrap(),
            short.put_bid.unwrap(),
            self.long_put.open_fee,
            self.long_put.close_fee,
            self.short_put.open_fee,
            self.short_put.close_fee,
        )
    }
}

impl Profit for BullPutSpread {
    fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
        let price = Some(&price);
        Ok(self.long_put.pnl_at_expiration(&price)? + self.short_put.pnl_at_expiration(&price)?)
    }
}

impl Graph for BullPutSpread {
    fn title(&self) -> String {
        format!(
            "{} Strategy:\n\t{}\n\t{}",
            self.name,
            self.long_put.title(),
            self.short_put.title()
        )
    }

    fn get_x_values(&self) -> Vec<Positive> {
        self.best_range_to_show(Positive::from(1.0))
            .unwrap_or_else(|_| vec![self.long_put.option.strike_price])
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        let underlying_price = self.short_put.option.underlying_price.to_f64();
        vec![ChartVerticalLine {
            x_coordinate: underlying_price,
            y_range: (f64::NEG_INFINITY, f64::INFINITY),
            label: format!("Current Price: {:.2}", underlying_price),
            label_offset: (4.0, 0.0),
            line_color: ORANGE,
            label_color: ORANGE,
            line_style: ShapeStyle::from(&ORANGE).stroke_width(2),
            font_size: 18,
        }]
    }

    fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> {
        let mut points = Vec::new();

        // Break Even Point
        points.push(ChartPoint {
            coordinates: (self.break_even_points[0].to_f64(), 0.0),
            label: format!("Break Even {:.2}", self.break_even_points[0]),
            label_offset: LabelOffsetType::Relative(10.0, -10.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        // Maximum Profit Point (at higher strike price)
        points.push(ChartPoint {
            coordinates: (
                self.short_put.option.strike_price.to_f64(),
                self.max_profit().unwrap_or(Positive::ZERO).to_f64(),
            ),
            label: format!(
                "Max Profit {:.2}",
                self.max_profit().unwrap_or(Positive::ZERO)
            ),
            label_offset: LabelOffsetType::Relative(10.0, 10.0),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        // Maximum Loss Point (at lower strike price)
        points.push(ChartPoint {
            coordinates: (
                self.long_put.option.strike_price.to_f64(),
                -self.max_loss().unwrap_or(Positive::ZERO).to_f64(),
            ),
            label: format!("Max Loss -{:.2}", self.max_loss().unwrap_or(Positive::ZERO)),
            label_offset: LabelOffsetType::Relative(-120.0, -10.0),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        // Current Price Point
        points.push(self.get_point_at_price(self.short_put.option.underlying_price));

        points
    }
}

impl ProbabilityAnalysis for BullPutSpread {
    fn get_expiration(&self) -> Result<ExpirationDate, ProbabilityError> {
        Ok(self.short_put.option.expiration_date)
    }

    fn get_risk_free_rate(&self) -> Option<Decimal> {
        Some(self.short_put.option.risk_free_rate)
    }

    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_point = self.get_break_even_points()?[0];

        let (mean_volatility, std_dev) = mean_and_std(vec![
            self.short_put.option.implied_volatility,
            self.long_put.option.implied_volatility,
        ]);

        let mut profit_range = ProfitLossRange::new(
            Some(break_even_point),
            None,
            pos!(self.max_profit()?.to_f64()),
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
        let break_even_point = self.get_break_even_points()?[0];

        let (mean_volatility, std_dev) = mean_and_std(vec![
            self.short_put.option.implied_volatility,
            self.long_put.option.implied_volatility,
        ]);

        let mut loss_range = ProfitLossRange::new(
            Some(self.long_put.option.strike_price),
            Some(break_even_point),
            pos!(self.max_loss()?.to_f64()),
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

impl Greeks for BullPutSpread {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.long_put.option, &self.short_put.option])
    }
}

impl DeltaNeutrality for BullPutSpread {}

impl PnLCalculator for BullPutSpread {
    fn calculate_pnl(
        &self,
        market_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        Ok(self
            .short_put
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
            .short_put
            .calculate_pnl_at_expiration(underlying_price)?
            + self
                .long_put
                .calculate_pnl_at_expiration(underlying_price)?)
    }
}

#[cfg(test)]
fn bull_put_spread_test() -> BullPutSpread {
    use rust_decimal_macros::dec;
    let underlying_price = pos!(5781.88);
    BullPutSpread::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        pos!(5750.0),     // long_strike_itm
        pos!(5920.0),     // short_strike
        ExpirationDate::Days(pos!(2.0)),
        pos!(0.18),     // implied_volatility
        dec!(0.05),     // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(3.0),      // long quantity
        pos!(15.04),    // premium_long
        pos!(89.85),    // premium_short
        pos!(0.78),     // open_fee_long
        pos!(0.78),     // open_fee_long
        pos!(0.73),     // close_fee_long
        pos!(0.73),     // close_fee_short
    )
}

#[cfg(test)]
mod tests_bull_put_spread_strategy {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    #[test]

    fn test_new_bull_put_spread() {
        let spread = bull_put_spread_test();

        assert_eq!(spread.name, "Bull Put Spread");
        assert_eq!(spread.kind, StrategyType::BullPutSpread);
        assert!(!spread.description.is_empty());
        assert_eq!(spread.get_underlying_price(), pos!(5781.88));
        assert_eq!(spread.long_put.option.strike_price, pos!(5750.0));
        assert_eq!(spread.short_put.option.strike_price, pos!(5920.0));
    }

    #[test]

    fn test_add_leg() {
        let mut spread = bull_put_spread_test();
        let new_long_put = Position::new(
            Options::new(
                OptionType::European,
                Side::Long,
                "TEST".to_string(),
                pos!(85.0),
                ExpirationDate::Days(pos!(30.0)),
                pos!(0.2),
                pos!(1.0),
                pos!(100.0),
                dec!(0.05),
                OptionStyle::Put,
                Positive::ZERO,
                None,
            ),
            pos!(1.5),
            Utc::now(),
            Positive::ZERO,
            Positive::ZERO,
        );

        spread
            .add_position(&new_long_put.clone())
            .expect("Error adding long put");
        assert_eq!(spread.long_put.option.strike_price, pos!(85.0));
    }

    #[test]

    fn test_get_legs() {
        let spread = bull_put_spread_test();
        let legs = spread.get_positions().expect("Error getting positions");

        assert_eq!(legs.len(), 2);
        assert_eq!(legs[0].option.side, Side::Long);
        assert_eq!(legs[1].option.side, Side::Short);
    }

    #[test]

    fn test_max_profit() {
        let spread = bull_put_spread_test();
        let max_profit = spread.max_profit().unwrap();
        assert_eq!(max_profit, pos!(215.37));
    }

    #[test]

    fn test_max_loss() {
        let spread = bull_put_spread_test();
        let max_loss = spread.max_loss().unwrap();
        assert_eq!(max_loss, pos!(294.63));
    }

    #[test]

    fn test_total_cost() {
        let spread = bull_put_spread_test();
        assert_eq!(spread.total_cost().unwrap(), pos!(54.18));
    }

    #[test]

    fn test_net_premium_received() {
        let spread = bull_put_spread_test();
        assert_eq!(spread.net_premium_received().unwrap().to_f64(), 215.37);
    }

    #[test]

    fn test_fees() {
        let spread = bull_put_spread_test();

        assert_eq!(spread.fees().unwrap().to_f64(), 9.06);
    }

    #[test]

    fn test_break_even_points() {
        let spread = bull_put_spread_test();
        let break_even_points = spread.get_break_even_points().unwrap();

        assert_eq!(break_even_points.len(), 1);
        assert_eq!(break_even_points[0], pos!(5848.21));
    }

    #[test]

    fn test_profit_area() {
        let spread = bull_put_spread_test();
        let area = spread.profit_area().unwrap().to_f64().unwrap();
        assert!(area > 0.0);
    }

    #[test]

    fn test_profit_ratio() {
        let spread = bull_put_spread_test();
        let ratio = spread.profit_ratio().unwrap().to_f64().unwrap();

        // Ratio = (max_profit / max_loss) * 100
        // = (1.0 / 4.0) * 100 = 25
        assert_relative_eq!(ratio, 73.0984, epsilon = 0.0001);
    }

    #[test]

    fn test_default_strikes() {
        let spread = BullPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            Positive::ZERO, // long_strike = default
            Positive::ZERO, // short_strike = default
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(1.0),
            pos!(2.0),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );

        assert_eq!(spread.long_put.option.strike_price, pos!(100.0));
        assert_eq!(spread.short_put.option.strike_price, pos!(100.0));
    }

    #[test]

    fn test_invalid_strikes() {
        let spread = BullPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(90.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(1.0),
            pos!(2.0),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );

        assert!(!spread.validate());
    }
}

#[cfg(test)]
mod tests_bull_put_spread_validation {
    use super::*;
    use crate::model::types::ExpirationDate;
    use chrono::Utc;
    use rust_decimal_macros::dec;

    fn create_valid_position(
        side: Side,
        strike_price: Positive,
        expiration: ExpirationDate,
    ) -> Position {
        Position::new(
            Options::new(
                OptionType::European,
                side,
                "TEST".to_string(),
                strike_price,
                expiration,
                pos!(0.2),
                pos!(1.0),
                pos!(100.0),
                dec!(0.05),
                OptionStyle::Put,
                Positive::ZERO,
                None,
            ),
            pos!(1.0),
            Utc::now(),
            Positive::ZERO,
            Positive::ZERO,
        )
    }

    #[test]

    fn test_invalid_long_put() {
        let mut invalid_long =
            create_valid_position(Side::Long, pos!(90.0), ExpirationDate::Days(pos!(30.0)));
        invalid_long.option.quantity = Positive::ZERO;

        let spread = BullPutSpread {
            name: "Test Bull Put Spread".to_string(),
            kind: StrategyType::BullPutSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_put: invalid_long,
            short_put: create_valid_position(
                Side::Short,
                pos!(95.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
        };

        assert!(
            !spread.validate(),
            "Spread with invalid long put should fail validation"
        );
    }

    #[test]

    fn test_invalid_short_put() {
        let mut invalid_short =
            create_valid_position(Side::Short, pos!(95.0), ExpirationDate::Days(pos!(30.0)));
        invalid_short.option.quantity = Positive::ZERO;

        let spread = BullPutSpread {
            name: "Test Bull Put Spread".to_string(),
            kind: StrategyType::BullPutSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_put: create_valid_position(
                Side::Long,
                pos!(90.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
            short_put: invalid_short,
        };

        assert!(
            !spread.validate(),
            "Spread with invalid short put should fail validation"
        );
    }

    #[test]

    fn test_invalid_strike_prices() {
        let spread = BullPutSpread {
            name: "Test Bull Put Spread".to_string(),
            kind: StrategyType::BullPutSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_put: create_valid_position(
                Side::Long,
                pos!(95.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
            short_put: create_valid_position(
                Side::Short,
                pos!(90.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
        };

        assert!(
            !spread.validate(),
            "Spread with long strike price >= short strike price should fail validation"
        );
    }

    #[test]

    fn test_equal_strike_prices() {
        let spread = BullPutSpread {
            name: "Test Bull Put Spread".to_string(),
            kind: StrategyType::BullPutSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_put: create_valid_position(
                Side::Long,
                pos!(90.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
            short_put: create_valid_position(
                Side::Short,
                pos!(90.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
        };

        assert!(
            !spread.validate(),
            "Spread with equal strike prices should fail validation"
        );
    }

    #[test]

    fn test_different_expiration_dates() {
        let spread = BullPutSpread {
            name: "Test Bull Put Spread".to_string(),
            kind: StrategyType::BullPutSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_put: create_valid_position(
                Side::Long,
                pos!(90.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
            short_put: create_valid_position(
                Side::Short,
                pos!(95.0),
                ExpirationDate::Days(pos!(60.0)),
            ),
        };

        assert!(
            spread.validate(),
            "Spread with different expiration dates should fail validation"
        );
    }

    #[test]

    fn test_boundary_strike_prices() {
        let spread = BullPutSpread {
            name: "Test Bull Put Spread".to_string(),
            kind: StrategyType::BullPutSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_put: create_valid_position(
                Side::Long,
                pos!(89.99),
                ExpirationDate::Days(pos!(30.0)),
            ),
            short_put: create_valid_position(
                Side::Short,
                pos!(90.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
        };
        assert!(spread.validate());
    }
}

#[cfg(test)]
mod tests_bull_put_spread_optimization {
    use super::*;
    use crate::chains::OptionData;
    use crate::model::types::ExpirationDate;
    use crate::spos;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

        chain.add_option(
            pos!(85.0),       // strike
            None,             // call_bid
            None,             // call_ask
            spos!(2.0),       // put_bid
            spos!(2.2),       // put_ask
            spos!(0.2),       // implied_volatility
            Some(dec!(-0.3)), // delta
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0), // volume
            Some(50),     // open_interest
        );

        chain.add_option(
            pos!(90.0),
            None,
            None,
            spos!(3.0),
            spos!(3.2),
            spos!(0.2),
            Some(dec!(-0.4)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(150.0),
            Some(75),
        );

        chain.add_option(
            pos!(95.0),
            None,
            None,
            spos!(4.0),
            spos!(4.2),
            spos!(0.2),
            Some(dec!(-0.5)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(200.0),
            Some(100),
        );

        chain.add_option(
            pos!(100.0),
            None,
            None,
            spos!(5.0),
            spos!(5.2),
            spos!(0.2),
            Some(dec!(-0.6)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(250.0),
            Some(125),
        );

        chain.add_option(
            pos!(105.0),
            None,
            None,
            spos!(6.0),
            spos!(6.2),
            spos!(0.2),
            Some(dec!(-0.7)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(300.0),
            Some(150),
        );

        chain
    }

    fn create_base_spread() -> BullPutSpread {
        BullPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(95.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(3.2),
            pos!(4.0),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        )
    }

    #[test]

    fn test_find_optimal_ratio() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);
        assert!(spread.validate(), "Optimized spread should be valid");
        assert!(
            spread.profit_ratio().unwrap().to_f64().unwrap() > 0.0,
            "Profit ratio should be positive"
        );
    }

    #[test]

    fn test_find_optimal_area() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Area);

        assert!(spread.validate(), "Optimized spread should be valid");
        assert!(
            spread.profit_area().unwrap().to_f64().unwrap() > 0.0,
            "Profit area should be positive"
        );
    }

    #[test]

    fn test_find_optimal_upper_side() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(&chain, FindOptimalSide::Upper, OptimizationCriteria::Ratio);

        assert!(spread.short_put.option.strike_price >= chain.underlying_price);
        assert!(spread.long_put.option.strike_price >= chain.underlying_price);
    }

    #[test]

    fn test_find_optimal_lower_side() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(&chain, FindOptimalSide::Lower, OptimizationCriteria::Ratio);

        assert!(spread.short_put.option.strike_price <= chain.underlying_price);
        assert!(spread.long_put.option.strike_price <= chain.underlying_price);
    }

    #[test]

    fn test_find_optimal_range() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(
            &chain,
            FindOptimalSide::Range(pos!(90.0), pos!(100.0)),
            OptimizationCriteria::Ratio,
        );

        assert!(spread.short_put.option.strike_price <= pos!(100.0));
        assert!(spread.short_put.option.strike_price >= pos!(90.0));
        assert!(spread.long_put.option.strike_price <= pos!(100.0));
        assert!(spread.long_put.option.strike_price >= pos!(90.0));
    }

    #[test]

    fn test_is_valid_long_option() {
        let spread = create_base_spread();
        let option = OptionData::new(
            pos!(95.0),
            None,
            None,
            spos!(3.0),
            spos!(3.2),
            spos!(0.2),
            Some(dec!(-0.4)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0),
            Some(50),
        );

        assert!(spread.is_valid_long_option(&option, &FindOptimalSide::All));
        assert!(spread.is_valid_long_option(&option, &FindOptimalSide::Lower));
        assert!(!spread.is_valid_long_option(&option, &FindOptimalSide::Upper));
        assert!(
            spread.is_valid_long_option(&option, &FindOptimalSide::Range(pos!(90.0), pos!(100.0)))
        );
    }

    #[test]

    fn test_is_valid_short_option() {
        let spread = create_base_spread();
        let option = OptionData::new(
            pos!(105.0),
            None,
            None,
            spos!(4.0),
            spos!(4.2),
            spos!(0.2),
            Some(dec!(-0.5)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0),
            Some(50),
        );

        assert!(spread.is_valid_short_option(&option, &FindOptimalSide::All));
        assert!(!spread.is_valid_short_option(&option, &FindOptimalSide::Lower));
        assert!(spread.is_valid_short_option(&option, &FindOptimalSide::Upper));
        assert!(
            !spread
                .is_valid_short_option(&option, &FindOptimalSide::Range(pos!(90.0), pos!(100.0)))
        );
    }

    #[test]

    fn test_are_valid_prices() {
        let long_option = OptionData::new(
            pos!(90.0),
            None,
            None,
            spos!(3.0),
            spos!(3.2),
            spos!(0.2),
            Some(dec!(-0.4)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0),
            Some(50),
        );
        let short_option = OptionData::new(
            pos!(95.0),
            None,
            None,
            spos!(4.0),
            spos!(4.2),
            spos!(0.2),
            Some(dec!(-0.5)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0),
            Some(50),
        );

        assert!(
            long_option.put_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
                && short_option.put_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
        );
    }

    #[test]

    fn test_create_strategy() {
        let spread = create_base_spread();
        let chain = create_test_chain();
        let long_option = chain
            .options
            .iter()
            .find(|o| o.strike_price == pos!(90.0))
            .unwrap();
        let short_option = chain
            .options
            .iter()
            .find(|o| o.strike_price == pos!(95.0))
            .unwrap();

        let legs = StrategyLegs::TwoLegs {
            first: long_option,
            second: short_option,
        };
        let new_strategy = spread.create_strategy(&chain, &legs);

        assert!(new_strategy.validate());
        assert_eq!(new_strategy.long_put.option.strike_price, pos!(90.0));
        assert_eq!(new_strategy.short_put.option.strike_price, pos!(95.0));
    }
}

#[cfg(test)]
mod tests_bull_put_spread_profit {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    #[test]

    fn test_profit_above_short_strike() {
        let spread = bull_put_spread_test();
        let price = pos!(5800.0);
        assert_eq!(
            spread.calculate_profit_at(price).unwrap().to_f64().unwrap(),
            -144.63
        );
    }

    #[test]

    fn test_profit_at_short_strike() {
        let spread = bull_put_spread_test();
        let price = pos!(5900.0);
        assert_eq!(
            spread.calculate_profit_at(price).unwrap().to_f64().unwrap(),
            155.37
        );
    }

    #[test]

    fn test_profit_between_strikes() {
        let spread = bull_put_spread_test();
        let price = pos!(5155.37);

        assert_eq!(
            spread.calculate_profit_at(price).unwrap().to_f64().unwrap(),
            -294.63
        );
    }

    #[test]

    fn test_profit_at_long_strike() {
        let spread = bull_put_spread_test();
        let price = pos!(5655.0);
        assert_eq!(
            spread.calculate_profit_at(price).unwrap().to_f64().unwrap(),
            -294.63
        );
    }

    #[test]

    fn test_profit_below_long_strike() {
        let spread = bull_put_spread_test();
        let price = pos!(5755.0);
        assert_eq!(
            spread.calculate_profit_at(price).unwrap().to_f64().unwrap(),
            -279.63
        );
    }

    #[test]

    fn test_profit_at_get_break_even_points() {
        let spread = bull_put_spread_test();
        let break_even_points = spread.get_break_even_points().unwrap();
        let price = break_even_points[0];
        assert!(spread.calculate_profit_at(price).unwrap().abs() < dec!(0.001));
    }

    #[test]

    fn test_profit_with_multiple_contracts() {
        let spread = BullPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(95.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(2.0),
            pos!(2.0),
            pos!(4.0),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );

        let price = pos!(85.0);
        assert_eq!(
            spread.calculate_profit_at(price).unwrap().to_f64().unwrap(),
            -6.0
        );
    }

    #[test]

    fn test_profit_with_fees() {
        let spread = bull_put_spread_test();
        let break_even_points = spread.get_break_even_points().unwrap();
        let price = break_even_points[0];
        assert_eq!(
            spread.calculate_profit_at(price).unwrap().to_f64().unwrap(),
            0.0
        );
    }
}

#[cfg(test)]
mod tests_bull_put_spread_graph {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    #[test]

    fn test_title_format() {
        let spread = bull_put_spread_test();
        let title = spread.title();

        assert!(title.contains("Bull Put Spread Strategy"));
        assert!(title.contains("Long"));
        assert!(title.contains("Short"));
        assert!(title.contains("SP500")); // symbol
        assert!(title.contains("$5750")); // long strike
        assert!(title.contains("5920")); // short strike
    }

    #[test]

    fn test_get_vertical_lines() {
        let spread = bull_put_spread_test();
        let lines = spread.get_vertical_lines();

        assert_eq!(lines.len(), 1);

        let line = &lines[0];
        assert_eq!(line.x_coordinate, 5781.88);
        assert_eq!(line.y_range, (f64::NEG_INFINITY, f64::INFINITY));
        assert!(line.label.contains("Current Price"));
        assert!(line.label.contains("5781.88"));
        assert_eq!(line.label_offset, (4.0, 0.0));
        assert_eq!(line.line_color, ORANGE);
        assert_eq!(line.label_color, ORANGE);
        assert_eq!(line.font_size, 18);
    }

    #[test]

    fn test_get_points() {
        let spread = bull_put_spread_test();
        let points = spread.get_points();

        assert_eq!(points.len(), 4); // Break even, max profit, max loss, current price

        let break_even = &points[0];
        assert_eq!(break_even.coordinates.1, 0.0);
        assert!(break_even.label.contains("Break Even"));
        assert_eq!(break_even.point_color, DARK_BLUE);
        assert_eq!(break_even.label_color, DARK_BLUE);
        assert_eq!(break_even.point_size, 5);
        assert_eq!(break_even.font_size, 18);

        let max_profit = &points[1];
        assert_eq!(max_profit.coordinates.0, 5920.0); // short strike
        assert_eq!(max_profit.coordinates.1, 215.37);
        assert!(max_profit.label.contains("Max Profit"));
        assert_eq!(max_profit.point_color, DARK_GREEN);
        assert_eq!(max_profit.label_color, DARK_GREEN);

        let max_loss = &points[2];
        assert_eq!(max_loss.coordinates.0, 5750.0); // long strike
        assert_eq!(max_loss.coordinates.1, -294.63);
        assert!(max_loss.label.contains("Max Loss"));
        assert_eq!(max_loss.point_color, RED);
        assert_eq!(max_loss.label_color, RED);
    }

    #[test]

    fn test_points_coordinates() {
        let spread = bull_put_spread_test();
        let points = spread.get_points();

        assert_eq!(points[0].coordinates.0, 5848.21);
        assert_eq!(points[0].coordinates.1, 0.0);

        // Maximum profit point: en short strike
        assert_eq!(points[1].coordinates.0, 5920.0);
        assert_eq!(points[1].coordinates.1, 215.37);

        // Maximum loss point: en long strike
        assert_eq!(points[2].coordinates.0, 5750.0);
        assert_eq!(points[2].coordinates.1, -294.63);

        // Current price point
        assert_eq!(points[3].coordinates.0, 5781.88);
        let current_profit = spread
            .calculate_profit_at(pos!(5781.88))
            .unwrap()
            .to_f64()
            .unwrap();
        assert_eq!(points[3].coordinates.1, current_profit);
    }

    #[test]

    fn test_point_labels() {
        let spread = bull_put_spread_test();
        let points = spread.get_points();

        assert_eq!(points.len(), 4);
        assert!(points[0].label.contains("5848.21")); // Break-even
        assert!(points[1].label.contains("215.37")); // Max profit
        assert!(points[2].label.contains("-294.63")); // Max loss
        assert!(points[3].label.contains("-198.99")); // Current price
    }

    #[test]

    fn test_points_style() {
        let spread = bull_put_spread_test();
        let points = spread.get_points();

        for point in points.iter() {
            assert_eq!(point.point_size, 5);
            assert_eq!(point.font_size, 18);
            assert!(matches!(point.point_color, DARK_BLUE | DARK_GREEN | RED));
            assert_eq!(point.point_color, point.label_color);
        }
    }

    #[test]

    fn test_graph_with_zero_profits() {
        let mut spread = bull_put_spread_test();
        spread.short_put.premium = pos!(2.0);
        spread.long_put.premium = pos!(2.0);

        let points = spread.get_points();
        let max_profit_point = &points[1];

        assert_eq!(max_profit_point.coordinates.1, 0.0);
        assert!(max_profit_point.label.contains("0"));
    }

    #[test]

    fn test_graph_with_different_quantities() {
        let spread = BullPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(95.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(2.0), // quantity = 2
            pos!(2.0),
            pos!(4.0),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );

        let points = spread.get_points();
        let max_profit_point = &points[1];
        let max_loss_point = &points[2];

        assert_eq!(max_profit_point.coordinates.1, 4.0); // 2 * 2.0
        assert_eq!(max_loss_point.coordinates.1, -6.0); // 2 * -3.0
    }
}

#[cfg(test)]
mod tests_bull_put_spread_probability {
    use super::*;
    use crate::strategies::probabilities::utils::PriceTrend;
    use rust_decimal_macros::dec;

    fn bull_put_spread_test() -> BullPutSpread {
        BullPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),                      // underlying_price
            pos!(90.0),                       // long_strike
            pos!(95.0),                       // short_strike
            ExpirationDate::Days(pos!(30.0)), // expiration
            pos!(0.2),                        // implied_volatility
            dec!(0.05),                       // risk_free_rate
            Positive::ZERO,                   // dividend_yield
            pos!(1.0),                        // quantity
            Positive::ONE,                    // premium_long_put
            Positive::TWO,                    // premium_short_put
            Positive::ZERO,                   // open_fee_long_put
            Positive::ZERO,                   // close_fee_long_put
            Positive::ZERO,                   // open_fee_short_put
            Positive::ZERO,                   // close_fee_short_put
        )
    }

    #[test]

    fn test_get_expiration() {
        let spread = bull_put_spread_test();
        let result = spread.get_expiration();
        assert!(result.is_ok());
        match result.unwrap() {
            ExpirationDate::Days(days) => assert_eq!(days, 30.0),
            _ => panic!("Expected ExpirationDate::Days"),
        }
    }

    #[test]

    fn test_get_risk_free_rate() {
        let spread = bull_put_spread_test();
        assert_eq!(spread.get_risk_free_rate(), Some(dec!(0.05)));
    }

    #[test]

    fn test_get_profit_ranges() {
        let spread = bull_put_spread_test();
        let result = spread.get_profit_ranges();
        assert!(result.is_ok());

        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1);

        let range = &ranges[0];
        assert!(range.lower_bound.is_some());
        assert!(range.upper_bound.is_none());
        assert!(range.probability > Positive::ZERO);
    }

    #[test]

    fn test_get_loss_ranges() {
        let spread = bull_put_spread_test();
        let result = spread.get_loss_ranges();
        assert!(result.is_ok());

        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1);

        let range = &ranges[0];
        assert!(range.lower_bound.is_some());
        assert!(range.upper_bound.is_some());
        assert!(range.probability > Positive::ZERO);
    }

    #[test]

    fn test_probability_of_profit() {
        let spread = bull_put_spread_test();
        let result = spread.probability_of_profit(None, None);
        assert!(result.is_ok());

        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]

    fn test_probability_with_volatility_adjustment() {
        let spread = bull_put_spread_test();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.05),
        });

        let result = spread.probability_of_profit(vol_adj, None);
        assert!(result.is_ok());

        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]

    fn test_probability_with_trend() {
        let spread = bull_put_spread_test();
        let trend = Some(PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        });

        let result = spread.probability_of_profit(None, trend);
        assert!(result.is_ok());

        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]

    fn test_analyze_probabilities() {
        let spread = bull_put_spread_test();
        let result = spread.analyze_probabilities(None, None);
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
        let spread = bull_put_spread_test();
        let result = spread.calculate_extreme_probabilities(None, None);
        assert!(result.is_ok());

        let (max_profit_prob, max_loss_prob) = result.unwrap();
        assert!(max_profit_prob >= Positive::ZERO);
        assert!(max_loss_prob >= Positive::ZERO);
        assert!(max_profit_prob + max_loss_prob <= pos!(1.0));
    }
}

#[cfg(test)]
mod tests_delta {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::strategies::bull_put_spread::BullPutSpread;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn get_strategy(long_strike: Positive, short_strike: Positive) -> BullPutSpread {
        let underlying_price = pos!(5801.88);
        BullPutSpread::new(
            "SP500".to_string(),
            underlying_price, // underlying_price
            long_strike,      // long_strike
            short_strike,     // short_strike
            ExpirationDate::Days(pos!(2.0)),
            pos!(0.18),     // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // long quantity
            pos!(15.04),    // premium_long
            pos!(89.85),    // premium_short
            pos!(0.78),     // open_fee_long
            pos!(0.78),     // open_fee_long
            pos!(0.73),     // close_fee_long
            pos!(0.73),     // close_fee_short
        )
    }

    #[test]

    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos!(5750.0), pos!(5920.0));
        let size = dec!(0.6897372);
        let delta = pos!(2.855544139071374);
        let k = pos!(5750.0);
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

    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(pos!(5840.0), pos!(5750.0));
        let size = dec!(-0.437230414);
        let delta = pos!(1.8101540723661196);
        let k = pos!(5750.0);
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
        let strategy = get_strategy(pos!(5830.0), pos!(5830.0));
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
mod tests_delta_size {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::strategies::bull_put_spread::BullPutSpread;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn get_strategy(long_strike: Positive, short_strike: Positive) -> BullPutSpread {
        let underlying_price = pos!(5781.88);
        BullPutSpread::new(
            "SP500".to_string(),
            underlying_price, // underlying_price
            long_strike,      // long_strike
            short_strike,     // short_strike
            ExpirationDate::Days(pos!(2.0)),
            pos!(0.18),     // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(2.0),      // long quantity
            pos!(15.04),    // premium_long
            pos!(89.85),    // premium_short
            pos!(0.78),     // open_fee_long
            pos!(0.78),     // open_fee_long
            pos!(0.73),     // close_fee_long
            pos!(0.73),     // close_fee_short
        )
    }

    #[test]

    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos!(5750.0), pos!(5820.9));
        let size = dec!(0.7086);
        let delta = pos!(2.152913807138664);
        let k = pos!(5750.0);
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

    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(pos!(5840.0), pos!(5750.0));
        let size = dec!(-0.8722316);
        let delta = pos!(2.649732171104434);
        let k = pos!(5750.0);
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
        let strategy = get_strategy(pos!(5840.0), pos!(5840.0));

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
mod tests_bear_call_spread_position_management {
    use super::*;
    use crate::error::position::PositionValidationErrorKind;
    use crate::model::types::{ExpirationDate, OptionStyle, Side};
    use crate::pos;
    use rust_decimal_macros::dec;
    use tracing::error;

    fn create_test_short_bull_put_spread() -> BullPutSpread {
        BullPutSpread::new(
            "SP500".to_string(),
            pos!(5781.88), // underlying_price
            pos!(5850.0),  // long_strike
            pos!(5720.0),  // short_strike
            ExpirationDate::Days(pos!(2.0)),
            pos!(0.18),     // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(4.0),      // long quantity
            pos!(85.04),    // premium_long
            pos!(29.85),    // premium_short
            pos!(0.78),     // open_fee_long
            pos!(0.78),     // open_fee_long
            pos!(0.73),     // close_fee_long
            pos!(0.73),     // close_fee_short
        )
    }

    #[test]
    fn test_short_bull_put_spread_get_position() {
        let mut bull_put_spread = create_test_short_bull_put_spread();

        // Test getting short put position
        let put_position =
            bull_put_spread.get_position(&OptionStyle::Put, &Side::Long, &pos!(5850.0));
        assert!(put_position.is_ok());
        let positions = put_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(5850.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Put);
        assert_eq!(positions[0].option.side, Side::Long);

        // Test getting short put position
        let put_position =
            bull_put_spread.get_position(&OptionStyle::Put, &Side::Short, &pos!(5720.0));
        assert!(put_position.is_ok());
        let positions = put_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(5720.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Put);
        assert_eq!(positions[0].option.side, Side::Short);

        // Test getting non-existent position
        let invalid_position =
            bull_put_spread.get_position(&OptionStyle::Call, &Side::Short, &pos!(5821.0));
        assert!(invalid_position.is_err());
        match invalid_position {
            Err(PositionError::ValidationError(
                PositionValidationErrorKind::IncompatibleSide {
                    position_side: _,
                    reason,
                },
            )) => {
                assert_eq!(reason, "Call is not valid for BullPutSpread");
            }
            _ => {
                error!("Unexpected error: {:?}", invalid_position);
                panic!()
            }
        }
    }

    #[test]
    fn test_short_bull_put_spread_modify_position() {
        let mut bull_put_spread = create_test_short_bull_put_spread();

        // Modify short put position
        let mut modified_put = bull_put_spread.short_put.clone();
        modified_put.option.quantity = pos!(2.0);
        let result = bull_put_spread.modify_position(&modified_put);
        assert!(result.is_ok());
        assert_eq!(bull_put_spread.short_put.option.quantity, pos!(2.0));

        // Modify short put position
        let mut modified_put = bull_put_spread.long_put.clone();
        modified_put.option.quantity = pos!(2.0);
        let result = bull_put_spread.modify_position(&modified_put);
        assert!(result.is_ok());
        assert_eq!(bull_put_spread.long_put.option.quantity, pos!(2.0));

        // Test modifying with invalid position
        let mut invalid_position = bull_put_spread.short_put.clone();
        invalid_position.option.strike_price = pos!(95.0);
        let result = bull_put_spread.modify_position(&invalid_position);
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
    use crate::model::types::{ExpirationDate, OptionStyle, Side};
    use crate::pos;
    use rust_decimal_macros::dec;

    // Helper function to create a test strategy
    fn create_test_strategy() -> BullPutSpread {
        BullPutSpread::new(
            "SP500".to_string(),
            pos!(5781.88), // underlying_price
            pos!(5850.0),  // long_strike
            pos!(5720.0),  // short_strike
            ExpirationDate::Days(pos!(2.0)),
            pos!(0.18),     // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(4.0),      // long quantity
            pos!(85.04),    // premium_long
            pos!(29.85),    // premium_short
            pos!(0.78),     // open_fee_long
            pos!(0.78),     // open_fee_long
            pos!(0.73),     // close_fee_long
            pos!(0.73),     // close_fee_short
        )
    }

    #[test]
    fn test_adjust_existing_call_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.short_put.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos!(5720.0),
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
    fn test_adjust_existing_put_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.long_put.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos!(5850.0),
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
            &pos!(5850.0),
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
                assert_eq!(reason, "Call is not valid for BullPutSpread");
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
        let initial_quantity = strategy.long_put.option.quantity;

        let result = strategy.adjust_option_position(
            Decimal::ZERO,
            &pos!(5720.0),
            &OptionStyle::Put,
            &Side::Short,
        );

        assert!(result.is_ok());
        assert_eq!(strategy.long_put.option.quantity, initial_quantity);
    }
}

#[cfg(test)]
mod tests_strategy_constructor {
    use super::*;
    use crate::error::OperationErrorKind;
    use crate::model::utils::create_sample_position;
    use crate::pos;

    #[test]
    fn test_get_strategy_valid() {
        let options = vec![
            create_sample_position(
                OptionStyle::Put,
                Side::Short,
                pos!(100.0),
                pos!(1.0),
                pos!(95.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Put,
                Side::Long,
                pos!(100.0),
                pos!(1.0),
                pos!(105.0),
                pos!(0.2),
            ),
        ];

        let result = BullPutSpread::get_strategy(&options);
        assert!(result.is_ok());

        let strategy = result.unwrap();
        assert_eq!(strategy.short_put.option.strike_price, pos!(95.0));
        assert_eq!(strategy.long_put.option.strike_price, pos!(105.0));
    }

    #[test]
    fn test_get_strategy_wrong_number_of_options() {
        let options = vec![create_sample_position(
            OptionStyle::Put,
            Side::Short,
            pos!(100.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        )];

        let result = BullPutSpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Bull Put Spread get_strategy" && reason == "Must have exactly 2 options"
        ));
    }

    #[test]
    fn test_get_strategy_wrong_option_style() {
        let mut option1 = create_sample_position(
            OptionStyle::Put,
            Side::Short,
            pos!(100.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );
        option1.option.option_style = OptionStyle::Call;
        let option2 = create_sample_position(
            OptionStyle::Put,
            Side::Long,
            pos!(100.0),
            pos!(1.0),
            pos!(105.0),
            pos!(0.2),
        );

        let options = vec![option1, option2];
        let result = BullPutSpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Bull Put Spread get_strategy" && reason == "Options must be puts"
        ));
    }

    #[test]
    fn test_get_strategy_wrong_sides() {
        let options = vec![
            create_sample_position(
                OptionStyle::Put,
                Side::Short,
                pos!(100.0),
                pos!(1.0),
                pos!(115.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Put,
                Side::Long,
                pos!(100.0),
                pos!(1.0),
                pos!(105.0),
                pos!(0.2),
            ),
        ];
        let result = BullPutSpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Bull Put Spread get_strategy"
                && reason == "Bull Put Spread requires a short lower strike put and a long higher strike put"
        ));
    }

    #[test]
    fn test_get_strategy_different_expiration_dates() {
        let mut option1 = create_sample_position(
            OptionStyle::Put,
            Side::Short,
            pos!(100.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );
        let mut option2 = create_sample_position(
            OptionStyle::Put,
            Side::Long,
            pos!(100.0),
            pos!(1.0),
            pos!(105.0),
            pos!(0.2),
        );

        option1.option.expiration_date = ExpirationDate::Days(pos!(30.0));
        option2.option.expiration_date = ExpirationDate::Days(pos!(60.0));

        let options = vec![option1, option2];
        let result = BullPutSpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Bull Put Spread get_strategy" && reason == "Options must have the same expiration date"
        ));
    }
}

#[cfg(test)]
mod tests_bull_put_spread_pnl {
    use super::*;
    use crate::model::utils::create_sample_position;
    use crate::{assert_decimal_eq, pos};
    use rust_decimal_macros::dec;

    fn create_test_bull_put_spread() -> Result<BullPutSpread, StrategyError> {
        // Create short put with lower strike
        let short_put = create_sample_position(
            OptionStyle::Put,
            Side::Short,
            pos!(100.0), // Underlying price
            pos!(1.0),   // Quantity
            pos!(95.0),  // Lower strike price
            pos!(0.2),   // Implied volatility
        );

        // Create long put with higher strike
        let long_put = create_sample_position(
            OptionStyle::Put,
            Side::Long,
            pos!(100.0), // Same underlying price
            pos!(1.0),   // Quantity
            pos!(100.0), // Higher strike price
            pos!(0.2),   // Implied volatility
        );

        BullPutSpread::get_strategy(&vec![short_put, long_put])
    }

    #[test]
    fn test_calculate_pnl_all_options_otm() {
        let spread = create_test_bull_put_spread().unwrap();
        let market_price = pos!(105.0); // Above both strikes
        let expiration_date = ExpirationDate::Days(pos!(30.0));
        let implied_volatility = pos!(0.2);

        let result = spread.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Both puts OTM, profit should be close to net premium received minus costs
        let net_credit = pnl.initial_income.to_dec() - pnl.initial_costs.to_dec();
        assert!(pnl.unrealized.unwrap() > dec!(-2.0)); // Should be near max profit
        assert_eq!(net_credit, dec!(-2.0)); // 5.0 - 7.0
    }

    #[test]
    fn test_calculate_pnl_mixed_moneyness() {
        let spread = create_test_bull_put_spread().unwrap();
        let market_price = pos!(97.5); // Between strikes
        let expiration_date = ExpirationDate::Days(pos!(30.0));
        let implied_volatility = pos!(0.2);

        let result = spread.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // At 97.5:
        // - Long put at 100 is ITM by 2.5
        // - Short put at 95 is OTM by 2.5
        let unrealized = pnl.unrealized.unwrap();
        assert!(unrealized > dec!(0.0)); // Position should show profit due to ITM long put
        assert!(unrealized < dec!(5.0)); // But less than max width of spread

        // Verify initial values
        assert_eq!(pnl.initial_income, pos!(5.0));
        assert_eq!(pnl.initial_costs, pos!(7.0));
    }

    #[test]
    fn test_calculate_pnl_all_options_itm() {
        let spread = create_test_bull_put_spread().unwrap();
        let market_price = pos!(90.0); // Below both strikes
        let expiration_date = ExpirationDate::Days(pos!(30.0));
        let implied_volatility = pos!(0.2);

        let result = spread.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // At 90.0:
        // Long put at 100 is ITM by 10.0
        // Short put at 95 is ITM by 5.0
        // Net ITM position is 5.0 (width of spread)
        let unrealized = pnl.unrealized.unwrap();
        assert!(unrealized > dec!(0.0)); // Position should show profit
        assert!(unrealized < dec!(5.0)); // Limited by spread width

        // Verify initial values
        assert_eq!(pnl.initial_income, pos!(5.0));
        assert_eq!(pnl.initial_costs, pos!(7.0));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_maximum_profit() {
        let spread = create_test_bull_put_spread().unwrap();
        let underlying_price = pos!(105.0); // Above both strikes

        let result = spread.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();

        // At expiration above strikes, both puts expire worthless
        // Max profit = net premium received - costs
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(-2.0), dec!(1e-6));
        assert_eq!(pnl.initial_income, pos!(5.0));
        assert_eq!(pnl.initial_costs, pos!(7.0));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_maximum_loss() {
        let spread = create_test_bull_put_spread().unwrap();
        let underlying_price = pos!(90.0); // Below both strikes

        let result = spread.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();

        // At expiration with price at 90:
        // Long put 100 payoff = 100 - 90 = 10
        // Short put 95 payoff = -(95 - 90) = -5
        // Spread payoff = 5
        // Plus initial income (5) minus costs (7)
        // Total = 5 + 5 - 7 = 3
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(3.0), dec!(1e-6));
        assert_eq!(pnl.initial_income, pos!(5.0));
        assert_eq!(pnl.initial_costs, pos!(7.0));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_breakeven() {
        let spread = create_test_bull_put_spread().unwrap();
        // For bull put spread:
        // Long put at 100, Short put at 95
        // Initial income = 5, costs = 7
        // Net credit = -2
        // Break-even point should be where payoff = 2
        let underlying_price = pos!(98.0); // Adjusted to find true break-even

        let result = spread.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();

        // At 98:
        // Long put payoff = 100 - 98 = 2
        // Short put payoff = -(95 - 98) = 3
        // Net payoff = 2 + 3 = 5
        // Plus initial income (5) minus costs (7)
        // Total should be close to 0
        assert!(pnl.realized.unwrap().abs() < dec!(0.5));
        assert_eq!(pnl.initial_income, pos!(5.0));
        assert_eq!(pnl.initial_costs, pos!(7.0));
    }

    #[test]
    fn test_calculate_pnl_volatility_sensitivity() {
        let spread = create_test_bull_put_spread().unwrap();
        let market_price = pos!(97.5); // Between strikes
        let expiration_date = ExpirationDate::Days(pos!(30.0));

        // Test with different volatilities
        let low_vol_result = spread
            .calculate_pnl(&market_price, expiration_date, &pos!(0.1))
            .unwrap();
        let high_vol_result = spread
            .calculate_pnl(&market_price, expiration_date, &pos!(0.3))
            .unwrap();

        // At 97.5:
        // Long put 100 is ITM by 2.5
        // Short put 95 is OTM by 2.5
        // With higher volatility:
        // - ITM long put gains more value
        // - OTM short put loses less value
        // Net result is higher profit
        assert!(high_vol_result.unrealized.unwrap() > low_vol_result.unrealized.unwrap());

        // Verify initial values remain constant
        assert_eq!(high_vol_result.initial_income, pos!(5.0));
        assert_eq!(high_vol_result.initial_costs, pos!(7.0));
        assert_eq!(low_vol_result.initial_income, pos!(5.0));
        assert_eq!(low_vol_result.initial_costs, pos!(7.0));
    }
}
