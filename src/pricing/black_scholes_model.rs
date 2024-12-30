/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 11/8/24
******************************************************************************/
use crate::constants::ZERO;
use crate::greeks::utils::{big_n, calculate_d_values};
use crate::model::decimal::f64_to_decimal;
use crate::model::option::Options;
use crate::model::types::{OptionStyle, OptionType, Side};
use num_traits::ToPrimitive;

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
#[allow(dead_code)]
pub fn black_scholes(option: &Options) -> f64 {
    let (d1, d2, expiry_time) = calculate_d1_d2_and_time(option);
    match option.option_type {
        OptionType::European => calculate_european_option_price(option, d1, d2, expiry_time),
        OptionType::American => ZERO,        // TODO: calculate this
        OptionType::Bermuda { .. } => ZERO,  // TODO: calculate this
        OptionType::Asian { .. } => ZERO,    // TODO: calculate this
        OptionType::Barrier { .. } => ZERO,  // TODO: calculate this
        OptionType::Binary { .. } => ZERO,   // TODO: calculate this
        OptionType::Lookback { .. } => ZERO, // TODO: calculate this
        OptionType::Compound { .. } => ZERO, // TODO: calculate this
        OptionType::Chooser { .. } => ZERO,  // TODO: calculate this
        OptionType::Cliquet { .. } => ZERO,  // TODO: calculate this
        OptionType::Rainbow { .. } => ZERO,  // TODO: calculate this
        OptionType::Spread { .. } => ZERO,   // TODO: calculate this
        OptionType::Quanto { .. } => ZERO,   // TODO: calculate this
        OptionType::Exchange { .. } => ZERO, // TODO: calculate this
        OptionType::Power { .. } => ZERO,    // TODO: calculate this
    }
}

/// Calculates the price of a European option.
///
/// This function calculates the price of a European option based on the given parameters.
/// The calculation varies depending on the position (long or short) stated in the `option`.
///
/// # Arguments
///
/// * `option` - A reference to an `Options` struct that contains the options details (e.g., side, strike price, etc.).
/// * `d1` - The d1 parameter used in the Black-Scholes model for pricing options.
/// * `d2` - The d2 parameter used in the Black-Scholes model for pricing options.
/// * `expiry_time` - The time remaining until the option's expiry, expressed in years.
///
/// # Returns
///
/// The calculated price of the European option as a floating-point number.
///
/// Note: This example uses placeholder values and the `Options` and `Side` structs should be defined accordingly in your codebase.
fn calculate_european_option_price(option: &Options, d1: f64, d2: f64, expiry_time: f64) -> f64 {
    match option.side {
        Side::Long => calculate_long_position(option, d1, d2, expiry_time),
        Side::Short => -calculate_long_position(option, d1, d2, expiry_time),
    }
}

