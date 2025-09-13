use super::base::{
    BreakEvenable, Optimizable, Positionable, Strategable, StrategyBasics, StrategyType, Validable,
};
use crate::{
    ExpirationDate, Options, Positive,
    chains::{StrategyLegs, chain::OptionChain, utils::OptionDataGroup},
    error::{
        GreeksError, OperationErrorKind,
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
    pos,
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
use pretty_simple_display::{DebugPretty, DisplaySimple};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use tracing::{debug, info};

/// The default description for the Long Butterfly Spread strategy.
pub const LONG_BUTTERFLY_DESCRIPTION: &str = "A long butterfly spread is created by buying one call at a lower strike price, \
    selling two calls at a middle strike price, and buying one call at a higher strike price, \
    all with the same expiration date. This strategy profits when the underlying price stays \
    near the middle strike price at expiration.";

/// Represents a Long Butterfly Spread options strategy.
///
/// A Long Butterfly Spread is created by combining three positions with the same expiration date:
/// buying a lower strike call, selling two middle strike calls, and buying a higher strike call.
/// This strategy has limited risk and limited profit potential, typically used when an investor
/// expects the underlying asset to have minimal volatility.
///
/// # Structure
///
/// The strategy consists of:
/// - A long call at a lower strike price
/// - Two short calls at a middle strike price
/// - A long call at a higher strike price
///
/// # Risk Profile
///
/// - Maximum profit occurs when the underlying price equals the middle strike price at expiration
/// - Limited risk, with maximum loss equal to the net premium paid
/// - Profitability range is constrained between the break-even points
///
/// # Attributes
#[derive(Clone, DebugPretty, DisplaySimple, Serialize, Deserialize)]
pub struct LongButterflySpread {
    /// Name identifier for the strategy
    pub name: String,
    /// The type classification of the strategy
    pub kind: StrategyType,
    /// Textual description of the strategy's purpose and characteristics
    pub description: String,
    /// Price points where the strategy transitions between profit and loss
    pub break_even_points: Vec<Positive>,
    /// The middle strike call positions that are sold (short)
    pub short_call: Position,
    /// The lower strike call position that is bought (long)
    pub long_call_low: Position,
    /// The higher strike call position that is bought (long)
    pub long_call_high: Position,
}

impl LongButterflySpread {
    /// Creates a new Long Butterfly Spread strategy with the specified parameters.
    ///
    /// A Long Butterfly Spread is created by buying one call at a lower strike price,
    /// selling two calls at a middle strike price, and buying one call at a higher strike price,
    /// all with the same expiration date. This strategy profits when the underlying price
    /// stays near the middle strike price at expiration.
    ///
    /// # Parameters
    /// * `underlying_symbol` - The ticker symbol of the underlying asset
    /// * `underlying_price` - The current market price of the underlying asset
    /// * `low_strike` - The strike price for the long call at the lower strike
    /// * `middle_strike` - The strike price for the two short calls
    /// * `high_strike` - The strike price for the long call at the higher strike
    /// * `expiration` - The expiration date for all options in the strategy
    /// * `implied_volatility` - The implied volatility used for option pricing
    /// * `risk_free_rate` - The risk-free interest rate used for option pricing
    /// * `dividend_yield` - The dividend yield of the underlying asset
    /// * `quantity` - The number of contracts for each position (note: middle position uses 2x this quantity)
    /// * `premium_low` - The premium paid for the long call at the lower strike
    /// * `premium_middle` - The premium received for each short call at the middle strike
    /// * `premium_high` - The premium paid for the long call at the higher strike
    /// * `open_fee_long_call` - Transaction fee for opening the short call positions
    /// * `close_fee_long_call` - Transaction fee for closing the short call positions
    /// * `open_fee_long_call_low` - Transaction fee for opening the lower strike long call
    /// * `close_fee_long_call_low` - Transaction fee for closing the lower strike long call
    /// * `open_fee_long_call_high` - Transaction fee for opening the higher strike long call
    /// * `close_fee_long_call_high` - Transaction fee for closing the higher strike long call
    ///
    /// # Returns
    /// A fully configured Long Butterfly Spread strategy with positions and break-even points calculated
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
        open_fee_short_call: Positive,
        close_fee_short_call: Positive,
        open_fee_long_call_low: Positive,
        close_fee_long_call_low: Positive,
        open_fee_long_call_high: Positive,
        close_fee_long_call_high: Positive,
    ) -> Self {
        let mut strategy = LongButterflySpread {
            name: "Long Butterfly".to_string(),
            kind: StrategyType::LongButterflySpread,
            description: LONG_BUTTERFLY_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call_low: Position::default(),
            short_call: Position::default(),
            long_call_high: Position::default(),
        };

        // Create two short calls at middle strike
        let long_calls = Options::new(
            OptionType::European,
            Side::Short,
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
        strategy.short_call = Position::new(
            long_calls,
            premium_middle,
            Utc::now(),
            open_fee_short_call,
            close_fee_short_call,
            None,
            None,
        );

        // Create long call at lower strike
        let long_call_low = Options::new(
            OptionType::European,
            Side::Long,
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
        strategy.long_call_low = Position::new(
            long_call_low,
            premium_low,
            Utc::now(),
            open_fee_long_call_low,
            close_fee_long_call_low,
            None,
            None,
        );

        // Create long call at higher strike
        let long_call_high = Options::new(
            OptionType::European,
            Side::Long,
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
        strategy.long_call_high = Position::new(
            long_call_high,
            premium_high,
            Utc::now(),
            open_fee_long_call_high,
            close_fee_long_call_high,
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

impl StrategyConstructor for LongButterflySpread {
    fn get_strategy(vec_positions: &[Position]) -> Result<Self, StrategyError> {
        // Long Butterfly Spread requires exactly 3 options
        if vec_positions.len() != 3 {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Long Butterfly Spread get_strategy".to_string(),
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
                    operation: "Long Butterfly Spread get_strategy".to_string(),
                    reason: "Options must be calls".to_string(),
                },
            ));
        }

        // Validate option configuration for Long Butterfly
        if lower_strike_position.option.side != Side::Long
            || middle_strike_position.option.side != Side::Short
            || higher_strike_position.option.side != Side::Long
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Long Butterfly Spread get_strategy".to_string(),
                    reason: "Long Butterfly requires long lower and higher strikes with a short middle strike".to_string(),
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
                    operation: "Long Butterfly Spread get_strategy".to_string(),
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
                    operation: "Long Butterfly Spread get_strategy".to_string(),
                    reason: "Options must have the same expiration date".to_string(),
                },
            ));
        }

        // Create strategy
        let strategy = LongButterflySpread {
            name: "Long Butterfly Spread".to_string(),
            kind: StrategyType::LongButterflySpread,
            description: LONG_BUTTERFLY_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call: Position::new(
                middle_strike_position.option.clone(),
                middle_strike_position.premium,
                Utc::now(),
                middle_strike_position.open_fee,
                middle_strike_position.close_fee,
                middle_strike_position.epic.clone(),
                middle_strike_position.extra_fields.clone(),
            ),
            long_call_low: Position::new(
                lower_strike_position.option.clone(),
                lower_strike_position.premium,
                Utc::now(),
                lower_strike_position.open_fee,
                lower_strike_position.close_fee,
                lower_strike_position.epic.clone(),
                lower_strike_position.extra_fields.clone(),
            ),
            long_call_high: Position::new(
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

impl BreakEvenable for LongButterflySpread {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        let left_net_value = self.calculate_profit_at(&self.long_call_low.option.strike_price)?
            / self.long_call_low.option.quantity;

        let right_net_value = self.calculate_profit_at(&self.long_call_high.option.strike_price)?
            / self.long_call_high.option.quantity;

        if left_net_value <= Decimal::ZERO {
            self.break_even_points
                .push((self.long_call_low.option.strike_price - left_net_value).round_to(2));
        }

        if right_net_value <= Decimal::ZERO {
            self.break_even_points
                .push((self.long_call_high.option.strike_price + right_net_value).round_to(2));
        }

        self.break_even_points.sort();
        Ok(())
    }
}

impl Validable for LongButterflySpread {
    fn validate(&self) -> bool {
        if !self.long_call_low.validate() {
            debug!("Long call (low strike) is invalid");
            return false;
        }
        if !self.short_call.validate() {
            debug!("Short calls (middle strike) are invalid");
            return false;
        }
        if !self.long_call_high.validate() {
            debug!("Long call (high strike) is invalid");
            return false;
        }

        if self.long_call_low.option.strike_price >= self.short_call.option.strike_price {
            debug!("Low strike must be lower than middle strike");
            return false;
        }
        if self.short_call.option.strike_price >= self.long_call_high.option.strike_price {
            debug!("Middle strike must be lower than high strike");
            return false;
        }

        if self.short_call.option.quantity != self.long_call_low.option.quantity * 2.0 {
            debug!("Middle strike quantity must be double the wing quantities");
            return false;
        }
        if self.long_call_low.option.quantity != self.long_call_high.option.quantity {
            debug!("Wing quantities must be equal");
            return false;
        }

        true
    }
}

impl Positionable for LongButterflySpread {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match &position.option.side {
            Side::Long => {
                // long_calls should be inserted first
                if position.option.strike_price < self.short_call.option.strike_price {
                    self.long_call_low = position.clone();
                    Ok(())
                } else {
                    self.long_call_high = position.clone();
                    Ok(())
                }
            }
            Side::Short => {
                self.short_call = position.clone();
                Ok(())
            }
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![
            &self.long_call_low,
            &self.short_call,
            &self.long_call_high,
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
            (Side::Short, OptionStyle::Call, strike)
                if *strike == self.short_call.option.strike_price =>
            {
                Ok(vec![&mut self.short_call])
            }

            (_, OptionStyle::Put, _) => Err(PositionError::invalid_position_type(
                *side,
                "Put not found in positions".to_string(),
            )),
            (Side::Long, OptionStyle::Call, strike)
                if *strike == self.long_call_low.option.strike_price =>
            {
                Ok(vec![&mut self.long_call_low])
            }
            (Side::Long, OptionStyle::Call, strike)
                if *strike == self.long_call_high.option.strike_price =>
            {
                Ok(vec![&mut self.long_call_high])
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
            (Side::Short, OptionStyle::Call, strike)
                if *strike == self.short_call.option.strike_price =>
            {
                self.short_call = position.clone();
            }

            (_, OptionStyle::Put, _) => {
                return Err(PositionError::invalid_position_type(
                    position.option.side,
                    "Put not found in positions".to_string(),
                ));
            }
            (Side::Long, OptionStyle::Call, strike)
                if *strike == self.long_call_low.option.strike_price =>
            {
                self.long_call_low = position.clone();
            }
            (Side::Long, OptionStyle::Call, strike)
                if *strike == self.long_call_high.option.strike_price =>
            {
                self.long_call_high = position.clone();
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

impl Strategable for LongButterflySpread {
    fn info(&self) -> Result<StrategyBasics, StrategyError> {
        Ok(StrategyBasics {
            name: self.name.clone(),
            kind: self.kind.clone(),
            description: self.description.clone(),
        })
    }
}

impl BasicAble for LongButterflySpread {
    fn get_title(&self) -> String {
        let strategy_title = format!("{:?} Strategy: ", self.kind);
        let leg_titles: Vec<String> = [
            self.long_call_low.get_title(),
            self.short_call.get_title(),
            self.long_call_high.get_title(),
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
        let long_call_low = &self.long_call_low.option;
        let short_call = &self.short_call.option;
        let long_call_high = &self.long_call_high.option;

        hash_set.insert(OptionBasicType {
            option_style: &long_call_low.option_style,
            side: &long_call_low.side,
            strike_price: &long_call_low.strike_price,
            expiration_date: &long_call_low.expiration_date,
        });
        hash_set.insert(OptionBasicType {
            option_style: &short_call.option_style,
            side: &short_call.side,
            strike_price: &short_call.strike_price,
            expiration_date: &short_call.expiration_date,
        });
        hash_set.insert(OptionBasicType {
            option_style: &long_call_high.option_style,
            side: &long_call_high.side,
            strike_price: &long_call_high.strike_price,
            expiration_date: &long_call_high.expiration_date,
        });

        hash_set
    }
    fn get_implied_volatility(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let options = [
            (
                &self.long_call_low.option,
                &self.long_call_low.option.implied_volatility,
            ),
            (
                &self.short_call.option,
                &self.short_call.option.implied_volatility,
            ),
            (
                &self.long_call_high.option,
                &self.long_call_high.option.implied_volatility,
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
                &self.long_call_low.option,
                &self.long_call_low.option.quantity,
            ),
            (&self.short_call.option, &self.short_call.option.quantity),
            (
                &self.long_call_high.option,
                &self.long_call_high.option.quantity,
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
        self.short_call.one_option()
    }
    fn one_option_mut(&mut self) -> &mut Options {
        self.short_call.one_option_mut()
    }
    fn set_expiration_date(
        &mut self,
        expiration_date: ExpirationDate,
    ) -> Result<(), StrategyError> {
        self.long_call_low.option.expiration_date = expiration_date;
        self.short_call.option.expiration_date = expiration_date;
        self.long_call_high.option.expiration_date = expiration_date;
        Ok(())
    }
    fn set_underlying_price(&mut self, price: &Positive) -> Result<(), StrategyError> {
        self.long_call_low.option.underlying_price = *price;
        self.long_call_low.premium = Positive::from(
            self.long_call_low
                .option
                .calculate_price_black_scholes()?
                .abs(),
        );
        self.short_call.option.underlying_price = *price;
        self.short_call.premium = Positive::from(
            self.short_call
                .option
                .calculate_price_black_scholes()?
                .abs(),
        );
        self.long_call_high.option.underlying_price = *price;
        self.long_call_high.premium = Positive::from(
            self.long_call_high
                .option
                .calculate_price_black_scholes()?
                .abs(),
        );
        Ok(())
    }
    fn set_implied_volatility(&mut self, volatility: &Positive) -> Result<(), StrategyError> {
        self.long_call_low.option.implied_volatility = *volatility;
        self.short_call.option.implied_volatility = *volatility;
        self.long_call_high.option.implied_volatility = *volatility;

        self.long_call_low.premium = Positive(
            self.long_call_low
                .option
                .calculate_price_black_scholes()?
                .abs(),
        );
        self.short_call.premium = Positive(
            self.short_call
                .option
                .calculate_price_black_scholes()?
                .abs(),
        );
        self.long_call_high.premium = Positive(
            self.long_call_high
                .option
                .calculate_price_black_scholes()?
                .abs(),
        );
        Ok(())
    }
}

impl Strategies for LongButterflySpread {
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        let profit = self.calculate_profit_at(&self.short_call.option.strike_price)?;
        if profit > Decimal::ZERO {
            Ok(profit.into())
        } else {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxProfitError {
                    reason: "max_profit is negative".to_string(),
                },
            ))
        }
    }
    fn get_max_loss(&self) -> Result<Positive, StrategyError> {
        let left_loss = self.calculate_profit_at(&self.long_call_low.option.strike_price)?;
        let right_loss = self.calculate_profit_at(&self.long_call_high.option.strike_price)?;
        let max_loss = left_loss.min(right_loss);
        if max_loss > Decimal::ZERO {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxLossError {
                    reason: "Max loss is negative".to_string(),
                },
            ))
        } else {
            Ok(max_loss.abs().into())
        }
    }
    fn get_profit_area(&self) -> Result<Decimal, StrategyError> {
        let high = self.get_max_profit().unwrap_or(Positive::ZERO);
        let break_even_points = self.get_break_even_points()?;

        let base = if break_even_points.len() == 2 {
            break_even_points[1] - break_even_points[0]
        } else {
            let break_even_point = break_even_points[0];

            if break_even_point < self.short_call.option.strike_price {
                self.calculate_profit_at(&self.long_call_high.option.strike_price)?
                    .into()
            } else {
                self.calculate_profit_at(&self.long_call_low.option.strike_price)?
                    .into()
            }
        };
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

impl Optimizable for LongButterflySpread {
    type Strategy = LongButterflySpread;

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
            .filter(move |(long_low, short, long_high)| {
                if side == FindOptimalSide::Center {
                    let atm_strike = match option_chain.atm_strike() {
                        Ok(atm_strike) => atm_strike,
                        _ => return false,
                    };
                    long_low.is_valid_optimal_side(underlying_price, &FindOptimalSide::Lower)
                        && short.is_valid_optimal_side(
                            underlying_price,
                            &FindOptimalSide::Range(*atm_strike, *atm_strike),
                        )
                        && long_high
                            .is_valid_optimal_side(underlying_price, &FindOptimalSide::Upper)
                } else {
                    long_low.is_valid_optimal_side(underlying_price, &side)
                        && short.is_valid_optimal_side(underlying_price, &side)
                        && long_high.is_valid_optimal_side(underlying_price, &side)
                }
            })
            .filter(move |(long_low, short, long_high)| {
                long_low.strike_price < short.strike_price
                    && short.strike_price < long_high.strike_price
            })
            // Filter out options with invalid bid/ask prices
            .filter(|(long_low, short, long_high)| {
                long_low.call_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && short.call_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && long_high.call_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
            })
            // Filter out options that don't meet strategy constraints
            .filter(move |(long_low, short, long_high)| {
                let legs = StrategyLegs::ThreeLegs {
                    first: long_low,
                    second: short,
                    third: long_high,
                };
                let strategy = strategy.create_strategy(option_chain, &legs);
                strategy.validate()
                    && strategy.get_max_profit().is_ok()
                    && strategy.get_max_loss().is_ok()
            })
            // Map to OptionDataGroup
            .map(move |(long_low, short, long_high)| {
                OptionDataGroup::Three(long_low, short, long_high)
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
            let (long_low, short, long_high) = match option_data_group {
                OptionDataGroup::Three(first, second, third) => (first, second, third),
                _ => panic!("Invalid OptionDataGroup"),
            };

            let legs = StrategyLegs::ThreeLegs {
                first: long_low,
                second: short,
                third: long_high,
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

                LongButterflySpread::new(
                    chain.symbol.clone(),
                    chain.underlying_price,
                    low_strike.strike_price,
                    middle_strike.strike_price,
                    high_strike.strike_price,
                    self.long_call_low.option.expiration_date,
                    implied_volatility,
                    self.long_call_low.option.risk_free_rate,
                    self.long_call_low.option.dividend_yield,
                    self.long_call_low.option.quantity,
                    low_strike.call_ask.unwrap(),
                    middle_strike.call_bid.unwrap(),
                    high_strike.call_ask.unwrap(),
                    self.short_call.open_fee,
                    self.short_call.close_fee,
                    self.long_call_low.open_fee,
                    self.long_call_low.close_fee,
                    self.long_call_high.open_fee,
                    self.long_call_high.close_fee,
                )
            }
            _ => panic!("Invalid number of legs for Long Butterfly strategy"),
        }
    }
}

impl Profit for LongButterflySpread {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, Box<dyn Error>> {
        let price = Some(price);
        Ok(self.long_call_low.pnl_at_expiration(&price)?
            + self.short_call.pnl_at_expiration(&price)?
            + self.long_call_high.pnl_at_expiration(&price)?)
    }
}

impl ProbabilityAnalysis for LongButterflySpread {
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_points = self.get_break_even_points()?;
        let option = &self.short_call.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            self.long_call_low.option.implied_volatility,
            self.short_call.option.implied_volatility,
            self.long_call_high.option.implied_volatility,
        ]);

        let mut profit_range = ProfitLossRange::new(
            Some(break_even_points[0]),
            Some(break_even_points[1]),
            pos!(self.get_max_profit()?.to_f64()),
        )?;

        profit_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            expiration_date,
            Some(risk_free_rate),
        )?;

        Ok(vec![profit_range])
    }

    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let mut ranges = Vec::new();
        let break_even_points = self.get_break_even_points()?;
        let option = &self.short_call.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            self.long_call_low.option.implied_volatility,
            self.short_call.option.implied_volatility,
            self.long_call_high.option.implied_volatility,
        ]);

        let volatility_adjustment = Some(VolatilityAdjustment {
            base_volatility: mean_volatility,
            std_dev_adjustment: std_dev,
        });

        let mut lower_loss_range = ProfitLossRange::new(
            Some(self.long_call_low.option.strike_price),
            Some(break_even_points[0]),
            pos!(self.get_max_loss()?.to_f64()),
        )?;

        lower_loss_range.calculate_probability(
            self.get_underlying_price(),
            volatility_adjustment.clone(),
            None,
            expiration_date,
            Some(risk_free_rate),
        )?;

        ranges.push(lower_loss_range);

        let mut upper_loss_range = ProfitLossRange::new(
            Some(break_even_points[1]),
            Some(self.long_call_high.option.strike_price),
            pos!(self.get_max_loss()?.to_f64()),
        )?;

        upper_loss_range.calculate_probability(
            self.get_underlying_price(),
            volatility_adjustment,
            None,
            expiration_date,
            Some(risk_free_rate),
        )?;

        ranges.push(upper_loss_range);

        Ok(ranges)
    }
}

impl Greeks for LongButterflySpread {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![
            &self.long_call_low.option,
            &self.short_call.option,
            &self.long_call_high.option,
        ])
    }
}

