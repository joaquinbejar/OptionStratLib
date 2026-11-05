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
use rust_decimal::{Decimal, RoundingStrategy};
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
/// [`RoundingStrategy::MidpointNearestEven`].
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
/// Only `cash` is rounded, to [`CashRounding::scale`] decimal places with
/// [`RoundingStrategy::MidpointNearestEven`] (banker's rounding). The rounding
/// is applied to the magnitude before the sign, so the two sides of a contract
/// get exactly opposite amounts. A value with no more decimal places than the
/// scale is left as it is. `underlying` and `extinguished_quantity` are never
/// rounded.
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
/// - [`SettlementError::ArithmeticOverflow`] when the position size or a cash
///   amount leaves the representable range.
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

    let (cash, underlying) = match (terms.settlement_type, terms.event, intrinsic) {
        (SettlementType::Cash, _, None)
        | (SettlementType::Physical, SettlementEvent::Expiry, None) => {
            return Err(SettlementError::MissingPrice {
                settlement_type: terms.settlement_type,
                event: terms.event,
            });
        }
        (SettlementType::Cash, _, Some(intrinsic)) => {
            let amount = rounded_product("cash settlement", intrinsic, size, terms.rounding)?;
            (signed(amount, option.side == Side::Long), Decimal::ZERO)
        }
        (SettlementType::Physical, SettlementEvent::Expiry, Some(intrinsic))
            if intrinsic.is_zero() =>
        {
            (Decimal::ZERO, Decimal::ZERO)
        }
        (SettlementType::Physical, _, _) => {
            let strike = option.strike_price.to_dec();
            let amount = rounded_product("physical cash", strike, size, terms.rounding)?;
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

/// The value of one unit of the option at `price`: `S−K` for a call and `K−S`
/// for a put, floored at zero. Zero means at or out of the money.
fn intrinsic_per_unit(
    style: OptionStyle,
    strike: Positive,
    price: Positive,
) -> Result<Decimal, SettlementError> {
    let (minuend, subtrahend) = match style {
        OptionStyle::Call => (price.to_dec(), strike.to_dec()),
        OptionStyle::Put => (strike.to_dec(), price.to_dec()),
    };
    let difference = minuend.checked_sub(subtrahend).ok_or_else(|| {
        overflow(
            "intrinsic value",
            format!("{minuend} minus {subtrahend} is out of range"),
        )
    })?;
    Ok(difference.max(Decimal::ZERO))
}

/// `left × right` rounded by `rounding`.
fn rounded_product(
    operation: &'static str,
    left: Decimal,
    right: Decimal,
    rounding: CashRounding,
) -> Result<Decimal, SettlementError> {
    let product = left
        .checked_mul(right)
        .ok_or_else(|| overflow(operation, format!("{left} times {right} is out of range")))?;
    Ok(product.round_dp_with_strategy(rounding.scale, RoundingStrategy::MidpointNearestEven))
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
