use super::base::{
    BreakEvenable, Optimizable, Positionable, Strategable, StrategyBasics, StrategyType, Validable,
};
use crate::error::strategies::BreakEvenErrorKind;
use crate::error::strategies::{ProfitLossErrorKind, StrategyError};
use crate::strategies::shared::decimal_from_f64;
use crate::strategies::shared::measured_max_profit;
use crate::strategies::shared::{
    CachedBreakEvens, apply_contract_size, common_contract_size, edit_refreshing_break_evens,
};
use crate::strategies::{
    BasicAble, Strategies, StrategyConstructor, delta_neutral::DeltaNeutrality,
    probabilities::ProbabilityAnalysis, utils::OptimizationCriteria,
};
use chrono::Utc;
use optionstratlib_analytics::analytics::ProfitLossRange;
use optionstratlib_analytics::analytics::VolatilityAdjustment;
use optionstratlib_analytics::error::probability::{ProbabilityError, ProfitLossRangeErrorKind};
use optionstratlib_analytics::pnl::{PnLCalculator, utils::PnL};
use optionstratlib_core::error::OperationErrorKind;
use optionstratlib_core::error::position::{PositionError, PositionValidationErrorKind};
use optionstratlib_core::model::ExpirationDate;
use optionstratlib_core::model::Options;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::{
    decimal::{d_add, d_div, d_sub, d_sum},
    position::Position,
    types::{OptionBasicType, OptionStyle, OptionType, Side},
    utils::mean_and_std,
};
#[cfg(test)]
use optionstratlib_core::spos;
use optionstratlib_core::{impl_json_debug_pretty, impl_json_display};
use optionstratlib_market::chains::utils::FindOptimalSide;
use optionstratlib_market::chains::{StrategyLegs, chain::OptionChain, utils::OptionDataGroup};
use optionstratlib_pricing::error::GreeksError;
use optionstratlib_pricing::error::PricingError;
use optionstratlib_pricing::greeks::Greeks;
use optionstratlib_pricing::pricing::OptionPricing;
use optionstratlib_pricing::pricing::Profit;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{debug, error, info};

/// The default description for the Bull Call Ladder strategy.
pub const BULL_CALL_LADDER_DESCRIPTION: &str = "A bull call ladder buys one call at a lower strike and sells one call \
    at a middle strike and one call at a higher strike, all with the same expiration. It is a \
    bull call spread financed further by a second short call: it profits from a moderate rise \
    in the underlying, and the loss is unlimited above the upper break-even.";

/// Represents a Bull Call Ladder options trading strategy.
///
/// A bull call ladder (also called a long call ladder) combines three call
/// options with the same expiration and quantity at three strikes:
/// - one long call at the lower strike `K1`,
/// - one short call at the middle strike `K2`,
/// - one short call at the higher strike `K3`.
///
/// ```text
/// P&L at expiry
///   |        ________
///   |       /        \
///   |______/          \
///   |                  \   unlimited loss
///   +----K1---K2----K3---\---> underlying
/// ```
///
/// Below `K1` every call expires worthless and the position keeps its net
/// premium. Between `K1` and `K2` it gains one for one, between `K2` and `K3`
/// it holds its maximum, and above `K3` it is net short one call, so the loss
/// is unlimited.
///
/// Until #706 this type was named `CallButterfly`, a name it never matched:
/// a textbook call butterfly is long the outer strikes and short twice the
/// middle one, which is [`crate::strategies::LongButterflySpread`]. The old
/// name is gone rather than kept as an alias, so code written for it fails
/// to compile instead of silently building something else.
#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
pub struct BullCallLadder {
    /// The name of the strategy, typically used for identification purposes.
    pub name: String,

    /// The type of strategy, `StrategyType::BullCallLadder`.
    pub kind: StrategyType,

    /// A detailed description of the strategy, its objectives, and potential outcomes.
    pub description: String,

    /// The price points at which the strategy breaks even (neither profits nor loses).
    /// A debit ladder has two; a credit ladder has only the upper one.
    pub break_even_points: Vec<Positive>,

    /// The long call position at the lower strike price.
    pub long_call: Position,

    /// The short call position at the middle strike price.
    pub short_call_low: Position,

    /// The short call position at the higher strike price.
    pub short_call_high: Position,
}

impl_json_debug_pretty!(BullCallLadder);
impl_json_display!(BullCallLadder);

impl BullCallLadder {
    /// Creates a new Bull Call Ladder options strategy.
    ///
    /// A Bull Call Ladder consists of, all with the same quantity:
    /// - 1 long call at the lower strike
    /// - 1 short call at the middle strike
    /// - 1 short call at the higher strike
    ///
    /// It is used for a moderately bullish view: the second short call
    /// finances the bull call spread further, at the cost of an unlimited
    /// loss above the upper break-even.
    ///
    /// # Parameters
    ///
    /// ## Asset Information
    /// * `underlying_symbol` - Symbol of the underlying asset (e.g., "SPY", "AAPL")
    /// * `underlying_price` - Current market price of the underlying asset
    /// * `dividend_yield` - Dividend yield of the underlying asset
    ///
    /// ## Strike Prices
    /// * `long_call_strike` - Strike price for the long call option (lowest)
    /// * `short_call_low_strike` - Strike price for the middle short call option
    /// * `short_call_high_strike` - Strike price for the higher short call option
    ///
    /// ## Market Parameters
    /// * `expiration` - Expiration date for all options in the strategy
    /// * `implied_volatility` - Implied volatility for the options
    /// * `risk_free_rate` - Current risk-free interest rate
    /// * `quantity` - Number of contracts for each position
    ///
    /// ## Premium and Fee Information
    /// * `premium_long_call` - Premium paid for the long call
    /// * `premium_short_call_low` - Premium received for the first short call
    /// * `premium_short_call_high` - Premium received for the second short call
    /// * `open_fee_long` - Fee to open the long call position
    /// * `close_fee_long` - Fee to close the long call position
    /// * `open_fee_short_low` - Fee to open the first short call position
    /// * `close_fee_short_low` - Fee to close the first short call position
    /// * `open_fee_short_high` - Fee to open the second short call position
    /// * `close_fee_short_high` - Fee to close the second short call position
    ///
    /// # Returns
    ///
    /// A fully initialized `BullCallLadder` strategy with all positions and break-even points calculated.
    ///
    /// # Errors
    ///
    /// Returns `StrategyError::InvalidStrategy` when the assembled strategy
    /// fails its own `validate` (#706): the strikes are not strictly ordered
    /// long < short low < short high, or a leg fails `Position::validate`.
    ///
    /// Returns `StrategyError` if any freshly-constructed leg cannot be added
    /// to the strategy or if the break-even calculation fails. In practice
    /// these branches are unreachable for a freshly-built call ladder and
    /// are surfaced only to keep the constructor panic-free.
    #[allow(clippy::too_many_arguments)]
    #[inline(never)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        long_call_strike: Positive,
        short_call_low_strike: Positive,
        short_call_high_strike: Positive,
        expiration: ExpirationDate,
        implied_volatility: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        quantity: Positive,
        premium_long_call: Positive,
        premium_short_call_low: Positive,
        premium_short_call_high: Positive,
        open_fee_long: Positive,
        close_fee_long: Positive,
        open_fee_short_low: Positive,
        close_fee_short_low: Positive,
        open_fee_short_high: Positive,
        close_fee_short_high: Positive,
    ) -> Result<Self, StrategyError> {
        let mut strategy = BullCallLadder {
            name: underlying_symbol.to_string(),
            kind: StrategyType::BullCallLadder,
            description: BULL_CALL_LADDER_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::default(),
            short_call_low: Position::default(),
            short_call_high: Position::default(),
        };
        let long_call_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            long_call_strike,
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
            open_fee_long,
            close_fee_long,
            None,
            None,
        );
        strategy.place_leg(&long_call)?;
        strategy.long_call = long_call;

        let short_call_low_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            short_call_low_strike,
            expiration,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        let short_call_low = Position::new(
            short_call_low_option,
            premium_short_call_low,
            Utc::now(),
            open_fee_short_low,
            close_fee_short_low,
            None,
            None,
        );
        strategy.place_leg(&short_call_low)?;
        strategy.short_call_low = short_call_low;

        let short_call_high_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            short_call_high_strike,
            expiration,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        let short_call_high = Position::new(
            short_call_high_option,
            premium_short_call_high,
            Utc::now(),
            open_fee_short_high,
            close_fee_short_high,
            None,
            None,
        );
        strategy.place_leg(&short_call_high)?;
        strategy.short_call_high = short_call_high;

        if !strategy.validate() {
            return Err(StrategyError::invalid_strategy(
                StrategyType::BullCallLadder,
                "the legs built by `new` fail validation",
            ));
        }
        strategy.update_break_even_points()?;
        Ok(strategy)
    }
}

