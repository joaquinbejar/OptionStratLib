//! Payoff contracts at expiry.
//!
//! The payoff of a contract at expiry is a pure function of its terms (spot,
//! strike, style, side and option type). It needs no pricing model, rate,
//! volatility or time to expiry, so it is a domain invariant owned by the
//! core model rather than by the pricing layer.

use crate::constants::ZERO;
use crate::error::{OptionsError, OptionsResult};
use crate::model::decimal::finite_decimal;
use crate::model::types::{
    AsianAveragingType, BarrierType, BinaryType, LookbackType, OptionStyle, OptionType, Side,
};
use num_traits::{FromPrimitive, ToPrimitive};
use positive::Positive;
use rust_decimal::Decimal;
use tracing::{trace, warn};

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
    ///   This field is used specifically for Lookback options where the payoff depends on the
    ///   minimum price reached.
    pub spot_min: Option<Positive>, // Lookback / down barriers
    /// * `spot_max` - The maximum observed price of the underlying asset during the option's life.
    ///   This field is used specifically for Lookback options where the payoff depends on the
    ///   maximum price reached.
    pub spot_max: Option<Positive>, // Lookback / up barriers
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

/// Calculates the standard payoff for an option given its information.
///
/// # Arguments
///
/// * `info` - A reference to a `PayoffInfo` struct that contains the essential details of the option such as its style, spot price, and strike price.
///
/// # Returns
///
/// * `f64` - The payoff value based on the type of the option (call or put).
///
/// This function evaluates the payoff based on the option style:
/// - For a call option: Max(spot price - strike price, 0)
/// - For a put option: Max(strike price - spot price, 0)
#[inline]
pub(crate) fn standard_payoff(info: &PayoffInfo) -> f64 {
    let spot: Decimal = info.spot.into();
    let strike: Decimal = info.strike.into();

    // `Positive - Positive` aborts whenever the result would be negative, i.e.
    // on every out-of-the-money call, so the moneyness is traced on the
    // `Decimal` values. Both operands live in `[0, Decimal::MAX]`, which makes
    // the difference always representable; the checked form keeps the raw
    // operator out of the kernel.
    let moneyness = spot.checked_sub(strike).unwrap_or(Decimal::ZERO);
    trace!("standard_payoff - spot: {}", spot);
    trace!("standard_payoff - info.strike: {}", strike);
    trace!("standard_payoff - (info.spot - info.strike): {}", moneyness);

    // The result of `(spot - strike).max(ZERO)` is a non-negative finite Decimal whenever
    // the inputs are themselves finite (which Positive guarantees), so to_f64 is expected
    // to succeed. We log and fall back to 0.0 instead of panicking if conversion ever fails.
    let payoff = match info.style {
        OptionStyle::Call => moneyness.max(Decimal::ZERO).to_f64().unwrap_or_else(|| {
            warn!(
                spot = %spot,
                strike = %strike,
                "standard_payoff(Call): to_f64 returned None; defaulting to 0.0"
            );
            0.0
        }),
        OptionStyle::Put => strike
            .checked_sub(spot)
            .unwrap_or(Decimal::ZERO)
            .max(Decimal::ZERO)
            .to_f64()
            .unwrap_or_else(|| {
                warn!(
                    spot = %spot,
                    strike = %strike,
                    "standard_payoff(Put): to_f64 returned None; defaulting to 0.0"
                );
                0.0
            }),
    };

    match info.side {
        Side::Long => payoff,
        Side::Short => -payoff,
    }
}

