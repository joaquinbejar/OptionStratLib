use super::base::{
    BreakEvenable, Optimizable, Positionable, Strategable, StrategyBasics, StrategyType, Validable,
};
use crate::{
    ExpirationDate, Options,
    chains::{StrategyLegs, chain::OptionChain, utils::OptionDataGroup},
    error::{
        GreeksError, OperationErrorKind, PricingError,
        position::{PositionError, PositionValidationErrorKind},
        probability::ProbabilityError,
        strategies::{ProfitLossErrorKind, StrategyError},
    },
    greeks::Greeks,
    model::{
        ProfitLossRange,
        position::Position,
        types::{OptionBasicType, OptionStyle, OptionType, Side},
        utils::mean_and_std,
    },
    pnl::{PnLCalculator, utils::PnL},
    pricing::payoff::Profit,
    strategies::{
        BasicAble, Strategies, StrategyConstructor,
        delta_neutral::DeltaNeutrality,
        probabilities::{core::ProbabilityAnalysis, utils::VolatilityAdjustment},
        utils::{FindOptimalSide, OptimizationCriteria},
    },
    test_strategy_traits,
};
use chrono::Utc;
use positive::{Positive, pos_or_panic};
use pretty_simple_display::{DebugPretty, DisplaySimple};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{debug, info};
use utoipa::ToSchema;

/// The default description for the Short Butterfly Spread strategy.
pub const SHORT_BUTTERFLY_DESCRIPTION: &str = "A short butterfly spread is created by selling one call at a lower strike price, \
    buying two calls at a middle strike price, and selling one call at a higher strike price, \
    all with the same expiration date. This strategy profits when the underlying price moves \
    significantly away from the middle strike price in either direction.";

/// Represents a Short Butterfly Spread options strategy.
///
/// A Short Butterfly Spread involves selling two options at the middle strike price
/// and buying one option each at the lower and upper strike prices. This strategy
/// profits when the underlying asset price moves significantly in either direction,
/// with maximum loss occurring when the price stays near the middle strike at expiration.
///
/// The strategy consists of:
/// - Long call at the lowest strike price
/// - Two short calls at the middle strike price (represented as `short_call_low` and `short_call_high`)
/// - Long call at the highest strike price
///
#[derive(Clone, DebugPretty, DisplaySimple, Serialize, Deserialize, ToSchema)]
pub struct ShortButterflySpread {
    /// The name of the strategy, typically including the underlying asset
    pub name: String,
    /// The type of strategy, should be StrategyType::ShortButterflySpread
    pub kind: StrategyType,
    /// A textual description of the strategy, including relevant details like strike prices
    pub description: String,
    /// The price points at which the strategy breaks even (typically two points)
    pub break_even_points: Vec<Positive>,
    /// The short call position at the lowest strike price
    pub long_call: Position,
    /// The first short call position at the middle strike price
    pub short_call_low: Position,
    /// The second short call position at the middle strike price
    pub short_call_high: Position,
}

impl ShortButterflySpread {
    /// Creates a new Short Butterfly Spread options strategy.
    ///
    /// A Short Butterfly Spread is created by selling one call at a lower strike price,
    /// buying two calls at a middle strike price, and selling one call at a higher strike price,
    /// all with the same expiration date. This strategy profits when the underlying price moves
    /// significantly away from the middle strike price in either direction.
    ///
    /// # Parameters
    ///
    /// * `underlying_symbol` - The ticker symbol of the underlying asset.
    /// * `underlying_price` - The current price of the underlying asset.
    /// * `low_strike` - The lower strike price for the short call.
    /// * `middle_strike` - The middle strike price for the short calls.
    /// * `high_strike` - The higher strike price for the short call.
    /// * `expiration` - The expiration date for all options in the strategy.
    /// * `implied_volatility` - The implied volatility used for pricing the options.
    /// * `risk_free_rate` - The risk-free interest rate as a decimal.
    /// * `dividend_yield` - The dividend yield of the underlying asset.
    /// * `quantity` - The quantity of contracts for each leg (except middle strike which uses 2x quantity).
    /// * `premium_low` - The premium for the lower strike short call.
    /// * `premium_middle` - The premium for the middle strike short calls.
    /// * `premium_high` - The premium for the higher strike short call.
    /// * `open_fee_short_call` - The fee to open the short call positions.
    /// * `close_fee_short_call` - The fee to close the short call positions.
    /// * `open_fee_short_call_low` - The fee to open the lower strike short call position.
    /// * `close_fee_short_call_low` - The fee to close the lower strike short call position.
    /// * `open_fee_short_call_high` - The fee to open the higher strike short call position.
    /// * `close_fee_short_call_high` - The fee to close the higher strike short call position.
    ///
    /// # Returns
    ///
    /// A fully initialized `ShortButterflySpread` strategy with calculated break-even points.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        low_strike: Positive,
        middle_strike: Positive,
        high_strike: Positive,
        expiration: ExpirationDate,
        implied_volatility: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        quantity: Positive,
        premium_low: Positive,
        premium_middle: Positive,
        premium_high: Positive,
        open_fee_long_call: Positive,
        close_fee_long_call: Positive,
        open_fee_short_call_low: Positive,
        close_fee_short_call_low: Positive,
        open_fee_short_call_high: Positive,
        close_fee_short_call_high: Positive,
    ) -> Self {
        let mut strategy = ShortButterflySpread {
            name: "Short Butterfly".to_string(),
            kind: StrategyType::ShortButterflySpread,
            description: SHORT_BUTTERFLY_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call_low: Position::default(),
            long_call: Position::default(),
            short_call_high: Position::default(),
        };

        // Create two short calls at middle strike
        let short_calls = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            middle_strike,
            expiration,
            implied_volatility,
            quantity * 2.0, // Double quantity for middle strike
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        strategy.long_call = Position::new(
            short_calls,
            premium_middle,
            Utc::now(),
            open_fee_long_call,
            close_fee_long_call,
            None,
            None,
        );

        // Create short call at lower strike
        let short_call_low = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            low_strike,
            expiration,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        strategy.short_call_low = Position::new(
            short_call_low,
            premium_low,
            Utc::now(),
            open_fee_short_call_low,
            close_fee_short_call_low,
            None,
            None,
        );

        // Create short call at higher strike
        let short_call_high = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol,
            high_strike,
            expiration,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        strategy.short_call_high = Position::new(
            short_call_high,
            premium_high,
            Utc::now(),
            open_fee_short_call_high,
            close_fee_short_call_high,
            None,
            None,
        );

        strategy.validate();

        strategy
            .update_break_even_points()
            .expect("Unable to update break even points");
        strategy
    }
}

impl StrategyConstructor for ShortButterflySpread {
    fn get_strategy(vec_positions: &[Position]) -> Result<Self, StrategyError> {
        // Short Butterfly Spread requires exactly 3 options
        if vec_positions.len() != 3 {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Short Butterfly Spread get_strategy".to_string(),
                    reason: "Must have exactly 3 options".to_string(),
                },
            ));
        }

        // Sort options by strike price
        let mut sorted_positions = vec_positions.to_vec();
        sorted_positions.sort_by(|a, b| {
            a.option
                .strike_price
                .partial_cmp(&b.option.strike_price)
                .unwrap()
        });

        let lower_strike_position = &sorted_positions[0];
        let middle_strike_position = &sorted_positions[1];
        let higher_strike_position = &sorted_positions[2];

        // Validate options are calls
        if lower_strike_position.option.option_style != OptionStyle::Call
            || middle_strike_position.option.option_style != OptionStyle::Call
            || higher_strike_position.option.option_style != OptionStyle::Call
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Short Butterfly Spread get_strategy".to_string(),
                    reason: "Options must be calls".to_string(),
                },
            ));
        }

        // Validate option configuration for Short Butterfly
        if lower_strike_position.option.side != Side::Short
            || middle_strike_position.option.side != Side::Long
            || higher_strike_position.option.side != Side::Short
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Short Butterfly Spread get_strategy".to_string(),
                    reason: "Short Butterfly requires short lower and higher strikes with a short middle strike".to_string(),
                },
            ));
        }

        // Validate strike symmetry
        let lower_strike = lower_strike_position.option.strike_price;
        let middle_strike = middle_strike_position.option.strike_price;
        let higher_strike = higher_strike_position.option.strike_price;

        if middle_strike - lower_strike != higher_strike - middle_strike {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Short Butterfly Spread get_strategy".to_string(),
                    reason: "Strikes must be symmetrical".to_string(),
                },
            ));
        }

        // Validate expiration dates match
        if vec_positions
            .iter()
            .any(|opt| opt.option.expiration_date != lower_strike_position.option.expiration_date)
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Short Butterfly Spread get_strategy".to_string(),
                    reason: "Options must have the same expiration date".to_string(),
                },
            ));
        }

        // Create strategy
        let strategy = ShortButterflySpread {
            name: "Short Butterfly Spread".to_string(),
            kind: StrategyType::ShortButterflySpread,
            description: SHORT_BUTTERFLY_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::new(
                middle_strike_position.option.clone(),
                middle_strike_position.premium,
                Utc::now(),
                middle_strike_position.open_fee,
                middle_strike_position.close_fee,
                middle_strike_position.epic.clone(),
                middle_strike_position.extra_fields.clone(),
            ),
            short_call_low: Position::new(
                lower_strike_position.option.clone(),
                lower_strike_position.premium,
                Utc::now(),
                lower_strike_position.open_fee,
                lower_strike_position.close_fee,
                lower_strike_position.epic.clone(),
                lower_strike_position.extra_fields.clone(),
            ),
            short_call_high: Position::new(
                higher_strike_position.option.clone(),
                higher_strike_position.premium,
                Utc::now(),
                higher_strike_position.open_fee,
                higher_strike_position.close_fee,
                higher_strike_position.epic.clone(),
                higher_strike_position.extra_fields.clone(),
            ),
        };

        Ok(strategy)
    }
}