/// Calculates the price of a long position in an option based on its style (Call or Put).
///
/// # Arguments
///
/// * `option` - A reference to an `Options` struct which contains the details of the option.
/// * `d1` - A floating-point value representing the first parameter (typically related to the Black-Scholes model).
/// * `d2` - A floating-point value representing the second parameter (typically related to the Black-Scholes model).
/// * `expiry_time` - A floating-point value representing the time to expiry of the option.
///
/// # Returns
///
/// A floating-point value representing the calculated price of the long position.
///
/// The function matches on the style of the option (Call or Put) and calls the respective price calculation function.
fn calculate_long_position(option: &Options, d1: f64, d2: f64, expiry_time: f64) -> f64 {
    match option.option_style {
        OptionStyle::Call => calculate_call_option_price(option, d1, d2, expiry_time),
        OptionStyle::Put => calculate_put_option_price(option, d1, d2, expiry_time),
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
fn calculate_d1_d2_and_time(option: &Options) -> (f64, f64, f64) {
    let calculated_time_to_expiry = option.time_to_expiration();
    let (d1, d2) = calculate_d_values(option).unwrap();
    (
        d1.to_f64().unwrap(),
        d2.to_f64().unwrap(),
        calculated_time_to_expiry,
    )
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
    let d1 = f64_to_decimal(d1).unwrap();
    let d2 = f64_to_decimal(d2).unwrap();
    let big_n_d1 = big_n(d1).unwrap().to_f64().unwrap();
    let big_n_d2 = big_n(d2).unwrap().to_f64().unwrap();

    option.underlying_price.value() * big_n_d1
        - option.strike_price.value() * (-option.risk_free_rate * t).exp() * big_n_d2
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
    let d1 = f64_to_decimal(d1).unwrap();
    let d2 = f64_to_decimal(d2).unwrap();
    let big_n_d1 = big_n(-d1).unwrap().to_f64().unwrap();
    let big_n_d2 = big_n(-d2).unwrap().to_f64().unwrap();

    option.strike_price.value() * (-option.risk_free_rate * t).exp() * big_n_d2
        - option.underlying_price.value() * big_n_d1
}

pub trait BlackScholes {
    fn get_option(&self) -> &Options;
    fn calculate_price_black_scholes(&self) -> f64 {
        let option = self.get_option();

        black_scholes(option)
    }
}

#[cfg(test)]
mod tests_black_scholes {
    use super::*;
    use crate::greeks::utils::{d1, d2};
    use crate::model::option::Options;
    use crate::model::types::Positive;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side, Positive::ZERO, SIZE_ONE};
    use crate::f2p;
    use approx::assert_relative_eq;

    fn mock_options_call() -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: f2p!(2476.6),
            strike_price: f2p!(2485.0),
            implied_volatility: 0.22,
            risk_free_rate: 0.006,
            expiration_date: ExpirationDate::Days(3.0),
            option_style: OptionStyle::Call,
            underlying_symbol: "GOLD".to_string(),
            quantity: SIZE_ONE,
            dividend_yield: ZERO,
            exotic_params: None,
        }
    }

    fn mock_options_simplest_call() -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: f2p!(100.0),
            strike_price: f2p!(100.0),
            implied_volatility: 0.01,
            risk_free_rate: ZERO,
            expiration_date: ExpirationDate::Days(365.0),
            option_style: OptionStyle::Call,
            underlying_symbol: "GOLD".to_string(),
            quantity: SIZE_ONE,
            dividend_yield: ZERO,

            exotic_params: None,
        }
    }

    fn mock_options_put() -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: f2p!(100.0),
            strike_price: f2p!(100.0),
            implied_volatility: 0.2,
            risk_free_rate: 0.05,
            expiration_date: ExpirationDate::Days(365.0), // 1 year from now
            option_style: OptionStyle::Put,
            underlying_symbol: "".to_string(),
            quantity: Positive::ZERO,
            dividend_yield: ZERO,
            exotic_params: None,
        }
    }

    #[test]
    fn test_black_scholes_simplest_call() {
        let mut option = mock_options_simplest_call();
        assert_relative_eq!(option.expiration_date.get_years(), 1.0, epsilon = 0.00001);
        let d1 = d1(
            option.underlying_price,
            option.strike_price,
            option.risk_free_rate,
            option.expiration_date.get_years(),
            option.implied_volatility,
        )
        .unwrap();
        assert_relative_eq!(d1.to_f64().unwrap(), 0.005, epsilon = 0.00001);
        let d2 = d2(
            option.underlying_price,
            option.strike_price,
            option.risk_free_rate,
            option.expiration_date.get_years(),
            option.implied_volatility,
        )
        .unwrap();

        assert_relative_eq!(d2.to_f64().unwrap(), -0.005, epsilon = 0.00001);
        let big_n_d1 = big_n(d1).unwrap();
        assert_relative_eq!(big_n_d1.to_f64().unwrap(), 0.501994, epsilon = 0.00001);
        let big_n_d2 = big_n(d2).unwrap();
        assert_relative_eq!(big_n_d2.to_f64().unwrap(), 0.498005, epsilon = 0.00001);

        let option_value = option.strike_price * big_n_d1.to_f64().unwrap()
            - option.underlying_price * big_n_d2.to_f64().unwrap();
        assert_relative_eq!(option_value.value(), 0.3989406, epsilon = 0.00001);
        let volatility = 0.2;
        let value_at_20 = volatility * option.strike_price * option_value;
        assert_relative_eq!(value_at_20, 7.97881, epsilon = 0.00001);

        let price = black_scholes(&option);

        assert_relative_eq!(price, 0.39894, epsilon = 0.001);
        assert_relative_eq!(price, option_value.value(), epsilon = 0.001);

        option.implied_volatility = 0.2;
        let price = black_scholes(&option);
        assert_relative_eq!(price, 7.965, epsilon = 0.001);

        option.implied_volatility = 0.2;
        option.strike_price = f2p!(50.0);
        let price = black_scholes(&option);
        assert_relative_eq!(price, 50.000, epsilon = 0.001);

        option.implied_volatility = 0.2;
        option.strike_price = f2p!(100.0);
        let price = black_scholes(&option);
        assert_relative_eq!(price, 7.96556, epsilon = 0.001);
    }

    #[test]
    fn test_black_scholes_simplest_call_2() {
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: f2p!(100.0),
            strike_price: f2p!(50.0),
            implied_volatility: 0.01,
            risk_free_rate: ZERO,
            expiration_date: ExpirationDate::Days(365.0),
            option_style: OptionStyle::Call,
            underlying_symbol: "GOLD".to_string(),
            quantity: SIZE_ONE,
            dividend_yield: ZERO,

            exotic_params: None,
        };
        // assert_relative_eq!(option.expiration_date.get_years(), 1.0, epsilon = 0.00001);
        let d1 = d1(
            option.underlying_price,
            option.strike_price,
            option.risk_free_rate,
            option.expiration_date.get_years(),
            option.implied_volatility,
        )
        .unwrap();
        assert_relative_eq!(d1.to_f64().unwrap(), 69.31971, epsilon = 0.00001);
        let d2 = d2(
            option.underlying_price,
            option.strike_price,
            option.risk_free_rate,
            option.expiration_date.get_years(),
            option.implied_volatility,
        )
        .unwrap();
        assert_relative_eq!(d2.to_f64().unwrap(), 69.3097180, epsilon = 0.00001);
        let big_n_d1 = big_n(d1).unwrap().to_f64().unwrap();
        assert_relative_eq!(big_n_d1, 1.0, epsilon = 0.00001);
        let big_n_d2 = big_n(d2).unwrap().to_f64().unwrap();
        assert_relative_eq!(big_n_d2, 1.0, epsilon = 0.00001);

        let option_value = option.underlying_price * big_n_d1 - option.strike_price * big_n_d2;
        assert_relative_eq!(option_value.value(), 50.0, epsilon = 0.00001);

        let volatility = 0.2;
        let value_at_20 = volatility * option.strike_price * option_value;
        assert_relative_eq!(value_at_20, 500.0, epsilon = 0.00001);

        let price = black_scholes(&option.clone());

        assert_relative_eq!(price, 50.0, epsilon = 0.001);
        assert_relative_eq!(price, option_value.value(), epsilon = 0.001);
    }

    #[test]
    fn test_black_scholes_simplest_call_3() {
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: f2p!(60.0),
            strike_price: f2p!(65.0),
            implied_volatility: 0.3,
            risk_free_rate: 0.08,
            expiration_date: ExpirationDate::Days(365.0 / 4.0),
            option_style: OptionStyle::Call,
            underlying_symbol: "GOLD".to_string(),
            quantity: SIZE_ONE,
            dividend_yield: ZERO,

            exotic_params: None,
        };
        assert_relative_eq!(option.expiration_date.get_years(), 0.25, epsilon = 0.00001);
        let d1 = d1(
            option.underlying_price,
            option.strike_price,
            option.risk_free_rate,
            option.expiration_date.get_years(),
            option.implied_volatility,
        )
        .unwrap();
        assert_relative_eq!(d1.to_f64().unwrap(), -0.325284, epsilon = 0.00001);
        let d2 = d2(
            option.underlying_price,
            option.strike_price,
            option.risk_free_rate,
            option.expiration_date.get_years(),
            option.implied_volatility,
        )
        .unwrap();
        assert_relative_eq!(d2.to_f64().unwrap(), -0.475284, epsilon = 0.00001);
        let big_n_d1 = big_n(d1).unwrap().to_f64().unwrap();
        assert_relative_eq!(big_n_d1, 0.3724827, epsilon = 0.00001);
        let big_n_d2 = big_n(d2).unwrap().to_f64().unwrap();
        assert_relative_eq!(big_n_d2, 0.3172920, epsilon = 0.00001);

        let option_value = option.underlying_price * big_n_d1
            - option.strike_price
                * big_n_d2
                * (-option.risk_free_rate * option.expiration_date.get_years()).exp();
        assert_relative_eq!(option_value.value(), 2.133368, epsilon = 0.00001);

        let price = black_scholes(&option.clone());

        assert_relative_eq!(price, 2.133368, epsilon = 0.001);
        assert_relative_eq!(price, option_value.value(), epsilon = 0.001);
        assert_relative_eq!(
            option.calculate_price_black_scholes(),
            option_value.value(),
            epsilon = 0.0001
        );
    }

    #[test]
    fn test_black_scholes_call_with_explicit_time_to_expiry() {
        let option = mock_options_call();
        let price = black_scholes(&option);
        assert_relative_eq!(price, 15.8756, epsilon = 0.001);
    }

    #[test]
    fn test_black_scholes_put_with_explicit_time_to_expiry() {
        let option = mock_options_put();
        let price = black_scholes(&option);

        assert_relative_eq!(price, 5.573526, epsilon = 0.001);
    }

    #[test]
    fn test_black_scholes_call_without_explicit_time_to_expiry() {
        let option = mock_options_call();
        let price = black_scholes(&option);
        assert_relative_eq!(price, 15.875638, epsilon = 0.001);
    }

    #[test]
    fn test_black_scholes_put_without_explicit_time_to_expiry() {
        let option = mock_options_put();
        let price = black_scholes(&option);

        assert_relative_eq!(price, 5.5735260, epsilon = 0.001);
    }
}

