use crate::constants::ZERO;
use crate::pricing::payoff::{standard_payoff, Payoff, PayoffInfo};
use approx::{AbsDiffEq, RelativeEq};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub};
use std::str::FromStr;

pub const PZERO: PositiveF64 = PositiveF64(ZERO);
pub const SIZE_ONE: PositiveF64 = PositiveF64(1.0);
pub const P_INFINITY: PositiveF64 = PositiveF64(f64::INFINITY);

#[derive(PartialEq, Clone, Copy)]
pub struct PositiveF64(f64);

#[macro_export]
macro_rules! pos {
    ($val:expr) => {
        PositiveF64::new($val).unwrap()
    };
}

#[macro_export]
macro_rules! spos {
    ($val:expr) => {
        Some(PositiveF64::new($val).unwrap())
    };
}

impl PositiveF64 {
    pub fn new(value: f64) -> Result<Self, String> {
        if value >= ZERO {
            Ok(PositiveF64(value))
        } else {
            Err(format!("PositiveF64 value must be positive, got {}", value))
        }
    }

    pub fn value(&self) -> f64 {
        self.0
    }

    pub fn max(self, other: PositiveF64) -> PositiveF64 {
        if self.0 > other.0 {
            self
        } else {
            other
        }
    }

    pub fn min(self, other: PositiveF64) -> PositiveF64 {
        if self.0 < other.0 {
            self
        } else {
            other
        }
    }

    pub fn floor(&self) -> PositiveF64 {
        PositiveF64(self.0.floor())
    }
}

impl From<PositiveF64> for f64 {
    fn from(pos_f64: PositiveF64) -> Self {
        pos_f64.0
    }
}

impl From<PositiveF64> for u64 {
    fn from(pos_u64: PositiveF64) -> Self {
        pos_u64.0 as u64
    }
}

impl PartialEq<f64> for PositiveF64 {
    fn eq(&self, other: &f64) -> bool {
        self.0 == *other
    }
}

impl fmt::Display for PositiveF64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(precision) = f.precision() {
            write!(f, "{:.1$}", self.0, precision)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl fmt::Debug for PositiveF64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(precision) = f.precision() {
            write!(f, "{:.1$}", self.0, precision)
        } else {
            write!(f, "{:?}", self.0)
        }
    }
}

impl Serialize for PositiveF64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(self.0)
    }
}

impl<'de> Deserialize<'de> for PositiveF64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        PositiveF64::new(value).map_err(serde::de::Error::custom)
    }
}

impl Add for PositiveF64 {
    type Output = PositiveF64;

    fn add(self, other: PositiveF64) -> PositiveF64 {
        PositiveF64(self.0 + other.0)
    }
}

impl Sub for PositiveF64 {
    type Output = PositiveF64;

    fn sub(self, rhs: Self) -> Self::Output {
        PositiveF64(self.0 - rhs.0)
    }
}

impl Div for PositiveF64 {
    type Output = PositiveF64;

    fn div(self, other: PositiveF64) -> PositiveF64 {
        PositiveF64(self.0 / other.0)
    }
}

impl Add<f64> for PositiveF64 {
    type Output = PositiveF64;

    fn add(self, rhs: f64) -> PositiveF64 {
        PositiveF64(self.0 + rhs)
    }
}

impl Sub<f64> for PositiveF64 {
    type Output = PositiveF64;

    fn sub(self, rhs: f64) -> PositiveF64 {
        PositiveF64(self.0 - rhs)
    }
}

impl AddAssign for PositiveF64 {
    fn add_assign(&mut self, other: PositiveF64) {
        self.0 += other.0;
    }
}

impl AddAssign<f64> for PositiveF64 {
    fn add_assign(&mut self, rhs: f64) {
        self.0 += rhs;
    }
}

impl MulAssign<f64> for PositiveF64 {
    fn mul_assign(&mut self, rhs: f64) {
        self.0 *= rhs;
    }
}