impl DeltaNeutrality for LongButterflySpread {}

impl PnLCalculator for LongButterflySpread {
    fn calculate_pnl(
        &self,
        market_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        Ok(self
            .short_call
            .calculate_pnl(market_price, expiration_date, implied_volatility)?
            + self.long_call_low.calculate_pnl(
                market_price,
                expiration_date,
                implied_volatility,
            )?
            + self.long_call_high.calculate_pnl(
                market_price,
                expiration_date,
                implied_volatility,
            )?)
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        Ok(self
            .short_call
            .calculate_pnl_at_expiration(underlying_price)?
            + self
                .long_call_low
                .calculate_pnl_at_expiration(underlying_price)?
            + self
                .long_call_high
                .calculate_pnl_at_expiration(underlying_price)?)
    }
}

test_strategy_traits!(LongButterflySpread, test_short_call_implementations);

#[cfg(test)]
mod tests_long_butterfly_spread {
    use super::*;
    use crate::model::ExpirationDate;
    use crate::pos;
    use rust_decimal_macros::dec;

    fn create_test_butterfly() -> LongButterflySpread {
        LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),                      // underlying_price
            pos!(90.0),                       // low_strike
            pos!(100.0),                      // middle_strike
            pos!(110.0),                      // high_strike
            ExpirationDate::Days(pos!(30.0)), // expiration
            pos!(0.2),                        // implied_volatility
            dec!(0.05),                       // risk_free_rate
            Positive::ZERO,                   // dividend_yield
            pos!(1.0),                        // quantity
            pos!(3.0),                        // premium_low
            Positive::TWO,                    // premium_middle
            Positive::ONE,                    // premium_high
            pos!(0.05),                       // open_fee_long_call
            pos!(0.05),                       // close_fee_long_call
            pos!(0.05),                       // open_fee_long_call_low
            pos!(0.05),                       // close_fee_long_call_low
            pos!(0.05),                       // open_fee_long_call_high
            pos!(0.05),                       // close_fee_long_call_high
        )
    }

    #[test]
    fn test_new_butterfly_basic_properties() {
        let butterfly = create_test_butterfly();

        assert_eq!(butterfly.name, "Long Butterfly");
        assert_eq!(butterfly.kind, StrategyType::LongButterflySpread);
        assert!(!butterfly.description.is_empty());
        assert!(butterfly.description.contains("long butterfly spread"));
    }

    #[test]
    fn test_butterfly_strikes() {
        let butterfly = create_test_butterfly();

        assert_eq!(butterfly.long_call_low.option.strike_price, pos!(90.0));
        assert_eq!(butterfly.short_call.option.strike_price, pos!(100.0));
        assert_eq!(butterfly.long_call_high.option.strike_price, pos!(110.0));
    }

    #[test]
    fn test_butterfly_quantities() {
        let butterfly = create_test_butterfly();

        assert_eq!(butterfly.long_call_low.option.quantity, pos!(1.0));
        assert_eq!(butterfly.short_call.option.quantity, pos!(2.0)); // Double quantity
        assert_eq!(butterfly.long_call_high.option.quantity, pos!(1.0));
    }

    #[test]
    fn test_butterfly_sides() {
        let butterfly = create_test_butterfly();

        assert_eq!(butterfly.long_call_low.option.side, Side::Long);
        assert_eq!(butterfly.short_call.option.side, Side::Short);
        assert_eq!(butterfly.long_call_high.option.side, Side::Long);
    }

    #[test]
    fn test_butterfly_option_styles() {
        let butterfly = create_test_butterfly();

        assert_eq!(
            butterfly.long_call_low.option.option_style,
            OptionStyle::Call
        );
        assert_eq!(butterfly.short_call.option.option_style, OptionStyle::Call);
        assert_eq!(
            butterfly.long_call_high.option.option_style,
            OptionStyle::Call
        );
    }

    #[test]
    fn test_butterfly_expiration_consistency() {
        let butterfly = create_test_butterfly();
        let expiration = ExpirationDate::Days(pos!(30.0));

        assert_eq!(
            format!("{:?}", butterfly.long_call_low.option.expiration_date),
            format!("{:?}", expiration)
        );
        assert_eq!(
            format!("{:?}", butterfly.short_call.option.expiration_date),
            format!("{:?}", expiration)
        );
        assert_eq!(
            format!("{:?}", butterfly.long_call_high.option.expiration_date),
            format!("{:?}", expiration)
        );
    }

    #[test]
    fn test_butterfly_fees_distribution() {
        let butterfly = LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(3.0),
            Positive::TWO,
            Positive::ONE,
            Positive::ONE, // open_fee_long_call
            pos!(0.05),    // close_fee_long_call
            pos!(0.05),    // open_fee_long_call_low
            pos!(0.05),    // close_fee_long_call_low
            Positive::ONE, // open_fee_long_call_high
            pos!(0.05),    // close_fee_long_call_high
        );

        assert_eq!(butterfly.long_call_low.open_fee, 0.05); // fees / 3
        assert_eq!(butterfly.short_call.open_fee, 1.0); // fees / 3
        assert_eq!(butterfly.long_call_high.open_fee, 1.0); // fees / 3
    }

    #[test]
    fn test_butterfly_break_even_points() {
        let butterfly = create_test_butterfly();
        let break_even_points = butterfly.break_even_points;

        assert_eq!(break_even_points.len(), 2);
        assert!(break_even_points[0] > butterfly.long_call_low.option.strike_price);
        assert!(break_even_points[0] < butterfly.short_call.option.strike_price);
        assert!(break_even_points[1] > butterfly.short_call.option.strike_price);
        assert!(break_even_points[1] < butterfly.long_call_high.option.strike_price);
    }

    #[test]
    fn test_butterfly_with_different_quantities() {
        let butterfly = LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(2.0), // quantity = 2
            pos!(3.0),
            Positive::TWO,
            Positive::ONE,
            pos!(0.05), // open_fee_long_call
            pos!(0.05), // close_fee_long_call
            pos!(0.05), // open_fee_long_call_low
            pos!(0.05), // close_fee_long_call_low
            pos!(0.05), // open_fee_long_call_high
            pos!(0.05), // close_fee_long_call_high
        );

        assert_eq!(butterfly.long_call_low.option.quantity, pos!(2.0));
        assert_eq!(butterfly.short_call.option.quantity, pos!(4.0)); // 2 * 2
        assert_eq!(butterfly.long_call_high.option.quantity, pos!(2.0));
    }

    #[test]
    fn test_butterfly_with_symmetric_strikes() {
        let butterfly = create_test_butterfly();

        let lower_width =
            butterfly.short_call.option.strike_price - butterfly.long_call_low.option.strike_price;
        let upper_width =
            butterfly.long_call_high.option.strike_price - butterfly.short_call.option.strike_price;

        assert_eq!(lower_width, upper_width);
    }

    #[test]
    fn test_butterfly_with_equal_implied_volatility() {
        let butterfly = create_test_butterfly();

        assert_eq!(
            butterfly.long_call_low.option.implied_volatility,
            butterfly.short_call.option.implied_volatility
        );
        assert_eq!(
            butterfly.short_call.option.implied_volatility,
            butterfly.long_call_high.option.implied_volatility
        );
    }

    #[test]
    fn test_butterfly_with_invalid_premiums() {
        let check_profit = LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
            pos!(1.05),  // open_fee_long_call
            pos!(10.05), // close_fee_long_call
            pos!(1.05),  // open_fee_long_call_low
            pos!(0.05),  // close_fee_long_call_low
            pos!(1.05),  // open_fee_long_call_high
            pos!(0.05),  // close_fee_long_call_high
        );
        assert!(check_profit.get_max_profit().is_err());
    }
}