impl StrategyConstructor for BullCallLadder {
    fn get_strategy(vec_positions: &[Position]) -> Result<Self, StrategyError> {
        // Need exactly 3 options for a call ladder
        if vec_positions.len() != 3 {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Bull Call Ladder get_strategy".to_string(),
                    reason: "Must have exactly 3 options".to_string(),
                },
            ));
        }

        // Sort options by strike price
        let mut sorted_positions = vec_positions.to_vec();
        // SAFETY: total order on Positive; f64 fallback to Equal is safe for stable sort
        sorted_positions.sort_by(|a, b| {
            a.option
                .strike_price
                .partial_cmp(&b.option.strike_price)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let [
            long_call_position,
            low_short_call_position,
            high_short_call_position,
        ] = sorted_positions.as_slice()
        else {
            return Err(StrategyError::invalid_parameters(
                "Bull Call Ladder get_strategy",
                "Must have exactly 3 options",
            ));
        };

        // Validate options are calls
        if long_call_position.option.option_style != OptionStyle::Call
            || low_short_call_position.option.option_style != OptionStyle::Call
            || high_short_call_position.option.option_style != OptionStyle::Call
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Bull Call Ladder get_strategy".to_string(),
                    reason: "Options must be calls".to_string(),
                },
            ));
        }

        // Validate option sides: long the lowest strike, short the other two,
        // the same convention as `new` and `validate` (#706).
        if long_call_position.option.side != Side::Long
            || low_short_call_position.option.side != Side::Short
            || high_short_call_position.option.side != Side::Short
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Bull Call Ladder get_strategy".to_string(),
                    reason: "Bull Call Ladder requires a long lowest-strike call and two short higher-strike calls".to_string(),
                },
            ));
        }

        // Validate expiration dates match
        if low_short_call_position.option.expiration_date
            != long_call_position.option.expiration_date
            || long_call_position.option.expiration_date
                != high_short_call_position.option.expiration_date
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Bull Call Ladder get_strategy".to_string(),
                    reason: "Options must have the same expiration date".to_string(),
                },
            ));
        }

        // Create positions
        let long_call = Position::new(
            long_call_position.option.clone(),
            long_call_position.premium,
            Utc::now(),
            long_call_position.open_fee,
            long_call_position.close_fee,
            long_call_position.epic.clone(),
            long_call_position.extra_fields.clone(),
        );

        let short_call_low = Position::new(
            low_short_call_position.option.clone(),
            low_short_call_position.premium,
            Utc::now(),
            low_short_call_position.open_fee,
            low_short_call_position.close_fee,
            low_short_call_position.epic.clone(),
            low_short_call_position.extra_fields.clone(),
        );

        let short_call_high = Position::new(
            high_short_call_position.option.clone(),
            high_short_call_position.premium,
            Utc::now(),
            high_short_call_position.open_fee,
            high_short_call_position.close_fee,
            high_short_call_position.epic.clone(),
            high_short_call_position.extra_fields.clone(),
        );

        // Create strategy
        let mut strategy = BullCallLadder {
            name: "Bull Call Ladder".to_string(),
            kind: StrategyType::BullCallLadder,
            description: BULL_CALL_LADDER_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call,
            short_call_low,
            short_call_high,
        };

        if !strategy.validate() {
            return Err(StrategyError::invalid_strategy(
                StrategyType::BullCallLadder,
                "the positions passed to `get_strategy` fail validation",
            ));
        }
        strategy.update_break_even_points()?;

        Ok(strategy)
    }
}

impl BreakEvenable for BullCallLadder {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        // Each wing contributes a break-even only where the payoff actually
        // crosses zero: a candidate below zero is a price the underlying
        // cannot reach, so the wing has none. A leg with no contracts has no
        // per-contract profit at all, which is reported rather than divided.
        let long_strike = self.long_call.option.strike_price.to_dec();
        let long_qty = self.long_call.option.position_size()?.to_dec();
        let long_profit = self.calculate_profit_at(&self.long_call.option.strike_price)?;
        let long_per_contract = d_div(
            long_profit,
            long_qty,
            "BullCallLadder::update_break_even_points",
        )?;
        let candidate_low = d_sub(
            long_strike,
            long_per_contract,
            "BullCallLadder::update_break_even_points",
        )?;
        if let Ok(be) = Positive::new_decimal(candidate_low) {
            self.break_even_points.push(be.checked_round_to(2)?);
        }

        let short_strike = self.short_call_high.option.strike_price.to_dec();
        let short_qty = self.short_call_high.option.position_size()?.to_dec();
        let short_profit = self.calculate_profit_at(&self.short_call_high.option.strike_price)?;
        let short_per_contract = d_div(
            short_profit,
            short_qty,
            "BullCallLadder::update_break_even_points",
        )?;
        let candidate_high = d_add(
            short_strike,
            short_per_contract,
            "BullCallLadder::update_break_even_points",
        )?;
        if let Ok(be) = Positive::new_decimal(candidate_high) {
            self.break_even_points.push(be.checked_round_to(2)?);
        }

        self.break_even_points.sort();
        Ok(())
    }
}

impl BullCallLadder {
    /// Places `position` in the leg its side and style select, without
    /// refreshing the break-evens: the constructors fill the legs through
    /// it, and [`Positionable::add_position`] wraps it.
    fn place_leg(&mut self, position: &Position) -> Result<(), PositionError> {
        match position.option.side {
            Side::Short => {
                // Both short calls sit above the long one, so comparing with
                // the long strike cannot tell them apart (it did, as a
                // butterfly, before #706). The first short fills the empty
                // slot and the pair is kept ordered by strike.
                if self.short_call_low.option.strike_price == Positive::ZERO {
                    self.short_call_low = position.clone();
                } else {
                    self.short_call_high = position.clone();
                }
                if self.short_call_high.option.strike_price != Positive::ZERO
                    && self.short_call_low.option.strike_price
                        > self.short_call_high.option.strike_price
                {
                    std::mem::swap(&mut self.short_call_low, &mut self.short_call_high);
                }
                Ok(())
            }
            Side::Long => {
                self.long_call = position.clone();
                Ok(())
            }
        }
    }

    /// Replaces the leg matching `position`, without refreshing the
    /// break-evens; [`Positionable::modify_position`] wraps it.
    fn replace_leg(&mut self, position: &Position) -> Result<(), PositionError> {
        if !position.validate() {
            return Err(PositionError::ValidationError(
                PositionValidationErrorKind::InvalidPosition {
                    reason: "Invalid position data".to_string(),
                },
            ));
        }

        if position.option.strike_price != self.long_call.option.strike_price
            && position.option.strike_price != self.short_call_low.option.strike_price
            && position.option.strike_price != self.short_call_high.option.strike_price
        {
            return Err(PositionError::invalid_position_type(
                position.option.side,
                "Strike not found in positions".to_string(),
            ));
        }

        if position.option.option_style == OptionStyle::Put {
            return Err(PositionError::invalid_position_type(
                position.option.side,
                "Put is not valid for BullCallLadder".to_string(),
            ));
        }

        if position.option.option_style == OptionStyle::Call && position.option.side == Side::Long {
            self.long_call = position.clone();
        }

        if position.option.strike_price == self.short_call_low.option.strike_price {
            self.short_call_low = position.clone();
        }
        if position.option.strike_price == self.short_call_high.option.strike_price {
            self.short_call_high = position.clone();
        }

        Ok(())
    }
}

impl CachedBreakEvens for BullCallLadder {
    fn break_evens_mut(&mut self) -> &mut Vec<Positive> {
        &mut self.break_even_points
    }
}

impl Positionable for BullCallLadder {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        edit_refreshing_break_evens(self, |strategy| strategy.place_leg(position))
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![
            &self.long_call,
            &self.short_call_low,
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
            (Side::Long, OptionStyle::Call, strike)
                if *strike == self.long_call.option.strike_price =>
            {
                Ok(vec![&mut self.long_call])
            }
            (_, OptionStyle::Put, _) => Err(PositionError::invalid_position_type(
                *side,
                "Put not found in positions".to_string(),
            )),
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
        edit_refreshing_break_evens(self, |strategy| strategy.replace_leg(position))
    }
}

impl Strategable for BullCallLadder {
    fn info(&self) -> Result<StrategyBasics, StrategyError> {
        Ok(StrategyBasics {
            name: self.name.clone(),
            kind: self.kind.clone(),
            description: self.description.clone(),
        })
    }
}

impl BasicAble for BullCallLadder {
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
    fn one_option(&self) -> Result<&Options, StrategyError> {
        self.long_call.one_option()
    }
    fn one_option_mut(&mut self) -> Result<&mut Options, StrategyError> {
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
        self.short_call_low.premium = Positive::new_decimal(
            self.short_call_low
                .option
                .calculate_price_black_scholes()?
                .abs(),
        )?;
        self.long_call.option.underlying_price = *price;
        self.long_call.premium =
            Positive::new_decimal(self.long_call.option.calculate_price_black_scholes()?.abs())?;
        self.short_call_high.option.underlying_price = *price;
        self.short_call_high.premium = Positive::new_decimal(
            self.short_call_high
                .option
                .calculate_price_black_scholes()?
                .abs(),
        )?;
        Ok(())
    }
    fn set_implied_volatility(&mut self, volatility: &Positive) -> Result<(), StrategyError> {
        self.short_call_low.option.implied_volatility = *volatility;
        self.long_call.option.implied_volatility = *volatility;
        self.short_call_high.option.implied_volatility = *volatility;

        self.short_call_low.premium = Positive::new_decimal(
            self.short_call_low
                .option
                .calculate_price_black_scholes()?
                .abs(),
        )?;
        self.long_call.premium =
            Positive::new_decimal(self.long_call.option.calculate_price_black_scholes()?.abs())?;
        self.short_call_high.premium = Positive::new_decimal(
            self.short_call_high
                .option
                .calculate_price_black_scholes()?
                .abs(),
        )?;
        Ok(())
    }
    fn get_contract_size(&self) -> Result<Positive, StrategyError> {
        common_contract_size(
            &[&self.long_call, &self.short_call_low, &self.short_call_high],
            "BullCallLadder::get_contract_size",
        )
    }
    fn set_contract_size(&mut self, contract_size: Positive) -> Result<(), StrategyError> {
        apply_contract_size(
            &mut [
                &mut self.long_call,
                &mut self.short_call_low,
                &mut self.short_call_high,
            ],
            contract_size,
            "BullCallLadder::set_contract_size",
        )?;
        self.update_break_even_points()
    }
}

