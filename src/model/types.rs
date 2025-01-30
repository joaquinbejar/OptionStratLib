use crate::constants::{DAYS_IN_A_YEAR, ZERO};
use crate::pricing::payoff::{standard_payoff, Payoff, PayoffInfo};
use crate::{pos, Positive};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use rust_decimal::Decimal;
use std::error::Error;


#[derive(Clone, PartialEq)]
pub enum ExpirationDate {
    Days(Positive),
    DateTime(DateTime<Utc>),
}

impl ExpirationDate {
    /// Calculates the time to expiration in years.
    ///
    /// Returns a `Result<Positive, Box<dyn Error>>`.
    ///
    /// # Arguments
    ///
    /// * `&self` - The `ExpirationDate` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the `DateTime` variant results in a negative duration
    /// indicating the expiration date is in the past.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::{Duration, Utc};
    /// use rust_decimal_macros::dec;
    /// use optionstratlib::model::types::{ExpirationDate, Positive};
    /// use optionstratlib::pos;
    ///
    /// let days = pos!(365.0);
    /// let expiration_date_days = ExpirationDate::Days(days);
    /// let years = expiration_date_days.get_years().unwrap();
    /// assert_eq!(years, pos!(1.0));
    ///
    /// let datetime = Utc::now() + Duration::days(365);
    /// let expiration_date_datetime = ExpirationDate::DateTime(datetime);
    /// let years = expiration_date_datetime.get_years().unwrap();
    /// assert_eq!(years, pos!(1.0));
    /// ```
    pub fn get_years(&self) -> Result<Positive, Box<dyn Error>> {
        match self {
            ExpirationDate::Days(days) => Ok(*days / DAYS_IN_A_YEAR),
            ExpirationDate::DateTime(datetime) => {
                let now = Utc::now();
                let duration = datetime.signed_duration_since(now);
                let num_days = duration.num_seconds() as f64 / (24.0 * 60.0 * 60.0);
                if num_days <= 0.0 {
                    return Ok(Positive::ZERO);
                }
                Ok(pos!(num_days) / DAYS_IN_A_YEAR)
            }
        }
    }

    /// Returns the expiration date as a `DateTime<Utc>`.
    ///
    /// For the `Days` variant, it calculates the date by adding the specified number of days to the current date and time.
    /// For the `DateTime` variant, it returns the stored `DateTime<Utc>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::{Duration, Utc};
    /// use rust_decimal_macros::dec;
    /// use optionstratlib::model::types::{ExpirationDate, Positive};
    /// use optionstratlib::pos;
    ///
    /// let days = pos!(30.0);
    /// let expiration_date_days = ExpirationDate::Days(days);
    /// let future_date = Utc::now() + Duration::days(30);
    /// let calculated_date = expiration_date_days.get_date().unwrap();
    /// // Check if dates are within a small tolerance (due to potential time differences during test)
    /// assert_eq!(calculated_date.date(), future_date.date());
    ///
    ///
    /// let datetime = Utc::now() + Duration::days(365);
    /// let expiration_date_datetime = ExpirationDate::DateTime(datetime);
    /// let stored_date = expiration_date_datetime.get_date().unwrap();
    /// assert_eq!(stored_date, datetime);
    ///
    /// ```
    pub fn get_date(&self) -> Result<DateTime<Utc>, Box<dyn Error>> {
        // Get today's date at 7:30 PM UTC
        let today = Utc::now().date_naive();
        let fixed_time = today.and_hms_opt(18, 30, 0).ok_or("Invalid time")?;
        let fixed_datetime = DateTime::<Utc>::from_naive_utc_and_offset(fixed_time, Utc);

        match self {
            ExpirationDate::Days(days) => Ok(fixed_datetime + Duration::days((*days).to_i64())),
            ExpirationDate::DateTime(datetime) => Ok(*datetime),
        }
    }

