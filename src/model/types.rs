use crate::constants::ZERO;
use crate::pricing::payoff::{Payoff, PayoffInfo, standard_payoff};
use crate::{ExpirationDate, Positive};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::Hash;
use utoipa::ToSchema;

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

/// Represents different types of assets that can be held in a balance.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, Default)]
pub enum UnderlyingAssetType {
    /// Cryptocurrency assets (e.g., BTC, ETH)
    Crypto,
    /// Stock/equity assets (e.g., AAPL, GOOGL)
    #[default]
    Stock,
    /// Options contracts
    Forex,
    /// Commodity assets (e.g., Gold, Oil)
    Commodity,
    /// Bond/fixed income securities
    Bond,
    /// Other asset types
    Other,
}

impl fmt::Display for UnderlyingAssetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnderlyingAssetType::Crypto => write!(f, "Crypto"),
            UnderlyingAssetType::Stock => write!(f, "Stock"),
            UnderlyingAssetType::Forex => write!(f, "Forex"),
            UnderlyingAssetType::Commodity => write!(f, "Commodity"),
            UnderlyingAssetType::Bond => write!(f, "Bond"),
            UnderlyingAssetType::Other => write!(f, "Other"),
        }
    }
}

/// Represents trading actions in a financial context.
///
/// This enum defines the fundamental trade operations that can be performed
/// in a trading system. These actions represent the direction of a trade
/// transaction.
///
/// `Action` is used to indicate whether a security is being acquired or disposed of,
/// and is commonly paired with other transaction details such as price, quantity,
/// and timing information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, ToSchema)]
pub enum Action {
    /// Represents a purchase transaction, where assets are acquired.
    #[default]
    Buy,
    /// Represents a selling transaction, where assets are disposed of.
    Sell,

    /// Action is not applicable to this type of transaction.
    Other,
}

/// Defines the directional exposure of a financial position.
///
/// This enum represents the market sentiment or directional bias of a position.
/// It indicates whether a trader expects to profit from rising prices (Long)
/// or falling prices (Short).
///
/// `Side` is a fundamental concept in trading that determines how profits and losses
/// are calculated and affects risk management considerations.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, ToSchema)]
pub enum Side {
    /// Represents a position that profits when the underlying asset's price increases.
    /// Long positions involve buying an asset with the expectation of selling at a higher price.
    #[default]
    Long,
    /// Represents a position that profits when the underlying asset's price decreases.
    /// Short positions involve selling an asset (often borrowed) with the expectation
    /// of buying it back at a lower price.
    Short,
}

/// Specifies the style of an option contract.
///
/// This enum defines the fundamental classification of options contracts based on
/// their exercise characteristics. The style determines when and how an option
/// can be exercised.
///
/// `OptionStyle` is a critical attribute for options contracts as it directly
/// affects valuation, pricing models, and exercise strategies.
#[derive(
    Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, Ord, PartialOrd, ToSchema,
)]
pub enum OptionStyle {
    /// Represents a call option, which gives the holder the right (but not obligation)
    /// to buy the underlying asset at the strike price before or at expiration.
    /// Call options typically increase in value when the underlying asset price rises.
    #[default]
    Call,
    /// Represents a put option, which gives the holder the right (but not obligation)
    /// to sell the underlying asset at the strike price before or at expiration.
    /// Put options typically increase in value when the underlying asset price falls.
    Put,
}

/// Represents the type of option in a financial context.
/// Options can be categorized into various types based on their characteristics and the conditions under which they can be exercised.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, ToSchema)]
pub enum OptionType {
    /// A European option can only be exercised at the expiry date.
    /// This type of option does not allow the holder to exercise the option before the specified expiration date.
    /// European options are simpler to price and analyze because their payoff is only determined at a single point in time.
    #[default]
    European,

    /// An American option can be exercised at any time before and including the expiry date.
    /// This provides the holder with more flexibility compared to European options, as the holder can choose the optimal time to exercise the option based on market conditions.
    /// The ability to exercise at any point adds complexity to the pricing model, typically requiring binomial or numerical methods.
    American,

    /// A Bermuda option can be exercised on specific dates before the expiry date.
    /// These specified dates are usually predetermined and occur at regular intervals (e.g., monthly or quarterly).
    /// Bermuda options offer a compromise between the flexibility of American options and the simplicity of European options.
    Bermuda {
        /// The specific dates on which the option can be exercised before expiry.
        exercise_dates: Vec<f64>,
    },