impl BreakEvenable for ShortButterflySpread {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        let left_net_value = self.calculate_profit_at(&self.short_call_low.option.strike_price)?
            / self.short_call_low.option.quantity;

        let right_net_value = self
            .calculate_profit_at(&self.short_call_high.option.strike_price)?
            / self.short_call_high.option.quantity;

        if left_net_value >= Decimal::ZERO {
            self.break_even_points
                .push((self.short_call_low.option.strike_price + left_net_value).round_to(2));
        }

        if right_net_value >= Decimal::ZERO {
            self.break_even_points
                .push((self.short_call_high.option.strike_price - right_net_value).round_to(2));
        }

        self.break_even_points.sort();
        Ok(())
    }
}

impl Validable for ShortButterflySpread {
    fn validate(&self) -> bool {
        if !self.short_call_low.validate() {
            debug!("Short call (low strike) is invalid");
            return false;
        }
        if !self.long_call.validate() {
            debug!("Long calls (middle strike) are invalid");
            return false;
        }
        if !self.short_call_high.validate() {
            debug!("Short call (high strike) is invalid");
            return false;
        }

        if self.short_call_low.option.strike_price >= self.long_call.option.strike_price {
            debug!("Low strike must be lower than middle strike");
            return false;
        }
        if self.long_call.option.strike_price >= self.short_call_high.option.strike_price {
            debug!("Middle strike must be lower than high strike");
            return false;
        }

        if self.long_call.option.quantity != self.short_call_low.option.quantity * 2.0 {
            debug!("Middle strike quantity must be double the wing quantities");
            return false;
        }
        if self.short_call_low.option.quantity != self.short_call_high.option.quantity {
            debug!("Wing quantities must be equal");
            return false;
        }

        true
    }
}

impl Positionable for ShortButterflySpread {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match &position.option.side {
            Side::Short => {
                // short_calls should be inserted first
                if position.option.strike_price < self.long_call.option.strike_price {
                    self.short_call_low = position.clone();
                    Ok(())
                } else {
                    self.short_call_high = position.clone();
                    Ok(())
                }
            }
            Side::Long => {
                self.long_call = position.clone();
                Ok(())
            }
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![
            &self.short_call_low,
            &self.long_call,
            &self.short_call_high,
        ])
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
            (Side::Long, OptionStyle::Call, strike)
                if *strike == self.long_call.option.strike_price =>
            {
                Ok(vec![&mut self.long_call])
            }

            (_, OptionStyle::Put, _) => Err(PositionError::invalid_position_type(
                *side,
                "Put not found in positions".to_string(),
            )),
            (Side::Short, OptionStyle::Call, strike)
                if *strike == self.short_call_low.option.strike_price =>
            {
                Ok(vec![&mut self.short_call_low])
            }
            (Side::Short, OptionStyle::Call, strike)
                if *strike == self.short_call_high.option.strike_price =>
            {
                Ok(vec![&mut self.short_call_high])
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
            (Side::Long, OptionStyle::Call, strike)
                if *strike == self.long_call.option.strike_price =>
            {
                self.long_call = position.clone();
            }

            (_, OptionStyle::Put, _) => {
                return Err(PositionError::invalid_position_type(
                    position.option.side,
                    "Put not found in positions".to_string(),
                ));
            }
            (Side::Short, OptionStyle::Call, strike)
                if *strike == self.short_call_low.option.strike_price =>
            {
                self.short_call_low = position.clone();
            }
            (Side::Short, OptionStyle::Call, strike)
                if *strike == self.short_call_high.option.strike_price =>
            {
                self.short_call_high = position.clone();
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

impl Strategable for ShortButterflySpread {
    fn info(&self) -> Result<StrategyBasics, StrategyError> {
        Ok(StrategyBasics {
            name: self.name.clone(),
            kind: self.kind.clone(),
            description: self.description.clone(),
        })
    }
}

impl BasicAble for ShortButterflySpread {
    fn get_title(&self) -> String {
        let strategy_title = format!("{:?} Strategy: ", self.kind);
        let leg_titles: Vec<String> = [
            self.short_call_low.get_title(),
            self.long_call.get_title(),
            self.short_call_high.get_title(),
        ]
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
        let short_call_low = &self.short_call_low.option;
        let long_call = &self.long_call.option;
        let short_call_high = &self.short_call_high.option;

        hash_set.insert(OptionBasicType {
            option_style: &short_call_low.option_style,
            side: &short_call_low.side,
            strike_price: &short_call_low.strike_price,
            expiration_date: &short_call_low.expiration_date,
        });
        hash_set.insert(OptionBasicType {
            option_style: &long_call.option_style,
            side: &long_call.side,
            strike_price: &long_call.strike_price,
            expiration_date: &long_call.expiration_date,
        });
        hash_set.insert(OptionBasicType {
            option_style: &short_call_high.option_style,
            side: &short_call_high.side,
            strike_price: &short_call_high.strike_price,
            expiration_date: &short_call_high.expiration_date,
        });

        hash_set
    }
    fn get_implied_volatility(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let options = [
            (
                &self.short_call_low.option,
                &self.short_call_low.option.implied_volatility,
            ),
            (
                &self.long_call.option,
                &self.long_call.option.implied_volatility,
            ),
            (
                &self.short_call_high.option,
                &self.short_call_high.option.implied_volatility,
            ),
        ];

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
        let options = [
            (
                &self.short_call_low.option,
                &self.short_call_low.option.quantity,
            ),
            (&self.long_call.option, &self.long_call.option.quantity),
            (
                &self.short_call_high.option,
                &self.short_call_high.option.quantity,
            ),
        ];

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
        self.short_call_low.option.expiration_date = expiration_date;
        self.long_call.option.expiration_date = expiration_date;
        self.short_call_high.option.expiration_date = expiration_date;
        Ok(())
    }
    fn set_underlying_price(&mut self, price: &Positive) -> Result<(), StrategyError> {
        self.short_call_low.option.underlying_price = *price;
        self.short_call_low.premium = Positive::from(
            self.short_call_low
                .option
                .calculate_price_black_scholes()?
                .abs(),
        );
        self.long_call.option.underlying_price = *price;
        self.long_call.premium =
            Positive::from(self.long_call.option.calculate_price_black_scholes()?.abs());
        self.short_call_high.option.underlying_price = *price;
        self.short_call_high.premium = Positive::from(
            self.short_call_high
                .option
                .calculate_price_black_scholes()?
                .abs(),
        );
        Ok(())
    }
    fn set_implied_volatility(&mut self, volatility: &Positive) -> Result<(), StrategyError> {
        self.short_call_low.option.implied_volatility = *volatility;
        self.long_call.option.implied_volatility = *volatility;
        self.short_call_high.option.implied_volatility = *volatility;

        self.short_call_low.premium = Positive(
            self.short_call_low
                .option
                .calculate_price_black_scholes()?
                .abs(),
        );
        self.long_call.premium =
            Positive(self.long_call.option.calculate_price_black_scholes()?.abs());
        self.short_call_high.premium = Positive(
            self.short_call_high
                .option
                .calculate_price_black_scholes()?
                .abs(),
        );
        Ok(())
    }
}

impl Strategies for ShortButterflySpread {
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        let left_profit = self.calculate_profit_at(&self.short_call_low.option.strike_price)?;
        let right_profit = self.calculate_profit_at(&self.short_call_high.option.strike_price)?;
        let max_profit = left_profit.max(right_profit);
        if max_profit > Decimal::ZERO {
            Ok(max_profit.into())
        } else {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxProfitError {
                    reason: "Max profit is negative".to_string(),
                },
            ))
        }
    }
    fn get_max_loss(&self) -> Result<Positive, StrategyError> {
        let loss = self.calculate_profit_at(&self.long_call.option.strike_price)?;
        if loss > Decimal::ZERO {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxLossError {
                    reason: "Max loss is negative".to_string(),
                },
            ))
        } else {
            Ok(loss.abs().into())
        }
    }
    fn get_profit_area(&self) -> Result<Decimal, StrategyError> {
        let break_even_points = self.get_break_even_points()?;
        let left_profit = self.calculate_profit_at(&self.short_call_low.option.strike_price)?;
        let right_profit = self.calculate_profit_at(&self.short_call_high.option.strike_price)?;

        let result = if break_even_points.len() == 2 {
            left_profit + right_profit
        } else {
            left_profit.max(right_profit)
        };
        Ok(result)
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

impl Optimizable for ShortButterflySpread {
    type Strategy = ShortButterflySpread;

    fn filter_combinations<'a>(
        &'a self,
        option_chain: &'a OptionChain,
        side: FindOptimalSide,
    ) -> impl Iterator<Item = OptionDataGroup<'a>> {
        let underlying_price = self.get_underlying_price();
        let strategy = self.clone();
        option_chain
            .get_triple_iter()
            // Filter out invalid combinations based on FindOptimalSide
            .filter(move |(short_low, short, short_high)| {
                if side == FindOptimalSide::Center {
                    let atm_strike = match option_chain.atm_strike() {
                        Ok(atm_strike) => atm_strike,
                        _ => return false,
                    };
                    short_low.is_valid_optimal_side(underlying_price, &FindOptimalSide::Lower)
                        && short.is_valid_optimal_side(
                            underlying_price,
                            &FindOptimalSide::Range(*atm_strike, *atm_strike),
                        )
                        && short_high
                            .is_valid_optimal_side(underlying_price, &FindOptimalSide::Upper)
                } else {
                    short_low.is_valid_optimal_side(underlying_price, &side)
                        && short.is_valid_optimal_side(underlying_price, &side)
                        && short_high.is_valid_optimal_side(underlying_price, &side)
                }
            })
            .filter(move |(short_low, short, short_high)| {
                short_low.strike_price < short.strike_price
                    && short.strike_price < short_high.strike_price
            })
            // Filter out options with invalid bid/ask prices
            .filter(|(short_low, short, short_high)| {
                short_low.call_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && short.call_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && short_high.call_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
            })
            // Filter out options that don't meet strategy constraints
            .filter(move |(short_low, short, short_high)| {
                let legs = StrategyLegs::ThreeLegs {
                    first: short_low,
                    second: short,
                    third: short_high,
                };
                let strategy = strategy.create_strategy(option_chain, &legs);
                strategy.validate()
                    && strategy.get_max_profit().is_ok()
                    && strategy.get_max_loss().is_ok()
            })
            // Map to OptionDataGroup
            .map(move |(short_low, short, short_high)| {
                OptionDataGroup::Three(short_low, short, short_high)
            })
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
            let (short_low, short, short_high) = match option_data_group {
                OptionDataGroup::Three(first, second, third) => (first, second, third),
                _ => panic!("Invalid OptionDataGroup"),
            };

            let legs = StrategyLegs::ThreeLegs {
                first: short_low,
                second: short,
                third: short_high,
            };
            let strategy = self.create_strategy(option_chain, &legs);
            // Calculate the current value based on the optimization criteria
            let current_value = match criteria {
                OptimizationCriteria::Ratio => strategy.get_profit_ratio().unwrap(),
                OptimizationCriteria::Area => strategy.get_profit_area().unwrap(),
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
        match legs {
            StrategyLegs::ThreeLegs {
                first: low_strike,
                second: middle_strike,
                third: high_strike,
            } => {
                let implied_volatility = middle_strike.implied_volatility;
                assert!(implied_volatility <= Positive::ONE);

                ShortButterflySpread::new(
                    chain.symbol.clone(),
                    chain.underlying_price,
                    low_strike.strike_price,
                    middle_strike.strike_price,
                    high_strike.strike_price,
                    self.short_call_low.option.expiration_date,
                    implied_volatility,
                    self.short_call_low.option.risk_free_rate,
                    self.short_call_low.option.dividend_yield,
                    self.short_call_low.option.quantity,
                    low_strike.call_bid.unwrap(),
                    middle_strike.call_ask.unwrap(),
                    high_strike.call_bid.unwrap(),
                    self.long_call.open_fee,
                    self.long_call.close_fee,
                    self.short_call_low.open_fee,
                    self.short_call_low.close_fee,
                    self.short_call_high.open_fee,
                    self.short_call_high.close_fee,
                )
            }
            _ => panic!("Invalid number of legs for Short Butterfly strategy"),
        }
    }
}

