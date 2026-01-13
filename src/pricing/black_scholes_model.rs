/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 11/8/24
******************************************************************************/
use crate::Options;
use crate::error::PricingError;
use crate::greeks::{big_n, calculate_d_values};
use crate::model::types::{OptionStyle, OptionType, Side};
use rust_decimal::{Decimal, MathematicalOps};
use tracing::trace;

/// Computes the price of an option using the Black-Scholes model.
///
/// # Arguments
///
/// * `option` - An `Options` struct containing all the relevant parameters for the option
///   (e.g., strike price, underlying asset price, volatility, etc.).
///
/// # Returns
///
/// * `Ok(Decimal)` - The calculated price of the option.
/// * `Err(PricingError)` - If the option type is not supported or calculation fails.
///
/// # Supported Option Types
///
/// Currently, only **European** options are supported by the Black-Scholes model.
/// The following exotic option types will return `PricingError::UnsupportedOptionType`:
/// - American, Bermuda, Asian, Barrier, Binary, Lookback, Compound, Chooser,
///   Cliquet, Rainbow, Spread, Quanto, Exchange, Power
///
/// # Description
///
/// This function leverages the Black-Scholes model to determine the price of
/// either a call option or a put option. It first calculates the `d1` and `d2`
/// parameters required for the model and then uses the appropriate pricing
/// formula based on the option style (Call or Put).
///
/// ## Black-Scholes Formula
///
/// For a **Call** option:
/// ```text
/// C = S * N(d1) - K * e^(-rT) * N(d2)
/// ```
///
/// For a **Put** option:
/// ```text
/// P = K * e^(-rT) * N(-d2) - S * N(-d1)
/// ```
///
/// Where:
/// - `S` = Current underlying price
/// - `K` = Strike price
/// - `r` = Risk-free interest rate
/// - `T` = Time to expiration (in years)
/// - `N()` = Cumulative standard normal distribution
/// - `d1 = (ln(S/K) + (r + σ²/2) * T) / (σ * √T)`
/// - `d2 = d1 - σ * √T`
///
pub fn black_scholes(option: &Options) -> Result<Decimal, PricingError> {
    let (d1, d2, expiry_time) = calculate_d1_d2_and_time(option)?;
    match option.option_type {
        OptionType::European => calculate_european_option_price(option, d1, d2, expiry_time),
        OptionType::American => Err(PricingError::unsupported_option_type(
            "American",
            "Black-Scholes",
        )),
        OptionType::Bermuda { .. } => Err(PricingError::unsupported_option_type(
            "Bermuda",
            "Black-Scholes",
        )),
        OptionType::Asian { .. } => crate::pricing::asian::asian_black_scholes(option),
        OptionType::Barrier { .. } => crate::pricing::barrier::barrier_black_scholes(option),
        OptionType::Binary { .. } => crate::pricing::binary::binary_black_scholes(option),
        OptionType::Lookback { .. } => crate::pricing::lookback::lookback_black_scholes(option),
        OptionType::Compound { .. } => crate::pricing::compound::compound_black_scholes(option),
        OptionType::Chooser { .. } => crate::pricing::chooser::chooser_black_scholes(option),
        OptionType::Cliquet { .. } => crate::pricing::cliquet::cliquet_black_scholes(option),
        OptionType::Rainbow { .. } => Err(PricingError::unsupported_option_type(
            "Rainbow",
            "Black-Scholes",
        )),
        OptionType::Spread { .. } => Err(PricingError::unsupported_option_type(
            "Spread",
            "Black-Scholes",
        )),
        OptionType::Quanto { .. } => Err(PricingError::unsupported_option_type(
            "Quanto",
            "Black-Scholes",
        )),
        OptionType::Exchange { .. } => Err(PricingError::unsupported_option_type(
            "Exchange",
            "Black-Scholes",
        )),
        OptionType::Power { .. } => Err(PricingError::unsupported_option_type(
            "Power",
            "Black-Scholes",
        )),
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
fn calculate_european_option_price(
    option: &Options,
    d1: Decimal,
    d2: Decimal,
    expiry_time: Decimal,
) -> Result<Decimal, PricingError> {
    match option.side {
        Side::Long => calculate_long_position(option, d1, d2, expiry_time),
        Side::Short => Ok(-calculate_long_position(option, d1, d2, expiry_time)?),
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
fn calculate_long_position(
    option: &Options,
    d1: Decimal,
    d2: Decimal,
    expiry_time: Decimal,
) -> Result<Decimal, PricingError> {
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
///   If not provided, it will be calculated based on the current date and the option's expiration date.
///
/// # Returns:
/// A tuple containing:
/// - `d1`: The first value computed based on the option's details and time to expiry.
/// - `d2`: The second value computed based on the option's details and time to expiry.
/// - `time_to_expiry`: The calculated or given time to expiry in years.
///
fn calculate_d1_d2_and_time(option: &Options) -> Result<(Decimal, Decimal, Decimal), PricingError> {
    let calculated_time_to_expiry: Decimal = option.time_to_expiration()?.to_dec();
    let (d1, d2) = calculate_d_values(option)?;
    Ok((d1, d2, calculated_time_to_expiry))
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
fn calculate_call_option_price(
    option: &Options,
    d1: Decimal,
    d2: Decimal,
    t: Decimal,
) -> Result<Decimal, PricingError> {
    let big_n_d1 = big_n(d1)?;
    let big_n_d2 = big_n(d2)?;

    // e^(−qT) * S * N(d1) − e^(−rT) * K * N(d2)
    let s_discounted =
        option.underlying_price.to_dec() * (-option.dividend_yield.to_dec() * t).exp();
    let k_discounted = (-option.risk_free_rate * t).exp() * option.strike_price.to_dec();

    let result = s_discounted * big_n_d1 - k_discounted * big_n_d2;
    trace!(
        "Call Option Price: {} - {} * {} * {} = {}",
        option.underlying_price,
        option.strike_price,
        (-option.risk_free_rate * t).exp(),
        big_n_d2,
        result
    );
    Ok(result)
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
fn calculate_put_option_price(
    option: &Options,
    d1: Decimal,
    d2: Decimal,
    t: Decimal,
) -> Result<Decimal, PricingError> {
    // N(–d1) and N(–d2)
    let big_n_neg_d1 = big_n(-d1)?;
    let big_n_neg_d2 = big_n(-d2)?;

    // Discount factors
    let s_discounted =
        option.underlying_price.to_dec() * (-option.dividend_yield.to_dec() * t).exp(); // e^(−qT)·S
    let k_discounted = option.strike_price.to_dec() * (-option.risk_free_rate * t).exp(); // e^(−rT)·K

    // P = K e^(−rT) N(−d2) − S e^(−qT) N(−d1)
    let result = k_discounted * big_n_neg_d2 - s_discounted * big_n_neg_d1;

    Ok(result)
}

/// A trait for financial instruments that can be priced using the Black-Scholes option pricing model.
///
/// This trait defines the interface for financial instruments that can have their price
/// calculated using the Black-Scholes formula. Implementors must provide access to their
/// underlying option data through the `get_option` method, which allows the default
/// implementation of `calculate_price_black_scholes` to perform the calculation.
///
/// # Examples
///
/// ```
/// use std::error::Error;
/// use optionstratlib::Options;
/// use optionstratlib::prelude::PricingError;
///
/// use optionstratlib::pricing::BlackScholes;
///
/// struct MyOption {
///     option: Options
/// }
///
/// impl BlackScholes for MyOption {
///     fn get_option(&self) -> Result<&Options, PricingError> {
///         Ok(&self.option)
///     }
/// }
///
/// ```
pub trait BlackScholes {
    /// Retrieves a reference to the options data required for Black-Scholes calculations.
    ///
    /// This method must be implemented by types that implement this trait.
    /// It provides access to the option parameters needed for pricing calculations.
    ///
    /// # Returns
    ///
    /// * `Result<&Options, PricingError>` - A reference to the Options struct on success,
    ///   or an error if the option data cannot be retrieved.
    fn get_option(&self) -> Result<&Options, PricingError>;

    /// Calculates the price of the option using the Black-Scholes model.
    ///
    /// This default implementation retrieves the option data via `get_option()`
    /// and then passes it to the `black_scholes` function to perform the calculation.
    ///
    /// # Returns
    ///
    /// * `Result<Decimal, PricingError>` - The calculated option price as a Decimal
    ///   on success, or an error if the calculation fails.
    fn calculate_price_black_scholes(&self) -> Result<Decimal, PricingError> {
        let option = self.get_option()?;
        black_scholes(option)
    }
}

#[cfg(test)]
mod tests_black_scholes {
    use super::*;
    use crate::constants::DAYS_IN_A_YEAR;
    use crate::greeks::{d1, d2};
    use crate::model::types::{OptionStyle, OptionType, Side};
    use crate::{ExpirationDate, Options, assert_decimal_eq};
    use positive::{Positive, assert_pos_relative_eq, pos_or_panic};
    use rust_decimal_macros::dec;

    fn mock_options_call() -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: pos_or_panic!(2476.6),
            strike_price: pos_or_panic!(2485.0),
            implied_volatility: pos_or_panic!(0.22),
            risk_free_rate: dec!(0.006),
            expiration_date: ExpirationDate::Days(pos_or_panic!(3.0)),
            option_style: OptionStyle::Call,
            underlying_symbol: "GOLD".to_string(),
            quantity: Positive::ONE,
            dividend_yield: Positive::ZERO,
            exotic_params: None,
        }
    }

    fn mock_options_simplest_call() -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: Positive::HUNDRED,
            strike_price: Positive::HUNDRED,
            implied_volatility: pos_or_panic!(0.01),
            risk_free_rate: Decimal::ZERO,
            expiration_date: ExpirationDate::Days(DAYS_IN_A_YEAR),
            option_style: OptionStyle::Call,
            underlying_symbol: "GOLD".to_string(),
            quantity: Positive::ONE,
            dividend_yield: Positive::ZERO,

            exotic_params: None,
        }
    }

    fn mock_options_put() -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: Positive::HUNDRED,
            strike_price: Positive::HUNDRED,
            implied_volatility: pos_or_panic!(0.2),
            risk_free_rate: dec!(0.05),
            expiration_date: ExpirationDate::Days(DAYS_IN_A_YEAR), // 1 year from now
            option_style: OptionStyle::Put,
            underlying_symbol: "".to_string(),
            quantity: Positive::ZERO,
            dividend_yield: Positive::ZERO,
            exotic_params: None,
        }
    }

    #[test]
    fn test_black_scholes_simplest_call() {
        let mut option = mock_options_simplest_call();
        assert_pos_relative_eq!(
            option.expiration_date.get_years().unwrap(),
            Positive::ONE,
            pos_or_panic!(0.00001)
        );
        let d1 = d1(
            option.underlying_price,
            option.strike_price,
            option.risk_free_rate,
            option.expiration_date.get_years().unwrap(),
            option.implied_volatility,
        )
        .unwrap();
        assert_decimal_eq!(d1, dec!(0.005), dec!(0.00001));
        let d2 = d2(
            option.underlying_price,
            option.strike_price,
            option.risk_free_rate,
            option.expiration_date.get_years().unwrap(),
            option.implied_volatility,
        )
        .unwrap();

        assert_decimal_eq!(d2, dec!(-0.005), dec!(0.00001));

        let big_n_d1 = big_n(d1).unwrap();
        assert_decimal_eq!(big_n_d1, dec!(0.501994), dec!(0.00001));

        let big_n_d2 = big_n(d2).unwrap();
        assert_decimal_eq!(big_n_d2, dec!(0.498005), dec!(0.00001));

        let option_value = option.strike_price * big_n_d1 - option.underlying_price * big_n_d2;
        assert_pos_relative_eq!(
            option_value,
            pos_or_panic!(0.3989406),
            pos_or_panic!(0.00001)
        );

        let price = black_scholes(&option).unwrap();
        assert_decimal_eq!(price, dec!(0.39894), dec!(0.00001));
        assert_decimal_eq!(price, option_value.to_dec(), dec!(0.00001));

        option.implied_volatility = pos_or_panic!(0.2);
        let price = black_scholes(&option).unwrap();
        assert_decimal_eq!(price, dec!(7.965), dec!(0.001));

        option.strike_price = pos_or_panic!(50.0);
        let price = black_scholes(&option).unwrap();
        assert_decimal_eq!(price, dec!(50.0), dec!(0.001));

        option.strike_price = Positive::HUNDRED;
        let price = black_scholes(&option).unwrap();
        assert_decimal_eq!(price, dec!(7.96556), dec!(0.001));
    }

    // #[test]
    // fn test_black_scholes_simplest_call_2() {
    //     let option = Options {
    //         option_type: OptionType::European,
    //         side: Side::Long,
    //         underlying_price: Positive::HUNDRED,
    //         strike_price: pos_or_panic!(50.0),
    //         implied_volatility: pos_or_panic!(0.01),
    //         risk_free_rate: Decimal::ZERO,
    //         expiration_date: ExpirationDate::Days(DAYS_IN_A_YEAR),
    //         option_style: OptionStyle::Call,
    //         underlying_symbol: "GOLD".to_string(),
    //         quantity: Positive::ONE,
    //         dividend_yield: Positive::ZERO,
    //
    //         exotic_params: None,
    //     };
    //     // assert_relative_eq!(option.expiration_date.get_years(), 1.0, epsilon = 0.00001);
    //     let d1 = d1(
    //         option.underlying_price,
    //         option.strike_price,
    //         option.risk_free_rate,
    //         option.expiration_date.get_years(),
    //         option.implied_volatility,
    //     );
    //     assert_relative_eq!(d1, 69.31971, epsilon = 0.00001);
    //     let d2 = d2(
    //         option.underlying_price,
    //         option.strike_price,
    //         option.risk_free_rate,
    //         option.expiration_date.get_years(),
    //         option.implied_volatility,
    //     );
    //     assert_relative_eq!(d2, 69.3097180, epsilon = 0.00001);
    //     let big_n_d1 = big_n(d1);
    //     assert_relative_eq!(big_n_d1, 1.0, epsilon = 0.00001);
    //     let big_n_d2 = big_n(d2);
    //     assert_relative_eq!(big_n_d2, 1.0, epsilon = 0.00001);
    //
    //     let option_value = option.underlying_price * big_n_d1 - option.strike_price * big_n_d2;
    //     assert_relative_eq!(option_value, 50.0, epsilon = 0.00001);
    //
    //     let volatility = 0.2;
    //     let value_at_20 = volatility * option.strike_price * option_value;
    //     assert_relative_eq!(value_at_20, 500.0, epsilon = 0.00001);
    //
    //     let price = black_scholes(&option.clone());
    //
    //     assert_relative_eq!(price, 50.0, epsilon = 0.001);
    //     assert_relative_eq!(price, option_value, epsilon = 0.001);
    // }
    //
    // #[test]
    // fn test_black_scholes_simplest_call_3() {
    //     let option = Options {
    //         option_type: OptionType::European,
    //         side: Side::Long,
    //         underlying_price: pos_or_panic!(60.0),
    //         strike_price: pos_or_panic!(65.0),
    //         implied_volatility: pos_or_panic!(0.3),
    //         risk_free_rate: dec!(0.08),
    //         expiration_date: ExpirationDate::Days(pos_or_panic!(365.0 / 4.0)),
    //         option_style: OptionStyle::Call,
    //         underlying_symbol: "GOLD".to_string(),
    //         quantity: Positive::ONE,
    //         dividend_yield: Positive::ZERO,
    //
    //         exotic_params: None,
    //     };
    //     assert_relative_eq!(option.expiration_date.get_years(), 0.25, epsilon = 0.00001);
    //     let d1 = d1(
    //         option.underlying_price,
    //         option.strike_price,
    //         option.risk_free_rate,
    //         option.expiration_date.get_years(),
    //         option.implied_volatility,
    //     );
    //     assert_relative_eq!(d1, -0.325284, epsilon = 0.00001);
    //     let d2 = d2(
    //         option.underlying_price,
    //         option.strike_price,
    //         option.risk_free_rate,
    //         option.expiration_date.get_years(),
    //         option.implied_volatility,
    //     );
    //     assert_relative_eq!(d2, -0.475284, epsilon = 0.00001);
    //     let big_n_d1 = big_n(d1);
    //     assert_relative_eq!(big_n_d1, 0.3724827, epsilon = 0.00001);
    //     let big_n_d2 = big_n(d2);
    //     assert_relative_eq!(big_n_d2, 0.3172920, epsilon = 0.00001);
    //
    //     let option_value = option.underlying_price * big_n_d1
    //         - option.strike_price
    //             * big_n_d2
    //             * (-option.risk_free_rate * option.expiration_date.get_years()).exp();
    //     assert_relative_eq!(option_value, 2.133368, epsilon = 0.00001);
    //
    //     let price = black_scholes(&option.clone());
    //
    //     assert_relative_eq!(price, 2.133368, epsilon = 0.001);
    //     assert_relative_eq!(price, option_value, epsilon = 0.001);
    //     assert_relative_eq!(
    //         option.calculate_price_black_scholes(),
    //         option_value,
    //         epsilon = 0.0001
    //     );
    // }

    #[test]
    fn test_black_scholes_call_with_explicit_time_to_expiry() {
        let option = mock_options_call();
        let price = black_scholes(&option).unwrap();
        assert_decimal_eq!(price, dec!(15.8756), dec!(0.001));
    }

    #[test]
    fn test_black_scholes_put_with_explicit_time_to_expiry() {
        let option = mock_options_put();
        let price = black_scholes(&option).unwrap();
        assert_decimal_eq!(price, dec!(5.573526), dec!(0.001));
    }

    #[test]
    fn test_black_scholes_call_without_explicit_time_to_expiry() {
        let option = mock_options_call();
        let price = black_scholes(&option).unwrap();
        assert_decimal_eq!(price, dec!(15.875638), dec!(0.001));
    }

    #[test]
    fn test_black_scholes_put_without_explicit_time_to_expiry() {
        let option = mock_options_put();
        let price = black_scholes(&option).unwrap();
        assert_decimal_eq!(price, dec!(5.5735260), dec!(0.001));
    }
}

#[cfg(test)]
mod tests_black_scholes_trait {
    use super::*;
    use crate::assert_decimal_eq;
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option;
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;

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
        fn get_option(&self) -> Result<&Options, PricingError> {
            Ok(&self.option)
        }
    }

    #[test]
    fn test_at_the_money_call() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,  // underlying price
            Positive::ONE,      // quantity
            Positive::HUNDRED,  // strike price
            pos_or_panic!(0.2), // volatility
        );
        let mock = MockOption::new(option);
        let price = mock.calculate_price_black_scholes().unwrap();
        assert_decimal_eq!(price, dec!(2.4492483), dec!(1e-5));
    }

    #[test]
    fn test_in_the_money_call() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,   // underlying price
            Positive::ONE,       // quantity
            pos_or_panic!(90.0), // strike price
            pos_or_panic!(0.2),  // volatility
        );
        let mock = MockOption::new(option);
        let price = mock.calculate_price_black_scholes().unwrap();
        assert_decimal_eq!(price, dec!(10.3477145), dec!(1e-5));
    }

    #[test]
    fn test_out_of_the_money_call() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,    // underlying price
            Positive::ONE,        // quantity
            pos_or_panic!(110.0), // strike price
            pos_or_panic!(0.2),   // volatility
        );
        let mock = MockOption::new(option);
        let price = mock.calculate_price_black_scholes().unwrap();
        assert_decimal_eq!(price, dec!(0.1377787), dec!(1e-5));
    }

    #[test]
    fn test_at_the_money_put() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            Positive::HUNDRED,  // underlying price
            Positive::ONE,      // quantity
            Positive::HUNDRED,  // strike price
            pos_or_panic!(0.2), // volatility
        );
        let mock = MockOption::new(option);
        let price = mock.calculate_price_black_scholes().unwrap();
        assert_decimal_eq!(price, dec!(2.1212907), dec!(1e-5));
    }

    #[test]
    fn test_high_volatility() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,  // underlying price
            Positive::ONE,      // quantity
            Positive::HUNDRED,  // strike price
            pos_or_panic!(0.5), // high volatility
        );
        let mock = MockOption::new(option);
        let price = mock.calculate_price_black_scholes().unwrap();
        assert_decimal_eq!(price, dec!(5.8651791), dec!(1e-5));
    }

    #[test]
    fn test_zero_volatility() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED, // underlying price
            Positive::ONE,     // quantity
            Positive::HUNDRED, // strike price
            Positive::ZERO,    // zero volatility
        );
        let mock = MockOption::new(option);
        let price = mock.calculate_price_black_scholes();
        assert!(price.is_err());
    }

    #[test]
    fn test_short_call() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            Positive::HUNDRED,  // underlying price
            Positive::ONE,      // quantity
            Positive::HUNDRED,  // strike price
            pos_or_panic!(0.2), // volatility
        );
        let mock = MockOption::new(option);
        let price = mock.calculate_price_black_scholes().unwrap();
        assert_decimal_eq!(price, dec!(-2.4492483), dec!(1e-5));
    }

    #[test]
    fn test_short_put() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            Positive::HUNDRED,  // underlying price
            Positive::ONE,      // quantity
            Positive::HUNDRED,  // strike price
            pos_or_panic!(0.2), // volatility
        );
        let mock = MockOption::new(option);
        let price = mock.calculate_price_black_scholes().unwrap();
        assert_decimal_eq!(price, dec!(-2.1212907), dec!(1e-5));
    }

    #[test]
    fn test_with_different_quantity() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,   // underlying price
            pos_or_panic!(10.0), // quantity
            Positive::HUNDRED,   // strike price
            pos_or_panic!(0.2),  // volatility
        );
        let mock = MockOption::new(option);
        let price = mock.calculate_price_black_scholes().unwrap();
        assert_decimal_eq!(price, dec!(2.4492483), dec!(1e-5));
    }
}

