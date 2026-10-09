/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 12/01/26
******************************************************************************/

//! # Shared Strategy Traits and Utilities
//!
//! This module provides shared traits for different strategy categories,
//! reducing code duplication across strategy implementations.
//!
//! ## Strategy Categories
//!
//! - **Spread strategies**: Two-leg strategies with upper and lower strikes
//! - **Butterfly strategies**: Three-strike strategies with wings and body
//! - **Condor strategies**: Four-strike strategies
//! - **Straddle/Strangle strategies**: Volatility-based strategies
//!
//! ## Usage
//!
//! Strategies implement these traits to gain access to common calculations
//! and reduce boilerplate code.

use crate::error::strategies::{ProfitLossErrorKind, StrategyError};
use crate::strategies::base::{BreakEvenable, Strategies, Validable};
use optionstratlib_analytics::analytics::ProfitLossRange;
use optionstratlib_analytics::analytics::probability::VolatilityAdjustment;
use optionstratlib_analytics::error::probability::ProbabilityError;
use optionstratlib_analytics::pnl::utils::PnL;
use optionstratlib_core::error::position::PositionError;
use optionstratlib_core::model::Options;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::{d_div, d_mul, d_sub};
use optionstratlib_core::model::leg::SpotPosition;
use optionstratlib_core::model::leg::traits::LegAble;
use optionstratlib_core::model::position::Position;
use optionstratlib_core::model::types::{OptionStyle, Side};
use optionstratlib_pricing::error::PricingError;
use rust_decimal::Decimal;

/// Marks a spot leg to market at `price`, in the convention
/// `Position::calculate_pnl` uses for an option leg (#728).
///
/// - `unrealized` is the change in the leg's value since entry,
///   `(price - cost_basis) * quantity`, signed by the side. A share has no
///   time value, so this is the same at any date and at expiry.
/// - `realized` is the entry cash flow, `initial_income - initial_costs`.
/// - `initial_costs` is [`LegAble::total_cost`]: the purchase and both fees
///   for a long leg, the fees alone for a short one; `initial_income` is the
///   short sale's proceeds, zero for a long leg.
///
/// # Errors
///
/// Returns [`PricingError`] when a cost, an income or the value change
/// leaves the `Decimal` or `Positive` range.
pub(crate) fn spot_leg_mark_to_market(
    spot: &SpotPosition,
    price: &Positive,
) -> Result<PnL, PricingError> {
    let change = d_mul(
        d_sub(
            price.to_dec(),
            spot.cost_basis.to_dec(),
            "shared::spot_leg_mark_to_market::price_change",
        )?,
        spot.quantity.to_dec(),
        "shared::spot_leg_mark_to_market::value_change",
    )?;
    let (unrealized, initial_income) = match spot.side {
        Side::Long => (change, Positive::ZERO),
        // `Decimal` is symmetric, so negating a representable value is
        // itself representable.
        Side::Short => (-change, spot.quantity.checked_mul(&spot.cost_basis)?),
    };
    let initial_costs = spot.total_cost()?;
    let realized = d_sub(
        initial_income.to_dec(),
        initial_costs.to_dec(),
        "shared::spot_leg_mark_to_market::realized",
    )?;
    Ok(PnL::new(
        Some(realized),
        Some(unrealized),
        initial_costs,
        initial_income,
        spot.date,
    ))
}

/// Trait for vertical spread strategies (two legs with different strikes).
///
/// Vertical spreads involve buying and selling options of the same type
/// (calls or puts) with different strike prices but the same expiration.
///
/// # Examples
///
/// - Bull Call Spread
/// - Bear Call Spread
/// - Bull Put Spread
/// - Bear Put Spread
pub trait SpreadStrategy {
    /// Returns the lower strike price of the spread.
    fn lower_strike(&self) -> Positive;

    /// Returns the upper strike price of the spread.
    fn upper_strike(&self) -> Positive;

    /// Returns the spread width (difference between strikes).
    ///
    /// # Returns
    ///
    /// The difference between upper and lower strike prices.
    ///
    /// # Errors
    ///
    /// Returns [`StrategyError::PositiveError`] when the upper strike sits
    /// below the lower one, which the public legs of a strategy allow.
    fn spread_width(&self) -> Result<Positive, StrategyError> {
        Ok(self.upper_strike().checked_sub(&self.lower_strike())?)
    }