impl Payoff for OptionType {
    /// Evaluates the payoff in the private `f64` kernel and converts the
    /// result through the checked [`finite_decimal`] at the boundary.
    fn payoff(&self, info: &PayoffInfo) -> OptionsResult<Decimal> {
        let value = option_type_payoff(self, info);
        finite_decimal(value).ok_or_else(|| payoff_not_representable(value))
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

/// The `f64` payoff kernel behind [`Payoff`] for [`OptionType`].
///
/// Private: the public boundary is the `Decimal` returned by
/// [`Payoff::payoff`]. The `Positive` spot inputs of [`PayoffInfo`] are
/// widened to `f64` here, which is exact for every value an `f64` literal
/// can spell.
fn option_type_payoff(option_type: &OptionType, info: &PayoffInfo) -> f64 {
    match option_type {
        OptionType::European | OptionType::American => standard_payoff(info),
        OptionType::Bermuda { .. } => standard_payoff(info),
        OptionType::Asian { averaging_type } => calculate_asian_payoff(averaging_type, info),
        OptionType::Barrier {
            barrier_type,
            barrier_level,
            rebate,
        } => calculate_barrier_payoff(barrier_type, barrier_level, rebate, info),
        OptionType::Binary { binary_type } => calculate_binary_payoff(binary_type, info),
        OptionType::Lookback { lookback_type } => match lookback_type {
            LookbackType::FixedStrike => standard_payoff(info),
            LookbackType::FloatingStrike => calculate_floating_strike_payoff(info),
            // `LookbackType` is `#[non_exhaustive]`.
            _ => standard_payoff(info),
        },
        OptionType::Compound { underlying_option } => option_type_payoff(underlying_option, info),
        OptionType::Chooser { .. } => {
            // The chooser is worth the better of the two intrinsics at
            // expiry. `Positive - Positive` aborts whenever the result
            // would be negative, i.e. for every out-of-the-money chooser,
            // so both legs are formed on `Decimal` — where the difference
            // of two values in `[0, Decimal::MAX]` is always
            // representable — and floored at zero before the comparison.
            let call_intrinsic = info
                .spot
                .to_dec()
                .checked_sub(info.strike.to_dec())
                .unwrap_or(Decimal::ZERO)
                .max(Decimal::ZERO);
            let put_intrinsic = info
                .strike
                .to_dec()
                .checked_sub(info.spot.to_dec())
                .unwrap_or(Decimal::ZERO)
                .max(Decimal::ZERO);
            Positive::new_decimal(call_intrinsic.max(put_intrinsic))
                .unwrap_or(Positive::ZERO)
                .to_f64()
        }
        OptionType::Cliquet { .. } => standard_payoff(info),
        OptionType::Rainbow { .. } | OptionType::Spread { .. } | OptionType::Exchange { .. } => {
            standard_payoff(info)
        }
        OptionType::Quanto { exchange_rate } => standard_payoff(info) * exchange_rate.to_f64(),
        OptionType::Power { exponent } => match info.style {
            OptionStyle::Call => {
                (info.spot.to_f64().powf(exponent.to_f64()) - info.strike.to_f64()).max(ZERO)
            }
            OptionStyle::Put => {
                // `Positive - f64` aborts whenever the result would be
                // negative, i.e. for every out-of-the-money power put, and
                // also when `S^n` has no `Decimal` representation. The
                // difference keeps the `Decimal` arithmetic the `Positive`
                // operator performed and is floored at zero; an `S^n` that
                // leaves the representable range is `+∞` in the limit,
                // where the put is worthless.
                let powered = info.spot.to_f64().powf(exponent.to_f64());
                match finite_decimal(powered) {
                    Some(powered_dec) => Positive::new_decimal(
                        info.strike
                            .to_dec()
                            .checked_sub(powered_dec)
                            .unwrap_or(Decimal::ZERO)
                            .max(Decimal::ZERO),
                    )
                    .unwrap_or(Positive::ZERO)
                    .to_f64(),
                    None => ZERO,
                }
            }
        },
        // `OptionType` is `#[non_exhaustive]`: a variant added upstream falls
        // back to the plain intrinsic value until it gets its own arm.
        _ => standard_payoff(info),
    }
}

/// Calculates the payoff of an Asian option based on the average spot prices.
///
/// # Parameters
/// - `averaging_type`: Specifies the method of averaging the spot prices. It can either be:
///   - `AsianAveragingType::Arithmetic`: Uses arithmetic mean for averaging.
///   - `AsianAveragingType::Geometric`: Uses geometric mean for averaging.
/// - `info`: A reference to a `PayoffInfo` object containing the details about the option such as
///   the spot prices, strike price, and option style (Call or Put).
///
/// # Returns
/// - The calculated payoff as a `f64`. If the spot prices are not present or their length is zero,
///   it will return ZERO (assumed to be a constant defined elsewhere).
///
/// # Calculation
/// - The function first calculates the average of the given spot prices based on the specified `averaging_type`.
/// - For arithmetic averaging, the sum of the spot prices is computed, divided by the number of prices.
/// - For geometric averaging, the mean is the log-sum `exp(mean(ln x_i))`; see
///   [`geometric_mean`].
/// - If the averaging fails due to invalid input (e.g., missing or zero-length spot prices), the result is ZERO.
///
/// - Once the average is calculated, the payoff is computed based on the option style:
///   - For a `Call` option: The payoff is the maximum of `(average - strike)` or ZERO.
///   - For a `Put` option: The payoff is the maximum of `(strike - average)` or ZERO.
///
/// # Assumptions:
/// - The `spot_prices` and their length (`spot_prices_len()`) are correctly passed via the `PayoffInfo` object.
/// - Constants `ZERO` and behavior for `Positive::ZERO.into()` are defined elsewhere in the code base.
///
fn calculate_asian_payoff(averaging_type: &AsianAveragingType, info: &PayoffInfo) -> f64 {
    let average = match (&info.spot_prices, info.spot_prices_len()) {
        (Some(spot_prices), Some(len)) if len > 0 => match averaging_type {
            AsianAveragingType::Arithmetic => {
                spot_prices.iter().map(Positive::to_f64).sum::<f64>() / len as f64
            }
            AsianAveragingType::Geometric => geometric_mean(spot_prices),
            // `AsianAveragingType` is `#[non_exhaustive]`: fall back to the
            // arithmetic mean, the conventional default for Asian options.
            _ => spot_prices.iter().map(Positive::to_f64).sum::<f64>() / len as f64,
        },
        _ => return ZERO,
    };
    match info.style {
        OptionStyle::Call => (average - info.strike.to_f64()).max(ZERO),
        // `Positive - f64` aborted on every out-of-the-money Asian put, where
        // the average is above the strike, and on a non-finite average
        // (#788). The difference is the one that operator formed, on
        // `Decimal`, floored at zero. A non-finite average (the geometric
        // product left the `f64` range before #806 took the mean as a
        // log-sum) has no put value, so it goes out as `NaN`, which the
        // `Decimal` boundary of `Payoff::payoff` reports as an error, as it
        // already does for the call.
        OptionStyle::Put => match Decimal::from_f64(average) {
            Some(average) => Positive::new_decimal(
                info.strike
                    .to_dec()
                    .checked_sub(average)
                    .unwrap_or(Decimal::ZERO)
                    .max(Decimal::ZERO),
            )
            .unwrap_or(Positive::ZERO)
            .to_f64(),
            None => f64::NAN,
        },
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
fn geometric_mean(fixings: &[Positive]) -> f64 {
    let Some(first) = fixings.first() else {
        return ZERO;
    };
    if fixings.contains(&Positive::ZERO) {
        return ZERO;
    }
    let centre = first.to_f64();
    let log_sum: f64 = fixings
        .iter()
        .map(|fixing| (fixing.to_f64() / centre).ln()) // scan-banned: allow -- f64 `ln` of a positive finite ratio, it does not abort
        .sum();
    centre * (log_sum / fixings.len() as f64).exp() // scan-banned: allow -- f64 `exp`: returns inf on overflow, it does not abort; a non-finite payoff is rejected at the `Decimal` boundary
}

/// Calculates the payoff for a financial instrument with a barrier feature.
///
/// # Arguments
///
/// * `barrier_type` - Specifies the type of barrier condition. Can be one of the following:
///     - `BarrierType::UpAndIn`: Payoff is only valid if the spot price has risen above or to the barrier level.
///     - `BarrierType::DownAndIn`: Payoff is only valid if the spot price has fallen below or to the barrier level.
///     - `BarrierType::UpAndOut`: Payoff is only valid if the spot price does not rise above the barrier level.
///     - `BarrierType::DownAndOut`: Payoff is only valid if the spot price does not fall below the barrier level.
/// * `barrier_level` - A reference to the barrier level price, which serves as the activation or deactivation threshold for the payoff.
/// * `info` - Contains information required to calculate the payoff, including the spot price and additional data for standard payoff calculations.
///
/// # Returns
///
/// Returns the calculated payoff as a `f64`. If the barrier conditions are met, the payoff will either be the standard payoff or zero, based on the barrier type.
///
/// # Behavior
///
/// 1. Evaluates whether the current spot price satisfies the barrier condition based on the given `barrier_type` and `barrier_level`.
/// 2. If the condition for an "In" type (`UpAndIn` or `DownAndIn`) barrier is met, the standard payoff is returned; otherwise, it returns `0.0`.
/// 3. If the condition for an "Out" type (`UpAndOut` or `DownAndOut`) barrier is met, the payoff is `0.0`; otherwise, it returns the standard payoff.
///
/// # Assumptions
///
/// * It is assumed that the `standard_payoff` function is defined elsewhere and provides the base payoff calculation.
/// * The `PayoffInfo` struct and the `BarrierType` enum are pre-defined and accessible in the same context.
///
/// # Errors
///
/// This function does not explicitly handle errors. Ensure that the inputs are valid for the `barrier_type`, `barrier_level`, and `info` parameters.
fn calculate_barrier_payoff(
    barrier_type: &BarrierType,
    barrier_level: &Positive,
    rebate: &Option<Positive>,
    info: &PayoffInfo,
) -> f64 {
    let level = barrier_level.to_f64();
    let barrier_condition = match barrier_type {
        BarrierType::UpAndIn | BarrierType::UpAndOut => {
            // Use spot_max if available, otherwise just current spot
            info.spot_max.unwrap_or(info.spot).to_f64() >= level
        }
        BarrierType::DownAndIn | BarrierType::DownAndOut => {
            // Use spot_min if available, otherwise just current spot
            info.spot_min.unwrap_or(info.spot).to_f64() <= level
        }
        // `BarrierType` is `#[non_exhaustive]`: an unknown barrier is treated as
        // never triggered, so the "In" arms below pay nothing and the "Out" arms
        // pay the standard payoff.
        _ => false,
    };
    let std_payoff = standard_payoff(info);
    match barrier_type {
        BarrierType::UpAndIn | BarrierType::DownAndIn => {
            if barrier_condition {
                std_payoff
            } else {
                0.0
            }
        }
        BarrierType::UpAndOut | BarrierType::DownAndOut => {
            if barrier_condition {
                rebate.map_or(0.0, |r| r.to_f64())
            } else {
                std_payoff
            }
        }
        _ => std_payoff,
    }
}

/// Calculates the payout for a binary option based on its type and associated payoff details.
///
/// # Parameters
///
/// - `binary_type`: An enum (`BinaryType`) representing the type of binary option. Supported types are:
///   - `CashOrNothing`: Pays a fixed amount (1.0) if the option expires in-the-money; otherwise, pays 0.0.
///   - `AssetOrNothing`: Pays the current spot price of the asset if the option expires in-the-money; otherwise, pays 0.0.
///   - `Gap`: Pays the absolute difference between the spot price and the strike price (if in-the-money); otherwise, pays 0.0.
///
/// - `info`: A reference to a `PayoffInfo` struct containing the following fields:
///   - `spot`: The current price of the underlying asset.
///   - `strike`: The strike price of the option.
///   - `style`: An enum (`OptionStyle`) representing whether the option is a call (long) or put (short):
///     - `Call`: In-the-money if `spot > strike`.
///     - `Put`: In-the-money if `spot < strike`.
///
/// # Returns
///
/// - A `f64` value representing the calculated payoff of the binary option based on the provided conditions.
///
/// # Logic
///
/// 1. Determine whether the option is in-the-money based on its style (`Call` or `Put`) and the relationship
///    between the `spot` price and the `strike` price.
///
/// 2. Calculate the payoff based on the type of binary option:
///
///    - **CashOrNothing**: Returns `1.0` if the option is in-the-money; otherwise, returns `0.0`.
///    - **AssetOrNothing**: Returns the `spot` price (converted into `f64`) if the option is in-the-money; otherwise, returns `0.0`.
///    - **Gap**: Returns the absolute difference between the `spot` and `strike` prices (converted into `f64`) if the option is in-the-money; otherwise, returns `0.0`.
///
/// # Notes
///
/// - The `to_f64` method is assumed to be implemented for the type of `spot` and `strike` to ensure compatibility with the calculations.
/// - The definition and behavior of `BinaryType`, `PayoffInfo`, and `OptionStyle` are external to this function.
///
fn calculate_binary_payoff(binary_type: &BinaryType, info: &PayoffInfo) -> f64 {
    let is_in_the_money = match info.style {
        OptionStyle::Call => info.spot > info.strike,
        OptionStyle::Put => info.spot < info.strike,
    };
    match binary_type {
        BinaryType::CashOrNothing => {
            if is_in_the_money {
                1.0
            } else {
                0.0
            }
        }
        BinaryType::AssetOrNothing => {
            if is_in_the_money {
                info.spot.to_f64()
            } else {
                0.0
            }
        }
        BinaryType::Gap => {
            if is_in_the_money {
                // For Gap options, the payoff is proportional to how far above/below the strike price
                // the underlying asset is at expiration
                (info.spot.to_f64() - info.strike.to_f64()).abs()
            } else {
                0.0
            }
        }
        // `BinaryType` is `#[non_exhaustive]`: an unknown binary type pays the
        // cash-or-nothing amount, the most conservative of the three.
        _ => {
            if is_in_the_money {
                1.0
            } else {
                0.0
            }
        }
    }
}

/// Calculates the payoff for a floating strike option based on the provided option information.
///
/// # Parameters
/// - `info`: A reference to a `PayoffInfo` struct that contains all necessary information for
///   calculating the payoff. The struct includes details such as the option style (call or put),
///   the spot value, and the minimum or maximum spot observed (as applicable).
///
/// # Returns
/// - A `f64` representing the calculated payoff amount for the floating strike option.
///
/// # Logic
/// 1. Determines the "extremum" based on the option style:
///    - For a call option (`OptionStyle::Call`), the extremum is the minimum spot value (`info.spot_min`).
///    - For a put option (`OptionStyle::Put`), the extremum is the maximum spot value (`info.spot_max`).
/// 2. Calculates the payoff based on the difference between the spot price (`info.spot.to_f64()`)
///    and the extremum:
///    - For a call option, the payoff is `spot - extremum` (or `spot` if `extremum` is unavailable).
///    - For a put option, the payoff is `extremum - spot` (or `-spot` if `extremum` is unavailable).
///
/// # Assumptions
/// - `info.to_f64()` correctly converts the spot value to a floating-point number (`f64`).
/// - `info.spot_min` and `info.spot_max` are `Option<f64>` values that might be `None`, in which case
///   the fallback value (`ZERO`) is used in the payoff calculation.
///
/// # Notes
/// - Ensure that the `info.spot.to_f64()` implementation and the extremum values (`spot_min`, `spot_max`)
///   are compatible with your application's floating-point requirements.
/// - The function handles missing extremum values gracefully using a default value of `ZERO`.
///
fn calculate_floating_strike_payoff(info: &PayoffInfo) -> f64 {
    let extremum = match info.style {
        OptionStyle::Call => info.spot_min,
        OptionStyle::Put => info.spot_max,
    };
    match info.style {
        OptionStyle::Call => info.spot.to_f64() - extremum.map_or(ZERO, |e| e.to_f64()),
        OptionStyle::Put => extremum.map_or(ZERO, |e| e.to_f64()) - info.spot.to_f64(),
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
        };
        assert_eq!(calculate_floating_strike_payoff(&info), 20.0);
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
        };
        assert_eq!(calculate_floating_strike_payoff(&info), 100.0);
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
        };
        assert_eq!(calculate_floating_strike_payoff(&info), 20.0);
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
        };
        assert_eq!(calculate_floating_strike_payoff(&info), -100.0);
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
        };
        assert_eq!(calculate_floating_strike_payoff(&info), 0.0);
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
        };
        assert_eq!(calculate_floating_strike_payoff(&info), 0.0);
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

    use crate::model::payoff::{Payoff, PayoffInfo};

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
        assert_eq!(option.payoff(&info).unwrap(), dec!(20));
    }
}

