use crate::pricing::payoff::{standard_payoff, Payoff, PayoffInfo};
use chrono::{DateTime, Utc};

#[allow(dead_code)]
pub enum ExpirationDate {
    Days(f64),
    DateTime(DateTime<Utc>),
}

impl ExpirationDate {
    pub(crate) fn get_years(&self) -> f64 {
        match self {
            ExpirationDate::Days(days) => days / 365.0,
            ExpirationDate::DateTime(datetime) => {
                let now = Utc::now();
                let duration = datetime.signed_duration_since(now);
                duration.num_days() as f64 / 365.0
            }
        }
    }
}

impl Default for ExpirationDate {
    fn default() -> Self {
        ExpirationDate::Days(365.0)
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum Side {
    Long,
    Short,
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum OptionStyle {
    Call,
    Put,
}

#[allow(dead_code)]
#[derive(Clone)]
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
#[derive(Clone)]
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
#[derive(Clone)]
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
#[derive(Clone)]
pub enum BinaryType {
    /// The option pays a fixed amount of cash if the underlying asset is above or below a certain level.
    CashOrNothing,
    /// The option pays the value of the underlying asset if it is above or below a certain level.
    AssetOrNothing,
}

/// Describes the type of lookback option.
#[allow(dead_code)]
#[derive(Clone)]
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
            OptionType::Chooser { .. } => {
                ((info.spot - info.strike).max(0.0)).max((info.strike - info.spot).max(0.0))
            }
            OptionType::Cliquet { .. } => standard_payoff(info),
            OptionType::Rainbow { .. }
            | OptionType::Spread { .. }
            | OptionType::Exchange { .. } => standard_payoff(info),
            OptionType::Quanto { exchange_rate } => standard_payoff(info) * exchange_rate,
            OptionType::Power { exponent } => match info.style {
                OptionStyle::Call => (info.spot.powf(*exponent) - info.strike).max(0.0),
                OptionStyle::Put => (info.strike - info.spot.powf(*exponent)).max(0.0),
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
        _ => return 0.0,
    };
    match info.style {
        OptionStyle::Call => (average - info.strike).max(0.0),
        OptionStyle::Put => (info.strike - average).max(0.0),
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
                info.spot
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
        OptionStyle::Call => info.spot - extremum.unwrap_or(0.0),
        OptionStyle::Put => extremum.unwrap_or(0.0) - info.spot,
    }
}

#[cfg(test)]
mod tests_payoff {
    use super::*;

    #[test]
    fn test_european_call() {
        let option = OptionType::European;
        let info = PayoffInfo {
            spot: 110.0,
            strike: 100.0,
            style: OptionStyle::Call,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info), 10.0);
    }

    #[test]
    fn test_european_put() {
        let option = OptionType::European;
        let info = PayoffInfo {
            spot: 90.0,
            strike: 100.0,
            style: OptionStyle::Put,
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
            spot: 100.0,
            strike: 100.0,
            style: OptionStyle::Call,
            spot_prices: Some(vec![90.0, 100.0, 110.0]),
            ..Default::default()
        };
        assert_eq!(option.payoff(&info), 0.0);
    }

    #[test]
    fn test_barrier_up_and_in_call() {
        let option = OptionType::Barrier {
            barrier_type: BarrierType::UpAndIn,
            barrier_level: 120.0,
        };
        let info = PayoffInfo {
            spot: 130.0,
            strike: 100.0,
            style: OptionStyle::Call,
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
            spot: 110.0,
            strike: 100.0,
            style: OptionStyle::Call,
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
            spot: 90.0,
            strike: 100.0,
            style: OptionStyle::Put,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info), 10.0);
    }

    #[test]
    fn test_quanto_call() {
        let option = OptionType::Quanto { exchange_rate: 1.5 };
        let info = PayoffInfo {
            spot: 110.0,
            strike: 100.0,
            style: OptionStyle::Call,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info), 15.0);
    }

    #[test]
    fn test_power_call() {
        let option = OptionType::Power { exponent: 2.0 };
        let info = PayoffInfo {
            spot: 10.0,
            strike: 90.0,
            style: OptionStyle::Call,
            ..Default::default()
        };
        assert_eq!(option.payoff(&info), 10.0);
    }
}
