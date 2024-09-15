/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 11/8/24
******************************************************************************/

use crate::greeks::utils::{big_n, d1, d2, n};
use crate::model::option::Options;
use crate::model::types::OptionStyle;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Greek {
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub rho: f64,
    pub rho_d: f64,
}

pub trait Greeks {
    fn greeks(&self) -> Greek;
}

/// Calculates the delta of a financial option.
///
/// The delta measures the sensitivity of the option's price to changes in the price
/// of the underlying asset. It is a key metric in options trading and risk management.
///
/// # Parameters
///
/// - `option`: A reference to an `Options` struct which holds relevant data for the option such as:
///   - `underlying_price`: The current price of the underlying asset.
///   - `strike_price`: The strike price of the option.
///   - `risk_free_rate`: The risk-free interest rate over the life of the option.
///   - `expiration_date`: The expiration date of the option, from which we get the time to expiration in years.
///   - `implied_volatility`: The implied volatility of the underlying asset.
///   - `dividend_yield`: The dividend yield of the underlying asset.
///   - `option_style`: The style of the option, which can be either a `Call` or a `Put`.
///
/// # Returns
///
/// - `f64`: The delta of the option.
///
/// The function internally calls the `d1` function to calculate a component needed for the delta.
/// Depending on the option style (`Call` or `Put`), it then computes the delta using the cumulative
/// distribution function (`big_n`) of the standard normal distribution.
///
/// # Note
///
/// This function assumes that all input values are properly validated and that `option.expiration_date.get_years()`
/// correctly returns the time to expiration in years.
///
/// # Panics
///
/// This function will not panic if the input `Options` struct adheres to the expected format and all methods
/// (like `get_years`) function correctly.
#[allow(dead_code)]
pub fn delta(option: &Options) -> f64 {
    let d1 = d1(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years(),
        option.implied_volatility,
    );

    match option.option_style {
        OptionStyle::Call => {
            (-option.dividend_yield * option.expiration_date.get_years()).exp() * big_n(d1)
        }
        OptionStyle::Put => {
            (-option.dividend_yield * option.expiration_date.get_years()).exp() * (big_n(d1) - 1.0)
        }
    }
}

/// Computes the gamma of an option.
///
/// Gamma measures the rate of change of delta with respect to changes in the underlying price of the asset.
/// It is a second-order derivative of the option price.
///
/// # Arguments
///
/// * `option` - A reference to an `Options` struct containing the necessary parameters to compute the gamma.
///
/// # Returns
///
/// A `f64` value representing the gamma of the option.
///
/// # Calculation
///
/// Gamma is computed using the following formula:
///
/// ```text
/// Gamma = (e^(-dividend_yield * T) * N'(d1)) / (S * σ * sqrt(T))
/// ```
///
/// Where:
/// * `N'(d1)` is the standard normal probability density function evaluated at `d1`.
/// * `S` is the underlying price of the asset.
/// * `σ` (sigma) is the implied volatility.
/// * `T` is the time to expiration in years.
///
/// The function first calculates `d1` using the `d1` function and then applies the gamma formula.
/// The exponential expression accounts for continuous dividend yield over the life of the option.
///
#[allow(dead_code)]
pub fn gamma(option: &Options) -> f64 {
    let d1 = d1(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years(),
        option.implied_volatility,
    );

    (-option.dividend_yield * option.expiration_date.get_years()).exp() * n(d1)
        / (option.underlying_price
            * option.implied_volatility
            * option.expiration_date.get_years().sqrt())
}

