/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Covered Call Strategy
//!
//! A covered call involves holding a long position in the underlying asset and
//! selling a call option on that same asset. This strategy provides limited upside
//! potential but offers some downside protection in the form of the premium received
//! for selling the call option.
//!
//! ## Key Characteristics
//!
//! - Limited profit potential (capped at strike price + premium received)
//! - Provides some downside protection via premium received
//! - Reduces the cost basis of the underlying asset
//! - Ideal for neutral to slightly bullish outlook
//!
//! ## Components
//!
//! - **Long Spot Position**: Ownership of the underlying asset
//! - **Short Call Option**: Sold call option at a strike above current price
//!
//! ## Example
//!
//! ```rust
//! use optionstratlib_strategies::strategies::covered_call::CoveredCall;
//! use optionstratlib_core::model::ExpirationDate;
//! use optionstratlib_core::pos_or_panic;
//! use optionstratlib_core::model::Positive;
//! use rust_decimal_macros::dec;
//!
//! let covered_call = CoveredCall::new(
//!     "AAPL".to_string(),
//!     pos_or_panic!(150.0),    // underlying price
//!     pos_or_panic!(155.0),    // call strike
//!     ExpirationDate::Days(pos_or_panic!(30.0)),
//!     pos_or_panic!(0.25),     // implied volatility
//!     dec!(0.05),     // risk-free rate
//!     pos_or_panic!(0.01),     // dividend yield
//!     Positive::HUNDRED,    // quantity (shares)
//!     pos_or_panic!(3.50),     // call premium received
//!     Positive::ONE,      // spot open fee
//!     Positive::ONE,      // spot close fee
//!     pos_or_panic!(0.65),     // call open fee
//!     pos_or_panic!(0.65),     // call close fee
//! );
//! ```

use super::base::{
    BreakEvenable, Optimizable, Positionable, Strategable, StrategyBasics, StrategyType, Validable,
};
use crate::error::StrategyError;
use crate::strategies::base::{lower_break_even, price_gap};
use crate::strategies::delta_neutral::DeltaNeutrality;
use crate::strategies::probabilities::core::ProbabilityAnalysis;
use crate::strategies::shared::spot_leg_mark_to_market;
use crate::strategies::shared::{
    apply_hedge_contract_size, common_contract_size, expiry_zones, price_zones,
};
use crate::strategies::{BasicAble, Strategies};
use chrono::Utc;
use optionstratlib_analytics::analytics::ProfitLossRange;
use optionstratlib_analytics::analytics::probability::VolatilityAdjustment;
use optionstratlib_analytics::error::probability::ProbabilityError;
use optionstratlib_analytics::pnl::PnLCalculator;
use optionstratlib_core::error::PositionError;
use optionstratlib_core::error::position::PositionValidationErrorKind;
use optionstratlib_core::model::ExpirationDate;
use optionstratlib_core::model::Options;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::PositiveError;
use optionstratlib_core::model::decimal::{d_add, d_div, d_mul, d_sub};
use optionstratlib_core::model::leg::traits::LegAble;
use optionstratlib_core::model::leg::{Leg, SpotPosition};
use optionstratlib_core::model::position::Position;
use optionstratlib_core::model::types::{OptionBasicType, OptionStyle, OptionType, Side};
use optionstratlib_core::model::utils::sub_floor_zero;
use optionstratlib_pricing::error::GreeksError;
use optionstratlib_pricing::error::PricingError;
use optionstratlib_pricing::greeks::Greeks;
use optionstratlib_pricing::greeks::LegGreeks;
use optionstratlib_pricing::pricing::Profit;
use rust_decimal::{Decimal, RoundingStrategy};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::debug;

/// Default description for the Covered Call strategy.
pub const COVERED_CALL_DESCRIPTION: &str = "A covered call is created by holding a long position \
    in the underlying asset and selling a call option on that same asset. This strategy provides \
    limited upside potential (capped at the strike price plus premium received) but offers some \
    downside protection through the premium received. It is ideal for investors with a neutral \
    to slightly bullish outlook who want to generate income from their holdings.";

/// Represents a Covered Call options trading strategy.
///
/// A Covered Call combines a long position in the underlying asset with a short
/// call option. This strategy is used to generate income from existing holdings
/// while accepting limited upside potential.
///
/// # Structure
///
/// - **Spot Leg**: Long position in the underlying asset
/// - **Option Leg**: Short call option at a strike price above current price
///
/// # Profit/Loss Profile
///
/// - **Maximum Profit**: (Strike Price - Cost Basis) + Premium Received
/// - **Maximum Loss**: Cost Basis - Premium Received (if underlying goes to zero)
/// - **Break-even**: Cost Basis - Premium Received per share
///
/// These hold when the call covers the shares one for one (`quantity ×
/// contract_size` equal to the spot quantity), as `CoveredCall::new` and
/// `BasicAble::set_contract_size` build it. A call resized to cover fewer or
/// more units changes the slope above its strike: an under-covered call
/// leaves the uncovered shares' upside, so the profit is unbounded, and an
/// over-covered call is net short above the strike, so the loss is. The
/// break-evens, max profit and max loss and the profit and loss zones then
/// follow the expiry P&L for that cover.
///
/// # Greeks
///
/// - **Delta**: Positive (long spot delta + short call delta)
/// - **Gamma**: Negative (from short call)
/// - **Theta**: Positive (benefits from time decay of short call)
/// - **Vega**: Negative (benefits from volatility decrease)
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
pub struct CoveredCall {
    /// The name of the strategy.
    pub name: String,