impl Profit for ShortButterflySpread {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, PricingError> {
        let price = Some(price);
        Ok(self.short_call_low.pnl_at_expiration(&price)?
            + self.long_call.pnl_at_expiration(&price)?
            + self.short_call_high.pnl_at_expiration(&price)?)
    }
}

impl ProbabilityAnalysis for ShortButterflySpread {
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let mut ranges = Vec::new();
        let break_even_points = self.get_break_even_points()?;
        let option = &self.long_call.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            self.short_call_low.option.implied_volatility,
            self.long_call.option.implied_volatility,
            self.short_call_high.option.implied_volatility,
        ]);

        let volatility_adjustment = Some(VolatilityAdjustment {
            base_volatility: mean_volatility,
            std_dev_adjustment: std_dev,
        });

        let mut lower_profit_range =
            ProfitLossRange::new(None, Some(break_even_points[0]), Positive::ZERO)?;

        lower_profit_range.calculate_probability(
            self.get_underlying_price(),
            volatility_adjustment.clone(),
            None,
            expiration_date,
            Some(risk_free_rate),
        )?;

        ranges.push(lower_profit_range);

        let mut upper_profit_range =
            ProfitLossRange::new(Some(break_even_points[1]), None, Positive::ZERO)?;

        upper_profit_range.calculate_probability(
            self.get_underlying_price(),
            volatility_adjustment,
            None,
            expiration_date,
            Some(risk_free_rate),
        )?;

        ranges.push(upper_profit_range);

        Ok(ranges)
    }

    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_points = self.get_break_even_points()?;
        let option = &self.long_call.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            self.short_call_low.option.implied_volatility,
            self.long_call.option.implied_volatility,
            self.short_call_high.option.implied_volatility,
        ]);

        let mut loss_range = ProfitLossRange::new(
            Some(break_even_points[0]),
            Some(break_even_points[1]),
            pos_or_panic!(self.get_max_loss()?.to_f64()),
        )?;

        loss_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            expiration_date,
            Some(risk_free_rate),
        )?;

        Ok(vec![loss_range])
    }
}

impl Greeks for ShortButterflySpread {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![
            &self.short_call_low.option,
            &self.long_call.option,
            &self.short_call_high.option,
        ])
    }
}

impl DeltaNeutrality for ShortButterflySpread {}

impl PnLCalculator for ShortButterflySpread {
    fn calculate_pnl(
        &self,
        market_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, PricingError> {
        Ok(self
            .long_call
            .calculate_pnl(market_price, expiration_date, implied_volatility)?
            + self.short_call_low.calculate_pnl(
                market_price,
                expiration_date,
                implied_volatility,
            )?
            + self.short_call_high.calculate_pnl(
                market_price,
                expiration_date,
                implied_volatility,
            )?)
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<PnL, PricingError> {
        Ok(self
            .long_call
            .calculate_pnl_at_expiration(underlying_price)?
            + self
                .short_call_low
                .calculate_pnl_at_expiration(underlying_price)?
            + self
                .short_call_high
                .calculate_pnl_at_expiration(underlying_price)?)
    }
}

test_strategy_traits!(ShortButterflySpread, test_short_call_implementations);

#[cfg(test)]
mod tests_short_butterfly_spread {
    use super::*;

    use crate::model::ExpirationDate;

    use rust_decimal_macros::dec;

    fn create_test_butterfly() -> ShortButterflySpread {
        ShortButterflySpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,                         // underlying_price
            pos_or_panic!(90.0),                       // low_strike
            Positive::HUNDRED,                         // middle_strike
            pos_or_panic!(110.0),                      // high_strike
            ExpirationDate::Days(pos_or_panic!(30.0)), // expiration
            pos_or_panic!(0.2),                        // implied_volatility
            dec!(0.05),                                // risk_free_rate
            Positive::ZERO,                            // dividend_yield
            Positive::ONE,                             // quantity
            pos_or_panic!(10.0),                       // premium_low
            Positive::ONE,                             // premium_middle
            pos_or_panic!(0.5),                        // premium_high
            pos_or_panic!(0.05),                       // open_fee_short_call
            pos_or_panic!(0.05),                       // close_fee_short_call
            pos_or_panic!(0.05),                       // open_fee_short_call_low
            pos_or_panic!(0.05),                       // close_fee_short_call_low
            pos_or_panic!(0.05),                       // open_fee_short_call_high
            pos_or_panic!(0.05),                       // close_fee_short_call_high
        )
    }

    #[test]
    fn test_new_butterfly_basic_properties() {
        let butterfly = create_test_butterfly();

        assert_eq!(butterfly.name, "Short Butterfly");
        assert_eq!(butterfly.kind, StrategyType::ShortButterflySpread);
        assert!(!butterfly.description.is_empty());
        assert!(butterfly.description.contains("short butterfly spread"));
    }

    #[test]
    fn test_butterfly_strikes() {
        let butterfly = create_test_butterfly();

        assert_eq!(
            butterfly.short_call_low.option.strike_price,
            pos_or_panic!(90.0)
        );
        assert_eq!(butterfly.long_call.option.strike_price, Positive::HUNDRED);
        assert_eq!(
            butterfly.short_call_high.option.strike_price,
            pos_or_panic!(110.0)
        );
    }

    #[test]
    fn test_butterfly_quantities() {
        let butterfly = create_test_butterfly();

        assert_eq!(butterfly.short_call_low.option.quantity, Positive::ONE);
        assert_eq!(butterfly.long_call.option.quantity, Positive::TWO); // Double quantity
        assert_eq!(butterfly.short_call_high.option.quantity, Positive::ONE);
    }

    #[test]
    fn test_butterfly_sides() {
        let butterfly = create_test_butterfly();

        assert_eq!(butterfly.short_call_low.option.side, Side::Short);
        assert_eq!(butterfly.long_call.option.side, Side::Long);
        assert_eq!(butterfly.short_call_high.option.side, Side::Short);
    }

    #[test]
    fn test_butterfly_option_styles() {
        let butterfly = create_test_butterfly();

        assert_eq!(
            butterfly.short_call_low.option.option_style,
            OptionStyle::Call
        );
        assert_eq!(butterfly.long_call.option.option_style, OptionStyle::Call);
        assert_eq!(
            butterfly.short_call_high.option.option_style,
            OptionStyle::Call
        );
    }

    #[test]
    fn test_butterfly_expiration_consistency() {
        let butterfly = create_test_butterfly();
        let expiration = ExpirationDate::Days(pos_or_panic!(30.0));

        assert_eq!(
            format!("{:?}", butterfly.short_call_low.option.expiration_date),
            format!("{:?}", expiration)
        );
        assert_eq!(
            format!("{:?}", butterfly.long_call.option.expiration_date),
            format!("{:?}", expiration)
        );
        assert_eq!(
            format!("{:?}", butterfly.short_call_high.option.expiration_date),
            format!("{:?}", expiration)
        );
    }

    #[test]
    fn test_butterfly_fees_distribution() {
        let butterfly = ShortButterflySpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::ONE,
            pos_or_panic!(3.0),
            Positive::TWO,
            Positive::ONE,
            Positive::ONE,       // open_fee_short_call
            pos_or_panic!(0.05), // close_fee_short_call
            Positive::ONE,       // open_fee_short_call_low
            pos_or_panic!(0.05), // close_fee_short_call_low
            Positive::ONE,       // open_fee_short_call_high
            pos_or_panic!(0.05), // close_fee_short_call_high
        );

        assert_eq!(butterfly.short_call_low.open_fee, 1.0); // fees / 3
        assert_eq!(butterfly.long_call.open_fee, 1.0); // fees / 3
        assert_eq!(butterfly.short_call_high.open_fee, 1.0); // fees / 3
    }

