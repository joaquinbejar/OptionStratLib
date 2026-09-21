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

pub(super) const SHORT_CALL_DESCRIPTION: &str = "A Short Call (or Naked Call) is an options strategy where the trader sells a call option without owning the underlying stock. \
    This strategy generates immediate income through the premium received but carries unlimited risk if the stock price rises significantly. \
    The breakeven point is the strike price plus the premium received. Short calls are generally used when the trader has a bearish or neutral outlook on the underlying asset.";

/// Represents the details and structure of a Short Call options trading strategy.
///
/// A Short Call strategy involves selling a call option, which gives the buyer
/// the right to purchase the underlying asset at a specific strike price before
/// the expiration date. This strategy is generally employed when the trader
/// expects minimal movement or a decrease in the price of the underlying asset.
///
/// # Fields
///
/// * `name` - A unique name or identifier for this specific instance of the strategy.
/// * `kind` - Specifies that this instance is of the `ShortCall` strategy type.
/// * `description` - A detailed explanation providing more information about the strategy instance.
/// * `break_even_points` - A vector containing the price points where the strategy does not yield
///   any profit or loss. These points are represented as positive values.
/// * `short_call` - Represents the short call position in the strategy, which involves selling
///   a call option to generate premium income.
#[derive(Clone, DebugPretty, DisplaySimple, Serialize, Deserialize, ToSchema)]
pub struct ShortCall {
    /// Name identifier for this specific strategy instance
    pub name: String,
    /// Identifies this as a ShortCall strategy type
    pub kind: StrategyType,
    /// Detailed description of this strategy instance
    pub description: String,
    /// Price points where the strategy neither makes nor loses money
    pub break_even_points: Vec<Positive>,
    /// The short call option
    pub(super) short_call: Position,
}

impl ShortCall {
    /// Creates a new `ShortCall` strategy instance with the specified parameters.
    ///
    /// The `new` function initializes a short call option strategy by creating an associated
    /// option position and adding it to the strategy. This function is marked with
    /// `#[allow(clippy::too_many_arguments)]` because it takes several parameters required to
    /// define the short call options and associated financial metrics.
    ///
    /// # Parameters
    ///
    /// - `underlying_symbol` (`String`): The symbol of the underlying asset for the short call option.
    /// - `short_call_strike` (`Positive`): The strike price of the short call option.
    /// - `short_call_expiration` (`ExpirationDate`): The expiration date of the short call option.
    /// - `implied_volatility` (`Positive`): The implied volatility of the short call option.
    /// - `quantity` (`Positive`): The quantity of contracts for the short call option.
    /// - `underlying_price` (`Positive`): The current price of the underlying asset.
    /// - `risk_free_rate` (`Decimal`): The risk-free interest rate as a percentage.
    /// - `dividend_yield` (`Positive`): The dividend yield of the underlying asset as a percentage.
    /// - `premium_short_call` (`Positive`): Premium received for selling the short call option.
    /// - `open_fee_short_call` (`Positive`): Opening fee for the short call position.
    /// - `close_fee_short_call` (`Positive`): Closing fee for the short call position.
    ///
    /// # Returns
    ///
    /// Returns an initialized `ShortCall` strategy instance. The instance includes the short call
    /// option position with the specified parameters.
    ///
    /// # Errors
    /// Returns `StrategyError` if the freshly-constructed short call leg
    /// cannot be added to the strategy. In practice this branch is
    /// unreachable for a freshly-built single-leg strategy and is surfaced
    /// only to keep the constructor panic-free.
    ///
    #[allow(clippy::too_many_arguments, dead_code)]
    fn new(
        underlying_symbol: String,
        short_call_strike: Positive,
        short_call_expiration: ExpirationDate,
        implied_volatility: Positive,
        quantity: Positive,
        underlying_price: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        premium_short_call: Positive,
        open_fee_short_call: Positive,
        close_fee_short_call: Positive,
    ) -> Result<Self, StrategyError> {
        let mut strategy = ShortCall::default();

        let short_call_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol,
            short_call_strike,
            short_call_expiration,
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
            None,
            None,
        );
        strategy.add_position(&short_call)?;

        Ok(strategy)
    }
}