#[cfg(test)]
mod tests_long_butterfly_validation {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_valid_position(side: Side, strike_price: Positive, quantity: Positive) -> Position {
        Position::new(
            Options::new(
                OptionType::European,
                side,
                "TEST".to_string(),
                strike_price,
                ExpirationDate::Days(pos!(30.0)),
                pos!(0.2),
                quantity,
                pos!(100.0),
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
    fn test_valid_long_butterfly() {
        let butterfly = LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            Positive::ONE,
            Positive::TWO,
            Positive::ONE,
            pos!(0.05), // open_fee_long_call
            pos!(0.05), // close_fee_long_call
            pos!(0.05), // open_fee_long_call_low
            pos!(0.05), // close_fee_long_call_low
            pos!(0.05), // open_fee_long_call_high
            pos!(0.05), // close_fee_long_call_high
        );
        assert!(butterfly.validate());
    }

    #[test]
    fn test_invalid_long_call_low() {
        let mut butterfly = LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            Positive::ONE,
            Positive::TWO,
            Positive::ONE,
            pos!(0.05), // open_fee_long_call
            pos!(0.05), // close_fee_long_call
            pos!(0.05), // open_fee_long_call_low
            pos!(0.05), // close_fee_long_call_low
            pos!(0.05), // open_fee_long_call_high
            pos!(0.05), // close_fee_long_call_high
        );
        butterfly.long_call_low = create_valid_position(Side::Long, pos!(90.0), Positive::ZERO);
        assert!(!butterfly.validate());
    }

