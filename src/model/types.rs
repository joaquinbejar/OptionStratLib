/// Re-export of core financial types from the standalone `financial_types` crate.
///
/// This module re-exports fundamental trading enums (`Action`, `Side`,
/// `OptionStyle`, `UnderlyingAssetType`) and option contract type definitions
/// (`OptionType`, sub-enums, `OptionBasicType`) from their respective
/// external crates.
pub use financial_types::{Action, OptionStyle, Side, UnderlyingAssetType};
pub use option_type::{
    AsianAveragingType, BarrierType, BinaryType, LookbackType, OptionBasicType, OptionType,
    RainbowType,
};

use crate::constants::ZERO;
use crate::pricing::payoff::{Payoff, PayoffInfo, standard_payoff};
use chrono::{DateTime, Utc};
use positive::Positive;
use rust_decimal::Decimal;

mod datetime_format {
    use super::*;
    use serde::{self, Deserialize, Deserializer, Serializer};

    #[allow(dead_code)]
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date.to_rfc3339();
        serializer.serialize_str(&s)
    }

    #[allow(dead_code)]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(serde::de::Error::custom)
    }
}

impl Payoff for OptionType {
    fn payoff(&self, info: &PayoffInfo) -> f64 {
        match self {
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
            OptionType::Compound { underlying_option } => underlying_option.payoff(info),
            OptionType::Chooser { .. } => (info.spot - info.strike)
                .max(Positive::ZERO)
                .max(
                    Positive::new_decimal(
                        (info.strike.to_dec() - info.spot.to_dec()).max(Decimal::ZERO),
                    )
                    .unwrap_or(Positive::ZERO),
                )
                .to_f64(),
            OptionType::Cliquet { .. } => standard_payoff(info),
            OptionType::Rainbow { .. }
            | OptionType::Spread { .. }
            | OptionType::Exchange { .. } => standard_payoff(info),
            OptionType::Quanto { exchange_rate } => standard_payoff(info) * exchange_rate.to_f64(),
            OptionType::Power { exponent } => match info.style {
                OptionStyle::Call => {
                    (info.spot.to_f64().powf(exponent.to_f64()) - info.strike).max(ZERO)
                }
                OptionStyle::Put => (info.strike - info.spot.to_f64().powf(exponent.to_f64()))
                    .max(Positive::ZERO)
                    .to_f64(),
            },
            // `OptionType` is `#[non_exhaustive]`: a variant added upstream falls
            // back to the plain intrinsic value until it gets its own arm.
            _ => standard_payoff(info),
        }
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
/// - For geometric averaging, the product of the spot prices is computed and the nth root of the product
///   is taken, where `n` is the number of prices.
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
            AsianAveragingType::Arithmetic => spot_prices.iter().sum::<f64>() / len as f64,
            AsianAveragingType::Geometric => {
                let product = spot_prices.iter().fold(1.0, |acc, &x| acc * x);
                product.powf(1.0 / len as f64)
            }
            // `AsianAveragingType` is `#[non_exhaustive]`: fall back to the
            // arithmetic mean, the conventional default for Asian options.
            _ => spot_prices.iter().sum::<f64>() / len as f64,
        },
        _ => return ZERO,
    };
    match info.style {
        OptionStyle::Call => (average - info.strike).max(ZERO),
        OptionStyle::Put => (info.strike - average).max(Positive::ZERO).into(),
    }
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
            info.spot_max.unwrap_or(info.spot.to_f64()) >= level
        }
        BarrierType::DownAndIn | BarrierType::DownAndOut => {
            // Use spot_min if available, otherwise just current spot
            info.spot_min.unwrap_or(info.spot.to_f64()) <= level
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
        OptionStyle::Call => info.spot.to_f64() - extremum.unwrap_or(ZERO),
        OptionStyle::Put => extremum.unwrap_or(ZERO) - info.spot.to_f64(),
    }
}

#[cfg(test)]
mod tests_payoff {
    use super::*;
    use positive::{Positive, pos_or_panic};

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
        assert_eq!(option.payoff(&info), 10.0);
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
        assert_eq!(option.payoff(&info), 10.0);
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
            spot_prices: Some(vec![90.0, 100.0, 110.0]),
            ..Default::default()
        };
        assert_eq!(option.payoff(&info), ZERO);
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
        assert_eq!(option.payoff(&info), 30.0);
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
        assert_eq!(option.payoff(&info), 1.0);
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
        assert_eq!(option.payoff(&info), 10.0);
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
        assert_eq!(option.payoff(&info), 15.0);
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
        assert_eq!(option.payoff(&info), 10.0);
    }
}

#[cfg(test)]
mod tests_calculate_floating_strike_payoff {
    use super::*;

    #[test]
    fn test_call_option_with_spot_min() {
        let info = PayoffInfo {
            spot: Positive::HUNDRED,
            strike: Positive::ZERO, // Not used in floating strike
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: None,
            spot_min: Some(80.0),
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
            spot_max: Some(120.0),
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
            spot_min: Some(100.0),
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
            spot_max: Some(100.0),
        };
        assert_eq!(calculate_floating_strike_payoff(&info), 0.0);
    }
}

