/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/8/24
******************************************************************************/
use crate::error::ChainError;
use crate::model::Position;
use crate::model::types::{OptionStyle, OptionType, Side};
use crate::{ExpirationDate, Options};
use chrono::{NaiveDateTime, TimeZone, Utc};
use positive::{Positive, pos_or_panic};
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use std::ops::Mul;

/// Converts a vector of `Positive` values to a vector of `f64` values.
///
/// This utility function transforms a collection of `Positive` type values
/// to standard floating-point values by applying the `to_f64()` method to each element.
/// The function consumes the input vector and returns a new vector containing the converted values.
///
/// # Parameters
///
/// * `vec` - A vector of `Positive` values to be converted.
///
/// # Returns
///
/// A vector of `f64` values corresponding to the input `Positive` values.
///
pub fn positive_f64_to_f64(vec: Vec<Positive>) -> Vec<f64> {
    vec.into_iter().map(|pos_f64| pos_f64.to_f64()).collect()
}

/// Creates a sample option contract with predefined parameters for testing or demonstration purposes.
///
/// This utility function simplifies the creation of option contracts by providing a standard
/// configuration with reasonable defaults. It creates a European-style option with a 30-day
/// expiration and a fixed risk-free rate of 5%.
///
/// # Parameters
///
/// * `option_style` - Specifies whether the option is a Call or Put.
/// * `side` - Determines if the position is Long or Short.
/// * `underlying_price` - The current market price of the underlying asset.
/// * `quantity` - The number of contracts in the position.
/// * `strike_price` - The price at which the option holder can exercise the option.
/// * `volatility` - The implied volatility used for pricing the option.
///
/// # Returns
///
/// An `Options` struct configured with the specified parameters and sensible defaults.
///
/// # Examples
///
/// ```rust
/// use optionstratlib::{OptionStyle, Side};
/// use positive::pos_or_panic;
/// use optionstratlib::model::utils::create_sample_option;
/// let option = create_sample_option(
///     OptionStyle::Call,
///     Side::Long,
///     pos_or_panic!(150.0),  // underlying price
///     pos_or_panic!(10.0),   // quantity
///     pos_or_panic!(155.0),  // strike price
///     pos_or_panic!(0.25),   // volatility (25%)
/// );
/// ```
#[allow(dead_code)]
pub fn create_sample_option(
    option_style: OptionStyle,
    side: Side,
    underlying_price: Positive,
    quantity: Positive,
    strike_price: Positive,
    volatility: Positive,
) -> Options {
    Options::new(
        OptionType::European,
        side,
        "AAPL".to_string(),
        strike_price,
        ExpirationDate::Days(pos_or_panic!(30.0)),
        volatility,
        quantity,
        underlying_price,
        dec!(0.05),
        option_style,
        pos_or_panic!(0.01),
        None,
    )
}

/// Creates a sample position for testing and demonstration purposes.
///
/// This function generates a `Position` instance with predefined values for some fields
/// while allowing customization of key option parameters. It's useful for creating test
/// scenarios, examples, or sample data for option position analysis.
///
/// # Parameters
///
/// * `option_style` - The style of the option (Call or Put)
/// * `side` - Whether the position is Long or Short
/// * `underlying_price` - The current price of the underlying asset
/// * `quantity` - The number of option contracts in the position
/// * `strike_price` - The price at which the option can be exercised
/// * `implied_volatility` - The market's forecast of likely movement in the underlying asset
///
/// # Returns
///
/// A `Position` instance with the specified parameters and these default values:
/// * European-style option
/// * "AAPL" as the underlying symbol
/// * 30-day expiration
/// * 5% risk-free rate
/// * 1% dividend yield
/// * Premium of $5.00
/// * Open and close fees of $0.50 each
/// * Current date and time
///
/// # Example
///
/// ```rust
/// use optionstratlib::model::utils::create_sample_position;
/// use optionstratlib::{OptionStyle, Side};
/// use positive::{pos_or_panic, Positive};
/// let sample_call = create_sample_position(
///     OptionStyle::Call,
///     Side::Long,
///     pos_or_panic!(150.0),  // underlying price
///     Positive::ONE,    // quantity
///     pos_or_panic!(155.0),  // strike price
///     pos_or_panic!(0.25)    // implied volatility
/// );
/// ```
#[allow(dead_code)]
pub fn create_sample_position(
    option_style: OptionStyle,
    side: Side,
    underlying_price: Positive,
    quantity: Positive,
    strike_price: Positive,
    implied_volatility: Positive,
) -> Position {
    Position {
        option: Options {
            option_type: OptionType::European,
            side,
            underlying_symbol: "AAPL".to_string(),
            strike_price,
            expiration_date: ExpirationDate::Days(pos_or_panic!(30.0)),
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate: dec!(0.05),
            option_style,
            dividend_yield: pos_or_panic!(0.01),
            exotic_params: None,
        },
        premium: pos_or_panic!(5.0),
        date: Utc::now(),
        open_fee: pos_or_panic!(0.5),
        close_fee: pos_or_panic!(0.5),
        epic: Some("Epic123".to_string()),
        extra_fields: None,
    }
}

