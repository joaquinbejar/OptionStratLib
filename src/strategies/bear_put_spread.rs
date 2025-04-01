/*
Bear Put Spread Strategy

A bear put spread involves buying a put option with a higher strike price and selling a put option with a lower strike price,
both with the same expiration date. This strategy is used when a moderate decline in the underlying asset's price is expected.

Key characteristics:
- Limited profit potential (difference between strikes minus net premium paid)
- Limited risk (net premium paid)
- Bearish strategy that profits from price decrease
- Both options have same expiration date
- Lower cost than buying puts outright
- Maximum profit achieved when price falls below lower strike
- Also known as a vertical put debit spread
*/
use crate::chains::StrategyLegs;
use crate::chains::chain::OptionChain;
use crate::chains::utils::OptionDataGroup;
use crate::constants::{DARK_BLUE, DARK_GREEN};
use crate::error::position::{PositionError, PositionValidationErrorKind};
use crate::error::probability::ProbabilityError;
use crate::error::strategies::{ProfitLossErrorKind, StrategyError};
use crate::error::{GreeksError, OperationErrorKind};
use crate::greeks::Greeks;
use crate::model::utils::mean_and_std;
use crate::model::{Position, ProfitLossRange};
use crate::pnl::utils::{PnL, PnLCalculator};
use crate::pricing::Profit;
use crate::strategies::base::{
    BreakEvenable, Optimizable, Positionable, Strategable, StrategyType, Validable,
};
use crate::strategies::probabilities::{ProbabilityAnalysis, VolatilityAdjustment};
use crate::strategies::utils::OptimizationCriteria;
use crate::strategies::{DeltaNeutrality, FindOptimalSide, Strategies};
use crate::strategies::{StrategyBasics, StrategyConstructor};
use crate::visualization::model::{ChartPoint, ChartVerticalLine, LabelOffsetType};
use crate::visualization::utils::Graph;
use crate::{ExpirationDate, OptionStyle, OptionType, Options, Positive, Side, pos};
use chrono::Utc;
use plotters::prelude::full_palette::ORANGE;
use plotters::prelude::{RED, ShapeStyle};
use rust_decimal::Decimal;
use std::error::Error;
use tracing::{debug, info};

const BEAR_PUT_SPREAD_DESCRIPTION: &str = "A bear put spread is created by buying a put option with a higher strike price \
    and simultaneously selling a put option with a lower strike price, both with the same \
    expiration date. This strategy is used when you expect a moderate decrease in the underlying \
    asset's price. The maximum profit is limited to the difference between strike prices minus \
    the net premium paid, while the maximum loss is limited to the net premium paid.";

/// Represents a Bear Put Spread options trading strategy.
///
/// A Bear Put Spread is a bearish options strategy that involves buying a put option at a
/// higher strike price and simultaneously selling another put option at a lower strike price,
/// both with the same expiration date. This strategy is used when expecting a moderate
/// decline in the price of the underlying asset.
///
/// The strategy benefits from limited risk (the net premium paid) and limited profit potential
/// (the difference between strike prices minus the net premium paid). It is less expensive than
/// buying a single put outright due to premium received from the short put.
///
/// # Attributes
#[derive(Clone, Debug)]
pub struct BearPutSpread {
    /// The name identifier for this specific strategy instance.
    pub name: String,
    /// The type of strategy, should be StrategyType::BearPutSpread.
    pub kind: StrategyType,
    /// A detailed description of this specific strategy implementation.
    pub description: String,
    /// The price points at which the strategy breaks even (neither profit nor loss).
    pub break_even_points: Vec<Positive>,
    /// The long put position with the higher strike price.
    long_put: Position,
    /// The short put position with the lower strike price.
    short_put: Position,
}

impl BearPutSpread {
    /// Creates a new Bear Put Spread options strategy.
    ///
    /// A bear put spread is created by buying a put option with a higher strike price
    /// and simultaneously selling a put option with a lower strike price, both with the same
    /// expiration date. This strategy is used when you expect a moderate decrease in the
    /// underlying asset's price.
    ///
    /// # Parameters
    ///
    /// * `underlying_symbol` - The symbol of the underlying asset.
    /// * `underlying_price` - The current price of the underlying asset.
    /// * `long_strike` - Strike price for the long put position. If set to zero, defaults to the underlying price.
    /// * `short_strike` - Strike price for the short put position. If set to zero, defaults to the underlying price.
    /// * `expiration` - The expiration date of the options contracts.
    /// * `implied_volatility` - The implied volatility used for option pricing calculations.
    /// * `risk_free_rate` - The risk-free interest rate used in option pricing models.
    /// * `dividend_yield` - The dividend yield of the underlying asset.
    /// * `quantity` - The number of option contracts in the strategy.
    /// * `premium_long_put` - The premium paid for the long put option.
    /// * `premium_short_put` - The premium received for the short put option.
    /// * `open_fee_long_put` - The fee paid when opening the long put position.
    /// * `close_fee_long_put` - The fee paid when closing the long put position.
    /// * `open_fee_short_put` - The fee paid when opening the short put position.
    /// * `close_fee_short_put` - The fee paid when closing the short put position.
    ///
    /// # Returns
    ///
    /// A validated `BearPutSpread` strategy instance with calculated break-even points.
    ///
    /// # Validation
    ///
    /// The function performs validation to ensure:
    /// - Both put positions are valid
    /// - The long put strike price is higher than the short put strike price
    ///
    /// # Note
    ///
    /// The maximum profit is limited to the difference between strike prices minus
    /// the net premium paid, while the maximum loss is limited to the net premium paid.
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

        let mut strategy = BearPutSpread {
            name: "Bear Put Spread".to_string(),
            kind: StrategyType::BearPutSpread,
            description: BEAR_PUT_SPREAD_DESCRIPTION.to_string(),
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

impl StrategyConstructor for BearPutSpread {
    fn get_strategy(vec_options: &[Position]) -> Result<Self, StrategyError> {
        // Need exactly 2 options for a bear put spread
        if vec_options.len() != 2 {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Bear Put Spread get_strategy".to_string(),
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
                    operation: "Bear Put Spread get_strategy".to_string(),
                    reason: "Options must be puts".to_string(),
                },
            ));
        }

        // Validate option sides
        if lower_strike_option.option.side != Side::Long
            || higher_strike_option.option.side != Side::Short
        {
            return Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters {
                operation: "Bear Put Spread get_strategy".to_string(),
                reason: "Bear Put Spread requires a long lower strike put and a short higher strike put".to_string(),
            }));
        }

        // Validate expiration dates match
        if lower_strike_option.option.expiration_date != higher_strike_option.option.expiration_date
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Bear Put Spread get_strategy".to_string(),
                    reason: "Options must have the same expiration date".to_string(),
                },
            ));
        }

        // Create positions
        let long_put = Position::new(
            lower_strike_option.option.clone(),
            lower_strike_option.premium,
            Utc::now(),
            lower_strike_option.open_fee,
            lower_strike_option.close_fee,
        );

        let short_put = Position::new(
            higher_strike_option.option.clone(),
            higher_strike_option.premium,
            Utc::now(),
            higher_strike_option.open_fee,
            higher_strike_option.close_fee,
        );

        // Create strategy
        let mut strategy = BearPutSpread {
            name: "Bear Put Spread".to_string(),
            kind: StrategyType::BearPutSpread,
            description: BEAR_PUT_SPREAD_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_put,
            short_put,
        };

        // Validate and update break-even points
        strategy.validate();
        strategy.update_break_even_points()?;

        Ok(strategy)
    }
}