#[cfg(test)]
mod tests_black_scholes_trait_bis {
    use super::*;
    use crate::assert_decimal_eq;
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option;
    use num_traits::FromPrimitive;
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    struct MockOption {
        option: Options,
    }

    impl MockOption {
        fn new(option: Options) -> Self {
            MockOption { option }
        }
    }

    impl BlackScholes for MockOption {
        fn get_option(&self) -> Result<&Options, PricingError> {
            Ok(&self.option)
        }
    }

    #[test]
    fn test_call_put_parity() {
        let call_option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
        );

        let put_option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
        );

        let call_mock = MockOption::new(call_option);
        let put_mock = MockOption::new(put_option);

        let call_price = call_mock.calculate_price_black_scholes().unwrap();
        let put_price = put_mock.calculate_price_black_scholes().unwrap();

        let r: f64 = 0.05;
        let t: f64 = 30.0 / 365.0;
        let s: f64 = 100.0;
        let k: f64 = 100.0;

        let parity_value = call_price - put_price;
        let theoretical_value = Decimal::from_f64(s - k * f64::exp(-r * t)).unwrap();
        assert_decimal_eq!(parity_value, theoretical_value, dec!(1e-1));
    }

    #[test]
    fn test_call_put_parity_short() {
        let call_option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
        );

        let put_option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
        );

        let call_mock = MockOption::new(call_option);
        let put_mock = MockOption::new(put_option);

        let call_price = call_mock.calculate_price_black_scholes().unwrap();
        let put_price = put_mock.calculate_price_black_scholes().unwrap();

        let r: f64 = 0.05;
        let t: f64 = 30.0 / 365.0;
        let s: f64 = 100.0;
        let k: f64 = 100.0;

        let parity_value = call_price - put_price;
        let theoretical_value = Decimal::from_f64(s - k * f64::exp(-r * t)).unwrap();
        assert_decimal_eq!(parity_value, -theoretical_value, dec!(1e-1));
    }

    #[test]
    fn test_monotonicity_with_strike() {
        let call1 = MockOption::new(create_sample_option(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            pos_or_panic!(90.0),
            pos_or_panic!(0.2),
        ));

        let call2 = MockOption::new(create_sample_option(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
        ));

        let call3 = MockOption::new(create_sample_option(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            pos_or_panic!(110.0),
            pos_or_panic!(0.2),
        ));

        let price1 = call1.calculate_price_black_scholes().unwrap();
        let price2 = call2.calculate_price_black_scholes().unwrap();
        let price3 = call3.calculate_price_black_scholes().unwrap();

        assert!(price1 > price2);
        assert!(price2 > price3);
    }

    #[test]
    fn test_zero_volatility_call() {
        let option = MockOption::new(create_sample_option(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            pos_or_panic!(95.0),
            Positive::ZERO,
        ));

        assert!(option.calculate_price_black_scholes().is_err());
    }

    #[test]
    fn test_deep_itm_call() {
        let option = MockOption::new(create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(150.0),
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
        ));

        let price = option.calculate_price_black_scholes().unwrap();
        let r: f64 = 0.05;
        let t: f64 = 30.0 / 365.0;
        let s: f64 = 150.0;
        let k: f64 = 100.0;

        let intrinsic_value = Decimal::from_f64(s - k * f64::exp(-r * t)).unwrap();
        assert_decimal_eq!(price, intrinsic_value, dec!(0.2));
    }

    #[test]
    fn test_deep_otm_call() {
        let option = MockOption::new(create_sample_option(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            pos_or_panic!(200.0),
            pos_or_panic!(0.2),
        ));

        let price = option.calculate_price_black_scholes().unwrap();
        assert!(price < dec!(0.1));
    }

    #[test]
    fn test_monotonicity_with_volatility() {
        let call1 = MockOption::new(create_sample_option(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.1),
        ));

        let call2 = MockOption::new(create_sample_option(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
        ));

        let call3 = MockOption::new(create_sample_option(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.3),
        ));

        let price1 = call1.calculate_price_black_scholes().unwrap();
        let price2 = call2.calculate_price_black_scholes().unwrap();
        let price3 = call3.calculate_price_black_scholes().unwrap();

        assert!(price1 < price2);
        assert!(price2 < price3);
    }
}