/// Creates a sample Options object with a specific expiration date.
///
/// This utility function simplifies the creation of option contracts for testing
/// or demonstration purposes by providing a specific expiration date using a NaiveDateTime.
/// It creates a European-style option with predefined parameters and a fixed risk-free rate of 5%
/// and dividend yield of 1%.
///
/// # Parameters
///
/// * `option_style` - The style of the option (Call or Put)
/// * `side` - The position side (Long or Short)
/// * `underlying_price` - The current price of the underlying asset
/// * `quantity` - The number of option contracts
/// * `strike_price` - The strike price of the option
/// * `volatility` - The implied volatility for pricing the option
/// * `naive_date` - The expiration date and time in naive format (will be converted to UTC)
///
/// # Returns
///
/// Returns a fully configured Options instance with "AAPL" as the underlying symbol.
pub fn create_sample_option_with_date(
    option_style: OptionStyle,
    side: Side,
    underlying_price: Positive,
    quantity: Positive,
    strike_price: Positive,
    volatility: Positive,
    naive_date: NaiveDateTime,
) -> Options {
    Options::new(
        OptionType::European,
        side,
        "AAPL".to_string(),
        strike_price,
        ExpirationDate::DateTime(Utc.from_utc_datetime(&naive_date)),
        volatility,
        quantity,
        underlying_price,
        dec!(0.05),
        option_style,
        pos_or_panic!(0.01),
        None,
    )
}

/// Creates a sample Options object with a specific expiration date in days.
///
/// This utility function simplifies the creation of option contracts for testing
/// or demonstration purposes by providing a specific expiration date expressed
/// in days.
/// It creates a European-style option with predefined parameters and a fixed
/// risk-free rate of 5% and dividend yield of 1%.
///
/// # Parameters
///
/// * `option_style` - The style of the option (Call or Put)
/// * `side` - The position side (Long or Short)
/// * `underlying_price` - The current price of the underlying asset
/// * `quantity` - The number of option contracts
/// * `strike_price` - The strike price of the option
/// * `volatility` - The implied volatility for pricing the option
/// * `expiration_days` - The expiration days
///
/// # Returns
///
/// Returns a fully configured Options instance with "AAPL" as the underlying symbol.
pub fn create_sample_option_with_days(
    option_style: OptionStyle,
    side: Side,
    underlying_price: Positive,
    quantity: Positive,
    strike_price: Positive,
    volatility: Positive,
    expiration_days: Positive,
) -> Options {
    Options::new(
        OptionType::European,
        side,
        "AAPL".to_string(),
        strike_price,
        ExpirationDate::Days(expiration_days),
        volatility,
        quantity,
        underlying_price,
        dec!(0.05),
        option_style,
        pos_or_panic!(0.01),
        None,
    )
}

/// Creates a simplified sample option contract for testing or demonstration purposes.
///
/// This function generates an Options instance with pre-defined values, requiring only
/// the specification of the option style (Call or Put) and market position (Long or Short).
/// It uses Apple Inc. (AAPL) as the underlying security with standard parameters suitable
/// for basic examples or testing scenarios.
///
/// # Parameters
///
/// * `option_style` - Specifies whether the option is a Call or Put
/// * `side` - Indicates whether the position is Long or Short
///
/// # Returns
///
/// Returns an `Options` instance with the following predefined values:
/// - European-style option
/// - AAPL as the underlying symbol
/// - Strike price of $100.0
/// - 30 days until expiration
/// - 20% implied volatility (0.2)
/// - Quantity of 1.0 contracts
/// - Underlying price of $100.0
/// - 5% risk-free rate
/// - 1% dividend yield
/// - No exotic parameters
///
/// # Examples
///
/// ```
/// use optionstratlib::model::utils::create_sample_option_simplest;
/// use optionstratlib::{OptionStyle, Side};
/// let long_call = create_sample_option_simplest(OptionStyle::Call, Side::Long);
/// let short_put = create_sample_option_simplest(OptionStyle::Put, Side::Short);
/// ```
pub fn create_sample_option_simplest(option_style: OptionStyle, side: Side) -> Options {
    Options::new(
        OptionType::European,
        side,
        "AAPL".to_string(),
        Positive::HUNDRED,
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        option_style,
        pos_or_panic!(0.01),
        None,
    )
}