    #[test]
    fn test_invalid_strike_order_low() {
        let butterfly = LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(100.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            Positive::ONE,
            Positive::TWO,
            Positive::ONE,
            pos!(0.05), // open_fee_long_call
            pos!(0.05), // close_fee_long_call
            pos!(0.05), // open_fee_long_call_low
            pos!(0.05), // close_fee_long_call_low
            pos!(0.05), // open_fee_long_call_high
            pos!(0.05), // close_fee_long_call_high
        );
        assert!(!butterfly.validate());
    }

    #[test]
    fn test_invalid_quantities() {
        let mut butterfly = LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            Positive::ONE,
            Positive::TWO,
            Positive::ONE,
            pos!(0.05), // open_fee_long_call
            pos!(0.05), // close_fee_long_call
            pos!(0.05), // open_fee_long_call_low
            pos!(0.05), // close_fee_long_call_low
            pos!(0.05), // open_fee_long_call_high
            pos!(0.05), // close_fee_long_call_high
        );
        butterfly.short_call = create_valid_position(Side::Short, pos!(100.0), pos!(1.0));
        assert!(!butterfly.validate());
    }

    #[test]
    fn test_unequal_wing_quantities() {
        let mut butterfly = LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            Positive::ONE,
            Positive::TWO,
            Positive::ONE,
            pos!(0.05), // open_fee_long_call
            pos!(0.05), // close_fee_long_call
            pos!(0.05), // open_fee_long_call_low
            pos!(0.05), // close_fee_long_call_low
            pos!(0.05), // open_fee_long_call_high
            pos!(0.05), // close_fee_long_call_high
        );
        butterfly.long_call_high = create_valid_position(Side::Long, pos!(110.0), pos!(2.0));
        assert!(!butterfly.validate());
    }
}