    /// An Asian option, where the payoff depends on the average price of the underlying asset over a certain period.
    /// There are two types of averaging methods: arithmetic and geometric.
    /// Asian options are useful for reducing the risk of market manipulation at the expiry date and are common in commodities markets.
    Asian {
        /// The method used to calculate the average price (arithmetic or geometric).
        averaging_type: AsianAveragingType,
    },

    /// A Barrier option becomes active or inactive only if the underlying asset reaches a certain barrier level.
    /// These options can be classified into knock-in or knock-out, and further into up-and-in, up-and-out, down-and-in, and down-and-out.
    /// Barrier options are used for hedging strategies and typically have lower premiums compared to standard options.
    Barrier {
        /// The type of barrier that triggers the option's activation or deactivation.
        barrier_type: BarrierType,
        /// The specific level that the underlying asset must reach for the barrier to be triggered.
        barrier_level: f64,
    },

    /// A Binary option provides a fixed payoff if the underlying asset is above or below a certain level at expiry.
    /// Also known as digital options, they include cash-or-nothing and asset-or-nothing types.
    /// Binary options are simpler to understand but can be riskier due to their all-or-nothing payoff structure.
    Binary {
        /// The specific type of binary option.
        binary_type: BinaryType,
    },

    /// A Lookback option allows the holder to "look back" over time and determine the payoff based on the maximum or minimum underlying asset price during the option's life.
    /// There are two main types: fixed strike, where the strike price is set at the beginning, and floating strike, where the strike price is set at the maximum or minimum observed price.
    /// Lookback options are useful for maximizing profit and are typically more expensive due to their enhanced payoff structure.
    Lookback {
        /// The specific type of lookback option.
        lookback_type: LookbackType,
    },

    /// A Compound option has an option as its underlying asset.
    /// This means the holder has the right to buy or sell another option.
    /// Compound options can be nested, adding layers of optionality and complexity, and are useful in structured finance and corporate finance.
    #[serde(skip)]
    #[schema(skip)] // DO NOT SERIALIZE THIS TYPE
    Compound {
        /// The underlying option, which can be any type of option, adding a layer of complexity.
        underlying_option: Box<OptionType>,
    },

    /// A Chooser option allows the holder to choose, at a certain date, whether the option will be a call or a put.
    /// This flexibility allows the holder to make a decision based on the prevailing market conditions at the choice date.
    /// Chooser options are valuable in volatile markets but can be expensive due to their flexibility.
    Chooser {
        /// The specific date on which the holder must choose whether the option becomes a call or a put.
        choice_date: f64,
    },

    /// A Cliquet option, also known as a ratchet option, resets its strike price at certain dates.
    /// This allows the option to capture gains periodically, locking in profits and reducing downside risk.
    /// Cliquet options are complex and often used in structured products and guaranteed equity bonds.
    Cliquet {
        /// The specific dates on which the strike price is reset.
        reset_dates: Vec<f64>,
    },

    /// A Rainbow option is based on the performance of two or more underlying assets.
    /// The payoff is typically based on the best or worst performing asset, or a combination of their performances.
    /// Rainbow options are useful for diversifying risk across multiple assets and are common in multi-asset portfolios.
    Rainbow {
        /// The number of underlying assets the option is based on.
        num_assets: usize,
    },

    /// A Spread option is based on the difference between the prices of two underlying assets.
    /// These options are used to profit from the relative performance of two assets, often in the same sector or market.
    /// Spread options can be used for arbitrage opportunities and to hedge against relative price movements.
    Spread {
        /// The price of the second asset involved in the spread.
        second_asset: f64,
    },

    /// A Quanto option has a payoff that depends on the underlying asset price in one currency, but the payoff is made in another currency at a fixed exchange rate.
    /// This type of option eliminates the currency risk for investors in a different currency zone.
    /// Quanto options are common in international markets where investors seek exposure to foreign assets without taking on currency risk.
    Quanto {
        /// The fixed exchange rate at which the payoff is converted.
        exchange_rate: f64,
    },

    /// An Exchange option gives the holder the right to exchange one asset for another.
    /// These options are often used in mergers and acquisitions, where one company's stock can be exchanged for another's.
    /// Exchange options provide flexibility in managing different asset exposures and can be tailored for specific corporate events.
    Exchange {
        /// The price of the second asset involved in the exchange.
        second_asset: f64,
    },

