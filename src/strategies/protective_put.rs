/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 12/01/26
******************************************************************************/

// Scoped allow: bulk migration of unchecked `[]` indexing to
// `.get().ok_or_else(..)` tracked as follow-ups to #341. The existing
// call sites are internal to this file and audited for invariant-bound
// indices (fixed-length buffers, just-pushed slices, etc.).
#![allow(clippy::indexing_slicing)]

//! # Protective Put Strategy
//!
//! A protective put (also known as a "married put") involves holding a long
//! position in the underlying asset and buying a put option on that same asset.
//! This strategy provides unlimited upside potential while limiting downside risk.

use super::base::{
    BreakEvenable, Optimizable, Positionable, Strategable, StrategyBasics, StrategyType, Validable,
};
use crate::Options;
use crate::error::probability::ProbabilityError;
use crate::error::{GreeksError, PositionError, PricingError, StrategyError};
use crate::greeks::Greeks;
use crate::model::ExpirationDate;
use crate::model::ProfitLossRange;
use crate::model::decimal::{d_add, d_div, d_mul, d_sub};
use crate::model::leg::traits::LegAble;
use crate::model::leg::{Leg, SpotPosition};
use crate::model::position::Position;
use crate::model::types::{OptionBasicType, OptionStyle, OptionType, Side};
use crate::pnl::PnLCalculator;
use crate::pricing::payoff::Profit;
use crate::strategies::base::price_gap;
use crate::strategies::delta_neutral::DeltaNeutrality;
use crate::strategies::probabilities::core::ProbabilityAnalysis;
use crate::strategies::probabilities::utils::VolatilityAdjustment;
use crate::strategies::{BasicAble, Strategies};
use chrono::Utc;
use positive::Positive;
use positive::PositiveError;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::debug;
use utoipa::ToSchema;

/// Default description for the Protective Put strategy.
pub const PROTECTIVE_PUT_DESCRIPTION: &str = "A protective put (married put) is a hedging strategy \
    that involves holding a long position in the underlying asset and buying a put option on that \
    same asset. This provides downside protection while maintaining unlimited upside potential.";

/// Represents a Protective Put options trading strategy.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct ProtectivePut {
    /// The name of the strategy.
    pub name: String,
    /// The type of strategy.
    pub kind: StrategyType,
    /// A textual description of this strategy instance.
    pub description: String,
    /// The price points at which the strategy breaks even.
    pub break_even_points: Vec<Positive>,
    /// The long spot position (underlying asset).
    pub spot_leg: SpotPosition,
    /// The long put option position (protective put).
    pub long_put: Position,
}