/// Creates a sample option with specified parameters and default values.
///
/// This function provides a convenient way to create an `Options` instance with common default values
/// while allowing customization of the most important parameters: side, option style, and strike price.
/// All other parameters are set to reasonable defaults for testing or demonstration purposes.
///
/// # Parameters
/// * `side` - The position side (Long or Short) for the option.
/// * `option_style` - The style of option (Call or Put).
/// * `strike` - The strike price of the option as a `Positive` value.
///
/// # Returns
/// An `Options` instance representing a European option on AAPL stock with:
/// * 30 days until expiration
/// * 20% implied volatility
/// * Quantity of 1.0
/// * Underlying price of $100.0
/// * 5% risk-free rate
/// * 1% dividend yield
/// * No exotic parameters
///
/// # Examples
/// ```
/// use optionstratlib::model::utils::create_sample_option_simplest_strike;
/// use optionstratlib::{ OptionStyle, Side};
/// use positive::pos_or_panic;
/// let long_call = create_sample_option_simplest_strike(
///     Side::Long,
///     OptionStyle::Call,
///     pos_or_panic!(105.0)
/// );
/// ```
pub fn create_sample_option_simplest_strike(
    side: Side,
    option_style: OptionStyle,
    strike: Positive,
) -> Options {
    Options::new(
        OptionType::European,
        side,
        "AAPL".to_string(),
        strike,
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        option_style,
        pos_or_panic!(0.01),
        None,
    )
}

/// Computes the mean and standard deviation of a vector containing `Positive` values.
///
/// # Arguments
///
/// * `vec` - A `Vec<Positive>` containing the data for which the mean and standard deviation
///   are to be calculated.
///
/// # Returns
///
/// A tuple containing:
/// - `Positive` - The mean of the provided vector.
/// - `Positive` - The standard deviation of the provided vector.
///
/// # Example
///
/// ```rust
/// use positive::Positive;
/// use optionstratlib::model::utils::mean_and_std;
///
/// let data = vec![Positive::new(2.0).unwrap(), Positive::new(4.0).unwrap(), Positive::new(4.0).unwrap(), Positive::new(4.0).unwrap(), Positive::new(5.0).unwrap(), Positive::new(5.0).unwrap(), Positive::new(7.0).unwrap(), Positive::new(9.0).unwrap()];
/// let (mean, std) = mean_and_std(data);
///
/// assert_eq!(mean.to_f64(), 5.0);
/// assert_eq!(std.to_f64(), 4.0_f64.sqrt());
/// ```
///
/// # Details
///
/// - The mean is computed by summing the `Positive` values and dividing by the count of elements.
/// - The standard deviation is derived from the variance, which is the average of the squared differences
///   from the mean. The variance is then converted into standard deviation by taking its square root.
/// - This function assumes the vector is non-empty and filled with valid `Positive` values.
///
/// Note: The `Positive` type and associated operations are defined in the `crate::model::types` module.
pub fn mean_and_std(vec: Vec<Positive>) -> (Positive, Positive) {
    let mean = vec.iter().sum::<Positive>() / vec.len() as f64;
    let variance = vec
        .iter()
        .map(|x| pos_or_panic!((x.to_f64() - mean.to_f64()).powi(2)))
        .sum::<Positive>()
        / vec.len() as f64;
    let std = variance.to_f64().sqrt();
    (mean, pos_or_panic!(std))
}

/// Trait for rounding operations on numeric types, specifically for financial calculations.
///
/// This trait provides methods to round a number to the nearest integer and to a specified
/// number of decimal places, ensuring precision and accuracy in financial computations.
///
pub trait ToRound {
    /// Rounds the number to the nearest integer.
    ///
    /// This method rounds the number to the nearest whole number, removing any fractional part.
    fn round(&self) -> Decimal;

    /// Rounds the number to a specified number of decimal places.
    ///
    /// This method rounds the number to the specified number of digits after the decimal point,
    /// providing control over the precision of the rounded value.
    ///
    /// # Arguments
    ///
    /// * `decimal_places` - The number of decimal places to round to.
    fn round_to(&self, decimal_places: u32) -> Decimal;
}

