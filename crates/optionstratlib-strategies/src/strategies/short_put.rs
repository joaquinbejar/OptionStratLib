use super::base::{BreakEvenable, Positionable, StrategyType};
use crate::strategies::base::lower_break_even;
use crate::strategies::shared::decimal_from_f64;
use crate::strategies::shared::{
    CachedBreakEvens, apply_contract_size, common_contract_size, edit_refreshing_break_evens,
};
use crate::strategies::shared::{measured_max_loss, measured_max_profit};
use optionstratlib_core::model::decimal::d_div;
use optionstratlib_core::{impl_json_debug_pretty, impl_json_display};

use crate::error::StrategyError;
use crate::error::strategies::ProfitLossErrorKind;
use crate::strategies::base::Optimizable;
use crate::strategies::base::price_gap;
use crate::strategies::delta_neutral::DeltaNeutrality;
use crate::strategies::probabilities::core::ProbabilityAnalysis;
use crate::strategies::utils::OptimizationCriteria;
use crate::strategies::{BasicAble, Strategable, Strategies, StrategyConstructor, Validable};
use chrono::Utc;
use optionstratlib_analytics::analytics::ProfitLossRange;
use optionstratlib_analytics::analytics::probability::VolatilityAdjustment;
use optionstratlib_analytics::error::ProbabilityError;
use optionstratlib_analytics::error::probability::ProfitLossRangeErrorKind;
use optionstratlib_analytics::pnl::DeltaAdjustment;
use optionstratlib_analytics::pnl::{PnL, PnLCalculator};
use optionstratlib_core::error::position::{PositionError, PositionValidationErrorKind};
use optionstratlib_core::model::ExpirationDate;
use optionstratlib_core::model::Options;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::{
    position::Position,
    types::{OptionBasicType, OptionStyle, OptionType, Side},
};
use optionstratlib_market::chains::OptionChain;
use optionstratlib_market::chains::utils::FindOptimalSide;
use optionstratlib_pricing::error::GreeksError;
use optionstratlib_pricing::error::PricingError;
use optionstratlib_pricing::greeks::Greeks;
use optionstratlib_pricing::pricing::OptionPricing;
use optionstratlib_pricing::pricing::Profit;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::debug;

pub(super) const SHORT_PUT_DESCRIPTION: &str = "A Short Put (or Naked Put) is an options strategy where the trader sells a put option without holding a short position in the underlying stock. \
    This strategy provides immediate income from the premium collected but includes substantial risk if the stock price falls below the strike price. \
    The breakeven point is the strike price minus the premium received. Short puts are typiputy employed when the trader has a bullish or neutral market outlook.";

/// Represents a Short Put options trading strategy.
///
/// A short put is a neutral to bullish strategy involving the sale of a put option.
/// This strategy generates a credit upfront, with the potential obligation to buy the underlying asset
/// at the strike price if the price falls below it. Below are the details stored in this struct:
///
/// Fields:
/// - `name`: The name identifier for this specific strategy instance. This is used to uniquely recognize
///   and distinguish this instance.
/// - `kind`: A field that identifies this as a ShortPut strategy type. It is of type `StrategyType`,
///   which categorizes different trading strategies.
/// - `description`: A detailed description of this strategy instance. This field allows for additional
///   explanation or metadata about why this strategy is being used or how it functions.
/// - `break_even_points`: A vector of price points (of type `Positive`) where the strategy neither makes
///   nor loses money. These are the threshold price levels that determine profitability.
/// - `short_put`: The short put position associated with this strategy. It is declared private (via
///   `pub(super)`) to restrict its accessibility from other modules, ensuring controlled and encapsulated
///   use.
#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
pub struct ShortPut {
    /// Name identifier for this specific strategy instance
    pub name: String,
    /// Identifies this as a ShortPut strategy type
    pub kind: StrategyType,
    /// Detailed description of this strategy instance
    pub description: String,
    /// Price points where the strategy neither makes nor loses money
    pub break_even_points: Vec<Positive>,
    /// The short put position
    pub(super) short_put: Position,
}