impl Div<f64> for PositiveF64 {
    type Output = PositiveF64;

    fn div(self, rhs: f64) -> PositiveF64 {
        PositiveF64(self.0 / rhs)
    }
}

impl PartialOrd for PositiveF64 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }

    fn le(&self, other: &Self) -> bool {
        self.0 <= other.0
    }

    fn ge(&self, other: &Self) -> bool {
        self.0 >= other.0
    }
}

impl Eq for PositiveF64 {}

impl Ord for PositiveF64 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

impl Neg for PositiveF64 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        panic!("Cannot negate a PositiveF64 value!");
    }
}

impl Mul for PositiveF64 {
    type Output = PositiveF64;

    fn mul(self, other: PositiveF64) -> PositiveF64 {
        PositiveF64(self.0 * other.0)
    }
}

impl Mul<f64> for PositiveF64 {
    type Output = PositiveF64;

    fn mul(self, rhs: f64) -> PositiveF64 {
        PositiveF64(self.0 * rhs)
    }
}

impl Default for PositiveF64 {
    fn default() -> Self {
        PositiveF64(ZERO)
    }
}

impl FromStr for PositiveF64 {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<f64>() {
            Ok(value) if value > 0.0 => Ok(PositiveF64(value)),
            Ok(value) => Err(format!("Value must be positive, got {}", value)),
            Err(e) => Err(format!("Failed to parse as f64: {}", e)),
        }
    }
}

impl From<f64> for PositiveF64 {
    fn from(value: f64) -> Self {
        PositiveF64::new(value).expect("Value must be positive")
    }
}

impl AbsDiffEq for PositiveF64 {
    type Epsilon = f64;

    fn default_epsilon() -> Self::Epsilon {
        f64::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        f64::abs_diff_eq(&self.0, &other.0, epsilon)
    }
}

impl RelativeEq for PositiveF64 {
    fn default_max_relative() -> Self::Epsilon {
        f64::default_max_relative()
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        f64::relative_eq(&self.0, &other.0, epsilon, max_relative)
    }
}

impl Sum for PositiveF64 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let sum = iter.fold(0.0, |acc, x| acc + x.value());
        PositiveF64::new(sum).unwrap_or(PZERO)
    }
}

impl<'a> Sum<&'a PositiveF64> for PositiveF64 {
    fn sum<I: Iterator<Item = &'a PositiveF64>>(iter: I) -> Self {
        let sum = iter.fold(0.0, |acc, x| acc + x.value());
        PositiveF64::new(sum).unwrap_or(PZERO)
    }
}

impl AddAssign<PositiveF64> for f64 {
    fn add_assign(&mut self, rhs: PositiveF64) {
        *self += rhs.0;
    }
}

impl Div<PositiveF64> for f64 {
    type Output = f64;

    fn div(self, rhs: PositiveF64) -> f64 {
        self / rhs.0
    }
}

impl Sub<PositiveF64> for f64 {
    type Output = f64;

    fn sub(self, rhs: PositiveF64) -> Self::Output {
        self - rhs.0
    }
}

impl Mul<PositiveF64> for f64 {
    type Output = f64;

    fn mul(self, rhs: PositiveF64) -> f64 {
        self * rhs.0
    }
}

impl Add<PositiveF64> for f64 {
    type Output = f64;

    fn add(self, rhs: PositiveF64) -> f64 {
        self + rhs.0
    }
}

#[allow(dead_code)]
#[derive(Clone, PartialEq)]
pub enum ExpirationDate {
    Days(f64),
    DateTime(DateTime<Utc>),
}

impl ExpirationDate {
    pub(crate) fn get_years(&self) -> f64 {
        match self {
            ExpirationDate::Days(days) => {
                if *days < 0.0 {
                    panic!("Days cannot be negative");
                }
                days / 365.0
            }
            ExpirationDate::DateTime(datetime) => {
                let now = Utc::now();
                let duration = datetime.signed_duration_since(now);
                let num_days = duration.num_days();
                if num_days < 0 {
                    panic!("DateTime results in negative duration");
                }
                num_days as f64 / 365.0
            }
        }
    }