#[cfg(test)]
mod tests_long_butterfly_profit {
    use super::*;
    use crate::constants::ZERO;
    use crate::model::ExpirationDate;
    use crate::pos;
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::str::FromStr;

    fn create_test() -> LongButterflySpread {
        LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(90.0),  // low_strike
            pos!(100.0), // middle_strike
            pos!(110.0), // high_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(3.0),      // premium_low
            Positive::TWO,  // premium_middle
            Positive::ONE,  // premium_high
            pos!(0.05),     // open_fee_long_call
            pos!(0.05),     // close_fee_long_call
            pos!(0.05),     // open_fee_long_call_low
            pos!(0.05),     // close_fee_long_call_low
            pos!(0.05),     // open_fee_long_call_high
            pos!(0.05),     // close_fee_long_call_high
        )
    }

    #[test]
    fn test_profit_at_middle_strike() {
        let butterfly = create_test();
        let profit = butterfly.calculate_profit_at(&pos!(100.0)).unwrap();
        assert!(profit > Decimal::ZERO);
        let expected = Positive::new_decimal(Decimal::from_str("9.6").unwrap()).unwrap();
        assert_eq!(profit, expected);
    }

    #[test]
    fn test_profit_below_lowest_strike() {
        let butterfly = create_test();
        let profit = butterfly
            .calculate_profit_at(&pos!(85.0))
            .unwrap()
            .to_f64()
            .unwrap();
        assert!(profit < ZERO);
        let expected = 0.4;
        assert_relative_eq!(-profit, expected, epsilon = 0.0001);
    }

    #[test]
    fn test_profit_above_highest_strike() {
        let butterfly = create_test();
        let profit = butterfly.calculate_profit_at(&pos!(115.0)).unwrap();
        assert!(profit < Decimal::ZERO);
        assert_relative_eq!(
            profit.to_f64().unwrap(),
            -butterfly.get_max_loss().unwrap().to_f64(),
            epsilon = 0.0001
        );
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
        let butterfly = LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(90.0),  // low_strike
            pos!(100.0), // middle_strike
            pos!(110.0), // high_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(2.0),      // quantity = 2
            pos!(3.0),      // premium_low
            Positive::TWO,  // premium_middle
            Positive::ONE,  // premium_high
            pos!(0.05),     // open_fee_long_call
            pos!(0.05),     // close_fee_long_call
            pos!(0.05),     // open_fee_long_call_low
            pos!(0.05),     // close_fee_long_call_low
            pos!(0.05),     // open_fee_long_call_high
            pos!(0.05),     // close_fee_long_call_high
        );

        let scaled_profit = butterfly
            .calculate_profit_at(&pos!(100.0))
            .unwrap()
            .to_f64()
            .unwrap();
        assert_relative_eq!(scaled_profit, 19.2, epsilon = 0.0001);
    }
}