#[cfg(test)]
mod tests_black_scholes_trait {
    use super::*;
    use crate::model::types::Positive;
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option;
    use crate::f2p;
    use approx::assert_relative_eq;

    // Mock struct to implement BlackScholes trait
    struct MockOption {
        option: Options,
    }

    impl MockOption {
        fn new(option: Options) -> Self {
            MockOption { option }
        }
    }

    impl BlackScholes for MockOption {
        fn get_option(&self) -> &Options {
            &self.option
        }
    }

    #[test]
    fn test_at_the_money_call() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(100.0), // underlying price
            f2p!(1.0),   // quantity
            f2p!(100.0), // strike price
            0.2,         // volatility
        );
        let mock = MockOption::new(option);
        let price = mock.calculate_price_black_scholes();
        assert_relative_eq!(price, 2.4933768, epsilon = 1e-7);
    }

    #[test]
    fn test_in_the_money_call() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(100.0), // underlying price
            f2p!(1.0),   // quantity
            f2p!(90.0),  // strike price
            0.2,         // volatility
        );
        let mock = MockOption::new(option);
        let price = mock.calculate_price_black_scholes();
        assert_relative_eq!(price, 10.427673877, epsilon = 1e-7);
    }

    #[test]
    fn test_out_of_the_money_call() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(100.0), // underlying price
            f2p!(1.0),   // quantity
            f2p!(110.0), // strike price
            0.2,         // volatility
        );
        let mock = MockOption::new(option);
        let price = mock.calculate_price_black_scholes();
        assert_relative_eq!(price, 0.14256994168, epsilon = 1e-7);
    }

    #[test]
    fn test_at_the_money_put() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            f2p!(100.0), // underlying price
            f2p!(1.0),   // quantity
            f2p!(100.0), // strike price
            0.2,         // volatility
        );
        let mock = MockOption::new(option);
        let price = mock.calculate_price_black_scholes();
        assert_relative_eq!(price, 2.08326119582, epsilon = 1e-7);
    }

    #[test]
    fn test_high_volatility() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(100.0), // underlying price
            f2p!(1.0),   // quantity
            f2p!(100.0), // strike price
            0.5,         // high volatility
        );
        let mock = MockOption::new(option);
        let price = mock.calculate_price_black_scholes();
        assert_relative_eq!(price, 5.9094479287, epsilon = 1e-7);
    }

    #[test]
    #[should_panic]
    fn test_zero_volatility() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(100.0), // underlying price
            f2p!(1.0),   // quantity
            f2p!(100.0), // strike price
            0.0,         // zero volatility
        );
        let mock = MockOption::new(option);
        let _price = mock.calculate_price_black_scholes();
    }

    #[test]
    fn test_short_call() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            f2p!(100.0), // underlying price
            f2p!(1.0),   // quantity
            f2p!(100.0), // strike price
            0.2,         // volatility
        );
        let mock = MockOption::new(option);
        let price = mock.calculate_price_black_scholes();
        assert_relative_eq!(price, -2.49337681940, epsilon = 1e-7);
    }

    #[test]
    fn test_short_put() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            f2p!(100.0), // underlying price
            f2p!(1.0),   // quantity
            f2p!(100.0), // strike price
            0.2,         // volatility
        );
        let mock = MockOption::new(option);
        let price = mock.calculate_price_black_scholes();
        assert_relative_eq!(price, -2.0832611958, epsilon = 1e-7);
    }

    #[test]
    fn test_with_different_quantity() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(100.0), // underlying price
            f2p!(10.0),  // quantity
            f2p!(100.0), // strike price
            0.2,         // volatility
        );
        let mock = MockOption::new(option);
        let price = mock.calculate_price_black_scholes();
        assert_relative_eq!(price, 2.4933768194037, epsilon = 1e-7);
    }
}