    pub(crate) fn get_date(&self) -> DateTime<Utc> {
        match self {
            ExpirationDate::Days(days) => Utc::now() + Duration::days(*days as i64),
            ExpirationDate::DateTime(datetime) => *datetime,
        }
    }

    pub(crate) fn get_date_string(&self) -> String {
        let date = self.get_date();
        date.format("%Y-%m-%d").to_string()
    }
}

impl Default for ExpirationDate {
    fn default() -> Self {
        ExpirationDate::Days(365.0)
    }
}

#[allow(dead_code)]
#[derive(Clone, PartialEq)]
pub enum Side {
    Long,
    Short,
}

#[allow(dead_code)]
#[derive(Clone, PartialEq)]
pub enum OptionStyle {
    Call,
    Put,
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
/// Represents the type of option in a financial context.
/// Options can be categorized into various types based on their characteristics and the conditions under which they can be exercised.
pub enum OptionType {
    /// A European option can only be exercised at the expiry date.
    /// This type of option does not allow the holder to exercise the option before the specified expiration date.
    /// European options are simpler to price and analyze because their payoff is only determined at a single point in time.
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

/// Describes how the average price is calculated for Asian options.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum AsianAveragingType {
    /// Arithmetic averaging calculates the average of the prices in a straightforward manner.
    /// This is the most common type of averaging for Asian options.
    Arithmetic,
    /// Geometric averaging calculates the average using the geometric mean.
    /// This can be less sensitive to extreme values compared to arithmetic averaging.
    Geometric,
}

/// Describes the type of barrier for Barrier options.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum BarrierType {
    /// The option becomes active only if the underlying asset price goes above a certain level.
    UpAndIn,
    /// The option becomes inactive if the underlying asset price goes above a certain level.
    UpAndOut,
    /// The option becomes active only if the underlying asset price goes below a certain level.
    DownAndIn,
    /// The option becomes inactive if the underlying asset price goes below a certain level.
    DownAndOut,
}

/// Describes the type of binary option.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum BinaryType {
    /// The option pays a fixed amount of cash if the underlying asset is above or below a certain level.
    CashOrNothing,
    /// The option pays the value of the underlying asset if it is above or below a certain level.
    AssetOrNothing,
}

/// Describes the type of lookback option.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum LookbackType {
    /// The strike price is fixed at the beginning, and the payoff is based on the maximum or minimum price of the underlying asset during the option's life.
    FixedStrike,
    /// The strike price is determined as the maximum or minimum price of the underlying asset during the option's life, providing the holder with the most advantageous strike price.
    FloatingStrike,
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
                .max(PZERO)
                .max((info.strike - info.spot).max(PZERO))
                .value(),
            OptionType::Cliquet { .. } => standard_payoff(info),
            OptionType::Rainbow { .. }
            | OptionType::Spread { .. }
            | OptionType::Exchange { .. } => standard_payoff(info),
            OptionType::Quanto { exchange_rate } => standard_payoff(info) * exchange_rate,
            OptionType::Power { exponent } => match info.style {
                OptionStyle::Call => (info.spot.value().powf(*exponent) - info.strike).max(ZERO),
                OptionStyle::Put => (info.strike - info.spot.value().powf(*exponent))
                    .max(PZERO)
                    .value(),
            },
        }
    }
}

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
        OptionStyle::Put => (info.strike.value() - average).max(ZERO),
    }
}