/// Calculates the optimal price range for an option based on its underlying price,
/// strike price, implied volatility, and expiration date.
///
/// # Parameters
/// - `underlying_price`: The price of the underlying asset, represented as a `Positive`.
/// - `strike_price`: The strike price of the option, represented as a `Positive`.
/// - `implied_volatility`: The market's implied volatility for the option, represented as a `Positive`.
/// - `expiration_date`: The expiration date of the option, passed as an `ExpirationDate`.
///
/// # Returns
/// A `Result` containing a tuple of two `Positive` values:
/// - The `min_price` represents the lower bound of the price range.
/// - The `max_price` represents the upper bound of the price range.
///
/// On success, the returned tuple includes both values rounded to a "nice" step value
/// for better usability. On failure, an error boxed in `ChainError` is returned.
///
/// # Calculations
/// 1. Determines the number of years to expiration by calculating days to expiry
///    and converting it into a fractional year.
/// 2. Determines the `volatility_factor` which adjusts for time-decay and a
///    confidence interval (set to 4.0 in this implementation).
/// 3. Calculates the lower and upper bounds of the price range based on the
///    `underlying_price` and the `volatility_factor`.
/// 4. Computes an adjusted range (`min_price` and `max_price`) by scaling
///    the `strike_price` by 70% and 130%, ensuring bounds are within realistic margins.
/// 5. Divides the adjusted range into increments (`step`) for easier rounding before
///    smoothing both bounds to user-friendly values.
///
/// # Errors
/// The function will return an error if:
/// - The extraction of `days_to_expiry` from the `expiration_date` fails.
/// - `years_to_expiry.sqrt()` returns a `None` (e.g. if `years_to_expiry` is negative, which it shouldn't be).
///
/// # Note
/// The constants such as the confidence interval (`4.0`) and scaling factors
/// for the `strike_price` (`0.7` and `1.3`) might be subject to change based
/// on different financial models or strategies.
pub fn calculate_optimal_price_range(
    underlying_price: Positive,
    strike_price: Positive,
    implied_volatility: Positive,
    expiration_date: ExpirationDate,
) -> Result<(Positive, Positive), ChainError> {
    let days_to_expiry = expiration_date.get_days()?;
    let years_to_expiry = Decimal::from(days_to_expiry) / dec!(365.0);
    let years_to_expiry_sqrt = years_to_expiry.sqrt().ok_or_else(|| {
        ChainError::invalid_price_calculation(
            "sqrt() failed to calculate for years_to_expiry value",
        )
    })?;

    let confidence_interval = dec!(4.0);
    let volatility_factor = implied_volatility * years_to_expiry_sqrt * confidence_interval;

    let lower_bound = underlying_price * (dec!(1.0) - volatility_factor);
    let upper_bound = underlying_price * (dec!(1.0) + volatility_factor);

    let min_price = lower_bound.min(strike_price.mul(dec!(0.7)));
    let max_price = upper_bound.max(strike_price.mul(dec!(1.3)));

    let step = (max_price - min_price) / dec!(20.0);
    let rounded_step = step.round_to_nice_number();

    let min_price_rounded = (min_price / rounded_step).floor() * rounded_step;
    let max_price_rounded = (max_price / rounded_step).ceiling() * rounded_step;

    Ok((min_price_rounded, max_price_rounded))
}

/// Generates a price vector for the payoff graph
pub fn generate_price_points(
    min_price: Decimal,
    max_price: Decimal,
    num_points: usize,
) -> Vec<Decimal> {
    let step = (max_price - min_price) / Decimal::from(num_points - 1);
    let mut prices = Vec::with_capacity(num_points);

    for i in 0..num_points {
        let price = min_price + step * Decimal::from(i);
        prices.push(price);
    }

    prices
}

#[cfg(test)]
mod tests_positive_f64_to_f64 {
    use super::*;

    #[test]
    fn test_positive_f64_to_f64_non_empty() {
        let positive_vec = vec![
            Positive::new(10.0).unwrap(),
            Positive::new(20.0).unwrap(),
            Positive::new(30.0).unwrap(),
        ];

        let f64_vec = positive_f64_to_f64(positive_vec);

        assert_eq!(f64_vec, vec![10.0, 20.0, 30.0]);
    }

    #[test]
    fn test_positive_f64_to_f64_single_element() {
        let positive_vec = vec![Positive::new(42.0).unwrap()];

        let f64_vec = positive_f64_to_f64(positive_vec);

        assert_eq!(f64_vec, vec![42.0]);
    }

    #[test]
    #[should_panic]
    fn test_positive_f64_to_f64_invalid_positivef64() {
        Positive::new(-10.0).unwrap();
    }
}

#[cfg(test)]
mod tests_mean_and_std {
    use super::*;

    use approx::assert_relative_eq;