#[cfg(test)]
mod tests_long_butterfly_delta {
    use super::*;
    use crate::model::types::OptionStyle;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::strategies::long_butterfly_spread::LongButterflySpread;
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};

    use rust_decimal_macros::dec;

    fn get_strategy(underlying_price: Positive) -> LongButterflySpread {
        LongButterflySpread::new(
            "SP500".to_string(),
            underlying_price, // underlying_price
            pos!(5710.0),     // long_strike_itm
            pos!(5820.0),     // long_strike
            pos!(6100.0),     // long_strike_otm
            ExpirationDate::Days(pos!(2.0)),
            pos!(0.18),     // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // long quantity
            pos!(49.65),    // premium_long
            pos!(42.93),    // premium_short
            Positive::ONE,  // open_fee_long
            pos!(0.05),     // open_fee_long_call
            pos!(0.05),     // close_fee_long_call
            pos!(0.05),     // open_fee_long_call_low
            pos!(0.05),     // close_fee_long_call_low
            pos!(0.05),     // open_fee_long_call_high
            pos!(0.05),     // close_fee_long_call_high
        )
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos!(5881.88));
        let size = dec!(-0.5970615569);
        let delta1 = pos!(0.60439151471911);
        let delta2 = pos!(175.125_739_348_840_2);
        let k1 = pos!(5710.0);
        let k2 = pos!(6100.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        let suggestion_zero = binding.first().unwrap();
        let suggestion_one = binding.last().unwrap();
        match suggestion_zero {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta1, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k1, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Long);
            }
            _ => panic!("Invalid suggestion"),
        }

        match suggestion_one {
            DeltaAdjustment::BuyOptions {
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

        let mut option = strategy.long_call_low.option.clone();
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
    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(pos!(5710.81));
        let size = dec!(0.3518);
        let delta = pos!(4.310_394_079_825_43);
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
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.short_call.option.clone();
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
        let strategy = get_strategy(pos!(5420.0));

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
mod tests_long_butterfly_delta_size {
    use super::*;
    use crate::model::types::OptionStyle;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::strategies::long_butterfly_spread::LongButterflySpread;
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::str::FromStr;

    fn get_strategy(underlying_price: Positive) -> LongButterflySpread {
        LongButterflySpread::new(
            "SP500".to_string(),
            underlying_price, // underlying_price
            pos!(5710.0),     // long_strike_itm
            pos!(5820.0),     // short_strike
            pos!(6100.0),     // long_strike_otm
            ExpirationDate::Days(pos!(2.0)),
            pos!(0.18),     // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(3.0),      // long quantity
            pos!(49.65),    // premium_long
            pos!(42.93),    // premium_short
            Positive::ONE,  // open_fee_long
            pos!(0.05),     // open_fee_long_call
            pos!(0.05),     // close_fee_long_call
            pos!(0.05),     // open_fee_long_call_low
            pos!(0.05),     // close_fee_long_call_low
            pos!(0.05),     // open_fee_long_call_high
            pos!(0.05),     // close_fee_long_call_high
        )
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos!(5881.85));
        let size = dec!(-1.7905);
        let delta1 = pos!(1.812583011030011);
        let delta2 = pos!(525.8051045358664);
        let k1 = pos!(5710.0);
        let k2 = pos!(6100.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        let suggestion_zero = binding.first().unwrap();
        let suggestion_one = binding.last().unwrap();
        match suggestion_zero {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta1, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k1, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Long);
            }
            _ => panic!("Invalid suggestion"),
        }

        match suggestion_one {
            DeltaAdjustment::BuyOptions {
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

        let mut option = strategy.long_call_low.option.clone();
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
    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(pos!(5710.88));
        let size = dec!(1.0558);
        let delta =
            Positive::new_decimal(Decimal::from_str("12.912467384337744").unwrap()).unwrap();
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
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.short_call.option.clone();
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
        let strategy = get_strategy(pos!(5410.0));

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
mod tests_long_butterfly_position_management {
    use super::*;
    use crate::error::position::PositionValidationErrorKind;
    use crate::model::types::{OptionStyle, Side};
    use crate::pos;
    use rust_decimal_macros::dec;
    use tracing::error;

    fn create_test_butterfly() -> LongButterflySpread {
        LongButterflySpread::new(
            "SP500".to_string(),
            pos!(5795.88), // underlying_price
            pos!(5710.0),  // long_strike_itm
            pos!(5780.0),  // short_strike
            pos!(5850.0),  // long_strike_otm
            ExpirationDate::Days(pos!(2.0)),
            pos!(0.18),     // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(2.0),      // long quantity
            pos!(113.3),    // premium_long_low
            pos!(64.20),    // premium_short
            pos!(31.65),    // premium_long_high
            pos!(0.05),     // open_fee_long_call
            pos!(0.05),     // close_fee_long_call
            pos!(0.05),     // open_fee_long_call_low
            pos!(0.05),     // close_fee_long_call_low
            pos!(0.05),     // open_fee_long_call_high
            pos!(0.05),     // close_fee_long_call_high
        )
    }

    #[test]
    fn test_short_butterfly_get_position() {
        let mut butterfly = create_test_butterfly();

        // Test getting short call position
        let call_position = butterfly.get_position(&OptionStyle::Call, &Side::Short, &pos!(5780.0));
        assert!(call_position.is_ok());
        let positions = call_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(5780.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Short);

        // Test getting non-existent position
        let invalid_position =
            butterfly.get_position(&OptionStyle::Call, &Side::Short, &pos!(2715.0));
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
    fn test_long_butterfly_get_position() {
        let mut butterfly = create_test_butterfly();

        // Test getting short call position
        let call_position = butterfly.get_position(&OptionStyle::Call, &Side::Long, &pos!(5710.0));
        assert!(call_position.is_ok());
        let positions = call_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(5710.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Long);

        // Test getting short put position
        let put_position = butterfly.get_position(&OptionStyle::Call, &Side::Long, &pos!(5850.0));
        assert!(put_position.is_ok());
        let positions = put_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(5850.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Long);

        // Test getting non-existent position
        let invalid_position =
            butterfly.get_position(&OptionStyle::Call, &Side::Long, &pos!(2715.0));
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
        let mut modified_call = butterfly.short_call.clone();
        modified_call.option.quantity = pos!(2.0);
        let result = butterfly.modify_position(&modified_call);
        assert!(result.is_ok());
        assert_eq!(butterfly.short_call.option.quantity, pos!(2.0));

        // Test modifying with invalid position
        let mut invalid_position = butterfly.short_call.clone();
        invalid_position.option.strike_price = pos!(95.0);
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

    #[test]
    fn test_long_butterfly_modify_position() {
        let mut butterfly = create_test_butterfly();

        // Modify long call position
        let mut modified_call = butterfly.long_call_low.clone();
        modified_call.option.quantity = pos!(2.0);
        let result = butterfly.modify_position(&modified_call);
        assert!(result.is_ok());
        assert_eq!(butterfly.long_call_low.option.quantity, pos!(2.0));

        // Modify long put position
        let mut modified_put = butterfly.long_call_high.clone();
        modified_put.option.quantity = pos!(2.0);
        let result = butterfly.modify_position(&modified_put);
        assert!(result.is_ok());
        assert_eq!(butterfly.long_call_high.option.quantity, pos!(2.0));

        // Test modifying with invalid position
        let mut invalid_position = butterfly.long_call_high.clone();
        invalid_position.option.strike_price = pos!(95.0);
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
mod tests_adjust_option_position_long {
    use super::*;
    use crate::model::types::{OptionStyle, Side};
    use crate::pos;
    use rust_decimal_macros::dec;

    // Helper function to create a test strategy
    fn create_test_strategy() -> LongButterflySpread {
        LongButterflySpread::new(
            "SP500".to_string(),
            pos!(5795.88), // underlying_price
            pos!(5710.0),  // long_strike_itm
            pos!(5780.0),  // short_strike
            pos!(5850.0),  // long_strike_otm
            ExpirationDate::Days(pos!(2.0)),
            pos!(0.18),     // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(2.0),      // long quantity
            pos!(113.3),    // premium_long_low
            pos!(64.20),    // premium_short
            pos!(31.65),    // premium_long_high
            pos!(0.05),     // open_fee_long_call
            pos!(0.05),     // close_fee_long_call
            pos!(0.05),     // open_fee_long_call_low
            pos!(0.05),     // close_fee_long_call_low
            pos!(0.05),     // open_fee_long_call_high
            pos!(0.05),     // close_fee_long_call_high
        )
    }

    #[test]
    fn test_adjust_existing_call_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.long_call_low.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos!(5710.0),
            &OptionStyle::Call,
            &Side::Long,
        );

        assert!(result.is_ok());
        assert_eq!(
            strategy.long_call_low.option.quantity,
            initial_quantity + adjustment
        );
    }

    #[test]
    fn test_adjust_existing_put_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.short_call.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos!(5780.0),
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
    fn test_adjust_nonexistent_position() {
        let mut strategy = create_test_strategy();

        // Try to adjust a non-existent long call position
        let result = strategy.adjust_option_position(
            Decimal::ONE,
            &pos!(110.0),
            &OptionStyle::Put,
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
                assert_eq!(reason, "Put not found in positions");
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
        let initial_quantity = strategy.long_call_high.option.quantity;

        let result = strategy.adjust_option_position(
            Decimal::ZERO,
            &pos!(5850.0),
            &OptionStyle::Call,
            &Side::Long,
        );

        assert!(result.is_ok());
        assert_eq!(strategy.long_call_high.option.quantity, initial_quantity);
    }
}

#[cfg(test)]
mod tests_long_butterfly_spread_constructor {
    use super::*;
    use crate::model::utils::create_sample_position;
    use crate::pos;

    #[test]
    fn test_get_strategy_valid() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(90.0),
                pos!(1.0),
                pos!(95.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(90.0),
                pos!(1.0),
                pos!(100.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(90.0),
                pos!(1.0),
                pos!(105.0),
                pos!(0.2),
            ),
        ];

        let result = LongButterflySpread::get_strategy(&options);
        assert!(result.is_ok());

        let strategy = result.unwrap();
        assert_eq!(strategy.long_call_low.option.strike_price, pos!(95.0));
        assert_eq!(strategy.short_call.option.strike_price, pos!(100.0));
        assert_eq!(strategy.long_call_high.option.strike_price, pos!(105.0));
    }

    #[test]
    fn test_get_strategy_wrong_number_of_options() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(90.0),
                pos!(1.0),
                pos!(95.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(90.0),
                pos!(1.0),
                pos!(100.0),
                pos!(0.2),
            ),
        ];

        let result = LongButterflySpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Long Butterfly Spread get_strategy" && reason == "Must have exactly 3 options"
        ));
    }

    #[test]
    fn test_get_strategy_wrong_option_style() {
        let mut option1 = create_sample_position(
            OptionStyle::Call,
            Side::Long,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );
        option1.option.option_style = OptionStyle::Put;
        let option2 = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(100.0),
            pos!(0.2),
        );
        let option3 = create_sample_position(
            OptionStyle::Call,
            Side::Long,
            pos!(90.0),
            pos!(1.0),
            pos!(105.0),
            pos!(0.2),
        );

        let options = vec![option1, option2, option3];
        let result = LongButterflySpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Long Butterfly Spread get_strategy" && reason == "Options must be calls"
        ));
    }

    #[test]
    fn test_get_strategy_wrong_sides() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(90.0),
                pos!(1.0),
                pos!(95.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(90.0),
                pos!(1.0),
                pos!(100.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(90.0),
                pos!(1.0),
                pos!(105.0),
                pos!(0.2),
            ),
        ];

        let result = LongButterflySpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Long Butterfly Spread get_strategy"
                && reason == "Long Butterfly requires long lower and higher strikes with a short middle strike"
        ));
    }

    #[test]
    fn test_get_strategy_asymmetric_strikes() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(90.0),
                pos!(1.0),
                pos!(95.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(90.0),
                pos!(1.0),
                pos!(101.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(90.0),
                pos!(1.0),
                pos!(105.0),
                pos!(0.2),
            ),
        ];

        let result = LongButterflySpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Long Butterfly Spread get_strategy" && reason == "Strikes must be symmetrical"
        ));
    }

    #[test]
    fn test_get_strategy_different_expiration_dates() {
        let mut option1 = create_sample_position(
            OptionStyle::Call,
            Side::Long,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );
        let mut option2 = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(100.0),
            pos!(0.2),
        );
        let mut option3 = create_sample_position(
            OptionStyle::Call,
            Side::Long,
            pos!(90.0),
            pos!(1.0),
            pos!(105.0),
            pos!(0.2),
        );

        option1.option.expiration_date = ExpirationDate::Days(pos!(30.0));
        option2.option.expiration_date = ExpirationDate::Days(pos!(60.0));
        option3.option.expiration_date = ExpirationDate::Days(pos!(30.0));

        let options = vec![option1, option2, option3];
        let result = LongButterflySpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Long Butterfly Spread get_strategy" && reason == "Options must have the same expiration date"
        ));
    }

    #[test]
    fn test_get_strategy_with_extra_conditions() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(90.0),
                pos!(1.0),
                pos!(90.0), // Lower strike price
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(90.0),
                pos!(1.0),
                pos!(100.0), // Middle strike price
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(90.0),
                pos!(1.0),
                pos!(110.0), // Higher strike price
                pos!(0.2),
            ),
        ];

        let result = LongButterflySpread::get_strategy(&options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_strategy_multiple_identical_strikes() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(100.0),
                pos!(1.0),
                pos!(95.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
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
                pos!(105.0),
                pos!(0.2),
            ),
        ];

        let result = LongButterflySpread::get_strategy(&options);
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod tests_long_butterfly_spread_pnl {
    use super::*;
    use crate::model::utils::create_sample_position;
    use crate::{assert_decimal_eq, pos};
    use rust_decimal_macros::dec;

    fn create_test_long_butterfly_spread() -> Result<LongButterflySpread, StrategyError> {
        // Create lower long call
        let lower_short_call = create_sample_position(
            OptionStyle::Call,
            Side::Long,
            pos!(100.0), // Underlying price
            pos!(1.0),   // Quantity
            pos!(95.0),  // Lower strike price
            pos!(0.2),   // Implied volatility
        );

        // Create middle short call
        let middle_short_call = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(100.0), // Same underlying price
            pos!(1.0),   // Quantity
            pos!(100.0), // Middle strike price (ATM)
            pos!(0.2),   // Implied volatility
        );

        // Create higher long call
        let higher_short_call = create_sample_position(
            OptionStyle::Call,
            Side::Long,
            pos!(100.0), // Same underlying price
            pos!(1.0),   // Quantity
            pos!(105.0), // Higher strike price
            pos!(0.2),   // Implied volatility
        );

        LongButterflySpread::get_strategy(&vec![
            lower_short_call,
            middle_short_call,
            higher_short_call,
        ])
    }

    #[test]
    fn test_calculate_pnl_below_strikes() {
        let spread = create_test_long_butterfly_spread().unwrap();
        let market_price = pos!(90.0); // Below all strikes
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.2);

        let result = spread.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // All options OTM, near max loss
        // Max loss should be the net debit paid
        assert!(pnl.unrealized.unwrap() < dec!(0.0));
        assert!(pnl.unrealized.unwrap() > dec!(-5.0)); // Not worse than max loss
    }

    #[test]
    fn test_calculate_pnl_between_strikes() {
        let spread = create_test_long_butterfly_spread().unwrap();
        let market_price = pos!(100.0); // At middle strike
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.1);

        let result = spread.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // More flexible assertions
        assert!(pnl.unrealized.is_some(), "Unrealized PnL should be present");

        // Check if unrealized PnL is within a reasonable range
        assert!(
            pnl.unrealized.unwrap() >= dec!(-10.0) && pnl.unrealized.unwrap() <= dec!(10.0),
            "Unrealized PnL should be within a reasonable range. Got: {}",
            pnl.unrealized.unwrap()
        );
    }

    #[test]
    fn test_calculate_pnl_above_strikes() {
        let spread = create_test_long_butterfly_spread().unwrap();
        let market_price = pos!(90.0);
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.2);

        let result = spread.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());
        // All options ITM, near max loss
        assert!(pnl.unrealized.unwrap() < dec!(0.0));
        assert!(pnl.unrealized.unwrap() > dec!(-5.0)); // Not worse than max loss
    }

    #[test]
    fn test_calculate_pnl_at_expiration_max_profit() {
        let spread = create_test_long_butterfly_spread().unwrap();
        let underlying_price = pos!(95.0); // At or below lowest strike

        let result = spread.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // Max loss should be the net debit paid
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(-8.0), dec!(1e-6));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_max_loss() {
        let spread = create_test_long_butterfly_spread().unwrap();
        let underlying_price = pos!(110.0); // Above highest strike

        let result = spread.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // Max loss should be the net debit paid (including fees)
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(2.0), dec!(1e-6));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_at_middle_strike() {
        let spread = create_test_long_butterfly_spread().unwrap();
        let underlying_price = pos!(100.0); // At middle strike

        let result = spread.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // Near max loss
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(-3.0), dec!(1e-6));
    }
}

