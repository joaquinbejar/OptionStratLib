// Scoped allow: bulk migration of unchecked `[]` indexing to
// `.get().ok_or_else(..)` tracked as follow-ups to #341. The existing
// call sites are internal to this file and audited for invariant-bound
// indices (fixed-length buffers, just-pushed slices, etc.).
#![allow(clippy::indexing_slicing)]

use super::base::{BreakEvenable, Positionable, StrategyType};
use crate::model::decimal::d_div;
use crate::strategies::base::lower_break_even;

use crate::analytics::probability::VolatilityAdjustment;
use crate::chains::OptionChain;
use crate::error::strategies::ProfitLossErrorKind;
use crate::error::{
    GreeksError, PricingError, ProbabilityError, StrategyError,
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
use crate::strategies::base::Optimizable;
use crate::strategies::base::price_gap;
use crate::strategies::delta_neutral::DeltaNeutrality;
use crate::strategies::probabilities::core::ProbabilityAnalysis;
use crate::strategies::utils::OptimizationCriteria;
use crate::strategies::{
    BasicAble, DeltaAdjustment, FindOptimalSide, Strategable, Strategies, StrategyConstructor,
    Validable,
};
use crate::{ExpirationDate, Options, test_strategy_traits};
use chrono::Utc;
use num_traits::FromPrimitive;
use positive::Positive;
use pretty_simple_display::{DebugPretty, DisplaySimple};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{debug, warn};
use utoipa::ToSchema;

pub(super) const LONG_PUT_DESCRIPTION: &str = "A Long Put is an options strategy where the trader purchases a put option, gaining the right (but not the obligation) to sell the underlying asset at the strike price until expiration. \
    This strategy requires an initial investment (the premium paid) and provides downside protection or profit potential if the underlying asset's price decreases. \
    The breakeven point is the strike price minus the premium paid. Long puts are commonly used as insurance against price declines or to express a bearish outlook.";

/// Represents a Long Put options trading strategy.
///
/// A Long Put strategy is used when a trader expects the price of the underlying asset
/// to decrease significantly. It involves purchasing a put option with the anticipation
/// of profiting as the underlying asset's price falls below the strike price of the option.
///
/// # Fields
///
/// * `name` - A unique name identifier for this specific instance of the Long Put strategy.
/// * `kind` - The type of strategy, identified specifically as `StrategyType::LongPut`.
/// * `description` - A detailed description of this particular instance of the Long Put strategy,
///   providing additional context or information.
/// * `break_even_points` - A vector of price points (`Positive`) where the strategy neither gains
///   nor loses money based on the underlying asset's movement.
/// * `long_put` - Represents the specific long put position within the strategy, detailing
///   the option contract being used.
#[derive(Clone, DebugPretty, DisplaySimple, Serialize, Deserialize, ToSchema)]
pub struct LongPut {
    /// Name identifier for this specific strategy instance
    pub name: String,
    /// Identifies this as a LongPut strategy type
    pub kind: StrategyType,
    /// Detailed description of this strategy instance
    pub description: String,
    /// Price points where the strategy neither makes nor loses money
    pub break_even_points: Vec<Positive>,
    /// The long put position
    pub(super) long_put: Position,
}

impl LongPut {
    /// Constructs a new instance of a `LongPut` strategy.
    ///
    /// This method initializes a `LongPut` strategy based on the given parameters,
    /// including details about the underlying asset, option specifications, and associated fees.
    ///
    /// # Parameters
    ///
    /// * `underlying_symbol` - Symbol of the underlying asset for the long put option.
    /// * `long_put_strike` - Strike price of the long put option. Must be a positive value.
    /// * `long_put_expiration` - Expiration date of the long put option.
    /// * `implied_volatility` - Implied volatility of the underlying asset. Must be a positive value.
    /// * `quantity` - Quantity of contracts. Must be a positive value.
    /// * `underlying_price` - Current price of the underlying asset. Must be a positive value.
    /// * `risk_free_rate` - Risk-free rate used for pricing the option (in decimal format).
    /// * `dividend_yield` - Dividend yield of the underlying asset. Must be a positive value.
    /// * `premium_long_put` - Premium cost of the long put option. Must be a positive value.
    /// * `open_fee_long_put` - Fee incurred when opening the long put position. Must be a positive value.
    /// * `close_fee_long_put` - Fee incurred when closing the long put position. Must be a positive value.
    ///
    /// # Returns
    ///
    /// A new instance of the `LongPut` strategy initialized with the provided parameters.
    ///
    /// # Errors
    /// Returns `StrategyError` if the freshly-constructed long put leg
    /// cannot be added to the strategy. In practice this branch is
    /// unreachable for a freshly-built single-leg strategy and is surfaced
    /// only to keep the constructor panic-free.
    ///
    #[allow(clippy::too_many_arguments, dead_code)]
    fn new(
        underlying_symbol: String,
        long_put_strike: Positive,
        long_put_expiration: ExpirationDate,
        implied_volatility: Positive,
        quantity: Positive,
        underlying_price: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        premium_long_put: Positive,
        open_fee_long_put: Positive,
        close_fee_long_put: Positive,
    ) -> Result<Self, StrategyError> {
        let mut strategy = LongPut::default();

        let long_put_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol,
            long_put_strike,
            long_put_expiration,
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
        strategy.add_position(&long_put)?;

        Ok(strategy)
    }
}

impl BasicAble for LongPut {
    fn get_title(&self) -> String {
        let strategy_title = format!("{:?} Strategy: ", self.kind);
        let leg_titles: Vec<String> = [self.long_put.get_title()]
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

        hash_set.insert(OptionBasicType {
            option_style: &long_put.option_style,
            side: &long_put.side,
            strike_price: &long_put.strike_price,
            expiration_date: &long_put.expiration_date,
        });

        hash_set
    }
    fn get_implied_volatility(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let options = [(
            &self.long_put.option,
            &self.long_put.option.implied_volatility,
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
        let options = [(&self.long_put.option, &self.long_put.option.quantity)];

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
        self.long_put.one_option()
    }
    fn one_option_mut(&mut self) -> &mut Options {
        self.long_put.one_option_mut()
    }
    fn set_expiration_date(
        &mut self,
        expiration_date: ExpirationDate,
    ) -> Result<(), StrategyError> {
        self.long_put.option.expiration_date = expiration_date;
        Ok(())
    }
    fn set_underlying_price(&mut self, price: &Positive) -> Result<(), StrategyError> {
        self.long_put.option.underlying_price = *price;
        self.long_put.premium =
            Positive::new_decimal(self.long_put.option.calculate_price_black_scholes()?.abs())
                .unwrap_or(Positive::ZERO);
        Ok(())
    }
    fn set_implied_volatility(&mut self, volatility: &Positive) -> Result<(), StrategyError> {
        self.long_put.option.implied_volatility = *volatility;
        self.long_put.premium =
            Positive::new_decimal(self.long_put.option.calculate_price_black_scholes()?.abs())
                .unwrap_or(Positive::ZERO);
        Ok(())
    }
}

impl Validable for LongPut {
    fn validate(&self) -> bool {
        if !self.long_put.validate() {
            debug!("Long call is invalid");
            return false;
        }
        true
    }
}

impl BreakEvenable for LongPut {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        // `lower_break_even` measures a distance below the strike and floors at
        // zero. A credit larger than the strike puts the break-even where the
        // underlying cannot trade, and `Positive::ZERO` is this layer's
        // sentinel for "no lower break-even" rather than an abort.
        let per_contract = d_div(
            self.get_net_cost()?,
            self.long_put.option.quantity.to_dec(),
            "LongPut::update_break_even_points",
        )?;
        self.break_even_points.push(
            lower_break_even(self.long_put.option.strike_price, per_contract)
                .checked_round_to(2)?,
        );

        Ok(())
    }
}

impl Strategies for LongPut {
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        // Max profit for a long put occurs at price = 0: strike - premium.
        let profit = self.calculate_profit_at(&Positive::ZERO)?;
        if profit >= Decimal::ZERO {
            Ok(Positive::new_decimal(profit)?)
        } else {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxProfitError {
                    reason: "Max profit is negative".to_string(),
                },
            ))
        }
    }
    fn get_max_loss(&self) -> Result<Positive, StrategyError> {
        // Max loss for a long put is the premium paid (at any price ≥ strike).
        Ok(self.get_total_cost()?)
    }
    fn get_profit_area(&self) -> Result<Decimal, StrategyError> {
        let high = self.get_max_profit().unwrap_or(Positive::ZERO);
        let break_even = self.break_even_points.first().ok_or_else(|| {
            StrategyError::empty_collection("LongPut::get_profit_area: no break-even points")
        })?;
        let base = price_gap(self.long_put.option.strike_price, *break_even);
        Ok(Decimal::from_f64(high.to_f64() * base.to_f64() / 200.0).unwrap_or(Decimal::ZERO))
    }
    fn get_profit_ratio(&self) -> Result<Decimal, StrategyError> {
        let max_profit = self.get_max_profit().unwrap_or(Positive::ZERO);
        let max_loss = self.get_max_loss().unwrap_or(Positive::ZERO);
        match (max_profit, max_loss) {
            (value, _) if value == Positive::ZERO => Ok(Decimal::ZERO),
            (_, value) if value == Positive::ZERO => Ok(Decimal::MAX),
            _ => Ok(
                Decimal::from_f64(max_profit.to_f64() / max_loss.to_f64() * 100.0)
                    .unwrap_or(Decimal::ZERO),
            ),
        }
    }
}

