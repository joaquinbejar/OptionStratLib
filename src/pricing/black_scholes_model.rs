/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 11/8/24
******************************************************************************/

use crate::greeks::utils::{big_n, calculate_d_values};
use crate::model::option::Options;
use crate::model::types::OptionStyle;

/// Computes the price of an option using the Black-Scholes model.
///
/// # Arguments
///
/// * `option`: An `Options` struct containing all the relevant parameters for the option (e.g., strike price, underlying asset price, etc.).
/// * `time_to_expiry`: An optional `f64` representing the time until the option's expiration in years.
///
/// # Returns
///
/// A `f64` representing the calculated price of the option.
///
/// # Description
///
/// This function leverages the Black-Scholes model to determine the price of
/// either a call option or a put option. It first calculates the `d1` and `d2`
/// parameters required for the model and then matches the `option_style` to
/// use the appropriate pricing formula for call or put options.
///
/// The function calls helper functions:
/// - `calculate_d1_d2_and_time()`: Computes the necessary `d1`, `d2`, and expiry time.
/// - `calculate_call_option_price()`: Computes the price of a call option.
/// - `calculate_put_option_price()`: Computes the price of a put option.
///
/// The function returns the computed price based on the type of option provided.
///
pub fn black_scholes(
    option: Options,
    time_to_expiry: Option<f64>, // Time until expiration in years
) -> f64 {
    let (d1, d2, expiry_time) = calculate_d1_d2_and_time(&option, time_to_expiry);
    match option.option_style {
        OptionStyle::Call => {
            calculate_call_option_price(&option, d1, d2, expiry_time)
        }
        OptionStyle::Put => {
            calculate_put_option_price(&option, d1, d2, expiry_time)
        }
    }
}

/// Calculates the d1 and d2 values along with the time to expiry for an option.
///
/// # Parameters:
/// - `option`: A reference to an instance of `Options` that contains the option details.
/// - `time_to_expiry`: An optional `f64` value representing the already known time to expiry.
///    If not provided, it will be calculated based on the current date and the option's expiration date.
///
/// # Returns:
/// A tuple containing:
/// - `d1`: The first value computed based on the option's details and time to expiry.
/// - `d2`: The second value computed based on the option's details and time to expiry.
/// - `time_to_expiry`: The calculated or given time to expiry in years.
///
fn calculate_d1_d2_and_time(option: &Options, time_to_expiry: Option<f64>) -> (f64, f64, f64) {
    match time_to_expiry {
        None => {
            let now = chrono::Utc::now();
            let duration = option.expiration_date - now;
            let calculated_time_to_expiry = duration.num_days() as f64 / 365.0;
            let (d1, d2) = calculate_d_values(option, calculated_time_to_expiry);
            (d1, d2, calculated_time_to_expiry)
        }
        Some(expiry_time) => {
            let (d1, d2) = calculate_d_values(option, expiry_time);
            (d1, d2, expiry_time)
        }
    }
}

/// Calculates the price of a call option using the Black-Scholes formula.
///
/// # Parameters
/// - `option`: A reference to an `Options` struct containing the details of the option.
/// - `d1`: The d1 parameter calculated from the Black-Scholes model.
/// - `d2`: The d2 parameter calculated from the Black-Scholes model.
/// - `t`: The time to expiration in years.
///
/// # Returns
/// The price of the call option.
///
fn calculate_call_option_price(option: &Options, d1: f64, d2: f64, t: f64) -> f64 {
    option.underlying_price * big_n(d1)
        - option.strike_price * (-option.risk_free_rate * t).exp() * big_n(d2)
}

/// Calculates the price of a European put option using the Black-Scholes model.
///
/// # Parameters
///
/// - `option`: A reference to an `Options` struct which contains the details of the option such
///   as strike price, risk-free rate, and underlying asset price.
/// - `d1`: A `f64` value representing the `d1` parameter used in the Black-Scholes formula.
/// - `d2`: A `f64` value representing the `d2` parameter used in the Black-Scholes formula.
/// - `t`: A `f64` value representing the time to maturity, in years.
///
/// # Returns
///
/// - A `f64` value representing the calculated price of the put option.
///
/// # Formula
///
/// The function performs the following calculation:
///
/// ```text
/// Put Option Price = (Strike Price * exp(-Risk-Free Rate * Time to Maturity) * N(-d2))
///                    - (Underlying Price * N(-d1))
/// ```
///
/// where:
///
/// - `N(x)` is the cumulative distribution function (CDF) of the standard normal distribution.
///
/// # Example
///
fn calculate_put_option_price(option: &Options, d1: f64, d2: f64, t: f64) -> f64 {
    option.strike_price * (-option.risk_free_rate * t).exp() * big_n(-d2)
        - option.underlying_price * big_n(-d1)
}

#[cfg(test)]
mod tests_black_scholes {
    use super::*;
    use crate::model::option::Options;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use approx::assert_relative_eq;
    use chrono::{Duration, Utc};

    fn mock_options_call() -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: 100.0,
            strike_price: 100.0,
            implied_volatility: 0.2,
            risk_free_rate: 0.05,
            expiration_date: Utc::now() + Duration::days(365), // 1 year from now
            option_style: OptionStyle::Call,
            underlying_symbol: "".to_string(),
            quantity: 0,
        }
    }

    fn mock_options_put() -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: 100.0,
            strike_price: 100.0,
            implied_volatility: 0.2,
            risk_free_rate: 0.05,
            expiration_date: Utc::now() + Duration::days(365), // 1 year from now
            option_style: OptionStyle::Put,
            underlying_symbol: "".to_string(),
            quantity: 0,
        }
    }

    #[test]
    fn test_black_scholes_call_with_explicit_time_to_expiry() {
        let option = mock_options_call();
        let time_to_expiry = Some(1.0); // 1 year
        let price = black_scholes(option, time_to_expiry);

        assert_relative_eq!(price, 4.877, epsilon = 0.001);
    }

    #[test]
    fn test_black_scholes_put_with_explicit_time_to_expiry() {
        let option = mock_options_put();
        let time_to_expiry = Some(1.0); // 1 year
        let price = black_scholes(option, time_to_expiry);

        assert_relative_eq!(price, 3.757e-117, epsilon = 0.001);
    }

    #[test]
    fn test_black_scholes_call_without_explicit_time_to_expiry() {
        let option = mock_options_call();
        let price = black_scholes(option, None); // No explicit time to expiry

        assert_relative_eq!(price, 4.864, epsilon = 0.001);
    }

    #[test]
    fn test_black_scholes_put_without_explicit_time_to_expiry() {
        let option = mock_options_put();
        let price = black_scholes(option, None); // No explicit time to expiry

        assert_relative_eq!(price, 1.811e-117, epsilon = 0.001);
    }
}