/// Computes the Theta value for a given option.
///
/// Theta measures the sensitivity of the option's price with respect to time decay.
/// It represents the rate at which the value of the option decreases as the expiration
/// date approaches, holding all other inputs constant.
///
/// # Parameters
/// - `option`: A reference to an `Options` struct that encapsulates various parameters
///   necessary for the calculation.
///   - `underlying_price`: Current price of the underlying asset.
///   - `strike_price`: Strike price of the option.
///   - `risk_free_rate`: Risk-free interest rate.
///   - `expiration_date`: Expiration date of the option (needs to provide `get_years` method).
///   - `implied_volatility`: Implied volatility of the underlying asset.
///   - `dividend_yield`: Expected dividend yield of the underlying asset.
///   - `option_style`: Style of the option, either `Call` or `Put`.
///
/// # Returns
/// - `f64`: The calculated Theta value for the given option.
///
/// # Formula
/// The function utilizes the Black-Scholes model to compute Theta. It applies
/// different formulas for call and put options:
///
/// For Call Options:
/// `common_term - risk_free_rate * strike_price * exp(-risk_free_rate * expiration_years) * big_n(d2) + dividend_yield * underlying_price * exp(-dividend_yield * expiration_years) * big_n(d1)`
///
/// For Put Options:
/// `common_term + risk_free_rate * strike_price * exp(-risk_free_rate * expiration_years) * big_n(-d2) - dividend_yield * underlying_price * exp(-dividend_yield * expiration_years) * big_n(-d1)`
///
/// Where:
/// - `common_term = -underlying_price * implied_volatility * exp(-dividend_yield * expiration_years) * n(d1) / (2.0 * sqrt(expiration_years))`
///
/// The `d1` and `d2` terms are intermediate calculations used in the Black-Scholes model.
///
/// - `d1` is calculated using the `d1` function.
/// - `d2` is calculated using the `d2` function.
///
/// - `n(.)` and `big_n(.)` refer to the probability density function (pdf) and the cumulative
///   distribution function (cdf) of the standard normal distribution respectively.
#[allow(dead_code)]
pub fn theta(option: &Options) -> f64 {
    let d1 = d1(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years(),
        option.implied_volatility,
    );
    let d2 = d2(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years(),
        option.implied_volatility,
    );

    let common_term = -option.underlying_price
        * option.implied_volatility
        * (-option.dividend_yield * option.expiration_date.get_years()).exp()
        * n(d1)
        / (2.0 * option.expiration_date.get_years().sqrt());

    match option.option_style {
        OptionStyle::Call => {
            common_term
                - option.risk_free_rate
                    * option.strike_price
                    * (-option.risk_free_rate * option.expiration_date.get_years()).exp()
                    * big_n(d2)
                + option.dividend_yield
                    * option.underlying_price
                    * (-option.dividend_yield * option.expiration_date.get_years()).exp()
                    * big_n(d1)
        }
        OptionStyle::Put => {
            common_term
                + option.risk_free_rate
                    * option.strike_price
                    * (-option.risk_free_rate * option.expiration_date.get_years()).exp()
                    * big_n(-d2)
                - option.dividend_yield
                    * option.underlying_price
                    * (-option.dividend_yield * option.expiration_date.get_years()).exp()
                    * big_n(-d1)
        }
    }
}

/// Calculates the "vega" of an option, which measures the sensitivity of the option's price
/// to changes in the volatility of the underlying asset. Vega indicates how much the price
/// of an option is expected to change for a 1% change in the implied volatility.
///
/// # Arguments
///
/// * `option` - A reference to an `Options` struct containing all the necessary parameters
///   for the calculation including underlying price, strike price, risk-free rate,
///   expiration date, implied volatility, and dividend yield.
///
/// # Returns
///
/// * `f64` - The calculated vega value of the option.
///
/// # Implementation Details
///
/// The formula used for calculating vega is based on the Black-Scholes option pricing model.
/// - `d1` is calculated using several parameters of the `Options` struct.
/// - The underlying price is then multiplied by the exponential term of the negative
///   dividend yield times the time to expiration.
/// - This product is further multiplied by the value of the normal distribution `n(d1)`
///   and the square root of the time to expiration.
///
#[allow(dead_code)]
pub fn vega(option: &Options) -> f64 {
    let d1 = d1(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years(),
        option.implied_volatility,
    );

    option.underlying_price
        * (-option.dividend_yield * option.expiration_date.get_years()).exp()
        * n(d1)
        * option.expiration_date.get_years().sqrt()
}