    /// Returns the short leg position.
    fn short_leg(&self) -> &Position;

    /// Returns the long leg position.
    fn long_leg(&self) -> &Position;
}

/// Trait for butterfly-type strategies (three strikes with wings and body).
///
/// Butterfly strategies involve three strike prices where the middle strike
/// (body) has twice the position size of the outer strikes (wings).
///
/// # Examples
///
/// - Long Call Butterfly
/// - Short Call Butterfly
/// - Iron Butterfly
pub trait ButterflyStrategy {
    /// Returns the wing strikes (lower, upper).
    fn wing_strikes(&self) -> (Positive, Positive);

    /// Returns the body (middle) strike.
    fn body_strike(&self) -> Positive;

    /// Returns the wing width (distance from body to each wing).
    ///
    /// # Returns
    ///
    /// The distance from the body strike to either wing.
    ///
    /// # Errors
    ///
    /// Returns [`StrategyError::PositiveError`] when the upper wing sits
    /// below the lower one.
    fn wing_width(&self) -> Result<Positive, StrategyError> {
        let (lower, upper) = self.wing_strikes();
        Ok(upper.checked_sub(&lower)?.checked_div_dec(Decimal::TWO)?)
    }

    /// Returns all positions in the butterfly.
    fn get_butterfly_positions(&self) -> Vec<&Position>;
}

/// Trait for condor-type strategies (four strikes).
///
/// Condor strategies involve four strike prices, typically with two
/// inner strikes (short positions) and two outer strikes (long positions).
///
/// # Examples
///
/// - Iron Condor
/// - Long Call Condor
/// - Long Put Condor
pub trait CondorStrategy {
    /// Returns all four strikes (lowest to highest).
    fn strikes(&self) -> (Positive, Positive, Positive, Positive);

    /// Returns the inner spread width (between the two middle strikes).
    ///
    /// # Errors
    ///
    /// Returns [`StrategyError::PositiveError`] when the upper middle strike
    /// sits below the lower middle one.
    fn inner_width(&self) -> Result<Positive, StrategyError> {
        let (_, lower_mid, upper_mid, _) = self.strikes();
        Ok(upper_mid.checked_sub(&lower_mid)?)
    }

    /// Returns the outer spread width (total width of the condor).
    ///
    /// # Errors
    ///
    /// Returns [`StrategyError::PositiveError`] when the highest strike sits
    /// below the lowest one.
    fn outer_width(&self) -> Result<Positive, StrategyError> {
        let (lowest, _, _, highest) = self.strikes();
        Ok(highest.checked_sub(&lowest)?)
    }

    /// Returns the put spread width (lower wing).
    ///
    /// # Errors
    ///
    /// Returns [`StrategyError::PositiveError`] when the lower middle strike
    /// sits below the lowest one.
    fn put_spread_width(&self) -> Result<Positive, StrategyError> {
        let (lowest, lower_mid, _, _) = self.strikes();
        Ok(lower_mid.checked_sub(&lowest)?)
    }

    /// Returns the call spread width (upper wing).
    ///
    /// # Errors
    ///
    /// Returns [`StrategyError::PositiveError`] when the highest strike sits
    /// below the upper middle one.
    fn call_spread_width(&self) -> Result<Positive, StrategyError> {
        let (_, _, upper_mid, highest) = self.strikes();
        Ok(highest.checked_sub(&upper_mid)?)
    }

    /// Returns all positions in the condor.
    fn get_condor_positions(&self) -> Vec<&Position>;
}

/// Trait for straddle-type strategies (same strike for call and put).
///
/// Straddle strategies involve buying or selling both a call and put
/// at the same strike price and expiration.
///
/// # Examples
///
/// - Long Straddle
/// - Short Straddle
pub trait StraddleStrategy {
    /// Returns the strike price (same for both call and put).
    fn strike(&self) -> Positive;

    /// Returns the call position.
    fn call_position(&self) -> &Position;

    /// Returns the put position.
    fn put_position(&self) -> &Position;