impl ProtectivePut {
    /// Creates a new Protective Put strategy.
    ///
    /// # Errors
    ///
    /// Returns `StrategyError` if the break-even calculation fails. In
    /// practice this branch is unreachable for a freshly-built protective
    /// put and is surfaced only to keep the constructor panic-free.
    #[allow(clippy::too_many_arguments)]
    #[inline(never)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        put_strike: Positive,
        expiration: ExpirationDate,
        implied_volatility: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        quantity: Positive,
        premium_long_put: Positive,
        spot_open_fee: Positive,
        spot_close_fee: Positive,
        put_open_fee: Positive,
        put_close_fee: Positive,
    ) -> Result<Self, StrategyError> {
        // Every per-share figure this strategy reports — the effective cost
        // basis, the break-even, the premium per share — divides by the size
        // of the spot leg. A protective put with no shares is not a position, and
        // rejecting it here keeps those divisors non-zero.
        if quantity == Positive::ZERO {
            return Err(StrategyError::invalid_parameters(
                "ProtectivePut::new",
                "quantity must be strictly positive: a protective put with no shares has no per-share basis",
            ));
        }

        // `protection_level` measures the strike against this price, so a
        // zero spot leaves it dividing by zero. The method stays fallible for
        // a deserialized position, which reaches the struct without passing
        // through here, but a position built by this constructor is sound.
        if underlying_price == Positive::ZERO {
            return Err(StrategyError::invalid_parameters(
                "ProtectivePut::new",
                "underlying price must be strictly positive: the protection level is measured against it",
            ));
        }

        let spot_leg = SpotPosition::new(
            underlying_symbol.clone(),
            quantity,
            underlying_price,
            Side::Long,
            Utc::now(),
            spot_open_fee,
            spot_close_fee,
        );

        let long_put_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            put_strike,
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
            put_open_fee / Positive::HUNDRED,
            put_close_fee / Positive::HUNDRED,
            None,
            None,
        );

        let mut strategy = ProtectivePut {
            name: format!("ProtectivePut_{}", underlying_symbol),
            kind: StrategyType::ProtectivePut,
            description: PROTECTIVE_PUT_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            spot_leg,
            long_put,
        };

        strategy.validate();
        strategy.update_break_even_points()?;
        Ok(strategy)
    }

    /// Returns the spot leg as a Leg enum.
    #[must_use]
    pub fn get_spot_leg(&self) -> Leg {
        Leg::Spot(self.spot_leg.clone())
    }

    /// Returns the long put leg as a Leg enum.
    #[must_use]
    pub fn get_put_leg(&self) -> Leg {
        Leg::option(self.long_put.clone())
    }

    /// Returns all legs of the strategy.
    #[must_use]
    pub fn get_legs(&self) -> Vec<Leg> {
        vec![self.get_spot_leg(), self.get_put_leg()]
    }

    /// Returns the put strike price.
    #[must_use]
    pub fn put_strike(&self) -> Positive {
        self.long_put.option.strike_price
    }

    /// Returns the underlying price (cost basis).
    #[must_use]
    pub fn underlying_price(&self) -> Positive {
        self.spot_leg.cost_basis
    }

    /// Returns the quantity of shares.
    #[must_use]
    pub fn quantity(&self) -> Positive {
        self.spot_leg.quantity
    }

    /// Calculates the net delta of the strategy.
    ///
    /// # Errors
    ///
    /// Propagates any [`GreeksError`] returned by
    /// [`LegAble::delta`] on the spot leg or the long-put leg.
    pub fn net_delta(&self) -> Result<Decimal, GreeksError> {
        let spot_delta = self.spot_leg.delta()?;
        let put_delta = self.long_put.delta()?;
        Ok(d_add(spot_delta, put_delta, "ProtectivePut::net_delta")?)
    }

    /// Calculates the maximum loss potential.
    ///
    /// # Errors
    ///
    /// Currently infallible — both branches compute
    /// `Positive::new_decimal(total_loss.max(Decimal::ZERO))
    /// .unwrap_or(Positive::ZERO)`, so any negative decomposition is
    /// clamped to `Positive::ZERO` rather than surfaced as an error.
    /// The `Result` signature is retained so future implementations
    /// that add checked arithmetic or validate the strike layout can
    /// return `PricingError::MethodError` without a breaking change.
    pub fn max_loss_potential(&self) -> Result<Positive, PricingError> {
        let put_strike = self.put_strike();
        let cost_basis = self.spot_leg.cost_basis;
        let quantity = self.spot_leg.quantity;
        let put_premium = self
            .long_put
            .premium
            .checked_mul(&self.long_put.option.quantity)?;
        let total_fees = self.total_fees()?;

        if cost_basis >= put_strike {
            let capital_loss = price_gap(cost_basis, put_strike).checked_mul(&quantity)?;
            let total_loss = d_add(
                d_add(
                    capital_loss.to_dec(),
                    put_premium.to_dec(),
                    "ProtectivePut::max_loss",
                )?,
                total_fees.to_dec(),
                "ProtectivePut::max_loss",
            )?;
            Ok(Positive::new_decimal(total_loss.max(Decimal::ZERO)).unwrap_or(Positive::ZERO))
        } else {
            let capital_gain = price_gap(put_strike, cost_basis).checked_mul(&quantity)?;
            let total_loss = d_sub(
                d_add(
                    put_premium.to_dec(),
                    total_fees.to_dec(),
                    "ProtectivePut::max_loss",
                )?,
                capital_gain.to_dec(),
                "ProtectivePut::max_loss",
            )?;
            Ok(Positive::new_decimal(total_loss.max(Decimal::ZERO)).unwrap_or(Positive::ZERO))
        }
    }

    /// Calculates total fees for all positions.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError`] when the running total leaves the `Positive`
    /// range. A fee at `Positive::MAX` used to abort inside the raw sum.
    pub fn total_fees(&self) -> Result<Positive, PositiveError> {
        let put_fees = self
            .long_put
            .open_fee
            .checked_add(&self.long_put.close_fee)?
            .checked_mul(&self.long_put.option.quantity)?;
        self.spot_leg
            .open_fee
            .checked_add(&self.spot_leg.close_fee)?
            .checked_add(&put_fees)
    }

    /// Returns the protection level as a percentage below current price.
    ///
    /// # Errors
    ///
    /// Returns [`StrategyError`] when the spot leg's cost basis is zero, which
    /// [`ProtectivePut::new`] rejects but a deserialized position can still
    /// carry, and when the difference or the ratio leaves the representable
    /// `Decimal` range.
    pub fn protection_level(&self) -> Result<Decimal, StrategyError> {
        let current_price = self.spot_leg.cost_basis.to_dec();
        let put_strike = self.long_put.option.strike_price.to_dec();
        let below = d_sub(
            current_price,
            put_strike,
            "ProtectivePut::protection_level/below",
        )?;
        let ratio = d_div(
            below,
            current_price,
            "ProtectivePut::protection_level/ratio",
        )?;
        d_mul(
            ratio,
            Decimal::ONE_HUNDRED,
            "ProtectivePut::protection_level",
        )
        .map_err(Into::into)
    }

    /// Calculates the effective cost basis (Price paid for spot + Premium paid for put).
    ///
    /// # Decision (issue #471): return `Result`
    ///
    /// The divisor is `spot_leg.quantity`, a `pub` field.
    /// [`ProtectivePut::new`] rejects a zero share count, but a
    /// `ProtectivePut` deserialized from JSON or mutated in place can carry
    /// one, and `Positive` has no value that means "undefined per-share
    /// figure". The product above the division and the sum below it can also
    /// leave the `Positive` range on their own.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the spot leg holds
    /// zero shares, so there is no per-share premium to divide out, and when
    /// `premium × quantity` or the final sum overflows.
    pub fn effective_cost_basis(&self) -> Result<Positive, PositiveError> {
        let premium_per_share = self
            .long_put
            .premium
            .checked_mul(&self.long_put.option.quantity)?
            .checked_div(&self.spot_leg.quantity)?;
        self.spot_leg.cost_basis.checked_add(&premium_per_share)
    }

    /// Checks if the put is out-of-the-money.
    #[must_use]
    pub fn is_put_otm(&self) -> bool {
        self.spot_leg.cost_basis > self.long_put.option.strike_price
    }
}