    /// A Power option has a payoff based on the underlying asset price raised to a certain power.
    /// This can amplify the gains (or losses) based on the underlying asset's performance.
    /// Power options are exotic derivatives and are used for speculative purposes and in scenarios where large movements in the underlying asset are expected.
    Power {
        /// The exponent to which the underlying asset price is raised.
        exponent: f64,
    },
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
            } => calculate_barrier_payoff(barrier_type, barrier_level, info),
            OptionType::Binary { binary_type } => calculate_binary_payoff(binary_type, info),
            OptionType::Lookback { lookback_type } => match lookback_type {
                LookbackType::FixedStrike => standard_payoff(info),
                LookbackType::FloatingStrike => calculate_floating_strike_payoff(info),
            },
            OptionType::Compound { underlying_option } => underlying_option.payoff(info),
            OptionType::Chooser { .. } => (info.spot - info.strike)
                .max(Positive::ZERO)
                .max(
                    (info.strike.to_dec() - info.spot.to_dec())
                        .max(Decimal::ZERO)
                        .into(),
                )
                .to_f64(),
            OptionType::Cliquet { .. } => standard_payoff(info),
            OptionType::Rainbow { .. }
            | OptionType::Spread { .. }
            | OptionType::Exchange { .. } => standard_payoff(info),
            OptionType::Quanto { exchange_rate } => standard_payoff(info) * exchange_rate,
            OptionType::Power { exponent } => match info.style {
                OptionStyle::Call => (info.spot.to_f64().powf(*exponent) - info.strike).max(ZERO),
                OptionStyle::Put => (info.strike - info.spot.to_f64().powf(*exponent))
                    .max(Positive::ZERO)
                    .to_f64(),
            },
        }
    }
}

/// A structure representing the basic properties of an option in financial terms.
/// This structure is designed to be lightweight and provides essential details
/// about an options contract.
///
/// # Generic Parameters
/// - `'a`: A lifetime parameter that ensures the references within the structure
///   are valid for the same lifetime.
///
/// # Fields
///
/// # Derives
/// - `Clone`: Enables creating a copy of the structure.
/// - `Copy`: Allows the structure to be copied instead of moved.
/// - `PartialEq`: Enables comparison for equality between two instances of the structure.
/// - `Serialize`: Provides functionality for serializing the structure into a format like JSON or others.
/// - `Debug`: Enables formatting the structure for debugging purposes.
/// - `Hash`: Makes the type hashable, allowing it to be stored in hash-based collections, such as `HashMap`.
/// - `Eq`: Indicates that the type guarantees the equality operator `==` is reflexive, symmetric, and transitive.
///
/// # Usage
/// This struct is ideal for applications dealing with option contracts where
/// the essential characteristics of an option need to be stored and managed efficiently.
///
/// # Example
/// ```rust
/// use optionstratlib::model::types::OptionBasicType;
/// use optionstratlib::{pos_or_panic, ExpirationDate, OptionStyle, Positive, Side};
/// let european_call_option = OptionBasicType {
///     option_style: &OptionStyle::Call,
///     side: &Side::Long,
///     strike_price: &Positive::new(100.0).unwrap(),
///     expiration_date: &ExpirationDate::Days(pos_or_panic!(30.0)),
/// };
/// ```
#[derive(Clone, Copy, PartialEq, Serialize, Debug, Hash, Eq, ToSchema)]
pub struct OptionBasicType<'a> {
    /// - `option_style`: A reference to the style of the option (e.g., European
    ///   or American) represented by the `OptionStyle` type.
    pub option_style: &'a OptionStyle,
    /// - `side`: A reference to the side of the option (e.g., Call or Put)
    ///   as defined by the `Side` type.
    pub side: &'a Side,
    /// - `strike_price`: A reference to the strike price of the option, which is
    ///   guaranteed to be positive, represented by the `Positive` type.
    pub strike_price: &'a Positive,
    /// - `expiration_date`: A reference to the expiration date of the option,
    ///   represented by the `ExpirationDate` type.
    pub expiration_date: &'a ExpirationDate,
}

/// Describes how the average price is calculated for Asian options.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum AsianAveragingType {
    /// Arithmetic averaging sums all observed prices and divides by the number of observations.
    Arithmetic,
    /// Geometric averaging takes the nth root of the product of n observed prices.
    Geometric,
}

/// Describes the type of barrier for Barrier options.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum BarrierType {
    /// The option becomes active if the underlying asset price goes above a certain level.
    UpAndIn,
    /// The option becomes inactive if the underlying asset price goes above a certain level.
    UpAndOut,
    /// The option becomes active if the underlying asset price goes below a certain level.
    DownAndIn,
    /// The option becomes inactive if the underlying asset price goes below a certain level.
    DownAndOut,
}

