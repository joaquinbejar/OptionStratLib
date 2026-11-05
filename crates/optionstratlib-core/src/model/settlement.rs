/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 5/11/26
******************************************************************************/

//! What expiry, exercise or assignment obliges an option position to pay,
//! receive or deliver. [`settlement_obligation`] documents the rules.

use crate::error::SettlementError;
use crate::model::Options;
use crate::model::types::{OptionStyle, OptionType, Side};
use chrono::{DateTime, Utc};
use positive::Positive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// How an exercised option is settled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum SettlementType {
    /// Only the intrinsic value changes hands, in cash.
    Cash,
    /// The underlying is delivered against the strike, paid in cash.
    Physical,
}

/// What ends the option position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum SettlementEvent {
    /// The contract reached expiry. It is exercised automatically when in the
    /// money and lapses otherwise; applies to both sides.
    Expiry,
    /// The holder exercises explicitly. Long positions only.
    Exercise,
    /// The writer is assigned explicitly. Short positions only.
    Assignment,
}

/// An observed settlement price of the underlying and where it came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct SettlementPrice {
    /// The price, per unit of the underlying.
    pub price: Positive,
    /// Who published the price, for example an exchange or an index name.
    /// Carried for the caller's audit trail; it does not affect the result.
    pub source: String,
    /// When the price was observed.
    pub observed_at: DateTime<Utc>,
}

/// How cash amounts are rounded: to `scale` decimal places with
/// [`rust_decimal::RoundingStrategy::MidpointNearestEven`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct CashRounding {
    /// Decimal places kept, for example 2 for cents. A value with fewer
    /// decimal places is left as it is.
    pub scale: u32,
}

/// The terms under which a settlement happens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct SettlementTerms {
    /// Cash or physical.
    pub settlement_type: SettlementType,
    /// What ends the position.
    pub event: SettlementEvent,
    /// The settlement price. See [`settlement_obligation`] for when it may be
    /// absent.
    pub price: Option<SettlementPrice>,
    /// The latest instant `price` may have been observed at.
    pub cutoff: DateTime<Utc>,
    /// How cash amounts are rounded.
    pub rounding: CashRounding,
}

/// What a settlement obliges a position to do, seen from that position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
pub struct SettlementObligation {
    /// The option quantity the event ends, in contracts: not multiplied by the
    /// contract size and the same for both sides.
    pub extinguished_quantity: Positive,
    /// Cash in account currency. Positive is received, negative is paid.
    pub cash: Decimal,
    /// Underlying in units of the underlying. Positive is received, negative
    /// is delivered.
    pub underlying: Decimal,
}