    /// The type of strategy (StrategyType::CoveredCall).
    pub kind: StrategyType,

    /// A textual description of this strategy instance.
    pub description: String,

    /// The price points at which the strategy breaks even.
    pub break_even_points: Vec<Positive>,

    /// The long spot position (underlying asset).
    pub spot_leg: SpotPosition,

    /// The short call option position.
    pub short_call: Position,
}

impl CoveredCall {
    /// Creates a new Covered Call strategy.
    ///
    /// # Arguments
    ///
    /// * `underlying_symbol` - The ticker symbol of the underlying asset
    /// * `underlying_price` - The current market price of the underlying asset
    /// * `call_strike` - The strike price for the short call option
    /// * `expiration` - The expiration date for the call option
    /// * `implied_volatility` - The implied volatility for option pricing
    /// * `risk_free_rate` - The risk-free interest rate
    /// * `dividend_yield` - The dividend yield of the underlying asset
    /// * `quantity` - The number of shares. The option leg covers the same
    ///   shares one for one: it is built as one-unit contracts, one per
    ///   share (#731). `BasicAble::set_contract_size` re-expresses it in
    ///   market contracts with the same payoff and fees.
    /// * `premium_short_call` - The premium received for selling the call
    /// * `spot_open_fee` - Fee to open the spot position
    /// * `spot_close_fee` - Fee to close the spot position
    /// * `call_open_fee` - Fee to open the call position, per share
    /// * `call_close_fee` - Fee to close the call position, per share
    ///
    /// # Returns
    ///
    /// A fully configured `CoveredCall` strategy instance.
    ///
    /// # Errors
    ///
    /// Returns `StrategyError::InvalidStrategy` when the assembled strategy
    /// fails its own `validate` (#696): the legs fail `CoveredCall::validate`.
    ///
    /// Returns `StrategyError` if the break-even calculation fails. In
    /// practice this branch is unreachable for a freshly-built covered
    /// call and is surfaced only to keep the constructor panic-free.
    #[allow(clippy::too_many_arguments)]
    #[inline(never)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        call_strike: Positive,
        expiration: ExpirationDate,
        implied_volatility: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        quantity: Positive,
        premium_short_call: Positive,
        spot_open_fee: Positive,
        spot_close_fee: Positive,
        call_open_fee: Positive,
        call_close_fee: Positive,
    ) -> Result<Self, StrategyError> {
        // Create the spot position (long underlying)
        // Every per-share figure this strategy reports — the effective cost
        // basis, the break-even, the premium per share — divides by the size
        // of the spot leg. A covered call with no shares is not a position, and
        // rejecting it here keeps those divisors non-zero.
        if quantity == Positive::ZERO {
            return Err(StrategyError::invalid_parameters(
                "CoveredCall::new",
                "quantity must be strictly positive: a covered call with no shares has no per-share basis",
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

        // Create the short call option
        let short_call_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol,
            call_strike,
            expiration,
            implied_volatility,
            quantity, // One option unit per share (#731)
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
            call_open_fee,
            call_close_fee,
            None,
            None,
        );

        let mut strategy = CoveredCall {
            name: "Covered Call".to_string(),
            kind: StrategyType::CoveredCall,
            description: COVERED_CALL_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            spot_leg,
            short_call,
        };

        if !strategy.validate() {
            return Err(StrategyError::invalid_strategy(
                StrategyType::CoveredCall,
                "the legs built by `new` fail validation",
            ));
        }
        strategy.update_break_even_points()?;

        Ok(strategy)
    }

    /// Returns the spot leg as a `Leg` enum.
    #[must_use]
    pub fn get_spot_leg(&self) -> Leg {
        Leg::Spot(self.spot_leg.clone())
    }

    /// Returns the option leg as a `Leg` enum.
    #[must_use]
    pub fn get_option_leg(&self) -> Leg {
        Leg::option(self.short_call.clone())
    }

    /// Returns all legs of the strategy.
    #[must_use]
    pub fn get_legs(&self) -> Vec<Leg> {
        vec![self.get_spot_leg(), self.get_option_leg()]
    }

    /// Returns the call strike price.
    #[must_use]
    pub fn call_strike(&self) -> Positive {
        self.short_call.option.strike_price
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

    /// Calculates the net delta of the covered call.
    ///
    /// Net Delta = Spot Delta + Option Delta
    /// For a covered call: typically positive but less than 1.0 per share
    ///
    /// # Errors
    ///
    /// Propagates any [`GreeksError`] returned by
    /// [`LegGreeks::delta`] on the spot leg or the short-call leg.
    pub fn net_delta(&self) -> Result<Decimal, GreeksError> {
        let spot_delta = self.spot_leg.delta()?;
        let option_delta = self.short_call.delta()?;
        Ok(d_add(spot_delta, option_delta, "CoveredCall::net_delta")?)
    }

    /// Calculates the effective cost basis after receiving premium.
    ///
    /// Effective Cost Basis = Original Cost Basis - Premium Received per Share
    ///
    /// A credit larger than the cost basis leaves nothing at risk per share,
    /// and the function keeps its existing answer of `Positive::ZERO` for
    /// that case rather than turning it into an error.
    ///
    /// # Decision (issue #471): return `Result`
    ///
    /// The divisor is `spot_leg.quantity`, a `pub` field. [`CoveredCall::new`]
    /// rejects a zero share count, but a `CoveredCall` deserialized from JSON
    /// or mutated in place can carry one, and `Positive` has no value that
    /// means "undefined per-share figure". The product above the division can
    /// also leave the `Positive` range on its own.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the spot leg holds
    /// zero shares, so there is no per-share premium to divide out, and when
    /// `premium × contract_size × quantity` overflows.
    pub fn effective_cost_basis(&self) -> Result<Positive, PositiveError> {
        let premium_per_share = self
            .short_call
            .premium
            .checked_mul(&self.short_call.option.contract_size)?
            .checked_mul(&self.short_call.option.quantity)?
            // A per-share figure rarely divides exactly — one option unit of
            // premium over three shares repeats — so the rounding is chosen
            // here rather than taken from the dependency's default.
            // `MidpointNearestEven` is what `d_div` already applies to every
            // `Decimal` division in this crate, so a per-share value rounds
            // the same way whichever path computes it.
            .checked_div_with_strategy(
                &self.spot_leg.quantity,
                RoundingStrategy::MidpointNearestEven,
            )?;
        if self.spot_leg.cost_basis > premium_per_share {
            self.spot_leg.cost_basis.checked_sub(&premium_per_share)
        } else {
            Ok(Positive::ZERO)
        }
    }

    /// Total fees: the share leg's open and close fees, plus the call's,
    /// which are per share and scale with its quantity as in
    /// `Position::fees`. This is what the payoff charges (#731).
    ///
    /// # Errors
    ///
    /// Returns [`PositionError`] when a sum or the call's product leaves the
    /// `Positive` range.
    pub fn total_fees(&self) -> Result<Positive, PositionError> {
        Ok(self
            .spot_leg
            .open_fee
            .checked_add(&self.spot_leg.close_fee)?
            .checked_add(&self.short_call.fees()?)?)
    }

    /// Calculates the maximum profit potential, floored at zero.
    ///
    /// Max Profit = (Strike - Cost Basis) × Quantity + Premium Received - Fees
    /// when the call covers the shares exactly, so it caps the gain at its
    /// strike.
    ///
    /// A call of `Q` units against `N` shares has slope `N - Q` above its
    /// strike. A call that covers fewer units than the shares leaves the
    /// profit unbounded, which is reported as `Positive::MAX`. One that
    /// covers more turns the P&L down above the strike, so it peaks at the
    /// strike (or at zero), and the larger of the two is reported.
    ///
    /// # Errors
    ///
    /// Returns [`PricingError`] when the call's size, premium or fees, or the
    /// expiry P&L at zero or at the strike, leaves the `Positive` or
    /// `Decimal` range.
    pub fn max_profit_potential(&self) -> Result<Positive, PricingError> {
        let strike = self.call_strike();
        let cost_basis = self.spot_leg.cost_basis;
        let quantity = self.spot_leg.quantity;
        // Checked: a premium and a quantity near the top of the
        // `Positive` range overflow, and the struct's public fields let such a
        // leg in without passing `new` (#696). Checked before the cover is
        // classified, so an unpriceable call is an error, not an unbounded
        // figure.
        let premium_received = self.premium_received()?;
        let total_fees = self.total_fees()?;
        let call_units = self.call_units()?;
        if call_units != quantity {
            if call_units < quantity {
                return Ok(Positive::MAX);
            }
            let best = self
                .profit_at_kinks()?
                .into_iter()
                .fold(Decimal::MIN, Decimal::max);
            return Ok(Positive::new_decimal(best.max(Decimal::ZERO)).unwrap_or(Positive::ZERO));
        }

        if strike >= cost_basis {
            let capital_gain = price_gap(strike, cost_basis).checked_mul(&quantity)?;
            // Fees larger than the gain plus the credit make the best case a
            // loss, which this `Positive` return reports as zero — the same
            // answer the branch below already gives.
            Ok(sub_floor_zero(
                capital_gain.checked_add(&premium_received)?,
                total_fees.to_dec_ref(),
            ))
        } else {
            // Strike below cost basis - max profit is just premium minus loss
            let capital_loss = price_gap(cost_basis, strike).checked_mul(&quantity)?;
            let downside = capital_loss.checked_add(&total_fees)?;
            if premium_received > downside {
                Ok(premium_received.checked_sub(&downside)?)
            } else {
                Ok(Positive::ZERO)
            }
        }
    }

    /// Calculates the maximum loss potential, floored at zero.
    ///
    /// Max Loss = Cost Basis × Quantity - Premium Received + Fees
    /// (occurs if underlying goes to zero) when the call covers the shares
    /// exactly.
    ///
    /// A call that covers more units than the shares is net short above its
    /// strike and its loss is unbounded, which is reported as
    /// `Positive::MAX`. One that covers fewer keeps a rising P&L on both
    /// sides of the strike, so the loss bottoms out at zero; the deeper of
    /// the P&L at zero and at the strike is reported.
    ///
    /// # Errors
    ///
    /// Returns [`PricingError`] when the call's size, premium or fees, the
    /// cost of the shares, or the expiry P&L at zero or at the strike, leaves
    /// the `Positive` or `Decimal` range.
    pub fn max_loss_potential(&self) -> Result<Positive, PricingError> {
        let cost_basis = self.spot_leg.cost_basis;
        let quantity = self.spot_leg.quantity;
        // Checked before the cover is classified; see `max_profit_potential`.
        let premium_received = self.premium_received()?;
        let total_fees = self.total_fees()?;
        let call_units = self.call_units()?;
        if call_units != quantity {
            if call_units > quantity {
                return Ok(Positive::MAX);
            }
            let worst = self
                .profit_at_kinks()?
                .into_iter()
                .fold(Decimal::MAX, Decimal::min);
            // `Decimal` is symmetric, so negating a representable value is
            // itself representable.
            return Ok(Positive::new_decimal((-worst).max(Decimal::ZERO)).unwrap_or(Positive::ZERO));
        }

        let total_investment = cost_basis.checked_mul(&quantity)?;
        let gross_outlay = total_investment.checked_add(&total_fees)?;
        if gross_outlay > premium_received {
            Ok(gross_outlay.checked_sub(&premium_received)?)
        } else {
            Ok(Positive::ZERO)
        }
    }

    /// The premium received for the call, `premium × contract_size ×
    /// quantity`.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError`] when the product leaves the `Positive`
    /// range.
    fn premium_received(&self) -> Result<Positive, PositiveError> {
        self.short_call
            .premium
            .checked_mul(&self.short_call.option.contract_size)?
            .checked_mul(&self.short_call.option.quantity)
    }

    /// The units of the underlying the call covers, `quantity ×
    /// contract_size`.
    ///
    /// # Errors
    ///
    /// Returns [`PricingError`] when the call's size leaves the `Positive`
    /// range.
    fn call_units(&self) -> Result<Positive, PricingError> {
        Ok(self.short_call.option.position_size()?)
    }

    /// Whether the call covers the shares exactly.
    ///
    /// # Errors
    ///
    /// Propagates [`CoveredCall::call_units`].
    fn is_exact_cover(&self) -> Result<bool, PricingError> {
        Ok(self.call_units()? == self.spot_leg.quantity)
    }

    /// The expiry P&L at the points where its slope can change or its
    /// extreme can sit: zero and the call strike.
    ///
    /// # Errors
    ///
    /// Propagates [`Profit::calculate_profit_at`].
    fn profit_at_kinks(&self) -> Result<[Decimal; 2], PricingError> {
        Ok([
            self.calculate_profit_at(&Positive::ZERO)?,
            self.calculate_profit_at(&self.call_strike())?,
        ])
    }

    /// The profit and loss zones of a call that does not cover the shares
    /// exactly, cut at the break-evens.
    ///
    /// # Errors
    ///
    /// Propagates [`expiry_zones`].
    fn mismatched_cover_zones(
        &self,
    ) -> Result<(Vec<ProfitLossRange>, Vec<ProfitLossRange>), ProbabilityError> {
        expiry_zones(&self.break_even_points, self.spot_leg.cost_basis, |price| {
            self.calculate_profit_at(price)
        })
    }

    /// The break-evens of a call that does not cover the shares exactly; see
    /// [`BreakEvenable::update_break_even_points`].
    ///
    /// # Errors
    ///
    /// Returns [`StrategyError`] when the call's size, the premium, the fees
    /// or a step of the arithmetic leaves the `Positive` or `Decimal` range.
    fn update_mismatched_break_even_points(&mut self) -> Result<(), StrategyError> {
        let shares = self.spot_leg.quantity.to_dec();
        let entry_price = self.spot_leg.cost_basis.to_dec();
        let call_units = self.call_units()?.to_dec();
        let strike = self.call_strike().to_dec();
        // P - F: what the call and the fees add to the shares' P&L.
        let carry = d_sub(
            self.premium_received()?.to_dec(),
            self.total_fees()?.to_dec(),
            "CoveredCall::break_even/carry",
        )?;

        let mut zeros = Vec::with_capacity(2);
        // C - (P - F) / N, at or below the strike.
        let below = d_sub(
            entry_price,
            d_div(carry, shares, "CoveredCall::break_even/carry_per_share")?,
            "CoveredCall::break_even/below",
        )?;
        if below >= Decimal::ZERO && below <= strike {
            zeros.push(below);
        }

        // (N C - Q K - P + F) / (N - Q), above the strike.
        let above = d_div(
            d_sub(
                d_sub(
                    d_mul(shares, entry_price, "CoveredCall::break_even/basis")?,
                    carry,
                    "CoveredCall::break_even/basis_net",
                )?,
                d_mul(call_units, strike, "CoveredCall::break_even/cap")?,
                "CoveredCall::break_even/above_numerator",
            )?,
            d_sub(shares, call_units, "CoveredCall::break_even/above_slope")?,
            "CoveredCall::break_even/above",
        )?;
        if above > strike {
            zeros.push(above);
        }

        for zero in zeros {
            if let Ok(be) = Positive::new_decimal(zero) {
                self.break_even_points.push(be.checked_round_to(2)?);
            }
        }
        Ok(())
    }

    /// Replaces the short call with `position` and recomputes the
    /// break-evens; on error the strategy is left as it was.
    ///
    /// # Errors
    ///
    /// Propagates [`BreakEvenable::update_break_even_points`] as a
    /// [`PositionError`].
    fn replace_short_call(&mut self, position: &Position) -> Result<(), PositionError> {
        let previous_call = std::mem::replace(&mut self.short_call, position.clone());
        let previous_break_evens = self.break_even_points.clone();
        if let Err(error) = self.update_break_even_points() {
            self.short_call = previous_call;
            self.break_even_points = previous_break_evens;
            return Err(error.into());
        }
        Ok(())
    }

    /// Checks if the call is currently in-the-money.
    #[must_use]
    pub fn is_call_itm(&self, current_price: Positive) -> bool {
        current_price > self.call_strike()
    }

    /// Calculates the probability of the call being assigned.
    ///
    /// This is a simplified calculation based on moneyness.
    #[must_use]
    pub fn assignment_probability(&self, current_price: Positive) -> Decimal {
        if current_price >= self.call_strike() {
            Decimal::ONE
        } else {
            current_price.to_dec() / self.call_strike().to_dec()
        }
    }
}

impl Validable for CoveredCall {
    fn validate(&self) -> bool {
        // Validate spot position
        if self.spot_leg.quantity == Positive::ZERO {
            debug!("Invalid: Spot quantity is zero");
            return false;
        }

        if self.spot_leg.side != Side::Long {
            debug!("Invalid: Spot position must be long");
            return false;
        }

        // Validate short call
        if self.short_call.option.side != Side::Short {
            debug!("Invalid: Call option must be short");
            return false;
        }

        if self.short_call.option.option_style != OptionStyle::Call {
            debug!("Invalid: Option must be a call");
            return false;
        }

        true
    }
}

impl BreakEvenable for CoveredCall {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    /// The zeros of the expiry P&L. With `N` shares bought at `C`, a call of
    /// `Q` units (`quantity × contract_size`) struck at `K`, premium received
    /// `P` and total fees `F`:
    ///
    /// - at or below the strike, `N (S - C) + P - F`, zero at
    ///   `S = C - (P - F) / N`;
    /// - above the strike, `(N - Q) S - N C + Q K + P - F`, zero at
    ///   `S = (N C - Q K - P + F) / (N - Q)` when `Q != N`.
    ///
    /// When the call covers the shares exactly the single break-even
    /// `C - (P - F) / N` is recorded, floored at zero, as before. Otherwise
    /// each zero is recorded only inside its own region, in ascending order.
    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points.clear();
        if !self.is_exact_cover()? {
            return self.update_mismatched_break_even_points();
        }

        // Break-even = Cost Basis - Premium Received per Share
        let premium_per_share = self
            .short_call
            .premium
            .checked_mul(&self.short_call.option.contract_size)?
            .checked_mul(&self.short_call.option.quantity)?
            // Same rounding choice as the per-share premium above.
            .checked_div_with_strategy(
                &self.spot_leg.quantity,
                RoundingStrategy::MidpointNearestEven,
            )?;
        // The call's fees count too: the payoff charges them (#731).
        let fees_per_share = self
            .total_fees()?
            // Same rounding choice as the per-share premium above.
            .checked_div_with_strategy(
                &self.spot_leg.quantity,
                RoundingStrategy::MidpointNearestEven,
            )?;

        // Net credit per share, which is negative when the fees swallow the
        // premium. `lower_break_even` floors the result at zero, the sentinel
        // for "the shares cannot be losing at any attainable price".
        let net_credit_per_share = d_sub(
            premium_per_share.to_dec(),
            fees_per_share.to_dec(),
            "CoveredCall::update_break_even_points",
        )?;
        let break_even = lower_break_even(self.spot_leg.cost_basis, net_credit_per_share);

        self.break_even_points.push(break_even.checked_round_to(2)?);
        Ok(())
    }
}

impl Positionable for CoveredCall {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        // Only allow adding/updating the short call position
        if position.option.side != Side::Short || position.option.option_style != OptionStyle::Call
        {
            return Err(PositionError::invalid_position_type(
                position.option.side,
                "CoveredCall only accepts short call positions".to_string(),
            ));
        }

        self.replace_short_call(position)
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.short_call])
    }

    fn get_position(
        &mut self,
        option_style: &OptionStyle,
        side: &Side,
        strike: &Positive,
    ) -> Result<Vec<&mut Position>, PositionError> {
        if *option_style == OptionStyle::Call
            && *side == Side::Short
            && *strike == self.short_call.option.strike_price
        {
            Ok(vec![&mut self.short_call])
        } else {
            Err(PositionError::invalid_position(
                "Position not found in CoveredCall",
            ))
        }
    }

    fn modify_position(&mut self, position: &Position) -> Result<(), PositionError> {
        if !position.validate() {
            return Err(PositionError::ValidationError(
                PositionValidationErrorKind::InvalidPosition {
                    reason: "Invalid position data".to_string(),
                },
            ));
        }

        if position.option.side == Side::Short
            && position.option.option_style == OptionStyle::Call
            && position.option.strike_price == self.short_call.option.strike_price
        {
            self.replace_short_call(position)
        } else {
            Err(PositionError::invalid_position(
                "Position does not match existing short call",
            ))
        }
    }
}