/// Represents different types of binary options, which are financial instruments that provide a fixed payout based on whether certain conditions are met.
///
/// # Variants
///
/// - `CashOrNothing`:
///   The option pays a fixed cash amount if the underlying asset's value is above or below a predefined level.
///
/// - `AssetOrNothing`:
///   The option pays the value of the underlying asset itself if the underlying asset's price is above or below a predefined level.
///
/// - `Gap`:
///   Pays out based on how far the underlying asset price is above the strike price at expiration.
///   The payout is proportional to the difference between the asset price and the strike price.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum BinaryType {
    /// The option pays a fixed amount of cash if the underlying asset is above or below a certain level.
    CashOrNothing,
    /// The option pays the value of the underlying asset if it is above or below a certain level.
    AssetOrNothing,
    /// Pays out if the underlying asset price is above the strike price at expiration, with the payout proportional to how far above the strike it is.
    Gap,
}

/// Describes the type of lookback option.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum LookbackType {
    /// The strike price is fixed at the beginning, and the payoff is based on the maximum or minimum price of the underlying asset during the option's life.
    FixedStrike,
    /// The strike price is determined as the maximum or minimum price of the underlying asset during the option's life, providing the holder with the most advantageous strike price.
    FloatingStrike,
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
    barrier_level: &f64,
    info: &PayoffInfo,
) -> f64 {
    let barrier_condition = match barrier_type {
        BarrierType::UpAndIn | BarrierType::UpAndOut => info.spot >= *barrier_level,
        BarrierType::DownAndIn | BarrierType::DownAndOut => info.spot <= *barrier_level,
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
                0.0
            } else {
                std_payoff
            }
        }
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
    use positive::pos_or_panic;
    use super::*;

    #[test]
    fn test_european_call() {
        let option = OptionType::European;
        let info = PayoffInfo {
            spot: pos_or_panic!(110.0),
            strike: pos_or_panic!(100.0),
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
            strike: pos_or_panic!(100.0),
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
            spot: pos_or_panic!(100.0),
            strike: pos_or_panic!(100.0),
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
            barrier_level: 120.0,
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(130.0),
            strike: pos_or_panic!(100.0),
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
            strike: pos_or_panic!(100.0),
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
            strike: pos_or_panic!(100.0),
            style: OptionStyle::Put,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info), 10.0);
    }

    #[test]
    fn test_quanto_call() {
        let option = OptionType::Quanto { exchange_rate: 1.5 };
        let info = PayoffInfo {
            spot: pos_or_panic!(110.0),
            strike: pos_or_panic!(100.0),
            style: OptionStyle::Call,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info), 15.0);
    }

    #[test]
    fn test_power_call() {
        let option = OptionType::Power { exponent: 2.0 };
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
    use positive::pos_or_panic;
    use super::*;

    #[test]
    fn test_call_option_with_spot_min() {
        let info = PayoffInfo {
            spot: pos_or_panic!(100.0),
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
            spot: pos_or_panic!(100.0),
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
            spot: pos_or_panic!(100.0),
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
            spot: pos_or_panic!(100.0),
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
            spot: pos_or_panic!(100.0),
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
            spot: pos_or_panic!(100.0),
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
    use positive::pos_or_panic;
    use super::*;

    #[test]
    fn test_asian_geometric_call() {
        let option = OptionType::Asian {
            averaging_type: AsianAveragingType::Geometric,
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(100.0),
            strike: pos_or_panic!(100.0),
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
            spot: pos_or_panic!(100.0),
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
            barrier_level: 90.0,
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(95.0),
            strike: pos_or_panic!(100.0),
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
            strike: pos_or_panic!(100.0),
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
            strike: pos_or_panic!(100.0),
            style: OptionStyle::Call,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info), 10.0);
    }

    #[test]
    fn test_chooser_option() {
        let option = OptionType::Chooser { choice_date: 30.0 };
        let info = PayoffInfo {
            spot: pos_or_panic!(110.0),
            strike: pos_or_panic!(100.0),
            style: OptionStyle::Call,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info), 10.0);
    }

    #[test]
    fn test_power_put() {
        let option = OptionType::Power { exponent: 2.0 };
        let info = PayoffInfo {
            spot: pos_or_panic!(8.0),
            strike: pos_or_panic!(100.0),
            style: OptionStyle::Put,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info), 36.0);
    }
}

#[cfg(test)]
mod tests_vec_collection {
    use positive::pos_or_panic;
    use crate::model::positive::Positive;