/// Computes what `terms` oblige the position in `option` to pay, receive or
/// deliver.
///
/// A pure function: it reads no clock and books nothing. It reads the option
/// type, side, call or put style, strike, quantity and contract size of
/// `option`; the `underlying_price` and `expiration_date` are not consulted,
/// because the price comes from `terms` and the caller decides when the event
/// happens.
/// Which short position gets assigned, whether an American option should be
/// exercised early, and the ledger and margin consequences of the result all
/// belong to the caller.
///
/// # Supported option types
///
/// [`OptionType::European`] and [`OptionType::American`]. Every other option
/// type is rejected with [`SettlementError::UnsupportedOptionType`].
///
/// # Rules
///
/// With strike `K`, settlement price `S`, `q` contracts and multiplier `m`
/// ([`Options::position_size`] is `q·m`):
///
/// | Event | Settlement | Condition | Cash | Underlying |
/// | --- | --- | --- | --- | --- |
/// | Expiry | either | at or out of the money | 0 | 0 |
/// | Expiry | cash | in the money | `\|S−K\|·q·m` | 0 |
/// | Expiry | physical | in the money | `K·q·m` | `q·m` |
/// | Exercise or assignment | cash | in the money | `\|S−K\|·q·m` | 0 |
/// | Exercise or assignment | cash | at or out of the money | 0 | 0 |
/// | Exercise or assignment | physical | any moneyness | `K·q·m` | `q·m` |
///
/// Every row extinguishes the `q` contracts. A position is in the money when it
/// is a call with `S > K` or a put with `S < K`; `S = K` is at the money. This
/// differs from [`Options::is_in_the_money`], which counts `S = K` as in the
/// money.
///
/// The signs say who receives, positive meaning the position receives:
///
/// | Position | Physical cash | Physical underlying | Cash settlement |
/// | --- | --- | --- | --- |
/// | Long call | pays | receives | receives |
/// | Short call | receives | delivers | pays |
/// | Long put | receives | delivers | receives |
/// | Short put | pays | receives | pays |
///
/// An explicit event is an instruction, so a physical one moves its flows even
/// out of the money, where a holder may still choose to exercise. A cash one
/// never pays more than the option's intrinsic value, which is never negative:
/// out of the money it extinguishes the contracts with no cashflow.
///
/// # Settlement price
///
/// A supplied price must have been observed at or before the cutoff, whatever
/// the event. An expiry needs a price to decide moneyness and a cash settlement
/// needs one to size its cashflow, so both fail without it. A physical explicit
/// exercise or assignment does not depend on the price, since strike and size
/// fix both flows, so it needs none.
///
/// # Rounding
///
/// `cash` is computed exactly from `S`, `K`, `q` and `m`, and rounded once at
/// the end, to [`CashRounding::scale`] decimal places with
/// [`rust_decimal::RoundingStrategy::MidpointNearestEven`] (banker's rounding).
/// No intermediate product is rounded, so it cannot create or break a tie. The
/// rounding is applied to the magnitude before the sign, so the two sides of a
/// contract get exactly opposite amounts. A value with no more decimal places
/// than the scale is left as it is. A `Decimal` holds at most 28 decimal places
/// and 96 bits, so a larger scale, or an amount too large for the scale, keeps
/// as many places as fit, still rounded from the exact value.
///
/// `underlying` is [`Options::position_size`], `q·m` at `Decimal` precision, with
/// no [`CashRounding`] applied. `extinguished_quantity` is `q` as given.
///
/// # Errors
///
/// - [`SettlementError::UnsupportedOptionType`] when `option` is neither European
///   nor American.
/// - [`SettlementError::EventSideMismatch`] for an exercise of a short
///   position or an assignment of a long one.
/// - [`SettlementError::PriceAfterCutoff`] when a supplied price was observed
///   after `terms.cutoff`.
/// - [`SettlementError::MissingPrice`] for an expiry or a cash settlement with
///   no price.
/// - [`SettlementError::ArithmeticOverflow`] when the position size or the
///   integer part of a cash amount leaves the representable range.
///
/// # Examples
///
/// ```rust
/// use chrono::{TimeZone, Utc};
/// use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
/// use optionstratlib_core::model::{
///     CashRounding, ExpirationDate, Options, Positive, SettlementEvent, SettlementPrice,
///     SettlementTerms, SettlementType, settlement_obligation,
/// };
/// use optionstratlib_core::pos_or_panic;
/// use rust_decimal_macros::dec;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let option = Options::new(
///     OptionType::European,
///     Side::Long,
///     "AAPL".to_string(),
///     pos_or_panic!(100.0),
///     ExpirationDate::Days(pos_or_panic!(30.0)),
///     pos_or_panic!(0.2),
///     pos_or_panic!(2.0),
///     pos_or_panic!(100.0),
///     dec!(0.05),
///     OptionStyle::Call,
///     Positive::ZERO,
///     None,
/// )
/// .with_contract_size(Positive::HUNDRED);
/// let cutoff = Utc
///     .timestamp_opt(1_700_000_000, 0)
///     .single()
///     .ok_or("not a valid instant")?;
/// let terms = SettlementTerms {
///     settlement_type: SettlementType::Physical,
///     event: SettlementEvent::Expiry,
///     price: Some(SettlementPrice {
///         price: pos_or_panic!(110.0),
///         source: "exchange close".to_string(),
///         observed_at: cutoff,
///     }),
///     cutoff,
///     rounding: CashRounding { scale: 2 },
/// };
///
/// let obligation = settlement_obligation(&option, &terms)?;
/// assert_eq!(obligation.extinguished_quantity, pos_or_panic!(2.0));
/// assert_eq!(obligation.underlying, dec!(200));
/// assert_eq!(obligation.cash, dec!(-20000));
/// # Ok(())
/// # }
/// ```
#[must_use = "the settlement obligation is the only output of this computation"]
pub fn settlement_obligation(
    option: &Options,
    terms: &SettlementTerms,
) -> Result<SettlementObligation, SettlementError> {
    check_option_type(&option.option_type)?;
    check_event_side(terms.event, option.side)?;
    let intrinsic = match observed_price(terms)? {
        Some(price) => Some(intrinsic_per_unit(
            option.option_style,
            option.strike_price,
            price,
        )?),
        None => None,
    };
    let size = option
        .position_size()
        .map_err(|error| overflow("position size", error))?
        .to_dec();
    let quantity = option.quantity.to_dec();
    let multiplier = option.contract_size.to_dec();

    let (cash, underlying) = match (terms.settlement_type, terms.event, intrinsic) {
        (SettlementType::Cash, _, None)
        | (SettlementType::Physical, SettlementEvent::Expiry, None) => {
            return Err(SettlementError::MissingPrice {
                settlement_type: terms.settlement_type,
                event: terms.event,
            });
        }
        (SettlementType::Cash, _, Some(intrinsic)) => {
            let amount = rounded_amount(
                "cash settlement",
                intrinsic,
                quantity,
                multiplier,
                terms.rounding,
            )?;
            (signed(amount, option.side == Side::Long), Decimal::ZERO)
        }
        (SettlementType::Physical, SettlementEvent::Expiry, Some(intrinsic))
            if intrinsic.is_zero() =>
        {
            (Decimal::ZERO, Decimal::ZERO)
        }
        (SettlementType::Physical, _, _) => {
            let strike = ExactDecimal::from_decimal(option.strike_price.to_dec())
                .ok_or_else(|| overflow("physical cash", "the strike is out of range"))?;
            let amount = rounded_amount(
                "physical cash",
                strike,
                quantity,
                multiplier,
                terms.rounding,
            )?;
            let receives_underlying = matches!(
                (option.side, option.option_style),
                (Side::Long, OptionStyle::Call) | (Side::Short, OptionStyle::Put)
            );
            (
                signed(amount, !receives_underlying),
                signed(size, receives_underlying),
            )
        }
    };

    Ok(SettlementObligation {
        extinguished_quantity: option.quantity,
        cash,
        underlying,
    })
}

/// Rejects every option type with no settlement rule.
fn check_option_type(option_type: &OptionType) -> Result<(), SettlementError> {
    match option_type {
        OptionType::European | OptionType::American => Ok(()),
        unsupported => Err(unsupported_option_type(unsupported)),
    }
}

/// Rejects an exercise of a short position and an assignment of a long one.
fn check_event_side(event: SettlementEvent, side: Side) -> Result<(), SettlementError> {
    match (event, side) {
        (SettlementEvent::Exercise, Side::Short) | (SettlementEvent::Assignment, Side::Long) => {
            Err(SettlementError::EventSideMismatch { event, side })
        }
        _ => Ok(()),
    }
}

/// The supplied price, once it is known to respect the cutoff.
fn observed_price(terms: &SettlementTerms) -> Result<Option<Positive>, SettlementError> {
    match &terms.price {
        Some(price) if price.observed_at > terms.cutoff => Err(SettlementError::PriceAfterCutoff {
            observed_at: price.observed_at,
            cutoff: terms.cutoff,
        }),
        Some(price) => Ok(Some(price.price)),
        None => Ok(None),
    }
}

/// The value of one unit of the option at `price`, exactly: `S−K` for a call
/// and `K−S` for a put, floored at zero. Zero means at or out of the money.
fn intrinsic_per_unit(
    style: OptionStyle,
    strike: Positive,
    price: Positive,
) -> Result<ExactDecimal, SettlementError> {
    let (minuend, subtrahend) = match style {
        OptionStyle::Call => (price.to_dec(), strike.to_dec()),
        OptionStyle::Put => (strike.to_dec(), price.to_dec()),
    };
    let scale = minuend.scale().max(subtrahend.scale());
    let aligned = |value: Decimal| {
        (value.scale()..scale).try_fold(
            ExactUint::from_u128(value.mantissa().unsigned_abs())?,
            |digits, _| digits.checked_mul_u32(10),
        )
    };
    let difference = aligned(minuend).zip(aligned(subtrahend)).ok_or_else(|| {
        overflow(
            "intrinsic value",
            format!("{minuend} minus {subtrahend} is out of range"),
        )
    })?;
    Ok(ExactDecimal {
        digits: difference
            .0
            .checked_sub(difference.1)
            .unwrap_or(ExactUint::ZERO),
        scale,
    })
}