    /// Returns the expiration date as a formatted string in `YYYY-MM-DD` format.
    ///
    /// This method calls `get_date()` to retrieve the `DateTime<Utc>` and then formats it into the specified string format.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::{Duration, Utc};
    /// use rust_decimal_macros::dec;
    /// use optionstratlib::model::types::{ExpirationDate, Positive};
    /// use optionstratlib::pos;
    ///
    /// let days = pos!(30.0);
    /// let expiration_date = ExpirationDate::Days(days);
    /// let date_string = expiration_date.get_date_string().unwrap();
    /// assert!(date_string.len() == 10); // YYYY-MM-DD format
    /// ```
    pub fn get_date_string(&self) -> Result<String, Box<dyn Error>> {
        let date = self.get_date()?;
        Ok(date.format("%Y-%m-%d").to_string())
    }

    /// Creates an `ExpirationDate` from a string.
    ///
    /// This function attempts to parse the input string `s` into an `ExpirationDate`. It supports various formats, including:
    ///
    /// 1. **Positive number of days:** Parses the string as a `Positive` number, representing days from now.
    /// 2. **RFC3339 DateTime:** Parses the string as an RFC3339 compliant date and time string.
    /// 3. **Numeric Date (YYYYMMDD):** Parses an 8-digit numeric string as year, month, and day. Sets the time to 23:59:59.
    /// 4. **Common Date formats:** Parses various common date formats (e.g., DD-MM-YYYY, DD MMM YYYY, etc.). Sets the time to 23:59:59.
    ///
    /// If none of the above formats can be parsed successfully, an error is returned.
    ///
    /// # Arguments
    ///
    /// * `s` - The input string to parse.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::{DateTime, Utc};
    /// use rust_decimal_macros::dec;
    /// use optionstratlib::model::types::{ExpirationDate, Positive};
    /// use optionstratlib::pos;
    ///
    /// let expiration_date_days = ExpirationDate::from_string("365").unwrap();
    /// assert_eq!(expiration_date_days, ExpirationDate::Days(pos!(365.0)));
    ///
    /// let rfc3339_string = "2025-01-01T12:00:00Z";
    /// let expiration_date_rfc3339 = ExpirationDate::from_string(rfc3339_string).unwrap();
    /// let datetime = DateTime::parse_from_rfc3339(rfc3339_string).unwrap();
    /// assert_eq!(expiration_date_rfc3339, ExpirationDate::DateTime(DateTime::from(datetime)));
    ///
    /// let numeric_date_string = "20250101";
    /// let expiration_date_numeric = ExpirationDate::from_string(numeric_date_string).unwrap();
    /// if let ExpirationDate::DateTime(dt) = expiration_date_numeric {
    ///     assert_eq!(dt.format("%Y%m%d").to_string(), numeric_date_string);
    /// } else {
    ///     panic!("Expected ExpirationDate::DateTime");
    /// }
    ///
    ///
    /// let common_date_string = "01-01-2025";
    /// let expiration_date_common = ExpirationDate::from_string(common_date_string).unwrap();
    /// if let ExpirationDate::DateTime(dt) = expiration_date_common {
    ///     assert_eq!(dt.format("%d-%m-%Y").to_string(), common_date_string);
    /// } else {
    ///     panic!("Expected ExpirationDate::DateTime");
    /// }
    /// ```
    pub fn from_string(s: &str) -> Result<Self, Box<dyn Error>> {
        // First try parsing as Positive (days)
        if let Ok(days) = s.parse::<Positive>() {
            return Ok(ExpirationDate::Days(days));
        }

        // Try parsing as RFC3339
        if let Ok(datetime) = DateTime::parse_from_rfc3339(s) {
            return Ok(ExpirationDate::DateTime(DateTime::from(datetime)));
        }

        // Try numeric date formats first
        if s.len() == 8 && s.chars().all(|c| c.is_ascii_digit()) {
            // Format: YYYYMMDD
            let year = s[0..4].parse::<i32>()?;
            let month = s[4..6].parse::<u32>()?;
            let day = s[6..8].parse::<u32>()?;

            if let Some(naive_datetime) = NaiveDate::from_ymd_opt(year, month, day)
                .and_then(|date| date.and_hms_opt(23, 59, 59))
            {
                let datetime = DateTime::<Utc>::from_naive_utc_and_offset(naive_datetime, Utc);
                return Ok(ExpirationDate::DateTime(datetime));
            }
        }

        // Try parsing common date formats
        let formats = [
            "%d-%m-%Y",     // "01-01-2025"
            "%d %b %Y",     // "30 jan 2025"
            "%d-%b-%Y",     // "30-jan-2025"
            "%d %B %Y",     // "30 january 2025"
            "%d-%B-%Y",     // "30-january-2025"
        ];

        for format in formats {
            if let Ok(naive_date) = NaiveDate::parse_from_str(s.to_lowercase().as_str(), format) {
                // Convert NaiveDate to DateTime<Utc> by setting time to end of day
                let naive_datetime = naive_date.and_hms_opt(18, 30, 00)
                    .ok_or_else(|| format!("Invalid time conversion for date: {s}"))?;

                let datetime = DateTime::<Utc>::from_naive_utc_and_offset(naive_datetime, Utc);
                return Ok(ExpirationDate::DateTime(datetime));
            }
        }

        // If none of the above worked, return error
        Err(format!("Failed to parse ExpirationDate from string: {s}").into())
    }
}