impl BreakEvenable for BearPutSpread {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        self.break_even_points.push(
            (self.long_put.option.strike_price - self.net_cost()? / self.long_put.option.quantity)
                .round_to(2),
        );

        Ok(())
    }
}

impl Positionable for BearPutSpread {
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
                "Call is not valid for BearPutSpread".to_string(),
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
                    "Call is not valid for BearPutSpread".to_string(),
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

impl Strategable for BearPutSpread {
    fn info(&self) -> Result<StrategyBasics, StrategyError> {
        Ok(StrategyBasics {
            name: self.name.clone(),
            kind: self.kind.clone(),
            description: self.description.clone(),
        })
    }
}

impl Strategies for BearPutSpread {
    fn get_underlying_price(&self) -> Positive {
        self.long_put.option.underlying_price
    }

    fn max_profit(&self) -> Result<Positive, StrategyError> {
        let profit = self.calculate_profit_at(self.short_put.option.strike_price)?;
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

    fn max_loss(&self) -> Result<Positive, StrategyError> {
        let loss = self.calculate_profit_at(self.long_put.option.strike_price)?;
        if loss <= Decimal::ZERO {
            Ok(loss.abs().into())
        } else {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxLossError {
                    reason: "Max loss must be negative".to_string(),
                },
            ))
        }
    }

