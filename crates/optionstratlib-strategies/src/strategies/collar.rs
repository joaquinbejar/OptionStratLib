/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 12/01/26
******************************************************************************/

//! # Collar Strategy
//!
//! A collar involves holding a long position in the underlying asset, buying an
//! out-of-the-money put option (protective put), and selling an out-of-the-money
//! call option (covered call). This strategy provides downside protection at the
//! cost of limiting upside potential.
//!
//! ## Key Characteristics
//!
//! - Limited profit potential (capped by short call strike)
//! - Limited loss potential (protected by long put strike)
//! - Can often be implemented for little to no cost (zero-cost collar)
//! - Ideal for protecting gains on existing long positions
//!
//! ## Components
//!
//! - **Long Spot Position**: Ownership of the underlying asset
//! - **Long Put Option**: Protective put at a lower strike price
//! - **Short Call Option**: Covered call at a higher strike price
//!
//! ## Profit/Loss Profile
//!
//! ```text
//! Profit ^
//!        |     ___________ <- Max profit (capped by short call)
//!        |    /
//!        |   /
//!        |--/-------------> Underlying Price
//!        | /
//!        |/______________ <- Max loss (limited by long put)
//!        |
//! ```
//!
//! ## Example
//!
//! ```rust
//! use optionstratlib_strategies::strategies::collar::Collar;
//! use optionstratlib_core::model::ExpirationDate;
//! use optionstratlib_core::pos_or_panic;
//! use optionstratlib_core::model::Positive;
//! use rust_decimal_macros::dec;
//!
//! let collar = Collar::new(
//!     "AAPL".to_string(),
//!     pos_or_panic!(150.0),    // underlying price
//!     pos_or_panic!(145.0),    // put strike (protection level)
//!     pos_or_panic!(160.0),    // call strike (profit cap)
//!     ExpirationDate::Days(pos_or_panic!(30.0)),
//!     pos_or_panic!(0.25),     // implied volatility
//!     dec!(0.05),              // risk-free rate
//!     pos_or_panic!(0.01),     // dividend yield
//!     Positive::HUNDRED,       // quantity (shares)
//!     pos_or_panic!(2.50),     // put premium paid
//!     pos_or_panic!(3.00),     // call premium received
//!     Positive::ONE,           // spot open fee
//!     Positive::ONE,           // spot close fee
//!     pos_or_panic!(0.65),     // put open fee
//!     pos_or_panic!(0.65),     // put close fee
//!     pos_or_panic!(0.65),     // call open fee
//!     pos_or_panic!(0.65),     // call close fee
//! );
//! ```

use super::base::{
    BreakEvenable, Optimizable, Positionable, Strategable, StrategyBasics, StrategyType, Validable,
};
use crate::error::StrategyError;
use crate::strategies::base::price_gap;
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
use optionstratlib_core::model::decimal::{d_add, d_div, d_mul, d_sub};
use optionstratlib_core::model::leg::traits::LegAble;
use optionstratlib_core::model::leg::{Leg, SpotPosition};
use optionstratlib_core::model::position::Position;
use optionstratlib_core::model::types::{OptionBasicType, OptionStyle, OptionType, Side};
use optionstratlib_pricing::error::GreeksError;
use optionstratlib_pricing::error::PricingError;
use optionstratlib_pricing::greeks::Greeks;
use optionstratlib_pricing::greeks::LegGreeks;
use optionstratlib_pricing::pricing::Profit;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::debug;

/// Default description for the Collar strategy.
pub const COLLAR_DESCRIPTION: &str = "A collar is a protective options strategy that involves \
    holding a long position in the underlying asset, buying a protective put option at a lower \
    strike price, and selling a covered call option at a higher strike price. This creates a \
    'collar' around the current price, limiting both upside potential and downside risk. The \
    strategy is ideal for investors who want to protect gains on existing positions while \
    potentially offsetting the cost of protection with premium received from the short call.";

/// Represents a Collar options trading strategy.
///
/// A Collar combines a long position in the underlying asset with a protective
/// put (long put) and a covered call (short call). This strategy is used to
/// protect existing holdings while accepting limited upside potential.
///
/// # Structure
///
/// - **Spot Leg**: Long position in the underlying asset
/// - **Long Put**: Protective put at a strike below current price
/// - **Short Call**: Covered call at a strike above current price
///
/// # Profit/Loss Profile
///
/// - **Maximum Profit**: (Call Strike - Cost Basis) + Net Premium
/// - **Maximum Loss**: (Cost Basis - Put Strike) - Net Premium
/// - **Break-even**: Cost Basis - Net Premium (if credit) or + Net Premium (if debit)
///
/// These hold when each option leg covers the shares one for one
/// (`quantity × contract_size` equal to the spot quantity), as `Collar::new`
/// and `BasicAble::set_contract_size` build them. A leg resized to cover
/// fewer or more units changes the slope beyond its strike: an under-hedged
/// put leaves the uncovered shares falling to zero, an under-covered call
/// leaves the profit unbounded, an over-covered call the loss. The
/// break-evens, max profit and max loss and the profit and loss zones then
/// follow the expiry P&L for that cover.
///
/// # Greeks
///
/// - **Delta**: Positive (long spot delta + long put delta + short call delta)
/// - **Gamma**: Mixed (positive from long put, negative from short call)
/// - **Theta**: Mixed (negative from long put, positive from short call)
/// - **Vega**: Mixed (positive from long put, negative from short call)
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
pub struct Collar {
    /// The name of the strategy.
    pub name: String,