    #[test]
    fn test_basic_mean_and_std() {
        let values = vec![
            Positive::TWO,
            pos_or_panic!(4.0),
            pos_or_panic!(4.0),
            pos_or_panic!(4.0),
            pos_or_panic!(5.0),
            pos_or_panic!(5.0),
            pos_or_panic!(7.0),
            pos_or_panic!(9.0),
        ];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.to_f64(), 5.0, epsilon = 0.0001);
        assert_relative_eq!(std.to_f64(), 2.0, epsilon = 0.0001);
    }

    #[test]
    fn test_identical_values() {
        let values = vec![
            pos_or_panic!(5.0),
            pos_or_panic!(5.0),
            pos_or_panic!(5.0),
            pos_or_panic!(5.0),
        ];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.to_f64(), 5.0, epsilon = 0.0001);
        assert_relative_eq!(std.to_f64(), 0.0, epsilon = 0.0001);
    }

    #[test]
    fn test_single_value() {
        let values = vec![pos_or_panic!(3.0)];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.to_f64(), 3.0, epsilon = 0.0001);
        assert_relative_eq!(std.to_f64(), 0.0, epsilon = 0.0001);
    }

    #[test]
    fn test_small_numbers() {
        let values = vec![pos_or_panic!(0.1), pos_or_panic!(0.2), pos_or_panic!(0.3)];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.to_f64(), 0.2, epsilon = 0.0001);
        assert_relative_eq!(std.to_f64(), 0.08164966, epsilon = 0.0001);
    }

    #[test]
    fn test_large_numbers() {
        let values = vec![
            pos_or_panic!(1000.0),
            pos_or_panic!(2000.0),
            pos_or_panic!(3000.0),
        ];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.to_f64(), 2000.0, epsilon = 0.0001);
        assert_relative_eq!(std.to_f64(), 816.4966, epsilon = 0.1);
    }

    #[test]
    fn test_mixed_range() {
        let values = vec![
            pos_or_panic!(0.5),
            pos_or_panic!(5.0),
            pos_or_panic!(50.0),
            pos_or_panic!(500.0),
        ];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.to_f64(), 138.875, epsilon = 0.001);
        assert_relative_eq!(std.to_f64(), 209.392, epsilon = 0.001);
    }

    #[test]
    #[should_panic]
    fn test_empty_vector() {
        let values: Vec<Positive> = vec![];
        let _ = mean_and_std(values);
    }

    #[test]
    fn test_symmetric_distribution() {
        let values = vec![
            Positive::ONE,
            Positive::TWO,
            pos_or_panic!(3.0),
            pos_or_panic!(4.0),
            pos_or_panic!(5.0),
        ];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.to_f64(), 3.0, epsilon = 0.0001);
        assert_relative_eq!(std.to_f64(), std::f64::consts::SQRT_2, epsilon = 0.0001);
    }

    #[test]
    fn test_result_is_positive() {
        let values = vec![Positive::ONE, Positive::TWO, pos_or_panic!(3.0)];
        let (mean, std) = mean_and_std(values);

        assert!(mean > Positive::ZERO);
        assert!(std > Positive::ZERO);
    }

    #[test]
    fn test_precision() {
        let values = vec![
            pos_or_panic!(1.23456789),
            pos_or_panic!(2.34567890),
            pos_or_panic!(3.45678901),
        ];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.to_f64(), 2.34567860, epsilon = 0.00000001);
        assert_relative_eq!(std.to_f64(), 0.90721797, epsilon = 0.00000001);
    }

    #[test]
    fn test_precision_bis() {
        let values = vec![
            pos_or_panic!(0.123456789),
            pos_or_panic!(0.134567890),
            pos_or_panic!(0.145678901),
        ];
        let (mean, std) = mean_and_std(values);

        assert_relative_eq!(mean.to_f64(), 0.13456786, epsilon = 0.00000001);
        assert_relative_eq!(std.to_f64(), 0.00907213, epsilon = 0.00000001);
    }
}

#[cfg(test)]
mod tests_model_utils {
    use super::*;

    #[test]
    fn test_calculate_optimal_price_range() {
        let underlying_price = pos_or_panic!(100.0);
        let strike_price = pos_or_panic!(90.0);
        let implied_volatility = pos_or_panic!(0.20);
        let expiration_date = ExpirationDate::Days(Positive::TWO);

        let (min_price, max_price) = calculate_optimal_price_range(
            underlying_price,
            strike_price,
            implied_volatility,
            expiration_date,
        )
        .unwrap();

        assert_eq!(min_price, pos_or_panic!(62.0));
        assert_eq!(max_price, pos_or_panic!(118.0));
    }
}