#[cfg(test)]
mod tests_butterfly_strategies {
    use super::*;
    use crate::constants::ZERO;
    use crate::model::ExpirationDate;
    use crate::pos;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn create_test_long() -> LongButterflySpread {
        LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(90.0),  // low_strike
            pos!(100.0), // middle_strike
            pos!(110.0), // high_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(3.0),      // premium_low
            Positive::TWO,  // premium_middle
            Positive::ONE,  // premium_high
            pos!(0.05),     // open_fee_long_call
            pos!(0.05),     // close_fee_long_call
            pos!(0.05),     // open_fee_long_call_low
            pos!(0.05),     // close_fee_long_call_low
            pos!(0.05),     // open_fee_long_call_high
            pos!(0.05),     // close_fee_long_call_high
        )
    }

    #[test]
    fn test_add_leg_long_butterfly() {
        let mut butterfly = create_test_long();
        let new_long = Position::new(
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
            .add_position(&new_long.clone())
            .expect("Failed to add position");
        assert_eq!(butterfly.long_call_low.option.strike_price, pos!(85.0));
    }

    #[test]
    fn test_get_legs() {
        let long_butterfly = create_test_long();
        assert_eq!(long_butterfly.get_positions().unwrap().len(), 3);
    }

    #[test]
    fn test_max_profit_long_butterfly() {
        let butterfly = create_test_long();
        let max_profit = butterfly.get_max_profit().unwrap().to_dec();
        // Max profit at middle strike
        let expected_profit = butterfly.calculate_profit_at(&pos!(100.0)).unwrap();
        assert_eq!(max_profit, expected_profit);
    }

    #[test]
    fn test_max_loss_long_butterfly() {
        let butterfly = create_test_long();
        let max_loss = butterfly.get_max_loss().unwrap().to_dec();
        // Max loss at wings
        let left_loss = butterfly.calculate_profit_at(&pos!(90.0)).unwrap();
        let right_loss = butterfly.calculate_profit_at(&pos!(110.0)).unwrap();
        assert_eq!(max_loss, left_loss.min(right_loss).abs());
    }

    #[test]
    fn test_total_cost() {
        let long_butterfly = create_test_long();
        assert!(long_butterfly.get_total_cost().unwrap() > Positive::ZERO);
    }

    #[test]
    fn test_fees() {
        let butterfly = LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(3.0),
            Positive::TWO,
            Positive::ONE,
            Positive::ONE, // open_fee_long_call
            Positive::ONE, // close_fee_long_call
            Positive::ONE, // open_fee_long_call_low
            Positive::ONE, // close_fee_long_call_low
            Positive::ONE, // open_fee_long_call_high
            Positive::ONE, // close_fee_long_call_high
        );
        assert_eq!(butterfly.get_fees().unwrap().to_f64(), 8.0);
    }

    #[test]
    fn test_fees_bis() {
        let butterfly = LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(2.0),
            pos!(3.0),
            Positive::TWO,
            Positive::ONE,
            Positive::ONE, // open_fee_long_call
            Positive::ONE, // close_fee_long_call
            Positive::ONE, // open_fee_long_call_low
            Positive::ONE, // close_fee_long_call_low
            Positive::ONE, // open_fee_long_call_high
            Positive::ONE, // close_fee_long_call_high
        );

        assert_eq!(butterfly.get_fees().unwrap(), pos!(16.0));
    }

    #[test]
    fn test_profit_area_long_butterfly() {
        let butterfly = create_test_long();
        let area = butterfly.get_profit_area().unwrap().to_f64().unwrap();
        assert!(area > ZERO);
    }

    #[test]
    fn test_profit_ratio() {
        let long_butterfly = create_test_long();
        assert!(long_butterfly.get_profit_ratio().unwrap().to_f64().unwrap() > ZERO);
    }

    #[test]
    fn test_break_even_points() {
        let long_butterfly = create_test_long();
        assert_eq!(long_butterfly.get_break_even_points().unwrap().len(), 2);
    }

    #[test]
    fn test_profits_with_quantities() {
        let long_butterfly = LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(2.0), // quantity = 2
            pos!(3.0),
            Positive::TWO,
            Positive::ONE,
            pos!(0.05), // open_fee_long_call
            pos!(0.05), // close_fee_long_call
            pos!(0.05), // open_fee_long_call_low
            pos!(0.05), // close_fee_long_call_low
            pos!(0.05), // open_fee_long_call_high
            pos!(0.05), // close_fee_long_call_high
        );

        let base_butterfly = create_test_long();
        assert_eq!(
            long_butterfly.get_max_profit().unwrap().to_f64(),
            base_butterfly.get_max_profit().unwrap().to_f64() * 2.0
        );
    }
}