    #[test]
    fn test_collect_empty_iterator() {
        let empty_vec: Vec<Positive> = Vec::new();
        let collected: Vec<Positive> = empty_vec.into_iter().collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn test_collect_single_value() {
        let values = vec![pos_or_panic!(1.0)];
        let collected: Vec<Positive> = values.into_iter().collect();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0], pos_or_panic!(1.0));
    }

    #[test]
    fn test_collect_multiple_values() {
        let values = vec![pos_or_panic!(1.0), pos_or_panic!(2.0), pos_or_panic!(3.0)];
        let collected: Vec<Positive> = values.into_iter().collect();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0], pos_or_panic!(1.0));
        assert_eq!(collected[1], pos_or_panic!(2.0));
        assert_eq!(collected[2], pos_or_panic!(3.0));
    }

    #[test]
    fn test_collect_from_filter() {
        let values = vec![
            pos_or_panic!(1.0),
            pos_or_panic!(2.0),
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
        let values = vec![pos_or_panic!(1.0), pos_or_panic!(2.0), pos_or_panic!(3.0)];
        let collected: Vec<Positive> = values
            .into_iter()
            .map(|x| pos_or_panic!(x.to_f64() * 2.0))
            .collect();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0], pos_or_panic!(2.0));
        assert_eq!(collected[1], pos_or_panic!(4.0));
        assert_eq!(collected[2], pos_or_panic!(6.0));
    }

    #[test]
    fn test_collect_from_chain() {
        let values1 = vec![pos_or_panic!(1.0), pos_or_panic!(2.0)];
        let values2 = vec![pos_or_panic!(3.0), pos_or_panic!(4.0)];
        let collected: Vec<Positive> = values1.into_iter().chain(values2).collect();
        assert_eq!(collected.len(), 4);
        assert_eq!(collected[0], pos_or_panic!(1.0));
        assert_eq!(collected[1], pos_or_panic!(2.0));
        assert_eq!(collected[2], pos_or_panic!(3.0));
        assert_eq!(collected[3], pos_or_panic!(4.0));
    }
}

#[cfg(test)]
mod test_asian_options {
    use positive::pos_or_panic;
    use crate::model::types::AsianAveragingType;
    use crate::model::{OptionStyle, OptionType, Side};

    use crate::pricing::{Payoff, PayoffInfo};

    #[test]
    fn test_asian_arithmetic_put() {
        let option = OptionType::Asian {
            averaging_type: AsianAveragingType::Arithmetic,
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(90.0),
            strike: pos_or_panic!(100.0),
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
            spot: pos_or_panic!(100.0),
            strike: pos_or_panic!(100.0),
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
    use positive::pos_or_panic;
    use crate::model::types::BarrierType;
    use crate::model::{OptionStyle, OptionType, Side};

    use crate::pricing::{Payoff, PayoffInfo};

    #[test]
    fn test_barrier_down_and_in_put() {
        let option = OptionType::Barrier {
            barrier_type: BarrierType::DownAndIn,
            barrier_level: 90.0,
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(100.0),
            strike: pos_or_panic!(100.0),
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
            barrier_level: 110.0,
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(120.0),
            strike: pos_or_panic!(100.0),
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
    use positive::pos_or_panic;
    use crate::model::{OptionStyle, OptionType, Side};

    use crate::pricing::{Payoff, PayoffInfo};

    #[test]
    fn test_cliquet_option_with_resets() {
        let option = OptionType::Cliquet {
            reset_dates: vec![30.0, 60.0, 90.0],
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(120.0),
            strike: pos_or_panic!(100.0),
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
    use positive::pos_or_panic;
    use crate::model::{OptionStyle, OptionType, Side};

    use crate::pricing::{Payoff, PayoffInfo};

    #[test]
    fn test_rainbow_option_multiple_assets() {
        let option = OptionType::Rainbow { num_assets: 3 };
        let info = PayoffInfo {
            spot: pos_or_panic!(120.0),
            strike: pos_or_panic!(100.0),
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: None,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info), 20.0);
    }
}

#[cfg(test)]
mod test_exchange_options {
    use positive::pos_or_panic;
    use crate::model::{OptionStyle, OptionType, Side};

    use crate::pricing::{Payoff, PayoffInfo};

    #[test]
    fn test_exchange_option_positive_diff() {
        let option = OptionType::Exchange { second_asset: 90.0 };
        let info = PayoffInfo {
            spot: pos_or_panic!(120.0),
            strike: pos_or_panic!(100.0),
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
            second_asset: 110.0,
        };
        let info = PayoffInfo {
            spot: pos_or_panic!(110.0),
            strike: pos_or_panic!(100.0),
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: None,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info), 10.0);
    }
}