impl_json_debug_pretty!(ShortPut);
impl_json_display!(ShortPut);

impl ShortPut {
    /// Creates a new `ShortPut` strategy instance with the given parameters.
    ///
    /// This function constructs a `ShortPut` strategy by initializing a short put position
    /// using the provided parameters and adding it to the strategy's list of positions. The
    /// required inputs are both descriptive and numerical attributes of the short put option,
    /// such as the underlying symbol, strike price, expiration date, volatility, and fees.
    ///
    /// # Parameters
    ///
    /// - `underlying_symbol`: A `String` representing the symbol of the underlying asset.
    /// - `short_put_strike`: A `Positive` value specifying the strike price of the short put option.
    ///   This should always be greater than zero.
    /// - `short_put_expiration`: The `ExpirationDate` when the short put option expires.
    /// - `implied_volatility`: A `Positive` value representing the implied volatility of the option.
    /// - `quantity`: A `Positive` value indicating the number of contracts in the position.
    /// - `underlying_price`: A `Positive` value representing the current price of the underlying asset.
    /// - `risk_free_rate`: A `Decimal` representing the risk-free interest rate, expressed as a decimal.
    /// - `dividend_yield`: A `Positive` value representing the dividend yield of the underlying asset.
    /// - `premium_short_put`: A `Positive` value representing the premium received when selling the put option.
    /// - `open_fee_short_put`: A `Positive` value indicating the opening fee for the short put position.
    /// - `close_fee_short_put`: A `Positive` value indicating the closing fee for the short put position.
    ///
    /// # Returns
    ///
    /// A new instance of `ShortPut` containing the initialized short put position.
    ///
    /// # Errors
    ///
    /// Returns `StrategyError::InvalidStrategy` when the assembled strategy
    /// fails its own `validate` (#696): the leg fails `Position::validate` (for
    /// example no premium).
    ///
    /// Returns `StrategyError` if the freshly-constructed short put leg
    /// cannot be added to the strategy. In practice this branch is
    /// unreachable for a freshly-built single-leg strategy and is surfaced
    /// only to keep the constructor panic-free.
    ///
    /// Returns `StrategyError` when the break-even computation fails, for
    /// example on a premium whose total overflows.
    ///
    #[allow(clippy::too_many_arguments, dead_code)]
    #[inline(never)]
    pub fn new(
        underlying_symbol: String,
        short_put_strike: Positive,
        short_put_expiration: ExpirationDate,
        implied_volatility: Positive,
        quantity: Positive,
        underlying_price: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        premium_short_put: Positive,
        open_fee_short_put: Positive,
        close_fee_short_put: Positive,
    ) -> Result<Self, StrategyError> {
        let mut strategy = ShortPut::default();

        let short_put_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol,
            short_put_strike,
            short_put_expiration,
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
                StrategyType::ShortPut,
                "the legs built by `new` fail validation",
            ));
        }
        strategy.update_break_even_points()?;
        Ok(strategy)
    }
}