    #[test]
    fn test_butterfly_break_even_points() {
        let butterfly = create_test_butterfly();
        let break_even_points = butterfly.break_even_points;

        assert_eq!(break_even_points.len(), 2);
        assert!(break_even_points[0] > butterfly.short_call_low.option.strike_price);
        assert!(break_even_points[0] < butterfly.long_call.option.strike_price);
        assert!(break_even_points[1] > butterfly.long_call.option.strike_price);
        assert!(break_even_points[1] < butterfly.short_call_high.option.strike_price);
    }

    #[test]
    fn test_butterfly_with_different_quantities() {
        let butterfly = ShortButterflySpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::TWO,
            pos_or_panic!(3.0),
            Positive::TWO,
            Positive::ONE,
            pos_or_panic!(0.05), // open_fee_short_call
            pos_or_panic!(0.05), // close_fee_short_call
            pos_or_panic!(0.05), // open_fee_short_call_low
            pos_or_panic!(0.05), // close_fee_short_call_low
            pos_or_panic!(0.05), // open_fee_short_call_high
            pos_or_panic!(0.05), // close_fee_short_call_high
        );

        assert_eq!(butterfly.short_call_low.option.quantity, Positive::TWO);
        assert_eq!(butterfly.long_call.option.quantity, pos_or_panic!(4.0)); // 2 * 2
        assert_eq!(butterfly.short_call_high.option.quantity, Positive::TWO);
    }

    #[test]
    fn test_butterfly_with_symmetric_strikes() {
        let butterfly = create_test_butterfly();

        let lower_width =
            butterfly.long_call.option.strike_price - butterfly.short_call_low.option.strike_price;
        let upper_width =
            butterfly.short_call_high.option.strike_price - butterfly.long_call.option.strike_price;

        assert_eq!(lower_width, upper_width);
    }

    #[test]
    fn test_butterfly_with_equal_implied_volatility() {
        let butterfly = create_test_butterfly();

        assert_eq!(
            butterfly.short_call_low.option.implied_volatility,
            butterfly.long_call.option.implied_volatility
        );
        assert_eq!(
            butterfly.long_call.option.implied_volatility,
            butterfly.short_call_high.option.implied_volatility
        );
    }

    #[test]
    fn test_butterfly_underlying_price_consistency() {
        let butterfly = create_test_butterfly();
        let underlying_price = Positive::HUNDRED;

        assert_eq!(
            butterfly.short_call_low.option.underlying_price,
            underlying_price
        );
        assert_eq!(
            butterfly.long_call.option.underlying_price,
            underlying_price
        );
        assert_eq!(
            butterfly.short_call_high.option.underlying_price,
            underlying_price
        );
    }

    #[test]
    fn test_butterfly_with_invalid_premiums() {
        let max_loss = ShortButterflySpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::ONE,
            pos_or_panic!(10.0),
            Positive::ONE,
            pos_or_panic!(10.0),
            pos_or_panic!(0.05), // open_fee_short_call
            pos_or_panic!(0.05), // close_fee_short_call
            pos_or_panic!(0.05), // open_fee_short_call_low
            pos_or_panic!(0.05), // close_fee_short_call_low
            pos_or_panic!(0.05), // open_fee_short_call_high
            pos_or_panic!(0.05), // close_fee_short_call_high
        );
        assert!(max_loss.get_max_loss().is_err());
    }

    #[test]
    fn test_butterfly_risk_free_rate_consistency() {
        let butterfly = create_test_butterfly();
        let risk_free_rate = dec!(0.05);

        assert_eq!(
            butterfly.short_call_low.option.risk_free_rate,
            risk_free_rate
        );
        assert_eq!(butterfly.long_call.option.risk_free_rate, risk_free_rate);
        assert_eq!(
            butterfly.short_call_high.option.risk_free_rate,
            risk_free_rate
        );
    }
}

#[cfg(test)]
mod tests_short_butterfly_validation {
    use super::*;

    use rust_decimal_macros::dec;

    fn create_valid_position(side: Side, strike_price: Positive, quantity: Positive) -> Position {
        Position::new(
            Options::new(
                OptionType::European,
                side,
                "TEST".to_string(),
                strike_price,
                ExpirationDate::Days(pos_or_panic!(30.0)),
                pos_or_panic!(0.2),
                quantity,
                Positive::HUNDRED,
                dec!(0.05),
                OptionStyle::Call,
                Positive::ZERO,
                None,
            ),
            Positive::ONE,
            Utc::now(),
            Positive::ZERO,
            Positive::ZERO,
            None,
            None,
        )
    }

    #[test]
    fn test_valid_short_butterfly() {
        let butterfly = ShortButterflySpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::ONE,
            Positive::ONE,
            Positive::TWO,
            Positive::ONE,
            pos_or_panic!(0.05), // open_fee_short_call
            pos_or_panic!(0.05), // close_fee_short_call
            pos_or_panic!(0.05), // open_fee_short_call_low
            pos_or_panic!(0.05), // close_fee_short_call_low
            pos_or_panic!(0.05), // open_fee_short_call_high
            pos_or_panic!(0.05), // close_fee_short_call_high
        );
        assert!(butterfly.validate());
    }

    #[test]
    fn test_invalid_short_call_low() {
        let mut butterfly = ShortButterflySpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::ONE,
            Positive::ONE,
            Positive::TWO,
            Positive::ONE,
            pos_or_panic!(0.05), // open_fee_short_call
            pos_or_panic!(0.05), // close_fee_short_call
            pos_or_panic!(0.05), // open_fee_short_call_low
            pos_or_panic!(0.05), // close_fee_short_call_low
            pos_or_panic!(0.05), // open_fee_short_call_high
            pos_or_panic!(0.05), // close_fee_short_call_high
        );
        butterfly.short_call_low =
            create_valid_position(Side::Short, pos_or_panic!(90.0), Positive::ZERO);
        assert!(!butterfly.validate());
    }

    #[test]
    fn test_invalid_strike_order_high() {
        let butterfly = ShortButterflySpread::new(
            "TEST".to_string(),
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            Positive::HUNDRED,
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::ONE,
            Positive::ONE,
            Positive::TWO,
            Positive::ONE,
            pos_or_panic!(0.05), // open_fee_short_call
            pos_or_panic!(0.05), // close_fee_short_call
            pos_or_panic!(0.05), // open_fee_short_call_low
            pos_or_panic!(0.05), // close_fee_short_call_low
            pos_or_panic!(0.05), // open_fee_short_call_high
            pos_or_panic!(0.05), // close_fee_short_call_high
        );
        assert!(!butterfly.validate());
    }

    #[test]
    fn test_invalid_middle_quantities() {
        let mut butterfly = ShortButterflySpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::ONE,
            Positive::ONE,
            Positive::TWO,
            Positive::ONE,
            pos_or_panic!(0.05), // open_fee_short_call
            pos_or_panic!(0.05), // close_fee_short_call
            pos_or_panic!(0.05), // open_fee_short_call_low
            pos_or_panic!(0.05), // close_fee_short_call_low
            pos_or_panic!(0.05), // open_fee_short_call_high
            pos_or_panic!(0.05), // close_fee_short_call_high
        );
        butterfly.long_call = create_valid_position(Side::Long, Positive::HUNDRED, Positive::ONE);
        assert!(!butterfly.validate());
    }

    #[test]
    fn test_unequal_wing_quantities_short() {
        let mut butterfly = ShortButterflySpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::ONE,
            Positive::ONE,
            Positive::TWO,
            Positive::ONE,
            pos_or_panic!(0.05), // open_fee_short_call
            pos_or_panic!(0.05), // close_fee_short_call
            pos_or_panic!(0.05), // open_fee_short_call_low
            pos_or_panic!(0.05), // close_fee_short_call_low
            pos_or_panic!(0.05), // open_fee_short_call_high
            pos_or_panic!(0.05), // close_fee_short_call_high
        );
        butterfly.short_call_high =
            create_valid_position(Side::Short, pos_or_panic!(110.0), Positive::TWO);
        assert!(!butterfly.validate());
    }
}