impl Profit for LongPut {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, PricingError> {
        let price = Some(price);
        Ok(self.long_put.pnl_at_expiration(&price)?)
    }
}

impl Positionable for LongPut {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match (position.option.option_style, position.option.side) {
            (OptionStyle::Put, Side::Long) => {
                self.long_put = position.clone();
                Ok(())
            }
            _ => Err(PositionError::invalid_position_style(
                position.option.option_style,
                "Position is a Put or Long, it is not valid for LongPut".to_string(),
            )),
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.long_put])
    }

    /// Gets mutable positions matching the specified criteria from the strategy.
    ///
    /// # Arguments
    /// * `option_style` - The style of the option (Put/Put)
    /// * `side` - The side of the position (Long/Long)
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
            (Side::Long, OptionStyle::Put, strike)
                if *strike == self.long_put.option.strike_price =>
            {
                Ok(vec![&mut self.long_put])
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
            (Side::Long, OptionStyle::Put, strike)
                if *strike == self.long_put.option.strike_price =>
            {
                self.long_put = position.clone();
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

impl StrategyConstructor for LongPut {
    fn get_strategy(_vec_positions: &[Position]) -> Result<Self, StrategyError> {
        Err(StrategyError::operation_not_supported(
            "get_strategy",
            "LongPut",
        ))
    }
}

impl Optimizable for LongPut {
    type Strategy = Self;

    fn find_optimal(
        &mut self,
        _option_chain: &OptionChain,
        _side: FindOptimalSide,
        _criteria: OptimizationCriteria,
    ) {
        warn!("find_optimal: stub — no optimization performed for LongPut");
    }
}

impl ProbabilityAnalysis for LongPut {
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        // Long put is profitable when price falls below break-even
        let break_even = self.break_even_points.first().ok_or_else(|| {
            ProbabilityError::RangeError(ProfitLossRangeErrorKind::InvalidBreakEvenPoints {
                reason: "No break-even points found for long put".to_string(),
            })
        })?;

        let option = &self.long_put.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let mut profit_range = ProfitLossRange::new(None, Some(*break_even), Positive::ZERO)?;

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
        // Long put has losses when price stays above break-even
        let break_even = self.break_even_points.first().ok_or_else(|| {
            ProbabilityError::RangeError(ProfitLossRangeErrorKind::InvalidBreakEvenPoints {
                reason: "No break-even points found for long put".to_string(),
            })
        })?;

        let option = &self.long_put.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let mut loss_range = ProfitLossRange::new(Some(*break_even), None, Positive::ZERO)?;

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

impl Greeks for LongPut {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.long_put.option])
    }
}