impl BasicAble for ShortPut {
    fn get_title(&self) -> String {
        let strategy_title = format!("{:?} Strategy: ", self.kind);
        let leg_titles: Vec<String> = [self.short_put.get_title()]
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
        let short_put = &self.short_put.option;

        hash_set.insert(OptionBasicType {
            option_style: &short_put.option_style,
            side: &short_put.side,
            strike_price: &short_put.strike_price,
            expiration_date: &short_put.expiration_date,
        });

        hash_set
    }
    fn get_implied_volatility(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let options = [(
            &self.short_put.option,
            &self.short_put.option.implied_volatility,
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
        let options = [(&self.short_put.option, &self.short_put.option.quantity)];

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
        self.short_put.one_option()
    }
    fn one_option_mut(&mut self) -> Result<&mut Options, StrategyError> {
        self.short_put.one_option_mut()
    }
    fn set_expiration_date(
        &mut self,
        expiration_date: ExpirationDate,
    ) -> Result<(), StrategyError> {
        self.short_put.option.expiration_date = expiration_date;
        Ok(())
    }
    fn set_underlying_price(&mut self, price: &Positive) -> Result<(), StrategyError> {
        self.short_put.option.underlying_price = *price;
        self.short_put.premium =
            Positive::new_decimal(self.short_put.option.calculate_price_black_scholes()?.abs())?;
        Ok(())
    }
    fn set_implied_volatility(&mut self, volatility: &Positive) -> Result<(), StrategyError> {
        self.short_put.option.implied_volatility = *volatility;
        self.short_put.premium =
            Positive::new_decimal(self.short_put.option.calculate_price_black_scholes()?.abs())?;
        Ok(())
    }
    fn get_contract_size(&self) -> Result<Positive, StrategyError> {
        common_contract_size(&[&self.short_put], "ShortPut::get_contract_size")
    }
    fn set_contract_size(&mut self, contract_size: Positive) -> Result<(), StrategyError> {
        apply_contract_size(
            &mut [&mut self.short_put],
            contract_size,
            "ShortPut::set_contract_size",
        )?;
        self.update_break_even_points()
    }
}

impl Validable for ShortPut {
    fn validate(&self) -> bool {
        if !self.short_put.validate() {
            debug!("Long call is invalid");
            return false;
        }
        true
    }
}

impl BreakEvenable for ShortPut {
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
            self.short_put.option.position_size()?.to_dec(),
            "ShortPut::update_break_even_points",
        )?;
        self.break_even_points.push(
            lower_break_even(self.short_put.option.strike_price, -per_contract)
                .checked_round_to(2)?,
        );

        Ok(())
    }
}