    /// The type of strategy (StrategyType::Collar).
    pub kind: StrategyType,

    /// A textual description of this strategy instance.
    pub description: String,

    /// The price points at which the strategy breaks even.
    pub break_even_points: Vec<Positive>,

    /// The long spot position (underlying asset).
    pub spot_leg: SpotPosition,

    /// The long put option position (protective put).
    pub long_put: Position,

    /// The short call option position (covered call).
    pub short_call: Position,
}

impl Collar {
    /// Creates a new Collar strategy.
    ///
    /// # Arguments
    ///
    /// * `underlying_symbol` - The ticker symbol of the underlying asset
    /// * `underlying_price` - The current market price of the underlying asset
    /// * `put_strike` - The strike price for the long put option (protection level)
    /// * `call_strike` - The strike price for the short call option (profit cap)
    /// * `expiration` - The expiration date for both options
    /// * `implied_volatility` - The implied volatility for option pricing
    /// * `risk_free_rate` - The risk-free interest rate
    /// * `dividend_yield` - The dividend yield of the underlying asset
    /// * `quantity` - The number of shares. The option leg covers the same
    ///   shares one for one: it is built as one-unit contracts, one per
    ///   share (#731). `BasicAble::set_contract_size` re-expresses it in
    ///   market contracts with the same payoff and fees.
    /// * `premium_long_put` - The premium paid for buying the put
    /// * `premium_short_call` - The premium received for selling the call
    /// * `spot_open_fee` - Fee to open the spot position
    /// * `spot_close_fee` - Fee to close the spot position
    /// * `put_open_fee` - Fee to open the put position, per share
    /// * `put_close_fee` - Fee to close the put position, per share
    /// * `call_open_fee` - Fee to open the call position, per share
    /// * `call_close_fee` - Fee to close the call position, per share
    ///
    /// # Returns
    ///
    /// A fully configured `Collar` strategy instance.
    ///
    /// # Errors
    ///
    /// Returns `StrategyError::InvalidStrategy` when the assembled strategy
    /// fails its own `validate` (#696): the put strike is not below the call
    /// strike.
    ///
    /// Returns `StrategyError` if the break-even calculation fails. In
    /// practice this branch is unreachable for a freshly-built collar and
    /// is surfaced only to keep the constructor panic-free.
    #[allow(clippy::too_many_arguments)]
    #[inline(never)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        put_strike: Positive,
        call_strike: Positive,
        expiration: ExpirationDate,
        implied_volatility: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        quantity: Positive,
        premium_long_put: Positive,
        premium_short_call: Positive,
        spot_open_fee: Positive,
        spot_close_fee: Positive,
        put_open_fee: Positive,
        put_close_fee: Positive,
        call_open_fee: Positive,
        call_close_fee: Positive,
    ) -> Result<Self, StrategyError> {
        // Create the spot position (long underlying)
        // Every per-share figure this strategy reports — the effective cost
        // basis, the break-even, the premium per share — divides by the size
        // of the spot leg. A collar with no shares is not a position, and
        // rejecting it here keeps those divisors non-zero.
        if quantity == Positive::ZERO {
            return Err(StrategyError::invalid_parameters(
                "Collar::new",
                "quantity must be strictly positive: a collar with no shares has no per-share basis",
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

        // Create the long put option (protective put)
        let long_put_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            put_strike,
            expiration,
            implied_volatility,
            quantity, // One option unit per share (#731)
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
            put_open_fee,
            put_close_fee,
            None,
            None,
        );

        // Create the short call option (covered call)
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

        let mut strategy = Collar {
            name: "Collar".to_string(),
            kind: StrategyType::Collar,
            description: COLLAR_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            spot_leg,
            long_put,
            short_call,
        };

        if !strategy.validate() {
            return Err(StrategyError::invalid_strategy(
                StrategyType::Collar,
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

    /// Returns the long put leg as a `Leg` enum.
    #[must_use]
    pub fn get_put_leg(&self) -> Leg {
        Leg::option(self.long_put.clone())
    }

    /// Returns the short call leg as a `Leg` enum.
    #[must_use]
    pub fn get_call_leg(&self) -> Leg {
        Leg::option(self.short_call.clone())
    }

    /// Returns all legs of the strategy.
    #[must_use]
    pub fn get_legs(&self) -> Vec<Leg> {
        vec![self.get_spot_leg(), self.get_put_leg(), self.get_call_leg()]
    }

    /// Returns the put strike price.
    #[must_use]
    pub fn put_strike(&self) -> Positive {
        self.long_put.option.strike_price
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

    /// Returns the collar width (distance between put and call strikes).
    ///
    /// A put struck above the call inverts the collar; the width of that
    /// region is zero, not a negative price.
    #[must_use]
    pub fn collar_width(&self) -> Positive {
        price_gap(self.call_strike(), self.put_strike())
    }

    /// Calculates the net premium (credit if positive, debit if negative).
    ///
    /// Net Premium = Call Premium Received - Put Premium Paid
    ///
    /// # Decision (issue #471): return `Result`, do not privatise the fields
    ///
    /// This used to return a bare `Decimal` over `premium * quantity` on both
    /// legs. `Position::premium` and `Options::quantity` are `pub`, so
    /// [`Collar::new`] cannot bound either product, and a signed premium has
    /// no sentinel that means "did not fit". Hiding the fields would have
    /// been a larger break than adding the error channel and would still
    /// leave `Collar` deserializable from arbitrary JSON, so the channel is
    /// what was added.
    ///
    /// # Errors
    ///
    /// Returns [`PricingError::Positive`] when either leg's `premium ×
    /// contract_size × quantity` leaves the `Positive` range, and
    /// [`PricingError::Decimal`]
    /// when their difference leaves the representable `Decimal` range.
    pub fn net_premium(&self) -> Result<Decimal, PricingError> {
        let call_premium = self
            .short_call
            .premium
            .checked_mul(&self.short_call.option.contract_size)?
            .checked_mul(&self.short_call.option.quantity)?;
        let put_premium = self
            .long_put
            .premium
            .checked_mul(&self.long_put.option.contract_size)?
            .checked_mul(&self.long_put.option.quantity)?;
        Ok(d_sub(
            call_premium.to_dec(),
            put_premium.to_dec(),
            "Collar::net_premium",
        )?)
    }

    /// Returns true if this is a zero-cost collar (net premium is approximately zero).
    ///
    /// # Errors
    ///
    /// Propagates [`Collar::net_premium`]: a premium total that does not fit
    /// is not a collar this predicate can classify, and `false` would read as
    /// "not zero-cost" rather than "not known".
    pub fn is_zero_cost(&self) -> Result<bool, PricingError> {
        Ok(self.net_premium()?.abs() < Decimal::new(1, 2)) // Less than $0.01
    }

    /// Returns true if this is a credit collar (net premium received).
    ///
    /// # Errors
    ///
    /// Propagates [`Collar::net_premium`]: a premium total that does not fit
    /// is not a collar this predicate can classify, and `false` would read as
    /// "a debit collar" rather than "not known".
    pub fn is_credit(&self) -> Result<bool, PricingError> {
        Ok(self.net_premium()? > Decimal::ZERO)
    }

    /// Calculates the net delta of the collar.
    ///
    /// Net Delta = Spot Delta + Put Delta + Call Delta
    ///
    /// # Errors
    ///
    /// Propagates any [`GreeksError`] returned by
    /// [`LegGreeks::delta`] on the spot leg, long-put leg or short-call leg.
    pub fn net_delta(&self) -> Result<Decimal, GreeksError> {
        let spot_delta = self.spot_leg.delta()?;
        let put_delta = self.long_put.delta()?;
        let call_delta = self.short_call.delta()?;
        Ok(d_add(
            d_add(spot_delta, put_delta, "Collar::net_delta")?,
            call_delta,
            "Collar::net_delta",
        )?)
    }

    /// Calculates the maximum profit potential, floored at zero.
    ///
    /// Max Profit = (Call Strike - Cost Basis) × Quantity + Net Premium - Fees
    /// when both option legs cover the shares exactly, so the call caps the
    /// gain at its strike.
    ///
    /// A leg that does not cover the shares changes the slope beyond its
    /// strike: `N - Q` above the call strike for `Q` call units against `N`
    /// shares, `N - P` below the put strike for `P` put units. A call that
    /// covers fewer units than the shares leaves the profit unbounded, which
    /// is reported as `Positive::MAX`. Otherwise the expiry P&L is piecewise
    /// linear and peaks at zero, at the put strike or at the call strike, and
    /// the largest of the three is reported.
    ///
    /// # Errors
    ///
    /// Returns [`PricingError`] when a leg's size, premium or fees, or the
    /// expiry P&L at a strike, leaves the `Positive` or `Decimal` range.
    pub fn max_profit_potential(&self) -> Result<Positive, PricingError> {
        let call_strike = self.call_strike();
        let cost_basis = self.spot_leg.cost_basis;
        let quantity = self.spot_leg.quantity;
        // Checked before the cover is classified, so an unpriceable leg is
        // an error, not an unbounded figure.
        let net_premium = self.net_premium()?;
        let total_fees = self.total_fees()?;
        let (put_units, call_units) = self.hedge_units()?;
        if put_units != quantity || call_units != quantity {
            if call_units < quantity {
                return Ok(Positive::MAX);
            }
            let best = self
                .profit_at_kinks()?
                .into_iter()
                .fold(Decimal::MIN, Decimal::max);
            return Ok(Positive::new_decimal(best.max(Decimal::ZERO)).unwrap_or(Positive::ZERO));
        }

        if call_strike >= cost_basis {
            let capital_gain = price_gap(call_strike, cost_basis).checked_mul(&quantity)?;
            let total_profit = d_sub(
                d_add(capital_gain.to_dec(), net_premium, "Collar::max_profit")?,
                total_fees.to_dec(),
                "Collar::max_profit",
            )?;
            Ok(Positive::new_decimal(total_profit.max(Decimal::ZERO)).unwrap_or(Positive::ZERO))
        } else {
            // Call strike below cost basis
            let capital_loss = price_gap(cost_basis, call_strike).checked_mul(&quantity)?;
            let total_profit = d_sub(
                d_sub(net_premium, capital_loss.to_dec(), "Collar::max_profit")?,
                total_fees.to_dec(),
                "Collar::max_profit",
            )?;
            Ok(Positive::new_decimal(total_profit.max(Decimal::ZERO)).unwrap_or(Positive::ZERO))
        }
    }

    /// Calculates the maximum loss potential, floored at zero.
    ///
    /// Max Loss = (Cost Basis - Put Strike) × Quantity - Net Premium + Fees
    /// when both option legs cover the shares exactly, so the put floors the
    /// loss at its strike.
    ///
    /// A call that covers more units than the shares is net short above its
    /// strike and its loss is unbounded, which is reported as
    /// `Positive::MAX`. Otherwise the expiry P&L bottoms out at zero, at the
    /// put strike or at the call strike: an under-hedged put leaves the
    /// unprotected shares falling to zero, and the deepest of the three is
    /// reported.
    ///
    /// # Errors
    ///
    /// Returns [`PricingError`] when a leg's size, premium or fees, or the
    /// expiry P&L at a strike, leaves the `Positive` or `Decimal` range.
    pub fn max_loss_potential(&self) -> Result<Positive, PricingError> {
        let put_strike = self.put_strike();
        let cost_basis = self.spot_leg.cost_basis;
        let quantity = self.spot_leg.quantity;
        // Checked before the cover is classified; see `max_profit_potential`.
        let net_premium = self.net_premium()?;
        let total_fees = self.total_fees()?;
        let (put_units, call_units) = self.hedge_units()?;
        if put_units != quantity || call_units != quantity {
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

        if cost_basis >= put_strike {
            let capital_loss = price_gap(cost_basis, put_strike).checked_mul(&quantity)?;
            let total_loss = d_add(
                d_sub(capital_loss.to_dec(), net_premium, "Collar::max_loss")?,
                total_fees.to_dec(),
                "Collar::max_loss",
            )?;
            Ok(Positive::new_decimal(total_loss.max(Decimal::ZERO)).unwrap_or(Positive::ZERO))
        } else {
            // Put strike above cost basis (unusual but possible)
            let capital_gain = price_gap(put_strike, cost_basis).checked_mul(&quantity)?;
            let total_loss = d_sub(
                d_sub(total_fees.to_dec(), net_premium, "Collar::max_loss")?,
                capital_gain.to_dec(),
                "Collar::max_loss",
            )?;
            Ok(Positive::new_decimal(total_loss.max(Decimal::ZERO)).unwrap_or(Positive::ZERO))
        }
    }

    /// Calculates total fees for all positions: the share leg's open and
    /// close fees, plus each option leg's, which are per share and scale with
    /// its quantity as in `Position::fees`. This is what the payoff charges;
    /// the option fees used to be counted once whatever the quantity (#731).
    ///
    /// # Errors
    ///
    /// Returns [`PositionError`] when a sum or an option leg's product leaves
    /// the `Positive` range.
    fn total_fees(&self) -> Result<Positive, PositionError> {
        Ok(self
            .spot_leg
            .open_fee
            .checked_add(&self.spot_leg.close_fee)?
            .checked_add(&self.long_put.fees()?)?
            .checked_add(&self.short_call.fees()?)?)
    }

    /// The units of the underlying each option leg covers, `quantity ×
    /// contract_size`, as `(put, call)`.
    ///
    /// # Errors
    ///
    /// Returns [`PricingError`] when a leg's size leaves the `Positive`
    /// range.
    fn hedge_units(&self) -> Result<(Positive, Positive), PricingError> {
        Ok((
            self.long_put.option.position_size()?,
            self.short_call.option.position_size()?,
        ))
    }

    /// Whether both option legs cover the shares exactly.
    ///
    /// # Errors
    ///
    /// Propagates [`Collar::hedge_units`].
    fn is_exact_hedge(&self) -> Result<bool, PricingError> {
        let (put_units, call_units) = self.hedge_units()?;
        let shares = self.spot_leg.quantity;
        Ok(put_units == shares && call_units == shares)
    }

    /// The expiry P&L at the points where its slope can change: zero, the
    /// put strike and the call strike.
    ///
    /// # Errors
    ///
    /// Propagates [`Profit::calculate_profit_at`].
    fn profit_at_kinks(&self) -> Result<[Decimal; 3], PricingError> {
        Ok([
            self.calculate_profit_at(&Positive::ZERO)?,
            self.calculate_profit_at(&self.put_strike())?,
            self.calculate_profit_at(&self.call_strike())?,
        ])
    }

    /// The profit and loss zones of a collar whose legs do not cover the
    /// shares exactly, cut at the break-evens.
    ///
    /// # Errors
    ///
    /// Propagates [`expiry_zones`].
    fn mismatched_hedge_zones(
        &self,
    ) -> Result<(Vec<ProfitLossRange>, Vec<ProfitLossRange>), ProbabilityError> {
        expiry_zones(&self.break_even_points, self.spot_leg.cost_basis, |price| {
            self.calculate_profit_at(price)
        })
    }

    /// Checks if the put is currently in-the-money.
    #[must_use]
    pub fn is_put_itm(&self, current_price: Positive) -> bool {
        current_price < self.put_strike()
    }

    /// Checks if the call is currently in-the-money.
    #[must_use]
    pub fn is_call_itm(&self, current_price: Positive) -> bool {
        current_price > self.call_strike()
    }
}

impl Validable for Collar {
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

        // Validate long put
        if self.long_put.option.side != Side::Long {
            debug!("Invalid: Put option must be long");
            return false;
        }

        if self.long_put.option.option_style != OptionStyle::Put {
            debug!("Invalid: Long option must be a put");
            return false;
        }

        // Validate short call
        if self.short_call.option.side != Side::Short {
            debug!("Invalid: Call option must be short");
            return false;
        }

        if self.short_call.option.option_style != OptionStyle::Call {
            debug!("Invalid: Short option must be a call");
            return false;
        }

        // Validate strike order: put_strike < underlying_price < call_strike (typical)
        // Note: This is the typical setup but not strictly required
        if self.put_strike() >= self.call_strike() {
            debug!("Invalid: Put strike must be less than call strike");
            return false;
        }

        true
    }
}

impl BreakEvenable for Collar {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    /// The zeros of the expiry P&L. With `N` shares bought at `C`, a put of
    /// `P` units struck at `Kp`, a call of `Q` units struck at `Kc` (units
    /// are `quantity × contract_size`), net premium `NP` (credit positive)
    /// and total fees `F`:
    ///
    /// - between the strikes, `N (S - C) + NP - F`, zero at
    ///   `S = C - (NP - F) / N`;
    /// - below the put strike, `(N - P) S - N C + P Kp + NP - F`, zero at
    ///   `S = (N C - P Kp - NP + F) / (N - P)` when `P != N`;
    /// - above the call strike, `(N - Q) S - N C + Q Kc + NP - F`, zero at
    ///   `S = (N C - Q Kc - NP + F) / (N - Q)` when `Q != N`.
    ///
    /// When both legs cover the shares exactly the single break-even
    /// `C - (NP - F) / N` is recorded wherever it falls, as before. Otherwise
    /// each zero is recorded only inside its own region, in ascending order.
    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points.clear();
        if !self.is_exact_hedge()? {
            return self.update_mismatched_break_even_points();
        }

        // Break-even = Cost Basis - Net Premium per Share
        let net_premium_per_share = d_div(
            self.net_premium()?,
            self.spot_leg.quantity.to_dec(),
            "Collar::update_break_even_points",
        )?;
        let fees_per_share = d_div(
            self.total_fees()?.to_dec(),
            self.spot_leg.quantity.to_dec(),
            "Collar::update_break_even_points",
        )?;

        // The credit lowers the break-even and the fees raise it. A net credit
        // larger than the cost basis puts the break-even below zero, where the
        // underlying cannot trade, and the collar records no break-even at
        // all — the same outcome the unchecked arithmetic produced before.
        let break_even = d_add(
            d_sub(
                self.spot_leg.cost_basis.to_dec(),
                net_premium_per_share,
                "Collar::update_break_even_points",
            )?,
            fees_per_share,
            "Collar::update_break_even_points",
        )?;

        if let Ok(be) = Positive::new_decimal(break_even) {
            self.break_even_points.push(be.checked_round_to(2)?);
        }

        Ok(())
    }
}

impl Collar {
    /// The break-evens of a collar whose option legs do not both cover the
    /// shares exactly; see [`BreakEvenable::update_break_even_points`].
    ///
    /// # Errors
    ///
    /// Returns [`StrategyError`] when a leg's size, the premium, the fees or
    /// a step of the arithmetic leaves the `Positive` or `Decimal` range.
    fn update_mismatched_break_even_points(&mut self) -> Result<(), StrategyError> {
        let shares = self.spot_leg.quantity.to_dec();
        let entry_price = self.spot_leg.cost_basis.to_dec();
        let (put_units, call_units) = self.hedge_units()?;
        let (put_units, call_units) = (put_units.to_dec(), call_units.to_dec());
        let put_strike = self.put_strike().to_dec();
        let call_strike = self.call_strike().to_dec();
        // NP - F: what the options and the fees add to the shares' P&L.
        let carry = d_sub(
            self.net_premium()?,
            self.total_fees()?.to_dec(),
            "Collar::break_even/carry",
        )?;
        // N C - NP + F, the constant every region's zero starts from.
        let basis = d_sub(
            d_mul(shares, entry_price, "Collar::break_even/basis")?,
            carry,
            "Collar::break_even/basis_net",
        )?;

        let mut zeros = Vec::with_capacity(3);
        if put_units != shares {
            // (N C - P Kp - NP + F) / (N - P), below the put strike.
            let below = d_div(
                d_sub(
                    basis,
                    d_mul(put_units, put_strike, "Collar::break_even/floor")?,
                    "Collar::break_even/below_numerator",
                )?,
                d_sub(shares, put_units, "Collar::break_even/below_slope")?,
                "Collar::break_even/below",
            )?;
            if below >= Decimal::ZERO && below < put_strike {
                zeros.push(below);
            }
        }

        // C - (NP - F) / N, between the strikes.
        let middle = d_sub(
            entry_price,
            d_div(carry, shares, "Collar::break_even/carry_per_share")?,
            "Collar::break_even/middle",
        )?;
        if middle >= put_strike && middle <= call_strike {
            zeros.push(middle);
        }

        if call_units != shares {
            // (N C - Q Kc - NP + F) / (N - Q), above the call strike.
            let above = d_div(
                d_sub(
                    basis,
                    d_mul(call_units, call_strike, "Collar::break_even/cap")?,
                    "Collar::break_even/above_numerator",
                )?,
                d_sub(shares, call_units, "Collar::break_even/above_slope")?,
                "Collar::break_even/above",
            )?;
            if above > call_strike {
                zeros.push(above);
            }
        }

        for zero in zeros {
            if let Ok(be) = Positive::new_decimal(zero) {
                self.break_even_points.push(be.checked_round_to(2)?);
            }
        }
        Ok(())
    }

    /// Replaces the option leg of `position`'s style (the long put or the
    /// short call) with `position` and recomputes the break-evens; on error
    /// the strategy is left as it was.
    ///
    /// # Errors
    ///
    /// Propagates [`BreakEvenable::update_break_even_points`] as a
    /// [`PositionError`].
    fn replace_option_leg(&mut self, position: &Position) -> Result<(), PositionError> {
        let style = position.option.option_style;
        let slot = match style {
            OptionStyle::Put => &mut self.long_put,
            OptionStyle::Call => &mut self.short_call,
        };
        let previous_leg = std::mem::replace(slot, position.clone());
        let previous_break_evens = self.break_even_points.clone();
        if let Err(error) = self.update_break_even_points() {
            match style {
                OptionStyle::Put => self.long_put = previous_leg,
                OptionStyle::Call => self.short_call = previous_leg,
            }
            self.break_even_points = previous_break_evens;
            return Err(error.into());
        }
        Ok(())
    }
}

impl Positionable for Collar {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match (position.option.option_style, position.option.side) {
            (OptionStyle::Put, Side::Long) | (OptionStyle::Call, Side::Short) => {
                self.replace_option_leg(position)
            }
            _ => Err(PositionError::invalid_position_type(
                position.option.side,
                "Collar only accepts long put or short call positions".to_string(),
            )),
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.long_put, &self.short_call])
    }

    fn get_position(
        &mut self,
        option_style: &OptionStyle,
        side: &Side,
        strike: &Positive,
    ) -> Result<Vec<&mut Position>, PositionError> {
        match (option_style, side) {
            (OptionStyle::Put, Side::Long) if *strike == self.long_put.option.strike_price => {
                Ok(vec![&mut self.long_put])
            }
            (OptionStyle::Call, Side::Short) if *strike == self.short_call.option.strike_price => {
                Ok(vec![&mut self.short_call])
            }
            _ => Err(PositionError::invalid_position(
                "Position not found in Collar",
            )),
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

        match (position.option.option_style, position.option.side) {
            (OptionStyle::Put, Side::Long)
                if position.option.strike_price == self.long_put.option.strike_price =>
            {
                self.replace_option_leg(position)
            }
            (OptionStyle::Call, Side::Short)
                if position.option.strike_price == self.short_call.option.strike_price =>
            {
                self.replace_option_leg(position)
            }
            _ => Err(PositionError::invalid_position(
                "Position does not match existing collar positions",
            )),
        }
    }
}

impl Strategable for Collar {
    fn info(&self) -> Result<StrategyBasics, StrategyError> {
        Ok(StrategyBasics {
            name: self.name.clone(),
            kind: self.kind.clone(),
            description: self.description.clone(),
        })
    }
}

impl BasicAble for Collar {
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
            "Collar Strategy:\n\t{} {} {} @ {}\n\t{}\n\t{}",
            self.spot_leg.side,
            self.spot_leg.quantity,
            self.spot_leg.symbol,
            self.spot_leg.cost_basis,
            self.long_put.get_title(),
            self.short_call.get_title()
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
        common_contract_size(
            &[&self.long_put, &self.short_call],
            "Collar::get_contract_size",
        )
    }
    fn set_contract_size(&mut self, contract_size: Positive) -> Result<(), StrategyError> {
        apply_hedge_contract_size(
            &mut [&mut self.long_put, &mut self.short_call],
            contract_size,
            "Collar::set_contract_size",
        )?;
        self.update_break_even_points()
    }
}

impl Strategies for Collar {
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        self.max_profit_potential().map_err(StrategyError::from)
    }

    fn get_max_loss(&self) -> Result<Positive, StrategyError> {
        self.max_loss_potential().map_err(StrategyError::from)
    }
}

impl Profit for Collar {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, PricingError> {
        // A leg that cannot be valued is an error, not a zero contribution:
        // the charts and the expiry P&L are built on this sum (#731).
        // Spot P&L
        let spot_pnl = self.spot_leg.pnl_at_price(*price)?;

        // Put P&L at expiration
        let put_pnl = self.long_put.pnl_at_expiration(&Some(price))?;

        // Call P&L at expiration
        let call_pnl = self.short_call.pnl_at_expiration(&Some(price))?;

        Ok(d_add(
            d_add(spot_pnl, put_pnl, "Collar::pnl_at_expiration")?,
            call_pnl,
            "Collar::pnl_at_expiration",
        )?)
    }
}

impl Greeks for Collar {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.long_put.option, &self.short_call.option])
    }

    fn delta(&self) -> Result<Decimal, GreeksError> {
        self.net_delta()
    }
}