fn calculate_barrier_payoff(
    barrier_type: &BarrierType,
    barrier_level: &f64,
    info: &PayoffInfo,
) -> f64 {
    let barrier_condition = match barrier_type {
        BarrierType::UpAndIn | BarrierType::UpAndOut => info.spot.value() >= *barrier_level,
        BarrierType::DownAndIn | BarrierType::DownAndOut => info.spot.value() <= *barrier_level,
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
                info.spot.value()
            } else {
                0.0
            }
        }
    }
}

fn calculate_floating_strike_payoff(info: &PayoffInfo) -> f64 {
    let extremum = match info.style {
        OptionStyle::Call => info.spot_min,
        OptionStyle::Put => info.spot_max,
    };
    match info.style {
        OptionStyle::Call => info.spot.value() - extremum.unwrap_or(ZERO),
        OptionStyle::Put => extremum.unwrap_or(ZERO) - info.spot,
    }
}

#[cfg(test)]
mod tests_payoff {
    use super::*;

    #[test]
    fn test_european_call() {
        let option = OptionType::European;
        let info = PayoffInfo {
            spot: pos!(110.0),
            strike: pos!(100.0),
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
            spot: pos!(90.0),
            strike: pos!(100.0),
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
            spot: pos!(100.0),
            strike: pos!(100.0),
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
            spot: pos!(130.0),
            strike: pos!(100.0),
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
            spot: pos!(110.0),
            strike: pos!(100.0),
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
            spot: pos!(90.0),
            strike: pos!(100.0),
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
            spot: pos!(110.0),
            strike: pos!(100.0),
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
            spot: pos!(10.0),
            strike: pos!(90.0),
            style: OptionStyle::Call,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info), 10.0);
    }
}

#[cfg(test)]
mod tests_expiration_date {
    use super::*;
    use chrono::{Duration, TimeZone};

    #[test]
    fn test_expiration_date_days() {
        let expiration = ExpirationDate::Days(365.0);
        assert_eq!(expiration.get_years(), 1.0);

        let expiration = ExpirationDate::Days(182.5);
        assert_eq!(expiration.get_years(), 0.5);

        let expiration = ExpirationDate::Days(ZERO);
        assert_eq!(expiration.get_years(), ZERO);
    }

    #[test]
    fn test_expiration_date_datetime() {
        // Test for a date exactly one year in the future
        let one_year_future = Utc::now() + Duration::days(365);
        let expiration = ExpirationDate::DateTime(one_year_future);
        assert!((expiration.get_years() - 1.0).abs() < 0.01); // Allow small deviation due to leap years

        // Test for a date 6 months in the future
        let six_months_future = Utc::now() + Duration::days(182);
        let expiration = ExpirationDate::DateTime(six_months_future);
        assert!((expiration.get_years() - 0.5).abs() < 0.01);
    }

    #[test]
    #[should_panic(expected = "DateTime results in negative duration")]
    fn test_expiration_date_datetime_specific() {
        // Test with a specific date
        let specific_date = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let expiration = ExpirationDate::DateTime(specific_date);

        // Calculate expected years (this will change based on when the test is run)
        let now = Utc::now();
        let expected_years = (specific_date - now).num_days() as f64 / 365.0;

        assert!((expiration.get_years() - expected_years).abs() < 0.01);
    }

    #[test]
    fn test_get_date_from_days() {
        let days = 30;
        let expiration = ExpirationDate::Days(days as f64);
        let expected_date = Utc::now() + Duration::days(days);
        let result = expiration.get_date();

        assert!((result - expected_date).num_seconds().abs() <= 1);
    }

    #[test]
    fn test_get_date_from_datetime() {
        let future_date = Utc::now() + Duration::days(60);
        let expiration = ExpirationDate::DateTime(future_date);
        let result = expiration.get_date();

        assert_eq!(result, future_date);
    }

    #[test]
    fn test_get_date_from_past_datetime() {
        let past_date = Utc::now() - Duration::days(30);
        let expiration = ExpirationDate::DateTime(past_date);
        let result = expiration.get_date();

        assert_eq!(result, past_date);
    }

