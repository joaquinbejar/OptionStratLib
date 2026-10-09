//! Payoff contracts at expiry.
//!
//! The payoff of a contract at expiry is a pure function of its terms (spot,
//! strike, style, side and option type). It needs no pricing model, rate,
//! volatility or time to expiry, so it is a domain invariant owned by the
//! core model rather than by the pricing layer.

use crate::constants::ZERO;
use crate::error::{OptionsError, OptionsResult};
use crate::model::decimal::{decimal_to_f64, finite_decimal};
use crate::model::option::ExoticParams;
use crate::model::types::{
    AsianAveragingType, BarrierType, BinaryType, LookbackType, OptionStyle, OptionType,
    RainbowType, Side,
};
use positive::Positive;
use rust_decimal::Decimal;

/// Defines a contract for calculating the payoff value of an option.
///
/// The `Payoff` trait establishes a standard interface for implementing different
/// option payoff calculations. Classes that implement this trait can define specific
/// payoff formulas for various option types (standard calls/puts, exotic options, etc.).
///
/// # Examples
///
/// Implementing the trait for a standard call option:
///
/// ```rust
/// use optionstratlib_core::error::{OptionsError, OptionsResult};
/// use optionstratlib_core::model::payoff::{Payoff, PayoffInfo};
/// use optionstratlib_core::model::Side;
/// use rust_decimal::Decimal;
/// struct CallOption;
///
/// impl Payoff for CallOption {
///     fn payoff(&self, info: &PayoffInfo) -> OptionsResult<Decimal> {
///         let intrinsic = info
///             .spot
///             .to_dec()
///             .checked_sub(info.strike.to_dec())
///             .ok_or_else(|| OptionsError::PayoffError {
///                 reason: "spot - strike overflowed".to_string(),
///             })?
///             .max(Decimal::ZERO);
///         match info.side {
///             Side::Long => Ok(intrinsic),
///             Side::Short => Ok(-intrinsic),
///         }
///     }
/// }
/// ```
///
/// # Usage
///
/// This trait is typically used within the options pricing module to:
/// - Create standardized payoff calculations for different option types
/// - Enable polymorphic handling of various option payoff strategies
/// - Support both standard and exotic option payoffs through a unified interface
pub trait Payoff {
    /// Calculates the payoff value of an option based on the provided information.
    ///
    /// # Parameters
    ///
    /// * `info` - A reference to a `PayoffInfo` struct containing all necessary data
    ///   for calculating the option's payoff, including spot price, strike price,
    ///   option style, position side, and additional parameters for exotic options.
    ///
    /// # Returns
    ///
    /// The payoff as a `Decimal`, signed by `info.side`: a short position's
    /// payoff is the negated long payoff, so the result can be negative.
    ///
    /// # Errors
    ///
    /// Returns [`OptionsError::PayoffError`] when the payoff is not
    /// representable as a `Decimal` (non-finite, or beyond the `Decimal`
    /// range).
    fn payoff(&self, info: &PayoffInfo) -> OptionsResult<Decimal>;
}
/// `PayoffInfo` is a struct that holds information about an option's payoff calculation parameters.
///
/// This structure encapsulates all the necessary variables to calculate the payoff of different
/// option types, including standard options (calls and puts) as well as exotic options like
/// Asian and Lookback options.
///
/// # Usage
///
/// This structure is typically used within the options pricing module to calculate
/// the final payoff value of different option types at expiration or exercise.
///
#[derive(Debug, Clone)]
pub struct PayoffInfo {
    /// * `spot` - The current market price of the underlying asset.
    ///   This value is used as the reference price for calculating option payoffs.
    pub spot: Positive,
    /// * `strike` - The strike price specified in the option contract.
    ///   This is the price at which the option holder can buy (for calls) or sell (for puts)
    ///   the underlying asset.
    pub strike: Positive,
    /// * `style` - Defines whether the option is a Call or Put.
    ///   Call options give the right to buy, while put options give the right to sell.
    pub style: OptionStyle,
    /// * `side` - Indicates whether the position is Long (bought) or Short (sold).
    ///   This affects the direction of the payoff calculation.
    pub side: Side,
    /// * `spot_prices` - A collection of historical spot prices used specifically for Asian options.
    ///   Asian options base their payoff on the average price of the underlying asset over a specified period.
    pub spot_prices: Option<Vec<Positive>>, // Asian
    /// * `spot_min` - The minimum observed price of the underlying asset during the option's life.
    ///   Lookback options read it for the minimum price reached, and a down barrier
    ///   (`DownAndIn`, `DownAndOut`) is hit when it is at or below the barrier level.
    ///   When `None`, a down barrier is judged from `spot` alone.
    pub spot_min: Option<Positive>, // Lookback / down barriers
    /// * `spot_max` - The maximum observed price of the underlying asset during the option's life.
    ///   Lookback options read it for the maximum price reached, and an up barrier
    ///   (`UpAndIn`, `UpAndOut`) is hit when it is at or above the barrier level.
    ///   When `None`, an up barrier is judged from `spot` alone.
    pub spot_max: Option<Positive>, // Lookback / up barriers
    /// * `exotic_params` - The exotic parameters of the contract, for the
    ///   payoffs that read more than the spot and strike: a two-asset
    ///   rainbow reads `rainbow_second_asset_price`, a cliquet its global
    ///   cap and floor (#844). [`crate::model::Options::payoff`] passes the
    ///   option's own; the other families ignore it, and `None` is the
    ///   default.
    pub exotic_params: Option<ExoticParams>,
}

impl Default for PayoffInfo {
    fn default() -> Self {
        PayoffInfo {
            spot: Positive::ZERO,
            strike: Positive::ZERO,
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: None,
            spot_min: None,
            spot_max: None,
            exotic_params: None,
        }
    }
}

impl PayoffInfo {
    /// Returns the length of the spot prices collection if it exists.
    ///
    /// This method provides a safe way to check the length of the historical spot prices
    /// vector without direct access to the optional field. It's particularly useful when
    /// working with Asian options, which use a collection of historical prices to calculate
    /// their payoff based on average price.
    ///
    /// # Returns
    ///
    /// * `Some(usize)` - The number of spot prices in the collection if it exists
    /// * `None` - If no spot prices are available (the vector is None)
    ///
    /// # Example
    ///
    /// ```
    /// use optionstratlib_core::model::payoff::PayoffInfo;
    /// use positive::Positive;
    /// use optionstratlib_core::model::types::{OptionStyle, Side};
    /// # fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let payoff_info = PayoffInfo {
    ///     spot: Positive::new(100.0)?,
    ///     strike: Positive::new(105.0)?,
    ///     style: OptionStyle::Call,
    ///     side: Side::Long,
    ///     spot_prices: Some(vec![
    ///         Positive::new(98.0)?,
    ///         Positive::new(99.0)?,
    ///         Positive::new(101.0)?,
    ///         Positive::new(102.0)?,
    ///     ]),
    ///     spot_min: None,
    ///     spot_max: None,
    ///     exotic_params: None,
    /// };
    ///
    /// assert_eq!(payoff_info.spot_prices_len(), Some(4));
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    pub fn spot_prices_len(&self) -> Option<usize> {
        self.spot_prices.as_ref().map(|vec| vec.len())
    }
}

/// The vanilla intrinsic value `max(S - K, 0)` (call) or `max(K - S, 0)`
/// (put) of a long position, in `Decimal`.
///
/// Both operands live in `[0, Decimal::MAX]`, so the difference is always
/// representable; the checked form keeps the raw operator out of the kernel.
#[inline]
fn intrinsic(spot: Decimal, strike: Decimal, style: OptionStyle) -> Decimal {
    let moneyness = match style {
        OptionStyle::Call => spot.checked_sub(strike),
        OptionStyle::Put => strike.checked_sub(spot),
    };
    moneyness.unwrap_or(Decimal::ZERO).max(Decimal::ZERO)
}

/// The vanilla intrinsic value of `info` for a long position.
#[inline]
fn vanilla(info: &PayoffInfo) -> Decimal {
    intrinsic(info.spot.to_dec(), info.strike.to_dec(), info.style)
}