    /// Returns true if this is a long straddle (buying both options).
    fn is_long(&self) -> bool;
}

/// Trait for strangle-type strategies (different strikes for call and put).
///
/// Strangle strategies involve buying or selling a call and put with
/// different strike prices but the same expiration.
///
/// # Examples
///
/// - Long Strangle
/// - Short Strangle
pub trait StrangleStrategy {
    /// Returns the call strike price.
    fn call_strike(&self) -> Positive;

    /// Returns the put strike price.
    fn put_strike(&self) -> Positive;

    /// Returns the strangle width (distance between strikes).
    ///
    /// # Errors
    ///
    /// Returns [`StrategyError::PositiveError`] when the call strike sits
    /// below the put strike.
    fn strangle_width(&self) -> Result<Positive, StrategyError> {
        Ok(self.call_strike().checked_sub(&self.put_strike())?)
    }

    /// Returns the call position.
    fn call_position(&self) -> &Position;

    /// Returns the put position.
    fn put_position(&self) -> &Position;

    /// Returns true if this is a long strangle (buying both options).
    fn is_long(&self) -> bool;
}

/// Helper function to calculate break-even for a credit spread.
///
/// # Arguments
///
/// * `short_strike` - Strike price of the short option
/// * `net_credit` - Net credit received from the spread
/// * `is_call_spread` - True if this is a call spread, false for put spread
///
/// # Returns
///
/// The break-even price for the spread.
///
/// # Errors
///
/// Returns [`StrategyError::PositiveError`] when the call break-even
/// overflows `Positive`, or when the net credit of a put spread is larger
/// than its short strike, which leaves no break-even above zero.
pub fn credit_spread_break_even(
    short_strike: Positive,
    net_credit: Positive,
    is_call_spread: bool,
) -> Result<Positive, StrategyError> {
    if is_call_spread {
        Ok(short_strike.checked_add(&net_credit)?)
    } else {
        Ok(short_strike.checked_sub(&net_credit)?)
    }
}

/// Helper function to calculate break-even for a debit spread.
///
/// # Arguments
///
/// * `long_strike` - Strike price of the long option
/// * `net_debit` - Net debit paid for the spread
/// * `is_call_spread` - True if this is a call spread, false for put spread
///
/// # Returns
///
/// The break-even price for the spread.
///
/// # Errors
///
/// Returns [`StrategyError::PositiveError`] when the call break-even
/// overflows `Positive`, or when the net debit of a put spread is larger
/// than its long strike, which leaves no break-even above zero.
pub fn debit_spread_break_even(
    long_strike: Positive,
    net_debit: Positive,
    is_call_spread: bool,
) -> Result<Positive, StrategyError> {
    if is_call_spread {
        Ok(long_strike.checked_add(&net_debit)?)
    } else {
        Ok(long_strike.checked_sub(&net_debit)?)
    }
}

/// Helper function to calculate the profit ratio.
///
/// # Arguments
///
/// * `max_profit` - Maximum profit of the strategy
/// * `max_loss` - Maximum loss of the strategy
///
/// # Returns
///
/// The profit ratio as a percentage, or an error if calculation fails.
///
/// # Errors
///
/// Returns [`StrategyError::OperationError`] when the ratio overflows
/// `Decimal`: a profit near `Positive::MAX` over a loss at the smallest
/// representable scale. The sentinel cases where `max_loss` or
/// `max_profit` is zero return `Ok`.
pub fn calculate_profit_ratio(
    max_profit: Positive,
    max_loss: Positive,
) -> Result<Decimal, StrategyError> {
    if max_loss == Positive::ZERO {
        return Ok(Decimal::MAX);
    }
    if max_profit == Positive::ZERO {
        return Ok(Decimal::ZERO);
    }
    let ratio = d_div(
        max_profit.to_dec(),
        max_loss.to_dec(),
        "calculate_profit_ratio",
    )?;
    Ok(d_mul(
        ratio,
        Decimal::ONE_HUNDRED,
        "calculate_profit_ratio",
    )?)
}