impl Strategies for BullCallLadder {
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        let max_profit = self.calculate_profit_at(&self.short_call_high.option.strike_price)?;
        if max_profit > Decimal::ZERO {
            Ok(Positive::new_decimal(max_profit)?)
        } else {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxProfitError {
                    reason: "Max profit is negative".to_string(),
                },
            ))
        }
    }

    fn get_max_loss(&self) -> Result<Positive, StrategyError> {
        Ok(Positive::MAX)
    }

    fn get_profit_area(&self) -> Result<Decimal, StrategyError> {
        let break_even = self.get_break_even_points()?;
        let [be0, be1] = break_even.as_slice() else {
            return Err(StrategyError::BreakEvenError(
                BreakEvenErrorKind::NoBreakEvenPoints,
            ));
        };
        let base_low_dec = d_sub(
            be1.to_dec(),
            be0.to_dec(),
            "BullCallLadder::get_profit_area base_low",
        )?;
        let base_low = Positive::new_decimal(base_low_dec)?;
        let max_profit = measured_max_profit(self)?;
        let short_high = self.short_call_high.option.strike_price.to_dec();
        let short_low = self.short_call_low.option.strike_price.to_dec();
        let base_high_dec = d_sub(
            short_high,
            short_low,
            "BullCallLadder::get_profit_area base_high",
        )?;
        let base_high = Positive::new_decimal(base_high_dec)?;
        decimal_from_f64((base_low.to_f64() + base_high.to_f64()) * max_profit.to_f64() / 2.0)
    }

    fn get_profit_ratio(&self) -> Result<Decimal, StrategyError> {
        // A zero or an unlimited (`Positive::MAX`) loss divides by one, so
        // the ratio is the profit itself in percent.
        let max_loss = match self.get_max_loss()? {
            value if value == Positive::ZERO || value == Positive::MAX => Positive::ONE,
            value => value,
        };
        let max_profit = measured_max_profit(self)?;
        Ok(Decimal::from(
            max_profit.checked_div(&max_loss)?.checked_mul_f64(100.0)?,
        ))
    }
}

impl Validable for BullCallLadder {
    fn validate(&self) -> bool {
        if self.name.is_empty() {
            error!("Symbol is required");
            return false;
        }
        if !self.long_call.validate() {
            return false;
        }
        if !self.short_call_low.validate() {
            return false;
        }
        if !self.short_call_high.validate() {
            return false;
        }
        if self.long_call.option.strike_price >= self.short_call_low.option.strike_price {
            debug!("Long call strike price must be less than short call strike price");
            return false;
        }
        if self.short_call_low.option.strike_price >= self.short_call_high.option.strike_price {
            debug!("Short call low strike price must be less than short call high strike price");
            return false;
        }
        true
    }
}

impl Optimizable for BullCallLadder {
    type Strategy = BullCallLadder;

    fn filter_combinations<'a>(
        &'a self,
        option_chain: &'a OptionChain,
        side: FindOptimalSide,
    ) -> impl Iterator<Item = OptionDataGroup<'a>> {
        let underlying_price = &self.long_call.option.underlying_price;
        let strategy = self.clone();
        option_chain
            .get_triple_iter()
            // Filter out invalid combinations based on FindOptimalSide
            .filter(move |(long, short_low, short_high)| {
                if side == FindOptimalSide::Center {
                    long.is_valid_optimal_side(underlying_price, &FindOptimalSide::Lower)
                        && short_low
                            .is_valid_optimal_side(underlying_price, &FindOptimalSide::Lower)
                        && short_high
                            .is_valid_optimal_side(underlying_price, &FindOptimalSide::Upper)
                } else {
                    long.is_valid_optimal_side(underlying_price, &side)
                        && short_low.is_valid_optimal_side(underlying_price, &side)
                        && short_high.is_valid_optimal_side(underlying_price, &side)
                }
            })
            // Filter out options with invalid bid/ask prices
            .filter(|(long, short_low, short_high)| {
                long.call_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && short_low.call_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && short_high.call_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
            })
            // Filter out options that don't meet strategy constraints
            .filter(move |(long, short_low, short_high)| {
                let legs = StrategyLegs::ThreeLegs {
                    first: long,
                    second: short_low,
                    third: short_high,
                };
                match strategy.create_strategy(option_chain, &legs) {
                    Ok(s) => s.validate() && s.get_max_profit().is_ok() && s.get_max_loss().is_ok(),
                    Err(_) => false,
                }
            })
            // Map to OptionDataGroup
            .map(move |(long, short_low, short_high)| {
                OptionDataGroup::Three(long, short_low, short_high)
            })
    }

    fn find_optimal(
        &mut self,
        option_chain: &OptionChain,
        side: FindOptimalSide,
        criteria: OptimizationCriteria,
    ) -> Result<(), StrategyError> {
        let mut best_value = Decimal::MIN;
        let mut found = false;
        let strategy_clone = self.clone();
        let options_iter = strategy_clone.filter_combinations(option_chain, side);

        for option_data_group in options_iter {
            // Unpack the OptionDataGroup into individual options
            let (long, short_low, short_high) = match option_data_group {
                OptionDataGroup::Three(first, second, third) => (first, second, third),
                other => {
                    tracing::warn!(
                        group = ?other,
                        "find_optimal: skipping unexpected OptionDataGroup variant"
                    );
                    continue;
                }
            };

            let legs = StrategyLegs::ThreeLegs {
                first: long,
                second: short_low,
                third: short_high,
            };
            let strategy = match self.create_strategy(option_chain, &legs) {
                Ok(s) => s,
                Err(e) => {
                    tracing::debug!(error = %e, "skipping invalid strategy combination");
                    continue;
                }
            };
            // Calculate the current value based on the optimization criteria
            let metric = match criteria {
                OptimizationCriteria::Ratio => strategy.get_profit_ratio(),
                OptimizationCriteria::Area => strategy.get_profit_area(),
            };
            let current_value = match metric {
                Ok(v) => v,
                Err(e) => {
                    tracing::debug!(error = %e, "skipping candidate with unscorable metric");
                    continue;
                }
            };

            if current_value > best_value {
                // Update the best value and replace the current strategy
                info!("Found better value: {}", current_value);
                best_value = current_value;
                *self = strategy.clone();
                found = true;
            }
        }

        if found {
            Ok(())
        } else {
            Err(StrategyError::no_valid_candidate(
                StrategyType::BullCallLadder,
            ))
        }
    }

    /// Constructs a `BullCallLadder` from the supplied chain and legs.
    ///
    /// # Errors
    ///
    /// Returns `StrategyError::OperationError` when the supplied legs are
    /// missing required quotes (`long_call.call_ask`,
    /// `short_call_low.call_bid`, `short_call_high.call_bid`) needed to
    /// price the strategy.
    fn create_strategy(
        &self,
        option_chain: &OptionChain,
        legs: &StrategyLegs,
    ) -> Result<BullCallLadder, StrategyError> {
        let (long_call, short_call_low, short_call_high) = match legs {
            StrategyLegs::ThreeLegs {
                first,
                second,
                third,
            } => (first, second, third),
            _ => {
                return Err(StrategyError::operation_not_supported(
                    "create_strategy",
                    "BullCallLadder requires exactly three legs (ThreeLegs)",
                ));
            }
        };

        if !long_call.validate() || !short_call_low.validate() || !short_call_high.validate() {
            return Err(StrategyError::invalid_parameters(
                "create_strategy",
                "one or more legs failed OptionData::validate",
            ));
        }
        let implied_volatility = long_call.implied_volatility;
        if implied_volatility > Positive::ONE {
            return Err(StrategyError::invalid_parameters(
                "create_strategy",
                &format!(
                    "implied volatility {implied_volatility} exceeds the supported maximum of 1.0"
                ),
            ));
        }
        let long_call_ask = long_call.call_ask.ok_or_else(|| {
            StrategyError::operation_not_supported(
                "create_strategy",
                "missing call_ask for long call leg",
            )
        })?;
        let short_call_low_bid = short_call_low.call_bid.ok_or_else(|| {
            StrategyError::operation_not_supported(
                "create_strategy",
                "missing call_bid for short_call_low leg",
            )
        })?;
        let short_call_high_bid = short_call_high.call_bid.ok_or_else(|| {
            StrategyError::operation_not_supported(
                "create_strategy",
                "missing call_bid for short_call_high leg",
            )
        })?;
        let mut strategy = BullCallLadder::new(
            option_chain.symbol.clone(),
            option_chain.underlying_price,
            long_call.strike_price,
            short_call_low.strike_price,
            short_call_high.strike_price,
            self.long_call.option.expiration_date,
            implied_volatility,
            self.long_call.option.risk_free_rate,
            self.long_call.option.dividend_yield,
            self.long_call.option.quantity,
            long_call_ask,
            short_call_low_bid,
            short_call_high_bid,
            self.long_call.open_fee,
            self.long_call.close_fee,
            self.short_call_low.open_fee,
            self.short_call_low.close_fee,
            self.short_call_high.open_fee,
            self.short_call_high.close_fee,
        )?;
        // The rebuilt legs keep the contract size of the strategy they
        // are rebuilt from.
        strategy.set_contract_size(self.get_contract_size()?)?;
        Ok(strategy)
    }
}

impl Profit for BullCallLadder {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, PricingError> {
        let price = Some(price);
        let long_call_itm_profit = self.long_call.pnl_at_expiration(&price)?;
        let long_call_otm_profit = self.short_call_low.pnl_at_expiration(&price)?;
        let short_call_profit = self.short_call_high.pnl_at_expiration(&price)?;
        Ok(d_sum(
            &[
                long_call_itm_profit,
                long_call_otm_profit,
                short_call_profit,
            ],
            "strategies::bull_call_ladder::profit_at",
        )?)
    }
}