#[cfg(test)]
mod test_rainbow_options {
    use crate::model::{OptionStyle, OptionType, RainbowType, Side};
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    use crate::model::payoff::{Payoff, PayoffInfo};

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
            ..Default::default()
        };
        assert_eq!(option.payoff(&info).unwrap(), dec!(20));
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
            ..Default::default()
        };
        assert_eq!(option.payoff(&info).unwrap(), dec!(20));
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
        assert_eq!(option.payoff(&info).unwrap(), dec!(20));
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
        assert_eq!(option.payoff(&info).unwrap(), dec!(10));
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
        assert_eq!(option.payoff(&info).unwrap(), dec!(10));
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

    /// The `f64` nearest to a call payoff at `Positive::MAX` rounds above
    /// `Decimal::MAX`, so the payoff has no `Decimal` representation.
    #[test]
    fn test_payoff_beyond_decimal_range_is_error() {
        let option = OptionType::European;
        let info = info(Positive::MAX, Positive::ZERO, OptionStyle::Call, Side::Long);
        assert!(matches!(
            option.payoff(&info),
            Err(OptionsError::PayoffError { .. })
        ));
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
            let mean = geometric_mean(&values);
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
            assert_eq!(geometric_mean(&values), 0.0);
        }
        assert_eq!(geometric_mean(&[]), 0.0);
    }
}