impl Validable for ProtectivePut {
    fn validate(&self) -> bool {
        if !self.long_put.validate() {
            debug!("Long put validation failed");
            return false;
        }
        if self.long_put.option.option_style != OptionStyle::Put {
            debug!("Long put must be a put option");
            return false;
        }
        if self.long_put.option.side != Side::Long {
            debug!("Long put must be a long position");
            return false;
        }
        if self.spot_leg.side != Side::Long {
            debug!("Spot leg must be a long position");
            return false;
        }
        true
    }
}

impl BreakEvenable for ProtectivePut {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points.clear();
        let entry_price = self.spot_leg.cost_basis.to_dec();
        let put_premium = self.long_put.premium.to_dec();
        let quantity = self.spot_leg.quantity.to_dec();
        let total_fees = self.total_fees()?;
        let break_even = d_add(
            d_add(entry_price, put_premium, "ProtectivePut::break_even")?,
            d_div(
                total_fees.to_dec(),
                quantity,
                "ProtectivePut::break_even/fees_per_share",
            )?,
            "ProtectivePut::break_even",
        )?;
        if let Ok(be) = Positive::new_decimal(break_even) {
            self.break_even_points.push(be.checked_round_to(2)?);
        }
        Ok(())
    }
}

impl Positionable for ProtectivePut {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        if position.option.option_style != OptionStyle::Put {
            return Err(PositionError::invalid_position_type(
                position.option.side,
                "Position must be a put option".to_string(),
            ));
        }
        self.long_put = position.clone();
        let _ = self.update_break_even_points();
        Ok(())
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.long_put])
    }

    fn get_position(
        &mut self,
        option_style: &OptionStyle,
        side: &Side,
        strike: &Positive,
    ) -> Result<Vec<&mut Position>, PositionError> {
        if *option_style == OptionStyle::Put
            && *side == Side::Long
            && *strike == self.long_put.option.strike_price
        {
            Ok(vec![&mut self.long_put])
        } else {
            Err(PositionError::invalid_position(
                "Position not found in ProtectivePut",
            ))
        }
    }

    fn modify_position(&mut self, position: &Position) -> Result<(), PositionError> {
        if position.option.option_style != OptionStyle::Put || position.option.side != Side::Long {
            return Err(PositionError::invalid_position_type(
                position.option.side,
                "ProtectivePut only accepts long put positions".to_string(),
            ));
        }

        self.long_put = position.clone();
        let _ = self.update_break_even_points();
        Ok(())
    }
}