impl ProbabilityAnalysis for BullCallLadder {
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_points = self.get_break_even_points()?;
        // A call ladder whose wings never cross zero profit has fewer than
        // two break-even points; the ranges below are bounded by both, so the
        // shortfall is reported rather than indexed past.
        let lower_break_even_point = *break_even_points.first().ok_or_else(|| {
            ProbabilityError::RangeError(ProfitLossRangeErrorKind::InvalidBreakEvenPoints {
                reason: "BullCallLadder has no lower break-even point".to_string(),
            })
        })?;
        let upper_break_even_point = *break_even_points.get(1).ok_or_else(|| {
            ProbabilityError::RangeError(ProfitLossRangeErrorKind::InvalidBreakEvenPoints {
                reason: "BullCallLadder has no upper break-even point".to_string(),
            })
        })?;
        let option = &self.long_call.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            self.long_call.option.implied_volatility,
            self.short_call_low.option.implied_volatility,
            self.short_call_high.option.implied_volatility,
        ])?;

        let mut profit_range = ProfitLossRange::new(
            Some(lower_break_even_point),
            Some(upper_break_even_point),
            Positive::ZERO,
        )?;

        profit_range.calculate_probability(
            &self.long_call.option.underlying_price,
            VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            },
            None,
            expiration_date,
            Some(risk_free_rate),
        )?;

        Ok(vec![profit_range])
    }

    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_points = self.get_break_even_points()?;
        // A call ladder whose wings never cross zero profit has fewer than
        // two break-even points; the ranges below are bounded by both, so the
        // shortfall is reported rather than indexed past.
        let lower_break_even_point = *break_even_points.first().ok_or_else(|| {
            ProbabilityError::RangeError(ProfitLossRangeErrorKind::InvalidBreakEvenPoints {
                reason: "BullCallLadder has no lower break-even point".to_string(),
            })
        })?;
        let upper_break_even_point = *break_even_points.get(1).ok_or_else(|| {
            ProbabilityError::RangeError(ProfitLossRangeErrorKind::InvalidBreakEvenPoints {
                reason: "BullCallLadder has no upper break-even point".to_string(),
            })
        })?;
        let option = &self.long_call.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            self.long_call.option.implied_volatility,
            self.short_call_low.option.implied_volatility,
            self.short_call_high.option.implied_volatility,
        ])?;

        let mut loss_range_lower =
            ProfitLossRange::new(None, Some(lower_break_even_point), Positive::ZERO)?;

        let mut loss_range_upper =
            ProfitLossRange::new(Some(upper_break_even_point), None, Positive::ZERO)?;

        loss_range_lower.calculate_probability(
            &self.long_call.option.underlying_price,
            VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            },
            None,
            expiration_date,
            Some(risk_free_rate),
        )?;

        loss_range_upper.calculate_probability(
            &self.long_call.option.underlying_price,
            VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            },
            None,
            expiration_date,
            Some(risk_free_rate),
        )?;

        Ok(vec![loss_range_lower, loss_range_upper])
    }
}

impl Greeks for BullCallLadder {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![
            &self.long_call.option,
            &self.short_call_low.option,
            &self.short_call_high.option,
        ])
    }
}

impl DeltaNeutrality for BullCallLadder {}

