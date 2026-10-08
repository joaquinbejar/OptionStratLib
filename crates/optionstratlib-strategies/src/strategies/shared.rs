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

use crate::error::strategies::StrategyError;
use optionstratlib_analytics::analytics::ProfitLossRange;
use optionstratlib_analytics::analytics::probability::VolatilityAdjustment;
use optionstratlib_analytics::error::probability::ProbabilityError;
use optionstratlib_analytics::pnl::utils::PnL;
use optionstratlib_core::model::Options;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::{d_mul, d_sub};
use optionstratlib_core::model::leg::SpotPosition;
use optionstratlib_core::model::leg::traits::LegAble;
use optionstratlib_core::model::position::Position;
use optionstratlib_core::model::types::Side;
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
    fn spread_width(&self) -> Positive {
        self.upper_strike() - self.lower_strike()
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
    fn wing_width(&self) -> Positive {
        let (lower, upper) = self.wing_strikes();
        (upper - lower) / Decimal::TWO
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
    fn inner_width(&self) -> Positive {
        let (_, lower_mid, upper_mid, _) = self.strikes();
        upper_mid - lower_mid
    }

    /// Returns the outer spread width (total width of the condor).
    fn outer_width(&self) -> Positive {
        let (lowest, _, _, highest) = self.strikes();
        highest - lowest
    }

    /// Returns the put spread width (lower wing).
    fn put_spread_width(&self) -> Positive {
        let (lowest, lower_mid, _, _) = self.strikes();
        lower_mid - lowest
    }

    /// Returns the call spread width (upper wing).
    fn call_spread_width(&self) -> Positive {
        let (_, _, upper_mid, highest) = self.strikes();
        highest - upper_mid
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
    fn strangle_width(&self) -> Positive {
        self.call_strike() - self.put_strike()
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
#[must_use]
pub fn credit_spread_break_even(
    short_strike: Positive,
    net_credit: Positive,
    is_call_spread: bool,
) -> Positive {
    if is_call_spread {
        short_strike + net_credit
    } else {
        short_strike - net_credit
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
#[must_use]
pub fn debit_spread_break_even(
    long_strike: Positive,
    net_debit: Positive,
    is_call_spread: bool,
) -> Positive {
    if is_call_spread {
        long_strike + net_debit
    } else {
        long_strike - net_debit
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
/// Currently infallible - every branch returns `Ok`, including the
/// sentinel cases where `max_loss` or `max_profit` is zero. The `Result`
/// signature is retained so future tweaks to the ratio definition (e.g.
/// checked division with rounding) can surface numerical failures without
/// breaking the public API.
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
    Ok(max_profit.to_dec() / max_loss.to_dec() * Decimal::ONE_HUNDRED)
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
#[must_use]
pub fn aggregate_fees(positions: &[&Position]) -> Positive {
    positions
        .iter()
        .map(|p| p.open_fee + p.close_fee)
        .fold(Positive::ZERO, |acc, fee| acc + fee)
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
#[must_use]
pub fn aggregate_premiums(positions: &[&Position]) -> Positive {
    positions
        .iter()
        .map(|p| p.premium)
        .fold(Positive::ZERO, |acc, premium| acc + premium)
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

#[cfg(test)]
mod tests_shared {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_credit_spread_break_even_call() {
        let short_strike = Positive::new(100.0).unwrap();
        let net_credit = Positive::new(5.0).unwrap();
        let break_even = credit_spread_break_even(short_strike, net_credit, true);
        assert_eq!(break_even, Positive::new(105.0).unwrap());
    }

    #[test]
    fn test_credit_spread_break_even_put() {
        let short_strike = Positive::new(100.0).unwrap();
        let net_credit = Positive::new(5.0).unwrap();
        let break_even = credit_spread_break_even(short_strike, net_credit, false);
        assert_eq!(break_even, Positive::new(95.0).unwrap());
    }

    #[test]
    fn test_debit_spread_break_even_call() {
        let long_strike = Positive::new(100.0).unwrap();
        let net_debit = Positive::new(3.0).unwrap();
        let break_even = debit_spread_break_even(long_strike, net_debit, true);
        assert_eq!(break_even, Positive::new(103.0).unwrap());
    }

    #[test]
    fn test_debit_spread_break_even_put() {
        let long_strike = Positive::new(100.0).unwrap();
        let net_debit = Positive::new(3.0).unwrap();
        let break_even = debit_spread_break_even(long_strike, net_debit, false);
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
    fn test_calculate_profit_ratio_zero_profit() {
        let max_profit = Positive::ZERO;
        let max_loss = Positive::new(100.0).unwrap();
        let ratio = calculate_profit_ratio(max_profit, max_loss).unwrap();
        assert_eq!(ratio, Decimal::ZERO);
    }
}