    fn profit_area(&self) -> Result<Decimal, StrategyError> {
        let high = self.max_profit().unwrap_or(Positive::ZERO);
        let base = self.break_even_points[0] - self.short_put.option.strike_price;
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

impl Validable for BearPutSpread {
    fn validate(&self) -> bool {
        if !self.long_put.validate() {
            debug!("Long put is invalid");
            return false;
        }
        if !self.short_put.validate() {
            debug!("Short put is invalid");
            return false;
        }
        if self.long_put.option.strike_price <= self.short_put.option.strike_price {
            debug!(
                "Long put strike price {} must be higher than short put strike price {}",
                self.long_put.option.strike_price, self.short_put.option.strike_price
            );
            return false;
        }
        true
    }
}

impl Optimizable for BearPutSpread {
    type Strategy = BearPutSpread;

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
            .filter(move |&(short, long)| {
                if side == FindOptimalSide::Center {
                    long.is_valid_optimal_side(underlying_price, &FindOptimalSide::Upper)
                        && short.is_valid_optimal_side(underlying_price, &FindOptimalSide::Lower)
                } else {
                    long.is_valid_optimal_side(underlying_price, &side)
                        && short.is_valid_optimal_side(underlying_price, &side)
                }
            })
            // Filter out options with invalid bid/ask prices
            .filter(|(short, long)| {
                long.put_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && short.put_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
            })
            // Filter out options that don't meet strategy constraints
            .filter(move |(short, long)| {
                let legs = StrategyLegs::TwoLegs {
                    first: short,
                    second: long,
                };
                let strategy = strategy.create_strategy(option_chain, &legs);
                strategy.validate() && strategy.max_profit().is_ok() && strategy.max_loss().is_ok()
            })
            // Map to OptionDataGroup
            .map(move |(short, long)| OptionDataGroup::Two(short, long))
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
            let (short, long) = match option_data_group {
                OptionDataGroup::Two(first, second) => (first, second),
                _ => panic!("Invalid OptionDataGroup"),
            };

            let legs = StrategyLegs::TwoLegs {
                first: short,
                second: long,
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
        let (short, long) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };
        let implied_volatility = long.implied_volatility.unwrap();
        assert!(implied_volatility <= Positive::ONE);
        BearPutSpread::new(
            chain.symbol.clone(),
            chain.underlying_price,
            long.strike_price,
            short.strike_price,
            self.long_put.option.expiration_date,
            implied_volatility,
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

impl Profit for BearPutSpread {
    fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
        let price = Some(&price);
        Ok(self.long_put.pnl_at_expiration(&price)? + self.short_put.pnl_at_expiration(&price)?)
    }
}

impl Graph for BearPutSpread {
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
            .unwrap_or_else(|_| vec![self.short_put.option.strike_price])
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        let underlying_price = self.long_put.option.underlying_price.to_f64();
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
            label_offset: LabelOffsetType::Relative(10.0, 5.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        // Maximum Profit Point (at lower strike price)
        points.push(ChartPoint {
            coordinates: (
                self.short_put.option.strike_price.to_f64(),
                self.max_profit().unwrap_or(Positive::ZERO).to_f64(),
            ),
            label: format!(
                "Max Profit {:.2}",
                self.max_profit().unwrap_or(Positive::ZERO)
            ),
            label_offset: LabelOffsetType::Relative(10.0, 5.0),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        // Maximum Loss Point (at higher strike price)
        points.push(ChartPoint {
            coordinates: (
                self.long_put.option.strike_price.to_f64(),
                -self.max_loss().unwrap_or(Positive::ZERO).to_f64(),
            ),
            label: format!("Max Loss -{:.2}", self.max_loss().unwrap_or(Positive::ZERO)),
            label_offset: LabelOffsetType::Relative(-60.0, -5.0),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        // Current Price Point
        points.push(self.get_point_at_price(self.long_put.option.underlying_price));

        points
    }
}

impl ProbabilityAnalysis for BearPutSpread {
    fn get_expiration(&self) -> Result<ExpirationDate, ProbabilityError> {
        Ok(self.long_put.option.expiration_date)
    }

    fn get_risk_free_rate(&self) -> Option<Decimal> {
        Some(self.long_put.option.risk_free_rate)
    }

    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_point = self.get_break_even_points()?[0];

        let (mean_volatility, std_dev) = mean_and_std(vec![
            self.long_put.option.implied_volatility,
            self.short_put.option.implied_volatility,
        ]);

        let mut profit_range = ProfitLossRange::new(
            Some(self.short_put.option.strike_price), // Price below short strike is max profit
            Some(break_even_point),                   // Upper bound is break even point
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
            self.long_put.option.implied_volatility,
            self.short_put.option.implied_volatility,
        ]);

        let mut loss_range = ProfitLossRange::new(
            Some(break_even_point), // Lower bound is break even point
            None,                   // No upper bound (theoretically)
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

impl Greeks for BearPutSpread {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.long_put.option, &self.short_put.option])
    }
}

impl DeltaNeutrality for BearPutSpread {}

impl PnLCalculator for BearPutSpread {
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
mod tests_bear_put_spread_strategy {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn create_test_spread() -> BearPutSpread {
        BearPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),                      // underlying_price
            pos!(105.0),                      // long_strike
            pos!(95.0),                       // short_strike
            ExpirationDate::Days(pos!(30.0)), // expiration
            pos!(0.2),                        // implied_volatility
            dec!(0.05),                       // risk_free_rate
            Positive::ZERO,                   // dividend_yield
            pos!(1.0),                        // quantity
            pos!(4.0),                        // premium_long_put
            pos!(2.0),                        // premium_short_put
            Positive::ZERO,                   // open_fee_long_put
            Positive::ZERO,                   // close_fee_long_put
            Positive::ZERO,                   // open_fee_short_put
            Positive::ZERO,                   // close_fee_short_put
        )
    }

    #[test]

    fn test_new_bear_put_spread() {
        let spread = create_test_spread();

        assert_eq!(spread.name, "Bear Put Spread");
        assert_eq!(spread.kind, StrategyType::BearPutSpread);
        assert!(!spread.description.is_empty());
        assert_eq!(spread.get_underlying_price(), pos!(100.0));
        assert_eq!(spread.long_put.option.strike_price, pos!(105.0));
        assert_eq!(spread.short_put.option.strike_price, pos!(95.0));
    }

    #[test]

    fn test_add_leg() {
        let mut spread = create_test_spread();
        let new_long_put = Position::new(
            Options::new(
                OptionType::European,
                Side::Long,
                "TEST".to_string(),
                pos!(110.0),
                ExpirationDate::Days(pos!(30.0)),
                pos!(0.2),
                pos!(1.0),
                pos!(100.0),
                dec!(0.05),
                OptionStyle::Put,
                Positive::ZERO,
                None,
            ),
            pos!(5.0),
            Utc::now(),
            Positive::ZERO,
            Positive::ZERO,
        );

        spread
            .add_position(&new_long_put.clone())
            .expect("Error adding long put");
        assert_eq!(spread.long_put.option.strike_price, pos!(110.0));
    }

    #[test]

    fn test_get_legs() {
        let spread = create_test_spread();
        let legs = spread.get_positions().expect("Error getting legs");

        assert_eq!(legs.len(), 2);
        assert_eq!(legs[0].option.side, Side::Long);
        assert_eq!(legs[1].option.side, Side::Short);
    }

    #[test]

    fn test_max_profit() {
        let spread = create_test_spread();
        let max_profit = spread.max_profit().unwrap();
        // Width (105 - 95 = 10) - Net Premium (4 - 2 = 2)
        assert_eq!(max_profit, pos!(8.0));
    }

    #[test]

    fn test_max_loss() {
        let spread = create_test_spread();
        let max_loss = spread.max_loss().unwrap();
        // Net Premium Paid (4 - 2 = 2)
        assert_eq!(max_loss, pos!(2.0));
    }

    #[test]

    fn test_total_cost() {
        let spread = create_test_spread();
        // Long Premium - Short Premium (4 - 2 = 2)
        assert_eq!(spread.total_cost().unwrap(), dec!(4.0));
    }

    #[test]

    fn test_net_premium_received() {
        let spread = create_test_spread();
        // Net Premium Received is actually Net Premium Paid in this case
        assert_eq!(spread.net_premium_received().unwrap().to_f64(), 0.0);
    }

    #[test]

    fn test_fees() {
        let spread = BearPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(105.0),
            pos!(95.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(4.0),
            pos!(2.0),
            pos!(0.5), // open_fee_long_put
            pos!(0.5), // close_fee_long_put
            pos!(0.5), // open_fee_short_put
            pos!(0.5), // close_fee_short_put
        );

        assert_eq!(spread.fees().unwrap().to_f64(), 2.0); // Total fees = 0.5 * 4
    }

    #[test]

    fn test_break_even_points() {
        let spread = create_test_spread();
        let break_even_points = spread.get_break_even_points().unwrap();

        assert_eq!(break_even_points.len(), 1);
        // Break-even = Long Strike - Net Premium / Quantity
        assert_eq!(break_even_points[0], pos!(103.0)); // 105 - 2/1
    }

    #[test]

    fn test_profit_area() {
        let spread = create_test_spread();
        let area = spread.profit_area().unwrap().to_f64().unwrap();
        assert!(area > 0.0);
    }

    #[test]

    fn test_profit_ratio() {
        let spread = create_test_spread();
        let ratio = spread.profit_ratio().unwrap().to_f64().unwrap();
        // Ratio = (max_profit / max_loss) * 100
        // = (8.0 / 2.0) * 100 = 400.0
        assert_eq!(ratio, 400.0);
    }

    #[test]

    fn test_default_strikes() {
        let spread = BearPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            Positive::ZERO, // long_strike = default
            Positive::ZERO, // short_strike = default
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(4.0),
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

    fn test_with_different_quantities() {
        let spread = BearPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(105.0),
            pos!(95.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(2.0), // quantity = 2
            pos!(4.0),
            pos!(2.0),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );

        let max_profit = spread.max_profit().unwrap();
        let max_loss = spread.max_loss().unwrap();

        // Max Profit = (Width * Quantity) - Net Premium
        assert_eq!(max_profit, pos!(16.0)); // (10 * 2) - (4 - 2)
        // Max Loss = Net Premium
        assert_eq!(max_loss, pos!(4.0));
    }
}

#[cfg(test)]
mod tests_bear_put_spread_validation {
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
            create_valid_position(Side::Long, pos!(105.0), ExpirationDate::Days(pos!(30.0)));
        invalid_long.option.quantity = Positive::ZERO;

        let spread = BearPutSpread {
            name: "Test Bear Put Spread".to_string(),
            kind: StrategyType::BearPutSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_put: invalid_long,
            short_put: create_valid_position(
                Side::Short,
                pos!(95.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
        };

        assert!(!spread.validate());
    }

    #[test]

    fn test_invalid_short_put() {
        let mut invalid_short =
            create_valid_position(Side::Short, pos!(95.0), ExpirationDate::Days(pos!(30.0)));
        invalid_short.option.quantity = Positive::ZERO;

        let spread = BearPutSpread {
            name: "Test Bear Put Spread".to_string(),
            kind: StrategyType::BearPutSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_put: create_valid_position(
                Side::Long,
                pos!(105.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
            short_put: invalid_short,
        };

        assert!(!spread.validate());
    }

    #[test]

    fn test_invalid_strike_prices() {
        let spread = BearPutSpread {
            name: "Test Bear Put Spread".to_string(),
            kind: StrategyType::BearPutSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_put: create_valid_position(
                Side::Long,
                pos!(95.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
            short_put: create_valid_position(
                Side::Short,
                pos!(105.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
        };

        assert!(!spread.validate());
    }

    #[test]

    fn test_equal_strike_prices() {
        let spread = BearPutSpread {
            name: "Test Bear Put Spread".to_string(),
            kind: StrategyType::BearPutSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_put: create_valid_position(
                Side::Long,
                pos!(100.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
            short_put: create_valid_position(
                Side::Short,
                pos!(100.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
        };

        assert!(!spread.validate());
    }

    #[test]

    fn test_valid_strike_prices() {
        let spread = BearPutSpread {
            name: "Test Bear Put Spread".to_string(),
            kind: StrategyType::BearPutSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_put: create_valid_position(
                Side::Long,
                pos!(105.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
            short_put: create_valid_position(
                Side::Short,
                pos!(95.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
        };

        assert!(spread.validate());
    }

    #[test]

    fn test_different_expiration_dates() {
        let spread = BearPutSpread {
            name: "Test Bear Put Spread".to_string(),
            kind: StrategyType::BearPutSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_put: create_valid_position(
                Side::Long,
                pos!(105.0),
                ExpirationDate::Days(pos!(30.0)),
            ),
            short_put: create_valid_position(
                Side::Short,
                pos!(95.0),
                ExpirationDate::Days(pos!(60.0)),
            ),
        };

        assert!(spread.validate());
    }
}

#[cfg(test)]
mod tests_bear_put_spread_optimization {
    use super::*;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    use crate::model::types::ExpirationDate;
    use crate::spos;

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(90.0), "2024-12-31".to_string(), None, None);

        // Add options with increasing strikes around the current price
        chain.add_option(
            pos!(85.0),       // strike
            None,             // call_bid
            None,             // call_ask
            spos!(8.0),       // put_bid
            spos!(8.2),       // put_ask
            spos!(0.2),       // implied_volatility
            Some(dec!(-0.8)), // delta
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0), // volume
            Some(50),     // open_interest
        );

        chain.add_option(
            pos!(90.0),
            None,
            None,
            spos!(6.0),
            spos!(6.2),
            spos!(0.2),
            Some(dec!(-0.7)),
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
            Some(dec!(-0.6)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(200.0),
            Some(100),
        );

        chain.add_option(
            pos!(100.0),
            None,
            None,
            spos!(2.5),
            spos!(2.7),
            spos!(0.2),
            Some(dec!(-0.5)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(250.0),
            Some(125),
        );

        chain.add_option(
            pos!(105.0),
            None,
            None,
            spos!(1.5),
            spos!(1.7),
            spos!(0.2),
            Some(dec!(-0.4)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(200.0),
            Some(100),
        );

        chain.add_option(
            pos!(110.0),
            None,
            None,
            spos!(0.8),
            spos!(1.0),
            spos!(0.2),
            Some(dec!(-0.3)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(150.0),
            Some(75),
        );

        chain.add_option(
            pos!(115.0),
            None,
            None,
            spos!(0.4),
            spos!(0.6),
            spos!(0.2),
            Some(dec!(-0.2)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0),
            Some(50),
        );

        chain
    }

    fn create_base_spread() -> BearPutSpread {
        BearPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(105.0),
            pos!(95.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(1.7), // premium_long_put
            pos!(4.0), // premium_short_put
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

        // The optimal ratio should choose strikes with maximum difference while minimizing cost
        assert!(spread.long_put.option.strike_price > spread.short_put.option.strike_price);
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

        // Area optimization should favor wider spreads with good probability of profit
        assert!(spread.long_put.option.strike_price > chain.underlying_price);
    }

    #[test]

    fn test_find_optimal_upper_side() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(&chain, FindOptimalSide::Upper, OptimizationCriteria::Ratio);

        // Both strikes should be above the underlying price
        assert!(spread.short_put.option.strike_price > chain.underlying_price);
        assert!(spread.long_put.option.strike_price > chain.underlying_price);
    }

    #[test]

    fn test_find_optimal_range() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(
            &chain,
            FindOptimalSide::Range(pos!(95.0), pos!(105.0)),
            OptimizationCriteria::Ratio,
        );

        // Strikes should be within the specified range
        assert!(spread.short_put.option.strike_price >= pos!(95.0));
        assert!(spread.long_put.option.strike_price <= pos!(105.0));
    }

    #[test]

    fn test_create_strategy() {
        let spread = create_base_spread();
        let chain = create_test_chain();

        let long_option = chain
            .options
            .iter()
            .find(|o| o.strike_price == pos!(95.0))
            .unwrap();
        let short_option = chain
            .options
            .iter()
            .find(|o| o.strike_price == pos!(105.0))
            .unwrap();

        let legs = StrategyLegs::TwoLegs {
            first: long_option,
            second: short_option,
        };
        let new_strategy = spread.create_strategy(&chain, &legs);

        assert!(new_strategy.validate());
        assert_eq!(new_strategy.long_put.option.strike_price, pos!(105.0));
        assert_eq!(new_strategy.short_put.option.strike_price, pos!(95.0));
        assert_eq!(new_strategy.long_put.option.implied_volatility, 0.2);

        // Verify premiums are set correctly
        assert_eq!(new_strategy.long_put.premium, 1.7); // put_ask from long option
        assert_eq!(new_strategy.short_put.premium, 4.0); // put_bid from short option
    }

    #[test]

    fn test_optimization_with_invalid_options() {
        let mut spread = create_base_spread();
        let mut chain = create_test_chain();

        // Add some invalid options to the chain
        chain.add_option(
            pos!(120.0),
            None,
            None,
            None, // Invalid: no put_bid
            None, // Invalid: no put_ask
            spos!(0.2),
            Some(dec!(-0.1)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(50.0),
            Some(25),
        );

        spread.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);

        // Should still find a valid optimization ignoring invalid options
        assert!(spread.validate());
        assert!(spread.max_profit().is_ok());
    }

    #[test]

    fn test_optimization_with_different_quantities() {
        let mut spread = BearPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(105.0),
            pos!(95.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(2.0), // quantity = 2
            pos!(1.7),
            pos!(4.0),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );

        let chain = create_test_chain();

        spread.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);

        assert!(spread.validate());
        assert_eq!(spread.long_put.option.quantity, pos!(2.0));
        assert_eq!(spread.short_put.option.quantity, pos!(2.0));
    }
}

#[cfg(test)]
mod tests_bear_put_spread_optimizable {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::spos;
    use crate::strategies::utils::FindOptimalSide;
    use crate::utils::setup_logger;
    use rust_decimal_macros::dec;

    fn create_mock_option_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-03-15".to_string(), None, None);

        chain.add_option(
            pos!(95.0),       // strike
            spos!(0.5),       // call_bid
            spos!(0.7),       // call_ask
            spos!(2.0),       // put_bid -
            spos!(2.2),       // put_ask
            spos!(0.2),       // implied_vol
            Some(dec!(-0.3)), // delta
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0), // volume
            Some(50),     // open_interest
        );

        // Strike ATM (100)
        chain.add_option(
            pos!(100.0),
            spos!(2.8),
            spos!(3.0),
            spos!(4.8),
            spos!(5.0),
            spos!(0.2),
            Some(dec!(-0.5)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(200.0),
            Some(100),
        );

        chain.add_option(
            pos!(105.0),
            spos!(5.8),
            spos!(6.0),
            spos!(8.8), // put_bid
            spos!(9.0), // put_ask
            spos!(0.2),
            Some(dec!(-0.7)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(150.0),
            Some(75),
        );

        chain
    }

    fn create_test_bear_put_spread() -> BearPutSpread {
        BearPutSpread::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(105.0), // long strike (higher)
            pos!(95.0),  // short strike (lower)
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(2.0), // premium short put
            pos!(8.8), // premium long put
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        )
    }

    #[test]

    fn test_filter_valid_combinations() {
        setup_logger();
        let spread = create_test_bear_put_spread();
        let chain = create_mock_option_chain();

        info!("Chain options:");
        for option in chain.options.iter() {
            info!(
                "Strike: {}, Put bid: {:?}, Put ask: {:?}",
                option.strike_price, option.put_bid, option.put_ask
            );
        }

        let combinations: Vec<_> = spread
            .filter_combinations(&chain, FindOptimalSide::All)
            .collect();

        info!("Found {} combinations", combinations.len());

        assert!(
            !combinations.is_empty(),
            "Should find at least one valid combination"
        );

        for combination in combinations {
            match combination {
                OptionDataGroup::Two(short, long) => {
                    // Short strike should be lower than long strike
                    assert!(short.strike_price < long.strike_price);

                    // Both options should have valid put prices
                    assert!(
                        short.put_bid.is_some(),
                        "Short put bid is missing for strike {}",
                        short.strike_price
                    );
                    assert!(
                        long.put_ask.is_some(),
                        "Long put ask is missing for strike {}",
                        long.strike_price
                    );

                    // Both options should have valid implied volatility
                    assert!(short.implied_volatility.is_some());
                    assert!(long.implied_volatility.is_some());

                    info!(
                        "Valid combination - Short strike: {}, Long strike: {}",
                        short.strike_price, long.strike_price
                    );
                }
                _ => panic!("Expected Two-leg combination"),
            }
        }
    }

    #[test]

    fn test_filter_invalid_prices() {
        let mut chain = create_mock_option_chain();
        // Add an option with invalid put prices
        chain.add_option(
            pos!(97.0),
            spos!(1.0),
            spos!(1.2),
            None, // Invalid: no put_bid
            None, // Invalid: no put_ask
            spos!(0.2),
            Some(dec!(-0.4)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(50.0),
            Some(25),
        );

        let spread = create_test_bear_put_spread();
        let combinations: Vec<_> = spread
            .filter_combinations(&chain, FindOptimalSide::Lower)
            .collect();

        for combination in combinations {
            match combination {
                OptionDataGroup::Two(short, long) => {
                    // Verify that options with invalid prices are filtered out
                    assert!(short.put_bid.unwrap() > Positive::ZERO);
                    assert!(long.put_ask.unwrap() > Positive::ZERO);
                }
                _ => panic!("Expected Two-leg combination"),
            }
        }
    }

    #[test]

    fn test_filter_with_different_optimal_sides() {
        let spread = create_test_bear_put_spread();
        let chain = create_mock_option_chain();

        // Test Lower side (typical for bear put spread)
        let lower_combinations: Vec<_> = spread
            .filter_combinations(&chain, FindOptimalSide::Lower)
            .collect();
        assert!(!lower_combinations.is_empty());

        // Test Upper side (should have fewer or no valid combinations)
        let upper_combinations: Vec<_> = spread
            .filter_combinations(&chain, FindOptimalSide::Upper)
            .collect();

        // Test All sides
        let all_combinations: Vec<_> = spread
            .filter_combinations(&chain, FindOptimalSide::All)
            .collect();

        assert!(all_combinations.len() >= lower_combinations.len());
        assert!(all_combinations.len() >= upper_combinations.len());
    }

    #[test]

    fn test_filter_empty_chain() {
        let spread = create_test_bear_put_spread();
        let empty_chain =
            OptionChain::new("TEST", pos!(100.0), "2024-03-15".to_string(), None, None);

        let combinations: Vec<_> = spread
            .filter_combinations(&empty_chain, FindOptimalSide::Lower)
            .collect();

        assert!(combinations.is_empty());
    }

    #[test]

    fn test_filter_strategy_constraints() {
        let spread = create_test_bear_put_spread();
        let mut chain = create_mock_option_chain();

        // Add an option that would create an invalid strategy (strikes too close)
        chain.add_option(
            pos!(99.9),
            spos!(1.0),
            spos!(1.2),
            spos!(3.0),
            spos!(3.2),
            spos!(0.2),
            Some(dec!(-0.5)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(50.0),
            Some(25),
        );

        let combinations: Vec<_> = spread
            .filter_combinations(&chain, FindOptimalSide::Lower)
            .collect();

        for combination in combinations {
            match combination {
                OptionDataGroup::Two(short, long) => {
                    // Verify that the strikes have enough width between them
                    assert!((long.strike_price - short.strike_price).to_f64() >= 1.0);
                }
                _ => panic!("Expected Two-leg combination"),
            }
        }
    }
}

#[cfg(test)]
mod tests_bear_put_spread_profit {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn create_test_spread() -> BearPutSpread {
        BearPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),                      // underlying_price
            pos!(105.0),                      // long_strike
            pos!(95.0),                       // short_strike
            ExpirationDate::Days(pos!(30.0)), // expiration
            pos!(0.2),                        // implied_volatility
            dec!(0.05),                       // risk_free_rate
            Positive::ZERO,                   // dividend_yield
            pos!(1.0),                        // quantity
            pos!(4.0),                        // premium_long_put
            pos!(2.0),                        // premium_short_put
            Positive::ZERO,                   // open_fee_long_put
            Positive::ZERO,                   // close_fee_long_put
            Positive::ZERO,                   // open_fee_short_put
            Positive::ZERO,                   // close_fee_short_put
        )
    }

    #[test]

    fn test_profit_at_max_profit() {
        let spread = create_test_spread();
        let price = pos!(90.0);

        // Max Profit = Width (105 - 95 = 10) - Net Premium (4 - 2 = 2) = 8
        assert_eq!(
            spread.calculate_profit_at(price).unwrap().to_f64().unwrap(),
            8.0
        );
    }

    #[test]

    fn test_profit_at_max_loss() {
        let spread = create_test_spread();
        let price = pos!(110.0);

        // Max Loss = Net Premium = 4 - 2 = 2
        assert_eq!(
            spread.calculate_profit_at(price).unwrap().to_f64().unwrap(),
            -2.0
        );
    }

    #[test]

    fn test_profit_at_short_strike() {
        let spread = create_test_spread();
        let price = pos!(95.0);

        // Profit at short strike = Max Profit = 8
        assert_eq!(
            spread.calculate_profit_at(price).unwrap().to_f64().unwrap(),
            8.0
        );
    }

    #[test]

    fn test_profit_at_long_strike() {
        let spread = create_test_spread();
        let price = pos!(105.0);

        // Loss at long strike = Max Loss = -2
        assert_eq!(
            spread.calculate_profit_at(price).unwrap().to_f64().unwrap(),
            -2.0
        );
    }

    #[test]

    fn test_profit_at_get_break_even_points() {
        let spread = create_test_spread();
        let price = pos!(103.0); // Break even = long strike - net premium = 105 - 2

        assert!(
            spread
                .calculate_profit_at(price)
                .unwrap()
                .to_f64()
                .unwrap()
                .abs()
                < 0.01
        );
    }

    #[test]

    fn test_profit_between_strikes() {
        let spread = create_test_spread();
        let price = pos!(100.0);

        let profit = spread.calculate_profit_at(price).unwrap().to_f64().unwrap();

        assert!(profit > -2.0);
        assert!(profit < 8.0);
    }

    #[test]

    fn test_profit_with_different_quantities() {
        let spread = BearPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(105.0),
            pos!(95.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(2.0), // quantity = 2
            pos!(4.0),
            pos!(2.0),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );

        let max_profit_price = pos!(90.0);
        let max_loss_price = pos!(110.0);

        // Con quantity = 2:
        // Max Profit = 2 * (Width - Net Premium) = 2 * (10 - 2) = 16
        assert_eq!(
            spread.calculate_profit_at(max_profit_price).unwrap(),
            dec!(16.0)
        );

        // Max Loss = 2 * Net Premium = 2 * 2 = 4
        assert_eq!(
            spread.calculate_profit_at(max_loss_price).unwrap(),
            dec!(-4.0)
        );
    }

    #[test]

    fn test_profit_with_fees() {
        let spread = BearPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(105.0),
            pos!(95.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(4.0),
            pos!(2.0),
            pos!(0.5), // open_fee_long_put
            pos!(0.5), // close_fee_long_put
            pos!(0.5), // open_fee_short_put
            pos!(0.5), // close_fee_short_put
        );

        let max_profit_price = pos!(90.0);

        // Max Profit = Width (105 - 95 = 10) - Net Premium (4 - 2 = 2) - Total Fees (0.5 * 4)
        // = 10 - 2 - 2 = 6
        assert_eq!(
            spread.calculate_profit_at(max_profit_price).unwrap(),
            dec!(6.0)
        );
    }

    #[test]

    fn test_profit_far_below_strikes() {
        let spread = create_test_spread();
        let price = pos!(80.0);

        // El profit debería ser igual al max profit
        assert_eq!(spread.calculate_profit_at(price).unwrap(), dec!(8.0));
    }

    #[test]

    fn test_profit_far_above_strikes() {
        let spread = create_test_spread();
        let price = pos!(120.0);

        assert_eq!(spread.calculate_profit_at(price).unwrap(), dec!(-2.0));
    }
}

#[cfg(test)]
mod tests_bear_put_spread_probability {
    use super::*;
    use crate::strategies::probabilities::utils::PriceTrend;
    use rust_decimal_macros::dec;

    fn create_test_spread() -> BearPutSpread {
        BearPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),                      // underlying_price
            pos!(105.0),                      // long_strike
            pos!(95.0),                       // short_strike
            ExpirationDate::Days(pos!(30.0)), // expiration
            pos!(0.2),                        // implied_volatility
            dec!(0.05),                       // risk_free_rate
            Positive::ZERO,                   // dividend_yield
            pos!(1.0),                        // quantity
            pos!(4.0),                        // premium_long_put
            pos!(2.0),                        // premium_short_put
            Positive::ZERO,                   // open_fee_long_put
            Positive::ZERO,                   // close_fee_long_put
            Positive::ZERO,                   // open_fee_short_put
            Positive::ZERO,                   // close_fee_short_put
        )
    }

    #[test]

    fn test_get_expiration() {
        let spread = create_test_spread();
        let result = spread.get_expiration();
        assert!(result.is_ok());
        match result.unwrap() {
            ExpirationDate::Days(days) => assert_eq!(days, 30.0),
            _ => panic!("Expected ExpirationDate::Days"),
        }
    }

    #[test]

    fn test_get_risk_free_rate() {
        let spread = create_test_spread();
        assert_eq!(spread.get_risk_free_rate(), Some(dec!(0.05)));
    }

    #[test]

    fn test_get_profit_ranges() {
        let spread = create_test_spread();
        let result = spread.get_profit_ranges();
        assert!(result.is_ok());

        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1);

        let range = &ranges[0];
        assert!(range.lower_bound.is_some());
        assert!(range.upper_bound.is_some());
        assert_eq!(range.lower_bound.unwrap(), pos!(95.0)); // short strike
        assert!(range.probability > Positive::ZERO);
    }

    #[test]

    fn test_get_loss_ranges() {
        let spread = create_test_spread();
        let result = spread.get_loss_ranges();
        assert!(result.is_ok());

        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1);

        let range = &ranges[0];
        assert!(range.lower_bound.is_some());
        assert!(range.upper_bound.is_none());
        assert!(range.probability > Positive::ZERO);
    }

    #[test]

    fn test_probability_of_profit() {
        let spread = create_test_spread();
        let result = spread.probability_of_profit(None, None);
        assert!(result.is_ok());

        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]

    fn test_probability_with_volatility_adjustment() {
        let spread = create_test_spread();
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
        let spread = create_test_spread();
        let trend = Some(PriceTrend {
            drift_rate: -0.1, // Negative drift for bearish trend
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
        let spread = create_test_spread();
        let result = spread.analyze_probabilities(None, None);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert!(analysis.probability_of_profit > Positive::ZERO);
        assert!(analysis.probability_of_max_profit >= Positive::ZERO);
        assert!(analysis.probability_of_max_loss >= Positive::ZERO);
        assert!(analysis.expected_value != Positive::ZERO);
        assert!(!analysis.break_even_points.is_empty());
        assert!(analysis.risk_reward_ratio > Positive::ZERO);
    }

    #[test]

    fn test_calculate_extreme_probabilities() {
        let spread = create_test_spread();
        let result = spread.calculate_extreme_probabilities(None, None);
        assert!(result.is_ok());

        let (max_profit_prob, max_loss_prob) = result.unwrap();
        assert!(max_profit_prob >= Positive::ZERO);
        assert!(max_loss_prob >= Positive::ZERO);
        assert!(max_profit_prob + max_loss_prob <= pos!(1.0));
    }
}

#[cfg(test)]
mod tests_bear_put_spread_graph {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn create_test_spread() -> BearPutSpread {
        BearPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),                      // underlying_price
            pos!(105.0),                      // long_strike
            pos!(95.0),                       // short_strike
            ExpirationDate::Days(pos!(30.0)), // expiration
            pos!(0.2),                        // implied_volatility
            dec!(0.05),                       // risk_free_rate
            Positive::ZERO,                   // dividend_yield
            pos!(1.0),                        // quantity
            pos!(4.0),                        // premium_long_put
            pos!(2.0),                        // premium_short_put
            Positive::ZERO,                   // fees
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        )
    }

    #[test]

    fn test_title_format() {
        let spread = create_test_spread();
        let title = spread.title();

        assert!(title.contains("Bear Put Spread Strategy"));
        assert!(title.contains("Long"));
        assert!(title.contains("Short"));
        assert!(title.contains("TEST")); // symbol
        assert!(title.contains("105")); // long strike
        assert!(title.contains("95")); // short strike
    }

    #[test]

    fn test_get_vertical_lines() {
        let spread = create_test_spread();
        let lines = spread.get_vertical_lines();

        assert_eq!(lines.len(), 1);

        let line = &lines[0];
        assert_eq!(line.x_coordinate, 100.0);
        assert_eq!(line.y_range, (f64::NEG_INFINITY, f64::INFINITY));
        assert!(line.label.contains("Current Price"));
        assert!(line.label.contains("100.00"));
        assert_eq!(line.label_offset, (4.0, 0.0));
        assert_eq!(line.line_color, ORANGE);
        assert_eq!(line.label_color, ORANGE);
        assert_eq!(line.font_size, 18);
    }

    #[test]

    fn test_get_points() {
        let spread = create_test_spread();
        let points = spread.get_points();

        assert_eq!(points.len(), 4); // Break even, max profit, max loss, current price

        // Break even point
        let break_even = &points[0];
        assert_eq!(break_even.coordinates.1, 0.0);
        assert!(break_even.label.contains("Break Even"));
        assert_eq!(break_even.point_color, DARK_BLUE);
        assert_eq!(break_even.label_color, DARK_BLUE);
        assert_eq!(break_even.point_size, 5);
        assert_eq!(break_even.font_size, 18);

        // Max profit point at short strike
        let max_profit = &points[1];
        assert_eq!(max_profit.coordinates.0, 95.0);
        assert!(max_profit.label.contains("Max Profit"));
        assert_eq!(max_profit.point_color, DARK_GREEN);
        assert_eq!(max_profit.label_color, DARK_GREEN);

        // Max loss point at long strike
        let max_loss = &points[2];
        assert_eq!(max_loss.coordinates.0, 105.0);
        assert!(max_loss.label.contains("Max Loss"));
        assert_eq!(max_loss.point_color, RED);
        assert_eq!(max_loss.label_color, RED);
    }

    #[test]

    fn test_points_coordinates() {
        let spread = create_test_spread();
        let points = spread.get_points();

        // Break even point
        assert_eq!(points[0].coordinates.1, 0.0);

        // Maximum profit point at short strike
        assert_eq!(points[1].coordinates.0, 95.0);
        assert_eq!(points[1].coordinates.1, 8.0); // Width (10) - Net Premium (2.0)

        // Maximum loss point at long strike
        assert_eq!(points[2].coordinates.0, 105.0);
        assert_eq!(points[2].coordinates.1, -2.0); // -Net Premium

        // Current price point
        assert_eq!(points[3].coordinates.0, 100.0);
        let current_profit = spread
            .calculate_profit_at(pos!(100.0))
            .unwrap()
            .to_f64()
            .unwrap();
        assert_eq!(points[3].coordinates.1, current_profit);
    }

    #[test]

    fn test_point_labels() {
        let spread = create_test_spread();
        let points = spread.get_points();

        assert_eq!(points.len(), 4);
        assert!(points[0].label.contains("Break Even"));
        assert!(points[1].label.contains("Max Profit"));
        assert!(points[2].label.contains("Max Loss"));
        assert!(points[3].label.contains("3.00"));
    }

    #[test]

    fn test_points_style() {
        let spread = create_test_spread();
        let points = spread.get_points();

        for point in points.iter() {
            assert_eq!(point.point_size, 5);
            assert_eq!(point.font_size, 18);
            assert!(matches!(point.point_color, DARK_BLUE | DARK_GREEN | RED));
            assert_eq!(point.point_color, point.label_color);
        }
    }

    #[test]

    fn test_graph_with_different_quantities() {
        let spread = BearPutSpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(105.0),
            pos!(95.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(2.0), // quantity = 2
            pos!(4.0),
            pos!(2.0),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );

        let points = spread.get_points();
        let max_profit_point = &points[1];
        let max_loss_point = &points[2];

        assert_eq!(max_profit_point.coordinates.1, 16.0); // 2 * (10.0 - 2.0)
        assert_eq!(max_loss_point.coordinates.1, -4.0); // 2 * -2.0
    }
}

#[cfg(test)]
mod tests_delta {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::strategies::bear_put_spread::BearPutSpread;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn get_strategy(long_strike: Positive, short_strike: Positive) -> BearPutSpread {
        let underlying_price = pos!(5810.5);
        BearPutSpread::new(
            "SP500".to_string(),
            underlying_price, // underlying_price
            long_strike,      // long_strike
            short_strike,     // short_strike
            ExpirationDate::Days(pos!(2.0)),
            pos!(0.18),     // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // long quantity
            pos!(85.04),    // premium_long
            pos!(29.85),    // premium_short
            pos!(0.78),     // open_fee_long
            pos!(0.78),     // open_fee_long
            pos!(0.73),     // close_fee_long
            pos!(0.73),     // close_fee_short
        )
    }

    #[test]

    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos!(5800.0), pos!(5820.0));
        let size = dec!(0.102723);
        let delta = pos!(0.23599920741322516);
        let k = pos!(5800.0);
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
        let strategy = get_strategy(pos!(5840.0), pos!(5820.0));
        let size = dec!(-0.0999046);
        let delta = pos!(0.18569835434604637);
        let k = pos!(5820.0);
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
        let strategy = get_strategy(pos!(5820.0), pos!(5820.0));

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
    use crate::strategies::bear_put_spread::BearPutSpread;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn get_strategy(long_strike: Positive, short_strike: Positive) -> BearPutSpread {
        let underlying_price = pos!(5781.88);
        BearPutSpread::new(
            "SP500".to_string(),
            underlying_price, // underlying_price
            long_strike,      // long_strike
            short_strike,     // short_strike
            ExpirationDate::Days(pos!(2.0)),
            pos!(0.18),     // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(2.0),      // long quantity
            pos!(85.04),    // premium_long
            pos!(29.85),    // premium_short
            pos!(0.78),     // open_fee_long
            pos!(0.78),     // open_fee_long
            pos!(0.73),     // close_fee_long
            pos!(0.73),     // close_fee_short
        )
    }

    #[test]

    fn create_test_reducing_adjustments() {
        let strike = pos!(5800.0);
        let strategy = get_strategy(strike, pos!(5820.0));
        let size = dec!(0.1942);
        let delta = pos!(0.3336989562679224);
        let k = pos!(5800.0);
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
            strategy.delta_neutrality().unwrap().net_delta + delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]

    fn create_test_increasing_adjustments() {
        let strike = pos!(5820.0);
        let strategy = get_strategy(pos!(5840.0), strike);
        let size = dec!(-0.1718);
        let delta = pos!(0.2529151481237256);
        let k = pos!(5820.0);
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
        let strategy = get_strategy(pos!(5820.0), pos!(5820.0));

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

    fn create_test_short_bear_put_spread() -> BearPutSpread {
        BearPutSpread::new(
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
    fn test_short_bear_put_spread_get_position() {
        let mut bear_put_spread = create_test_short_bear_put_spread();

        // Test getting short put position
        let put_position =
            bear_put_spread.get_position(&OptionStyle::Put, &Side::Long, &pos!(5850.0));
        assert!(put_position.is_ok());
        let positions = put_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(5850.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Put);
        assert_eq!(positions[0].option.side, Side::Long);

        // Test getting short put position
        let put_position =
            bear_put_spread.get_position(&OptionStyle::Put, &Side::Short, &pos!(5720.0));
        assert!(put_position.is_ok());
        let positions = put_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(5720.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Put);
        assert_eq!(positions[0].option.side, Side::Short);

        // Test getting non-existent position
        let invalid_position =
            bear_put_spread.get_position(&OptionStyle::Call, &Side::Short, &pos!(5821.0));
        assert!(invalid_position.is_err());
        match invalid_position {
            Err(PositionError::ValidationError(
                PositionValidationErrorKind::IncompatibleSide {
                    position_side: _,
                    reason,
                },
            )) => {
                assert_eq!(reason, "Call is not valid for BearPutSpread");
            }
            _ => {
                error!("Unexpected error: {:?}", invalid_position);
                panic!()
            }
        }
    }

    #[test]
    fn test_short_bear_put_spread_modify_position() {
        let mut bear_put_spread = create_test_short_bear_put_spread();

        // Modify short put position
        let mut modified_put = bear_put_spread.short_put.clone();
        modified_put.option.quantity = pos!(2.0);
        let result = bear_put_spread.modify_position(&modified_put);
        assert!(result.is_ok());
        assert_eq!(bear_put_spread.short_put.option.quantity, pos!(2.0));

        // Modify short put position
        let mut modified_put = bear_put_spread.long_put.clone();
        modified_put.option.quantity = pos!(2.0);
        let result = bear_put_spread.modify_position(&modified_put);
        assert!(result.is_ok());
        assert_eq!(bear_put_spread.long_put.option.quantity, pos!(2.0));

        // Test modifying with invalid position
        let mut invalid_position = bear_put_spread.short_put.clone();
        invalid_position.option.strike_price = pos!(95.0);
        let result = bear_put_spread.modify_position(&invalid_position);
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
    fn create_test_strategy() -> BearPutSpread {
        BearPutSpread::new(
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
                assert_eq!(reason, "Call is not valid for BearPutSpread");
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
mod tests_bear_put_spread_constructor {
    use super::*;
    use crate::model::utils::create_sample_position;
    use crate::pos;

    #[test]
    fn test_get_strategy_valid() {
        let options = vec![
            create_sample_position(
                OptionStyle::Put,
                Side::Long,
                pos!(90.0),
                pos!(1.0),
                pos!(95.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Put,
                Side::Short,
                pos!(90.0),
                pos!(1.0),
                pos!(105.0),
                pos!(0.2),
            ),
        ];

        let result = BearPutSpread::get_strategy(&options);
        assert!(result.is_ok());

        let strategy = result.unwrap();
        assert_eq!(strategy.long_put.option.strike_price, pos!(95.0));
        assert_eq!(strategy.short_put.option.strike_price, pos!(105.0));
    }

    #[test]
    fn test_get_strategy_wrong_number_of_options() {
        let options = vec![create_sample_position(
            OptionStyle::Put,
            Side::Long,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        )];

        let result = BearPutSpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Bear Put Spread get_strategy" && reason == "Must have exactly 2 options"
        ));
    }

    #[test]
    fn test_get_strategy_wrong_option_style() {
        let mut option1 = create_sample_position(
            OptionStyle::Put,
            Side::Long,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );
        option1.option.option_style = OptionStyle::Call;
        let option2 = create_sample_position(
            OptionStyle::Put,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(105.0),
            pos!(0.2),
        );

        let options = vec![option1, option2];
        let result = BearPutSpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Bear Put Spread get_strategy" && reason == "Options must be puts"
        ));
    }

    #[test]
    fn test_get_strategy_wrong_sides() {
        let options = vec![
            create_sample_position(
                OptionStyle::Put,
                Side::Short,
                pos!(90.0),
                pos!(1.0),
                pos!(95.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Put,
                Side::Long,
                pos!(90.0),
                pos!(1.0),
                pos!(105.0),
                pos!(0.2),
            ),
        ];
        let result = BearPutSpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Bear Put Spread get_strategy"
                && reason == "Bear Put Spread requires a long lower strike put and a short higher strike put"
        ));
    }

    #[test]
    fn test_get_strategy_different_expiration_dates() {
        let mut option1 = create_sample_position(
            OptionStyle::Put,
            Side::Long,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );
        let mut option2 = create_sample_position(
            OptionStyle::Put,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(105.0),
            pos!(0.2),
        );

        option1.option.expiration_date = ExpirationDate::Days(pos!(30.0));
        option2.option.expiration_date = ExpirationDate::Days(pos!(60.0));

        let options = vec![option1, option2];
        let result = BearPutSpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Bear Put Spread get_strategy" && reason == "Options must have the same expiration date"
        ));
    }
}

#[cfg(test)]
mod tests_bear_put_spread_pnl {
    use super::*;
    use crate::model::utils::create_sample_position;
    use crate::{assert_decimal_eq, pos};
    use rust_decimal_macros::dec;

    /// Helper function to create a standard Bear Put Spread for testing
    fn create_test_bear_put_spread() -> Result<BearPutSpread, StrategyError> {
        // Create short put with higher strike
        let short_put = create_sample_position(
            OptionStyle::Put,
            Side::Short,
            pos!(100.0), // Underlying price
            pos!(1.0),   // Quantity
            pos!(100.0), // Strike price (ATM)
            pos!(0.2),   // Implied volatility
        );

        // Create long put with lower strike
        let long_put = create_sample_position(
            OptionStyle::Put,
            Side::Long,
            pos!(100.0), // Same underlying price
            pos!(1.0),   // Quantity
            pos!(95.0),  // Lower strike price
            pos!(0.2),   // Implied volatility
        );

        BearPutSpread::get_strategy(&vec![short_put, long_put])
    }

    /// Test PnL calculation when underlying price is below both strikes
    #[test]
    fn test_calculate_pnl_below_strikes() {
        let spread = create_test_bear_put_spread().unwrap();
        let market_price = pos!(90.0); // Below both strikes
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.2);

        let result = spread.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Both options ITM, but loss mitigated by long put
        assert!(pnl.unrealized.unwrap() < dec!(0.0)); // Some loss
        assert!(pnl.unrealized.unwrap() > dec!(-5.0)); // But not max loss
    }

    /// Test PnL calculation when underlying price is between strikes
    #[test]
    fn test_calculate_pnl_between_strikes() {
        let spread = create_test_bear_put_spread().unwrap();
        let market_price = pos!(97.5); // Between strikes
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.1);

        let result = spread.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Short put OTM, long put close to ITM
        assert!(pnl.unrealized.unwrap() < dec!(0.0)); // Some loss
        assert!(pnl.unrealized.unwrap() > dec!(-5.0)); // But not max loss
    }

    /// Test PnL calculation when underlying price is above both strikes
    #[test]
    fn test_calculate_pnl_above_strikes() {
        let spread = create_test_bear_put_spread().unwrap();
        let market_price = pos!(110.0); // Above both strikes
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.2);

        let result = spread.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Both options OTM, should be close to max profit
        assert!(pnl.unrealized.unwrap() > dec!(-2.0)); // Near max profit
        assert!(pnl.unrealized.unwrap() < dec!(2.0));
    }

    /// Test PnL calculation at expiration when underlying is below both strikes (max loss)
    #[test]
    fn test_calculate_pnl_at_expiration_max_loss() {
        let spread = create_test_bear_put_spread().unwrap();
        let underlying_price = pos!(90.0); // Well below both strikes

        let result = spread.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // Max loss = spread width (5.0) - net premium received (0.0) + fees (2.0)
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(-7.0), dec!(1e-6));
        assert_eq!(pnl.initial_income, pos!(5.0));
        assert_eq!(pnl.initial_costs, pos!(7.0));
    }

    /// Test PnL calculation at expiration when underlying is at the higher strike
    #[test]
    fn test_calculate_pnl_at_expiration_max_profit() {
        let spread = create_test_bear_put_spread().unwrap();
        let underlying_price = pos!(110.0); // Above both strikes

        let result = spread.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // At expiration, both options expire worthless
        // Max profit is the net premium received minus fees
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(-2.0), dec!(1e-6)); // Premium received - costs
        assert_eq!(pnl.initial_income, pos!(5.0));
        assert_eq!(pnl.initial_costs, pos!(7.0));
    }

    /// Test PnL calculation at expiration when underlying is between strikes
    #[test]
    fn test_calculate_pnl_at_expiration_between_strikes() {
        let spread = create_test_bear_put_spread().unwrap();
        let underlying_price = pos!(97.5); // Between strikes

        let result = spread.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // Loss should be: (100.0 - 97.5) = 2.5 intrinsic value of short put
        // Plus costs (7.0) minus income (5.0)
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(-4.5), dec!(1e-6));
    }

    /// Test PnL calculation with higher volatility
    #[test]
    fn test_calculate_pnl_with_higher_volatility() {
        let spread = create_test_bear_put_spread().unwrap();
        let market_price = pos!(100.0);
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.4); // Higher volatility

        let result = spread.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // With higher volatility, both options are worth more
        // Net effect should be slightly negative as short gamma position
        assert!(pnl.unrealized.unwrap() < dec!(0.0));
        // But still capped by the spread width
        assert!(pnl.unrealized.unwrap() > dec!(-5.0));
    }

    /// Test PnL calculation at expiration at the short strike
    #[test]
    fn test_calculate_pnl_at_expiration_at_short_strike() {
        let spread = create_test_bear_put_spread().unwrap();
        let underlying_price = pos!(100.0); // At short strike

        let result = spread.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // At the short strike, short put is ATM
        // Loss should be just the costs minus income
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(-2.0), dec!(1e-6));
    }
}
