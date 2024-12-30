use crate::pricing::payoff::{standard_payoff, Payoff, PayoffInfo};
use approx::{AbsDiffEq, RelativeEq};
use chrono::{DateTime, Duration, Utc};
use num_traits::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub};
use std::str::FromStr;
use crate::constants::ZERO;
use crate::Positive;

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

    pub fn from_string(s: &String) -> Result<Self, String> {
        if let Ok(days) = s.parse::<f64>() {
            Ok(ExpirationDate::Days(days))
        } else if let Ok(datetime) = DateTime::parse_from_rfc3339(s) {
            Ok(ExpirationDate::DateTime(DateTime::from(datetime)))
        } else {
            Err(format!("Failed to parse ExpirationDate from string: {}", s))
        }
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
                .max(Positive::ZERO)
                .max((info.strike - info.spot).max(Positive::ZERO))
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
        BarrierType::UpAndIn | BarrierType::UpAndOut => info.spot.value() >= *barrier_level.to_f64(),
        BarrierType::DownAndIn | BarrierType::DownAndOut => info.spot.value() <= *barrier_level.to_f64(),
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
                info.spot.to_f64()
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
        OptionStyle::Call => info.spot.to_f64() - extremum.unwrap_or(ZERO),
        OptionStyle::Put => extremum.unwrap_or(ZERO) - info.spot.to_f64(),
    }
}

#[cfg(test)]
mod tests_payoff {
    use crate::f2p;
    use super::*;

    #[test]
    fn test_european_call() {
        let option = OptionType::European;
        let info = PayoffInfo {
            spot: f2p!(110.0),
            strike: f2p!(100.0),
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
            spot: f2p!(90.0),
            strike: f2p!(100.0),
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
            spot: f2p!(100.0),
            strike: f2p!(100.0),
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
            spot: f2p!(130.0),
            strike: f2p!(100.0),
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
            spot: f2p!(110.0),
            strike: f2p!(100.0),
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
            spot: f2p!(90.0),
            strike: f2p!(100.0),
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
            spot: f2p!(110.0),
            strike: f2p!(100.0),
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
            spot: f2p!(10.0),
            strike: f2p!(90.0),
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
    use crate::f2p;
    use super::*;

    #[test]
    fn test_call_option_with_spot_min() {
        let info = PayoffInfo {
            spot: f2p!(100.0),
            strike: f2p!(0.0), // Not used in floating strike
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
            spot: f2p!(100.0),
            strike: f2p!(0.0),
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
            spot: f2p!(100.0),
            strike: f2p!(0.0),
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
            spot: f2p!(100.0),
            strike: f2p!(0.0),
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
            spot: f2p!(100.0),
            strike: f2p!(0.0),
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
            spot: f2p!(100.0),
            strike: f2p!(0.0),
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
    use crate::f2p;
    use super::*;

    #[test]
    fn test_asian_geometric_call() {
        let option = OptionType::Asian {
            averaging_type: AsianAveragingType::Geometric,
        };
        let info = PayoffInfo {
            spot: f2p!(100.0),
            strike: f2p!(100.0),
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
            spot: f2p!(100.0),
            strike: f2p!(95.0),
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
            spot: f2p!(95.0),
            strike: f2p!(100.0),
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
            spot: f2p!(90.0),
            strike: f2p!(100.0),
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
            spot: f2p!(110.0),
            strike: f2p!(100.0),
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
            spot: f2p!(110.0),
            strike: f2p!(100.0),
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
            spot: f2p!(8.0),
            strike: f2p!(100.0),
            style: OptionStyle::Put,
            side: Side::Long,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info), 36.0);
    }
}



#[cfg(test)]
mod tests_vec_collection {
    use crate::model::positive::Positive;
    use super::*;
    use crate::f2p;

    #[test]
    fn test_collect_empty_iterator() {
        let empty_vec: Vec<Positive> = Vec::new();
        let collected: Vec<Positive> = empty_vec.into_iter().collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn test_collect_single_value() {
        let values = vec![f2p!(1.0)];
        let collected: Vec<Positive> = values.into_iter().collect();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0], f2p!(1.0));
    }

    #[test]
    fn test_collect_multiple_values() {
        let values = vec![f2p!(1.0), f2p!(2.0), f2p!(3.0)];
        let collected: Vec<Positive> = values.into_iter().collect();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0], f2p!(1.0));
        assert_eq!(collected[1], f2p!(2.0));
        assert_eq!(collected[2], f2p!(3.0));
    }

    #[test]
    fn test_collect_from_filter() {
        let values = vec![f2p!(1.0), f2p!(2.0), f2p!(3.0), f2p!(4.0)];
        let collected: Vec<Positive> = values.into_iter().filter(|x| x.to_f64() > 2.0).collect();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0], f2p!(3.0));
        assert_eq!(collected[1], f2p!(4.0));
    }

    #[test]
    fn test_collect_from_map() {
        let values = vec![f2p!(1.0), f2p!(2.0), f2p!(3.0)];
        let collected: Vec<Positive> =
            values.into_iter().map(|x| f2p!(x.value() * 2.0)).collect();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0], f2p!(2.0));
        assert_eq!(collected[1], f2p!(4.0));
        assert_eq!(collected[2], f2p!(6.0));
    }

    #[test]
    fn test_collect_from_chain() {
        let values1 = vec![f2p!(1.0), f2p!(2.0)];
        let values2 = vec![f2p!(3.0), f2p!(4.0)];
        let collected: Vec<Positive> = values1.into_iter().chain(values2).collect();
        assert_eq!(collected.len(), 4);
        assert_eq!(collected[0], f2p!(1.0));
        assert_eq!(collected[1], f2p!(2.0));
        assert_eq!(collected[2], f2p!(3.0));
        assert_eq!(collected[3], f2p!(4.0));
    }
}