/// `value` for a long position, `-value` for a short one. A zero payoff is
/// zero for either side: `Decimal` negation would give a negative zero.
#[inline]
fn signed(value: Decimal, side: Side) -> Decimal {
    match side {
        Side::Short if !value.is_zero() => -value,
        _ => value,
    }
}

impl Payoff for OptionType {
    /// The payoff at expiry of one unit of the contract, signed by
    /// `info.side`.
    ///
    /// Every family is valued for a long position by the private kernel and
    /// signed once here, so a short position's payoff is always the negated
    /// long payoff (#844). The kernel is the contract's terminal payoff, the
    /// value the pricing kernels of `optionstratlib-pricing` return at
    /// `T = 0`:
    ///
    /// | Family | Long payoff |
    /// | --- | --- |
    /// | European, American, Bermuda, fixed-strike Lookback | `max(S - K, 0)` call, `max(K - S, 0)` put |
    /// | Asian | the vanilla payoff on the average of `spot_prices`; with no fixings the averaging window is the expiry instant, whose average is `S` |
    /// | Barrier | the vanilla payoff or the rebate, by the barrier state (see the barrier kernel) |
    /// | Binary | cash-or-nothing `1`, asset-or-nothing `S`, gap `\|S - K\|`, when in the money |
    /// | Floating-strike Lookback | `S - min(S_min, S)` call, `max(S_max, S) - S` put: `0` with no observed extremes |
    /// | Compound | the vanilla payoff of the compound strike on the underlying option's own long payoff `U`: `max(U - K, 0)` call, `max(K - U, 0)` put |
    /// | Chooser | `max(S - K, K - S, 0)` |
    /// | Cliquet | `0` accrued (no reset fixings are known), clamped by the global floor and cap of `exotic_params` |
    /// | Rainbow (two assets) | the vanilla payoff on `max(S, S2)` (best of) or `min(S, S2)` (worst of), `S2` the `rainbow_second_asset_price` of `exotic_params` |
    /// | Spread | `max(S - S2 - K, 0)` call, `max(K - (S - S2), 0)` put |
    /// | Exchange | `max(S - S2, 0)` for either style |
    /// | Quanto | the vanilla payoff times the fixed exchange rate |
    /// | Power | `max(S^n - K, 0)` call, `max(K - S^n, 0)` put |
    ///
    /// The result is normalized (no trailing zeros), the form the `f64`
    /// kernel produced through `Decimal::from_f64` before #844, so an exact
    /// payoff keeps the representation it serialized and displayed with.
    fn payoff(&self, info: &PayoffInfo) -> OptionsResult<Decimal> {
        Ok(signed(long_payoff(self, info)?, info.side).normalize())
    }
}

/// Error for a kernel payoff that has no `Decimal` representation.
#[cold]
#[inline(never)]
fn payoff_not_representable(value: f64) -> OptionsError {
    OptionsError::PayoffError {
        reason: format!(
            "payoff {value} is not representable as a Decimal (non-finite or out of range)"
        ),
    }
}

/// Error for a payoff that needs data `info` does not carry.
#[cold]
#[inline(never)]
fn payoff_missing(reason: &str) -> OptionsError {
    OptionsError::PayoffError {
        reason: reason.to_string(),
    }
}

/// Error for a payoff whose checked arithmetic leaves the `Decimal` range.
#[cold]
#[inline(never)]
fn payoff_overflow(operation: &str) -> OptionsError {
    OptionsError::PayoffError {
        reason: format!("{operation} left the Decimal range"),
    }
}

/// An `f64` kernel value at the `Decimal` boundary.
fn from_kernel(value: f64) -> OptionsResult<Decimal> {
    finite_decimal(value).ok_or_else(|| payoff_not_representable(value))
}

/// The payoff of one unit of `option_type` for a long position: the
/// contract's terminal payoff, in `Decimal`. [`Payoff::payoff`] signs it.
///
/// `f64` is used only where the pricing kernels use it too: the Asian
/// averages of observed fixings and the power `S^n`.
fn long_payoff(option_type: &OptionType, info: &PayoffInfo) -> OptionsResult<Decimal> {
    match option_type {
        OptionType::European | OptionType::American | OptionType::Bermuda { .. } => {
            Ok(vanilla(info))
        }
        OptionType::Asian { averaging_type } => asian_payoff(averaging_type, info),
        OptionType::Barrier {
            barrier_type,
            barrier_level,
            rebate,
        } => Ok(barrier_payoff(barrier_type, barrier_level, rebate, info)),
        OptionType::Binary { binary_type } => Ok(binary_payoff(binary_type, info)),
        OptionType::Lookback {
            lookback_type: LookbackType::FloatingStrike,
        } => Ok(floating_strike_payoff(info)),
        // A fixed strike at expiry is the vanilla payoff, the extremum being
        // the spot; `LookbackType` is `#[non_exhaustive]`.
        OptionType::Lookback { .. } => Ok(vanilla(info)),
        OptionType::Compound { underlying_option } => {
            // The underlying option expires with the compound here, so it is
            // worth its own long payoff at the spot, and the compound is the
            // vanilla payoff of its strike on that value, as
            // `compound_black_scholes` prices it at `T = 0` (#844).
            let underlying = long_payoff(underlying_option, info)?;
            Ok(intrinsic(underlying, info.strike.to_dec(), info.style))
        }
        OptionType::Chooser { .. } => {
            Ok(
                intrinsic(info.spot.to_dec(), info.strike.to_dec(), OptionStyle::Call).max(
                    intrinsic(info.spot.to_dec(), info.strike.to_dec(), OptionStyle::Put),
                ),
            )
        }
        OptionType::Cliquet { .. } => Ok(cliquet_payoff(info)),
        OptionType::Rainbow {
            num_assets,
            rainbow_type,
        } => rainbow_payoff(*num_assets, rainbow_type, info),
        OptionType::Spread { second_asset } => {
            let spread = info
                .spot
                .to_dec()
                .checked_sub(second_asset.to_dec())
                .ok_or_else(|| payoff_overflow("spread S1 - S2"))?;
            let strike = info.strike.to_dec();
            let value = match info.style {
                OptionStyle::Call => spread.checked_sub(strike),
                OptionStyle::Put => strike.checked_sub(spread),
            }
            .ok_or_else(|| payoff_overflow("spread payoff"))?;
            Ok(value.max(Decimal::ZERO))
        }
        // The right to exchange the second asset for the first: no strike,
        // and the same payoff for either style, as `exchange_black_scholes`
        // prices it.
        OptionType::Exchange { second_asset } => Ok(intrinsic(
            info.spot.to_dec(),
            second_asset.to_dec(),
            OptionStyle::Call,
        )),
        OptionType::Quanto { exchange_rate } => vanilla(info)
            .checked_mul(exchange_rate.to_dec())
            .ok_or_else(|| payoff_overflow("quanto payoff times the exchange rate")),
        OptionType::Power { exponent } => power_payoff(*exponent, info),
        // `OptionType` is `#[non_exhaustive]`: a variant added upstream falls
        // back to the plain intrinsic value until it gets its own arm.
        _ => Ok(vanilla(info)),
    }
}

/// The long payoff of an Asian option.
///
/// With observed fixings in `spot_prices` the payoff is the vanilla payoff
/// on their average (arithmetic, or geometric through [`geometric_mean`]),
/// computed in `f64` as before #844. With none, the averaging window ends at
/// the expiry instant it starts from, so the average is the spot itself and
/// the payoff is the vanilla intrinsic value: the value
/// `asian_black_scholes` prices a contract at `T = 0`. It used to be `0`
/// (#844).
fn asian_payoff(averaging_type: &AsianAveragingType, info: &PayoffInfo) -> OptionsResult<Decimal> {
    let fixings = match info.spot_prices.as_deref() {
        Some(fixings) if !fixings.is_empty() => fixings,
        _ => return Ok(vanilla(info)),
    };
    let average = match averaging_type {
        AsianAveragingType::Geometric => geometric_mean(fixings)?,
        // `AsianAveragingType` is `#[non_exhaustive]`: fall back to the
        // arithmetic mean, the conventional default for Asian options.
        _ => {
            let mut total = ZERO;
            for fixing in fixings {
                total += decimal_to_f64(fixing.to_dec())?;
            }
            total / fixings.len() as f64
        }
    };
    match info.style {
        OptionStyle::Call => {
            from_kernel((average - decimal_to_f64(info.strike.to_dec())?).max(ZERO))
        }
        // The put is formed on `Decimal`, as since #788; a non-finite average
        // has no put value and is reported as unrepresentable.
        OptionStyle::Put => {
            let average = from_kernel(average)?;
            Ok(intrinsic(average, info.strike.to_dec(), OptionStyle::Put))
        }
    }
}