impl PnLCalculator for BullCallLadder {
    fn calculate_pnl(
        &self,
        market_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, PricingError> {
        // `PnL::try_add` adds the legs and reports a total that leaves the
        // `Positive` range; `PnL` has no `+` since #788.
        let mut total =
            self.long_call
                .calculate_pnl(market_price, expiration_date, implied_volatility)?;
        total = total.try_add(&self.short_call_low.calculate_pnl(
            market_price,
            expiration_date,
            implied_volatility,
        )?)?;
        total = total.try_add(&self.short_call_high.calculate_pnl(
            market_price,
            expiration_date,
            implied_volatility,
        )?)?;
        Ok(total)
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<PnL, PricingError> {
        // `PnL::try_add` adds the legs and reports a total that leaves the
        // `Positive` range; `PnL` has no `+` since #788.
        let mut total = self
            .long_call
            .calculate_pnl_at_expiration(underlying_price)?;
        total = total.try_add(
            &self
                .short_call_low
                .calculate_pnl_at_expiration(underlying_price)?,
        )?;
        total = total.try_add(
            &self
                .short_call_high
                .calculate_pnl_at_expiration(underlying_price)?,
        )?;
        Ok(total)
    }
}

#[cfg(test)]
crate::strategies::macros::test_strategy_traits!(BullCallLadder, test_short_call_implementations);

#[cfg(test)]
mod tests_bull_call_ladder {
    use super::*;

    use approx::assert_relative_eq;
    use optionstratlib_core::pos_or_panic;
    use rust_decimal_macros::dec;

    fn setup() -> BullCallLadder {
        BullCallLadder::new(
            "AAPL".to_string(),
            pos_or_panic!(150.0),
            pos_or_panic!(155.0),
            pos_or_panic!(157.5),
            pos_or_panic!(160.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.01),
            pos_or_panic!(0.02),
            Positive::ONE,
            pos_or_panic!(45.0),
            pos_or_panic!(20.5),
            pos_or_panic!(30.0),
            pos_or_panic!(0.1),
            pos_or_panic!(0.1),
            pos_or_panic!(0.1),
            pos_or_panic!(0.1),
            pos_or_panic!(0.1),
            pos_or_panic!(0.1),
        )
        .unwrap()
    }

    #[test]
    fn test_new() {
        let strategy = setup();
        assert_eq!(strategy.name, "AAPL");
        assert_eq!(strategy.kind, StrategyType::BullCallLadder);
        assert!(
            strategy
                .description
                .contains("A bull call ladder buys one call")
        );
    }

    #[test]
    fn test_get_break_even_points() {
        let strategy = setup();
        assert_eq!(strategy.get_break_even_points().unwrap()[0], 150.1);
    }

    #[test]
    fn test_calculate_profit_at() {
        let strategy = setup();
        let price = 172.0;
        assert!(strategy.calculate_profit_at(&pos_or_panic!(price)).unwrap() < Decimal::ZERO);
    }

    #[test]
    fn test_max_profit() {
        let strategy = setup();
        assert!(strategy.get_max_profit().unwrap_or(Positive::ZERO) > Positive::ZERO);
    }

    #[test]
    fn test_net_premium_received() {
        let strategy = setup();
        assert_relative_eq!(
            strategy.get_net_premium_received().unwrap().to_f64(),
            4.9,
            epsilon = 0.0001
        );
    }

    #[test]
    fn test_fees() {
        let strategy = setup();
        assert_relative_eq!(
            strategy.get_fees().unwrap().to_f64(),
            0.6,
            epsilon = f64::EPSILON
        );
    }
}

#[cfg(test)]
mod tests_bull_call_ladder_validation {
    use super::*;
    use optionstratlib_core::pos_or_panic;

    use rust_decimal_macros::dec;

    fn setup_basic_strategy() -> BullCallLadder {
        BullCallLadder::new(
            "AAPL".to_string(),
            pos_or_panic!(150.0),
            pos_or_panic!(145.0),
            pos_or_panic!(150.0),
            pos_or_panic!(155.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.01),
            pos_or_panic!(0.02),
            Positive::ONE,
            pos_or_panic!(7.0),
            pos_or_panic!(5.0),
            pos_or_panic!(3.0),
            pos_or_panic!(4.0),
            pos_or_panic!(0.1),
            pos_or_panic!(0.1),
            pos_or_panic!(0.1),
            pos_or_panic!(0.1),
            pos_or_panic!(0.1),
        )
        .unwrap()
    }

    #[test]
    fn test_validate_empty_symbol() {
        let mut strategy = setup_basic_strategy();
        strategy.name = "".to_string();
        assert!(!strategy.validate());
    }

    #[test]
    fn test_validate_valid_strategy() {
        let strategy = setup_basic_strategy();
        assert!(strategy.validate());
    }
}

#[cfg(test)]
mod tests_bull_call_ladder_delta {
    use super::*;
    use optionstratlib_core::assert_pos_relative_eq;
    use optionstratlib_core::pos_or_panic;

    use crate::strategies::bull_call_ladder::BullCallLadder;
    use crate::strategies::delta_neutral::DeltaNeutrality;
    use optionstratlib_analytics::pnl::DeltaAdjustment;
    use optionstratlib_core::assert_decimal_eq;
    use optionstratlib_core::model::types::OptionStyle;
    use optionstratlib_pricing::greeks::DELTA_THRESHOLD;
    use rust_decimal_macros::dec;

    fn get_strategy(underlying_price: Positive) -> BullCallLadder {
        BullCallLadder::new(
            "SP500".to_string(),
            underlying_price,      // underlying_price
            pos_or_panic!(5750.0), // long_strike_itm
            pos_or_panic!(5800.0), // long_strike_otm
            pos_or_panic!(5850.0), // short_strike
            ExpirationDate::Days(Positive::TWO),
            pos_or_panic!(0.18),  // implied_volatility
            dec!(0.05),           // risk_free_rate
            Positive::ZERO,       // dividend_yield
            Positive::ONE,        // long quantity
            pos_or_panic!(95.8),  // short_quantity
            pos_or_panic!(31.65), // premium_long_itm
            pos_or_panic!(85.04), // premium_long_otm
            pos_or_panic!(53.04), // premium_short
            pos_or_panic!(0.78),  // open_fee_long
            pos_or_panic!(0.73),  // close_fee_long
            pos_or_panic!(0.73),  // close_fee_short
            pos_or_panic!(0.78),  // close_fee_short
            pos_or_panic!(0.73),  // close_fee_short
        )
        .unwrap()
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos_or_panic!(5901.88));
        let size = dec!(-0.687410);
        let delta = pos_or_panic!(0.7040502965074396);
        let k = pos_or_panic!(5750.0);
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
                assert_pos_relative_eq!(
                    *quantity,
                    delta,
                    Positive::new_decimal(DELTA_THRESHOLD).unwrap()
                );
                assert_pos_relative_eq!(
                    *strike,
                    k,
                    Positive::new_decimal(DELTA_THRESHOLD).unwrap()
                );
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
        let strategy = get_strategy(pos_or_panic!(5781.88));
        let size = dec!(0.055904);
        let delta1 = pos_or_panic!(0.0833378661861126);
        // #706: the fixture used to put 5850 in `short_call_low` and 5800 in
        // `short_call_high`, which its own `validate` rejects. Ordered, the
        // first short leg the adjustment reaches is the 5800 call, so the
        // buy-back quantity is the net delta over that call's delta (was
        // 0.2835618144021385 on the 5850 call); the net-delta-zero check
        // below holds either way.
        let delta2 = pos_or_panic!(0.1338190182607754);
        let k1 = pos_or_panic!(5750.0);
        let k2 = pos_or_panic!(5800.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        match &binding[0] {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(
                    *quantity,
                    delta1,
                    Positive::new_decimal(DELTA_THRESHOLD).unwrap()
                );
                assert_pos_relative_eq!(
                    *strike,
                    k1,
                    Positive::new_decimal(DELTA_THRESHOLD).unwrap()
                );
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Long);
            }
            _ => panic!("Invalid suggestion"),
        }

        match &binding[1] {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(
                    *quantity,
                    delta2,
                    Positive::new_decimal(DELTA_THRESHOLD).unwrap()
                );
                assert_pos_relative_eq!(
                    *strike,
                    k2,
                    Positive::new_decimal(DELTA_THRESHOLD).unwrap()
                );
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut short_call_low = strategy.short_call_low.option.clone();
        let short_call_high = strategy.short_call_high.option.clone();
        let long_call = strategy.long_call.option.clone();
        short_call_low.quantity += delta2;

        let delta = short_call_low.delta().unwrap()
            + short_call_high.delta().unwrap()
            + long_call.delta().unwrap();
        assert_decimal_eq!(delta, Decimal::ZERO, DELTA_THRESHOLD);
    }

    #[test]
    fn create_test_no_adjustments() {
        let strategy = get_strategy(pos_or_panic!(5794.4));

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
mod tests_bull_call_ladder_delta_size {
    use super::*;
    use optionstratlib_core::assert_pos_relative_eq;
    use optionstratlib_core::pos_or_panic;

    use crate::strategies::bull_call_ladder::BullCallLadder;
    use crate::strategies::delta_neutral::DeltaNeutrality;
    use optionstratlib_analytics::pnl::DeltaAdjustment;
    use optionstratlib_core::assert_decimal_eq;
    use optionstratlib_core::model::types::OptionStyle;
    use optionstratlib_pricing::greeks::DELTA_THRESHOLD;
    use rust_decimal_macros::dec;

    fn get_strategy(underlying_price: Positive) -> BullCallLadder {
        BullCallLadder::new(
            "SP500".to_string(),
            underlying_price,      // underlying_price
            pos_or_panic!(5750.0), // long_strike_itm
            pos_or_panic!(5800.0), // long_strike_otm
            pos_or_panic!(5850.0), // short_strike
            ExpirationDate::Days(Positive::TWO),
            pos_or_panic!(0.18),  // implied_volatility
            dec!(0.05),           // risk_free_rate
            Positive::ZERO,       // dividend_yield
            Positive::ONE,        // long quantity
            pos_or_panic!(97.8),  // short_quantity
            pos_or_panic!(31.65), // premium_long_itm
            pos_or_panic!(85.04), // premium_long_otm
            pos_or_panic!(53.04), // premium_short
            pos_or_panic!(0.78),  // open_fee_long
            pos_or_panic!(0.73),  // close_fee_long
            pos_or_panic!(0.73),  // close_fee_short
            pos_or_panic!(0.78),  // close_fee_short
            pos_or_panic!(0.73),
        )
        .unwrap()
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos_or_panic!(5881.88));
        let size = dec!(-0.5699325);
        let delta = pos_or_panic!(0.5948524360242063);
        let k = pos_or_panic!(5750.0);
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
                assert_pos_relative_eq!(
                    *quantity,
                    delta,
                    Positive::new_decimal(DELTA_THRESHOLD).unwrap()
                );
                assert_pos_relative_eq!(
                    *strike,
                    k,
                    Positive::new_decimal(DELTA_THRESHOLD).unwrap()
                );
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
        let strategy = get_strategy(pos_or_panic!(5781.88));
        let size = dec!(0.05590);
        let delta1 = pos_or_panic!(0.0833378661861126);
        // #706: the fixture used to put 5850 in `short_call_low` and 5800 in
        // `short_call_high`, which its own `validate` rejects. Ordered, the
        // first short leg the adjustment reaches is the 5800 call, so the
        // buy-back quantity is the net delta over that call's delta (was
        // 0.2835618144021385 on the 5850 call); the net-delta-zero check
        // below holds either way.
        let delta2 = pos_or_panic!(0.1338190182607754);
        let k1 = pos_or_panic!(5750.0);
        let k2 = pos_or_panic!(5800.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();

        match &binding[0] {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(
                    *quantity,
                    delta1,
                    Positive::new_decimal(DELTA_THRESHOLD).unwrap()
                );
                assert_pos_relative_eq!(
                    *strike,
                    k1,
                    Positive::new_decimal(DELTA_THRESHOLD).unwrap()
                );
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Long);
            }
            _ => panic!("Invalid suggestion"),
        }

        match &binding[1] {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(
                    *quantity,
                    delta2,
                    Positive::new_decimal(DELTA_THRESHOLD).unwrap()
                );
                assert_pos_relative_eq!(
                    *strike,
                    k2,
                    Positive::new_decimal(DELTA_THRESHOLD).unwrap()
                );
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.short_call_low.option.clone();
        option.quantity = delta2;
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
        let strategy = get_strategy(pos_or_panic!(5794.4));

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
mod tests_bull_call_ladder_optimizable {
    use super::*;

    use approx::assert_relative_eq;
    use optionstratlib_core::pos_or_panic;
    use rust_decimal_macros::dec;

    fn create_test_option_chain() -> OptionChain {
        let mut chain = OptionChain::new(
            "TEST",
            Positive::HUNDRED,
            "2024-12-19".to_string(),
            None,
            None,
        );

        // Add options with different strikes
        chain.add_option(
            pos_or_panic!(95.0), // strike
            spos!(6.0),          // call_bid
            spos!(6.2),          // call_ask
            spos!(1.0),          // put_bid
            spos!(1.2),          // put_ask
            pos_or_panic!(0.2),  // iv
            Some(dec!(0.4)),     // delta
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0), // volume
            Some(50),     // open interest
            None,
        );

        chain.add_option(
            Positive::HUNDRED,
            spos!(3.0),
            spos!(3.2),
            spos!(3.0),
            spos!(3.2),
            pos_or_panic!(0.2),
            Some(dec!(0.5)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(200.0),
            Some(100),
            None,
        );

        chain.add_option(
            pos_or_panic!(105.0),
            spos!(1.0),
            spos!(1.2),
            spos!(6.0),
            spos!(6.2),
            pos_or_panic!(0.2),
            Some(dec!(0.6)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0),
            Some(50),
            None,
        );

        chain
    }

    fn setup_test_ladder() -> BullCallLadder {
        BullCallLadder::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(95.0),
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.01),
            pos_or_panic!(0.02),
            Positive::ONE,
            pos_or_panic!(6.2), // long call ask
            pos_or_panic!(3.0), // short call bid low
            Positive::ONE,      // short call bid high
            pos_or_panic!(0.1),
            pos_or_panic!(0.1),
            pos_or_panic!(0.1),
            pos_or_panic!(0.1),
            pos_or_panic!(0.1),
            pos_or_panic!(0.1),
        )
        .unwrap()
    }

    #[test]
    fn test_find_optimal_ratio() {
        let mut ladder = setup_test_ladder();
        let chain = create_test_option_chain();

        ladder
            .find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio)
            .unwrap();

        // Verify the optimization resulted in valid strikes
        assert!(ladder.long_call.option.strike_price < ladder.short_call_low.option.strike_price);
        assert!(
            ladder.short_call_low.option.strike_price < ladder.short_call_high.option.strike_price
        );

        // Verify the strategy is valid
        assert!(ladder.validate());
        assert!(ladder.get_max_profit().is_ok());
        assert!(ladder.get_max_loss().is_ok());
    }

    #[test]
    fn test_find_optimal_area() {
        let mut ladder = setup_test_ladder();
        let chain = create_test_option_chain();

        ladder
            .find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Area)
            .unwrap();

        // Verify the optimization resulted in valid strikes
        assert!(ladder.long_call.option.strike_price < ladder.short_call_low.option.strike_price);
        assert!(
            ladder.short_call_low.option.strike_price < ladder.short_call_high.option.strike_price
        );

        // Verify the strategy is valid
        assert!(ladder.validate());
        assert!(ladder.get_max_profit().is_ok());
        assert!(ladder.get_max_loss().is_ok());
    }