#[allow(dead_code)]
pub fn rho(option: &Options) -> f64 {
    let d2 = d2(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years(),
        option.implied_volatility,
    );

    match option.option_style {
        OptionStyle::Call => {
            option.strike_price
                * option.expiration_date.get_years()
                * (-option.risk_free_rate * option.expiration_date.get_years()).exp()
                * big_n(d2)
        }
        OptionStyle::Put => {
            -option.strike_price
                * option.expiration_date.get_years()
                * (-option.risk_free_rate * option.expiration_date.get_years()).exp()
                * big_n(-d2)
        }
    }
}

#[allow(dead_code)]
pub fn rho_d(option: &Options) -> f64 {
    let d1 = d1(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years(),
        option.implied_volatility,
    );

    match option.option_style {
        OptionStyle::Call => {
            -option.expiration_date.get_years()
                * option.underlying_price
                * (-option.dividend_yield * option.expiration_date.get_years()).exp()
                * big_n(d1)
        }
        OptionStyle::Put => {
            option.expiration_date.get_years()
                * option.underlying_price
                * (-option.dividend_yield * option.expiration_date.get_years()).exp()
                * big_n(-d1)
        }
    }
}

#[cfg(test)]
mod tests_greeks_equations {
    use super::*;
    use crate::model::option::Options;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use approx::assert_relative_eq;

    fn create_test_option(style: OptionStyle) -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: 100.0,
            strike_price: 100.0,
            risk_free_rate: 0.05,
            dividend_yield: 0.02,
            implied_volatility: 7.2,
            option_style: style,
            underlying_symbol: "".to_string(),
            expiration_date: Default::default(),
            quantity: 0,
            exotic_params: None,
        }
    }

    #[test]
    fn test_delta() {
        let call_option = create_test_option(OptionStyle::Call);
        let put_option = create_test_option(OptionStyle::Put);

        assert_relative_eq!(delta(&call_option), 0.9801, epsilon = 1e-4);
        assert_relative_eq!(delta(&put_option), -0.000151, epsilon = 1e-6);
    }

    #[test]
    fn test_gamma() {
        let option = create_test_option(OptionStyle::Call); // Gamma is the same for calls and puts

        assert_relative_eq!(gamma(&option), 8.124480543702491e-7, epsilon = 1e-6);
    }

    #[test]
    fn test_theta() {
        let call_option = create_test_option(OptionStyle::Call);
        let put_option = create_test_option(OptionStyle::Put);

        assert_relative_eq!(theta(&call_option), 1.7487299, epsilon = 1e-4);
        assert_relative_eq!(theta(&put_option), 4.5444, epsilon = 1e-4);
    }

    #[test]
    fn test_vega() {
        let option = create_test_option(OptionStyle::Call); // Vega is the same for calls and puts
        assert_relative_eq!(vega(&option), 0.05849, epsilon = 1e-4);
    }

    #[test]
    fn test_rho() {
        let call_option = create_test_option(OptionStyle::Call);
        let put_option = create_test_option(OptionStyle::Put);

        assert_relative_eq!(rho(&call_option), 0.015544, epsilon = 1e-4);
        assert_relative_eq!(rho(&put_option), -95.1073, epsilon = 1e-4);
    }

    #[test]
    fn test_rho_d() {
        let call_option = create_test_option(OptionStyle::Call);
        let put_option = create_test_option(OptionStyle::Put);

        assert_relative_eq!(rho_d(&call_option), -98.00468, epsilon = 1e-4);
        assert_relative_eq!(rho_d(&put_option), 0.01518, epsilon = 1e-4);
    }
}