impl Strategable for ProtectivePut {
    fn info(&self) -> Result<StrategyBasics, StrategyError> {
        Ok(StrategyBasics {
            name: self.name.clone(),
            kind: self.kind.clone(),
            description: self.description.clone(),
        })
    }
}

impl BasicAble for ProtectivePut {
    // Without this override the trait default aborts the process, and it is
    // reached from `get_underlying_price`, `get_max_min_strikes`,
    // `delta_neutrality` and every probability method. The long put carries the
    // same underlying, expiration and rate as the spot leg, so it answers for
    // the strategy.
    fn one_option(&self) -> &Options {
        self.long_put.one_option()
    }

    fn one_option_mut(&mut self) -> &mut Options {
        self.long_put.one_option_mut()
    }

    fn get_title(&self) -> String {
        format!(
            "Protective Put Strategy:\n\t{} {} {} @ {}\n\t{}",
            self.spot_leg.side,
            self.spot_leg.quantity,
            self.spot_leg.symbol,
            self.spot_leg.cost_basis,
            self.long_put.get_title()
        )
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
        let mut map = HashMap::new();
        let long_put = &self.long_put.option;
        map.insert(
            OptionBasicType {
                option_style: &long_put.option_style,
                side: &long_put.side,
                strike_price: &long_put.strike_price,
                expiration_date: &long_put.expiration_date,
            },
            &long_put.implied_volatility,
        );
        map
    }

    fn get_quantity(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let mut map = HashMap::new();
        let long_put = &self.long_put.option;
        map.insert(
            OptionBasicType {
                option_style: &long_put.option_style,
                side: &long_put.side,
                strike_price: &long_put.strike_price,
                expiration_date: &long_put.expiration_date,
            },
            &long_put.quantity,
        );
        map
    }
}

impl Strategies for ProtectivePut {
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        Ok(Positive::new_decimal(Decimal::MAX).unwrap_or(Positive::ZERO))
    }

    fn get_max_loss(&self) -> Result<Positive, StrategyError> {
        self.max_loss_potential().map_err(StrategyError::from)
    }
}

impl Profit for ProtectivePut {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, PricingError> {
        let spot_pnl = self.spot_leg.pnl_at_price(*price)?;
        let put_pnl = self
            .long_put
            .pnl_at_expiration(&Some(price))
            .unwrap_or(Decimal::ZERO);
        Ok(d_add(
            spot_pnl,
            put_pnl,
            "ProtectivePut::pnl_at_expiration",
        )?)
    }
}

impl Greeks for ProtectivePut {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.long_put.option])
    }

    fn delta(&self) -> Result<Decimal, GreeksError> {
        self.net_delta()
    }
}