/// Whether `error` is a strategy reporting the sign of its own extreme: a
/// best case that still loses (`MaxProfitError`) or a worst case that still
/// gains (`MaxLossError`).
///
/// Those two kinds carry only sign reports. Legs that do not form the
/// strategy (an inverted vertical) are `StrategyError::InvalidStrategy`
/// (#803) and propagate like any other error.
///
/// The profit area and the profit ratio read such a strategy as having no
/// profit, or no loss, to measure: zero, with zero loss shown as an
/// unbounded ratio. That is the meaning they have always given it; every
/// other error is propagated (#788).
#[inline]
pub(crate) fn is_extreme_sign_error(error: &StrategyError) -> bool {
    matches!(
        error,
        StrategyError::ProfitLossError(
            ProfitLossErrorKind::MaxProfitError { .. } | ProfitLossErrorKind::MaxLossError { .. }
        )
    )
}

/// The maximum profit the profit area and ratio measure: zero for a strategy
/// that reports a best case that still loses (see [`is_extreme_sign_error`]).
///
/// # Errors
///
/// Propagates every other error of `Strategies::get_max_profit`.
pub(crate) fn measured_max_profit<S: Strategies + ?Sized>(
    strategy: &S,
) -> Result<Positive, StrategyError> {
    match strategy.get_max_profit() {
        Err(error) if is_extreme_sign_error(&error) => Ok(Positive::ZERO),
        other => other,
    }
}

/// The maximum loss the profit area and ratio measure: zero for a strategy
/// that reports a worst case that still gains (see [`is_extreme_sign_error`]).
///
/// # Errors
///
/// Propagates every other error of `Strategies::get_max_loss`.
pub(crate) fn measured_max_loss<S: Strategies + ?Sized>(
    strategy: &S,
) -> Result<Positive, StrategyError> {
    match strategy.get_max_loss() {
        Err(error) if is_extreme_sign_error(&error) => Ok(Positive::ZERO),
        other => other,
    }
}

/// The `f64` nearest to `value`, through
/// [`decimal_to_f64`](optionstratlib_core::model::decimal::decimal_to_f64),
/// the correctly rounded conversion of #670 (#828). `Positive::to_f64` and
/// `ToPrimitive::to_f64` are `rust_decimal`'s `as_f64`, which is not always
/// the nearest `f64`.
///
/// # Errors
///
/// [`StrategyError`] wrapping the conversion's `DecimalError`.
pub(crate) fn f64_of(value: impl Into<Decimal>) -> Result<f64, StrategyError> {
    Ok(optionstratlib_core::model::decimal::decimal_to_f64(
        value.into(),
    )?)
}

/// Converts an `f64` figure (an area, a ratio) to `Decimal`.
///
/// # Errors
///
/// Returns [`StrategyError::NumericConversion`] for a `NaN`, an infinity or
/// a magnitude past `Decimal`, instead of answering with a number nobody
/// computed (#788).
#[inline]
pub(crate) fn decimal_from_f64(value: f64) -> Result<Decimal, StrategyError> {
    <Decimal as num_traits::FromPrimitive>::from_f64(value)
        .ok_or_else(|| StrategyError::numeric_conversion(value))
}

/// Helper function to aggregate fees from multiple positions.
///
/// # Arguments
///
/// * `positions` - Slice of position references
///
/// # Returns
///
/// Total fees (open + close) for all positions.
///
/// # Errors
///
/// Returns [`StrategyError::PositiveError`] when the total overflows
/// `Positive`.
pub fn aggregate_fees(positions: &[&Position]) -> Result<Positive, StrategyError> {
    positions.iter().try_fold(Positive::ZERO, |acc, p| {
        let fee = p.open_fee.checked_add(&p.close_fee)?;
        Ok(acc.checked_add(&fee)?)
    })
}

/// Helper function to aggregate premiums from multiple positions.
///
/// # Arguments
///
/// * `positions` - Slice of position references
///
/// # Returns
///
/// Total premium for all positions.
///
/// # Errors
///
/// Returns [`StrategyError::PositiveError`] when the total overflows
/// `Positive`.
pub fn aggregate_premiums(positions: &[&Position]) -> Result<Positive, StrategyError> {
    positions
        .iter()
        .try_fold(Positive::ZERO, |acc, p| Ok(acc.checked_add(&p.premium)?))
}