impl BasicAble for ShortCall {
    fn get_title(&self) -> String {
        let strategy_title = format!("{:?} Strategy: ", self.kind);
        let leg_titles: Vec<String> = [self.short_call.get_title()]
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
        let short_call = &self.short_call.option;

        hash_set.insert(OptionBasicType {
            option_style: &short_call.option_style,
            side: &short_call.side,
            strike_price: &short_call.strike_price,
            expiration_date: &short_call.expiration_date,
        });

        hash_set
    }
    fn get_implied_volatility(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let options = [(
            &self.short_call.option,
            &self.short_call.option.implied_volatility,
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
        let options = [(&self.short_call.option, &self.short_call.option.quantity)];

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
        self.short_call.option.expiration_date = expiration_date;
        Ok(())
    }
    fn set_underlying_price(&mut self, price: &Positive) -> Result<(), StrategyError> {
        self.short_call.option.underlying_price = *price;
        self.short_call.premium = Positive::new_decimal(
            self.short_call
                .option
                .calculate_price_black_scholes()?
                .abs(),
        )
        .unwrap_or(Positive::ZERO);
        Ok(())
    }
    fn set_implied_volatility(&mut self, volatility: &Positive) -> Result<(), StrategyError> {
        self.short_call.option.implied_volatility = *volatility;
        self.short_call.premium = Positive::new_decimal(
            self.short_call
                .option
                .calculate_price_black_scholes()?
                .abs(),
        )
        .unwrap_or(Positive::ZERO);
        Ok(())
    }
}

impl Validable for ShortCall {
    fn validate(&self) -> bool {
        if !self.short_call.validate() {
            debug!("Long call is invalid");
            return false;
        }
        true
    }
}

impl BreakEvenable for ShortCall {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        // For a short call, net_cost() from Position returns (fees - premium_received).
        // Break-even = strike + (premium_received - fees) / quantity
        // So, break-even = strike - (fees - premium_received) / quantity
        // Which is strike - (net_cost_from_position / quantity)
        // `lower_break_even` measures a distance below the strike and floors at
        // zero. A credit larger than the strike puts the break-even where the
        // underlying cannot trade, and `Positive::ZERO` is this layer's
        // sentinel for "no lower break-even" rather than an abort.
        let per_contract = d_div(
            self.short_call.net_cost()?,
            self.short_call.option.quantity.to_dec(),
            "ShortCall::update_break_even_points",
        )?;
        self.break_even_points.push(
            lower_break_even(self.short_call.option.strike_price, per_contract)
                .checked_round_to(2)?,
        );

        Ok(())
    }
}

impl Strategies for ShortCall {
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        // Max profit for a short call is the net premium received (at any price ≤ strike).
        self.get_net_premium_received()
    }
    fn get_max_loss(&self) -> Result<Positive, StrategyError> {
        Ok(Positive::MAX) // Theoretically unlimited
    }
    fn get_profit_area(&self) -> Result<Decimal, StrategyError> {
        let high = self.get_max_profit().unwrap_or(Positive::ZERO);
        let break_even = self.break_even_points.first().ok_or_else(|| {
            StrategyError::empty_collection("ShortCall::get_profit_area: no break-even points")
        })?;
        let base = price_gap(self.short_call.option.strike_price, *break_even);
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

impl Profit for ShortCall {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, PricingError> {
        let price = Some(price);
        Ok(self.short_call.pnl_at_expiration(&price)?)
    }
}

impl Positionable for ShortCall {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match (position.option.option_style, position.option.side) {
            (OptionStyle::Call, Side::Short) => {
                self.short_call = position.clone();
                Ok(())
            }
            _ => Err(PositionError::invalid_position_style(
                position.option.option_style,
                "Position is a Put or Long, it is not valid for ShortCall".to_string(),
            )),
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.short_call])
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
            (Side::Short, OptionStyle::Call, strike)
                if *strike == self.short_call.option.strike_price =>
            {
                self.short_call = position.clone();
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

impl StrategyConstructor for ShortCall {
    fn get_strategy(_vec_positions: &[Position]) -> Result<Self, StrategyError> {
        Err(StrategyError::operation_not_supported(
            "get_strategy",
            "ShortCall",
        ))
    }
}

impl Optimizable for ShortCall {
    type Strategy = Self;

    fn find_optimal(
        &mut self,
        _option_chain: &OptionChain,
        _side: FindOptimalSide,
        _criteria: OptimizationCriteria,
    ) {
        warn!("find_optimal: stub — no optimization performed for ShortCall");
    }
}

impl ProbabilityAnalysis for ShortCall {
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        // Short call is profitable when price stays below break-even
        let break_even = self.break_even_points.first().ok_or_else(|| {
            ProbabilityError::RangeError(ProfitLossRangeErrorKind::InvalidBreakEvenPoints {
                reason: "No break-even points found for short call".to_string(),
            })
        })?;

        let option = &self.short_call.option;
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
        // Short call has losses when price rises above break-even
        let break_even = self.break_even_points.first().ok_or_else(|| {
            ProbabilityError::RangeError(ProfitLossRangeErrorKind::InvalidBreakEvenPoints {
                reason: "No break-even points found for short call".to_string(),
            })
        })?;

        let option = &self.short_call.option;
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

impl Greeks for ShortCall {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.short_call.option])
    }
}

impl DeltaNeutrality for ShortCall {}

impl PnLCalculator for ShortCall {
    fn calculate_pnl(
        &self,
        market_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, PricingError> {
        self.short_call
            .calculate_pnl(market_price, expiration_date, implied_volatility)
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<PnL, PricingError> {
        self.short_call
            .calculate_pnl_at_expiration(underlying_price)
    }

    fn adjustments_pnl(&self, _adjustment: &DeltaAdjustment) -> Result<PnL, PricingError> {
        // Single-leg strategies like ShortCall don't typically require delta adjustments
        // as they are directional strategies. Delta adjustments are more relevant for
        // complex multi-leg strategies aiming for delta neutrality.
        Err(PricingError::DeltaAdjustmentNotApplicable {
            strategy: "ShortCall",
        })
    }
}

impl Strategable for ShortCall {}

test_strategy_traits!(ShortCall, test_short_call_implementations);