impl PnLCalculator for Collar {
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
        total = total.try_add(&self.long_put.calculate_pnl(
            underlying_price,
            expiration_date,
            implied_volatility,
        )?)?;
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
        let put_cost = self
            .long_put
            .premium
            .checked_mul(&self.long_put.option.contract_size)?
            .checked_mul(&self.long_put.option.quantity)?;
        let call_income = self
            .short_call
            .premium
            .checked_mul(&self.short_call.option.contract_size)?
            .checked_mul(&self.short_call.option.quantity)?;

        Ok(optionstratlib_analytics::pnl::utils::PnL {
            realized: None,
            unrealized: Some(profit),
            initial_costs: spot_cost.checked_add(&put_cost)?,
            initial_income: call_income,
            date_time: Utc::now(),
        })
    }
}

impl DeltaNeutrality for Collar {}

impl Optimizable for Collar {
    type Strategy = Collar;
}

impl crate::strategies::StrategyConstructor for Collar {}

impl ProbabilityAnalysis for Collar {
    /// With both legs covering the shares exactly the profit zone runs from
    /// the break-even to the call strike. A mismatched leg lets the P&L cross
    /// zero beyond a strike too, and the zones then follow the sign of the
    /// expiry P&L between the break-evens.
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        if !self.is_exact_hedge()? {
            let (mut profit, _) = self.mismatched_hedge_zones()?;
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

        // Profit range: from break-even up to call strike (capped profit)
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

    /// With both legs covering the shares exactly the loss zone runs from the
    /// put strike up to the break-even. A mismatched leg follows the sign of
    /// the expiry P&L, as in [`ProbabilityAnalysis::get_profit_ranges`].
    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        if !self.is_exact_hedge()? {
            let (_, mut loss) = self.mismatched_hedge_zones()?;
            price_zones(&mut loss, &self.spot_leg.cost_basis, &self.long_put.option)?;
            return Ok(loss);
        }
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

        // Loss range: from put strike up to break-even
        let mut loss_range = ProfitLossRange::new(
            Some(self.put_strike()),
            Some(break_even_point),
            Positive::ZERO,
        )?;

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

impl std::fmt::Display for Collar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Collar: {} {} @ {} + Long {} Put @ {} + Short {} Call @ {}",
            self.spot_leg.side,
            self.spot_leg.quantity,
            self.spot_leg.cost_basis,
            self.long_put.option.strike_price,
            self.long_put.premium,
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

    fn create_test_collar() -> Collar {
        Collar::new(
            "AAPL".to_string(),
            pos_or_panic!(150.0), // underlying price
            pos_or_panic!(145.0), // put strike
            pos_or_panic!(160.0), // call strike
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.25), // implied volatility
            dec!(0.05),          // risk-free rate
            pos_or_panic!(0.01), // dividend yield
            Positive::HUNDRED,   // quantity
            pos_or_panic!(2.50), // put premium
            pos_or_panic!(3.00), // call premium
            Positive::ONE,       // spot open fee
            Positive::ONE,       // spot close fee
            pos_or_panic!(0.65), // put open fee
            pos_or_panic!(0.65), // put close fee
            pos_or_panic!(0.65), // call open fee
            pos_or_panic!(0.65), // call close fee
        )
        .unwrap()
    }