#[cfg(test)]
mod tests_butterfly_optimizable {
    use super::*;
    use crate::model::ExpirationDate;
    use crate::{pos, spos};

    use rust_decimal_macros::dec;

    fn create_test_option_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

        for strike in [85.0, 90.0, 95.0, 100.0, 105.0, 110.0, 115.0] {
            chain.add_option(
                pos!(strike),
                spos!(5.0),      // call_bid
                spos!(5.2),      // call_ask
                spos!(5.0),      // put_bid
                spos!(5.2),      // put_ask
                pos!(0.2),       // implied_volatility
                Some(dec!(0.5)), // delta
                Some(dec!(0.2)),
                Some(dec!(0.2)),
                spos!(100.0), // volume
                Some(50),     // open_interest
                None,
            );
        }
        chain
    }

    fn create_test_long() -> LongButterflySpread {
        LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(3.0),
            Positive::TWO,
            Positive::ONE,
            pos!(0.05), // open_fee_long_call
            pos!(0.05), // close_fee_long_call
            pos!(0.05), // open_fee_long_call_low
            pos!(0.05), // close_fee_long_call_low
            pos!(0.05), // open_fee_long_call_high
            pos!(0.05), // close_fee_long_call_high
        )
    }

    #[test]
    fn test_find_optimal_area() {
        let mut butterfly = create_test_long();
        let chain = create_test_option_chain();
        let initial_area = butterfly.get_profit_area().unwrap();

        butterfly.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Area);

        assert!(butterfly.validate());
        assert!(butterfly.get_profit_area().unwrap() >= initial_area);
    }

    #[test]
    fn test_valid_strike_order() {
        let mut butterfly = create_test_long();
        let chain = create_test_option_chain();

        butterfly.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);

        assert!(
            butterfly.long_call_low.option.strike_price < butterfly.short_call.option.strike_price
        );
        assert!(
            butterfly.short_call.option.strike_price < butterfly.long_call_high.option.strike_price
        );
    }

    #[test]
    fn test_find_optimal_area_long() {
        let mut butterfly = create_test_long();
        let chain = create_test_option_chain();
        let initial_area = butterfly.get_profit_area().unwrap();

        butterfly.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Area);

        assert!(butterfly.validate());
        assert!(butterfly.get_profit_area().unwrap() >= initial_area);
    }

    #[test]
    fn test_find_optimal_with_range() {
        let mut long_butterfly = create_test_long();
        let chain = create_test_option_chain();

        long_butterfly.find_optimal(
            &chain,
            FindOptimalSide::Range(pos!(95.0), pos!(105.0)),
            OptimizationCriteria::Ratio,
        );

        assert!(long_butterfly.short_call.option.strike_price >= pos!(95.0));
        assert!(long_butterfly.short_call.option.strike_price <= pos!(105.0));
    }
}

#[cfg(test)]
mod tests_butterfly_probability {
    use super::*;
    use crate::model::ExpirationDate;
    use crate::pos;
    use rust_decimal_macros::dec;

    fn create_test_long() -> LongButterflySpread {
        LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            pos!(10.0),
            Positive::TWO,
            Positive::ONE,
            pos!(0.05), // open_fee_long_call
            pos!(0.05), // close_fee_long_call
            pos!(0.05), // open_fee_long_call_low
            pos!(0.05), // close_fee_long_call_low
            pos!(0.05), // open_fee_long_call_high
            pos!(0.05), // close_fee_long_call_high
        )
    }

    mod long_butterfly_tests {
        use super::*;

        #[test]
        fn test_get_expiration() {
            let butterfly = create_test_long();
            let expiration_date = *butterfly.get_expiration().values().next().unwrap();
            assert_eq!(expiration_date, &ExpirationDate::Days(pos!(30.0)));
        }

        #[test]
        fn test_get_risk_free_rate() {
            let butterfly = create_test_long();
            assert_eq!(
                **butterfly.get_risk_free_rate().values().next().unwrap(),
                dec!(0.05)
            );
        }

        #[test]
        fn test_get_profit_ranges() {
            let butterfly = create_test_long();
            let ranges = butterfly.get_profit_ranges().unwrap();

            assert_eq!(ranges.len(), 1);
            let range = &ranges[0];

            let break_even_points = butterfly.get_break_even_points().unwrap();
            assert_eq!(range.lower_bound.unwrap(), break_even_points[0]);
            assert_eq!(range.upper_bound.unwrap(), break_even_points[1]);
            assert!(range.probability > Positive::ZERO);
        }

        #[test]
        fn test_get_loss_ranges() {
            let butterfly = create_test_long();
            let ranges = butterfly.get_loss_ranges().unwrap();

            assert_eq!(ranges.len(), 2);

            let lower_range = &ranges[0];
            assert_eq!(
                lower_range.lower_bound.unwrap(),
                butterfly.long_call_low.option.strike_price
            );
            assert_eq!(
                lower_range.upper_bound.unwrap(),
                butterfly.get_break_even_points().unwrap()[0]
            );
            assert!(lower_range.probability > Positive::ZERO);

            let upper_range = &ranges[1];
            assert_eq!(
                upper_range.lower_bound.unwrap(),
                butterfly.get_break_even_points().unwrap()[1]
            );
            assert_eq!(
                upper_range.upper_bound.unwrap(),
                butterfly.long_call_high.option.strike_price
            );
            assert!(upper_range.probability > Positive::ZERO);
        }
    }

    #[test]
    fn test_volatility_calculations() {
        let long_butterfly = create_test_long();
        let long_ranges = long_butterfly.get_profit_ranges().unwrap();
        assert!(!long_ranges.is_empty());
        assert!(long_ranges[0].probability > Positive::ZERO);
    }

    #[test]
    fn test_probability_sum() {
        let long_butterfly = create_test_long();
        let long_profit_ranges = long_butterfly.get_profit_ranges().unwrap();
        let long_loss_ranges = long_butterfly.get_loss_ranges().unwrap();
        let long_total_prob = long_profit_ranges
            .iter()
            .map(|r| r.probability.to_f64())
            .sum::<f64>()
            + long_loss_ranges
                .iter()
                .map(|r| r.probability.to_f64())
                .sum::<f64>();
        assert!((long_total_prob - 1.0).abs() < 0.1);
    }
}