#[cfg(test)]
mod tests_short_butterfly_profit {
    use super::*;

    use crate::model::ExpirationDate;

    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::str::FromStr;

    fn create_test() -> ShortButterflySpread {
        ShortButterflySpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,    // underlying_price
            pos_or_panic!(90.0),  // low_strike
            Positive::HUNDRED,    // middle_strike
            pos_or_panic!(110.0), // high_strike
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),  // implied_volatility
            dec!(0.05),          // risk_free_rate
            Positive::ZERO,      // dividend_yield
            Positive::ONE,       // quantity
            pos_or_panic!(3.0),  // premium_low
            Positive::TWO,       // premium_middle
            Positive::ONE,       // premium_high
            pos_or_panic!(0.05), // open_fee_short_call
            pos_or_panic!(0.05), // close_fee_short_call
            pos_or_panic!(0.05), // open_fee_short_call_low
            pos_or_panic!(0.05), // close_fee_short_call_low
            pos_or_panic!(0.05), // open_fee_short_call_high
            pos_or_panic!(0.05), // close_fee_short_call_high
        )
    }

    #[test]
    fn test_profit_at_middle_strike() {
        let butterfly = create_test();
        let profit = butterfly.calculate_profit_at(&Positive::HUNDRED).unwrap();
        assert!(profit < Decimal::ZERO);
        let expected = Positive::new_decimal(Decimal::from_str("10.4").unwrap()).unwrap();

        assert_eq!(-profit, expected);
    }

    #[test]
    fn test_profit_at_break_even_points() {
        let butterfly = create_test();
        let break_even_points = butterfly.get_break_even_points().unwrap();

        for &point in break_even_points {
            let profit = butterfly
                .calculate_profit_at(&point)
                .unwrap()
                .to_f64()
                .unwrap();
            assert_relative_eq!(profit, 0.0, epsilon = 0.01);
        }
    }

    #[test]
    fn test_profit_with_different_quantities() {
        let butterfly = ShortButterflySpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::TWO, // quantity = 2
            pos_or_panic!(3.0),
            Positive::TWO,
            Positive::ONE,
            pos_or_panic!(0.05), // open_fee_short_call
            pos_or_panic!(0.05), // close_fee_short_call
            pos_or_panic!(0.05), // open_fee_short_call_low
            pos_or_panic!(0.05), // close_fee_short_call_low
            pos_or_panic!(0.05), // open_fee_short_call_high
            pos_or_panic!(0.05), // close_fee_short_call_high
        );
        let scaled_profit = butterfly
            .calculate_profit_at(&pos_or_panic!(85.0))
            .unwrap()
            .to_f64()
            .unwrap();
        assert_relative_eq!(scaled_profit, -0.8, epsilon = 0.0001);
    }

    #[test]
    fn test_profit_symmetry() {
        let butterfly = create_test();
        let low_extreme_profit = butterfly
            .calculate_profit_at(&pos_or_panic!(85.0))
            .unwrap()
            .to_f64()
            .unwrap();
        let high_extreme_profit = butterfly
            .calculate_profit_at(&pos_or_panic!(115.0))
            .unwrap()
            .to_f64()
            .unwrap();

        assert_relative_eq!(low_extreme_profit, high_extreme_profit, epsilon = 0.01);
    }

    #[test]
    fn test_profit_with_fees() {
        let butterfly = ShortButterflySpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::ONE,
            pos_or_panic!(3.0),
            Positive::TWO,
            Positive::ONE,
            Positive::ZERO, // open_fee_short_call
            Positive::ZERO, // close_fee_short_call
            Positive::ZERO, // open_fee_short_call_low
            Positive::ZERO, // close_fee_short_call_low
            Positive::ZERO, // open_fee_short_call_high
            Positive::ZERO, // close_fee_short_call_high
        );

        let base_butterfly = create_test();
        let profit_without_fees = butterfly
            .calculate_profit_at(&pos_or_panic!(85.0))
            .unwrap()
            .to_f64()
            .unwrap();
        let profit_with_fees = base_butterfly
            .calculate_profit_at(&pos_or_panic!(85.0))
            .unwrap()
            .to_f64()
            .unwrap();
        assert!(profit_with_fees < profit_without_fees);
    }
}

#[cfg(test)]
mod tests_short_butterfly_delta {
    use super::*;

    use crate::assert_decimal_eq;
    use crate::model::types::OptionStyle;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::str::FromStr;

