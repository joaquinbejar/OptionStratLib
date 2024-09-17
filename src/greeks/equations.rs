/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 11/8/24
******************************************************************************/
use tracing::info;
use crate::constants::ZERO;
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
    if option.implied_volatility == 0.0 {
        let sign = if option.is_long() { 1.0 } else { -1.0 };
        match option.option_style {
            OptionStyle::Call => {
                if option.underlying_price >= option.strike_price {
                    return sign * 1.0; // Delta es 1 para Call in-the-money
                } else {
                    return 0.0; // Delta es 0 para Call out-of-the-money
                }
            },
            OptionStyle::Put => {
                if option.underlying_price <= option.strike_price {
                    return sign * -1.0; // Delta es -1 para Put in-the-money
                } else {
                    return 0.0; // Delta es 0 para Put out-of-the-money
                }
            }
        }
    }

    let d1 = d1(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years(),
        option.implied_volatility,
    );
    let sign = if option.is_long() { 1.0 } else { -1.0 };

    match option.option_style {
        OptionStyle::Call => {
            info!("{}", d1);
            sign * big_n(d1) * (-option.dividend_yield * option.time_to_expiration()).exp()
        },
        OptionStyle::Put => sign * (big_n(d1) - 1.0) * (-option.dividend_yield * option.time_to_expiration()).exp(),
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

/// Calculates the rho of an options contract.
///
/// Rho measures the sensitivity of the options price to changes in the risk-free interest rate.
/// This function computes the rho based on the given options parameters.
///
/// # Parameters
///
/// - `option`: A reference to an `Options` struct which contains all necessary information about the options contract.
///
/// The `Options` struct should include the following fields:
/// - `underlying_price`: The current price of the underlying asset.
/// - `strike_price`: The strike price of the option.
/// - `risk_free_rate`: The risk-free interest rate.
/// - `expiration_date`: An object providing the expiration date of the option, with a method `get_years()` that returns the term in years.
/// - `implied_volatility`: The implied volatility of the option.
/// - `option_style`: The style of the option, either `Call` or `Put`.
///
/// # Returns
///
/// A `f64` value representing the rho of the options contract.
///
/// # Formula
///
/// For a Call option:
/// \[ \rho = K \cdot T \cdot e^{-rT} \cdot N(d2) \]
///
/// For a Put option:
/// \[ \rho = -K \cdot T \cdot e^{-rT} \cdot N(-d2) \]
///
/// Where:
/// - \( K \) is `strike_price`
/// - \( T \) is `expiration_date.get_years()`
/// - \( r \) is `risk_free_rate`
/// - \( N(d2) \) is the cumulative distribution function of the standard normal distribution evaluated at `d2`
///
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

/// Computes the dividend rate sensitivity (rho) of an option.
///
/// The `rho_d` function calculates the sensitivity of the option price
/// with respect to the dividend yield of the underlying asset. It takes
/// into account whether the option is a call or a put and uses various
/// financial parameters to compute the result.
///
/// # Parameters
///
/// - `option`: A reference to an `Options` struct that holds important
///   information about the option including underlying price, strike
///   price, risk-free rate, expiration date, implied volatility, and
///   dividend yield.
///
/// # Returns
///
/// Returns a `f64` value representing the rate of change of the option
/// price concerning the dividend yield.
///
/// # Calculations
///
/// - First, the function calculates the `d1` value using the provided
///   option parameters.
/// - Then, it matches on the option style (`Call` or `Put`) to compute
///   the corresponding rho value.
/// - For a `Call` option, the rho value is calculated using the formula:
///
/// ```text
/// -T * S * e^(-q * T) * N(d1)
/// ```
///
/// - For a `Put` option, the rho value is calculated using the formula:
///
/// ```text
/// T * S * e^(-q * T) * N(-d1)
/// ```
///
/// where:
/// - `T` is the expiration time in years,
/// - `S` is the underlying price,
/// - `q` is the dividend yield,
/// - `N` is the cumulative distribution function of the standard normal distribution,
/// - `d1` is a calculated parameter for option pricing.
///
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
mod tests_delta_equations {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle, Side};
    use approx::assert_relative_eq;
    use crate::constants::ZERO;
    use crate::model::utils::create_sample_option;
    use crate::utils::logger::setup_logger;


    #[test]
    fn test_delta_no_volatility_itm() {
        setup_logger();
        let option = create_sample_option(OptionStyle::Call, Side::Long, 150.0, 1, 150.0, ZERO);
        let delta_value = delta(&option);
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, 1.0, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_no_volatility_otm() {
        setup_logger();
        let option = create_sample_option(OptionStyle::Call, Side::Long, 110.0, 1, 150.0, ZERO);
        let delta_value = delta(&option);
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, ZERO, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_no_volatility_itm_put() {
        setup_logger();
        let option = create_sample_option(OptionStyle::Put, Side::Long, 150.0, 1, 150.0, ZERO);
        let delta_value = delta(&option);
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, -1.0, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_no_volatility_otm_put() {
        setup_logger();
        let option = create_sample_option(OptionStyle::Put, Side::Long, 160.0, 1, 150.0, ZERO);
        let delta_value = delta(&option);
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, ZERO, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_no_volatility_itm_short() {
        setup_logger();
        let option = create_sample_option(OptionStyle::Call, Side::Short, 150.0, 1, 150.0, ZERO);
        let delta_value = delta(&option);
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, -1.0, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_no_volatility_otm_short() {
        setup_logger();
        let option = create_sample_option(OptionStyle::Call, Side::Short, 110.0, 1, 150.0, ZERO);
        let delta_value = delta(&option);
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, ZERO, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_no_volatility_itm_put_short() {
        setup_logger();
        let option = create_sample_option(OptionStyle::Put, Side::Short, 150.0, 1, 150.0, ZERO);
        let delta_value = delta(&option);
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, 1.0, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_no_volatility_otm_put_short() {
        setup_logger();
        let option = create_sample_option(OptionStyle::Put, Side::Short, 160.0, 1, 150.0, ZERO);
        let delta_value = delta(&option);
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, ZERO, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_deep_in_the_money_call() {
        setup_logger();
        let option = create_sample_option(OptionStyle::Call, Side::Long, 150.0, 1, 100.0, 0.20);
        let delta_value = delta(&option);
        info!("Deep ITM Call Delta: {}", delta_value);
        assert_relative_eq!(delta_value, 0.9991784198733309, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_deep_out_of_the_money_call() {
        let option = create_sample_option(OptionStyle::Call, Side::Long, 50.0, 1, 100.0, 0.20);
        let delta_value = delta(&option);
        info!("Deep OTM Call Delta: {}", delta_value);
        assert_relative_eq!(delta_value, 2.0418256951423236e-33, epsilon = 1e-4);
    }

    #[test]
    fn test_delta_at_the_money_put() {
        let option = create_sample_option(OptionStyle::Put, Side::Long, 100.0, 1, 100.0, 0.20);
        let delta_value = delta(&option);
        info!("ATM Put Delta: {}", delta_value);
        assert_relative_eq!(delta_value, -0.4596584975686261, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_short_term_high_volatility() {
        let mut option = create_sample_option(OptionStyle::Call, Side::Long, 100.0, 1, 100.0, 0.50);
        option.expiration_date = ExpirationDate::Days(7.0);
        let delta_value = delta(&option);
        info!("Short-term High Vol Call Delta: {}", delta_value);
        assert_relative_eq!(delta_value, 0.519229469584234, epsilon = 1e-4);
    }

    #[test]
    fn test_delta_long_term_low_volatility() {
        let mut option = create_sample_option(OptionStyle::Put, Side::Long, 100.0, 1, 100.0, 0.10);
        option.expiration_date = ExpirationDate::Days(365.0);
        let delta_value = delta(&option);
        info!("Long-term Low Vol Put Delta: {}", delta_value);
        assert_relative_eq!(delta_value, -0.2882625994992622, epsilon = 1e-8);
    }
}


#[cfg(test)]
mod tests_gamma_equations {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle, Side};
    use approx::assert_relative_eq;
    use crate::model::utils::create_sample_option;
    use crate::utils::logger::setup_logger;

    #[test]
    fn test_gamma_deep_in_the_money_call() {
        setup_logger();
        let option = create_sample_option(OptionStyle::Call, Side::Long, 150.0, 1, 120.0, 0.2);
        let gamma_value = gamma(&option);
        info!("Deep ITM Call Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.000016049457791525, epsilon = 1e-8);
    }

    #[test]
    fn test_gamma_deep_out_of_the_money_call() {
        let option = create_sample_option(OptionStyle::Call, Side::Long, 50.0, 1, 100.0, 0.20);
        let gamma_value = gamma(&option);
        info!("Deep OTM Call Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 8.596799333253201e-33, epsilon = 1e-34);
    }

    #[test]
    fn test_gamma_at_the_money_put() {
        let option = create_sample_option(OptionStyle::Put, Side::Long, 100.0, 1, 100.0, 0.20);
        let gamma_value = gamma(&option);
        info!("ATM Put Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.06917076441486919, epsilon = 1e-8);
    }

    #[test]
    fn test_gamma_short_term_high_volatility() {
        let mut option = create_sample_option(OptionStyle::Call, Side::Long, 100.0, 1, 100.0, 0.50);
        option.expiration_date = ExpirationDate::Days(7.0);
        let gamma_value = gamma(&option);
        info!("Short-term High Vol Call Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.05753657912620555, epsilon = 1e-8);
    }

    #[test]
    fn test_gamma_long_term_low_volatility() {
        let mut option = create_sample_option(OptionStyle::Put, Side::Long, 100.0, 1, 100.0, 0.10);
        option.expiration_date = ExpirationDate::Days(365.0);
        let gamma_value = gamma(&option);
        info!("Long-term Low Vol Put Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.033953150664723986, epsilon = 1e-8);
    }

    #[test]
    fn test_gamma_zero_volatility() {
        let option = create_sample_option(OptionStyle::Call, Side::Long, 100.0, 1, 100.0, 0.0);
        let gamma_value = gamma(&option);
        info!("Zero Volatility Call Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.0, epsilon = 1e-8);
    }

    #[test]
    fn test_gamma_extreme_high_volatility() {
        let option = create_sample_option(OptionStyle::Put, Side::Short, 100.0, 1, 100.0, 5.0);
        let gamma_value = gamma(&option);
        info!("Extreme High Volatility Put Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.000123456789, epsilon = 1e-8); // Ajusta el valor según cálculos
    }
}