impl Default for ExpirationDate {
    fn default() -> Self {
        ExpirationDate::Days(pos!(365.0))
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
    use super::*;
    use crate::pos;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    use crate::constants::DAYS_IN_A_YEAR;
    use chrono::{Days, Duration};

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_expiration_date_days() {
        let expiration = ExpirationDate::Days(DAYS_IN_A_YEAR);
        assert_eq!(expiration.get_years().unwrap(), 1.0);

        let expiration = ExpirationDate::Days(pos!(182.5));
        assert_eq!(expiration.get_years().unwrap(), 0.5);

        let expiration = ExpirationDate::Days(Positive::ZERO);
        assert_eq!(expiration.get_years().unwrap(), ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_expiration_date_datetime() {
        // Test for a date exactly one year in the future
        let one_year_future = Utc::now() + Duration::days(365);
        let expiration = ExpirationDate::DateTime(one_year_future);
        assert!((expiration.get_years().unwrap().to_f64() - 1.0).abs() < 0.01); // Allow small deviation due to leap years

        // Test for a date 6 months in the future
        let six_months_future = Utc::now() + Duration::days(182);
        let expiration = ExpirationDate::DateTime(six_months_future);
        assert!((expiration.get_years().unwrap().to_f64() - 0.5).abs() < 0.01);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_expiration_date_datetime_specific() {
        // Test with a specific date
        // let specific_date = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let specific_date = Utc::now() + Duration::days(1);
        let expiration = ExpirationDate::DateTime(specific_date);
        assert!(expiration.get_years().unwrap() > Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_date_from_days() {
        let days = pos!(30.0);
        let expiration = ExpirationDate::Days(days);
        let today = Utc::now().date_naive();
        let expected_date = today.checked_add_days(Days::new(days.to_i64() as u64))
            .unwrap()
            .and_hms_opt(18, 30, 0)
            .unwrap();
        let expected_datetime = DateTime::<Utc>::from_naive_utc_and_offset(expected_date, Utc);
        let result = expiration.get_date().unwrap();

        // Calculate the difference in seconds
        let difference = (result - expected_datetime).num_seconds();
        
        assert!(difference.abs() <= 1);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_date_from_datetime() {
        let future_date = Utc::now() + Duration::days(60);
        let expiration = ExpirationDate::DateTime(future_date);
        let result = expiration.get_date().unwrap();

        assert_eq!(result, future_date);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_date_from_past_datetime() {
        let past_date = Utc::now() - Duration::days(30);
        let expiration = ExpirationDate::DateTime(past_date);
        let result = expiration.get_date().unwrap();
        assert_eq!(result, past_date);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_date_from_zero_days() {
        let expiration = ExpirationDate::Days(Positive::ZERO);

        // Create today's date at 18:30 UTC
        let today = Utc::now().date_naive();
        let expected_date = today.and_hms_opt(18, 30, 0)
            .unwrap();
        let expected_datetime = DateTime::<Utc>::from_naive_utc_and_offset(expected_date, Utc);
        let result = expiration.get_date().unwrap();

        // Calculate the difference in seconds
        let difference = (result - expected_datetime).num_seconds();

        assert!(difference.abs() <= 1);
    }
    
    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_positive_days() {
        let expiration = ExpirationDate::Days(DAYS_IN_A_YEAR);
        let years = expiration.get_years().unwrap();
        assert_eq!(years, 1.0);
    }

    #[cfg(test)]
    mod tests_expiration_date_formatting {
        use super::*;
        use chrono::TimeZone;

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_get_date_string_days() {
            let today = Utc::now();
            let expiration = ExpirationDate::Days(pos!(30.0));
            let date_str = expiration.get_date_string().unwrap();
            let expected_date = (today + Duration::days(30)).format("%Y-%m-%d").to_string();
            assert_eq!(date_str, expected_date);
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_get_date_string_datetime() {
            let specific_date = Utc.with_ymd_and_hms(2024, 12, 31, 0, 0, 0).unwrap();
            let expiration = ExpirationDate::DateTime(specific_date);
            assert_eq!(expiration.get_date_string().unwrap(), "2024-12-31");
        }
    }
}

#[cfg(test)]
mod tests_calculate_floating_strike_payoff {
    use super::*;
    use crate::pos;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_call_option_with_spot_min() {
        let info = PayoffInfo {
            spot: pos!(100.0),
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_call_option_without_spot_min() {
        let info = PayoffInfo {
            spot: pos!(100.0),
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_put_option_with_spot_max() {
        let info = PayoffInfo {
            spot: pos!(100.0),
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_put_option_without_spot_max() {
        let info = PayoffInfo {
            spot: pos!(100.0),
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_call_option_spot_equals_min() {
        let info = PayoffInfo {
            spot: pos!(100.0),
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_put_option_spot_equals_max() {
        let info = PayoffInfo {
            spot: pos!(100.0),
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
    use crate::pos;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
mod tests_vec_collection {
    use crate::model::positive::Positive;
    use crate::pos;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_collect_empty_iterator() {
        let empty_vec: Vec<Positive> = Vec::new();
        let collected: Vec<Positive> = empty_vec.into_iter().collect();
        assert!(collected.is_empty());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_collect_single_value() {
        let values = vec![pos!(1.0)];
        let collected: Vec<Positive> = values.into_iter().collect();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0], pos!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_collect_multiple_values() {
        let values = vec![pos!(1.0), pos!(2.0), pos!(3.0)];
        let collected: Vec<Positive> = values.into_iter().collect();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0], pos!(1.0));
        assert_eq!(collected[1], pos!(2.0));
        assert_eq!(collected[2], pos!(3.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_collect_from_filter() {
        let values = vec![pos!(1.0), pos!(2.0), pos!(3.0), pos!(4.0)];
        let collected: Vec<Positive> = values.into_iter().filter(|x| x.to_f64() > 2.0).collect();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0], pos!(3.0));
        assert_eq!(collected[1], pos!(4.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_collect_from_map() {
        let values = vec![pos!(1.0), pos!(2.0), pos!(3.0)];
        let collected: Vec<Positive> = values.into_iter().map(|x| pos!(x.to_f64() * 2.0)).collect();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0], pos!(2.0));
        assert_eq!(collected[1], pos!(4.0));
        assert_eq!(collected[2], pos!(6.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_collect_from_chain() {
        let values1 = vec![pos!(1.0), pos!(2.0)];
        let values2 = vec![pos!(3.0), pos!(4.0)];
        let collected: Vec<Positive> = values1.into_iter().chain(values2).collect();
        assert_eq!(collected.len(), 4);
        assert_eq!(collected[0], pos!(1.0));
        assert_eq!(collected[1], pos!(2.0));
        assert_eq!(collected[2], pos!(3.0));
        assert_eq!(collected[3], pos!(4.0));
    }
}

#[cfg(test)]
mod test_expiration_date {
    use chrono::{Local, Timelike, Utc};
    use crate::model::ExpirationDate;
    use crate::{assert_pos_relative_eq, pos};
    use crate::utils::time::get_today_formatted;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_from_string_valid_days() {
        let result = ExpirationDate::from_string("30.0");
        assert!(result.is_ok());
        match result.unwrap() {
            ExpirationDate::Days(days) => assert_eq!(days, 30.0),
            _ => panic!("Expected Days variant"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_from_string_valid_datetime() {
        let result = ExpirationDate::from_string("2024-12-31T00:00:00Z");
        assert!(result.is_ok());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_from_string_format_one() {
        let result = ExpirationDate::from_string("30 jan 2025");
        assert!(result.is_ok());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_from_string_format_two() {
        let result = ExpirationDate::from_string("30-jan-2025");
        assert!(result.is_ok());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_from_string_format_three() {
        let result = ExpirationDate::from_string("20250101");
        assert!(result.is_ok());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn four() {
        let result = ExpirationDate::from_string("30-01-2025");
        assert!(result.is_ok());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_from_string_invalid_format() {
        let result = ExpirationDate::from_string("invalid date");
        assert!(result.is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_from_string_format_today() {
        let today = get_today_formatted();
        let result = ExpirationDate::from_string(&*today);
        assert!(result.is_ok());
        let expiration = result.unwrap();
        assert!(expiration.get_date_string().is_ok());

        let today = Local::now().date_naive();
        assert_eq!(expiration.get_date_string().unwrap(), today.format("%Y-%m-%d").to_string());

        // Get current UTC time
        let current_utc_time = Utc::now().time();
        let years = expiration.get_years().unwrap();

        // Check years based on current UTC time
        if current_utc_time < Utc::now().date_naive().and_hms_opt(18, 30, 0).unwrap().time() {
            // Before 18:30 UTC
            assert!(years > 0.0, "Years should be greater than 0 before 18:30 UTC");
        } else {
            // After 18:30 UTC
            assert_eq!(years.to_f64(), 0.0, "Years should be 0 after 18:30 UTC");
        }

        assert!(expiration.get_date_string().is_ok());

        // Get the date
        let date = expiration.get_date().unwrap();

        // Additional checks for the date components
        assert_eq!(date.hour(), 18, "Hour should be 18");
        assert_eq!(date.minute(), 30);
        assert_eq!(date.second(), 0);

        assert!(expiration.get_date_string().is_ok());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_from_expiration_date_zero() {
        let zero_expiration_date = ExpirationDate::Days(pos!(0.0));

        let today = Local::now().date_naive();
        assert_eq!(zero_expiration_date.get_date_string().unwrap(), today.format("%Y-%m-%d").to_string());
        let years = zero_expiration_date.get_years().unwrap();
        assert_pos_relative_eq!(years, pos!(0.0), pos!(1e-3));

        assert!(zero_expiration_date.get_date_string().is_ok());

        // Get the date
        let date = zero_expiration_date.get_date().unwrap();

        // Additional checks for the date components
        assert_eq!(date.hour(), 18, "Hour should be 18");
        assert_eq!(date.minute(), 30);
        assert_eq!(date.second(), 0);

        assert!(zero_expiration_date.get_date_string().is_ok());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_from_expiration_date_almost_zero() {
        let zero_expiration_date = ExpirationDate::Days(pos!(0.5));
        let today = Local::now().date_naive();
        assert_eq!(zero_expiration_date.get_date_string().unwrap(), today.format("%Y-%m-%d").to_string());
        let years = zero_expiration_date.get_years().unwrap();
        assert_pos_relative_eq!(years, pos!(0.001369), pos!(1e-3));
        
        assert!(zero_expiration_date.get_date_string().is_ok());

        // Get the date
        let date = zero_expiration_date.get_date().unwrap();

        // Additional checks for the date components
        assert_eq!(date.hour(), 18, "Hour should be 18");
        assert_eq!(date.minute(), 30);
        assert_eq!(date.second(), 0);

        assert!(zero_expiration_date.get_date_string().is_ok());
    }
}

#[cfg(test)]
mod test_asian_options {
    use crate::model::types::AsianAveragingType;
    use crate::model::{OptionStyle, OptionType, Side};
    use crate::pos;
    use crate::pricing::{Payoff, PayoffInfo};

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_asian_arithmetic_put() {
        let option = OptionType::Asian {
            averaging_type: AsianAveragingType::Arithmetic,
        };
        let info = PayoffInfo {
            spot: pos!(90.0),
            strike: pos!(100.0),
            style: OptionStyle::Put,
            side: Side::Long,
            spot_prices: Some(vec![85.0, 90.0, 95.0]),
            ..Default::default()
        };
        assert_eq!(option.payoff(&info), 10.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_asian_no_spot_prices() {
        let option = OptionType::Asian {
            averaging_type: AsianAveragingType::Arithmetic,
        };
        let info = PayoffInfo {
            spot: pos!(100.0),
            strike: pos!(100.0),
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
    use crate::pos;
    use crate::pricing::{Payoff, PayoffInfo};

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_barrier_down_and_in_put() {
        let option = OptionType::Barrier {
            barrier_type: BarrierType::DownAndIn,
            barrier_level: 90.0,
        };
        let info = PayoffInfo {
            spot: pos!(100.0),
            strike: pos!(100.0),
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: None,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info), 0.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_barrier_up_and_out_call() {
        let option = OptionType::Barrier {
            barrier_type: BarrierType::UpAndOut,
            barrier_level: 110.0,
        };
        let info = PayoffInfo {
            spot: pos!(120.0),
            strike: pos!(100.0),
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
    use crate::pos;
    use crate::pricing::{Payoff, PayoffInfo};

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_cliquet_option_with_resets() {
        let option = OptionType::Cliquet {
            reset_dates: vec![30.0, 60.0, 90.0],
        };
        let info = PayoffInfo {
            spot: pos!(120.0),
            strike: pos!(100.0),
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
    use crate::model::{OptionStyle, OptionType, Side};
    use crate::pos;
    use crate::pricing::{Payoff, PayoffInfo};

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_rainbow_option_multiple_assets() {
        let option = OptionType::Rainbow { num_assets: 3 };
        let info = PayoffInfo {
            spot: pos!(120.0),
            strike: pos!(100.0),
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
    use crate::model::{OptionStyle, OptionType, Side};
    use crate::pos;
    use crate::pricing::{Payoff, PayoffInfo};

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_exchange_option_positive_diff() {
        let option = OptionType::Exchange { second_asset: 90.0 };
        let info = PayoffInfo {
            spot: pos!(120.0),
            strike: pos!(100.0),
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: None,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info), 20.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_exchange_option_negative_diff() {
        let option = OptionType::Exchange {
            second_asset: 110.0,
        };
        let info = PayoffInfo {
            spot: pos!(110.0),
            strike: pos!(100.0),
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: None,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info), 10.0);
    }
}