#[cfg(test)]
mod tests_black_scholes_bis {
    use super::*;
    use crate::model::types::{OptionStyle, Side};
    use crate::{ExpirationDate, assert_decimal_eq};
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    fn create_base_option(side: Side, style: OptionStyle) -> Options {
        Options::new(
            OptionType::European,
            side,
            "TEST".to_string(),
            Positive::HUNDRED, // strike
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2), // volatility
            Positive::ONE,      // quantity
            Positive::HUNDRED,  // underlying price
            dec!(0.05),         // risk-free rate
            style,
            Positive::ZERO, // dividend yield
            None,
        )
    }

    #[test]
    fn test_call_option_at_the_money() {
        let option = create_base_option(Side::Long, OptionStyle::Call);
        let price = black_scholes(&option).unwrap();
        assert_decimal_eq!(price, dec!(2.49), dec!(0.01));
    }

    #[test]
    fn test_put_option_at_the_money() {
        let option = create_base_option(Side::Long, OptionStyle::Put);
        let price = black_scholes(&option).unwrap();
        assert_decimal_eq!(price, dec!(2.08), dec!(0.01));
    }

    #[test]
    fn test_call_option_in_the_money() {
        let mut option = create_base_option(Side::Long, OptionStyle::Call);
        option.strike_price = pos_or_panic!(90.0);
        let price = black_scholes(&option).unwrap();
        assert_decimal_eq!(price, dec!(10.42), dec!(0.01));
    }

    #[test]
    fn test_put_option_in_the_money() {
        let mut option = create_base_option(Side::Long, OptionStyle::Put);
        option.strike_price = pos_or_panic!(110.0);
        let price = black_scholes(&option).unwrap();
        assert_decimal_eq!(price, dec!(9.69), dec!(0.01));
    }

    #[test]
    fn test_call_option_out_of_money() {
        let mut option = create_base_option(Side::Long, OptionStyle::Call);
        option.strike_price = pos_or_panic!(110.0);
        let price = black_scholes(&option).unwrap();
        assert_decimal_eq!(price, dec!(0.14), dec!(0.01));
    }

    #[test]
    fn test_put_option_out_of_money() {
        let mut option = create_base_option(Side::Long, OptionStyle::Put);
        option.strike_price = pos_or_panic!(90.0);
        let price = black_scholes(&option).unwrap();
        assert_decimal_eq!(price, dec!(0.05), dec!(0.01));
    }

    #[test]
    fn test_short_call_option() {
        let option = create_base_option(Side::Short, OptionStyle::Call);
        let price = black_scholes(&option).unwrap();
        assert_decimal_eq!(price, dec!(-2.49), dec!(0.01));
    }

    #[test]
    fn test_short_put_option() {
        let option = create_base_option(Side::Short, OptionStyle::Put);
        let price = black_scholes(&option).unwrap();
        assert_decimal_eq!(price, dec!(-2.08), dec!(0.01));
    }

    #[test]
    fn test_zero_volatility() {
        let mut option = create_base_option(Side::Long, OptionStyle::Call);
        option.implied_volatility = Positive::ZERO;
        assert!(black_scholes(&option).is_err());
    }

    #[test]
    fn test_high_volatility() {
        let mut option = create_base_option(Side::Long, OptionStyle::Call);
        option.implied_volatility = pos_or_panic!(0.5);
        let price = black_scholes(&option).unwrap();
        assert_decimal_eq!(price, dec!(5.90), dec!(0.01));
    }

    #[test]
    fn test_put_call_parity() {
        let call = create_base_option(Side::Long, OptionStyle::Call);
        let put = create_base_option(Side::Long, OptionStyle::Put);

        let call_price = black_scholes(&call).unwrap();
        let put_price = black_scholes(&put).unwrap();

        let risk_free_discount =
            (-call.risk_free_rate * call.expiration_date.get_years().unwrap().to_dec()).exp();

        // Put-Call Parity formula: C - P = S - K * e^(-rT)
        let left_side = call_price - put_price;
        let right_side =
            call.underlying_price.to_dec() - (call.strike_price.to_dec() * risk_free_discount);

        assert_decimal_eq!(left_side, right_side, dec!(0.01));
    }

    #[test]
    fn test_different_maturities() {
        let mut short_term = create_base_option(Side::Long, OptionStyle::Call);
        short_term.expiration_date = ExpirationDate::Days(pos_or_panic!(7.0));

        let mut long_term = create_base_option(Side::Long, OptionStyle::Call);
        long_term.expiration_date = ExpirationDate::Days(pos_or_panic!(365.0));

        let short_term_price = black_scholes(&short_term).unwrap();
        let long_term_price = black_scholes(&long_term).unwrap();

        assert!(long_term_price > short_term_price);
    }

    #[test]
    fn test_different_quantities() {
        let option_qty_1 = create_base_option(Side::Long, OptionStyle::Call);
        let mut option_qty_10 = create_base_option(Side::Long, OptionStyle::Call);
        option_qty_10.quantity = pos_or_panic!(10.0);

        let price_qty_1 = black_scholes(&option_qty_1).unwrap();
        let price_qty_10 = black_scholes(&option_qty_10).unwrap();

        assert_decimal_eq!(price_qty_1, price_qty_10, dec!(0.01));
    }

    #[test]
    fn test_with_dividend_yield() {
        let mut option = create_base_option(Side::Long, OptionStyle::Call);
        option.dividend_yield = Positive::ZERO;
        let price = black_scholes(&option).unwrap();
        assert_decimal_eq!(price, dec!(2.49), dec!(0.01));
    }
}