/// `amount × quantity × multiplier`, computed exactly and rounded once by
/// `rounding`.
fn rounded_amount(
    operation: &'static str,
    amount: ExactDecimal,
    quantity: Decimal,
    multiplier: Decimal,
    rounding: CashRounding,
) -> Result<Decimal, SettlementError> {
    let out_of_range = || {
        overflow(
            operation,
            format!("{quantity} times {multiplier} leaves the range"),
        )
    };
    let digits = amount
        .digits
        .checked_mul_u128(quantity.mantissa().unsigned_abs())
        .and_then(|digits| digits.checked_mul_u128(multiplier.mantissa().unsigned_abs()))
        .ok_or_else(out_of_range)?;
    let scale = amount
        .scale
        .checked_add(quantity.scale())
        .and_then(|scale| scale.checked_add(multiplier.scale()))
        .ok_or_else(out_of_range)?;
    ExactDecimal { digits, scale }
        .rounded(rounding.scale)
        .ok_or_else(out_of_range)
}

/// Limbs of an [`ExactUint`]: 384 bits hold every amount computed here. An
/// intrinsic value aligned to scale 28 is below `2^190`, and the mantissas of
/// the quantity and the multiplier are each below `2^96`.
const EXACT_LIMBS: usize = 12;

/// The bits of one limb of an [`ExactUint`].
const LIMB_BITS: u32 = u32::BITS;

/// Selects the low limb of a `u128`.
const LIMB_MASK: u128 = 0xFFFF_FFFF;

/// The most decimal places a `Decimal` holds.
const MAX_DECIMAL_SCALE: u32 = 28;

/// An unsigned integer wide enough to hold a settlement amount without rounding,
/// least significant limb first. Every operation returns `None` on overflow
/// rather than truncating, which the bound on [`EXACT_LIMBS`] rules out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExactUint([u32; EXACT_LIMBS]);

impl ExactUint {
    const ZERO: Self = Self([0; EXACT_LIMBS]);

    fn from_u128(value: u128) -> Option<Self> {
        let mut limbs = [0; EXACT_LIMBS];
        let mut rest = value;
        for limb in limbs.iter_mut().take(4) {
            *limb = u32::try_from(rest & LIMB_MASK).ok()?;
            rest = rest.checked_shr(LIMB_BITS)?;
        }
        Some(Self(limbs))
    }

    /// The value, when it fits in 128 bits.
    fn to_u128(self) -> Option<u128> {
        if self.0.iter().skip(4).any(|limb| *limb != 0) {
            return None;
        }
        self.0.iter().take(4).rev().try_fold(0_u128, |value, limb| {
            Some(value.checked_shl(LIMB_BITS)? | u128::from(*limb))
        })
    }

    fn is_zero(self) -> bool {
        self == Self::ZERO
    }

    fn is_odd(self) -> bool {
        self.0.first().is_some_and(|limb| limb & 1 == 1)
    }

    fn checked_add(self, other: Self) -> Option<Self> {
        let mut limbs = self.0;
        let mut carry = 0_u64;
        for (limb, addend) in limbs.iter_mut().zip(other.0) {
            let sum = u64::from(*limb)
                .checked_add(u64::from(addend))?
                .checked_add(carry)?;
            (*limb, carry) = split_limb(sum)?;
        }
        (carry == 0).then_some(Self(limbs))
    }

    /// `None` when `other` is larger.
    fn checked_sub(self, other: Self) -> Option<Self> {
        let mut limbs = self.0;
        let mut borrow = false;
        for (limb, subtrahend) in limbs.iter_mut().zip(other.0) {
            let (partial, first) = limb.overflowing_sub(subtrahend);
            let (difference, second) = partial.overflowing_sub(u32::from(borrow));
            *limb = difference;
            borrow = first || second;
        }
        (!borrow).then_some(Self(limbs))
    }

    fn checked_mul_u32(self, factor: u32) -> Option<Self> {
        let mut limbs = self.0;
        let mut carry = 0_u64;
        for limb in &mut limbs {
            let product = u64::from(*limb)
                .checked_mul(u64::from(factor))?
                .checked_add(carry)?;
            (*limb, carry) = split_limb(product)?;
        }
        (carry == 0).then_some(Self(limbs))
    }

    /// Schoolbook multiplication, from the most significant limb of `factor`.
    fn checked_mul_u128(self, factor: u128) -> Option<Self> {
        let factor = Self::from_u128(factor)?;
        factor
            .0
            .iter()
            .take(4)
            .rev()
            .try_fold(Self::ZERO, |product, limb| {
                product
                    .shifted_one_limb()?
                    .checked_add(self.checked_mul_u32(*limb)?)
            })
    }

    fn shifted_one_limb(self) -> Option<Self> {
        let mut limbs = self.0;
        limbs.rotate_right(1);
        (limbs.first() == Some(&0)).then_some(Self(limbs))
    }

    /// The quotient and the remainder of a division by ten.
    fn divided_by_ten(self) -> Option<(Self, u32)> {
        let mut limbs = self.0;
        let mut remainder = 0_u64;
        for limb in limbs.iter_mut().rev() {
            let dividend = remainder.checked_shl(LIMB_BITS)? | u64::from(*limb);
            *limb = u32::try_from(dividend.checked_div(10)?).ok()?;
            remainder = dividend.checked_rem(10)?;
        }
        Some((Self(limbs), u32::try_from(remainder).ok()?))
    }
}

/// The low limb of `value` and what carries into the next one.
#[inline]
fn split_limb(value: u64) -> Option<(u32, u64)> {
    let low = u32::try_from(value & u64::from(u32::MAX)).ok()?;
    Some((low, value.checked_shr(LIMB_BITS)?))
}

/// An exact non-negative decimal: `digits / 10^scale`.
#[derive(Debug, Clone, Copy)]
struct ExactDecimal {
    digits: ExactUint,
    scale: u32,
}