/// The contract size the option legs of a strategy share.
///
/// # Errors
///
/// Returns `StrategyError::OperationError(InvalidParameters { .. })` when
/// `legs` is empty or the legs carry different contract sizes.
pub(crate) fn common_contract_size(
    legs: &[&Position],
    operation: &str,
) -> Result<Positive, StrategyError> {
    let mut sizes = legs.iter().map(|leg| leg.option.contract_size);
    let Some(first) = sizes.next() else {
        return Err(StrategyError::invalid_parameters(
            operation,
            "the strategy has no option legs",
        ));
    };
    if sizes.any(|size| size != first) {
        return Err(StrategyError::invalid_parameters(
            operation,
            "the option legs carry different contract sizes",
        ));
    }
    Ok(first)
}

/// Sets the contract size of every leg in `legs`, keeping the quantities in
/// contracts.
///
/// # Errors
///
/// Returns `StrategyError::OperationError(InvalidParameters { .. })` when
/// `contract_size` is zero; no leg is changed then.
pub(crate) fn apply_contract_size(
    legs: &mut [&mut Position],
    contract_size: Positive,
    operation: &str,
) -> Result<(), StrategyError> {
    if contract_size == Positive::ZERO {
        return Err(StrategyError::invalid_parameters(
            operation,
            "contract size must be strictly positive",
        ));
    }
    for leg in legs.iter_mut() {
        leg.option.contract_size = contract_size;
    }
    Ok(())
}

/// Re-expresses the option legs of a covered strategy in contracts of
/// `contract_size` units, keeping the units of the underlying each leg covers
/// and its fee per unit.
///
/// Covered strategies (`CoveredCall`, `Collar`, `ProtectivePut`) size their
/// option legs against the shares of the spot leg and take their option fees
/// per share (#731). A leg of `quantity` contracts of `old` units becomes
/// `quantity × old / contract_size` contracts of `contract_size` units, and
/// its per-contract fees scale by `contract_size / old`, so the payoff, the
/// premium and the fees of the strategy are unchanged.
///
/// The new legs are computed before any leg changes, so an error leaves the
/// legs untouched.
///
/// # Errors
///
/// Returns `StrategyError::OperationError(InvalidParameters { .. })` when
/// `contract_size` is zero, and a `PositiveError` when a quantity or a fee
/// leaves the `Positive` range.
pub(crate) fn apply_hedge_contract_size(
    legs: &mut [&mut Position],
    contract_size: Positive,
    operation: &str,
) -> Result<(), StrategyError> {
    if contract_size == Positive::ZERO {
        return Err(StrategyError::invalid_parameters(
            operation,
            "contract size must be strictly positive",
        ));
    }
    let mut resized = Vec::with_capacity(legs.len());
    for leg in legs.iter() {
        let old = leg.option.contract_size;
        let quantity = leg.option.position_size()?.checked_div(&contract_size)?;
        let open_fee = leg
            .open_fee
            .checked_mul(&contract_size)?
            .checked_div(&old)?;
        let close_fee = leg
            .close_fee
            .checked_mul(&contract_size)?
            .checked_div(&old)?;
        resized.push((quantity, open_fee, close_fee));
    }
    for (leg, (quantity, open_fee, close_fee)) in legs.iter_mut().zip(resized) {
        leg.option.quantity = quantity;
        leg.option.contract_size = contract_size;
        leg.open_fee = open_fee;
        leg.close_fee = close_fee;
    }
    Ok(())
}