/// Geometric mean of the fixings, as the log-sum `exp(mean(ln x_i))` (#806).
///
/// The product of the fixings, rooted afterwards, leaves the `f64` range on
/// realistic inputs (100 fixings at `1e4` multiply to `1e400`), so the mean
/// is taken in log space. The log-sum is centred on the first fixing `m`,
/// `m * exp(mean(ln(x_i / m)))`, which is the same quantity: the logarithms
/// of `x_i / m` sit near zero, where `ln` and `exp` keep their relative
/// accuracy, whereas `ln x_i` near 9 carries about `1e-15` absolute error,
/// which `exp` turns into tens of ulps. Against a 60-digit reference over
/// 2 001 fixing sets (3 to 252 fixings, levels `1e-2` to `1e4`) the centred
/// form is within 3 ulps (median 0), the uncentred one within 56 (median 2),
/// and the product overflows or underflows on 297 of them. A zero fixing makes the mean
/// zero, as it made the product. An empty slice has no mean; the caller
/// never passes one.
///
/// # Errors
///
/// Propagates the [`DecimalError`](crate::error::DecimalError) of
/// [`decimal_to_f64`] (#828).
fn geometric_mean(fixings: &[Positive]) -> OptionsResult<f64> {
    let Some(first) = fixings.first() else {
        return Ok(ZERO);
    };
    if fixings.contains(&Positive::ZERO) {
        return Ok(ZERO);
    }
    let centre = decimal_to_f64(first.to_dec())?;
    let mut log_sum = ZERO;
    for fixing in fixings {
        log_sum += (decimal_to_f64(fixing.to_dec())? / centre).ln(); // scan-banned: allow -- f64 `ln` of a positive finite ratio, it does not abort
    }
    Ok(centre * (log_sum / fixings.len() as f64).exp()) // scan-banned: allow -- f64 `exp`: returns inf on overflow, it does not abort; a non-finite payoff is rejected at the `Decimal` boundary
}

/// The long payoff at expiry of a barrier option (Reiner-Rubinstein contract
/// terms); [`Payoff::payoff`] signs it by `info.side`, rebate included.
///
/// # Barrier state at expiry
///
/// The barrier counts as hit when the extreme of the path reached it: for an
/// up barrier `spot_max >= barrier_level`, for a down barrier
/// `spot_min <= barrier_level`. A missing extreme falls back to the final
/// `spot`, so with neither field set the state is judged from the expiry
/// price alone, and a path that touched the barrier and came back is treated
/// as never hit. [`crate::model::Options::payoff`] sets neither field: it
/// knows only the current underlying price, which is also the state
/// `barrier_black_scholes` prices at `T = 0`.
///
/// # Payoff
///
/// | Contract | Barrier hit | Barrier not hit |
/// | --- | --- | --- |
/// | knock-in (`UpAndIn`, `DownAndIn`) | vanilla payoff | rebate |
/// | knock-out (`UpAndOut`, `DownAndOut`) | rebate | vanilla payoff |
///
/// The rebate is `rebate` (zero when `None`): a knock-in pays it at expiry
/// when it never came alive (Haug's `E` term), a knock-out pays it on the
/// hit (`F`). A long position receives it and a short one pays it. A barrier
/// type added upstream (`BarrierType` is `#[non_exhaustive]`) pays the
/// vanilla payoff.
///
/// Until #826 an unhit knock-in paid zero instead of its rebate, and the
/// rebate of a hit knock-out was not signed by the side, so the payoff
/// disagreed with the Reiner-Rubinstein price as `T → 0`.
fn barrier_payoff(
    barrier_type: &BarrierType,
    barrier_level: &Positive,
    rebate: &Option<Positive>,
    info: &PayoffInfo,
) -> Decimal {
    let hit = match barrier_type {
        BarrierType::UpAndIn | BarrierType::UpAndOut => {
            info.spot_max.unwrap_or(info.spot) >= *barrier_level
        }
        BarrierType::DownAndIn | BarrierType::DownAndOut => {
            info.spot_min.unwrap_or(info.spot) <= *barrier_level
        }
        // `BarrierType` is `#[non_exhaustive]`: an unknown barrier is never
        // triggered, and pays the vanilla payoff below.
        _ => false,
    };
    let rebate = rebate.map_or(Decimal::ZERO, |amount| amount.to_dec());
    match barrier_type {
        BarrierType::UpAndIn | BarrierType::DownAndIn if !hit => rebate,
        BarrierType::UpAndOut | BarrierType::DownAndOut if hit => rebate,
        _ => vanilla(info),
    }
}

/// The long payoff of a binary option when it expires in the money (`S > K`
/// for a call, `S < K` for a put), and zero otherwise:
///
/// - **CashOrNothing**: `1`.
/// - **AssetOrNothing**: the spot `S`.
/// - **Gap**: `|S - K|`, the trigger and the payoff strike being the same
///   `K`, as `binary_black_scholes` prices it.
///
/// A binary type added upstream (`BinaryType` is `#[non_exhaustive]`) pays
/// the cash-or-nothing amount, the most conservative of the three.
fn binary_payoff(binary_type: &BinaryType, info: &PayoffInfo) -> Decimal {
    let in_the_money = match info.style {
        OptionStyle::Call => info.spot > info.strike,
        OptionStyle::Put => info.spot < info.strike,
    };
    if !in_the_money {
        return Decimal::ZERO;
    }
    match binary_type {
        BinaryType::AssetOrNothing => info.spot.to_dec(),
        BinaryType::Gap => vanilla(info),
        _ => Decimal::ONE,
    }
}

/// The long payoff of a floating-strike lookback option.
///
/// A call buys at the lowest price of the path, `S - S_min`; a put sells at
/// the highest, `S_max - S`. The path ends at the expiry spot, so the
/// extremes include it: `S_min` is `min(spot_min, S)` and `S_max` is
/// `max(spot_max, S)`, and a missing extreme is the spot alone. With no
/// observed extremes the payoff is `0`, the value `lookback_black_scholes`
/// prices a new contract at `T = 0`; it used to be `±S` (#844).
fn floating_strike_payoff(info: &PayoffInfo) -> Decimal {
    let spot = info.spot.to_dec();
    match info.style {
        OptionStyle::Call => {
            let low = info.spot_min.map_or(spot, |low| low.to_dec().min(spot));
            spot.checked_sub(low).unwrap_or(Decimal::ZERO)
        }
        OptionStyle::Put => {
            let high = info.spot_max.map_or(spot, |high| high.to_dec().max(spot));
            high.checked_sub(spot).unwrap_or(Decimal::ZERO)
        }
    }
}

/// The long payoff of a cliquet option at expiry.
///
/// A cliquet pays the sum of its capped and floored period returns. No reset
/// fixings reach [`PayoffInfo`], so no period has accrued and the sum is `0`,
/// clamped by the global cap and then the global floor of `exotic_params`
/// when set: the value `cliquet_black_scholes` prices at `T = 0`. It used to
/// be the vanilla payoff on the spot (#844).
fn cliquet_payoff(info: &PayoffInfo) -> Decimal {
    let mut total = Decimal::ZERO;
    if let Some(params) = &info.exotic_params {
        if let Some(cap) = params.cliquet_global_cap {
            total = total.min(cap);
        }
        if let Some(floor) = params.cliquet_global_floor {
            total = total.max(floor);
        }
    }
    total
}