    fn get_strategy(underlying_price: Positive) -> ShortButterflySpread {
        ShortButterflySpread::new(
            "SP500".to_string(),
            underlying_price,      // underlying_price
            pos_or_panic!(5700.0), // short_strike_itm
            pos_or_panic!(5780.0), // short_strike
            pos_or_panic!(5850.0), // short_strike_otm
            ExpirationDate::Days(Positive::TWO),
            pos_or_panic!(0.18),   // implied_volatility
            Decimal::ZERO,         // risk_free_rate
            Positive::ZERO,        // dividend_yield
            Positive::ONE,         // short quantity
            pos_or_panic!(119.01), // premium_short
            pos_or_panic!(66.0),   // premium_short
            pos_or_panic!(29.85),  // open_fee_short
            pos_or_panic!(0.05),   // open_fee_short_call
            pos_or_panic!(0.05),   // close_fee_short_call
            pos_or_panic!(0.05),   // open_fee_short_call_low
            pos_or_panic!(0.05),   // close_fee_short_call_low
            pos_or_panic!(0.05),   // open_fee_short_call_high
            pos_or_panic!(0.05),   // close_fee_short_call_high
        )
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos_or_panic!(5781.88));
        let size = dec!(-0.0259);
        let delta =
            Positive::new_decimal(Decimal::from_str("0.05072646985065364").unwrap()).unwrap();
        let k = pos_or_panic!(5780.0);
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
        let strategy = get_strategy(pos_or_panic!(5881.88));
        let size = dec!(0.16077612);
        let delta1 = pos_or_panic!(0.16224251196539);
        let delta2 = pos_or_panic!(0.17740792537439);
        let k1 = pos_or_panic!(5700.0);
        let k2 = pos_or_panic!(5780.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        match &binding[0] {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta1, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k1, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        match &binding[1] {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta2, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k2, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Long);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.short_call_low.option.clone();
        option.quantity = delta1;
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
        let strategy = get_strategy(pos_or_panic!(5788.55));

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
mod tests_short_butterfly_delta_size {
    use super::*;

    use crate::assert_decimal_eq;
    use crate::model::types::OptionStyle;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    fn get_strategy(underlying_price: Positive) -> ShortButterflySpread {
        ShortButterflySpread::new(
            "SP500".to_string(),
            underlying_price,      // underlying_price
            pos_or_panic!(5700.0), // short_strike_itm
            pos_or_panic!(5780.0), // short_strike
            pos_or_panic!(5850.0), // short_strike_otm
            ExpirationDate::Days(Positive::TWO),
            pos_or_panic!(0.18),   // implied_volatility
            dec!(0.05),            // risk_free_rate
            Positive::ZERO,        // dividend_yield
            pos_or_panic!(3.0),    // short quantity
            pos_or_panic!(119.01), // premium_short
            pos_or_panic!(66.0),   // premium_short
            pos_or_panic!(29.85),  // open_fee_short
            pos_or_panic!(0.05),   // open_fee_short_call
            pos_or_panic!(0.05),   // close_fee_short_call
            pos_or_panic!(0.05),   // open_fee_short_call_low
            pos_or_panic!(0.05),   // close_fee_short_call_low
            pos_or_panic!(0.05),   // open_fee_short_call_high
            pos_or_panic!(0.05),   // close_fee_short_call_high
        )
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos_or_panic!(5781.88));
        let size = dec!(-0.0593);
        let delta = pos_or_panic!(0.11409430831966512);
        let k = pos_or_panic!(5780.0);
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
        let strategy = get_strategy(pos_or_panic!(5881.88));
        let size = dec!(0.4787);
        let delta1 = pos_or_panic!(0.4828726371186378);
        let delta2 = pos_or_panic!(0.5262977508284383);
        let k1 = pos_or_panic!(5700.0);
        let k2 = pos_or_panic!(5780.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        match &binding[0] {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta1, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k1, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        match &binding[1] {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta2, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k2, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Long);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.short_call_low.option.clone();
        option.quantity = delta1;
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
        let strategy = get_strategy(pos_or_panic!(5786.99));

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
mod tests_adjust_option_position_short {
    use super::*;

    use crate::model::types::{OptionStyle, Side};

    use rust_decimal_macros::dec;

    // Helper function to create a test strategy
    fn create_test_strategy() -> ShortButterflySpread {
        ShortButterflySpread::new(
            "SP500".to_string(),
            pos_or_panic!(5781.88), // underlying_price
            pos_or_panic!(5700.0),  // short_strike_itm
            pos_or_panic!(5780.0),  // short_strike
            pos_or_panic!(5850.0),  // short_strike_otm
            ExpirationDate::Days(Positive::TWO),
            pos_or_panic!(0.18),   // implied_volatility
            dec!(0.05),            // risk_free_rate
            Positive::ZERO,        // dividend_yield
            Positive::ONE,         // short quantity
            pos_or_panic!(119.01), // premium_short
            pos_or_panic!(66.0),   // premium_short
            pos_or_panic!(29.85),  // open_fee_short
            pos_or_panic!(0.05),
            pos_or_panic!(0.05),
            pos_or_panic!(0.05),
            pos_or_panic!(0.05),
            pos_or_panic!(0.05),
            pos_or_panic!(0.05),
        )
    }

    #[test]
    fn test_adjust_existing_call_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.short_call_low.option.quantity;
        let adjustment = Positive::ONE;

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos_or_panic!(5700.0),
            &OptionStyle::Call,
            &Side::Short,
        );

        assert!(result.is_ok());
        assert_eq!(
            strategy.short_call_low.option.quantity,
            initial_quantity + adjustment
        );
    }

    #[test]
    fn test_adjust_existing_short_call_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.long_call.option.quantity;
        let adjustment = Positive::ONE;

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos_or_panic!(5780.0),
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
    fn test_adjust_nonexistent_position() {
        let mut strategy = create_test_strategy();

        // Try to adjust a non-existent short call position
        let result = strategy.adjust_option_position(
            Decimal::ONE,
            &pos_or_panic!(5780.0),
            &OptionStyle::Put,
            &Side::Long,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        // StrategyError wraps PositionError, so we check the error message
        assert!(err.to_string().contains("Put not found in positions"));
    }

    #[test]
    fn test_adjust_with_invalid_strike() {
        let mut strategy = create_test_strategy();

        // Try to adjust position with wrong strike price
        let result = strategy.adjust_option_position(
            Decimal::ONE,
            &Positive::HUNDRED, // Invalid strike price
            &OptionStyle::Call,
            &Side::Short,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_zero_quantity_adjustment() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.short_call_high.option.quantity;

        let result = strategy.adjust_option_position(
            Decimal::ZERO,
            &pos_or_panic!(5850.0),
            &OptionStyle::Call,
            &Side::Short,
        );

        assert!(result.is_ok());
        assert_eq!(strategy.short_call_high.option.quantity, initial_quantity);
    }
}

#[cfg(test)]
mod tests_short_butterfly_position_management {
    use super::*;

    use crate::error::position::PositionValidationErrorKind;
    use crate::model::types::{OptionStyle, Side};

    use rust_decimal_macros::dec;
    use tracing::error;

    fn create_test_butterfly() -> ShortButterflySpread {
        ShortButterflySpread::new(
            "SP500".to_string(),
            pos_or_panic!(5781.88), // underlying_price
            pos_or_panic!(5700.0),  // short_strike_itm
            pos_or_panic!(5780.0),  // short_strike
            pos_or_panic!(5850.0),  // short_strike_otm
            ExpirationDate::Days(Positive::TWO),
            pos_or_panic!(0.18),   // implied_volatility
            dec!(0.05),            // risk_free_rate
            Positive::ZERO,        // dividend_yield
            Positive::ONE,         // short quantity
            pos_or_panic!(119.01), // premium_short
            pos_or_panic!(66.0),   // premium_short
            pos_or_panic!(29.85),  // open_fee_short
            pos_or_panic!(0.05),
            pos_or_panic!(0.05),
            pos_or_panic!(0.05),
            pos_or_panic!(0.05),
            pos_or_panic!(0.05),
            pos_or_panic!(0.05),
        )
    }

    #[test]
    fn test_short_butterfly_get_position() {
        let mut butterfly = create_test_butterfly();

        // Test getting short call position
        let call_position =
            butterfly.get_position(&OptionStyle::Call, &Side::Long, &pos_or_panic!(5780.0));
        assert!(call_position.is_ok());
        let positions = call_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos_or_panic!(5780.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Long);

        // Test getting short put position
        let put_position =
            butterfly.get_position(&OptionStyle::Put, &Side::Short, &pos_or_panic!(2560.0));
        assert!(put_position.is_err());

        // Test getting non-existent position
        let invalid_position =
            butterfly.get_position(&OptionStyle::Call, &Side::Short, &pos_or_panic!(2715.0));
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
    fn test_short_butterfly_modify_position() {
        let mut butterfly = create_test_butterfly();

        // Modify short call position
        let mut modified_call = butterfly.long_call.clone();
        modified_call.option.quantity = Positive::TWO;
        let result = butterfly.modify_position(&modified_call);
        assert!(result.is_ok());
        assert_eq!(butterfly.long_call.option.quantity, Positive::TWO);

        // Test modifying with invalid position
        let mut invalid_position = butterfly.long_call.clone();
        invalid_position.option.strike_price = pos_or_panic!(95.0);
        let result = butterfly.modify_position(&invalid_position);
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
mod tests_short_butterfly_spread_constructor {
    use super::*;

    use crate::model::utils::create_sample_position;

    #[test]
    fn test_get_strategy_valid() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos_or_panic!(90.0),
                Positive::ONE,
                pos_or_panic!(95.0),
                pos_or_panic!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos_or_panic!(90.0),
                Positive::ONE,
                Positive::HUNDRED,
                pos_or_panic!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos_or_panic!(90.0),
                Positive::ONE,
                pos_or_panic!(105.0),
                pos_or_panic!(0.2),
            ),
        ];

        let result = ShortButterflySpread::get_strategy(&options);
        assert!(result.is_ok());

        let strategy = result.unwrap();
        assert_eq!(
            strategy.short_call_low.option.strike_price,
            pos_or_panic!(95.0)
        );
        assert_eq!(strategy.long_call.option.strike_price, Positive::HUNDRED);
        assert_eq!(
            strategy.short_call_high.option.strike_price,
            pos_or_panic!(105.0)
        );
    }

    #[test]
    fn test_get_strategy_wrong_number_of_options() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos_or_panic!(90.0),
                Positive::ONE,
                pos_or_panic!(95.0),
                pos_or_panic!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos_or_panic!(90.0),
                Positive::ONE,
                Positive::HUNDRED,
                pos_or_panic!(0.2),
            ),
        ];

        let result = ShortButterflySpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Short Butterfly Spread get_strategy" && reason == "Must have exactly 3 options"
        ));
    }

    #[test]
    fn test_get_strategy_wrong_option_style() {
        let mut option1 = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(90.0),
            Positive::ONE,
            pos_or_panic!(95.0),
            pos_or_panic!(0.2),
        );
        option1.option.option_style = OptionStyle::Put;
        let option2 = create_sample_position(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(90.0),
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
        );
        let option3 = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(90.0),
            Positive::ONE,
            pos_or_panic!(105.0),
            pos_or_panic!(0.2),
        );

        let options = vec![option1, option2, option3];
        let result = ShortButterflySpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Short Butterfly Spread get_strategy" && reason == "Options must be calls"
        ));
    }

    #[test]
    fn test_get_strategy_wrong_sides() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos_or_panic!(90.0),
                Positive::ONE,
                pos_or_panic!(95.0),
                pos_or_panic!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos_or_panic!(90.0),
                Positive::ONE,
                Positive::HUNDRED,
                pos_or_panic!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos_or_panic!(90.0),
                Positive::ONE,
                pos_or_panic!(105.0),
                pos_or_panic!(0.2),
            ),
        ];

        let result = ShortButterflySpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Short Butterfly Spread get_strategy"
                && reason == "Short Butterfly requires short lower and higher strikes with a short middle strike"
        ));
    }

    #[test]
    fn test_get_strategy_asymmetric_strikes() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos_or_panic!(90.0),
                Positive::ONE,
                pos_or_panic!(95.0),
                pos_or_panic!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos_or_panic!(90.0),
                Positive::ONE,
                pos_or_panic!(101.0),
                pos_or_panic!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos_or_panic!(90.0),
                Positive::ONE,
                pos_or_panic!(105.0),
                pos_or_panic!(0.2),
            ),
        ];

        let result = ShortButterflySpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Short Butterfly Spread get_strategy" && reason == "Strikes must be symmetrical"
        ));
    }

    #[test]
    fn test_get_strategy_different_expiration_dates() {
        let mut option1 = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(90.0),
            Positive::ONE,
            pos_or_panic!(95.0),
            pos_or_panic!(0.2),
        );
        let mut option2 = create_sample_position(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(90.0),
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
        );
        let mut option3 = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(90.0),
            Positive::ONE,
            pos_or_panic!(105.0),
            pos_or_panic!(0.2),
        );

        option1.option.expiration_date = ExpirationDate::Days(pos_or_panic!(30.0));
        option2.option.expiration_date = ExpirationDate::Days(pos_or_panic!(60.0));
        option3.option.expiration_date = ExpirationDate::Days(pos_or_panic!(30.0));

        let options = vec![option1, option2, option3];
        let result = ShortButterflySpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Short Butterfly Spread get_strategy" && reason == "Options must have the same expiration date"
        ));
    }
}