#[cfg(test)]
mod tests_black_scholes_trait_bis {
    use super::*;
    use crate::model::types::Positive;
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option;
    use crate::f2p;
    use approx::assert_relative_eq;

    struct MockOption {
        option: Options,
    }

    impl MockOption {
        fn new(option: Options) -> Self {
            MockOption { option }
        }
    }

    impl BlackScholes for MockOption {
        fn get_option(&self) -> &Options {
            &self.option
        }
    }

    #[test]
    fn test_call_put_parity() {
        let call_option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(100.0),
            f2p!(1.0),
            f2p!(100.0),
            0.2,
        );

        let put_option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            f2p!(100.0),
            f2p!(1.0),
            f2p!(100.0),
            0.2,
        );

        let call_mock = MockOption::new(call_option);
        let put_mock = MockOption::new(put_option);

        let call_price = call_mock.calculate_price_black_scholes();
        let put_price = put_mock.calculate_price_black_scholes();

        let r: f64 = 0.05;
        let t: f64 = 30.0 / 365.0;
        let s: f64 = 100.0;
        let k: f64 = 100.0;

        let parity_value = call_price - put_price;
        let theoretical_value = s - k * f64::exp(-r * t);

        assert_relative_eq!(parity_value, theoretical_value, epsilon = 0.01);
    }

    #[test]
    fn test_call_put_parity_short() {
        let call_option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            f2p!(100.0),
            f2p!(1.0),
            f2p!(100.0),
            0.2,
        );

        let put_option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            f2p!(100.0),
            f2p!(1.0),
            f2p!(100.0),
            0.2,
        );

        let call_mock = MockOption::new(call_option);
        let put_mock = MockOption::new(put_option);

        let call_price = call_mock.calculate_price_black_scholes();
        let put_price = put_mock.calculate_price_black_scholes();

        let r: f64 = 0.05;
        let t: f64 = 30.0 / 365.0;
        let s: f64 = 100.0;
        let k: f64 = 100.0;

        let parity_value = call_price - put_price;
        let theoretical_value = s - k * f64::exp(-r * t);

        assert_relative_eq!(parity_value, -theoretical_value, epsilon = 0.01);
    }

    #[test]
    fn test_monotonicity_with_strike() {
        let call1 = MockOption::new(create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(100.0),
            f2p!(1.0),
            f2p!(90.0),
            0.2,
        ));

        let call2 = MockOption::new(create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(100.0),
            f2p!(1.0),
            f2p!(100.0),
            0.2,
        ));

        let call3 = MockOption::new(create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(100.0),
            f2p!(1.0),
            f2p!(110.0),
            0.2,
        ));

        let price1 = call1.calculate_price_black_scholes();
        let price2 = call2.calculate_price_black_scholes();
        let price3 = call3.calculate_price_black_scholes();

        assert!(price1 > price2);
        assert!(price2 > price3);
    }

    #[test]
    #[should_panic]
    fn test_zero_volatility_call() {
        let option = MockOption::new(create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(100.0),
            f2p!(1.0),
            f2p!(95.0),
            0.0,
        ));

        let _price = option.calculate_price_black_scholes();
        let r: f64 = 0.05;
        let t: f64 = 30.0 / 365.0;
        let s: f64 = 100.0;
        let k: f64 = 95.0;

        let _theoretical_price = 0.0f64.max(s - k * f64::exp(-r * t));
    }

    #[test]
    fn test_deep_itm_call() {
        let option = MockOption::new(create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(150.0),
            f2p!(1.0),
            f2p!(100.0),
            0.2,
        ));

        let price = option.calculate_price_black_scholes();
        let r: f64 = 0.05;
        let t: f64 = 30.0 / 365.0;
        let s: f64 = 150.0;
        let k: f64 = 100.0;

        let intrinsic_value = s - k * f64::exp(-r * t);
        assert!(price > intrinsic_value);
        assert_relative_eq!(price, intrinsic_value, epsilon = 5.0);
    }

    #[test]
    fn test_deep_otm_call() {
        let option = MockOption::new(create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(100.0),
            f2p!(1.0),
            f2p!(200.0),
            0.2,
        ));

        let price = option.calculate_price_black_scholes();
        assert!(price < 0.1);
    }

    #[test]
    fn test_monotonicity_with_volatility() {
        let call1 = MockOption::new(create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(100.0),
            f2p!(1.0),
            f2p!(100.0),
            0.1,
        ));

        let call2 = MockOption::new(create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(100.0),
            f2p!(1.0),
            f2p!(100.0),
            0.2,
        ));

        let call3 = MockOption::new(create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(100.0),
            f2p!(1.0),
            f2p!(100.0),
            0.3,
        ));

        let price1 = call1.calculate_price_black_scholes();
        let price2 = call2.calculate_price_black_scholes();
        let price3 = call3.calculate_price_black_scholes();

        assert!(price1 < price2);
        assert!(price2 < price3);
    }
}