    #[test]
    fn test_create_strategy() {
        let ladder = setup_test_ladder();
        let chain = create_test_option_chain();

        let legs = StrategyLegs::ThreeLegs {
            first: chain.options.iter().next().unwrap(),
            second: chain.options.iter().nth(1).unwrap(),
            third: chain.options.iter().nth(2).unwrap(),
        };

        let new_strategy = ladder.create_strategy(&chain, &legs).unwrap();

        // Verify the new strategy has correct properties
        assert_relative_eq!(
            new_strategy.get_underlying_price().unwrap().to_f64(),
            100.0,
            epsilon = 0.001
        );
        assert!(new_strategy.validate());
    }

    #[test]
    fn test_create_strategy_invalid_legs() {
        let ladder = setup_test_ladder();
        let chain = create_test_option_chain();

        let legs = StrategyLegs::TwoLegs {
            first: chain.options.iter().next().unwrap(),
            second: chain.options.iter().nth(1).unwrap(),
        };

        // Wrong number of legs now returns a typed error (issue #323).
        let result = ladder.create_strategy(&chain, &legs);
        match result {
            Err(StrategyError::OperationError(OperationErrorKind::NotSupported {
                operation,
                ..
            })) => assert_eq!(operation, "create_strategy"),
            other => panic!("expected NotSupported error, got {other:?}"),
        }
    }

    #[test]
    fn test_filter_combinations_empty_chain() {
        let ladder = setup_test_ladder();
        let empty_chain = OptionChain::new(
            "TEST",
            Positive::HUNDRED,
            "2024-12-19".to_string(),
            None,
            None,
        );

        let combinations: Vec<_> = ladder
            .filter_combinations(&empty_chain, FindOptimalSide::All)
            .collect();

        assert!(
            combinations.is_empty(),
            "Empty chain should yield no combinations"
        );
    }
}

#[cfg(test)]
mod tests_bull_call_ladder_probability {
    use super::*;

    use num_traits::ToPrimitive;
    use optionstratlib_analytics::analytics::probability::PriceTrend;
    use optionstratlib_core::assert_pos_relative_eq;
    use optionstratlib_core::pos_or_panic;
    use rust_decimal_macros::dec;

    fn price_trend(
        drift_rate: rust_decimal::Decimal,
        confidence: rust_decimal::Decimal,
    ) -> PriceTrend {
        match PriceTrend::new(drift_rate, confidence) {
            Ok(trend) => trend,
            Err(e) => panic!("valid trend: {e}"),
        }
    }

    /// Creates a test Bull Call Ladder with standard parameters based on SP500
    fn create_test_ladder() -> BullCallLadder {
        BullCallLadder::new(
            "SP500".to_string(),
            pos_or_panic!(5781.88), // underlying_price
            pos_or_panic!(5750.0),  // long_call_strike
            pos_or_panic!(5800.0),  // short_call_low_strike
            pos_or_panic!(5850.0),  // short_call_high_strike
            ExpirationDate::Days(Positive::TWO),
            pos_or_panic!(0.18),  // implied_volatility
            dec!(0.05),           // risk_free_rate
            Positive::ZERO,       // dividend_yield
            pos_or_panic!(3.0),   // long quantity
            pos_or_panic!(85.04), // premium_long_itm
            pos_or_panic!(53.04), // premium_long_otm
            pos_or_panic!(28.85), // premium_short
            pos_or_panic!(0.78),  // premium_short
            pos_or_panic!(0.78),  // open_fee_long
            pos_or_panic!(0.78),  // close_fee_long
            pos_or_panic!(0.73),  // close_fee_short
            pos_or_panic!(0.73),  // close_fee_short
            pos_or_panic!(0.72),  // open_fee_short
        )
        .unwrap()
    }

    #[test]
    fn test_get_expiration() {
        let ladder = create_test_ladder();
        let expiration = *ladder.get_expiration().values().next().unwrap();
        assert_eq!(expiration, &ExpirationDate::Days(Positive::TWO));
    }

    #[test]
    fn test_get_risk_free_rate() {
        let ladder = create_test_ladder();
        assert_eq!(
            ladder
                .get_risk_free_rate()
                .unwrap()
                .values()
                .next()
                .unwrap()
                .to_f64()
                .unwrap(),
            0.05
        );
    }

    #[test]
    fn test_get_profit_ranges() {
        let ladder = create_test_ladder();
        let result = ladder.get_profit_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();

        assert_eq!(ranges.len(), 1);
        let range = &ranges[0];

        // Verify range bounds
        assert!(range.lower_bound.is_some());
        assert!(range.upper_bound.is_some());
        assert!(range.probability > Positive::ZERO);
        assert!(range.probability <= Positive::ONE);

        // Verify bounds are within strike prices
        assert!(range.lower_bound.unwrap() >= ladder.long_call.option.strike_price);
        assert!(range.upper_bound.unwrap() >= ladder.short_call_high.option.strike_price);
    }

    #[test]
    fn test_get_loss_ranges() {
        let ladder = create_test_ladder();
        let result = ladder.get_loss_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();

        assert_eq!(ranges.len(), 2); // Should have two loss ranges

        // Test lower loss range
        let lower_range = &ranges[0];
        assert!(lower_range.lower_bound.is_none());
        assert!(lower_range.upper_bound.is_some());
        assert!(lower_range.probability > Positive::ZERO);

        // Test upper loss range
        let upper_range = &ranges[1];
        assert!(upper_range.lower_bound.is_some());
        assert!(upper_range.upper_bound.is_none());
        assert!(upper_range.probability > Positive::ZERO);
    }

    #[test]
    fn test_probability_sum_to_one() {
        let ladder = create_test_ladder();

        let profit_ranges = ladder.get_profit_ranges().unwrap();
        let loss_ranges = ladder.get_loss_ranges().unwrap();

        let total_profit_prob: Positive = profit_ranges.iter().map(|r| r.probability).sum();

        let total_loss_prob: Positive = loss_ranges.iter().map(|r| r.probability).sum();

        assert_pos_relative_eq!(
            total_profit_prob + total_loss_prob,
            Positive::ONE,
            pos_or_panic!(0.0001)
        );
    }

    #[test]
    fn test_break_even_points_validity() {
        let ladder = create_test_ladder();
        let break_even_points = ladder.get_break_even_points().unwrap();

        assert_eq!(break_even_points.len(), 2);
        // Break-even points should be within strike prices
        assert!(break_even_points[0] >= ladder.long_call.option.strike_price);
        assert!(break_even_points[1] >= ladder.short_call_high.option.strike_price);
        // Break-even points should be between adjacent strikes
        assert!(break_even_points[0] < ladder.short_call_low.option.strike_price);
        assert!(break_even_points[1] > ladder.short_call_low.option.strike_price);
    }

    #[test]
    fn test_with_volatility_adjustment() {
        let ladder = create_test_ladder();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos_or_panic!(0.25),
            std_dev_adjustment: pos_or_panic!(0.05),
        });

        let prob = ladder.probability_of_profit(vol_adj, None);
        assert!(prob.is_ok());
        let probability = prob.unwrap();
        assert!(probability > Positive::ZERO);
        assert!(probability <= Positive::ONE);
    }

    #[test]
    fn test_with_price_trend() {
        let ladder = create_test_ladder();
        let trend = Some(price_trend(dec!(0.1), dec!(0.95)));

        let prob = ladder.probability_of_profit(None, trend);
        assert!(prob.is_ok());
        let probability = prob.unwrap();
        assert!(probability > Positive::ZERO);
        assert!(probability <= Positive::ONE);
    }

    #[test]
    fn test_analyze_probabilities() {
        let ladder = create_test_ladder();
        let analysis = ladder.analyze_probabilities(None, None).unwrap();

        assert!(analysis.probability_of_profit > Positive::ZERO);
        assert!(analysis.expected_value > Decimal::ZERO);
        assert_eq!(analysis.break_even_points.len(), 2);
        assert!(analysis.risk_reward_ratio > Positive::ZERO);
    }

    #[test]
    fn test_near_expiration() {
        let mut ladder = create_test_ladder();
        ladder.long_call.option.expiration_date = ExpirationDate::Days(pos_or_panic!(0.5));
        ladder.short_call_low.option.expiration_date = ExpirationDate::Days(pos_or_panic!(0.5));
        ladder.short_call_high.option.expiration_date = ExpirationDate::Days(pos_or_panic!(0.5));

        let prob = ladder.probability_of_profit(None, None).unwrap();
        // Near expiration probabilities should be more extreme
        assert!(prob < pos_or_panic!(0.3) || prob > pos_or_panic!(0.7));
    }

    #[test]
    fn test_high_volatility_scenario() {
        // This structure is short the two middle calls, so it is short
        // volatility: wider price distributions push the spot away from the
        // profit zone and the expected value falls. `None` prices at the
        // strategy's own implied volatility, so raising the legs' IV from
        // 0.18 to 0.5 must lower it. (Before #619 `None` priced at a hidden
        // flat 0.2 and the 0.5 below never reached the model.)
        let base = create_test_ladder();
        let base_ev = match base.expected_value(None, None) {
            Ok(ev) => ev,
            Err(error) => panic!("expected value at the base volatility: {error}"),
        };

        let mut ladder = create_test_ladder();
        ladder.long_call.option.implied_volatility = pos_or_panic!(0.5);
        ladder.short_call_low.option.implied_volatility = pos_or_panic!(0.5);
        ladder.short_call_high.option.implied_volatility = pos_or_panic!(0.5);
        let high_ev = match ladder.expected_value(None, None) {
            Ok(ev) => ev,
            Err(error) => panic!("expected value at 0.5 volatility: {error}"),
        };

        assert!(base_ev > Decimal::ZERO, "base expected value {base_ev}");
        assert!(
            high_ev < base_ev,
            "a short-volatility structure must lose expected value as volatility rises: {high_ev} vs {base_ev}"
        );
    }

    /// The case that surfaced #623 (then filed against `CallButterfly`, the
    /// name this ladder carried before #706). At 0.5 volatility the ladder
    /// loses money on average; the expected value used to floor that to
    /// zero, indistinguishable from a break-even structure. It is signed now.
    #[test]
    fn test_expected_value_negative_at_high_volatility() {
        let ladder = create_test_ladder();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos_or_panic!(0.5),
            std_dev_adjustment: Positive::ZERO,
        });
        match ladder.expected_value(vol_adj, None) {
            Ok(ev) => {
                assert!(ev < Decimal::ZERO, "expected value {ev}");
                assert_eq!(ev, dec!(-52.98441360311891782860));
            }
            Err(error) => panic!("expected value at 0.5 volatility: {error}"),
        }
    }

    #[test]
    fn test_extreme_probabilities() {
        let ladder = create_test_ladder();
        let result = ladder.calculate_extreme_probabilities(None, None);

        assert!(result.is_ok());
        let (max_profit_prob, max_loss_prob) = result.unwrap();

        assert!(max_profit_prob >= Positive::ZERO);
        assert!(max_loss_prob >= Positive::ZERO);
        assert!(max_profit_prob + max_loss_prob <= Positive::ONE);
    }
}