#[cfg(test)]
mod tests_option_type {
    use super::*;
    use positive::pos_or_panic;

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
            spot_prices: Some(vec![90.0, 100.0, 110.0]),
            ..Default::default()
        };

        assert_eq!(option.payoff(&info), 0.0);
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
            spot_prices: Some(vec![90.0, 100.0, 110.0]),
            ..Default::default()
        };

        let expected_payoff = 4.67;
        assert!((option.payoff(&info) - expected_payoff).abs() < 0.01);
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
        assert_eq!(option.payoff(&info), 5.0);
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
        assert_eq!(option.payoff(&info), 90.0);
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
        assert_eq!(option.payoff(&info), 10.0);
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
        assert_eq!(option.payoff(&info), 10.0);
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
        assert_eq!(option.payoff(&info), 36.0);
    }
}

#[cfg(test)]
mod tests_vec_collection {
    use positive::{Positive, pos_or_panic};

    #[test]
    fn test_collect_empty_iterator() {
        let empty_vec: Vec<Positive> = Vec::new();
        let collected: Vec<Positive> = empty_vec.into_iter().collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn test_collect_single_value() {
        let values = vec![Positive::ONE];
        let collected: Vec<Positive> = values.into_iter().collect();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0], Positive::ONE);
    }

    #[test]
    fn test_collect_multiple_values() {
        let values = vec![Positive::ONE, Positive::TWO, pos_or_panic!(3.0)];
        let collected: Vec<Positive> = values.into_iter().collect();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0], Positive::ONE);
        assert_eq!(collected[1], Positive::TWO);
        assert_eq!(collected[2], pos_or_panic!(3.0));
    }

    #[test]
    fn test_collect_from_filter() {
        let values = vec![
            Positive::ONE,
            Positive::TWO,
            pos_or_panic!(3.0),
            pos_or_panic!(4.0),
        ];
        let collected: Vec<Positive> = values.into_iter().filter(|x| x.to_f64() > 2.0).collect();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0], pos_or_panic!(3.0));
        assert_eq!(collected[1], pos_or_panic!(4.0));
    }

    #[test]
    fn test_collect_from_map() {
        let values = vec![Positive::ONE, Positive::TWO, pos_or_panic!(3.0)];
        let collected: Vec<Positive> = values
            .into_iter()
            .map(|x| pos_or_panic!(x.to_f64() * 2.0))
            .collect();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0], Positive::TWO);
        assert_eq!(collected[1], pos_or_panic!(4.0));
        assert_eq!(collected[2], pos_or_panic!(6.0));
    }

    #[test]
    fn test_collect_from_chain() {
        let values1 = vec![Positive::ONE, Positive::TWO];
        let values2 = vec![pos_or_panic!(3.0), pos_or_panic!(4.0)];
        let collected: Vec<Positive> = values1.into_iter().chain(values2).collect();
        assert_eq!(collected.len(), 4);
        assert_eq!(collected[0], Positive::ONE);
        assert_eq!(collected[1], Positive::TWO);
        assert_eq!(collected[2], pos_or_panic!(3.0));
        assert_eq!(collected[3], pos_or_panic!(4.0));
    }
}

#[cfg(test)]
mod test_asian_options {
    use crate::model::types::AsianAveragingType;
    use crate::model::{OptionStyle, OptionType, Side};
    use positive::{Positive, pos_or_panic};

    use crate::pricing::{Payoff, PayoffInfo};

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
            spot_prices: Some(vec![85.0, 90.0, 95.0]),
            ..Default::default()
        };
        assert_eq!(option.payoff(&info), 10.0);
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
        assert_eq!(option.payoff(&info), 0.0);
    }
}

#[cfg(test)]
mod test_barrier_options {
    use crate::model::types::BarrierType;
    use crate::model::{OptionStyle, OptionType, Side};
    use positive::{Positive, pos_or_panic};

    use crate::pricing::{Payoff, PayoffInfo};

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
        assert_eq!(option.payoff(&info), 0.0);
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
        assert_eq!(option.payoff(&info), 0.0);
    }
}

#[cfg(test)]
mod test_cliquet_options {
    use crate::model::{OptionStyle, OptionType, Side};
    use positive::{Positive, pos_or_panic};

    use crate::pricing::{Payoff, PayoffInfo};

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
        assert_eq!(option.payoff(&info), 20.0);
    }
}

#[cfg(test)]
mod test_rainbow_options {
    use crate::model::{OptionStyle, OptionType, RainbowType, Side};
    use positive::{Positive, pos_or_panic};

    use crate::pricing::{Payoff, PayoffInfo};

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
        assert_eq!(option.payoff(&info), 20.0);
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
        assert_eq!(option.payoff(&info), 20.0);
    }
}

#[cfg(test)]
mod test_exchange_options {
    use crate::model::{OptionStyle, OptionType, Side};
    use positive::{Positive, pos_or_panic};

    use crate::pricing::{Payoff, PayoffInfo};

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
        assert_eq!(option.payoff(&info), 20.0);
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
        assert_eq!(option.payoff(&info), 10.0);
    }
}