impl Strategable for CoveredCall {
    fn info(&self) -> Result<StrategyBasics, StrategyError> {
        Ok(StrategyBasics {
            name: self.name.clone(),
            kind: self.kind.clone(),
            description: self.description.clone(),
        })
    }
}

impl BasicAble for CoveredCall {
    // Without this override the trait default aborts the process, and it is
    // reached from `get_underlying_price`, `get_max_min_strikes`,
    // `delta_neutrality` and every probability method. The short call carries the
    // same underlying, expiration and rate as the spot leg, so it answers for
    // the strategy.
    fn one_option(&self) -> &Options {
        self.short_call.one_option()
    }

    fn one_option_mut(&mut self) -> &mut Options {
        self.short_call.one_option_mut()
    }

    fn get_title(&self) -> String {
        format!(
            "CoveredCall Strategy:\n\t{} {} {} @ {}\n\t{}",
            self.spot_leg.side,
            self.spot_leg.quantity,
            self.spot_leg.symbol,
            self.spot_leg.cost_basis,
            self.short_call.get_title()
        )
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
        let mut map = HashMap::new();
        let short_call = &self.short_call.option;
        map.insert(
            OptionBasicType {
                option_style: &short_call.option_style,
                side: &short_call.side,
                strike_price: &short_call.strike_price,
                expiration_date: &short_call.expiration_date,
            },
            &short_call.implied_volatility,
        );
        map
    }