/// The long payoff of a two-asset rainbow option: the vanilla payoff on the
/// best (`max(S, S2)`) or the worst (`min(S, S2)`) of the two assets, `S2`
/// being `exotic_params.rainbow_second_asset_price`. It used to be the
/// vanilla payoff on `S` alone (#844).
///
/// # Errors
///
/// Returns [`OptionsError::PayoffError`] when the contract is not on two
/// assets, when `exotic_params` carries no second asset price, or for a
/// rainbow type added upstream (`RainbowType` is `#[non_exhaustive]`), as
/// `rainbow_black_scholes` rejects them.
fn rainbow_payoff(
    num_assets: usize,
    rainbow_type: &RainbowType,
    info: &PayoffInfo,
) -> OptionsResult<Decimal> {
    if num_assets != 2 {
        return Err(payoff_missing(
            "a rainbow payoff is defined on two assets only",
        ));
    }
    let second = info
        .exotic_params
        .as_ref()
        .and_then(|params| params.rainbow_second_asset_price)
        .ok_or_else(|| {
            payoff_missing("a rainbow payoff needs exotic_params.rainbow_second_asset_price")
        })?
        .to_dec();
    let first = info.spot.to_dec();
    let reference = match rainbow_type {
        RainbowType::BestOf => first.max(second),
        RainbowType::WorstOf => first.min(second),
        _ => return Err(payoff_missing("unsupported rainbow type")),
    };
    Ok(intrinsic(reference, info.strike.to_dec(), info.style))
}

/// The long payoff of a power option, `max(S^n - K, 0)` (call) or
/// `max(K - S^n, 0)` (put), with `S^n` formed in `f64` and taken to
/// `Decimal` before the strike is subtracted, as `power_black_scholes` does
/// at `T = 0`.
///
/// # Errors
///
/// Returns [`OptionsError::PayoffError`] for a call whose `S^n` has no
/// `Decimal` representation. The put is worthless there: `S^n` is `+∞` in
/// the limit.
fn power_payoff(exponent: Positive, info: &PayoffInfo) -> OptionsResult<Decimal> {
    let powered = decimal_to_f64(info.spot.to_dec())?.powf(decimal_to_f64(exponent.to_dec())?);
    match (finite_decimal(powered), info.style) {
        (Some(powered), style) => Ok(intrinsic(powered, info.strike.to_dec(), style)),
        (None, OptionStyle::Put) => Ok(Decimal::ZERO),
        (None, OptionStyle::Call) => Err(payoff_not_representable(powered)),
    }
}

#[cfg(test)]
mod tests_payoff {
    use super::*;
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    #[test]
    fn test_european_call() {
        let option = OptionType::European;
        let info = PayoffInfo {
            spot: pos_or_panic!(110.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Call,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info).unwrap(), dec!(10));
    }

    #[test]
    fn test_european_put() {
        let option = OptionType::European;
        let info = PayoffInfo {
            spot: pos_or_panic!(90.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Put,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info).unwrap(), dec!(10));
    }

    #[test]
    fn test_asian_arithmetic_call() {
        let option = OptionType::Asian {
            averaging_type: AsianAveragingType::Arithmetic,
        };
        let info = PayoffInfo {
            spot: Positive::HUNDRED,
            strike: Positive::HUNDRED,
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: Some(vec![
                pos_or_panic!(90.0),
                pos_or_panic!(100.0),
                pos_or_panic!(110.0),
            ]),
            ..Default::default()
        };
        assert_eq!(option.payoff(&info).unwrap(), Decimal::ZERO);
    }

    #[test]
    fn test_barrier_up_and_in_call() {
        let option = OptionType::Barrier {
            barrier_type: BarrierType::UpAndIn,
            barrier_level: pos_or_panic!(120.0),
            rebate: None,
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(130.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Call,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info).unwrap(), dec!(30));
    }

    #[test]
    fn test_binary_cash_or_nothing_call() {
        let option = OptionType::Binary {
            binary_type: BinaryType::CashOrNothing,
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(110.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Call,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info).unwrap(), dec!(1));
    }

    #[test]
    fn test_lookback_fixed_strike_put() {
        let option = OptionType::Lookback {
            lookback_type: LookbackType::FixedStrike,
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(90.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Put,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info).unwrap(), dec!(10));
    }

    #[test]
    fn test_quanto_call() {
        let option = OptionType::Quanto {
            exchange_rate: pos_or_panic!(1.5),
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(110.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Call,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info).unwrap(), dec!(15));
    }

    #[test]
    fn test_power_call() {
        let option = OptionType::Power {
            exponent: pos_or_panic!(2.0),
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(10.0),
            strike: pos_or_panic!(90.0),
            style: OptionStyle::Call,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info).unwrap(), dec!(10));
    }

    /// An out-of-the-money chooser used to abort with
    /// `Positive invariant broken in sub: result would be non-positive`,
    /// because `info.spot - info.strike` is a `Positive` subtraction. The
    /// chooser keeps the better of the two intrinsics, so `S < K` is worth
    /// `K - S`, never a panic.
    #[test]
    fn test_chooser_out_of_the_money_call_returns_put_intrinsic() {
        let option = OptionType::Chooser {
            choice_date: Positive::ONE,
        };
        let info = PayoffInfo {
            spot: Positive::HUNDRED,
            strike: pos_or_panic!(110.0),
            style: OptionStyle::Call,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info).unwrap(), dec!(10));
    }

    #[test]
    fn test_chooser_in_the_money_call_keeps_its_value() {
        let option = OptionType::Chooser {
            choice_date: Positive::ONE,
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(110.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Call,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info).unwrap(), dec!(10));
    }

    #[test]
    fn test_chooser_at_the_money_is_zero() {
        let option = OptionType::Chooser {
            choice_date: Positive::ONE,
        };
        let info = PayoffInfo {
            spot: Positive::HUNDRED,
            strike: Positive::HUNDRED,
            style: OptionStyle::Call,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info).unwrap(), Decimal::ZERO);
    }

    /// A power put with `S^n > K` used to abort with
    /// `Positive invariant broken in sub_f64: result would be non-positive`.
    /// `100² = 10000` is far above the strike, so the put is worthless.
    #[test]
    fn test_power_put_out_of_the_money_is_zero() {
        let option = OptionType::Power {
            exponent: pos_or_panic!(2.0),
        };
        let info = PayoffInfo {
            spot: Positive::HUNDRED,
            strike: pos_or_panic!(110.0),
            style: OptionStyle::Put,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info).unwrap(), Decimal::ZERO);
    }

    #[test]
    fn test_power_put_in_the_money_keeps_its_value() {
        let option = OptionType::Power {
            exponent: pos_or_panic!(2.0),
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(10.0),
            strike: pos_or_panic!(110.0),
            style: OptionStyle::Put,
            side: Side::Long,
            ..Default::default()
        };
        // 110 - 10² = 10
        assert_eq!(option.payoff(&info).unwrap(), dec!(10));
    }

    #[test]
    fn test_power_put_at_the_money_is_zero() {
        let option = OptionType::Power {
            exponent: pos_or_panic!(2.0),
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(10.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Put,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info).unwrap(), Decimal::ZERO);
    }

    /// `S^n` beyond the `Decimal` range is `+∞` in the limit, where the put is
    /// worthless. It used to abort inside the `Positive` conversion.
    #[test]
    fn test_power_put_unrepresentable_power_is_zero() {
        let option = OptionType::Power {
            exponent: Positive::MAX,
        };
        let info = PayoffInfo {
            spot: Positive::HUNDRED,
            strike: Positive::HUNDRED,
            style: OptionStyle::Put,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info).unwrap(), Decimal::ZERO);
    }
}

#[cfg(test)]
mod tests_calculate_floating_strike_payoff {
    use super::*;
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    #[test]
    fn test_call_option_with_spot_min() {
        let info = PayoffInfo {
            spot: Positive::HUNDRED,
            strike: Positive::ZERO, // Not used in floating strike
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: None,
            spot_min: Some(pos_or_panic!(80.0)),
            spot_max: None,
            exotic_params: None,
        };
        assert_eq!(floating_strike_payoff(&info), dec!(20));
    }