    #[test]
    fn test_collar_creation() {
        let collar = create_test_collar();

        assert_eq!(collar.name, "Collar");
        assert_eq!(collar.kind, StrategyType::Collar);
        assert_eq!(collar.spot_leg.symbol, "AAPL");
        assert_eq!(collar.spot_leg.quantity, Positive::HUNDRED);
        assert_eq!(collar.spot_leg.cost_basis, pos_or_panic!(150.0));
        assert_eq!(collar.spot_leg.side, Side::Long);
        assert_eq!(collar.long_put.option.strike_price, pos_or_panic!(145.0));
        assert_eq!(collar.long_put.option.side, Side::Long);
        assert_eq!(collar.short_call.option.strike_price, pos_or_panic!(160.0));
        assert_eq!(collar.short_call.option.side, Side::Short);
    }

    #[test]
    fn test_collar_validation() {
        let collar = create_test_collar();
        assert!(collar.validate());
    }

    #[test]
    fn test_break_even_calculation() {
        let collar = create_test_collar();

        assert!(!collar.break_even_points.is_empty());
    }

    #[test]
    fn test_net_premium() {
        let collar = create_test_collar();
        let net_premium = collar.net_premium();

        // Call premium (3.00) - Put premium (2.50) = 0.50 credit per share,
        // on 100 shares (the legs are sized in shares since #731) = 50.00.
        assert!(net_premium.is_ok_and(|p| p > Decimal::ZERO)); // Credit collar
    }