impl ExactDecimal {
    fn from_decimal(value: Decimal) -> Option<Self> {
        Some(Self {
            digits: ExactUint::from_u128(value.mantissa().unsigned_abs())?,
            scale: value.scale(),
        })
    }

    fn is_zero(self) -> bool {
        self.digits.is_zero()
    }

    /// The value rounded half to even at `scale` decimal places, or at as many
    /// as a `Decimal` can hold, or `None` when its integer part does not fit.
    fn rounded(self, scale: u32) -> Option<Decimal> {
        let target = scale.min(MAX_DECIMAL_SCALE);
        let mut digits = self.digits;
        let mut current = self.scale;
        // The most significant digit dropped, and whether anything below it was
        // not zero: what tells a tie from a value on either side of it.
        let mut dropped = 0;
        let mut below_dropped = false;
        loop {
            if current <= target {
                let rounds_up = dropped > 5 || (dropped == 5 && (below_dropped || digits.is_odd()));
                let rounded = if rounds_up {
                    digits.checked_add(ExactUint::from_u128(1)?)?
                } else {
                    digits
                };
                let value = rounded
                    .to_u128()
                    .and_then(|mantissa| i128::try_from(mantissa).ok())
                    .and_then(|mantissa| Decimal::try_from_i128_with_scale(mantissa, current).ok());
                if value.is_some() {
                    return value;
                }
                if current == 0 {
                    return None;
                }
            }
            let (quotient, digit) = digits.divided_by_ten()?;
            below_dropped = below_dropped || dropped != 0;
            dropped = digit;
            digits = quotient;
            current = current.checked_sub(1)?;
        }
    }
}

/// `magnitude` with the sign of a flow the position receives or pays. A zero
/// stays unsigned, so a side with nothing to pay never shows `-0`.
#[inline]
fn signed(magnitude: Decimal, received: bool) -> Decimal {
    if received || magnitude.is_zero() {
        magnitude
    } else {
        -magnitude
    }
}

#[cold]
#[inline(never)]
fn unsupported_option_type(option_type: &OptionType) -> SettlementError {
    SettlementError::UnsupportedOptionType {
        option_type: option_type.clone(),
    }
}