    #[test]
    fn test_call_option_without_spot_min() {
        let info = PayoffInfo {
            spot: Positive::HUNDRED,
            strike: Positive::ZERO,
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: None,
            spot_min: None,
            spot_max: None,
            exotic_params: None,
        };
        assert_eq!(floating_strike_payoff(&info), Decimal::ZERO);
    }

    #[test]
    fn test_put_option_with_spot_max() {
        let info = PayoffInfo {
            spot: Positive::HUNDRED,
            strike: Positive::ZERO,
            style: OptionStyle::Put,
            side: Side::Long,
            spot_prices: None,
            spot_min: None,
            spot_max: Some(pos_or_panic!(120.0)),
            exotic_params: None,
        };
        assert_eq!(floating_strike_payoff(&info), dec!(20));
    }

    #[test]
    fn test_put_option_without_spot_max() {
        let info = PayoffInfo {
            spot: Positive::HUNDRED,
            strike: Positive::ZERO,
            style: OptionStyle::Put,
            side: Side::Long,
            spot_prices: None,
            spot_min: None,
            spot_max: None,
            exotic_params: None,
        };
        assert_eq!(floating_strike_payoff(&info), Decimal::ZERO);
    }

    #[test]
    fn test_call_option_spot_equals_min() {
        let info = PayoffInfo {
            spot: Positive::HUNDRED,
            strike: Positive::ZERO,
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: None,
            spot_min: Some(pos_or_panic!(100.0)),
            spot_max: None,
            exotic_params: None,
        };
        assert_eq!(floating_strike_payoff(&info), Decimal::ZERO);
    }

    #[test]
    fn test_put_option_spot_equals_max() {
        let info = PayoffInfo {
            spot: Positive::HUNDRED,
            strike: Positive::ZERO,
            style: OptionStyle::Put,
            side: Side::Long,
            spot_prices: None,
            spot_min: None,
            spot_max: Some(pos_or_panic!(100.0)),
            exotic_params: None,
        };
        assert_eq!(floating_strike_payoff(&info), Decimal::ZERO);
    }
}

#[cfg(test)]
mod test_asian_options {
    use crate::model::types::AsianAveragingType;
    use crate::model::{OptionStyle, OptionType, Side};
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    use crate::model::payoff::{Payoff, PayoffInfo};

    #[test]
    fn test_asian_arithmetic_put() {
        let option = OptionType::Asian {
            averaging_type: AsianAveragingType::Arithmetic,
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(90.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Put,
            side: Side::Long,
            spot_prices: Some(vec![
                pos_or_panic!(85.0),
                pos_or_panic!(90.0),
                pos_or_panic!(95.0),
            ]),
            ..Default::default()
        };
        assert_eq!(option.payoff(&info).unwrap(), dec!(10));
    }

    #[test]
    fn test_asian_no_spot_prices() {
        let option = OptionType::Asian {
            averaging_type: AsianAveragingType::Arithmetic,
        };
        let info = PayoffInfo {
            spot: Positive::HUNDRED,
            strike: Positive::HUNDRED,
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: None,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info).unwrap(), dec!(0));
    }
}

#[cfg(test)]
mod test_barrier_options {
    use crate::model::types::BarrierType;
    use crate::model::{OptionStyle, OptionType, Side};
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    use crate::model::payoff::{Payoff, PayoffInfo};

    #[test]
    fn test_barrier_down_and_in_put() {
        let option = OptionType::Barrier {
            barrier_type: BarrierType::DownAndIn,
            barrier_level: pos_or_panic!(110.0),
            rebate: None,
        };
        let info = PayoffInfo {
            spot: Positive::HUNDRED,
            strike: Positive::HUNDRED,
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: None,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info).unwrap(), dec!(0));
    }

    #[test]
    fn test_barrier_up_and_out_call() {
        let option = OptionType::Barrier {
            barrier_type: BarrierType::UpAndOut,
            barrier_level: pos_or_panic!(110.0),
            rebate: None,
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(120.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: None,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info).unwrap(), dec!(0));
    }
}

#[cfg(test)]
mod test_cliquet_options {
    use crate::model::{OptionStyle, OptionType, Side};
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    use crate::model::option::ExoticParams;
    use crate::model::payoff::{Payoff, PayoffInfo};
    use rust_decimal::Decimal;

    #[test]
    fn test_cliquet_option_with_resets() {
        let option = OptionType::Cliquet {
            reset_dates: vec![
                pos_or_panic!(30.0),
                pos_or_panic!(60.0),
                pos_or_panic!(90.0),
            ],
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(120.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: None,
            ..Default::default()
        };
        // No reset fixings reach the payoff, so nothing has accrued: 0, the
        // value `cliquet_black_scholes` gives at `T = 0`. It was the vanilla
        // payoff on the spot, 20 (#844).
        assert_eq!(option.payoff(&info).unwrap(), Decimal::ZERO);

        // The global floor and cap of the exotic parameters clamp it.
        let floored = PayoffInfo {
            exotic_params: Some(ExoticParams {
                cliquet_global_floor: Some(dec!(2)),
                ..ExoticParams::default()
            }),
            ..info.clone()
        };
        assert_eq!(option.payoff(&floored).unwrap(), dec!(2));
        let capped = PayoffInfo {
            exotic_params: Some(ExoticParams {
                cliquet_global_cap: Some(dec!(-1)),
                ..ExoticParams::default()
            }),
            ..info
        };
        assert_eq!(option.payoff(&capped).unwrap(), dec!(-1));
    }
}

#[cfg(test)]
mod test_rainbow_options {
    use crate::error::OptionsError;
    use crate::model::option::ExoticParams;
    use crate::model::{OptionStyle, OptionType, RainbowType, Side};
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    use crate::model::payoff::{Payoff, PayoffInfo};

    fn second_asset(price: f64) -> Option<ExoticParams> {
        Some(ExoticParams {
            rainbow_second_asset_price: Some(pos_or_panic!(price)),
            ..ExoticParams::default()
        })
    }