/// The profit and the loss zones of an expiry P&L whose zeros are
/// `break_evens`, in ascending order.
///
/// The price line is cut at each break-even and each piece is classified by
/// the sign of `profit_at` at a point inside it: half the first break-even,
/// the midpoint of two consecutive ones, and twice the last. A piece whose
/// probe is exactly zero joins neither list, and a piece that starts and
/// ends at zero is skipped. With no break-even the whole line is one piece,
/// probed at `reference`.
///
/// The covered strategies use this when an option leg does not cover the
/// shares exactly, so the P&L can cross zero on either side of a strike.
/// The ranges carry no probability yet; see [`price_zones`].
///
/// # Errors
///
/// Returns [`ProbabilityError`] when a probe point leaves the `Positive`
/// range, when `profit_at` fails, or when two break-evens are not in
/// ascending order.
pub(crate) fn expiry_zones<F>(
    break_evens: &[Positive],
    reference: Positive,
    profit_at: F,
) -> Result<(Vec<ProfitLossRange>, Vec<ProfitLossRange>), ProbabilityError>
where
    F: Fn(&Positive) -> Result<Decimal, PricingError>,
{
    let mut pieces: Vec<(Option<Positive>, Option<Positive>, Positive)> =
        Vec::with_capacity(break_evens.len());
    match (break_evens.first(), break_evens.last()) {
        (Some(&first), Some(&last)) => {
            if first > Positive::ZERO {
                pieces.push((None, Some(first), first.checked_div(&Positive::TWO)?));
            }
            for pair in break_evens.windows(2) {
                if let [lower, upper] = pair {
                    let probe = lower.checked_add(upper)?.checked_div(&Positive::TWO)?;
                    pieces.push((Some(*lower), Some(*upper), probe));
                }
            }
            let probe = if last > Positive::ZERO {
                last.checked_mul(&Positive::TWO)?
            } else {
                Positive::ONE
            };
            pieces.push((Some(last), None, probe));
        }
        _ => pieces.push((None, None, reference)),
    }

    let mut profit = Vec::new();
    let mut loss = Vec::new();
    for (lower, upper, probe) in pieces {
        let pnl = profit_at(&probe)?;
        if pnl > Decimal::ZERO {
            profit.push(ProfitLossRange::new(lower, upper, Positive::ZERO)?);
        } else if pnl < Decimal::ZERO {
            loss.push(ProfitLossRange::new(lower, upper, Positive::ZERO)?);
        }
    }
    Ok((profit, loss))
}

/// Prices each range in `ranges` at expiry, with the spot at `spot` and the
/// volatility, expiration and risk-free rate of `option`.
///
/// # Errors
///
/// Propagates [`ProfitLossRange::calculate_probability`].
pub(crate) fn price_zones(
    ranges: &mut [ProfitLossRange],
    spot: &Positive,
    option: &Options,
) -> Result<(), ProbabilityError> {
    for range in ranges.iter_mut() {
        range.calculate_probability(
            spot,
            VolatilityAdjustment {
                base_volatility: option.implied_volatility,
                std_dev_adjustment: Positive::ZERO,
            },
            None,
            &option.expiration_date,
            Some(option.risk_free_rate),
        )?;
    }
    Ok(())
}

/// A strategy that caches its break-even points in a field, so an edit to
/// one of its legs can refresh them (#780).
pub(crate) trait CachedBreakEvens: BreakEvenable + Validable + Clone {
    /// The cached break-even points.
    fn break_evens_mut(&mut self) -> &mut Vec<Positive>;
}

/// Applies `edit` to `strategy` and refreshes its cached break-evens, with
/// the roll-back contract of #771 (#780).
///
/// - When the edited strategy validates, its break-evens are recomputed. If
///   the recomputation fails, the strategy (legs and break-evens) is
///   restored and the error is returned.
/// - When it does not validate there are no break-evens to report, so they
///   are cleared and the edit stands. This is the case of a strategy
///   assembled leg by leg from its `Default`, whose remaining legs are
///   still placeholders, and of an edit that leaves the legs inconsistent,
///   which `add_position` has always accepted.
/// - When `edit` itself fails, the strategy is restored and its error is
///   returned.
///
/// # Errors
///
/// Returns the error of `edit`, or the failed recomputation as a
/// [`PositionError`].
pub(crate) fn edit_refreshing_break_evens<S, F>(
    strategy: &mut S,
    edit: F,
) -> Result<(), PositionError>
where
    S: CachedBreakEvens,
    F: FnOnce(&mut S) -> Result<(), PositionError>,
{
    let previous = strategy.clone();
    if let Err(error) = edit(strategy) {
        *strategy = previous;
        return Err(error);
    }
    if !strategy.validate() {
        strategy.break_evens_mut().clear();
        return Ok(());
    }
    if let Err(error) = strategy.update_break_even_points() {
        *strategy = previous;
        return Err(error.into());
    }
    Ok(())
}