#[cold]
#[inline(never)]
fn overflow(operation: &'static str, reason: impl Display) -> SettlementError {
    SettlementError::ArithmeticOverflow {
        operation,
        reason: reason.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ExpirationDate;
    use crate::model::types::{AsianAveragingType, BarrierType};
    use chrono::TimeDelta;
    use rust_decimal_macros::dec;
    use serde::de::DeserializeOwned;

    const STRIKE: Decimal = dec!(100);
    const IN_THE_MONEY_CALL: Decimal = dec!(110);
    const IN_THE_MONEY_PUT: Decimal = dec!(90);
    /// The contracts of every fixture.
    const QUANTITY: Decimal = dec!(2);
    /// The multiplier of every fixture.
    const MULTIPLIER: Decimal = dec!(100);
    /// `QUANTITY × MULTIPLIER`.
    const SIZE: Decimal = dec!(200);
    /// `|S−K| × SIZE`, the same at both in-the-money prices.
    const CASH_AMOUNT: Decimal = dec!(2000);
    /// `STRIKE × SIZE`.
    const PHYSICAL_CASH: Decimal = dec!(20000);

    fn pos(value: Decimal) -> Positive {
        Positive::new_decimal(value).unwrap_or(Positive::ZERO)
    }

    fn cutoff() -> DateTime<Utc> {
        DateTime::<Utc>::UNIX_EPOCH + TimeDelta::days(20_000)
    }

    fn contract(
        option_type: OptionType,
        side: Side,
        style: OptionStyle,
        strike: Decimal,
        quantity: Decimal,
        multiplier: Decimal,
    ) -> Options {
        Options::new(
            option_type,
            side,
            "TEST".to_string(),
            pos(strike),
            ExpirationDate::Days(Positive::ONE),
            pos(dec!(0.2)),
            pos(quantity),
            pos(strike),
            dec!(0.05),
            style,
            Positive::ZERO,
            None,
        )
        .with_contract_size(pos(multiplier))
    }

    fn fixture(side: Side, style: OptionStyle) -> Options {
        contract(
            OptionType::European,
            side,
            style,
            STRIKE,
            QUANTITY,
            MULTIPLIER,
        )
    }

    fn observed(price: Decimal) -> SettlementPrice {
        SettlementPrice {
            price: pos(price),
            source: "test close".to_string(),
            observed_at: cutoff(),
        }
    }

    fn terms(
        settlement_type: SettlementType,
        event: SettlementEvent,
        price: Option<Decimal>,
    ) -> SettlementTerms {
        SettlementTerms {
            settlement_type,
            event,
            price: price.map(observed),
            cutoff: cutoff(),
            rounding: CashRounding { scale: 2 },
        }
    }

    fn itm_price(style: OptionStyle) -> Decimal {
        match style {
            OptionStyle::Call => IN_THE_MONEY_CALL,
            OptionStyle::Put => IN_THE_MONEY_PUT,
        }
    }

    fn otm_price(style: OptionStyle) -> Decimal {
        match style {
            OptionStyle::Call => IN_THE_MONEY_PUT,
            OptionStyle::Put => IN_THE_MONEY_CALL,
        }
    }

    /// The event that executes a position on `side` explicitly.
    fn explicit_event(side: Side) -> SettlementEvent {
        match side {
            Side::Long => SettlementEvent::Exercise,
            Side::Short => SettlementEvent::Assignment,
        }
    }

    fn obligation(cash: Decimal, underlying: Decimal) -> SettlementObligation {
        SettlementObligation {
            extinguished_quantity: pos(QUANTITY),
            cash,
            underlying,
        }
    }

    /// The flows of a physical execution, from the table in `settlement_obligation`.
    fn physical_flows(side: Side, style: OptionStyle) -> SettlementObligation {
        match (side, style) {
            (Side::Long, OptionStyle::Call) => obligation(-PHYSICAL_CASH, SIZE),
            (Side::Short, OptionStyle::Call) => obligation(PHYSICAL_CASH, -SIZE),
            (Side::Long, OptionStyle::Put) => obligation(PHYSICAL_CASH, -SIZE),
            (Side::Short, OptionStyle::Put) => obligation(-PHYSICAL_CASH, SIZE),
        }
    }

    /// The flow of a cash execution in the money.
    fn cash_flows(side: Side) -> SettlementObligation {
        match side {
            Side::Long => obligation(CASH_AMOUNT, Decimal::ZERO),
            Side::Short => obligation(-CASH_AMOUNT, Decimal::ZERO),
        }
    }

    const SIDES: [Side; 2] = [Side::Long, Side::Short];
    const STYLES: [OptionStyle; 2] = [OptionStyle::Call, OptionStyle::Put];

    fn each_position(check: impl Fn(Side, OptionStyle)) {
        for side in SIDES {
            for style in STYLES {
                check(side, style);
            }
        }
    }

    fn settle(side: Side, style: OptionStyle, terms: &SettlementTerms) -> SettlementObligation {
        match settlement_obligation(&fixture(side, style), terms) {
            Ok(settled) => settled,
            Err(error) => panic!("{side:?} {style:?}: {error}"),
        }
    }

    #[test]
    fn test_settlement_expiry_cash_itm_pays_intrinsic_value() {
        each_position(|side, style| {
            let terms = terms(
                SettlementType::Cash,
                SettlementEvent::Expiry,
                Some(itm_price(style)),
            );
            assert_eq!(settle(side, style, &terms), cash_flows(side));
        });
    }

    #[test]
    fn test_settlement_expiry_physical_itm_moves_underlying_and_cash() {
        each_position(|side, style| {
            let terms = terms(
                SettlementType::Physical,
                SettlementEvent::Expiry,
                Some(itm_price(style)),
            );
            assert_eq!(settle(side, style, &terms), physical_flows(side, style));
        });
    }

    #[test]
    fn test_settlement_expiry_atm_extinguishes_with_no_flows() {
        each_position(|side, style| {
            for settlement_type in [SettlementType::Cash, SettlementType::Physical] {
                let terms = terms(settlement_type, SettlementEvent::Expiry, Some(STRIKE));
                assert_eq!(
                    settle(side, style, &terms),
                    obligation(Decimal::ZERO, Decimal::ZERO)
                );
            }
        });
    }

    #[test]
    fn test_settlement_expiry_otm_extinguishes_with_no_flows() {
        each_position(|side, style| {
            for settlement_type in [SettlementType::Cash, SettlementType::Physical] {
                let terms = terms(
                    settlement_type,
                    SettlementEvent::Expiry,
                    Some(otm_price(style)),
                );
                assert_eq!(
                    settle(side, style, &terms),
                    obligation(Decimal::ZERO, Decimal::ZERO)
                );
            }
        });
    }

    #[test]
    fn test_settlement_explicit_cash_itm_pays_intrinsic_value() {
        each_position(|side, style| {
            let terms = terms(
                SettlementType::Cash,
                explicit_event(side),
                Some(itm_price(style)),
            );
            assert_eq!(settle(side, style, &terms), cash_flows(side));
        });
    }

    #[test]
    fn test_settlement_explicit_cash_atm_has_no_cashflow() {
        each_position(|side, style| {
            let terms = terms(SettlementType::Cash, explicit_event(side), Some(STRIKE));
            assert_eq!(
                settle(side, style, &terms),
                obligation(Decimal::ZERO, Decimal::ZERO)
            );
        });
    }

    #[test]
    fn test_settlement_explicit_cash_otm_has_no_cashflow() {
        each_position(|side, style| {
            let terms = terms(
                SettlementType::Cash,
                explicit_event(side),
                Some(otm_price(style)),
            );
            assert_eq!(
                settle(side, style, &terms),
                obligation(Decimal::ZERO, Decimal::ZERO)
            );
        });
    }

    #[test]
    fn test_settlement_explicit_physical_executes_at_any_moneyness() {
        each_position(|side, style| {
            for price in [itm_price(style), STRIKE, otm_price(style)] {
                let terms = terms(SettlementType::Physical, explicit_event(side), Some(price));
                assert_eq!(settle(side, style, &terms), physical_flows(side, style));
            }
        });
    }

    #[test]
    fn test_settlement_explicit_physical_without_price_executes() {
        each_position(|side, style| {
            let terms = terms(SettlementType::Physical, explicit_event(side), None);
            assert_eq!(settle(side, style, &terms), physical_flows(side, style));
        });
    }

    #[test]
    fn test_settlement_multiplier_scales_flows_not_extinguished_quantity() {
        let option = contract(
            OptionType::American,
            Side::Long,
            OptionStyle::Call,
            STRIKE,
            dec!(3),
            dec!(10),
        );
        let physical = terms(
            SettlementType::Physical,
            SettlementEvent::Exercise,
            Some(IN_THE_MONEY_CALL),
        );
        assert_eq!(
            settlement_obligation(&option, &physical),
            Ok(SettlementObligation {
                extinguished_quantity: pos(dec!(3)),
                cash: dec!(-3000),
                underlying: dec!(30),
            })
        );
        let cash = terms(
            SettlementType::Cash,
            SettlementEvent::Exercise,
            Some(IN_THE_MONEY_CALL),
        );
        assert_eq!(
            settlement_obligation(&option, &cash),
            Ok(SettlementObligation {
                extinguished_quantity: pos(dec!(3)),
                cash: dec!(300),
                underlying: Decimal::ZERO,
            })
        );
    }

    #[test]
    fn test_settlement_unit_multiplier_is_the_quantity() {
        let option = contract(
            OptionType::European,
            Side::Short,
            OptionStyle::Put,
            STRIKE,
            dec!(5),
            Decimal::ONE,
        );
        let terms = terms(SettlementType::Physical, SettlementEvent::Assignment, None);
        assert_eq!(
            settlement_obligation(&option, &terms),
            Ok(SettlementObligation {
                extinguished_quantity: pos(dec!(5)),
                cash: dec!(-500),
                underlying: dec!(5),
            })
        );
    }

    #[test]
    fn test_settlement_cash_rounding_midpoint_goes_to_even() {
        for (price, expected) in [
            (dec!(100.125), dec!(0.12)),
            (dec!(100.135), dec!(0.14)),
            (dec!(100.126), dec!(0.13)),
        ] {
            for (side, sign) in [
                (Side::Long, Decimal::ONE),
                (Side::Short, Decimal::NEGATIVE_ONE),
            ] {
                let option = contract(
                    OptionType::European,
                    side,
                    OptionStyle::Call,
                    STRIKE,
                    Decimal::ONE,
                    Decimal::ONE,
                );
                let terms = terms(SettlementType::Cash, SettlementEvent::Expiry, Some(price));
                assert_eq!(
                    settlement_obligation(&option, &terms).map(|o| o.cash),
                    Ok(expected * sign)
                );
            }
        }
    }

    #[test]
    fn test_settlement_physical_cash_rounds_the_exact_triple_product() {
        // q·m is 1 − 1e-30, which a Decimal rounds to 1: going through it would
        // turn 101.01499…9 into the tie 101.015, and that tie into 101.02.
        let option = contract(
            OptionType::European,
            Side::Long,
            OptionStyle::Call,
            dec!(101.015),
            dec!(1.000000000000001),
            dec!(0.999999999999999),
        );
        let terms = terms(SettlementType::Physical, SettlementEvent::Exercise, None);

        assert!(matches!(
            settlement_obligation(&option, &terms),
            Ok(SettlementObligation { cash, .. }) if cash == dec!(-101.01)
        ));
    }

    #[test]
    fn test_settlement_cash_settlement_rounds_the_exact_triple_product() {
        // The same false tie through the intrinsic value: 1.015 × (1 − 1e-30).
        let option = contract(
            OptionType::European,
            Side::Long,
            OptionStyle::Call,
            dec!(100),
            dec!(1.000000000000001),
            dec!(0.999999999999999),
        );
        let terms = terms(
            SettlementType::Cash,
            SettlementEvent::Expiry,
            Some(dec!(101.015)),
        );

        assert!(matches!(
            settlement_obligation(&option, &terms),
            Ok(SettlementObligation { cash, .. }) if cash == dec!(1.01)
        ));
    }

    #[test]
    fn test_settlement_cash_rounds_the_exact_intrinsic_value() {
        // S − K is 1000000000000.13499…9, which a Decimal rounds to the tie
        // 1000000000000.135, and that tie to .14.
        let option = contract(
            OptionType::European,
            Side::Long,
            OptionStyle::Call,
            dec!(0.0000000000000000000000000001),
            dec!(1),
            dec!(1),
        );
        let terms = terms(
            SettlementType::Cash,
            SettlementEvent::Expiry,
            Some(dec!(1000000000000.135)),
        );

        assert!(matches!(
            settlement_obligation(&option, &terms),
            Ok(SettlementObligation { cash, .. }) if cash == dec!(1000000000000.13)
        ));
    }

    #[test]
    fn test_settlement_expiry_keeps_an_exact_intrinsic_value_above_zero() {
        // S − K is 1e-28 at a scale a Decimal holds, and it is in the money.
        let option = contract(
            OptionType::European,
            Side::Long,
            OptionStyle::Call,
            dec!(1),
            dec!(1),
            dec!(1),
        );
        let terms = terms(
            SettlementType::Physical,
            SettlementEvent::Expiry,
            Some(dec!(1.0000000000000000000000000001)),
        );

        assert!(matches!(
            settlement_obligation(&option, &terms),
            Ok(SettlementObligation { cash, underlying, .. })
                if cash == dec!(-1) && underlying == dec!(1)
        ));
    }

    #[test]
    fn test_settlement_rounding_a_true_tie_still_goes_to_even() {
        for (strike, expected) in [
            (dec!(101.015), dec!(-101.02)),
            (dec!(101.025), dec!(-101.02)),
        ] {
            let option = contract(
                OptionType::European,
                Side::Long,
                OptionStyle::Call,
                strike,
                dec!(1),
                dec!(1),
            );
            let terms = terms(SettlementType::Physical, SettlementEvent::Exercise, None);

            assert!(matches!(
                settlement_obligation(&option, &terms),
                Ok(SettlementObligation { cash, .. }) if cash == expected
            ));
        }
    }

    #[test]
    fn test_settlement_rounding_scale_above_decimal_precision_keeps_what_fits() {
        // The exact product has 30 decimal places, and a Decimal holds 28.
        let option = contract(
            OptionType::European,
            Side::Long,
            OptionStyle::Call,
            dec!(1),
            dec!(1.000000000000001),
            dec!(0.999999999999999),
        );
        let mut terms = terms(SettlementType::Physical, SettlementEvent::Exercise, None);
        terms.rounding = CashRounding { scale: 40 };

        assert!(matches!(
            settlement_obligation(&option, &terms),
            Ok(SettlementObligation { cash, .. }) if cash == dec!(-1)
        ));
    }

    #[test]
    fn test_settlement_large_amount_keeps_the_places_that_fit() {
        // 8e18 × 1.0000000001 at scale 10 needs more than 96 bits; at scale 9
        // it fits, rounded from the exact value.
        let option = contract(
            OptionType::European,
            Side::Long,
            OptionStyle::Call,
            dec!(8000000000000000000),
            dec!(1.0000000001),
            dec!(1),
        );
        let mut terms = terms(SettlementType::Physical, SettlementEvent::Exercise, None);
        terms.rounding = CashRounding { scale: 28 };

        assert!(matches!(
            settlement_obligation(&option, &terms),
            Ok(SettlementObligation { cash, .. }) if cash == dec!(-8000000000800000000)
        ));
    }

    /// `ExactUint::from_u128` for a test: every `u128` fits, so a zero here can
    /// only fail the comparison that follows.
    fn exact(value: u128) -> ExactUint {
        ExactUint::from_u128(value).unwrap_or(ExactUint::ZERO)
    }

    #[test]
    fn test_exact_uint_arithmetic_matches_u128() {
        let left: u128 = 0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF;
        let right: u128 = 0xFFFF_FFFF;
        assert_eq!(exact(u128::MAX).to_u128(), Some(u128::MAX));
        assert_eq!(
            exact(left)
                .checked_mul_u128(right)
                .and_then(ExactUint::to_u128),
            left.checked_mul(right)
        );
        assert_eq!(
            exact(left)
                .checked_add(exact(right))
                .and_then(ExactUint::to_u128),
            left.checked_add(right)
        );
        assert_eq!(
            exact(left)
                .checked_sub(exact(right))
                .and_then(ExactUint::to_u128),
            left.checked_sub(right)
        );
        assert_eq!(exact(right).checked_sub(exact(left)), None);
        assert_eq!(
            exact(left)
                .divided_by_ten()
                .and_then(|(quotient, _)| quotient.to_u128()),
            Some(left / 10)
        );
        assert_eq!(
            exact(left)
                .divided_by_ten()
                .map(|(_, remainder)| u128::from(remainder)),
            Some(left % 10)
        );
    }

    #[test]
    fn test_exact_uint_holds_the_largest_amount_and_refuses_more() {
        // The bound behind EXACT_LIMBS: below 2^190 × 2^96 × 2^96.
        let mantissa: u128 = (1 << 96) - 1;
        let largest = (0..28)
            .try_fold(exact(mantissa), |digits, _| digits.checked_mul_u32(10))
            .and_then(|digits| digits.checked_mul_u128(mantissa))
            .and_then(|digits| digits.checked_mul_u128(mantissa));
        assert!(largest.is_some());
        assert_eq!(
            largest.and_then(|digits| digits.checked_mul_u128(1 << 96)),
            None
        );
    }

    #[test]
    fn test_settlement_rounding_applies_to_physical_cash_only() {
        let option = contract(
            OptionType::European,
            Side::Long,
            OptionStyle::Call,
            dec!(33.335),
            dec!(0.125),
            dec!(8),
        );
        let terms = terms(SettlementType::Physical, SettlementEvent::Exercise, None);
        assert_eq!(
            settlement_obligation(&option, &terms),
            Ok(SettlementObligation {
                extinguished_quantity: pos(dec!(0.125)),
                cash: dec!(-33.34),
                underlying: Decimal::ONE,
            })
        );
    }

    #[test]
    fn test_settlement_rounding_scale_decides_the_decimal_places() {
        let option = contract(
            OptionType::European,
            Side::Long,
            OptionStyle::Call,
            STRIKE,
            Decimal::ONE,
            Decimal::ONE,
        );
        let mut terms = terms(
            SettlementType::Cash,
            SettlementEvent::Expiry,
            Some(dec!(101.5)),
        );
        terms.rounding = CashRounding { scale: 0 };
        assert_eq!(
            settlement_obligation(&option, &terms).map(|o| o.cash),
            Ok(dec!(2))
        );
        terms.rounding = CashRounding { scale: 40 };
        assert_eq!(
            settlement_obligation(&option, &terms).map(|o| o.cash),
            Ok(dec!(1.5))
        );
    }

    #[test]
    fn test_settlement_no_flow_is_never_negative_zero() {
        let terms = terms(SettlementType::Cash, SettlementEvent::Expiry, Some(STRIKE));
        let result = settlement_obligation(&fixture(Side::Short, OptionStyle::Call), &terms);
        assert!(matches!(
            result,
            Ok(SettlementObligation { cash, underlying, .. })
                if !cash.is_sign_negative() && !underlying.is_sign_negative()
        ));
    }

    #[test]
    fn test_settlement_cash_rounded_to_zero_is_not_negative_zero() {
        let option = contract(
            OptionType::European,
            Side::Short,
            OptionStyle::Call,
            STRIKE,
            Decimal::ONE,
            Decimal::ONE,
        );
        let terms = terms(
            SettlementType::Cash,
            SettlementEvent::Expiry,
            Some(dec!(100.001)),
        );
        let cash = settlement_obligation(&option, &terms).map(|o| o.cash);
        assert_eq!(cash, Ok(Decimal::ZERO));
        assert!(matches!(cash, Ok(value) if !value.is_sign_negative()));
    }

    #[test]
    fn test_settlement_zero_quantity_has_no_flows() {
        let option = contract(
            OptionType::European,
            Side::Long,
            OptionStyle::Call,
            STRIKE,
            Decimal::ZERO,
            MULTIPLIER,
        );
        let terms = terms(
            SettlementType::Physical,
            SettlementEvent::Expiry,
            Some(IN_THE_MONEY_CALL),
        );
        let result = settlement_obligation(&option, &terms);
        assert!(matches!(
            result,
            Ok(SettlementObligation { extinguished_quantity, cash, underlying })
                if extinguished_quantity.is_zero() && cash.is_zero() && underlying.is_zero()
        ));
    }

    #[test]
    fn test_settlement_american_settles_like_european() {
        for settlement_type in [SettlementType::Cash, SettlementType::Physical] {
            let terms = terms(
                settlement_type,
                SettlementEvent::Exercise,
                Some(IN_THE_MONEY_CALL),
            );
            let american = contract(
                OptionType::American,
                Side::Long,
                OptionStyle::Call,
                STRIKE,
                QUANTITY,
                MULTIPLIER,
            );
            assert_eq!(
                settlement_obligation(&american, &terms),
                settlement_obligation(&fixture(Side::Long, OptionStyle::Call), &terms)
            );
        }
    }

    #[test]
    fn test_settlement_unsupported_option_type_is_rejected() {
        let terms = terms(
            SettlementType::Cash,
            SettlementEvent::Expiry,
            Some(IN_THE_MONEY_CALL),
        );
        for option_type in [
            OptionType::Bermuda {
                exercise_dates: vec![Positive::ONE],
            },
            OptionType::Asian {
                averaging_type: AsianAveragingType::Arithmetic,
            },
            OptionType::Barrier {
                barrier_type: BarrierType::UpAndIn,
                barrier_level: pos(dec!(120)),
                rebate: None,
            },
            OptionType::Cliquet {
                reset_dates: vec![Positive::ONE],
            },
        ] {
            let option = contract(
                option_type.clone(),
                Side::Long,
                OptionStyle::Call,
                STRIKE,
                QUANTITY,
                MULTIPLIER,
            );
            assert_eq!(
                settlement_obligation(&option, &terms),
                Err(SettlementError::UnsupportedOptionType { option_type })
            );
        }
    }

    #[test]
    fn test_settlement_expiry_without_price_is_missing_price() {
        each_position(|side, style| {
            for settlement_type in [SettlementType::Cash, SettlementType::Physical] {
                let terms = terms(settlement_type, SettlementEvent::Expiry, None);
                assert_eq!(
                    settlement_obligation(&fixture(side, style), &terms),
                    Err(SettlementError::MissingPrice {
                        settlement_type,
                        event: SettlementEvent::Expiry,
                    })
                );
            }
        });
    }

    #[test]
    fn test_settlement_explicit_cash_without_price_is_missing_price() {
        each_position(|side, style| {
            let event = explicit_event(side);
            let terms = terms(SettlementType::Cash, event, None);
            assert_eq!(
                settlement_obligation(&fixture(side, style), &terms),
                Err(SettlementError::MissingPrice {
                    settlement_type: SettlementType::Cash,
                    event,
                })
            );
        });
    }

    #[test]
    fn test_settlement_price_after_cutoff_is_rejected_for_every_event() {
        let observed_at = cutoff() + TimeDelta::seconds(1);
        each_position(|side, style| {
            for event in [SettlementEvent::Expiry, explicit_event(side)] {
                for settlement_type in [SettlementType::Cash, SettlementType::Physical] {
                    let mut terms = terms(settlement_type, event, Some(STRIKE));
                    if let Some(price) = terms.price.as_mut() {
                        price.observed_at = observed_at;
                    }
                    assert_eq!(
                        settlement_obligation(&fixture(side, style), &terms),
                        Err(SettlementError::PriceAfterCutoff {
                            observed_at,
                            cutoff: cutoff(),
                        })
                    );
                }
            }
        });
    }

    #[test]
    fn test_settlement_price_observed_at_or_before_cutoff_is_accepted() {
        for offset in [TimeDelta::zero(), TimeDelta::seconds(-1)] {
            let mut terms = terms(
                SettlementType::Cash,
                SettlementEvent::Expiry,
                Some(IN_THE_MONEY_CALL),
            );
            if let Some(price) = terms.price.as_mut() {
                price.observed_at = cutoff() + offset;
            }
            assert_eq!(
                settle(Side::Long, OptionStyle::Call, &terms),
                cash_flows(Side::Long)
            );
        }
    }

    #[test]
    fn test_settlement_exercise_of_short_is_event_side_mismatch() {
        for style in STYLES {
            let terms = terms(SettlementType::Physical, SettlementEvent::Exercise, None);
            assert_eq!(
                settlement_obligation(&fixture(Side::Short, style), &terms),
                Err(SettlementError::EventSideMismatch {
                    event: SettlementEvent::Exercise,
                    side: Side::Short,
                })
            );
        }
    }

    #[test]
    fn test_settlement_assignment_of_long_is_event_side_mismatch() {
        for style in STYLES {
            let terms = terms(SettlementType::Physical, SettlementEvent::Assignment, None);
            assert_eq!(
                settlement_obligation(&fixture(Side::Long, style), &terms),
                Err(SettlementError::EventSideMismatch {
                    event: SettlementEvent::Assignment,
                    side: Side::Long,
                })
            );
        }
    }

    fn overflow_operation(
        result: Result<SettlementObligation, SettlementError>,
    ) -> Option<&'static str> {
        match result {
            Err(SettlementError::ArithmeticOverflow { operation, .. }) => Some(operation),
            _ => None,
        }
    }

    #[test]
    fn test_settlement_position_size_overflow_is_an_error() {
        let option = contract(
            OptionType::European,
            Side::Long,
            OptionStyle::Call,
            STRIKE,
            Decimal::MAX,
            dec!(2),
        );
        let terms = terms(SettlementType::Physical, SettlementEvent::Exercise, None);
        assert_eq!(
            overflow_operation(settlement_obligation(&option, &terms)),
            Some("position size")
        );
    }

    #[test]
    fn test_settlement_physical_cash_overflow_is_an_error() {
        let option = contract(
            OptionType::European,
            Side::Long,
            OptionStyle::Call,
            dec!(10000000000000000000000000000),
            dec!(100),
            Decimal::ONE,
        );
        let terms = terms(SettlementType::Physical, SettlementEvent::Exercise, None);
        assert_eq!(
            overflow_operation(settlement_obligation(&option, &terms)),
            Some("physical cash")
        );
    }

    #[test]
    fn test_settlement_cash_overflow_is_an_error() {
        let option = contract(
            OptionType::European,
            Side::Long,
            OptionStyle::Put,
            dec!(10000000000000000000000000000),
            dec!(100),
            Decimal::ONE,
        );
        let terms = terms(
            SettlementType::Cash,
            SettlementEvent::Expiry,
            Some(Decimal::ONE),
        );
        assert_eq!(
            overflow_operation(settlement_obligation(&option, &terms)),
            Some("cash settlement")
        );
    }

    #[test]
    fn test_settlement_types_serialize_as_snake_case() {
        assert_eq!(
            serde_json::to_string(&SettlementType::Physical)
                .ok()
                .as_deref(),
            Some("\"physical\"")
        );
        assert_eq!(
            serde_json::to_string(&SettlementEvent::Assignment)
                .ok()
                .as_deref(),
            Some("\"assignment\"")
        );
    }

    #[test]
    fn test_settlement_terms_round_trip_through_json() {
        let terms = terms(
            SettlementType::Cash,
            SettlementEvent::Expiry,
            Some(IN_THE_MONEY_CALL),
        );
        let json = serde_json::to_string(&terms);
        assert!(json.is_ok());
        let parsed = json.and_then(|text| serde_json::from_str::<SettlementTerms>(&text));
        assert_eq!(parsed.ok(), Some(terms));
    }

    #[test]
    fn test_settlement_terms_reject_unknown_fields() {
        let terms = terms(
            SettlementType::Cash,
            SettlementEvent::Expiry,
            Some(IN_THE_MONEY_CALL),
        );
        assert_rejects_unknown_field(&CashRounding { scale: 2 });
        assert_rejects_unknown_field(&observed(IN_THE_MONEY_CALL));
        assert_rejects_unknown_field(&terms);
    }

    /// Asserts that `value` parses back from JSON and stops parsing once its
    /// object gains a field the type does not declare.
    fn assert_rejects_unknown_field<T>(value: &T)
    where
        T: Serialize + DeserializeOwned,
    {
        let mut json = serde_json::to_value(value).unwrap_or_default();
        assert!(serde_json::from_value::<T>(json.clone()).is_ok());
        if let Some(fields) = json.as_object_mut() {
            fields.insert("unexpected".to_string(), serde_json::Value::Bool(true));
        }
        let error = serde_json::from_value::<T>(json)
            .err()
            .map(|e| e.to_string());
        assert!(
            error
                .as_deref()
                .is_some_and(|text| text.contains("unknown field")),
            "{error:?}"
        );
    }
}