    fn get_quantity(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let mut map = HashMap::new();
        let short_call = &self.short_call.option;
        map.insert(
            OptionBasicType {
                option_style: &short_call.option_style,
                side: &short_call.side,
                strike_price: &short_call.strike_price,
                expiration_date: &short_call.expiration_date,
            },
            &short_call.quantity,
        );
        map
    }
    fn get_contract_size(&self) -> Result<Positive, StrategyError> {
        common_contract_size(&[&self.short_call], "CoveredCall::get_contract_size")
    }
    fn set_contract_size(&mut self, contract_size: Positive) -> Result<(), StrategyError> {
        apply_hedge_contract_size(
            &mut [&mut self.short_call],
            contract_size,
            "CoveredCall::set_contract_size",
        )?;
        self.update_break_even_points()
    }
}

impl Strategies for CoveredCall {
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        self.max_profit_potential().map_err(StrategyError::from)
    }

    fn get_max_loss(&self) -> Result<Positive, StrategyError> {
        self.max_loss_potential().map_err(StrategyError::from)
    }
}

impl Profit for CoveredCall {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, PricingError> {
        // A leg that cannot be valued is an error, not a zero contribution:
        // the charts and the expiry P&L are built on this sum (#731).
        // Spot P&L
        let spot_pnl = self.spot_leg.pnl_at_price(*price)?;