    /// `net_premium` reads two `pub` premium fields. Setting one past the
    /// range `Collar::new` produced makes the product unrepresentable, and
    /// the getter reports it instead of aborting.
    #[test]
    fn test_collar_net_premium_overflowing_premium_is_reported() {
        let mut collar = create_test_collar();
        collar.short_call.premium = Positive::MAX;
        collar.short_call.option.quantity = Positive::TWO;

        assert!(collar.net_premium().is_err());
    }

    /// The two predicates over `net_premium` report the same failure rather
    /// than collapsing it into `false`, which would read as a definite
    /// classification.
    #[test]
    fn test_collar_credit_predicates_propagate_the_premium_overflow() {
        let mut collar = create_test_collar();
        collar.long_put.premium = Positive::MAX;
        collar.long_put.option.quantity = Positive::TWO;

        assert!(collar.is_zero_cost().is_err());
        assert!(collar.is_credit().is_err());
    }

    #[test]
    fn test_is_credit() {
        let collar = create_test_collar();
        assert_eq!(collar.is_credit().ok(), Some(true));
    }

    #[test]
    fn test_collar_width() {
        let collar = create_test_collar();
        assert_eq!(collar.collar_width(), pos_or_panic!(15.0)); // 160 - 145
    }

    #[test]
    fn test_put_strike() {
        let collar = create_test_collar();
        assert_eq!(collar.put_strike(), pos_or_panic!(145.0));
    }