#[cfg(test)]
mod tests_bull_call_ladder_position_management {
    use super::*;
    use optionstratlib_core::pos_or_panic;

    use optionstratlib_core::error::position::PositionValidationErrorKind;
    use optionstratlib_core::model::types::{OptionStyle, Side};

    use rust_decimal_macros::dec;

    fn create_test_bull_call_ladder() -> BullCallLadder {
        BullCallLadder::new(
            "SP500".to_string(),
            pos_or_panic!(5781.88), // underlying_price
            pos_or_panic!(5750.0),  // long_call_strike
            pos_or_panic!(5800.0),  // short_call_low_strike
            pos_or_panic!(5850.0),  // short_call_high_strike
            ExpirationDate::Days(Positive::TWO),
            pos_or_panic!(0.18),  // implied_volatility
            dec!(0.05),           // risk_free_rate
            Positive::ZERO,       // dividend_yield
            pos_or_panic!(3.0),   // long quantity
            pos_or_panic!(85.04), // premium_long_itm
            pos_or_panic!(53.04), // premium_long_otm
            pos_or_panic!(28.85), // premium_short
            pos_or_panic!(0.78),  // premium_short
            pos_or_panic!(0.78),  // open_fee_long
            pos_or_panic!(0.78),  // close_fee_long
            pos_or_panic!(0.73),  // close_fee_short
            pos_or_panic!(0.73),  // close_fee_short
            pos_or_panic!(0.72),  // open_fee_short
        )
        .unwrap()
    }

    #[test]
    fn test_bull_call_ladder_short_get_position() {
        let mut ladder = create_test_bull_call_ladder();

        // Test getting short call position
        let call_position =
            ladder.get_position(&OptionStyle::Call, &Side::Short, &pos_or_panic!(5800.0));
        assert!(call_position.is_ok());
        let positions = call_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos_or_panic!(5800.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Short);

        // Test getting short put position
        let put_position =
            ladder.get_position(&OptionStyle::Call, &Side::Short, &pos_or_panic!(5850.0));
        assert!(put_position.is_ok());
        let positions = put_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos_or_panic!(5850.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Short);

        // Test getting non-existent position
        let invalid_position =
            ladder.get_position(&OptionStyle::Call, &Side::Short, &pos_or_panic!(2715.0));
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
    fn test_bull_call_ladder_long_get_position() {
        let mut ladder = create_test_bull_call_ladder();

        // Test getting short call position
        let call_position =
            ladder.get_position(&OptionStyle::Call, &Side::Long, &pos_or_panic!(5750.0));
        assert!(call_position.is_ok());
        let positions = call_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos_or_panic!(5750.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Long);

        // Test getting non-existent position
        let invalid_position =
            ladder.get_position(&OptionStyle::Call, &Side::Long, &pos_or_panic!(2715.0));
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
    fn test_bull_call_ladder_short_modify_position() {
        let mut ladder = create_test_bull_call_ladder();

        // Modify short call position
        let mut modified_call = ladder.short_call_low.clone();
        modified_call.option.quantity = Positive::TWO;
        let result = ladder.modify_position(&modified_call);
        assert!(result.is_ok());
        assert_eq!(ladder.short_call_low.option.quantity, Positive::TWO);

        // Modify short put position
        let mut modified_put = ladder.short_call_high.clone();
        modified_put.option.quantity = Positive::TWO;
        let result = ladder.modify_position(&modified_put);
        assert!(result.is_ok());
        assert_eq!(ladder.short_call_high.option.quantity, Positive::TWO);

        // Test modifying with invalid position
        let mut invalid_position = ladder.short_call_high.clone();
        invalid_position.option.strike_price = pos_or_panic!(95.0);
        let result = ladder.modify_position(&invalid_position);
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
    fn test_bull_call_ladder_long_modify_position() {
        let mut ladder = create_test_bull_call_ladder();

        // Modify long call position
        let mut modified_call = ladder.long_call.clone();
        modified_call.option.quantity = Positive::TWO;
        let result = ladder.modify_position(&modified_call);
        assert!(result.is_ok());
        assert_eq!(ladder.long_call.option.quantity, Positive::TWO);

        // Test modifying with invalid position
        let mut invalid_position = ladder.long_call.clone();
        invalid_position.option.strike_price = pos_or_panic!(95.0);
        let result = ladder.modify_position(&invalid_position);
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
    use optionstratlib_core::pos_or_panic;

    use optionstratlib_core::model::types::{OptionStyle, Side};

    use rust_decimal_macros::dec;

    // Helper function to create a test strategy
    fn create_test_strategy() -> BullCallLadder {
        BullCallLadder::new(
            "SP500".to_string(),
            pos_or_panic!(5781.88), // underlying_price
            pos_or_panic!(5750.0),  // long_call_strike
            pos_or_panic!(5800.0),  // short_call_low_strike
            pos_or_panic!(5850.0),  // short_call_high_strike
            ExpirationDate::Days(Positive::TWO),
            pos_or_panic!(0.18),  // implied_volatility
            dec!(0.05),           // risk_free_rate
            Positive::ZERO,       // dividend_yield
            pos_or_panic!(3.0),   // long quantity
            pos_or_panic!(85.04), // premium_long_itm
            pos_or_panic!(53.04), // premium_long_otm
            pos_or_panic!(28.85), // premium_short
            pos_or_panic!(0.78),  // premium_short
            pos_or_panic!(0.78),  // open_fee_long
            pos_or_panic!(0.78),  // close_fee_long
            pos_or_panic!(0.73),  // close_fee_short
            pos_or_panic!(0.73),  // close_fee_short
            pos_or_panic!(0.72),  // open_fee_short
        )
        .unwrap()
    }

    #[test]
    fn test_adjust_existing_call_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.short_call_low.option.quantity;
        let adjustment = Positive::ONE;

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos_or_panic!(5800.0),
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
    fn test_adjust_existing_long_call_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.long_call.option.quantity;
        let adjustment = Positive::ONE;

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos_or_panic!(5750.0),
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

        // Try to adjust a non-existent long call position
        let result = strategy.adjust_option_position(
            Decimal::ONE,
            &pos_or_panic!(110.0),
            &OptionStyle::Call,
            &Side::Long,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        // StrategyError wraps PositionError, so we check the error message
        assert!(err.to_string().contains("Strike not found in positions"));
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
        let initial_quantity = strategy.long_call.option.quantity;

        let result = strategy.adjust_option_position(
            Decimal::ZERO,
            &pos_or_panic!(5750.0),
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
    use optionstratlib_core::pos_or_panic;

    use optionstratlib_core::model::utils::create_sample_position;

    fn call(side: Side, strike: Positive) -> Position {
        create_sample_position(
            OptionStyle::Call,
            side,
            Positive::HUNDRED,
            Positive::ONE,
            strike,
            pos_or_panic!(0.2),
        )
    }

    #[test]
    fn test_get_strategy_valid() {
        // Long the lowest strike, short the middle and the highest (#706).
        let result = BullCallLadder::get_strategy(&[
            call(Side::Short, pos_or_panic!(105.0)),
            call(Side::Long, pos_or_panic!(95.0)),
            call(Side::Short, Positive::HUNDRED),
        ]);
        let ladder = result.unwrap();
        assert_eq!(ladder.long_call.option.strike_price, pos_or_panic!(95.0));
        assert_eq!(ladder.short_call_low.option.strike_price, Positive::HUNDRED);
        assert_eq!(
            ladder.short_call_high.option.strike_price,
            pos_or_panic!(105.0)
        );
        assert!(ladder.validate());
    }

    #[test]
    fn test_bull_call_ladder_get_strategy_old_convention_rejected() {
        // Short low / long middle / short high was what this builder took as
        // `CallButterfly` before #706; it fails the ladder's own `validate`.
        let result = BullCallLadder::get_strategy(&[
            call(Side::Short, pos_or_panic!(95.0)),
            call(Side::Long, Positive::HUNDRED),
            call(Side::Short, pos_or_panic!(105.0)),
        ]);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Bull Call Ladder get_strategy"
                && reason == "Bull Call Ladder requires a long lowest-strike call and two short higher-strike calls"
        ));
    }

    #[test]
    fn test_bull_call_ladder_get_strategy_invalid_leg_rejected() {
        let mut short_high = call(Side::Short, pos_or_panic!(105.0));
        short_high.premium = Positive::ZERO;
        let result = BullCallLadder::get_strategy(&[
            call(Side::Long, pos_or_panic!(95.0)),
            call(Side::Short, Positive::HUNDRED),
            short_high,
        ]);
        assert!(matches!(
            result,
            Err(StrategyError::InvalidStrategy {
                strategy: StrategyType::BullCallLadder,
                ..
            })
        ));
    }

    #[test]
    fn test_bull_call_ladder_get_strategy_equal_short_strikes_rejected() {
        let result = BullCallLadder::get_strategy(&[
            call(Side::Long, pos_or_panic!(95.0)),
            call(Side::Short, Positive::HUNDRED),
            call(Side::Short, Positive::HUNDRED),
        ]);
        assert!(matches!(
            result,
            Err(StrategyError::InvalidStrategy {
                strategy: StrategyType::BullCallLadder,
                ..
            })
        ));
    }

    #[test]
    fn test_get_strategy_wrong_number_of_options() {
        let result = BullCallLadder::get_strategy(&[
            call(Side::Long, pos_or_panic!(95.0)),
            call(Side::Short, Positive::HUNDRED),
        ]);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Bull Call Ladder get_strategy" && reason == "Must have exactly 3 options"
        ));
    }

    #[test]
    fn test_get_strategy_wrong_option_style() {
        let mut put = call(Side::Long, pos_or_panic!(95.0));
        put.option.option_style = OptionStyle::Put;
        let result = BullCallLadder::get_strategy(&[
            put,
            call(Side::Short, Positive::HUNDRED),
            call(Side::Short, pos_or_panic!(105.0)),
        ]);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Bull Call Ladder get_strategy" && reason == "Options must be calls"
        ));
    }