#[cfg(test)]
mod tests_short_butterfly_spread_pnl {
    use super::*;

    use crate::assert_decimal_eq;
    use crate::model::utils::create_sample_position;
    use rust_decimal_macros::dec;

    fn create_test_short_butterfly_spread() -> Result<ShortButterflySpread, StrategyError> {
        // Create lower short call
        let lower_long_call = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            Positive::HUNDRED,   // Underlying price
            Positive::ONE,       // Quantity
            pos_or_panic!(95.0), // Lower strike price
            pos_or_panic!(0.2),  // Implied volatility
        );

        // Create middle short call
        let middle_long_call = create_sample_position(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,  // Same underlying price
            Positive::ONE,      // Quantity
            Positive::HUNDRED,  // Middle strike price (ATM)
            pos_or_panic!(0.2), // Implied volatility
        );

        // Create higher short call
        let higher_long_call = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            Positive::HUNDRED,    // Same underlying price
            Positive::ONE,        // Quantity
            pos_or_panic!(105.0), // Higher strike price
            pos_or_panic!(0.2),   // Implied volatility
        );

        ShortButterflySpread::get_strategy(&[lower_long_call, middle_long_call, higher_long_call])
    }

    #[test]
    fn test_calculate_pnl_below_strikes() {
        let spread = create_test_short_butterfly_spread().unwrap();
        let market_price = pos_or_panic!(90.0); // Below all strikes
        let expiration_date = ExpirationDate::Days(pos_or_panic!(20.0));
        let implied_volatility = pos_or_panic!(0.2);

        let result = spread.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // All options OTM, near max profit
        assert!(pnl.unrealized.unwrap() > dec!(0.0));
        assert!(pnl.unrealized.unwrap() < dec!(5.0)); // Not better than max profit
    }

    #[test]
    fn test_calculate_pnl_between_strikes() {
        let spread = create_test_short_butterfly_spread().unwrap();
        let market_price = pos_or_panic!(99.0);
        let expiration_date = ExpirationDate::Days(pos_or_panic!(20.0));
        let implied_volatility = pos_or_panic!(0.1);

        let result = spread.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Near max profit, as at-the-money
        assert!(pnl.unrealized.unwrap() > dec!(0.0));
        assert!(pnl.unrealized.unwrap() < dec!(5.0));
    }

    #[test]
    fn test_calculate_pnl_above_strikes() {
        let spread = create_test_short_butterfly_spread().unwrap();
        let market_price = pos_or_panic!(90.0);
        let expiration_date = ExpirationDate::Days(pos_or_panic!(20.0));
        let implied_volatility = pos_or_panic!(0.2);

        let result = spread.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // All options ITM, near max profit
        assert!(pnl.unrealized.unwrap() > dec!(0.0));
        assert!(pnl.unrealized.unwrap() < dec!(5.0)); // Not better than max profit
    }

    #[test]
    fn test_calculate_pnl_at_expiration_max_profit() {
        let spread = create_test_short_butterfly_spread().unwrap();
        let underlying_price = pos_or_panic!(95.0); // At or below lowest strike

        let result = spread.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // Max profit should be the net credit received
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(2.0), dec!(1e-6));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_max_loss() {
        let spread = create_test_short_butterfly_spread().unwrap();
        let underlying_price = pos_or_panic!(110.0); // Above highest strike

        let result = spread.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // Max loss should be the net credit received minus spread width
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(-8.0), dec!(1e-6));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_at_middle_strike() {
        let spread = create_test_short_butterfly_spread().unwrap();
        let underlying_price = Positive::HUNDRED; // At middle strike

        let result = spread.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // Near max profit
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(-3.0), dec!(1e-6));
    }
}

#[cfg(test)]
mod tests_butterfly_strategies {
    use super::*;

    use crate::constants::ZERO;
    use crate::model::ExpirationDate;

    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn create_test_short() -> ShortButterflySpread {
        ShortButterflySpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,    // underlying_price
            pos_or_panic!(90.0),  // low_strike
            Positive::HUNDRED,    // middle_strike
            pos_or_panic!(110.0), // high_strike
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),  // implied_volatility
            dec!(0.05),          // risk_free_rate
            Positive::ZERO,      // dividend_yield
            Positive::ONE,       // quantity
            pos_or_panic!(10.0), // premium_low
            pos_or_panic!(5.0),  // premium_middle
            Positive::ONE,       // premium_high
            pos_or_panic!(0.05), // open_fee_short_call
            pos_or_panic!(0.05), // close_fee_short_call
            pos_or_panic!(0.05), // open_fee_short_call_low
            pos_or_panic!(0.05), // close_fee_short_call_low
            pos_or_panic!(0.05), // open_fee_short_call_high
            pos_or_panic!(0.05), // close_fee_short_call_high
        )
    }

    #[test]
    fn test_underlying_price() {
        let short_butterfly = create_test_short();
        assert_eq!(short_butterfly.get_underlying_price(), &Positive::HUNDRED);
    }

    #[test]
    fn test_add_leg_short_butterfly() {
        let mut butterfly = create_test_short();
        let new_short = Position::new(
            Options::new(
                OptionType::European,
                Side::Short,
                "TEST".to_string(),
                pos_or_panic!(85.0),
                ExpirationDate::Days(pos_or_panic!(30.0)),
                pos_or_panic!(0.2),
                Positive::ONE,
                Positive::HUNDRED,
                dec!(0.05),
                OptionStyle::Call,
                Positive::ZERO,
                None,
            ),
            Positive::ONE,
            Utc::now(),
            Positive::ZERO,
            Positive::ZERO,
            None,
            None,
        );

        butterfly
            .add_position(&new_short.clone())
            .expect("Failed to add position");
        assert_eq!(
            butterfly.short_call_low.option.strike_price,
            pos_or_panic!(85.0)
        );
    }

    #[test]
    fn test_get_legs() {
        let short_butterfly = create_test_short();
        assert_eq!(short_butterfly.get_positions().unwrap().len(), 3);
    }

    #[test]
    fn test_max_loss_short_butterfly() {
        let butterfly = create_test_short();
        let max_loss = butterfly.get_max_loss().unwrap();
        // Max loss at middle strike
        let expected_loss = 9.4;
        assert_eq!(max_loss.to_f64(), expected_loss);
    }

    #[test]
    fn test_total_cost() {
        let short_butterfly = create_test_short();
        assert!(short_butterfly.get_total_cost().unwrap() > Positive::ZERO);
    }

    #[test]
    fn test_fees() {
        let butterfly = ShortButterflySpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::ONE,
            pos_or_panic!(3.0),
            Positive::TWO,
            Positive::ONE,
            Positive::ONE, // open_fee_short_call
            Positive::ONE, // close_fee_short_call
            Positive::ONE, // open_fee_short_call_low
            Positive::ONE, // close_fee_short_call_low
            Positive::ONE, // open_fee_short_call_high
            Positive::ONE, // close_fee_short_call_high
        );
        assert_eq!(butterfly.get_fees().unwrap().to_f64(), 8.0);
    }

    #[test]
    fn test_fees_bis() {
        let butterfly = ShortButterflySpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::TWO,
            pos_or_panic!(3.0),
            Positive::TWO,
            Positive::ONE,
            Positive::ONE, // open_fee_short_call
            Positive::ONE, // close_fee_short_call
            Positive::ONE, // open_fee_short_call_low
            Positive::ONE, // close_fee_short_call_low
            Positive::ONE, // open_fee_short_call_high
            Positive::ONE, // close_fee_short_call_high
        );

        assert_eq!(butterfly.get_fees().unwrap(), pos_or_panic!(16.0));
    }

    #[test]
    fn test_profit_area_short_butterfly() {
        let butterfly = create_test_short();
        let area = butterfly.get_profit_area().unwrap().to_f64().unwrap();
        assert!(area >= ZERO);
    }

    #[test]
    fn test_profit_ratio() {
        let short_butterfly = create_test_short();
        assert!(
            short_butterfly
                .get_profit_ratio()
                .unwrap()
                .to_f64()
                .unwrap()
                >= ZERO
        );
    }

    #[test]
    fn test_break_even_points() {
        let short_butterfly = create_test_short();
        assert_eq!(short_butterfly.get_break_even_points().unwrap().len(), 2);
    }

    #[test]
    fn test_profits_with_quantities() {
        let short_butterfly = ShortButterflySpread::new(
            "SP500".to_string(),
            pos_or_panic!(5781.88), // underlying_price
            pos_or_panic!(5700.0),  // short_strike_itm
            pos_or_panic!(5780.0),  // long_strike
            pos_or_panic!(5850.0),  // short_strike_otm
            ExpirationDate::Days(Positive::TWO),
            pos_or_panic!(0.18),   // implied_volatility
            dec!(0.05),            // risk_free_rate
            Positive::ZERO,        // dividend_yield
            pos_or_panic!(1.1),    // long quantity
            pos_or_panic!(119.01), // premium_long
            pos_or_panic!(66.0),   // premium_short
            pos_or_panic!(29.85),  // open_fee_long
            pos_or_panic!(0.05),
            pos_or_panic!(0.05),
            pos_or_panic!(0.05),
            pos_or_panic!(0.05),
            pos_or_panic!(0.05),
            pos_or_panic!(0.05),
        );
        assert_eq!(
            short_butterfly.get_max_profit().unwrap().to_f64(),
            pos_or_panic!(18.106)
        );
    }
}