    #[test]
    fn test_call_strike() {
        let collar = create_test_collar();
        assert_eq!(collar.call_strike(), pos_or_panic!(160.0));
    }

    #[test]
    fn test_underlying_price() {
        let collar = create_test_collar();
        assert_eq!(collar.underlying_price(), pos_or_panic!(150.0));
    }

    #[test]
    fn test_quantity() {
        let collar = create_test_collar();
        assert_eq!(collar.quantity(), Positive::HUNDRED);
    }

    #[test]
    fn test_profit_at_call_strike() {
        let collar = create_test_collar();

        // At call strike, should have maximum profit
        let profit = collar.calculate_profit_at(&pos_or_panic!(160.0)).unwrap();
        assert!(profit > Decimal::ZERO);
    }

    #[test]
    fn test_profit_above_call_strike() {
        let collar = create_test_collar();

        // Above call strike, profit is capped
        let profit_at_strike = collar.calculate_profit_at(&pos_or_panic!(160.0)).unwrap();
        let profit_above = collar.calculate_profit_at(&pos_or_panic!(180.0)).unwrap();

        // Both should be positive (profitable when price rises)
        assert!(profit_at_strike > Decimal::ZERO);
        assert!(profit_above > Decimal::ZERO);
    }

    #[test]
    fn test_loss_at_put_strike() {
        let collar = create_test_collar();

        // At put strike, should have defined P&L
        let _loss = collar.calculate_profit_at(&pos_or_panic!(145.0)).unwrap();
    }