impl Strategies for ShortPut {
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        // Max profit for a short put is the net premium received (at any price ≥ strike).
        self.get_net_premium_received()
    }
    fn get_max_loss(&self) -> Result<Positive, StrategyError> {
        // Max loss for a short put occurs at price = 0: strike - premium.
        let loss = self.calculate_profit_at(&Positive::ZERO)?;
        if loss <= Decimal::ZERO {
            Ok(Positive::new_decimal(loss.abs())?)
        } else {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxLossError {
                    reason: "Max loss is negative".to_string(),
                },
            ))
        }
    }
    fn get_profit_area(&self) -> Result<Decimal, StrategyError> {
        let high = measured_max_profit(self)?;
        let break_even = self.break_even_points.first().ok_or_else(|| {
            StrategyError::empty_collection("ShortPut::get_profit_area: no break-even points")
        })?;
        let base = price_gap(self.short_put.option.strike_price, *break_even);
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

impl Profit for ShortPut {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, PricingError> {
        let price = Some(price);
        Ok(self.short_put.pnl_at_expiration(&price)?)
    }
}

impl ShortPut {
    /// Places `position` in the leg its side and style select, without
    /// refreshing the break-evens: the constructors fill the legs through
    /// it, and [`Positionable::add_position`] wraps it.
    fn place_leg(&mut self, position: &Position) -> Result<(), PositionError> {
        match (position.option.option_style, position.option.side) {
            (OptionStyle::Put, Side::Short) => {
                self.short_put = position.clone();
                Ok(())
            }
            _ => Err(PositionError::invalid_position_style(
                position.option.option_style,
                "Position is a Put or Long, it is not valid for ShortPut".to_string(),
            )),
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
            (Side::Short, OptionStyle::Put, strike)
                if *strike == self.short_put.option.strike_price =>
            {
                self.short_put = position.clone();
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

impl CachedBreakEvens for ShortPut {
    fn break_evens_mut(&mut self) -> &mut Vec<Positive> {
        &mut self.break_even_points
    }
}

impl Positionable for ShortPut {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        edit_refreshing_break_evens(self, |strategy| strategy.place_leg(position))
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.short_put])
    }

    /// Gets mutable positions matching the specified criteria from the strategy.
    ///
    /// # Arguments
    /// * `option_style` - The style of the option (Put/Put)
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
            (Side::Short, OptionStyle::Put, strike)
                if *strike == self.short_put.option.strike_price =>
            {
                Ok(vec![&mut self.short_put])
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
        edit_refreshing_break_evens(self, |strategy| strategy.replace_leg(position))
    }
}

impl StrategyConstructor for ShortPut {
    fn get_strategy(_vec_positions: &[Position]) -> Result<Self, StrategyError> {
        Err(StrategyError::operation_not_supported(
            "get_strategy",
            "ShortPut",
        ))
    }
}

impl Optimizable for ShortPut {
    type Strategy = Self;

    fn find_optimal(
        &mut self,
        _option_chain: &OptionChain,
        _side: FindOptimalSide,
        _criteria: OptimizationCriteria,
    ) -> Result<(), StrategyError> {
        Err(StrategyError::operation_not_supported(
            "find_optimal",
            "ShortPut",
        ))
    }
}

impl ProbabilityAnalysis for ShortPut {
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        // Short put is profitable when price stays above break-even
        let break_even = self.break_even_points.first().ok_or_else(|| {
            ProbabilityError::RangeError(ProfitLossRangeErrorKind::InvalidBreakEvenPoints {
                reason: "No break-even points found for short put".to_string(),
            })
        })?;

        let option = &self.short_put.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let mut profit_range = ProfitLossRange::new(Some(*break_even), None, Positive::ZERO)?;

        profit_range.calculate_probability(
            &self.short_put.option.underlying_price,
            VolatilityAdjustment {
                base_volatility: option.implied_volatility,
                std_dev_adjustment: Positive::ZERO,
            },
            None,
            expiration_date,
            Some(risk_free_rate),
        )?;

        Ok(vec![profit_range])
    }

    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        // Short put has losses when price falls below break-even
        let break_even = self.break_even_points.first().ok_or_else(|| {
            ProbabilityError::RangeError(ProfitLossRangeErrorKind::InvalidBreakEvenPoints {
                reason: "No break-even points found for short put".to_string(),
            })
        })?;

        let option = &self.short_put.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let mut loss_range = ProfitLossRange::new(None, Some(*break_even), Positive::ZERO)?;

        loss_range.calculate_probability(
            &self.short_put.option.underlying_price,
            VolatilityAdjustment {
                base_volatility: option.implied_volatility,
                std_dev_adjustment: Positive::ZERO,
            },
            None,
            expiration_date,
            Some(risk_free_rate),
        )?;

        Ok(vec![loss_range])
    }
}

impl Greeks for ShortPut {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.short_put.option])
    }
}

impl DeltaNeutrality for ShortPut {}

impl PnLCalculator for ShortPut {
    fn calculate_pnl(
        &self,
        market_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, PricingError> {
        self.short_put
            .calculate_pnl(market_price, expiration_date, implied_volatility)
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<PnL, PricingError> {
        self.short_put.calculate_pnl_at_expiration(underlying_price)
    }

    fn adjustments_pnl(&self, _adjustment: &DeltaAdjustment) -> Result<PnL, PricingError> {
        // Single-leg strategies like ShortPut don't typically require delta adjustments
        // as they are directional strategies. Delta adjustments are more relevant for
        // complex multi-leg strategies aiming for delta neutrality.
        Err(PricingError::DeltaAdjustmentNotApplicable {
            strategy: "ShortPut",
        })
    }
}

impl Strategable for ShortPut {}

#[cfg(test)]
crate::strategies::macros::test_strategy_traits!(ShortPut, test_short_put_implementations);