#[cfg(test)]
mod tests_butterfly_optimizable {
    use super::*;

    use crate::model::ExpirationDate;

    use rust_decimal_macros::dec;

    fn create_test_option_chain() -> OptionChain {
        let mut chain = OptionChain::new(
            "TEST",
            Positive::HUNDRED,
            "2024-12-31".to_string(),
            None,
            None,
        );

        for strike in [85.0, 90.0, 95.0, 100.0, 105.0, 110.0, 115.0] {
            chain.add_option(
                pos_or_panic!(strike),
                spos!(5.0),         // call_bid
                spos!(5.2),         // call_ask
                spos!(5.0),         // put_bid
                spos!(5.2),         // put_ask
                pos_or_panic!(0.2), // implied_volatility
                Some(dec!(0.5)),    // delta
                Some(dec!(0.2)),
                Some(dec!(0.2)),
                spos!(100.0), // volume
                Some(50),     // open_interest
                None,
            );
        }
        chain
    }

    fn create_test_short() -> ShortButterflySpread {
        ShortButterflySpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::ONE,
            pos_or_panic!(3.0),
            Positive::TWO,
            Positive::ONE,
            pos_or_panic!(0.05), // open_fee_short_call
            pos_or_panic!(0.05), // close_fee_short_call
            pos_or_panic!(0.05), // open_fee_short_call_low
            pos_or_panic!(0.05), // close_fee_short_call_low
            pos_or_panic!(0.05), // open_fee_short_call_high
            pos_or_panic!(0.05), // close_fee_short_call_high
        )
    }

    #[test]
    fn test_find_optimal_area() {
        let mut butterfly = create_test_short();
        let chain = create_test_option_chain();
        let initial_area = butterfly.get_profit_area().unwrap();

        butterfly.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Area);

        assert!(butterfly.validate());
        assert!(butterfly.get_profit_area().unwrap() >= initial_area);
    }

    #[test]
    fn test_valid_strike_order() {
        let mut butterfly = create_test_short();
        let chain = create_test_option_chain();

        butterfly.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);

        assert!(
            butterfly.short_call_low.option.strike_price < butterfly.long_call.option.strike_price
        );
        assert!(
            butterfly.long_call.option.strike_price < butterfly.short_call_high.option.strike_price
        );
    }

    #[test]
    fn test_find_optimal_ratio_short() {
        let mut butterfly = create_test_short();
        let chain = create_test_option_chain();
        let initial_ratio = butterfly.get_profit_ratio().unwrap();

        butterfly.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);

        assert!(butterfly.validate());
        assert!(butterfly.get_profit_ratio().unwrap() >= initial_ratio);
    }

    #[test]
    fn test_find_optimal_area_short() {
        let mut butterfly = create_test_short();
        let chain = create_test_option_chain();
        let initial_area = butterfly.get_profit_area().unwrap();

        butterfly.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Area);

        assert!(butterfly.validate());
        assert!(butterfly.get_profit_area().unwrap() >= initial_area);
    }

    #[test]
    fn test_valid_strike_order_short() {
        let mut butterfly = create_test_short();
        let chain = create_test_option_chain();

        butterfly.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);

        assert!(
            butterfly.short_call_low.option.strike_price < butterfly.long_call.option.strike_price
        );
        assert!(
            butterfly.long_call.option.strike_price < butterfly.short_call_high.option.strike_price
        );
    }

    #[test]
    fn test_find_optimal_with_range() {
        let mut short_butterfly = create_test_short();
        let chain = create_test_option_chain();

        short_butterfly.find_optimal(
            &chain,
            FindOptimalSide::Range(pos_or_panic!(95.0), pos_or_panic!(105.0)),
            OptimizationCriteria::Ratio,
        );
        short_butterfly.find_optimal(
            &chain,
            FindOptimalSide::Range(pos_or_panic!(95.0), pos_or_panic!(105.0)),
            OptimizationCriteria::Ratio,
        );

        assert!(short_butterfly.long_call.option.strike_price >= pos_or_panic!(95.0));
        assert!(short_butterfly.long_call.option.strike_price <= pos_or_panic!(105.0));
        assert!(short_butterfly.long_call.option.strike_price >= pos_or_panic!(95.0));
        assert!(short_butterfly.long_call.option.strike_price <= pos_or_panic!(105.0));
    }
}

#[cfg(test)]
mod tests_butterfly_probability {
    use super::*;

    use crate::model::ExpirationDate;

    use rust_decimal_macros::dec;

    fn create_test_short() -> ShortButterflySpread {
        ShortButterflySpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::ONE,
            pos_or_panic!(10.0),
            Positive::TWO,
            Positive::ONE,
            pos_or_panic!(0.05), // open_fee_short_call
            pos_or_panic!(0.05), // close_fee_short_call
            pos_or_panic!(0.05), // open_fee_short_call_low
            pos_or_panic!(0.05), // close_fee_short_call_low
            pos_or_panic!(0.05), // open_fee_short_call_high
            pos_or_panic!(0.05), // close_fee_short_call_high
        )
    }

    mod short_butterfly_tests {
        use super::*;

        #[test]
        fn test_get_expiration() {
            let butterfly = create_test_short();
            let expiration_date = *butterfly.get_expiration().values().next().unwrap();
            assert_eq!(expiration_date, &ExpirationDate::Days(pos_or_panic!(30.0)));
        }

        #[test]
        fn test_get_risk_free_rate() {
            let butterfly = create_test_short();
            assert_eq!(
                **butterfly.get_risk_free_rate().values().next().unwrap(),
                dec!(0.05)
            );
        }

        #[test]
        fn test_get_profit_ranges() {
            let butterfly = ShortButterflySpread::new(
                "SP500".to_string(),
                pos_or_panic!(5781.88), // underlying_price
                pos_or_panic!(5700.0),  // short_strike_itm
                pos_or_panic!(5780.0),  // long_strike
                pos_or_panic!(5850.0),  // short_strike_otm
                ExpirationDate::Days(Positive::TWO),
                pos_or_panic!(0.18),   // implied_volatility
                dec!(0.05),            // risk_free_rate
                Positive::ZERO,        // dividend_yield
                pos_or_panic!(1.1),    // long quantity
                pos_or_panic!(119.01), // premium_long
                pos_or_panic!(66.0),   // premium_short
                pos_or_panic!(29.85),  // open_fee_long
                pos_or_panic!(0.05),
                pos_or_panic!(0.05),
                pos_or_panic!(0.05),
                pos_or_panic!(0.05),
                pos_or_panic!(0.05),
                pos_or_panic!(0.05),
            );
            let ranges = butterfly.get_profit_ranges().unwrap();

            assert!(ranges[0].upper_bound.is_some());
            assert!(ranges[1].lower_bound.is_some());
        }

        #[test]
        fn test_get_loss_ranges() {
            let butterfly = create_test_short();
            let result = butterfly.get_loss_ranges();
            assert!(result.is_ok());
            let ranges = result.unwrap();
            assert_eq!(ranges.len(), 1); // Long strangle has one loss range
            assert!(ranges[0].lower_bound.is_some());
            assert!(ranges[0].upper_bound.is_some());
        }
    }

    #[test]
    fn test_volatility_calculations() {
        let short_butterfly = create_test_short();
        let short_ranges = short_butterfly.get_profit_ranges().unwrap();
        assert!(!short_ranges.is_empty());
        assert!(short_ranges[0].probability > Positive::ZERO);
    }

    #[test]
    fn test_probability_sum() {
        let short_butterfly = create_test_short();
        let short_profit_ranges = short_butterfly.get_profit_ranges().unwrap();
        let short_loss_ranges = short_butterfly.get_loss_ranges().unwrap();
        let short_total_prob = short_profit_ranges
            .iter()
            .map(|r| r.probability.to_f64())
            .sum::<f64>()
            + short_loss_ranges
                .iter()
                .map(|r| r.probability.to_f64())
                .sum::<f64>();
        assert!((short_total_prob - 1.0).abs() < 0.1);

        let short_profit_ranges = short_butterfly.get_profit_ranges().unwrap();
        let short_loss_ranges = short_butterfly.get_loss_ranges().unwrap();
        let short_total_prob = short_profit_ranges
            .iter()
            .map(|r| r.probability.to_f64())
            .sum::<f64>()
            + short_loss_ranges
                .iter()
                .map(|r| r.probability.to_f64())
                .sum::<f64>();
        assert!((short_total_prob - 1.0).abs() < 0.1);
    }
}