impl DeltaNeutrality for LongPut {}

impl PnLCalculator for LongPut {
    fn calculate_pnl(
        &self,
        market_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, PricingError> {
        self.long_put
            .calculate_pnl(market_price, expiration_date, implied_volatility)
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<PnL, PricingError> {
        self.long_put.calculate_pnl_at_expiration(underlying_price)
    }

    fn adjustments_pnl(&self, _adjustment: &DeltaAdjustment) -> Result<PnL, PricingError> {
        // Single-leg strategies like LongPut don't typically require delta adjustments
        // as they are directional strategies. Delta adjustments are more relevant for
        // complex multi-leg strategies aiming for delta neutrality.
        Err(PricingError::DeltaAdjustmentNotApplicable {
            strategy: "LongPut",
        })
    }
}

impl Strategable for LongPut {}

test_strategy_traits!(LongPut, test_long_put_implementations);

#[cfg(test)]
mod tests_break_even {
    use super::*;
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    /// A long put pays a debit, so its lower break-even sits *below* the
    /// strike: a 100 strike bought for 5 breaks even at 95. Negating the
    /// per-contract cost moved it the other way and reported 105. The
    /// `StrategyConstructor` property cannot reach this, since `LongPut`
    /// answers `OperationNotSupported` and that branch never runs.
    #[test]
    fn test_long_put_lower_break_even_sits_below_the_strike() {
        let mut long_put = LongPut::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.20),
            Positive::ONE,
            Positive::HUNDRED,
            dec!(0.05),
            Positive::ZERO,
            pos_or_panic!(5.0),
            Positive::ZERO,
            Positive::ZERO,
        )
        .unwrap();
        long_put.update_break_even_points().unwrap();

        let break_evens = long_put.get_break_even_points().unwrap();
        assert_eq!(break_evens.len(), 1);
        assert_eq!(break_evens[0], pos_or_panic!(95.0));
    }
}