        // Option P&L at expiration
        let option_pnl = self.short_call.pnl_at_expiration(&Some(price))?;

        Ok(d_add(
            spot_pnl,
            option_pnl,
            "CoveredCall::pnl_at_expiration",
        )?)
    }
}

impl Greeks for CoveredCall {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.short_call.option])
    }

    fn delta(&self) -> Result<Decimal, GreeksError> {
        self.net_delta()
    }
}

impl PnLCalculator for CoveredCall {
    /// Marks the strategy to market at `underlying_price` (#728).
    ///
    /// Each option leg goes through `Position::calculate_pnl` with the given
    /// expiration and volatility, and the share leg is valued at
    /// `underlying_price`; the legs are summed with `PnL::try_add`.
    /// `unrealized` is the change in the book's value since entry and
    /// `realized` the entry cash flows. The payoff at expiry is
    /// [`PnLCalculator::calculate_pnl_at_expiration`], a separate path.
    fn calculate_pnl(
        &self,
        underlying_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<optionstratlib_analytics::pnl::utils::PnL, PricingError> {
        // Until #728 this returned the expiry P&L and ignored the date and
        // the volatility.
        let mut total = spot_leg_mark_to_market(&self.spot_leg, underlying_price)?;
        total = total.try_add(&self.short_call.calculate_pnl(
            underlying_price,
            expiration_date,
            implied_volatility,
        )?)?;
        Ok(total)
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<optionstratlib_analytics::pnl::utils::PnL, PricingError> {
        let profit = self.calculate_profit_at(underlying_price)?;
        let spot_cost = self.spot_leg.total_cost()?;
        let premium_received = self
            .short_call
            .premium
            .checked_mul(&self.short_call.option.contract_size)?
            .checked_mul(&self.short_call.option.quantity)?;

        Ok(optionstratlib_analytics::pnl::utils::PnL {
            realized: None,
            unrealized: Some(profit),
            initial_costs: spot_cost,
            initial_income: premium_received,
            date_time: Utc::now(),
        })
    }
}

impl DeltaNeutrality for CoveredCall {}

impl Optimizable for CoveredCall {
    type Strategy = CoveredCall;
}

impl crate::strategies::StrategyConstructor for CoveredCall {}

impl ProbabilityAnalysis for CoveredCall {
    /// With the call covering the shares exactly the profit zone runs from
    /// the break-even up to the strike. A mismatched call lets the P&L cross
    /// zero above the strike too, and the zones then follow the sign of the
    /// expiry P&L between the break-evens.
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        if !self.is_exact_cover()? {
            let (mut profit, _) = self.mismatched_cover_zones()?;
            price_zones(
                &mut profit,
                &self.spot_leg.cost_basis,
                &self.short_call.option,
            )?;
            return Ok(profit);
        }
        let break_even_point =
            self.break_even_points
                .first()
                .copied()
                .ok_or(ProbabilityError::MissingMetric {
                    metric: "break_even_point",
                })?;

        let option = &self.short_call.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        // Profit range: from break-even up to strike (capped profit)
        let mut profit_range = ProfitLossRange::new(
            Some(break_even_point),
            Some(self.call_strike()),
            Positive::ZERO,
        )?;

        profit_range.calculate_probability(
            &self.spot_leg.cost_basis,
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

    /// With the call covering the shares exactly the loss zone runs from zero
    /// up to the break-even. A mismatched call follows the sign of the expiry
    /// P&L between the break-evens.
    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        if !self.is_exact_cover()? {
            let (_, mut loss) = self.mismatched_cover_zones()?;
            price_zones(
                &mut loss,
                &self.spot_leg.cost_basis,
                &self.short_call.option,
            )?;
            return Ok(loss);
        }
        let break_even_point =
            self.break_even_points
                .first()
                .copied()
                .ok_or(ProbabilityError::MissingMetric {
                    metric: "break_even_point",
                })?;

        let option = &self.short_call.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        // Loss range: from zero up to break-even
        let mut loss_range =
            ProfitLossRange::new(Some(Positive::ZERO), Some(break_even_point), Positive::ZERO)?;

        loss_range.calculate_probability(
            &self.spot_leg.cost_basis,
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

impl std::fmt::Display for CoveredCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CoveredCall: {} {} @ {} + Short {} Call @ {}",
            self.spot_leg.side,
            self.spot_leg.quantity,
            self.spot_leg.cost_basis,
            self.short_call.option.strike_price,
            self.short_call.premium
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use optionstratlib_core::pos_or_panic;
    use rust_decimal_macros::dec;

    fn create_test_covered_call() -> CoveredCall {
        CoveredCall::new(
            "AAPL".to_string(),
            pos_or_panic!(150.0),
            pos_or_panic!(155.0),
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

    /// A per-share premium that does not divide exactly: one option unit of
    /// 3.50 spread over three shares is 1.1666… repeating. The rounding is
    /// this crate's choice rather than the dependency's default, so the
    /// quotient is pinned here — if the strategy ever changes, the last digit
    /// moves and this test says so.
    #[test]
    fn test_covered_call_per_share_premium_rounds_to_nearest_even() {
        let mut cc = create_test_covered_call();
        cc.spot_leg.quantity = pos_or_panic!(3.0);
        cc.spot_leg.cost_basis = Positive::HUNDRED;
        // One call unit against three shares keeps the quotient repeating;
        // `new` sizes the call in shares since #731, so it is set here.
        cc.short_call.option.quantity = Positive::ONE;

        // 100 - 3.50/3. The quotient repeats, so it is rounded, and the
        // literal is written as a `Decimal` rather than through the `f64`
        // path of `pos_or_panic!`, which cannot carry these digits.
        assert_eq!(
            cc.effective_cost_basis().unwrap().to_dec(),
            dec!(98.83333333333333333333333333)
        );
    }

    #[test]
    fn test_covered_call_creation() {
        let cc = create_test_covered_call();

        assert_eq!(cc.name, "Covered Call");
        assert_eq!(cc.kind, StrategyType::CoveredCall);
        assert_eq!(cc.spot_leg.symbol, "AAPL");
        assert_eq!(cc.spot_leg.quantity, Positive::HUNDRED);
        assert_eq!(cc.spot_leg.cost_basis, pos_or_panic!(150.0));
        assert_eq!(cc.spot_leg.side, Side::Long);
        assert_eq!(cc.short_call.option.strike_price, pos_or_panic!(155.0));
        assert_eq!(cc.short_call.option.side, Side::Short);
    }

    #[test]
    fn test_covered_call_validation() {
        let cc = create_test_covered_call();
        assert!(cc.validate());
    }

    #[test]
    fn test_break_even_calculation() {
        let cc = create_test_covered_call();

        // Break-even should be cost basis minus premium received per share
        assert!(!cc.break_even_points.is_empty());
        let break_even = cc.break_even_points[0];
        assert!(break_even < pos_or_panic!(150.0)); // Should be below cost basis
    }

    #[test]
    fn test_effective_cost_basis() {
        let cc = create_test_covered_call();
        let effective = cc.effective_cost_basis();

        // Effective cost basis should be lower than original
        assert!(effective.is_ok_and(|e| e < cc.spot_leg.cost_basis));
    }

    /// `effective_cost_basis` divides by `spot_leg.quantity`, a `pub` field.
    /// `CoveredCall::new` rejects a zero share count, but a covered call
    /// mutated in place or deserialized from JSON can carry one, and there is
    /// no per-share cost basis to report for it.
    #[test]
    fn test_covered_call_effective_cost_basis_zero_shares_is_reported() {
        let mut cc = create_test_covered_call();
        cc.spot_leg.quantity = Positive::ZERO;

        assert!(cc.effective_cost_basis().is_err());
    }

    #[test]
    fn test_profit_at_strike() {
        let cc = create_test_covered_call();

        // At strike price, should have maximum profit
        let profit = cc.calculate_profit_at(&pos_or_panic!(155.0)).unwrap();
        assert!(profit > Decimal::ZERO);
    }

    #[test]
    fn test_profit_above_strike() {
        let cc = create_test_covered_call();

        // Above strike, the short call gets exercised
        // Spot gains continue but are offset by short call losses
        let profit_at_strike = cc.calculate_profit_at(&pos_or_panic!(155.0)).unwrap();
        let profit_above = cc.calculate_profit_at(&pos_or_panic!(170.0)).unwrap();

        // Both should be positive (profitable strategy when price rises)
        assert!(profit_at_strike > Decimal::ZERO);
        assert!(profit_above > Decimal::ZERO);
    }

    #[test]
    fn test_loss_at_zero() {
        let cc = create_test_covered_call();

        // At zero, maximum loss
        let loss = cc.calculate_profit_at(&pos_or_panic!(0.01)).unwrap();
        assert!(loss < Decimal::ZERO);
    }

    #[test]
    fn test_get_legs() {
        let cc = create_test_covered_call();
        let legs = cc.get_legs();

        assert_eq!(legs.len(), 2);
        assert!(legs[0].is_spot());
        assert!(legs[1].is_option());
    }

    #[test]
    fn test_net_delta() {
        let cc = create_test_covered_call();
        let delta = cc.net_delta().unwrap();

        // Net delta should be positive but less than spot quantity
        // (spot delta is +100, short call delta is negative)
        assert!(delta > Decimal::ZERO);
        assert!(delta < dec!(100.0));
    }

    #[test]
    fn test_is_call_itm() {
        let cc = create_test_covered_call();

        assert!(!cc.is_call_itm(pos_or_panic!(150.0))); // Below strike
        assert!(!cc.is_call_itm(pos_or_panic!(155.0))); // At strike
        assert!(cc.is_call_itm(pos_or_panic!(160.0))); // Above strike
    }

    #[test]
    fn test_display() {
        let cc = create_test_covered_call();
        let display = format!("{}", cc);

        assert!(display.contains("CoveredCall"));
        assert!(display.contains("Long"));
        assert!(display.contains("100"));
    }

    #[test]
    fn test_get_title() {
        let cc = create_test_covered_call();
        let title = cc.get_title();

        assert!(title.contains("CoveredCall"));
        assert!(title.contains("AAPL"));
    }

    #[test]
    fn test_underlying_price() {
        let cc = create_test_covered_call();
        assert_eq!(cc.underlying_price(), pos_or_panic!(150.0));
    }

    #[test]
    fn test_call_strike() {
        let cc = create_test_covered_call();
        assert_eq!(cc.call_strike(), pos_or_panic!(155.0));
    }

    #[test]
    fn test_quantity() {
        let cc = create_test_covered_call();
        assert_eq!(cc.quantity(), Positive::HUNDRED);
    }

    /// A short call whose premium times its quantity leaves the
    /// `Positive` range reports the overflow instead of aborting. `new` would
    /// not build it, but the public fields can; the panic-freedom property
    /// found this once #696 drove its rejected cases through them.
    #[test]
    fn test_covered_call_premium_overflow_reports_error() {
        let mut cc = create_test_covered_call();
        cc.short_call.premium = Positive::MAX;
        cc.short_call.option.quantity = pos_or_panic!(10000.0);
        assert!(cc.max_profit_potential().is_err());
        assert!(cc.max_loss_potential().is_err());
    }
}
