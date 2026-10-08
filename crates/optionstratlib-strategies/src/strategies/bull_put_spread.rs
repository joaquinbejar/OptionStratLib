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
    BreakEvenable, Optimizable, Positionable, Strategable, StrategyBasics, StrategyType, Validable,
};
use super::shared::SpreadStrategy;
use crate::error::strategies::{ProfitLossErrorKind, StrategyError};
use crate::strategies::base::{lower_break_even, price_gap};
use crate::strategies::shared::decimal_from_f64;
use crate::strategies::shared::{
    CachedBreakEvens, apply_contract_size, common_contract_size, edit_refreshing_break_evens,
};
use crate::strategies::shared::{measured_max_loss, measured_max_profit};
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
    decimal::{d_div, d_mul, d_sub, d_sum},
    position::Position,
    types::{OptionBasicType, OptionStyle, OptionType, Side},
    utils::mean_and_std,
};
#[cfg(test)]
use optionstratlib_core::pos_or_panic;
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
use tracing::debug;

/// The default description for the Bull Put Spread strategy.
pub const BULL_PUT_SPREAD_DESCRIPTION: &str = "A bull put spread is created by buying a put option with a lower strike price \
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
#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
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
    pub long_put: Position,

    /// The short put position (higher strike price) that generates premium income
    pub short_put: Position,
}

impl_json_debug_pretty!(BullPutSpread);
impl_json_display!(BullPutSpread);

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
    ///
    /// # Errors
    ///
    /// Returns `StrategyError::InvalidStrategy` when the assembled strategy
    /// fails its own `validate` (#696): the long strike is not below the short
    /// strike, or a leg fails `Position::validate` (for example a short put
    /// with no premium).
    ///
    /// Returns `StrategyError` if either freshly-constructed leg cannot be
    /// added to the strategy or if the break-even calculation fails. In
    /// practice these branches are unreachable for a freshly-built bull
    /// put spread and are surfaced only to keep the constructor
    /// panic-free.
    #[allow(clippy::too_many_arguments)]
    #[inline(never)]
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
    ) -> Result<Self, StrategyError> {
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
            None,
            None,
        );
        strategy.place_leg(&long_put)?;

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
            None,
            None,
        );
        strategy.place_leg(&short_put)?;

        if !strategy.validate() {
            return Err(StrategyError::invalid_strategy(
                StrategyType::BullPutSpread,
                "the legs built by `new` fail validation",
            ));
        }

        strategy.update_break_even_points()?;

        Ok(strategy)
    }
}