/// The single position of a single-leg request, checked against the leg the
/// strategy holds (#831).
///
/// # Errors
///
/// Returns `StrategyError::OperationError` unless `positions` holds exactly
/// one position of `style` on `side`.
pub(crate) fn single_leg_position<'a>(
    positions: &'a [Position],
    style: OptionStyle,
    side: Side,
    strategy: &str,
) -> Result<&'a Position, StrategyError> {
    let operation = format!("{strategy} get_strategy");
    let [position] = positions else {
        return Err(StrategyError::invalid_parameters(
            &operation,
            "must have exactly 1 option",
        ));
    };
    if position.option.option_style != style || position.option.side != side {
        return Err(StrategyError::invalid_parameters(
            &operation,
            &format!("the option must be a {side:?} {style:?}"),
        ));
    }
    Ok(position)
}

/// The share leg of a covered request (#831): present, long, and on the
/// underlying of every option position.
///
/// # Errors
///
/// Returns `StrategyError::OperationError` when `spot_leg` is missing, is
/// short, or names another underlying than one of `positions`.
pub(crate) fn covered_spot_leg<'a>(
    spot_leg: Option<&'a SpotPosition>,
    positions: &[Position],
    strategy: &str,
) -> Result<&'a SpotPosition, StrategyError> {
    let operation = format!("{strategy} get_strategy_with_spot");
    let Some(spot_leg) = spot_leg else {
        return Err(StrategyError::invalid_parameters(
            &operation,
            "the strategy holds the underlying: a share leg is required",
        ));
    };
    if spot_leg.side != Side::Long {
        return Err(StrategyError::invalid_parameters(
            &operation,
            "the share leg must be long",
        ));
    }
    if positions
        .iter()
        .any(|position| position.option.underlying_symbol != spot_leg.symbol)
    {
        return Err(StrategyError::invalid_parameters(
            &operation,
            "the share leg and the options must be on the same underlying",
        ));
    }
    Ok(spot_leg)
}

#[cfg(test)]
mod tests_shared {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_credit_spread_break_even_call() {
        let short_strike = Positive::new(100.0).unwrap();
        let net_credit = Positive::new(5.0).unwrap();
        let break_even = credit_spread_break_even(short_strike, net_credit, true).unwrap();
        assert_eq!(break_even, Positive::new(105.0).unwrap());
    }

    #[test]
    fn test_credit_spread_break_even_put() {
        let short_strike = Positive::new(100.0).unwrap();
        let net_credit = Positive::new(5.0).unwrap();
        let break_even = credit_spread_break_even(short_strike, net_credit, false).unwrap();
        assert_eq!(break_even, Positive::new(95.0).unwrap());
    }

    #[test]
    fn test_debit_spread_break_even_call() {
        let long_strike = Positive::new(100.0).unwrap();
        let net_debit = Positive::new(3.0).unwrap();
        let break_even = debit_spread_break_even(long_strike, net_debit, true).unwrap();
        assert_eq!(break_even, Positive::new(103.0).unwrap());
    }

    #[test]
    fn test_debit_spread_break_even_put() {
        let long_strike = Positive::new(100.0).unwrap();
        let net_debit = Positive::new(3.0).unwrap();
        let break_even = debit_spread_break_even(long_strike, net_debit, false).unwrap();
        assert_eq!(break_even, Positive::new(97.0).unwrap());
    }

    #[test]
    fn test_calculate_profit_ratio() {
        let max_profit = Positive::new(50.0).unwrap();
        let max_loss = Positive::new(100.0).unwrap();
        let ratio = calculate_profit_ratio(max_profit, max_loss).unwrap();
        assert_eq!(ratio, dec!(50));
    }

    #[test]
    fn test_calculate_profit_ratio_zero_loss() {
        let max_profit = Positive::new(50.0).unwrap();
        let max_loss = Positive::ZERO;
        let ratio = calculate_profit_ratio(max_profit, max_loss).unwrap();
        assert_eq!(ratio, Decimal::MAX);
    }