    #[test]
    fn test_get_date_from_zero_days() {
        let expiration = ExpirationDate::Days(0.0);
        let expected_date = Utc::now();
        let result = expiration.get_date();

        assert!((result - expected_date).num_seconds().abs() <= 1);
    }

    #[test]
    fn test_get_date_from_fractional_days() {
        let days = 1.0;
        let expiration = ExpirationDate::Days(days);
        let expected_date =
            Utc::now() + Duration::milliseconds((days * 24.0 * 60.0 * 60.0 * 1000.0) as i64);
        let result = expiration.get_date();

        assert!((result - expected_date).num_seconds().abs() <= 1);
    }

    #[test]
    #[should_panic(expected = "Days cannot be negative")]
    fn test_negative_days_panic() {
        let expiration = ExpirationDate::Days(-10.0);
        expiration.get_years();
    }

    #[test]
    #[should_panic(expected = "DateTime results in negative duration")]
    fn test_negative_datetime_panic() {
        let past_date = Utc::now() - Duration::days(10);
        let expiration = ExpirationDate::DateTime(past_date);
        expiration.get_years();
    }

    #[test]
    fn test_positive_days() {
        let expiration = ExpirationDate::Days(365.0);
        let years = expiration.get_years();
        assert_eq!(years, 1.0);
    }

    #[cfg(test)]
    mod tests_expiration_date_formatting {
        use super::*;
        use chrono::TimeZone;

        #[test]
        fn test_get_date_string_days() {
            let today = Utc::now();
            let expiration = ExpirationDate::Days(30.0);
            let date_str = expiration.get_date_string();
            let expected_date = (today + Duration::days(30)).format("%Y-%m-%d").to_string();
            assert_eq!(date_str, expected_date);
        }

        #[test]
        fn test_get_date_string_datetime() {
            let specific_date = Utc.with_ymd_and_hms(2024, 12, 31, 0, 0, 0).unwrap();
            let expiration = ExpirationDate::DateTime(specific_date);
            assert_eq!(expiration.get_date_string(), "2024-12-31");
        }
    }
}

#[cfg(test)]
mod tests_calculate_floating_strike_payoff {
    use super::*;