impl PnLCalculator for ProtectivePut {
    fn calculate_pnl(
        &self,
        underlying_price: &Positive,
        _expiration_date: ExpirationDate,
        _implied_volatility: &Positive,
    ) -> Result<crate::pnl::utils::PnL, PricingError> {
        self.calculate_pnl_at_expiration(underlying_price)
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<crate::pnl::utils::PnL, PricingError> {
        let profit = self.calculate_profit_at(underlying_price)?;
        let spot_cost = self.spot_leg.total_cost()?;
        let put_cost = self
            .long_put
            .premium
            .checked_mul(&self.long_put.option.quantity)?;
        Ok(crate::pnl::utils::PnL {
            realized: None,
            unrealized: Some(profit),
            initial_costs: spot_cost.checked_add(&put_cost)?,
            initial_income: Positive::ZERO,
            date_time: Utc::now(),
        })
    }
}

impl DeltaNeutrality for ProtectivePut {}

impl Optimizable for ProtectivePut {
    type Strategy = ProtectivePut;
}

impl crate::strategies::StrategyConstructor for ProtectivePut {}

impl ProbabilityAnalysis for ProtectivePut {
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_point =
            self.break_even_points
                .first()
                .copied()
                .ok_or(ProbabilityError::MissingMetric {
                    metric: "break_even_point",
                })?;
        let option = &self.long_put.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;
        let mut profit_range = ProfitLossRange::new(Some(break_even_point), None, Positive::ZERO)?;
        profit_range.calculate_probability(
            &self.spot_leg.cost_basis,
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
        let break_even_point =
            self.break_even_points
                .first()
                .copied()
                .ok_or(ProbabilityError::MissingMetric {
                    metric: "break_even_point",
                })?;
        let option = &self.long_put.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;
        let mut loss_range = ProfitLossRange::new(
            Some(self.put_strike()),
            Some(break_even_point),
            Positive::ZERO,
        )?;
        loss_range.calculate_probability(
            &self.spot_leg.cost_basis,
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

impl std::fmt::Display for ProtectivePut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Protective Put Strategy")?;
        writeln!(f, "======================")?;
        writeln!(f, "Symbol: {}", self.spot_leg.symbol)?;
        writeln!(f, "Underlying Price: ${:.2}", self.spot_leg.cost_basis)?;
        writeln!(f, "Put Strike: ${:.2}", self.long_put.option.strike_price)?;
        writeln!(f, "Put Premium: ${:.2}", self.long_put.premium)?;
        writeln!(f, "Quantity: {}", self.spot_leg.quantity)?;
        writeln!(f, "Expiration: {}", self.long_put.option.expiration_date)?;
        match self.protection_level() {
            Ok(level) => writeln!(f, "Protection Level: {level:.2}%")?,
            Err(_) => writeln!(f, "Protection Level: n/a")?,
        }
        if let Ok(break_evens) = self.get_break_even_points() {
            writeln!(f, "Break-even: ${:.2}", break_evens[0])?;
        }
        if let Ok(max_loss) = self.max_loss_potential() {
            writeln!(f, "Max Loss: ${:.2}", max_loss)?;
        }
        writeln!(f, "Max Profit: Unlimited")?;
        if let Ok(delta) = self.net_delta() {
            writeln!(f, "Net Delta: {:.4}", delta)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    fn create_test_protective_put() -> ProtectivePut {
        ProtectivePut::new(
            "AAPL".to_string(),
            pos_or_panic!(150.0),
            pos_or_panic!(145.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.25),
            dec!(0.05),
            pos_or_panic!(0.01),
            Positive::HUNDRED,
            pos_or_panic!(3.50),
            Positive::ONE,
            Positive::ONE,
            pos_or_panic!(0.65),
            pos_or_panic!(0.65),
        )
        .unwrap()
    }

    #[test]
    fn test_new_protective_put() {
        let pp = create_test_protective_put();
        assert_eq!(pp.spot_leg.symbol, "AAPL");
        assert_eq!(pp.spot_leg.cost_basis, pos_or_panic!(150.0));
        assert_eq!(pp.long_put.option.strike_price, pos_or_panic!(145.0));
    }

    /// `effective_cost_basis` divides by `spot_leg.quantity`, a `pub` field.
    /// `ProtectivePut::new` rejects a zero share count, but a protective put
    /// mutated in place or deserialized from JSON can carry one, and there is
    /// no per-share cost basis to report for it.
    #[test]
    fn test_protective_put_effective_cost_basis_zero_shares_is_reported() {
        let mut pp = create_test_protective_put();
        pp.spot_leg.quantity = Positive::ZERO;

        assert!(pp.effective_cost_basis().is_err());
    }

    /// The premium leg is `pub` too, so the numerator can leave the
    /// `Positive` range before the division is reached.
    #[test]
    fn test_protective_put_effective_cost_basis_overflowing_premium_is_reported() {
        let mut pp = create_test_protective_put();
        pp.long_put.premium = Positive::MAX;
        pp.long_put.option.quantity = Positive::TWO;

        assert!(pp.effective_cost_basis().is_err());
    }

    #[test]
    fn test_underlying_price() {
        let pp = create_test_protective_put();
        assert_eq!(pp.underlying_price(), pos_or_panic!(150.0));
    }

    #[test]
    fn test_put_strike() {
        let pp = create_test_protective_put();
        assert_eq!(pp.put_strike(), pos_or_panic!(145.0));
    }

    #[test]
    fn test_quantity() {
        let pp = create_test_protective_put();
        assert_eq!(pp.quantity(), Positive::HUNDRED);
    }

    #[test]
    fn test_break_even_points() {
        let pp = create_test_protective_put();
        let break_evens = pp.get_break_even_points().unwrap();
        assert_eq!(break_evens.len(), 1);
        assert!(break_evens[0] > pp.spot_leg.cost_basis);
    }

    #[test]
    fn test_max_loss() {
        let pp = create_test_protective_put();
        let max_loss = pp.max_loss_potential().unwrap();
        assert!(max_loss > Positive::ZERO);
    }

    #[test]
    fn test_validate() {
        let pp = create_test_protective_put();
        assert!(pp.validate());
    }

    #[test]
    fn test_profit_at_high_price() {
        let pp = create_test_protective_put();
        let profit = pp.calculate_profit_at(&pos_or_panic!(200.0)).unwrap();
        assert!(profit > Decimal::ZERO);
    }

    #[test]
    fn test_is_put_otm() {
        let pp = create_test_protective_put();
        assert!(pp.is_put_otm());
    }

    #[test]
    fn test_protection_level() {
        let pp = create_test_protective_put();
        let protection = pp.protection_level().unwrap();
        assert!(protection > Decimal::ZERO);
    }

    #[test]
    fn test_get_legs() {
        let pp = create_test_protective_put();
        let legs = pp.get_legs();
        assert_eq!(legs.len(), 2);
        assert!(legs[0].is_spot());
        assert!(legs[1].is_option());
    }

    #[test]
    fn test_total_fees() {
        let pp = create_test_protective_put();
        let fees = pp.total_fees().unwrap();
        assert!(fees > Positive::ZERO);
    }

    #[test]
    fn test_display() {
        let pp = create_test_protective_put();
        let display = format!("{}", pp);
        assert!(display.contains("Protective Put Strategy"));
        assert!(display.contains("AAPL"));
    }

    #[test]
    fn test_get_title() {
        let pp = create_test_protective_put();
        let title = pp.get_title();
        assert!(title.contains("Protective Put"));
        assert!(title.contains("AAPL"));
    }

    #[test]
    fn test_strategy_type() {
        let pp = create_test_protective_put();
        assert_eq!(pp.kind, StrategyType::ProtectivePut);
    }

    #[test]
    fn test_positions() {
        let pp = create_test_protective_put();
        let positions = Positionable::get_positions(&pp).unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.option_style, OptionStyle::Put);
    }
}