impl StrategyConstructor for BullPutSpread {
    fn get_strategy(vec_positions: &[Position]) -> Result<Self, StrategyError> {
        // Need exactly 2 options for a bull put spread
        if vec_positions.len() != 2 {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Bull Put Spread get_strategy".to_string(),
                    reason: "Must have exactly 2 options".to_string(),
                },
            ));
        }

        // Sort options by strike price to identify short and long positions
        let mut sorted_positions = vec_positions.to_vec();
        // SAFETY: total order on Positive; f64 fallback to Equal is safe for stable sort
        sorted_positions.sort_by(|a, b| {
            a.option
                .strike_price
                .partial_cmp(&b.option.strike_price)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let [lower_strike_option, higher_strike_option] = sorted_positions.as_slice() else {
            return Err(StrategyError::invalid_parameters(
                "Bull Put Spread get_strategy",
                "Must have exactly 2 options",
            ));
        };

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

        // Validate option sides - long lower strike put, short higher strike put
        if lower_strike_option.option.side != Side::Long
            || higher_strike_option.option.side != Side::Short
        {
            return Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters {
                operation: "Bull Put Spread get_strategy".to_string(),
                reason: "Bull Put Spread requires a long lower strike put and a short higher strike put".to_string(),
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
        let long_put = Position::new(
            lower_strike_option.option.clone(),
            lower_strike_option.premium,
            Utc::now(),
            lower_strike_option.open_fee,
            lower_strike_option.close_fee,
            lower_strike_option.epic.clone(),
            lower_strike_option.extra_fields.clone(),
        );

        let short_put = Position::new(
            higher_strike_option.option.clone(),
            higher_strike_option.premium,
            Utc::now(),
            higher_strike_option.open_fee,
            higher_strike_option.close_fee,
            higher_strike_option.epic.clone(),
            higher_strike_option.extra_fields.clone(),
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
        if !strategy.validate() {
            return Err(StrategyError::invalid_strategy(
                StrategyType::BullPutSpread,
                "the positions passed to `get_strategy` fail validation",
            ));
        }
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

        // The net cost per contract, which on this credit spread is negative
        // and moves the break-even below the short strike. `lower_break_even`
        // takes the distance down from the strike and floors it at zero, where
        // a credit above the strike leaves no attainable losing price.
        let per_contract = d_div(
            self.get_net_cost()?,
            self.short_put.option.position_size()?.to_dec(),
            "BullPutSpread::update_break_even_points",
        )?;
        self.break_even_points.push(
            lower_break_even(self.short_put.option.strike_price, -per_contract)
                .checked_round_to(2)?,
        );

        Ok(())
    }
}

impl BullPutSpread {
    /// Places `position` in the leg its side and style select, without
    /// refreshing the break-evens: the constructors fill the legs through
    /// it, and [`Positionable::add_position`] wraps it.
    fn place_leg(&mut self, position: &Position) -> Result<(), PositionError> {
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

impl CachedBreakEvens for BullPutSpread {
    fn break_evens_mut(&mut self) -> &mut Vec<Positive> {
        &mut self.break_even_points
    }
}

impl Positionable for BullPutSpread {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        edit_refreshing_break_evens(self, |strategy| strategy.place_leg(position))
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
        edit_refreshing_break_evens(self, |strategy| strategy.replace_leg(position))
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

impl BasicAble for BullPutSpread {
    fn get_title(&self) -> String {
        let strategy_title = format!("{:?} Strategy: ", self.kind);
        let leg_titles: Vec<String> = [self.long_put.get_title(), self.short_put.get_title()]
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
        let long_put = &self.long_put.option;
        let short_put = &self.short_put.option;
        hash_set.insert(OptionBasicType {
            option_style: &long_put.option_style,
            side: &long_put.side,
            strike_price: &long_put.strike_price,
            expiration_date: &long_put.expiration_date,
        });
        hash_set.insert(OptionBasicType {
            option_style: &short_put.option_style,
            side: &short_put.side,
            strike_price: &short_put.strike_price,
            expiration_date: &short_put.expiration_date,
        });

        hash_set
    }
    fn get_implied_volatility(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let options = [
            (
                &self.long_put.option,
                &self.long_put.option.implied_volatility,
            ),
            (
                &self.short_put.option,
                &self.short_put.option.implied_volatility,
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
            (&self.long_put.option, &self.long_put.option.quantity),
            (&self.short_put.option, &self.short_put.option.quantity),
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
        self.long_put.one_option()
    }
    fn one_option_mut(&mut self) -> Result<&mut Options, StrategyError> {
        self.long_put.one_option_mut()
    }
    fn set_expiration_date(
        &mut self,
        expiration_date: ExpirationDate,
    ) -> Result<(), StrategyError> {
        self.long_put.option.expiration_date = expiration_date;
        self.short_put.option.expiration_date = expiration_date;
        Ok(())
    }
    fn set_underlying_price(&mut self, price: &Positive) -> Result<(), StrategyError> {
        self.long_put.option.underlying_price = *price;
        self.long_put.premium =
            Positive::new_decimal(self.long_put.option.calculate_price_black_scholes()?.abs())?;
        self.short_put.option.underlying_price = *price;
        self.short_put.premium =
            Positive::new_decimal(self.short_put.option.calculate_price_black_scholes()?.abs())?;
        Ok(())
    }
    fn set_implied_volatility(&mut self, volatility: &Positive) -> Result<(), StrategyError> {
        self.long_put.option.implied_volatility = *volatility;
        self.short_put.option.implied_volatility = *volatility;
        self.long_put.premium =
            Positive::new_decimal(self.long_put.option.calculate_price_black_scholes()?.abs())?;
        self.short_put.premium =
            Positive::new_decimal(self.short_put.option.calculate_price_black_scholes()?.abs())?;
        Ok(())
    }
    fn get_contract_size(&self) -> Result<Positive, StrategyError> {
        common_contract_size(
            &[&self.long_put, &self.short_put],
            "BullPutSpread::get_contract_size",
        )
    }
    fn set_contract_size(&mut self, contract_size: Positive) -> Result<(), StrategyError> {
        apply_contract_size(
            &mut [&mut self.long_put, &mut self.short_put],
            contract_size,
            "BullPutSpread::set_contract_size",
        )?;
        self.update_break_even_points()
    }
}

impl Strategies for BullPutSpread {
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        let net_premium_received = self.get_net_premium_received()?;
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
    fn get_max_loss(&self) -> Result<Positive, StrategyError> {
        let short_strike = self.short_put.option.strike_price.to_dec();
        let long_strike = self.long_put.option.strike_price.to_dec();
        let width = short_strike - long_strike;
        // An inverted vertical is not a bull put spread: a structural
        // failure, not a report on the sign of the loss (#803).
        if width < Decimal::ZERO {
            return Err(StrategyError::invalid_strategy(
                StrategyType::BullPutSpread,
                "get_max_loss: the short put strike must be above the long put strike",
            ));
        }
        let qty = self.short_put.option.position_size()?.to_dec();
        let net_prem = self.get_net_premium_received()?.to_dec();
        let exposure = d_mul(width, qty, "BullPutSpread::get_max_loss")?;
        let max_loss_dec = d_sub(exposure, net_prem, "BullPutSpread::get_max_loss")?;
        Positive::new_decimal(max_loss_dec).map_err(|_| {
            StrategyError::ProfitLossError(ProfitLossErrorKind::MaxLossError {
                reason: "Max loss is negative".to_string(),
            })
        })
    }
    fn get_profit_area(&self) -> Result<Decimal, StrategyError> {
        let high = measured_max_profit(self)?;
        let break_even = *self.break_even_points.first().ok_or_else(|| {
            StrategyError::empty_collection("BullPutSpread::get_profit_area: no break-even points")
        })?;
        // The distance between the strike and the break-even, in whichever
        // order the two fall.
        let strike = self.short_put.option.strike_price;
        let base = price_gap(strike.max(break_even), strike.min(break_even));
        decimal_from_f64(high.to_f64() * base.to_f64() / 200.0)
    }
    fn get_profit_ratio(&self) -> Result<Decimal, StrategyError> {
        let max_profit = measured_max_profit(self)?;
        let max_loss = measured_max_loss(self)?;
        match (max_profit, max_loss) {
            (value, _) if value == Positive::ZERO => Ok(Decimal::ZERO),
            (_, value) if value == Positive::ZERO => Ok(Decimal::MAX),
            _ => decimal_from_f64(max_profit.to_f64() / max_loss.to_f64() * 100.0),
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
    /// 1. Computes the underlying price via `&self.long_put.option.underlying_price`.
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
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use rust_decimal_macros::dec;
    /// use tracing::info;
    /// use optionstratlib_market::chains::chain::OptionChain;
    /// use optionstratlib_market::chains::utils::OptionDataGroup;
    /// use optionstratlib_core::model::ExpirationDate;
    /// use optionstratlib_core::model::Positive;
    /// use optionstratlib_core::pos_or_panic;
    /// use optionstratlib_strategies::strategies::base::Optimizable;
    /// use optionstratlib_strategies::strategies::bull_put_spread::BullPutSpread;
    /// use optionstratlib_market::chains::utils::FindOptimalSide;
    ///
    /// let underlying_price = pos_or_panic!(5810.0);
    /// let option_chain = OptionChain::new("TEST", underlying_price, "2024-01-01".to_string(), None, None);
    /// let bull_put_spread_strategy = BullPutSpread::new(
    ///         "SP500".to_string(),
    ///         underlying_price,   // underlying_price
    ///         pos_or_panic!(5750.0),   // long_strike
    ///         pos_or_panic!(5920.0),   // short_strike
    ///         ExpirationDate::Days(Positive::TWO),
    ///         pos_or_panic!(0.18),   // implied_volatility
    ///         dec!(0.05),   // risk_free_rate
    ///         Positive::ZERO,   // dividend_yield
    ///         Positive::ONE,   // long quantity
    ///         pos_or_panic!(15.04),   // premium_long
    ///         pos_or_panic!(89.85),   // premium_short
    ///         pos_or_panic!(0.78),   // open_fee_long
    ///         pos_or_panic!(0.78),   // open_fee_long
    ///         pos_or_panic!(0.73),   // close_fee_long
    ///         pos_or_panic!(0.73),   // close_fee_short
    ///     )?;
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
    /// # Ok(())
    /// # }
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
    /// - [`OptionChain::get_double_iter`](optionstratlib_market::chains::OptionChain::get_double_iter)
    /// - [`OptionData::is_valid_optimal_side`](optionstratlib_market::chains::OptionData::is_valid_optimal_side)
    /// - [`BullPutSpread::validate`](crate::strategies::bull_put_spread::BullPutSpread::validate)
    fn filter_combinations<'a>(
        &'a self,
        option_chain: &'a OptionChain,
        side: FindOptimalSide,
    ) -> impl Iterator<Item = OptionDataGroup<'a>> {
        let underlying_price = &self.long_put.option.underlying_price;
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
                match strategy.create_strategy(option_chain, &legs) {
                    Ok(s) => s.validate() && s.get_max_profit().is_ok() && s.get_max_loss().is_ok(),
                    Err(_) => false,
                }
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
                other => {
                    tracing::warn!(
                        group = ?other,
                        "find_optimal: skipping unexpected OptionDataGroup variant"
                    );
                    continue;
                }
            };

            let legs = StrategyLegs::TwoLegs {
                first: long_option,
                second: short_option,
            };
            let strategy = match self.create_strategy(option_chain, &legs) {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!(error = %e, "skipping invalid strategy combination");
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
                    tracing::warn!(error = %e, "skipping candidate with unscorable metric");
                    continue;
                }
            };

            if current_value > best_value {
                // Update the best value and replace the current strategy
                debug!("Found better value: {}", current_value);
                best_value = current_value;
                *self = strategy.clone();
            }
        }
    }

    /// Constructs a `BullPutSpread` from the supplied chain and legs.
    ///
    /// # Errors
    ///
    /// Returns `StrategyError::OperationError` when the supplied legs are
    /// missing required quotes (`long.put_ask`, `short.put_bid`) needed to
    /// price the spread.
    fn create_strategy(
        &self,
        chain: &OptionChain,
        legs: &StrategyLegs,
    ) -> Result<Self::Strategy, StrategyError> {
        let (long, short) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => {
                return Err(StrategyError::operation_not_supported(
                    "create_strategy",
                    "BullPutSpread requires exactly two legs (TwoLegs)",
                ));
            }
        };
        let implied_volatility = long.implied_volatility;
        if implied_volatility > Positive::ONE {
            return Err(StrategyError::invalid_parameters(
                "create_strategy",
                &format!(
                    "implied volatility {implied_volatility} exceeds the supported maximum of 1.0"
                ),
            ));
        }
        let long_put_ask = long.put_ask.ok_or_else(|| {
            StrategyError::operation_not_supported(
                "create_strategy",
                "missing put_ask for long leg",
            )
        })?;
        let short_put_bid = short.put_bid.ok_or_else(|| {
            StrategyError::operation_not_supported(
                "create_strategy",
                "missing put_bid for short leg",
            )
        })?;
        let mut strategy = BullPutSpread::new(
            chain.symbol.clone(),
            chain.underlying_price,
            long.strike_price,
            short.strike_price,
            self.long_put.option.expiration_date,
            implied_volatility,
            self.long_put.option.risk_free_rate,
            self.long_put.option.dividend_yield,
            self.long_put.option.quantity,
            long_put_ask,
            short_put_bid,
            self.long_put.open_fee,
            self.long_put.close_fee,
            self.short_put.open_fee,
            self.short_put.close_fee,
        )?;
        // The rebuilt legs keep the contract size of the strategy they
        // are rebuilt from.
        strategy.set_contract_size(self.get_contract_size()?)?;
        Ok(strategy)
    }
}

impl Profit for BullPutSpread {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, PricingError> {
        let price = Some(price);
        Ok(d_sum(
            &[
                self.long_put.pnl_at_expiration(&price)?,
                self.short_put.pnl_at_expiration(&price)?,
            ],
            "strategies::bull_put_spread::profit_at",
        )?)
    }
}

impl ProbabilityAnalysis for BullPutSpread {
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_point = *self.get_break_even_points()?.first().ok_or_else(|| {
            ProbabilityError::RangeError(ProfitLossRangeErrorKind::InvalidBreakEvenPoints {
                reason: "BullPutSpread has no break-even point".to_string(),
            })
        })?;
        let option = &self.short_put.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            self.short_put.option.implied_volatility,
            self.long_put.option.implied_volatility,
        ])?;

        let mut profit_range = ProfitLossRange::new(Some(break_even_point), None, Positive::ZERO)?;

        profit_range.calculate_probability(
            &self.long_put.option.underlying_price,
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
        let break_even_point = *self.get_break_even_points()?.first().ok_or_else(|| {
            ProbabilityError::RangeError(ProfitLossRangeErrorKind::InvalidBreakEvenPoints {
                reason: "BullPutSpread has no break-even point".to_string(),
            })
        })?;
        let option = &self.long_put.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            self.short_put.option.implied_volatility,
            self.long_put.option.implied_volatility,
        ])?;

        let mut loss_range = ProfitLossRange::new(
            Some(self.long_put.option.strike_price),
            Some(break_even_point),
            Positive::ZERO,
        )?;

        loss_range.calculate_probability(
            &self.long_put.option.underlying_price,
            VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            },
            None,
            expiration_date,
            Some(risk_free_rate),
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

impl SpreadStrategy for BullPutSpread {
    fn lower_strike(&self) -> Positive {
        self.long_put.option.strike_price
    }

    fn upper_strike(&self) -> Positive {
        self.short_put.option.strike_price
    }

    fn short_leg(&self) -> &Position {
        &self.short_put
    }

    fn long_leg(&self) -> &Position {
        &self.long_put
    }
}

impl PnLCalculator for BullPutSpread {
    fn calculate_pnl(
        &self,
        market_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, PricingError> {
        // `impl Add for PnL` returns `Self`, so a leg total that leaves the
        // `Positive` range has nowhere to be reported and aborts instead.
        // `PnL::try_add` adds the same fields and reports it.
        let mut total =
            self.short_put
                .calculate_pnl(market_price, expiration_date, implied_volatility)?;
        total = total.try_add(&self.long_put.calculate_pnl(
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
        // `impl Add for PnL` returns `Self`, so a leg total that leaves the
        // `Positive` range has nowhere to be reported and aborts instead.
        // `PnL::try_add` adds the same fields and reports it.
        let mut total = self
            .short_put
            .calculate_pnl_at_expiration(underlying_price)?;
        total = total.try_add(
            &self
                .long_put
                .calculate_pnl_at_expiration(underlying_price)?,
        )?;
        Ok(total)
    }
}

#[cfg(test)]
crate::strategies::macros::test_strategy_traits!(BullPutSpread, test_short_call_implementations);

#[cfg(test)]
fn bull_put_spread_test() -> BullPutSpread {
    use rust_decimal_macros::dec;
    let underlying_price = pos_or_panic!(5781.88);
    BullPutSpread::new(
        "SP500".to_string(),
        underlying_price,      // underlying_price
        pos_or_panic!(5750.0), // long_strike_itm
        pos_or_panic!(5920.0), // short_strike
        ExpirationDate::Days(Positive::TWO),
        pos_or_panic!(0.18),  // implied_volatility
        dec!(0.05),           // risk_free_rate
        Positive::ZERO,       // dividend_yield
        pos_or_panic!(3.0),   // long quantity
        pos_or_panic!(15.04), // premium_long
        pos_or_panic!(89.85), // premium_short
        pos_or_panic!(0.78),  // open_fee_long
        pos_or_panic!(0.78),  // open_fee_long
        pos_or_panic!(0.73),  // close_fee_long
        pos_or_panic!(0.73),  // close_fee_short
    )
    .unwrap()
}

#[cfg(test)]
mod tests_bull_put_spread_strategy {
    use super::*;

    use optionstratlib_core::model::ExpirationDate;

    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    /// An inverted vertical (the short strike below the long one, reachable
    /// through the `pub` legs) is a structural failure: `get_max_loss`
    /// reports it as `InvalidStrategy`, and the profit ratio passes it on
    /// instead of reading it as zero loss (#803).
    #[test]
    fn test_inverted_strikes_propagate_as_invalid_strategy() {
        let mut spread = bull_put_spread_test();
        spread.short_put.option.strike_price = pos_or_panic!(5700.0);

        let is_invalid = |error: StrategyError| {
            matches!(
                error,
                StrategyError::InvalidStrategy {
                    strategy: StrategyType::BullPutSpread,
                    ..
                }
            )
        };
        assert!(spread.get_max_loss().is_err_and(is_invalid));
        assert!(spread.get_profit_ratio().is_err_and(is_invalid));
    }

    #[test]
    fn test_new_bull_put_spread() {
        let spread = bull_put_spread_test();

        assert_eq!(spread.name, "Bull Put Spread");
        assert_eq!(spread.kind, StrategyType::BullPutSpread);
        assert!(!spread.description.is_empty());
        assert_eq!(
            spread.get_underlying_price().unwrap(),
            &pos_or_panic!(5781.88)
        );
        assert_eq!(spread.long_put.option.strike_price, pos_or_panic!(5750.0));
        assert_eq!(spread.short_put.option.strike_price, pos_or_panic!(5920.0));
    }

    #[test]
    fn test_add_leg() {
        let mut spread = bull_put_spread_test();
        let new_long_put = Position::new(
            Options::new(
                OptionType::European,
                Side::Long,
                "TEST".to_string(),
                pos_or_panic!(85.0),
                ExpirationDate::Days(pos_or_panic!(30.0)),
                pos_or_panic!(0.2),
                Positive::ONE,
                Positive::HUNDRED,
                dec!(0.05),
                OptionStyle::Put,
                Positive::ZERO,
                None,
            ),
            pos_or_panic!(1.5),
            Utc::now(),
            Positive::ZERO,
            Positive::ZERO,
            None,
            None,
        );

        spread
            .add_position(&new_long_put)
            .expect("Error adding long put");
        assert_eq!(spread.long_put.option.strike_price, pos_or_panic!(85.0));
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
        let max_profit = spread.get_max_profit().unwrap();
        assert_eq!(max_profit, pos_or_panic!(215.37));
    }

    #[test]
    fn test_max_loss() {
        let spread = bull_put_spread_test();
        let max_loss = spread.get_max_loss().unwrap();
        assert_eq!(max_loss, pos_or_panic!(294.63));
    }

    #[test]
    fn test_total_cost() {
        let spread = bull_put_spread_test();
        assert_eq!(spread.get_total_cost().unwrap(), pos_or_panic!(54.18));
    }

    #[test]
    fn test_net_premium_received() {
        let spread = bull_put_spread_test();
        assert_eq!(spread.get_net_premium_received().unwrap().to_f64(), 215.37);
    }

    #[test]
    fn test_fees() {
        let spread = bull_put_spread_test();

        assert_eq!(spread.get_fees().unwrap().to_f64(), 9.06);
    }

    #[test]
    fn test_break_even_points() {
        let spread = bull_put_spread_test();
        let break_even_points = spread.get_break_even_points().unwrap();

        assert_eq!(break_even_points.len(), 1);
        assert_eq!(break_even_points[0], pos_or_panic!(5848.21));
    }

    #[test]
    fn test_profit_area() {
        let spread = bull_put_spread_test();
        let area = spread.get_profit_area().unwrap().to_f64().unwrap();
        assert!(area > 0.0);
    }

    #[test]
    fn test_profit_ratio() {
        let spread = bull_put_spread_test();
        let ratio = spread.get_profit_ratio().unwrap().to_f64().unwrap();

        // Ratio = (max_profit / max_loss) * 100
        // = (1.0 / 4.0) * 100 = 25
        assert_relative_eq!(ratio, 73.0984, epsilon = 0.0001);
    }

    fn new_spread(
        long_strike: Positive,
        short_strike: Positive,
    ) -> Result<BullPutSpread, StrategyError> {
        BullPutSpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            long_strike,
            short_strike,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::ONE,
            Positive::ONE,
            Positive::TWO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        )
    }

    #[test]
    fn test_default_strikes() {
        // A zero long strike defaults to the underlying price.
        let spread = new_spread(Positive::ZERO, pos_or_panic!(105.0)).unwrap();
        assert_eq!(spread.long_put.option.strike_price, Positive::HUNDRED);
        assert_eq!(spread.short_put.option.strike_price, pos_or_panic!(105.0));

        // A zero short strike defaults to the underlying price.
        let spread = new_spread(pos_or_panic!(95.0), Positive::ZERO).unwrap();
        assert_eq!(spread.long_put.option.strike_price, pos_or_panic!(95.0));
        assert_eq!(spread.short_put.option.strike_price, Positive::HUNDRED);
    }

    #[test]
    fn test_bull_put_spread_both_default_strikes_rejected() {
        // Both strikes default to the underlying price, which is no vertical (#696).
        assert!(matches!(
            new_spread(Positive::ZERO, Positive::ZERO),
            Err(StrategyError::InvalidStrategy {
                strategy: StrategyType::BullPutSpread,
                ..
            })
        ));
    }

    #[test]
    fn test_invalid_strikes() {
        // The long put above the short put is a bear put spread (#696).
        assert!(matches!(
            new_spread(pos_or_panic!(95.0), pos_or_panic!(90.0)),
            Err(StrategyError::InvalidStrategy {
                strategy: StrategyType::BullPutSpread,
                ..
            })
        ));
    }
}

#[cfg(test)]
mod tests_bull_put_spread_validation {
    use super::*;

    use optionstratlib_core::model::ExpirationDate;

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
                pos_or_panic!(0.2),
                Positive::ONE,
                Positive::HUNDRED,
                dec!(0.05),
                OptionStyle::Put,
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
    fn test_invalid_long_put() {
        let mut invalid_long = create_valid_position(
            Side::Long,
            pos_or_panic!(90.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
        );
        invalid_long.option.quantity = Positive::ZERO;

        let spread = BullPutSpread {
            name: "Test Bull Put Spread".to_string(),
            kind: StrategyType::BullPutSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_put: invalid_long,
            short_put: create_valid_position(
                Side::Short,
                pos_or_panic!(95.0),
                ExpirationDate::Days(pos_or_panic!(30.0)),
            ),
        };

        assert!(
            !spread.validate(),
            "Spread with invalid long put should fail validation"
        );
    }

    #[test]
    fn test_invalid_short_put() {
        let mut invalid_short = create_valid_position(
            Side::Short,
            pos_or_panic!(95.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
        );
        invalid_short.option.quantity = Positive::ZERO;

        let spread = BullPutSpread {
            name: "Test Bull Put Spread".to_string(),
            kind: StrategyType::BullPutSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_put: create_valid_position(
                Side::Long,
                pos_or_panic!(90.0),
                ExpirationDate::Days(pos_or_panic!(30.0)),
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
                pos_or_panic!(95.0),
                ExpirationDate::Days(pos_or_panic!(30.0)),
            ),
            short_put: create_valid_position(
                Side::Short,
                pos_or_panic!(90.0),
                ExpirationDate::Days(pos_or_panic!(30.0)),
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
                pos_or_panic!(90.0),
                ExpirationDate::Days(pos_or_panic!(30.0)),
            ),
            short_put: create_valid_position(
                Side::Short,
                pos_or_panic!(90.0),
                ExpirationDate::Days(pos_or_panic!(30.0)),
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
                pos_or_panic!(90.0),
                ExpirationDate::Days(pos_or_panic!(30.0)),
            ),
            short_put: create_valid_position(
                Side::Short,
                pos_or_panic!(95.0),
                ExpirationDate::Days(pos_or_panic!(60.0)),
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
                pos_or_panic!(89.99),
                ExpirationDate::Days(pos_or_panic!(30.0)),
            ),
            short_put: create_valid_position(
                Side::Short,
                pos_or_panic!(90.0),
                ExpirationDate::Days(pos_or_panic!(30.0)),
            ),
        };
        assert!(spread.validate());
    }
}

#[cfg(test)]
mod tests_bull_put_spread_optimization {
    use super::*;

    use optionstratlib_core::model::ExpirationDate;
    use optionstratlib_market::chains::OptionData;

    use num_traits::ToPrimitive;
    use optionstratlib_core::spos;
    use rust_decimal_macros::dec;

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new(
            "TEST",
            Positive::HUNDRED,
            "2024-12-31".to_string(),
            None,
            None,
        );

        chain.add_option(
            pos_or_panic!(85.0), // strike
            spos!(2.4),          // call_bid
            spos!(2.6),          // call_ask
            spos!(2.0),          // put_bid
            spos!(2.2),          // put_ask
            pos_or_panic!(0.2),  // implied_volatility
            Some(dec!(-0.3)),    // delta
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0), // volume
            Some(50),     // open_interest
            None,
        );

        chain.add_option(
            pos_or_panic!(90.0),
            spos!(3.4),
            spos!(3.6),
            spos!(3.0),
            spos!(3.2),
            pos_or_panic!(0.2),
            Some(dec!(-0.4)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(150.0),
            Some(75),
            None,
        );

        chain.add_option(
            pos_or_panic!(95.0),
            spos!(4.4),
            spos!(4.6),
            spos!(4.0),
            spos!(4.2),
            pos_or_panic!(0.2),
            Some(dec!(-0.5)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(200.0),
            Some(100),
            None,
        );

        chain.add_option(
            Positive::HUNDRED,
            spos!(5.4),
            spos!(5.6),
            spos!(5.0),
            spos!(5.2),
            pos_or_panic!(0.2),
            Some(dec!(-0.6)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(250.0),
            Some(125),
            None,
        );

        chain.add_option(
            pos_or_panic!(105.0),
            spos!(6.4),
            spos!(6.6),
            spos!(6.0),
            spos!(6.2),
            pos_or_panic!(0.2),
            Some(dec!(-0.7)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(300.0),
            Some(150),
            None,
        );

        chain
    }

    fn create_base_spread() -> BullPutSpread {
        BullPutSpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(90.0),
            pos_or_panic!(95.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::ONE,
            pos_or_panic!(3.2),
            pos_or_panic!(4.0),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        )
        .unwrap()
    }

    #[test]
    fn test_find_optimal_ratio() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);
        assert!(spread.validate(), "Optimized spread should be valid");
        assert!(
            spread.get_profit_ratio().unwrap().to_f64().unwrap() > 0.0,
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
            spread.get_profit_area().unwrap().to_f64().unwrap() > 0.0,
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
            FindOptimalSide::Range(pos_or_panic!(90.0), Positive::HUNDRED),
            OptimizationCriteria::Ratio,
        );

        assert!(spread.short_put.option.strike_price <= Positive::HUNDRED);
        assert!(spread.short_put.option.strike_price >= pos_or_panic!(90.0));
        assert!(spread.long_put.option.strike_price <= Positive::HUNDRED);
        assert!(spread.long_put.option.strike_price >= pos_or_panic!(90.0));
    }

    #[test]
    fn test_is_valid_long_option() {
        let spread = create_base_spread();
        let option = OptionData::new(
            pos_or_panic!(95.0),
            None,
            None,
            spos!(3.0),
            spos!(3.2),
            pos_or_panic!(0.2),
            Some(dec!(-0.4)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0),
            Some(50),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        assert!(spread.is_valid_optimal_option(&option, &FindOptimalSide::All));
        assert!(spread.is_valid_optimal_option(&option, &FindOptimalSide::Lower));
        assert!(!spread.is_valid_optimal_option(&option, &FindOptimalSide::Upper));
        assert!(spread.is_valid_optimal_option(
            &option,
            &FindOptimalSide::Range(pos_or_panic!(90.0), Positive::HUNDRED)
        ));
    }

    #[test]
    fn test_is_valid_short_option() {
        let spread = create_base_spread();
        let option = OptionData::new(
            pos_or_panic!(105.0),
            None,
            None,
            spos!(4.0),
            spos!(4.2),
            pos_or_panic!(0.2),
            Some(dec!(-0.5)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0),
            Some(50),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        assert!(spread.is_valid_optimal_option(&option, &FindOptimalSide::All));
        assert!(!spread.is_valid_optimal_option(&option, &FindOptimalSide::Lower));
        assert!(spread.is_valid_optimal_option(&option, &FindOptimalSide::Upper));
        assert!(!spread.is_valid_optimal_option(
            &option,
            &FindOptimalSide::Range(pos_or_panic!(90.0), Positive::HUNDRED)
        ));
    }

    #[test]
    fn test_are_valid_prices() {
        let long_option = OptionData::new(
            pos_or_panic!(90.0),
            None,
            None,
            spos!(3.0),
            spos!(3.2),
            pos_or_panic!(0.2),
            Some(dec!(-0.4)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0),
            Some(50),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let short_option = OptionData::new(
            pos_or_panic!(95.0),
            None,
            None,
            spos!(4.0),
            spos!(4.2),
            pos_or_panic!(0.2),
            Some(dec!(-0.5)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0),
            Some(50),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
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
            .find(|o| o.strike_price == pos_or_panic!(90.0))
            .unwrap();
        let short_option = chain
            .options
            .iter()
            .find(|o| o.strike_price == pos_or_panic!(95.0))
            .unwrap();

        let legs = StrategyLegs::TwoLegs {
            first: long_option,
            second: short_option,
        };
        let new_strategy = spread.create_strategy(&chain, &legs).unwrap();

        assert!(new_strategy.validate());
        assert_eq!(
            new_strategy.long_put.option.strike_price,
            pos_or_panic!(90.0)
        );
        assert_eq!(
            new_strategy.short_put.option.strike_price,
            pos_or_panic!(95.0)
        );
    }
}

#[cfg(test)]
mod tests_bull_put_spread_profit {
    use super::*;

    use optionstratlib_core::model::ExpirationDate;

    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    #[test]
    fn test_profit_above_short_strike() {
        let spread = bull_put_spread_test();
        let price = pos_or_panic!(5800.0);
        assert_eq!(
            spread
                .calculate_profit_at(&price)
                .unwrap()
                .to_f64()
                .unwrap(),
            -144.63
        );
    }

    #[test]
    fn test_profit_at_short_strike() {
        let spread = bull_put_spread_test();
        let price = pos_or_panic!(5900.0);
        assert_eq!(
            spread
                .calculate_profit_at(&price)
                .unwrap()
                .to_f64()
                .unwrap(),
            155.37
        );
    }

    #[test]
    fn test_profit_between_strikes() {
        let spread = bull_put_spread_test();
        let price = pos_or_panic!(5155.37);

        assert_eq!(
            spread
                .calculate_profit_at(&price)
                .unwrap()
                .to_f64()
                .unwrap(),
            -294.63
        );
    }

    #[test]
    fn test_profit_at_long_strike() {
        let spread = bull_put_spread_test();
        let price = pos_or_panic!(5655.0);
        assert_eq!(
            spread
                .calculate_profit_at(&price)
                .unwrap()
                .to_f64()
                .unwrap(),
            -294.63
        );
    }

    #[test]
    fn test_profit_below_long_strike() {
        let spread = bull_put_spread_test();
        let price = pos_or_panic!(5755.0);
        assert_eq!(
            spread
                .calculate_profit_at(&price)
                .unwrap()
                .to_f64()
                .unwrap(),
            -279.63
        );
    }

    #[test]
    fn test_profit_at_get_break_even_points() {
        let spread = bull_put_spread_test();
        let break_even_points = spread.get_break_even_points().unwrap();
        let price = break_even_points[0];
        assert!(spread.calculate_profit_at(&price).unwrap().abs() < dec!(0.001));
    }

    #[test]
    fn test_profit_with_multiple_contracts() {
        let spread = BullPutSpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(90.0),
            pos_or_panic!(95.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::TWO,
            Positive::TWO,
            pos_or_panic!(4.0),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        )
        .unwrap();

        let price = pos_or_panic!(85.0);
        assert_eq!(
            spread
                .calculate_profit_at(&price)
                .unwrap()
                .to_f64()
                .unwrap(),
            -6.0
        );
    }

    #[test]
    fn test_profit_with_fees() {
        let spread = bull_put_spread_test();
        let break_even_points = spread.get_break_even_points().unwrap();
        let price = break_even_points[0];
        assert_eq!(
            spread
                .calculate_profit_at(&price)
                .unwrap()
                .to_f64()
                .unwrap(),
            0.0
        );
    }
}

#[cfg(test)]
mod tests_bull_put_spread_graph {
    use super::*;

    #[test]
    fn test_title_format() {
        let spread = bull_put_spread_test();
        let title = spread.get_title();
        assert!(title.contains("BullPutSpread Strategy"));
        assert!(title.contains("SP500 @ $5750 Long Put European Option"));
        assert!(title.contains("SP500 @ $5920 Short Put European Option"));
    }
}

#[cfg(test)]
mod tests_bull_put_spread_probability {
    use super::*;

    use optionstratlib_analytics::analytics::probability::PriceTrend;
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

    fn bull_put_spread_test() -> BullPutSpread {
        BullPutSpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,                         // underlying_price
            pos_or_panic!(90.0),                       // long_strike
            pos_or_panic!(95.0),                       // short_strike
            ExpirationDate::Days(pos_or_panic!(30.0)), // expiration
            pos_or_panic!(0.2),                        // implied_volatility
            dec!(0.05),                                // risk_free_rate
            Positive::ZERO,                            // dividend_yield
            Positive::ONE,                             // quantity
            Positive::ONE,                             // premium_long_put
            Positive::TWO,                             // premium_short_put
            Positive::ZERO,                            // open_fee_long_put
            Positive::ZERO,                            // close_fee_long_put
            Positive::ZERO,                            // open_fee_short_put
            Positive::ZERO,                            // close_fee_short_put
        )
        .unwrap()
    }

    #[test]
    fn test_get_expiration() {
        let spread = bull_put_spread_test();
        let expiration_date = *spread.get_expiration().values().next().unwrap();
        assert_eq!(expiration_date, &ExpirationDate::Days(pos_or_panic!(30.0)));
    }

    #[test]
    fn test_get_risk_free_rate() {
        let spread = bull_put_spread_test();
        assert_eq!(
            *spread
                .get_risk_free_rate()
                .unwrap()
                .values()
                .next()
                .unwrap(),
            &dec!(0.05)
        );
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
        assert!(prob <= Positive::ONE);
    }

    #[test]
    fn test_probability_with_volatility_adjustment() {
        let spread = bull_put_spread_test();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos_or_panic!(0.25),
            std_dev_adjustment: pos_or_panic!(0.05),
        });

        let result = spread.probability_of_profit(vol_adj, None);
        assert!(result.is_ok());

        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= Positive::ONE);
    }

    #[test]
    fn test_probability_with_trend() {
        let spread = bull_put_spread_test();
        let trend = Some(price_trend(dec!(0.1), dec!(0.95)));

        let result = spread.probability_of_profit(None, trend);
        assert!(result.is_ok());

        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= Positive::ONE);
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
        assert!(analysis.expected_value > Decimal::ZERO);
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
        assert!(max_profit_prob + max_loss_prob <= Positive::ONE);
    }
}

#[cfg(test)]
mod tests_delta {
    use super::*;
    use optionstratlib_core::assert_pos_relative_eq;

    use crate::strategies::bull_put_spread::BullPutSpread;
    use crate::strategies::delta_neutral::DeltaNeutrality;
    use optionstratlib_analytics::pnl::DeltaAdjustment;
    use optionstratlib_core::assert_decimal_eq;
    use optionstratlib_core::model::types::OptionStyle;
    use optionstratlib_pricing::greeks::DELTA_THRESHOLD;
    use rust_decimal_macros::dec;

    // The delta tests drive the adjustment engine through both signs of net
    // delta and through zero, which takes inverted and equal strikes. `new`
    // rejects such legs since #696, so the strategy is built on valid
    // placeholder strikes and the requested strikes are set on the legs.
    fn get_strategy(long_strike: Positive, short_strike: Positive) -> BullPutSpread {
        let underlying_price = pos_or_panic!(5801.88);
        let mut strategy = BullPutSpread::new(
            "SP500".to_string(),
            underlying_price, // underlying_price
            Positive::ONE,    // long_strike placeholder
            Positive::TWO,    // short_strike placeholder
            ExpirationDate::Days(Positive::TWO),
            pos_or_panic!(0.18),  // implied_volatility
            dec!(0.05),           // risk_free_rate
            Positive::ZERO,       // dividend_yield
            Positive::ONE,        // long quantity
            pos_or_panic!(15.04), // premium_long
            pos_or_panic!(89.85), // premium_short
            pos_or_panic!(0.78),  // open_fee_long
            pos_or_panic!(0.78),  // open_fee_long
            pos_or_panic!(0.73),  // close_fee_long
            pos_or_panic!(0.73),  // close_fee_short
        )
        .unwrap();
        strategy.long_put.option.strike_price = long_strike;
        strategy.short_put.option.strike_price = short_strike;
        strategy.update_break_even_points().unwrap();
        strategy
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos_or_panic!(5750.0), pos_or_panic!(5920.0));
        let size = dec!(0.6897372);
        let delta = pos_or_panic!(2.855544139071374);
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
        let strategy = get_strategy(pos_or_panic!(5840.0), pos_or_panic!(5750.0));
        let size = dec!(-0.437230414);
        let delta = pos_or_panic!(1.8101540723661196);
        let k = pos_or_panic!(5750.0);
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
        let strategy = get_strategy(pos_or_panic!(5830.0), pos_or_panic!(5830.0));
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
    use optionstratlib_core::assert_pos_relative_eq;

    use crate::strategies::bull_put_spread::BullPutSpread;
    use crate::strategies::delta_neutral::DeltaNeutrality;
    use optionstratlib_analytics::pnl::DeltaAdjustment;
    use optionstratlib_core::assert_decimal_eq;
    use optionstratlib_core::model::types::OptionStyle;
    use optionstratlib_pricing::greeks::DELTA_THRESHOLD;
    use rust_decimal_macros::dec;

    // The delta tests drive the adjustment engine through both signs of net
    // delta and through zero, which takes inverted and equal strikes. `new`
    // rejects such legs since #696, so the strategy is built on valid
    // placeholder strikes and the requested strikes are set on the legs.
    fn get_strategy(long_strike: Positive, short_strike: Positive) -> BullPutSpread {
        let underlying_price = pos_or_panic!(5781.88);
        let mut strategy = BullPutSpread::new(
            "SP500".to_string(),
            underlying_price, // underlying_price
            Positive::ONE,    // long_strike placeholder
            Positive::TWO,    // short_strike placeholder
            ExpirationDate::Days(Positive::TWO),
            pos_or_panic!(0.18),  // implied_volatility
            dec!(0.05),           // risk_free_rate
            Positive::ZERO,       // dividend_yield
            Positive::TWO,        // long quantity
            pos_or_panic!(15.04), // premium_long
            pos_or_panic!(89.85), // premium_short
            pos_or_panic!(0.78),  // open_fee_long
            pos_or_panic!(0.78),  // open_fee_long
            pos_or_panic!(0.73),  // close_fee_long
            pos_or_panic!(0.73),  // close_fee_short
        )
        .unwrap();
        strategy.long_put.option.strike_price = long_strike;
        strategy.short_put.option.strike_price = short_strike;
        strategy.update_break_even_points().unwrap();
        strategy
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos_or_panic!(5750.0), pos_or_panic!(5820.9));
        let size = dec!(0.7086);
        let delta = pos_or_panic!(2.152913807138664);
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
        let strategy = get_strategy(pos_or_panic!(5840.0), pos_or_panic!(5750.0));
        let size = dec!(-0.8722316);
        let delta = pos_or_panic!(2.649732171104434);
        let k = pos_or_panic!(5750.0);
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
        let strategy = get_strategy(pos_or_panic!(5840.0), pos_or_panic!(5840.0));

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

    use optionstratlib_core::error::position::PositionValidationErrorKind;
    use optionstratlib_core::model::types::{OptionStyle, Side};

    use rust_decimal_macros::dec;
    use tracing::error;

    fn create_test_short_bull_put_spread() -> BullPutSpread {
        BullPutSpread::new(
            "SP500".to_string(),
            pos_or_panic!(5781.88), // underlying_price
            // Textbook legs: long the lower strike, short the higher (#696).
            pos_or_panic!(5720.0), // long_strike
            pos_or_panic!(5850.0), // short_strike
            ExpirationDate::Days(Positive::TWO),
            pos_or_panic!(0.18),  // implied_volatility
            dec!(0.05),           // risk_free_rate
            Positive::ZERO,       // dividend_yield
            pos_or_panic!(4.0),   // long quantity
            pos_or_panic!(85.04), // premium_long
            pos_or_panic!(29.85), // premium_short
            pos_or_panic!(0.78),  // open_fee_long
            pos_or_panic!(0.78),  // open_fee_long
            pos_or_panic!(0.73),  // close_fee_long
            pos_or_panic!(0.73),  // close_fee_short
        )
        .unwrap()
    }

    #[test]
    fn test_short_bull_put_spread_get_position() {
        let mut bull_put_spread = create_test_short_bull_put_spread();

        // Test getting short put position
        let put_position =
            bull_put_spread.get_position(&OptionStyle::Put, &Side::Long, &pos_or_panic!(5720.0));
        assert!(put_position.is_ok());
        let positions = put_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos_or_panic!(5720.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Put);
        assert_eq!(positions[0].option.side, Side::Long);

        // Test getting short put position
        let put_position =
            bull_put_spread.get_position(&OptionStyle::Put, &Side::Short, &pos_or_panic!(5850.0));
        assert!(put_position.is_ok());
        let positions = put_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos_or_panic!(5850.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Put);
        assert_eq!(positions[0].option.side, Side::Short);

        // Test getting non-existent position
        let invalid_position =
            bull_put_spread.get_position(&OptionStyle::Call, &Side::Short, &pos_or_panic!(5821.0));
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
        modified_put.option.quantity = Positive::TWO;
        let result = bull_put_spread.modify_position(&modified_put);
        assert!(result.is_ok());
        assert_eq!(bull_put_spread.short_put.option.quantity, Positive::TWO);

        // Modify short put position
        let mut modified_put = bull_put_spread.long_put.clone();
        modified_put.option.quantity = Positive::TWO;
        let result = bull_put_spread.modify_position(&modified_put);
        assert!(result.is_ok());
        assert_eq!(bull_put_spread.long_put.option.quantity, Positive::TWO);

        // Test modifying with invalid position
        let mut invalid_position = bull_put_spread.short_put.clone();
        invalid_position.option.strike_price = pos_or_panic!(95.0);
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

    use optionstratlib_core::model::types::{OptionStyle, Side};

    use rust_decimal_macros::dec;

    // Helper function to create a test strategy
    fn create_test_strategy() -> BullPutSpread {
        BullPutSpread::new(
            "SP500".to_string(),
            pos_or_panic!(5781.88), // underlying_price
            // Textbook legs: long the lower strike, short the higher (#696).
            pos_or_panic!(5720.0), // long_strike
            pos_or_panic!(5850.0), // short_strike
            ExpirationDate::Days(Positive::TWO),
            pos_or_panic!(0.18),  // implied_volatility
            dec!(0.05),           // risk_free_rate
            Positive::ZERO,       // dividend_yield
            pos_or_panic!(4.0),   // long quantity
            pos_or_panic!(85.04), // premium_long
            pos_or_panic!(29.85), // premium_short
            pos_or_panic!(0.78),  // open_fee_long
            pos_or_panic!(0.78),  // open_fee_long
            pos_or_panic!(0.73),  // close_fee_long
            pos_or_panic!(0.73),  // close_fee_short
        )
        .unwrap()
    }

    #[test]
    fn test_adjust_existing_call_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.short_put.option.quantity;
        let adjustment = Positive::ONE;

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos_or_panic!(5850.0),
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
        let adjustment = Positive::ONE;

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos_or_panic!(5720.0),
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
            &pos_or_panic!(5720.0),
            &OptionStyle::Call,
            &Side::Long,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        // StrategyError wraps PositionError, so we check the error message
        assert!(
            err.to_string()
                .contains("Call is not valid for BullPutSpread")
        );
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
        let initial_quantity = strategy.long_put.option.quantity;

        let result = strategy.adjust_option_position(
            Decimal::ZERO,
            &pos_or_panic!(5850.0),
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

    use optionstratlib_core::error::OperationErrorKind;
    use optionstratlib_core::model::utils::create_sample_position;

    #[test]
    fn test_get_strategy_valid() {
        // Textbook legs: long the lower-strike put, short the higher (#696).
        let options = vec![
            create_sample_position(
                OptionStyle::Put,
                Side::Long,
                Positive::HUNDRED,
                Positive::ONE,
                pos_or_panic!(95.0),
                pos_or_panic!(0.2),
            ),
            create_sample_position(
                OptionStyle::Put,
                Side::Short,
                Positive::HUNDRED,
                Positive::ONE,
                pos_or_panic!(105.0),
                pos_or_panic!(0.2),
            ),
        ];

        let result = BullPutSpread::get_strategy(&options);
        assert!(result.is_ok());

        let strategy = result.unwrap();
        assert_eq!(strategy.long_put.option.strike_price, pos_or_panic!(95.0));
        assert_eq!(strategy.short_put.option.strike_price, pos_or_panic!(105.0));
        assert!(strategy.validate());
    }

    #[test]
    fn test_bull_put_spread_get_strategy_textbook_legs_accepted_in_any_order() {
        // The builder sorts by strike, so the input order does not matter.
        let short_put = create_sample_position(
            OptionStyle::Put,
            Side::Short,
            Positive::HUNDRED,
            Positive::ONE,
            pos_or_panic!(105.0),
            pos_or_panic!(0.2),
        );
        let long_put = create_sample_position(
            OptionStyle::Put,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            pos_or_panic!(95.0),
            pos_or_panic!(0.2),
        );

        let strategy = BullPutSpread::get_strategy(&[short_put, long_put]).unwrap();
        assert_eq!(strategy.long_put.option.side, Side::Long);
        assert_eq!(strategy.long_put.option.strike_price, pos_or_panic!(95.0));
        assert_eq!(strategy.short_put.option.side, Side::Short);
        assert_eq!(strategy.short_put.option.strike_price, pos_or_panic!(105.0));
    }

    #[test]
    fn test_bull_put_spread_get_strategy_inverted_legs_rejected() {
        // Short the lower strike and long the higher is a bear put spread (#696).
        let options = vec![
            create_sample_position(
                OptionStyle::Put,
                Side::Short,
                Positive::HUNDRED,
                Positive::ONE,
                pos_or_panic!(95.0),
                pos_or_panic!(0.2),
            ),
            create_sample_position(
                OptionStyle::Put,
                Side::Long,
                Positive::HUNDRED,
                Positive::ONE,
                pos_or_panic!(105.0),
                pos_or_panic!(0.2),
            ),
        ];

        let result = BullPutSpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Bull Put Spread get_strategy"
                && reason == "Bull Put Spread requires a long lower strike put and a short higher strike put"
        ));
    }

    #[test]
    fn test_bull_put_spread_get_strategy_invalid_leg_rejected() {
        // Sides and strikes are right, but a short leg with no premium fails
        // `validate`, which `get_strategy` no longer ignores (#696).
        let long_put = create_sample_position(
            OptionStyle::Put,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            pos_or_panic!(95.0),
            pos_or_panic!(0.2),
        );
        let mut short_put = create_sample_position(
            OptionStyle::Put,
            Side::Short,
            Positive::HUNDRED,
            Positive::ONE,
            pos_or_panic!(105.0),
            pos_or_panic!(0.2),
        );
        short_put.premium = Positive::ZERO;

        let result = BullPutSpread::get_strategy(&[long_put, short_put]);
        assert!(matches!(
            result,
            Err(StrategyError::InvalidStrategy {
                strategy: StrategyType::BullPutSpread,
                ..
            })
        ));
    }

    #[test]
    fn test_get_strategy_wrong_number_of_options() {
        let options = vec![create_sample_position(
            OptionStyle::Put,
            Side::Short,
            Positive::HUNDRED,
            Positive::ONE,
            pos_or_panic!(95.0),
            pos_or_panic!(0.2),
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
            Positive::HUNDRED,
            Positive::ONE,
            pos_or_panic!(95.0),
            pos_or_panic!(0.2),
        );
        option1.option.option_style = OptionStyle::Call;
        let option2 = create_sample_position(
            OptionStyle::Put,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            pos_or_panic!(105.0),
            pos_or_panic!(0.2),
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
                Side::Long,
                Positive::HUNDRED,
                Positive::ONE,
                pos_or_panic!(115.0),
                pos_or_panic!(0.2),
            ),
            create_sample_position(
                OptionStyle::Put,
                Side::Long,
                Positive::HUNDRED,
                Positive::ONE,
                pos_or_panic!(105.0),
                pos_or_panic!(0.2),
            ),
        ];
        let result = BullPutSpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Bull Put Spread get_strategy"
                && reason == "Bull Put Spread requires a long lower strike put and a short higher strike put"
        ));
    }

    #[test]
    fn test_get_strategy_different_expiration_dates() {
        let mut option1 = create_sample_position(
            OptionStyle::Put,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            pos_or_panic!(95.0),
            pos_or_panic!(0.2),
        );
        let mut option2 = create_sample_position(
            OptionStyle::Put,
            Side::Short,
            Positive::HUNDRED,
            Positive::ONE,
            pos_or_panic!(105.0),
            pos_or_panic!(0.2),
        );

        option1.option.expiration_date = ExpirationDate::Days(pos_or_panic!(30.0));
        option2.option.expiration_date = ExpirationDate::Days(pos_or_panic!(60.0));

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

    use optionstratlib_core::assert_decimal_eq;
    use optionstratlib_core::model::utils::create_sample_position;
    use rust_decimal_macros::dec;

    // Textbook bull put spread (#696): long the 95 put for 1.00, short the
    // 100 put for 3.00, no fees. Net credit 2.00, spread width 5.00, so the
    // hand-computed expiry payoff is +2.00 above 100, -3.00 below 95 and
    // crosses zero at 100 - 2 = 98.
    fn create_test_bull_put_spread() -> Result<BullPutSpread, StrategyError> {
        let mut long_put = create_sample_position(
            OptionStyle::Put,
            Side::Long,
            Positive::HUNDRED,   // Underlying price
            Positive::ONE,       // Quantity
            pos_or_panic!(95.0), // Lower strike price
            pos_or_panic!(0.2),  // Implied volatility
        );
        long_put.premium = Positive::ONE;
        long_put.open_fee = Positive::ZERO;
        long_put.close_fee = Positive::ZERO;

        let mut short_put = create_sample_position(
            OptionStyle::Put,
            Side::Short,
            Positive::HUNDRED,  // Same underlying price
            Positive::ONE,      // Quantity
            Positive::HUNDRED,  // Higher strike price
            pos_or_panic!(0.2), // Implied volatility
        );
        short_put.premium = pos_or_panic!(3.0);
        short_put.open_fee = Positive::ZERO;
        short_put.close_fee = Positive::ZERO;

        BullPutSpread::get_strategy(&[short_put, long_put])
    }

    #[test]
    fn test_bull_put_spread_is_credit_spread_with_hand_computed_break_even() {
        let spread = create_test_bull_put_spread().unwrap();

        // A credit spread: the short leg's premium exceeds the long leg's.
        assert_eq!(spread.get_net_premium_received().unwrap(), Positive::TWO);
        assert_decimal_eq!(spread.get_net_cost().unwrap(), dec!(-2.0), dec!(1e-9));

        // Break-even = short strike - net credit = 100 - 2 = 98.
        assert_eq!(
            spread.get_break_even_points().unwrap(),
            &vec![pos_or_panic!(98.0)]
        );
        // Max profit is the credit, max loss the width minus the credit.
        assert_eq!(spread.get_max_profit().unwrap(), Positive::TWO);
        assert_eq!(spread.get_max_loss().unwrap(), pos_or_panic!(3.0));
    }

    #[test]
    fn test_calculate_pnl_all_options_otm() {
        let spread = create_test_bull_put_spread().unwrap();
        let market_price = pos_or_panic!(105.0); // Above both strikes
        let expiration_date = ExpirationDate::Days(pos_or_panic!(30.0));
        let implied_volatility = pos_or_panic!(0.2);

        let result = spread.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        // `unrealized` is the change in the legs' Black-Scholes value since
        // entry at 100. The spread is long delta, so a rally gains, and no
        // move can exceed the 5.00 width.
        let unrealized = pnl.unrealized.unwrap();
        assert!(unrealized > Decimal::ZERO);
        assert!(unrealized < dec!(5.0));
        assert_eq!(pnl.initial_income, pos_or_panic!(3.0));
        assert_eq!(pnl.initial_costs, Positive::ONE);
    }

    #[test]
    fn test_calculate_pnl_mixed_moneyness() {
        let spread = create_test_bull_put_spread().unwrap();
        let market_price = pos_or_panic!(97.5); // Between strikes
        let expiration_date = ExpirationDate::Days(pos_or_panic!(30.0));
        let implied_volatility = pos_or_panic!(0.2);

        let result = spread.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        // A fall from the 100 entry hurts a long-delta spread, within the width.
        let pnl = result.unwrap();
        let unrealized = pnl.unrealized.unwrap();
        assert!(unrealized < Decimal::ZERO);
        assert!(unrealized > dec!(-5.0));

        // Initial values: the 3.00 short premium in, the 1.00 long premium out.
        assert_eq!(pnl.initial_income, pos_or_panic!(3.0));
        assert_eq!(pnl.initial_costs, Positive::ONE);
    }

    #[test]
    fn test_calculate_pnl_all_options_itm() {
        let spread = create_test_bull_put_spread().unwrap();
        let market_price = pos_or_panic!(90.0); // Below both strikes
        let expiration_date = ExpirationDate::Days(pos_or_panic!(30.0));
        let implied_volatility = pos_or_panic!(0.2);

        let result = spread.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        // Both puts ITM: the position is losing, within the width.
        let pnl = result.unwrap();
        let unrealized = pnl.unrealized.unwrap();
        assert!(unrealized < Decimal::ZERO);
        assert!(unrealized > dec!(-5.0));

        // Initial values: the 3.00 short premium in, the 1.00 long premium out.
        assert_eq!(pnl.initial_income, pos_or_panic!(3.0));
        assert_eq!(pnl.initial_costs, Positive::ONE);
    }

    #[test]
    fn test_calculate_pnl_at_expiration_maximum_profit() {
        let spread = create_test_bull_put_spread().unwrap();

        // Above 100 both puts expire worthless and the credit is kept.
        let pnl = spread
            .calculate_pnl_at_expiration(&pos_or_panic!(105.0))
            .unwrap();
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(2.0), dec!(1e-6));
        let pnl = spread
            .calculate_pnl_at_expiration(&Positive::HUNDRED)
            .unwrap();
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(2.0), dec!(1e-6));
        assert_eq!(pnl.initial_income, pos_or_panic!(3.0));
        assert_eq!(pnl.initial_costs, Positive::ONE);
    }

    #[test]
    fn test_calculate_pnl_at_expiration_maximum_loss() {
        let spread = create_test_bull_put_spread().unwrap();

        // At 90: long 95 put pays 5, short 100 put owes 10, plus the 2 credit.
        // Total = 5 - 10 + 2 = -3, the max loss; the same at the 95 strike.
        let pnl = spread
            .calculate_pnl_at_expiration(&pos_or_panic!(90.0))
            .unwrap();
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(-3.0), dec!(1e-6));
        let pnl = spread
            .calculate_pnl_at_expiration(&pos_or_panic!(95.0))
            .unwrap();
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(-3.0), dec!(1e-6));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_breakeven() {
        let spread = create_test_bull_put_spread().unwrap();

        // At 98: short 100 put owes 2, which the 2 credit offsets exactly.
        let pnl = spread
            .calculate_pnl_at_expiration(&pos_or_panic!(98.0))
            .unwrap();
        assert_decimal_eq!(pnl.realized.unwrap(), Decimal::ZERO, dec!(1e-6));

        // At 97.5, between the strikes: 2 - 2.5 = -0.5.
        let pnl = spread
            .calculate_pnl_at_expiration(&pos_or_panic!(97.5))
            .unwrap();
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(-0.5), dec!(1e-6));
    }

    #[test]
    fn test_calculate_pnl_volatility_sensitivity() {
        let spread = create_test_bull_put_spread().unwrap();
        let market_price = pos_or_panic!(97.5); // Between strikes
        let expiration_date = ExpirationDate::Days(pos_or_panic!(30.0));

        let low_vol_result = spread
            .calculate_pnl(&market_price, expiration_date, &pos_or_panic!(0.1))
            .unwrap();
        let high_vol_result = spread
            .calculate_pnl(&market_price, expiration_date, &pos_or_panic!(0.3))
            .unwrap();

        // Near the money the short 100 put carries more vega than the long
        // 95 put, so the spread is short vega and loses as volatility rises.
        assert!(high_vol_result.unrealized.unwrap() < low_vol_result.unrealized.unwrap());

        // Initial values do not depend on the pricing volatility.
        assert_eq!(high_vol_result.initial_income, pos_or_panic!(3.0));
        assert_eq!(high_vol_result.initial_costs, Positive::ONE);
        assert_eq!(low_vol_result.initial_income, pos_or_panic!(3.0));
        assert_eq!(low_vol_result.initial_costs, Positive::ONE);
    }
}