    #[test]
    fn test_spread_break_even_put_past_zero_errs() {
        let strike = Positive::new(10.0).unwrap();
        let amount = Positive::new(20.0).unwrap();
        assert!(credit_spread_break_even(strike, amount, false).is_err());
        assert!(debit_spread_break_even(strike, amount, false).is_err());
    }

    #[test]
    fn test_spread_break_even_call_overflow_errs() {
        let amount = Positive::new(20.0).unwrap();
        assert!(credit_spread_break_even(Positive::MAX, amount, true).is_err());
        assert!(debit_spread_break_even(Positive::MAX, amount, true).is_err());
    }

    #[test]
    fn test_calculate_profit_ratio_overflow_errs() {
        let tiny = Positive::new_decimal(Decimal::new(1, 28)).unwrap();
        assert!(calculate_profit_ratio(Positive::MAX, tiny).is_err());
    }

    #[test]
    fn test_aggregate_fees_and_premiums_overflow_errs() {
        let mut position = Position {
            open_fee: Positive::MAX,
            close_fee: Positive::ONE,
            premium: Positive::MAX,
            ..Position::default()
        };
        assert!(aggregate_fees(&[&position]).is_err());
        assert!(aggregate_premiums(&[&position, &position]).is_err());

        position.open_fee = Positive::ONE;
        position.close_fee = Positive::TWO;
        position.premium = Positive::new(3.0).unwrap();
        assert_eq!(
            aggregate_fees(&[&position, &position]).unwrap(),
            Positive::new(6.0).unwrap()
        );
        assert_eq!(
            aggregate_premiums(&[&position, &position]).unwrap(),
            Positive::new(6.0).unwrap()
        );
    }

    /// `measured_max_profit` / `measured_max_loss` read a sign report as
    /// zero and propagate every other error (#788).
    #[test]
    fn test_measured_extremes_map_only_sign_reports() {
        use crate::strategies::base::{BasicAble, Positionable};

        struct Stub {
            profit: fn() -> Result<Positive, StrategyError>,
            loss: fn() -> Result<Positive, StrategyError>,
        }
        impl Validable for Stub {}
        impl Positionable for Stub {}
        impl BreakEvenable for Stub {}
        impl BasicAble for Stub {}
        impl Strategies for Stub {
            fn get_max_profit(&self) -> Result<Positive, StrategyError> {
                (self.profit)()
            }
            fn get_max_loss(&self) -> Result<Positive, StrategyError> {
                (self.loss)()
            }
        }

        let signs = Stub {
            profit: || {
                Err(StrategyError::ProfitLossError(
                    ProfitLossErrorKind::MaxProfitError {
                        reason: "Max profit is negative".to_string(),
                    },
                ))
            },
            loss: || {
                Err(StrategyError::ProfitLossError(
                    ProfitLossErrorKind::MaxLossError {
                        reason: "Max loss is negative".to_string(),
                    },
                ))
            },
        };
        assert_eq!(measured_max_profit(&signs).unwrap(), Positive::ZERO);
        assert_eq!(measured_max_loss(&signs).unwrap(), Positive::ZERO);

        let failures = Stub {
            profit: || Err(StrategyError::empty_collection("profit")),
            loss: || Err(StrategyError::numeric_conversion(f64::NAN)),
        };
        assert!(matches!(
            measured_max_profit(&failures),
            Err(StrategyError::EmptyCollection { .. })
        ));
        assert!(matches!(
            measured_max_loss(&failures),
            Err(StrategyError::NumericConversion { .. })
        ));

        let values = Stub {
            profit: || Ok(Positive::TEN),
            loss: || Ok(Positive::ONE),
        };
        assert_eq!(measured_max_profit(&values).unwrap(), Positive::TEN);
        assert_eq!(measured_max_loss(&values).unwrap(), Positive::ONE);
    }

    #[test]
    fn test_calculate_profit_ratio_zero_profit() {
        let max_profit = Positive::ZERO;
        let max_loss = Positive::new(100.0).unwrap();
        let ratio = calculate_profit_ratio(max_profit, max_loss).unwrap();
        assert_eq!(ratio, Decimal::ZERO);
    }
}