    #[test]
    fn test_call_option_with_spot_min() {
        let info = PayoffInfo {
            spot: pos!(100.0),
            strike: pos!(0.0), // Not used in floating strike
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
            spot: pos!(100.0),
            strike: pos!(0.0),
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
            spot: pos!(100.0),
            strike: pos!(0.0),
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
            spot: pos!(100.0),
            strike: pos!(0.0),
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
            spot: pos!(100.0),
            strike: pos!(0.0),
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
            spot: pos!(100.0),
            strike: pos!(0.0),
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
mod tests_positive_f64 {
    use super::*;
    use std::panic;

    #[test]
    fn test_positive_f64_creation() {
        assert!(PositiveF64::new(0.0).is_ok());
        assert!(PositiveF64::new(1.0).is_ok());
        assert!(PositiveF64::new(-1.0).is_err());
    }

    #[test]
    fn test_positive_f64_value() {
        let pos = PositiveF64::new(5.0).unwrap();
        assert_eq!(pos.value(), 5.0);
    }

    #[test]
    fn test_positive_f64_from() {
        let pos = PositiveF64::new(3.0).unwrap();
        let f: f64 = pos.into();
        assert_eq!(f, 3.0);
    }

    #[test]
    fn test_positive_f64_eq() {
        let pos = PositiveF64::new(2.0).unwrap();
        assert_eq!(pos, 2.0);
        assert_ne!(pos, 3.0);
    }

    #[test]
    fn test_positive_f64_display() {
        let pos = PositiveF64::new(4.5).unwrap();
        assert_eq!(format!("{}", pos), "4.5");
    }

    #[test]
    fn test_positive_f64_debug() {
        let pos = PositiveF64::new(4.5).unwrap();
        assert_eq!(format!("{:?}", pos), "4.5");
    }

    #[test]
    fn test_positive_f64_display_decimal_fix() {
        let pos = PositiveF64::new(4.578923789423789).unwrap();
        assert_eq!(format!("{:.2}", pos), "4.58");
        assert_eq!(format!("{:.3}", pos), "4.579");
        assert_eq!(format!("{:.0}", pos), "5");
    }

    #[test]
    fn test_positive_f64_add() {
        let a = PositiveF64::new(2.0).unwrap();
        let b = PositiveF64::new(3.0).unwrap();
        assert_eq!((a + b).value(), 5.0);
    }

    #[test]
    fn test_positive_f64_div() {
        let a = PositiveF64::new(6.0).unwrap();
        let b = PositiveF64::new(2.0).unwrap();
        assert_eq!((a / b).value(), 3.0);
    }

    #[test]
    fn test_positive_f64_div_f64() {
        let a = PositiveF64::new(6.0).unwrap();
        assert_eq!((a / 2.0).value(), 3.0);
    }

    #[test]
    fn test_f64_mul_positive_f64() {
        let a = 2.0;
        let b = PositiveF64::new(3.0).unwrap();
        assert_eq!(a * b, 6.0);
    }

    #[test]
    fn test_positive_f64_mul() {
        let a = PositiveF64::new(2.0).unwrap();
        let b = PositiveF64::new(3.0).unwrap();
        assert_eq!((a * b).value(), 6.0);
    }

    #[test]
    fn test_positive_f64_mul_f64() {
        let a = PositiveF64::new(2.0).unwrap();
        assert_eq!((a * 3.0).value(), 6.0);
    }

    #[test]
    fn test_positive_f64_default() {
        assert_eq!(PositiveF64::default().value(), 0.0);
    }

    #[test]
    fn test_f64_div_positive_f64() {
        let a = 6.0;
        let b = PositiveF64::new(2.0).unwrap();
        assert_eq!(a / b, 3.0);
    }

    #[test]
    fn test_pos_macro() {
        assert_eq!(pos!(5.0).value(), 5.0);
        let result = panic::catch_unwind(|| pos!(-1.0));
        assert!(result.is_err());
    }

    #[test]
    fn test_constants() {
        assert_eq!(PZERO.value(), 0.0);
        assert_eq!(SIZE_ONE.value(), 1.0);
    }
}

#[cfg(test)]
mod tests_positive_f64_extended {
    use super::*;

    #[test]
    fn test_positive_f64_ordering() {
        let a = pos!(1.0);
        let b = pos!(2.0);
        let c = pos!(2.0);

        assert!(a < b);
        assert!(b > a);
        assert!(b >= c);
        assert!(b <= c);
    }

    #[test]
    fn test_positive_f64_add_assign() {
        let mut a = pos!(1.0);
        let b = pos!(2.0);
        a += b;
        assert_eq!(a.value(), 3.0);
    }

    #[test]
    fn test_positive_f64_mul_assign() {
        let mut a = pos!(2.0);
        a *= 3.0;
        assert_eq!(a.value(), 6.0);
    }

    #[test]
    fn test_positive_f64_from_string() {
        assert_eq!(PositiveF64::from_str("1.5").unwrap().value(), 1.5);
        assert!(PositiveF64::from_str("-1.5").is_err());
        assert!(PositiveF64::from_str("invalid").is_err());
    }

    #[test]
    fn test_positive_f64_max_min() {
        let a = pos!(1.0);
        let b = pos!(2.0);
        assert_eq!(a.max(b).value(), 2.0);
        assert_eq!(a.min(b).value(), 1.0);
    }

    #[test]
    fn test_positive_f64_floor() {
        let a = pos!(1.7);
        assert_eq!(a.floor().value(), 1.0);
    }

    #[test]
    #[should_panic(expected = "Cannot negate a PositiveF64 value!")]
    fn test_positive_f64_neg() {
        let a = pos!(1.0);
        let _ = -a;
    }
}

#[cfg(test)]
mod tests_option_type {
    use super::*;

    #[test]
    fn test_asian_geometric_call() {
        let option = OptionType::Asian {
            averaging_type: AsianAveragingType::Geometric,
        };
        let info = PayoffInfo {
            spot: pos!(100.0),
            strike: pos!(100.0),
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
            spot: pos!(100.0),
            strike: pos!(95.0),
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
            spot: pos!(95.0),
            strike: pos!(100.0),
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
            spot: pos!(90.0),
            strike: pos!(100.0),
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
            spot: pos!(110.0),
            strike: pos!(100.0),
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
            spot: pos!(110.0),
            strike: pos!(100.0),
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
            spot: pos!(8.0),
            strike: pos!(100.0),
            style: OptionStyle::Put,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info), 36.0);
    }
}

#[cfg(test)]
mod tests_positive_f64_sum {
    use super::*;

    #[test]
    fn test_sum_owned_values() {
        let values = vec![pos!(1.0), pos!(2.0), pos!(3.0)];
        let sum: PositiveF64 = values.into_iter().sum();
        assert_eq!(sum.value(), 6.0);
    }

    #[test]
    fn test_sum_referenced_values() {
        let values = [pos!(1.0), pos!(2.0), pos!(3.0)];
        let sum: PositiveF64 = values.iter().sum();
        assert_eq!(sum.value(), 6.0);
    }

    #[test]
    fn test_sum_empty_iterator() {
        let values: Vec<PositiveF64> = vec![];
        let sum: PositiveF64 = values.into_iter().sum();
        assert_eq!(sum.value(), 0.0);
    }
}

#[cfg(test)]
mod tests_vec_collection {
    use super::*;
    use crate::pos;

    #[test]
    fn test_collect_empty_iterator() {
        let empty_vec: Vec<PositiveF64> = Vec::new();
        let collected: Vec<PositiveF64> = empty_vec.into_iter().collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn test_collect_single_value() {
        let values = vec![pos!(1.0)];
        let collected: Vec<PositiveF64> = values.into_iter().collect();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0], pos!(1.0));
    }

    #[test]
    fn test_collect_multiple_values() {
        let values = vec![pos!(1.0), pos!(2.0), pos!(3.0)];
        let collected: Vec<PositiveF64> = values.into_iter().collect();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0], pos!(1.0));
        assert_eq!(collected[1], pos!(2.0));
        assert_eq!(collected[2], pos!(3.0));
    }

    #[test]
    fn test_collect_from_filter() {
        let values = vec![pos!(1.0), pos!(2.0), pos!(3.0), pos!(4.0)];
        let collected: Vec<PositiveF64> = values.into_iter().filter(|x| x.value() > 2.0).collect();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0], pos!(3.0));
        assert_eq!(collected[1], pos!(4.0));
    }

    #[test]
    fn test_collect_from_map() {
        let values = vec![pos!(1.0), pos!(2.0), pos!(3.0)];
        let collected: Vec<PositiveF64> =
            values.into_iter().map(|x| pos!(x.value() * 2.0)).collect();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0], pos!(2.0));
        assert_eq!(collected[1], pos!(4.0));
        assert_eq!(collected[2], pos!(6.0));
    }

    #[test]
    fn test_collect_from_chain() {
        let values1 = vec![pos!(1.0), pos!(2.0)];
        let values2 = vec![pos!(3.0), pos!(4.0)];
        let collected: Vec<PositiveF64> = values1.into_iter().chain(values2).collect();
        assert_eq!(collected.len(), 4);
        assert_eq!(collected[0], pos!(1.0));
        assert_eq!(collected[1], pos!(2.0));
        assert_eq!(collected[2], pos!(3.0));
        assert_eq!(collected[3], pos!(4.0));
    }
}