    #[test]
    fn test_rainbow_option_best_of() {
        let option = OptionType::Rainbow {
            num_assets: 2,
            rainbow_type: RainbowType::BestOf,
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(120.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: None,
            exotic_params: second_asset(130.0),
            ..Default::default()
        };
        // The best of 120 and 130 against 100 (#844); on the first asset
        // alone it read 20.
        assert_eq!(option.payoff(&info).unwrap(), dec!(30));

        // Without the second asset the payoff is undefined, as the pricer
        // rejects it.
        let missing = PayoffInfo {
            exotic_params: None,
            ..info
        };
        assert!(matches!(
            option.payoff(&missing),
            Err(OptionsError::PayoffError { .. })
        ));
    }

    #[test]
    fn test_rainbow_option_worst_of() {
        let option = OptionType::Rainbow {
            num_assets: 2,
            rainbow_type: RainbowType::WorstOf,
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(80.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Put,
            side: Side::Long,
            spot_prices: None,
            exotic_params: second_asset(70.0),
            ..Default::default()
        };
        // A put on the worst of 80 and 70 against 100 (#844).
        assert_eq!(option.payoff(&info).unwrap(), dec!(30));
    }
}

#[cfg(test)]
mod test_exchange_options {
    use crate::model::{OptionStyle, OptionType, Side};
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    use crate::model::payoff::{Payoff, PayoffInfo};

    #[test]
    fn test_exchange_option_positive_diff() {
        let option = OptionType::Exchange {
            second_asset: pos_or_panic!(90.0),
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(120.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: None,
            ..Default::default()
        };
        // `max(S1 - S2, 0) = 120 - 90`, with no strike; it was the vanilla
        // call on 120 at 100, 20 (#844).
        assert_eq!(option.payoff(&info).unwrap(), dec!(30));
    }

    #[test]
    fn test_exchange_option_negative_diff() {
        let option = OptionType::Exchange {
            second_asset: pos_or_panic!(110.0),
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(110.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: None,
            ..Default::default()
        };
        // Both assets at 110: nothing to gain from the exchange, 0; it was
        // the vanilla call on 110 at 100, 10 (#844).
        assert_eq!(option.payoff(&info).unwrap(), dec!(0));
    }
}

#[cfg(test)]
mod tests_option_type {
    use super::*;
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    #[test]
    fn test_asian_geometric_call() {
        let option = OptionType::Asian {
            averaging_type: AsianAveragingType::Geometric,
        };
        let info = PayoffInfo {
            spot: Positive::HUNDRED,
            strike: Positive::HUNDRED,
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: Some(vec![
                pos_or_panic!(90.0),
                pos_or_panic!(100.0),
                pos_or_panic!(110.0),
            ]),
            ..Default::default()
        };

        assert_eq!(option.payoff(&info).unwrap(), dec!(0));
    }

    #[test]
    fn test_asian_geometric_call_positive_payoff() {
        let option = OptionType::Asian {
            averaging_type: AsianAveragingType::Geometric,
        };
        let info = PayoffInfo {
            spot: Positive::HUNDRED,
            strike: pos_or_panic!(95.0),
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: Some(vec![
                pos_or_panic!(90.0),
                pos_or_panic!(100.0),
                pos_or_panic!(110.0),
            ]),
            ..Default::default()
        };

        let expected_payoff = dec!(4.67);
        assert!((option.payoff(&info).unwrap() - expected_payoff).abs() < dec!(0.01));
    }

    #[test]
    fn test_barrier_down_and_out_put() {
        let option = OptionType::Barrier {
            barrier_type: BarrierType::DownAndOut,
            barrier_level: pos_or_panic!(90.0),
            rebate: None,
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(95.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Put,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info).unwrap(), dec!(5));
    }

    #[test]
    fn test_binary_asset_or_nothing_put() {
        let option = OptionType::Binary {
            binary_type: BinaryType::AssetOrNothing,
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(90.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Put,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info).unwrap(), dec!(90));
    }

    #[test]
    fn test_compound_option() {
        let inner_option = OptionType::European;
        let option = OptionType::Compound {
            underlying_option: Box::new(inner_option),
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(110.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Call,
            side: Side::Long,
            ..Default::default()
        };
        // The underlying call is worth 10 at expiry; a call on it struck at
        // 100 is out of the money, 0. It returned the underlying's 10,
        // ignoring the compound strike (#844).
        assert_eq!(option.payoff(&info).unwrap(), dec!(0));
        let cheap = PayoffInfo {
            spot: pos_or_panic!(110.0),
            strike: pos_or_panic!(4.0),
            style: OptionStyle::Call,
            side: Side::Long,
            ..Default::default()
        };
        // Strike 4: the underlying call on 110 at 4 is worth 106, so the
        // compound call pays `106 - 4 = 102`.
        assert_eq!(option.payoff(&cheap).unwrap(), dec!(102));
    }

    #[test]
    fn test_chooser_option() {
        let option = OptionType::Chooser {
            choice_date: pos_or_panic!(30.0),
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(110.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Call,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info).unwrap(), dec!(10));
    }

    #[test]
    fn test_power_put() {
        let option = OptionType::Power {
            exponent: pos_or_panic!(2.0),
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(8.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Put,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info).unwrap(), dec!(36));
    }
}

#[cfg(test)]
mod tests_standard_payoff {
    use super::*;
    use crate::model::types::OptionType;
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    #[test]
    fn test_call_option_in_the_money() {
        let option_type = OptionType::European;
        let info = PayoffInfo {
            spot: pos_or_panic!(110.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: None,
            spot_min: None,
            spot_max: None,
            exotic_params: None,
        };
        assert_eq!(option_type.payoff(&info).unwrap(), dec!(10));
    }

    #[test]
    fn test_call_option_at_the_money() {
        let option_type = OptionType::European;
        let info = PayoffInfo {
            spot: Positive::HUNDRED,
            strike: Positive::HUNDRED,
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: None,
            spot_min: None,
            spot_max: None,
            exotic_params: None,
        };
        assert_eq!(option_type.payoff(&info).unwrap(), dec!(0));
    }

    #[test]
    fn test_call_option_out_of_the_money() {
        let option_type = OptionType::European;
        let info = PayoffInfo {
            spot: pos_or_panic!(90.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: None,
            spot_min: None,
            spot_max: None,
            exotic_params: None,
        };
        assert_eq!(option_type.payoff(&info).unwrap(), dec!(0));
    }

    #[test]
    fn test_put_option_in_the_money() {
        let option_type = OptionType::European;
        let info = PayoffInfo {
            spot: pos_or_panic!(90.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Put,
            side: Side::Long,
            spot_prices: None,
            spot_min: None,
            spot_max: None,
            exotic_params: None,
        };
        assert_eq!(option_type.payoff(&info).unwrap(), dec!(10));
    }

    #[test]
    fn test_put_option_at_the_money() {
        let option_type = OptionType::European;
        let info = PayoffInfo {
            spot: Positive::HUNDRED,
            strike: Positive::HUNDRED,
            style: OptionStyle::Put,
            side: Side::Long,
            spot_prices: None,
            spot_min: None,
            spot_max: None,
            exotic_params: None,
        };
        assert_eq!(option_type.payoff(&info).unwrap(), dec!(0));
    }

    #[test]
    fn test_put_option_out_of_the_money() {
        let option_type = OptionType::European;
        let info = PayoffInfo {
            spot: pos_or_panic!(110.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Put,
            side: Side::Long,
            spot_prices: None,
            spot_min: None,
            spot_max: None,
            exotic_params: None,
        };
        assert_eq!(option_type.payoff(&info).unwrap(), dec!(0));
    }
}

/// Pins the `Decimal` results of the `Payoff` boundary (#637) to the values
/// the former `f64` signature returned, converted with `Decimal::from_f64`.
#[cfg(test)]
mod tests_decimal_boundary_equivalence {
    use super::*;
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    fn info(spot: Positive, strike: Positive, style: OptionStyle, side: Side) -> PayoffInfo {
        PayoffInfo {
            spot,
            strike,
            style,
            side,
            ..Default::default()
        }
    }

    #[test]
    fn test_payoff_asian_geometric_call_pins_legacy_f64_digits() {
        let option = OptionType::Asian {
            averaging_type: AsianAveragingType::Geometric,
        };
        let info = PayoffInfo {
            spot_prices: Some(vec![
                pos_or_panic!(90.0),
                pos_or_panic!(100.0),
                pos_or_panic!(110.0),
            ]),
            ..info(
                Positive::HUNDRED,
                pos_or_panic!(95.0),
                OptionStyle::Call,
                Side::Long,
            )
        };
        // Re-baselined by #806: the geometric mean is the centred log-sum,
        // 99.66554934125965 in `f64`, where the product form gave
        // 99.6655493412596; the exact mean is 99.6655493412596363794..., so
        // the payoff moves from 4.66554934125961 to 4.66554934125965 against
        // an exact 4.6655493412596363794....
        assert_eq!(option.payoff(&info).unwrap(), dec!(4.66554934125965));
    }

    #[test]
    fn test_payoff_short_side_negates_long_payoff() {
        let option = OptionType::European;
        let long = info(
            pos_or_panic!(110.0),
            Positive::HUNDRED,
            OptionStyle::Call,
            Side::Long,
        );
        let short = PayoffInfo {
            side: Side::Short,
            ..long.clone()
        };
        assert_eq!(option.payoff(&long).unwrap(), dec!(10));
        assert_eq!(option.payoff(&short).unwrap(), dec!(-10));
    }

    #[test]
    fn test_payoff_fractional_strike_is_exact() {
        let option = OptionType::European;
        let info = info(
            pos_or_panic!(101.25),
            pos_or_panic!(100.5),
            OptionStyle::Call,
            Side::Long,
        );
        assert_eq!(option.payoff(&info).unwrap(), dec!(0.75));
    }

    #[test]
    fn test_payoff_up_and_in_barrier_uses_spot_max() {
        let option = OptionType::Barrier {
            barrier_type: BarrierType::UpAndIn,
            barrier_level: pos_or_panic!(120.0),
            rebate: None,
        };
        let touched = PayoffInfo {
            spot_max: Some(pos_or_panic!(125.0)),
            ..info(
                pos_or_panic!(110.0),
                Positive::HUNDRED,
                OptionStyle::Call,
                Side::Long,
            )
        };
        let untouched = PayoffInfo {
            spot_max: Some(pos_or_panic!(115.0)),
            ..touched.clone()
        };
        assert_eq!(option.payoff(&touched).unwrap(), dec!(10));
        assert_eq!(option.payoff(&untouched).unwrap(), Decimal::ZERO);
    }

    #[test]
    fn test_payoff_down_and_out_barrier_uses_spot_min_and_pays_rebate() {
        let option = OptionType::Barrier {
            barrier_type: BarrierType::DownAndOut,
            barrier_level: pos_or_panic!(90.0),
            rebate: Some(pos_or_panic!(2.5)),
        };
        let knocked_out = PayoffInfo {
            spot_min: Some(pos_or_panic!(85.0)),
            ..info(
                pos_or_panic!(95.0),
                Positive::HUNDRED,
                OptionStyle::Put,
                Side::Long,
            )
        };
        assert_eq!(option.payoff(&knocked_out).unwrap(), dec!(2.5));
    }

    fn barrier_with_rebate(barrier_type: BarrierType, level: f64) -> OptionType {
        OptionType::Barrier {
            barrier_type,
            barrier_level: pos_or_panic!(level),
            rebate: Some(pos_or_panic!(3.0)),
        }
    }

    // An unhit knock-in pays its rebate at expiry (Haug's `E` term); it
    // paid zero before #826.
    #[test]
    fn test_payoff_unhit_knock_in_pays_the_rebate() {
        let spot = Positive::HUNDRED;
        for (barrier_type, level) in [
            (BarrierType::DownAndIn, 95.0),
            (BarrierType::UpAndIn, 105.0),
        ] {
            let option = barrier_with_rebate(barrier_type, level);
            for style in [OptionStyle::Call, OptionStyle::Put] {
                let long = info(spot, pos_or_panic!(90.0), style, Side::Long);
                let short = info(spot, pos_or_panic!(90.0), style, Side::Short);
                assert_eq!(
                    option.payoff(&long).unwrap(),
                    dec!(3),
                    "{barrier_type:?} {style:?}"
                );
                assert_eq!(
                    option.payoff(&short).unwrap(),
                    dec!(-3),
                    "{barrier_type:?} {style:?}"
                );
            }
        }
    }

    // A hit knock-in is the vanilla payoff, with no rebate.
    #[test]
    fn test_payoff_hit_knock_in_is_the_vanilla_payoff() {
        let option = barrier_with_rebate(BarrierType::DownAndIn, 95.0);
        let hit = PayoffInfo {
            spot_min: Some(pos_or_panic!(94.0)),
            ..info(
                Positive::HUNDRED,
                pos_or_panic!(90.0),
                OptionStyle::Call,
                Side::Long,
            )
        };
        assert_eq!(option.payoff(&hit).unwrap(), dec!(10));
    }

    // The rebate of a hit knock-out is paid by a short; it came back with a
    // positive sign before #826.
    #[test]
    fn test_payoff_hit_knock_out_rebate_is_signed_by_side() {
        let option = barrier_with_rebate(BarrierType::UpAndOut, 105.0);
        let long = info(
            pos_or_panic!(106.0),
            Positive::HUNDRED,
            OptionStyle::Call,
            Side::Long,
        );
        let short = info(
            pos_or_panic!(106.0),
            Positive::HUNDRED,
            OptionStyle::Call,
            Side::Short,
        );
        assert_eq!(option.payoff(&long).unwrap(), dec!(3));
        assert_eq!(option.payoff(&short).unwrap(), dec!(-3));
    }

    // Without a rebate an unhit knock-in still pays nothing, for either side.
    #[test]
    fn test_payoff_unhit_knock_in_without_rebate_is_zero() {
        let option = OptionType::Barrier {
            barrier_type: BarrierType::DownAndIn,
            barrier_level: pos_or_panic!(95.0),
            rebate: Some(Positive::ZERO),
        };
        let short = info(
            Positive::HUNDRED,
            pos_or_panic!(90.0),
            OptionStyle::Call,
            Side::Short,
        );
        let payoff = option.payoff(&short).unwrap();
        assert_eq!(payoff, Decimal::ZERO);
        assert!(!payoff.is_sign_negative());
    }

    // `Options::payoff` passes no path extremes, so the barrier is judged
    // from the expiry spot alone: a down-and-in whose spot ends above the
    // barrier is unhit and pays its rebate, scaled by the position size.
    #[test]
    fn test_options_payoff_judges_the_barrier_from_the_spot() {
        let mut option =
            crate::model::utils::create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.option_type = barrier_with_rebate(BarrierType::DownAndIn, 95.0);
        option.underlying_price = Positive::HUNDRED;
        option.quantity = Positive::TWO;
        assert_eq!(option.payoff().unwrap(), dec!(6));
        option.underlying_price = pos_or_panic!(95.0);
        let vanilla_at_95 = OptionType::European
            .payoff(&info(
                pos_or_panic!(95.0),
                option.strike_price,
                OptionStyle::Call,
                Side::Long,
            ))
            .unwrap();
        assert_eq!(option.payoff().unwrap(), vanilla_at_95 * dec!(2));
    }

    #[test]
    fn test_payoff_floating_strike_lookback_through_trait() {
        let option = OptionType::Lookback {
            lookback_type: LookbackType::FloatingStrike,
        };
        let call = PayoffInfo {
            spot_min: Some(pos_or_panic!(80.0)),
            ..info(
                Positive::HUNDRED,
                Positive::ZERO,
                OptionStyle::Call,
                Side::Long,
            )
        };
        let put = PayoffInfo {
            spot_max: Some(pos_or_panic!(120.0)),
            ..info(
                Positive::HUNDRED,
                Positive::ZERO,
                OptionStyle::Put,
                Side::Long,
            )
        };
        assert_eq!(option.payoff(&call).unwrap(), dec!(20));
        assert_eq!(option.payoff(&put).unwrap(), dec!(20));
    }

    #[test]
    fn test_payoff_binary_gap_and_asset_or_nothing() {
        let gap = OptionType::Binary {
            binary_type: BinaryType::Gap,
        };
        let asset = OptionType::Binary {
            binary_type: BinaryType::AssetOrNothing,
        };
        let itm_call = info(
            pos_or_panic!(112.5),
            Positive::HUNDRED,
            OptionStyle::Call,
            Side::Long,
        );
        assert_eq!(gap.payoff(&itm_call).unwrap(), dec!(12.5));
        assert_eq!(asset.payoff(&itm_call).unwrap(), dec!(112.5));
    }

    /// `S^n` overflows `f64` for a power call, so the kernel value is `+∞`:
    /// the boundary reports it instead of returning a non-finite number.
    #[test]
    fn test_payoff_unrepresentable_power_call_is_error() {
        let option = OptionType::Power {
            exponent: Positive::MAX,
        };
        let info = info(
            Positive::HUNDRED,
            Positive::HUNDRED,
            OptionStyle::Call,
            Side::Long,
        );
        assert!(matches!(
            option.payoff(&info),
            Err(OptionsError::PayoffError { .. })
        ));
    }

    /// A call at `Positive::MAX` struck at zero pays `Decimal::MAX`
    /// exactly. Through the `f64` kernel before #844 the nearest `f64`
    /// rounded above `Decimal::MAX` and the payoff was an error.
    #[test]
    fn test_payoff_at_decimal_max_is_exact() {
        let option = OptionType::European;
        let info = info(Positive::MAX, Positive::ZERO, OptionStyle::Call, Side::Long);
        assert_eq!(option.payoff(&info).unwrap(), Decimal::MAX);
    }
}

#[cfg(test)]
mod tests_geometric_mean {
    use super::*;
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    /// Distance between two finite `f64`s of the same sign, in units in the
    /// last place.
    fn ulps(a: f64, b: f64) -> u64 {
        (a.to_bits() as i64 - b.to_bits() as i64).unsigned_abs()
    }

    fn fixings(values: impl IntoIterator<Item = Decimal>) -> Vec<Positive> {
        values
            .into_iter()
            .map(|value| Positive::new_decimal(value).unwrap())
            .collect()
    }

    /// The reference geometric means below are `exp(mean(ln x_i))` evaluated
    /// in 60-digit decimal arithmetic (Python's `decimal`, whose `ln` and
    /// `exp` are correctly rounded) on the exact fixings, then rounded to the
    /// nearest `f64`. The kernel must land within 3 ulps of each, its error
    /// bound over 2 001 random fixing sets (#806).
    const REFERENCE_ULPS: u64 = 3;

    #[test]
    fn test_geometric_mean_matches_the_reference() {
        let cases: [(Vec<Positive>, f64); 4] = [
            // 99.665549341259636379441742406...
            (fixings([dec!(90), dec!(100), dec!(110)]), 99.66554934125963),
            // 10000 exactly.
            (fixings(std::iter::repeat_n(dec!(10000), 100)), 10000.0),
            // 10 000 + 10 i for i < 252: 11231.426621138467604475800653...
            (
                fixings((0..252).map(|i| dec!(10000) + Decimal::from(10 * i))),
                11231.426621138467,
            ),
            // Nine orders of magnitude apart: 72.112478515370419616462753...
            (
                fixings([dec!(0.01), dec!(1000000), dec!(37.5)]),
                72.11247851537043,
            ),
        ];
        for (values, reference) in cases {
            let mean = match geometric_mean(&values) {
                Ok(mean) => mean,
                Err(e) => panic!("geometric mean must convert: {e:?}"),
            };
            assert!(
                ulps(mean, reference) <= REFERENCE_ULPS,
                "{} fixings: {mean} against {reference}",
                values.len()
            );
        }
    }

    /// 100 fixings at `1e4` multiply to `1e400`, past `f64::MAX`: the product
    /// form returned an infinite average and the payoff an error. The
    /// log-sum gives the average, `1e4`, exactly (#806).
    #[test]
    fn test_geometric_asian_with_100_fixings_at_1e4_prices() {
        let option = OptionType::Asian {
            averaging_type: AsianAveragingType::Geometric,
        };
        for (style, strike, expected) in [
            (OptionStyle::Call, dec!(9000), dec!(1000)),
            (OptionStyle::Put, dec!(10500), dec!(500)),
        ] {
            let info = PayoffInfo {
                spot: pos_or_panic!(10000.0),
                strike: Positive::new_decimal(strike).unwrap(),
                style,
                side: Side::Long,
                spot_prices: Some(vec![pos_or_panic!(10000.0); 100]),
                ..Default::default()
            };
            assert_eq!(option.payoff(&info).unwrap(), expected, "{style:?}");
        }
    }

    /// A zero fixing makes the geometric mean zero, as it made the product.
    #[test]
    fn test_geometric_mean_with_a_zero_fixing_is_zero() {
        for values in [
            fixings([dec!(0), dec!(100), dec!(110)]),
            fixings([dec!(90), dec!(100), dec!(0)]),
        ] {
            assert!(matches!(geometric_mean(&values), Ok(mean) if mean == 0.0));
        }
        assert!(matches!(geometric_mean(&[]), Ok(mean) if mean == 0.0));
    }
}

#[cfg(test)]
mod tests_terminal_payoff {
    //! The payoff of each family is its contract's terminal payoff, signed by
    //! the side (#844). `optionstratlib-pricing`'s `terminal_payoff_test`
    //! checks each against its kernel at `T = 0`; these pin the values.
    use super::*;
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    fn info(spot: f64, style: OptionStyle, side: Side) -> PayoffInfo {
        PayoffInfo {
            spot: pos_or_panic!(spot),
            strike: Positive::HUNDRED,
            style,
            side,
            ..Default::default()
        }
    }

    /// A short cash-or-nothing call in the money pays out 1; it read `+1`.
    #[test]
    fn test_short_binary_chooser_and_power_are_signed() {
        let short_call = info(105.0, OptionStyle::Call, Side::Short);
        let binary = OptionType::Binary {
            binary_type: BinaryType::CashOrNothing,
        };
        assert_eq!(binary.payoff(&short_call).unwrap(), dec!(-1));
        let chooser = OptionType::Chooser {
            choice_date: pos_or_panic!(30.0),
        };
        assert_eq!(chooser.payoff(&short_call).unwrap(), dec!(-5));
        let power = OptionType::Power {
            exponent: Positive::ONE,
        };
        assert_eq!(power.payoff(&short_call).unwrap(), dec!(-5));
        // Out of the money the short pays nothing, as an unsigned zero.
        let otm = info(95.0, OptionStyle::Call, Side::Short);
        let zero = binary.payoff(&otm).unwrap();
        assert!(zero.is_zero() && !zero.is_sign_negative());
    }

    /// With no observed extremes a floating-strike lookback locks nothing in:
    /// `0`, where it paid `S` (call) or `-S` (put). The extremes include the
    /// expiry spot.
    #[test]
    fn test_floating_strike_lookback_without_extremes_is_zero() {
        let lookback = OptionType::Lookback {
            lookback_type: LookbackType::FloatingStrike,
        };
        for style in [OptionStyle::Call, OptionStyle::Put] {
            assert_eq!(
                lookback.payoff(&info(105.0, style, Side::Long)).unwrap(),
                Decimal::ZERO
            );
        }
        let call = PayoffInfo {
            spot_min: Some(pos_or_panic!(90.0)),
            ..info(105.0, OptionStyle::Call, Side::Short)
        };
        assert_eq!(lookback.payoff(&call).unwrap(), dec!(-15));
        // A minimum above the spot is not the path's minimum: the spot is.
        let stale = PayoffInfo {
            spot_min: Some(pos_or_panic!(110.0)),
            ..info(105.0, OptionStyle::Call, Side::Long)
        };
        assert_eq!(lookback.payoff(&stale).unwrap(), Decimal::ZERO);
    }

    /// With no fixings the averaging window is the expiry instant, so the
    /// average is the spot: intrinsic value, 5, where it paid 0. With
    /// fixings a short position pays the long payoff; it received it.
    #[test]
    fn test_asian_payoff_without_fixings_is_intrinsic_and_signed() {
        let asian = OptionType::Asian {
            averaging_type: AsianAveragingType::Arithmetic,
        };
        assert_eq!(
            asian
                .payoff(&info(105.0, OptionStyle::Call, Side::Long))
                .unwrap(),
            dec!(5)
        );
        let fixed = PayoffInfo {
            spot_prices: Some(vec![pos_or_panic!(104.0), pos_or_panic!(108.0)]),
            ..info(105.0, OptionStyle::Call, Side::Short)
        };
        assert_eq!(asian.payoff(&fixed).unwrap(), dec!(-6));
    }

    /// A call on a call, both struck at 100, with the spot at 105: the
    /// underlying is worth 5, below the compound strike, so the compound pays
    /// 0. It paid the underlying's 5 (#844).
    #[test]
    fn test_compound_pays_on_its_strike() {
        let compound = OptionType::Compound {
            underlying_option: Box::new(OptionType::European),
        };
        assert_eq!(
            compound
                .payoff(&info(105.0, OptionStyle::Call, Side::Long))
                .unwrap(),
            Decimal::ZERO
        );
        // The underlying takes the compound's style, as in
        // `compound_black_scholes`: a put on a put at 105 is a put struck at
        // 100 on a worthless put, `100 - 0 = 100`.
        assert_eq!(
            compound
                .payoff(&info(105.0, OptionStyle::Put, Side::Long))
                .unwrap(),
            dec!(100)
        );
    }

    /// The vanilla payoff is the exact `Decimal` difference: `50.07 - 50` was
    /// `0.0700000000000003` through `f64`.
    #[test]
    fn test_vanilla_payoff_is_exact() {
        let call = PayoffInfo {
            spot: Positive::new_decimal(dec!(50.07)).unwrap(),
            strike: pos_or_panic!(50.0),
            style: OptionStyle::Call,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(OptionType::European.payoff(&call).unwrap(), dec!(0.07));
    }
}