    #[test]
    fn test_get_strategy_wrong_sides() {
        let result = BullCallLadder::get_strategy(&[
            call(Side::Long, pos_or_panic!(95.0)),
            call(Side::Long, Positive::HUNDRED),
            call(Side::Short, pos_or_panic!(105.0)),
        ]);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Bull Call Ladder get_strategy"
                && reason == "Bull Call Ladder requires a long lowest-strike call and two short higher-strike calls"
        ));
    }

    #[test]
    fn test_bull_call_ladder_add_position_orders_the_short_calls() {
        // Through `add_position` alone, in either order, the lower short
        // strike lands in `short_call_low` (#706).
        for shorts in [
            [Positive::HUNDRED, pos_or_panic!(105.0)],
            [pos_or_panic!(105.0), Positive::HUNDRED],
        ] {
            let mut ladder = BullCallLadder::default();
            ladder
                .add_position(&call(Side::Long, pos_or_panic!(95.0)))
                .unwrap();
            for strike in shorts {
                ladder.add_position(&call(Side::Short, strike)).unwrap();
            }
            assert_eq!(ladder.long_call.option.strike_price, pos_or_panic!(95.0));
            assert_eq!(ladder.short_call_low.option.strike_price, Positive::HUNDRED);
            assert_eq!(
                ladder.short_call_high.option.strike_price,
                pos_or_panic!(105.0)
            );
            assert!(ladder.validate());
        }
    }

    #[test]
    fn test_get_strategy_different_expiration_dates() {
        let mut options = vec![
            call(Side::Long, pos_or_panic!(95.0)),
            call(Side::Short, Positive::HUNDRED),
            call(Side::Short, pos_or_panic!(105.0)),
        ];
        options[1].option.expiration_date = ExpirationDate::Days(pos_or_panic!(60.0));

        let result = BullCallLadder::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Bull Call Ladder get_strategy" && reason == "Options must have the same expiration date"
        ));
    }
}

#[cfg(test)]
mod tests_bull_call_ladder_pnl {
    use super::*;
    use optionstratlib_core::pos_or_panic;

    use optionstratlib_core::assert_decimal_eq;
    use optionstratlib_core::model::utils::create_sample_position;
    use rust_decimal_macros::dec;

    fn setup_test_strategy() -> BullCallLadder {
        BullCallLadder::new(
            "AAPL".to_string(),
            pos_or_panic!(150.0),
            pos_or_panic!(145.0),
            pos_or_panic!(150.0),
            pos_or_panic!(155.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.01),
            pos_or_panic!(0.02),
            Positive::ONE,
            pos_or_panic!(7.0),
            pos_or_panic!(5.0),
            pos_or_panic!(3.0),
            pos_or_panic!(4.0),
            pos_or_panic!(0.1),
            pos_or_panic!(0.1),
            pos_or_panic!(0.1),
            pos_or_panic!(0.1),
            pos_or_panic!(0.1),
        )
        .unwrap()
    }

    // Long the 95 call for 7.50, short the 100 call for 4.50 and the 105 call
    // for 2.40, no fees. Net debit 0.60, so the hand-computed expiry payoff is
    // -0.60 up to 95, rises one for one to +4.40 at 100, holds it to 105 and
    // falls one for one above: break-evens 95.60 and 105 + 4.40 = 109.40.
    fn create_test_ladder() -> Result<BullCallLadder, StrategyError> {
        let leg = |side: Side, strike: Positive, premium: Positive| {
            let mut position = create_sample_position(
                OptionStyle::Call,
                side,
                Positive::HUNDRED,
                Positive::ONE,
                strike,
                pos_or_panic!(0.2),
            );
            position.premium = premium;
            position.open_fee = Positive::ZERO;
            position.close_fee = Positive::ZERO;
            position
        };
        BullCallLadder::get_strategy(&[
            leg(Side::Long, pos_or_panic!(95.0), pos_or_panic!(7.5)),
            leg(Side::Short, Positive::HUNDRED, pos_or_panic!(4.5)),
            leg(Side::Short, pos_or_panic!(105.0), pos_or_panic!(2.4)),
        ])
    }

    #[test]
    fn test_bull_call_ladder_hand_computed_break_evens() {
        let ladder = create_test_ladder().unwrap();
        assert_eq!(
            ladder.get_break_even_points().unwrap(),
            &vec![pos_or_panic!(95.6), pos_or_panic!(109.4)]
        );
        assert_decimal_eq!(ladder.get_net_cost().unwrap(), dec!(0.6), dec!(1e-9));
        assert_eq!(ladder.get_max_profit().unwrap(), pos_or_panic!(4.4));
        // Net short one call above the top strike: the loss is unlimited.
        assert_eq!(ladder.get_max_loss().unwrap(), Positive::MAX);
    }

    #[test]
    fn test_calculate_pnl_at_expiration_below_strikes() {
        let ladder = create_test_ladder().unwrap();
        // Every call expires worthless: the debit is lost.
        let pnl = ladder
            .calculate_pnl_at_expiration(&pos_or_panic!(90.0))
            .unwrap();
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(-0.6), dec!(1e-6));
        assert_eq!(pnl.initial_income, pos_or_panic!(6.9));
        assert_eq!(pnl.initial_costs, pos_or_panic!(7.5));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_between_strikes() {
        let ladder = create_test_ladder().unwrap();
        // 97.5: the long 95 call pays 2.50, less the 0.60 debit.
        let pnl = ladder
            .calculate_pnl_at_expiration(&pos_or_panic!(97.5))
            .unwrap();
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(1.9), dec!(1e-6));
        // 95.6: the lower break-even.
        let pnl = ladder
            .calculate_pnl_at_expiration(&pos_or_panic!(95.6))
            .unwrap();
        assert_decimal_eq!(pnl.realized.unwrap(), Decimal::ZERO, dec!(1e-6));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_max_profit() {
        let ladder = create_test_ladder().unwrap();
        // Anywhere from 100 to 105: 5.00 of spread, less the 0.60 debit.
        for price in [
            Positive::HUNDRED,
            pos_or_panic!(102.5),
            pos_or_panic!(105.0),
        ] {
            let pnl = ladder.calculate_pnl_at_expiration(&price).unwrap();
            assert_decimal_eq!(pnl.realized.unwrap(), dec!(4.4), dec!(1e-6));
        }
    }

    #[test]
    fn test_calculate_pnl_at_expiration_above_strikes() {
        let ladder = create_test_ladder().unwrap();
        // 109.4: the upper break-even.
        let pnl = ladder
            .calculate_pnl_at_expiration(&pos_or_panic!(109.4))
            .unwrap();
        assert_decimal_eq!(pnl.realized.unwrap(), Decimal::ZERO, dec!(1e-6));
        // 115: 4.40 - (115 - 105) = -5.60, and falling one for one.
        let pnl = ladder
            .calculate_pnl_at_expiration(&pos_or_panic!(115.0))
            .unwrap();
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(-5.6), dec!(1e-6));
    }

    #[test]
    fn test_calculate_pnl_rally_from_entry() {
        let ladder = create_test_ladder().unwrap();
        // `unrealized` is the change in the legs' Black-Scholes value since
        // entry at 100. Net short one call, the ladder loses on a sharp rally.
        let pnl = ladder
            .calculate_pnl(
                &pos_or_panic!(120.0),
                ExpirationDate::Days(pos_or_panic!(20.0)),
                &pos_or_panic!(0.2),
            )
            .unwrap();
        assert!(pnl.unrealized.unwrap() < Decimal::ZERO);
    }

    #[test]
    fn test_profit_below_lower_strike() {
        let strategy = setup_test_strategy();
        let profit = strategy.calculate_profit_at(&pos_or_panic!(140.0)).unwrap();
        assert!(profit <= Decimal::ZERO);
    }

    #[test]
    fn test_profit_above_upper_strike() {
        let strategy = setup_test_strategy();
        let profit = strategy.calculate_profit_at(&pos_or_panic!(160.0)).unwrap();
        assert!(profit <= Decimal::ZERO);
    }

    #[test]
    fn test_profit_ratio() {
        let strategy = setup_test_strategy();
        let ratio = strategy.get_profit_ratio().unwrap();
        assert!(ratio > Decimal::ZERO);
    }
}