    #[test]
    fn test_loss_below_put_strike() {
        let collar = create_test_collar();

        // Below put strike, loss is capped by the protective put
        let loss_at_strike = collar.calculate_profit_at(&pos_or_panic!(145.0)).unwrap();
        let loss_below = collar.calculate_profit_at(&pos_or_panic!(120.0)).unwrap();

        // Both values should be defined (the put provides protection)
        assert!(loss_at_strike != Decimal::MAX);
        assert!(loss_below != Decimal::MAX);
    }

    #[test]
    fn test_get_legs() {
        let collar = create_test_collar();
        let legs = collar.get_legs();

        assert_eq!(legs.len(), 3);
        assert!(legs[0].is_spot());
        assert!(legs[1].is_option());
        assert!(legs[2].is_option());
    }

    #[test]
    fn test_is_put_itm() {
        let collar = create_test_collar();

        assert!(collar.is_put_itm(pos_or_panic!(140.0))); // Below put strike
        assert!(!collar.is_put_itm(pos_or_panic!(145.0))); // At put strike
        assert!(!collar.is_put_itm(pos_or_panic!(150.0))); // Above put strike
    }

    #[test]
    fn test_is_call_itm() {
        let collar = create_test_collar();

        assert!(!collar.is_call_itm(pos_or_panic!(155.0))); // Below call strike
        assert!(!collar.is_call_itm(pos_or_panic!(160.0))); // At call strike
        assert!(collar.is_call_itm(pos_or_panic!(165.0))); // Above call strike
    }

    #[test]
    fn test_display() {
        let collar = create_test_collar();
        let display = format!("{}", collar);

        assert!(display.contains("Collar"));
        assert!(display.contains("Long"));
        assert!(display.contains("100"));
    }

    #[test]
    fn test_get_title() {
        let collar = create_test_collar();
        let title = collar.get_title();

        assert!(title.contains("Collar"));
        assert!(title.contains("AAPL"));
    }

    #[test]
    fn test_max_profit() {
        let collar = create_test_collar();
        let max_profit = collar.get_max_profit();

        assert!(max_profit.is_ok());
    }

    #[test]
    fn test_max_loss() {
        let collar = create_test_collar();
        let max_loss = collar.get_max_loss();

        assert!(max_loss.is_ok());
    }

    #[test]
    fn test_get_positions() {
        let collar = create_test_collar();
        let positions = collar.get_positions().unwrap();

        assert_eq!(positions.len(), 2);
    }
}
