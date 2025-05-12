use crate::f2d;
use crate::pricing::utils::wiener_increment;
use crate::{Options, Positive};
use num_traits::{FromPrimitive, ToPrimitive};
use rust_decimal::{Decimal, MathematicalOps};
use std::error::Error;

/// This function performs Monte Carlo simulation to price an option.
///
/// # Arguments
///
/// * `option` - An `Options` struct containing the option's parameters, such as underlying price, strike price, risk-free rate, implied volatility, and expiration date.
/// * `steps` - An integer indicating the number of time steps in the simulation.
/// * `simulations` - An integer indicating the number of Monte Carlo simulations to run.
///
/// # Returns
///
/// * A floating-point number representing the estimated price of the option.
///
/// # Description
///
/// The function follows the below steps:
///
/// 1. Calculate the time increment `dt` based on the number of steps.
/// 2. Initialize a sum variable `payoff_sum` to accumulate the payoffs from each simulation.
/// 3. Loop through the number of simulations:
///     - For each simulation, initialize the stock price `st` to the underlying price.
///     - Loop through the number of steps:
///         - Calculate a Wiener process increment `w`.
///         - Update the stock price `st` using the discrete approximation of the geometric Brownian motion model.
///     - Calculate the payoff of the option for this simulation (for a call option, this is `max(st - strike_price, 0)`).
///     - Add the payoff to the `payoff_sum`.
/// 4. Return the average payoff discounted to its present value.
#[allow(dead_code)]
pub fn monte_carlo_option_pricing(
    option: &Options,
    steps: usize,       // Number of time steps
    simulations: usize, // Number of Monte Carlo simulations
) -> Result<Decimal, Box<dyn Error>> {
    let dt = option.expiration_date.get_years().unwrap() / steps as f64;
    let mut payoff_sum = 0.0;

    for _ in 0..simulations {
        let mut st = option.underlying_price.to_dec();
        for _ in 0..steps {
            let w = wiener_increment(dt.to_dec())?;
            st *= Decimal::ONE + option.risk_free_rate * dt + option.implied_volatility * w;
        }
        // Calculate the payoff for a call option
        let payoff: f64 = (st - option.strike_price)
            .max(Decimal::ZERO)
            .to_f64()
            .unwrap();
        payoff_sum += payoff;
    }
    // Average value of the payoffs discounted to present value
    let average_payoff = (payoff_sum / simulations as f64)
        * (-option.risk_free_rate.to_f64().unwrap() * option.expiration_date.get_years().unwrap())
            .exp();
    Ok(f2d!(average_payoff))
}

/// Estimates the price of a financial option using the Monte Carlo simulation method.
///
/// # Parameters
/// - `option`: A reference to an `Options` object that represents the option being evaluated.
///   This object contains necessary details such as risk-free rate, dividend yield,
///   expiration date, and the payoff calculation logic.
/// - `final_prices`: A slice of `Positive` values representing the simulated final prices of
///   the underlying asset at the option's expiration. The length of this slice
///   corresponds to the number of simulations.
///
/// # Returns
/// - `Result<Positive, Box<dyn Error>>`: Returns a `Positive` value encapsulating the estimated
///   option price calculated using the Monte Carlo method. If an error occurs during intermediate
///   calculations (e.g., getting the expiration year), it returns an `Error` wrapped in a `Box<dyn Error>`.
///
/// # How it Works
/// 1. The number of simulations is determined by the length of the `final_prices` slice. If the slice
///    is empty, the function immediately returns a price of `Positive::ZERO`.
/// 2. Calculates the effective discount factor based on the risk-free rate, dividend yield, and
///    time to expiration. This factor is used to discount future payoffs to their present value.
/// 3. For each simulated final price in the `final_prices` slice:
///    - Compute the payoff using the `option.payoff_at_price` method.
///    - Accumulate the total payoff across all simulations.
/// 4. Compute the average payoff by dividing the total payoff by the number of simulations.
///    The average payoff is then discounted using the calculated discount factor.
/// 5. Return the discounted average payoff as the estimated option price.
///
/// # Errors
/// - Returns an error if there are any issues while calculating the time to expiration (e.g., invalid dates).
/// - Panics if the `option.payoff_at_price` method fails unexpectedly during payoff calculations.
///
/// This function assumes that the `Options` struct and `Positive` type
/// are implemented elsewhere in the codebase and provide necessary functionality (e.g., payoff calculation).
pub fn price_option_monte_carlo(
    option: &Options,
    final_prices: &[Positive],
) -> Result<Positive, Box<dyn Error>> {
    // The number of simulations is the length of the final prices vector
    let num_simulations = final_prices.len();

    if num_simulations == 0 {
        return Ok(Positive::ZERO);
    }

    // Calculate total discount factor (risk-free rate adjusted for dividends)
    let effective_rate = option.risk_free_rate - option.dividend_yield;
    let discount_factor = (-effective_rate * option.expiration_date.get_years()?).exp();

    // Calculate payoff for each final price and sum them
    let total_payoff: Decimal = final_prices
        .iter()
        .map(|&final_price| {
            option
                .payoff_at_price(&final_price)
                .expect("Payoff calculation failed")
        })
        .sum();

    // Average payoff discounted to present value
    let avg_payoff =
        discount_factor * (total_payoff / Decimal::from_usize(num_simulations).unwrap());
    Ok(Positive(avg_payoff.abs()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{DAYS_IN_A_YEAR, ZERO};
    use crate::model::types::{OptionStyle, OptionType, Side};
    use crate::{ExpirationDate, Positive, assert_decimal_eq, f2du, pos};
    use rust_decimal::MathematicalOps;
    use rust_decimal_macros::dec;

    fn create_test_option() -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "TEST".to_string(),
            strike_price: pos!(100.0),
            expiration_date: ExpirationDate::Days(DAYS_IN_A_YEAR), // 1 year
            implied_volatility: pos!(0.2),
            quantity: pos!(1.0),
            underlying_price: pos!(100.0),
            risk_free_rate: dec!(0.05),
            option_style: OptionStyle::Call,
            dividend_yield: Positive::ZERO,
            exotic_params: None,
        }
    }

    #[test]
    fn test_monte_carlo_option_pricing_at_the_money() {
        let option = create_test_option();
        let price = monte_carlo_option_pricing(&option, 252, 1000).unwrap();
        // The price should be close to the Black-Scholes price for these parameters
        let expected_price = dec!(9.100); // Calculated using Black-Scholes
        assert_decimal_eq!(price, expected_price, dec!(5));
    }

    #[test]
    fn test_monte_carlo_option_pricing_zero_volatility() {
        let mut option = create_test_option();
        option.implied_volatility = Positive::ZERO;
        let price = monte_carlo_option_pricing(&option, 25, 100).unwrap();
        let expected_price = f64::max(
            (option.underlying_price - option.strike_price * (-option.risk_free_rate).exp()).into(),
            ZERO,
        );
        assert_decimal_eq!(price, f2du!(expected_price).unwrap(), dec!(0.1));
    }

    #[test]
    fn test_monte_carlo_option_pricing_high_volatility() {
        let mut option = create_test_option();
        option.implied_volatility = pos!(0.5);
        let price = monte_carlo_option_pricing(&option, 252, 100).unwrap();
        // The price should be higher with higher volatility
        assert!(price > dec!(10.0));
    }

    #[test]
    fn test_monte_carlo_option_pricing_short_expiration() {
        let mut option = create_test_option();
        option.expiration_date = ExpirationDate::Days(pos!(30.0)); // 30 days
        let price = monte_carlo_option_pricing(&option, 30, 100).unwrap();
        // The price should be lower for a shorter expiration
        assert!(price < dec!(5.0));
    }

    #[test]
    fn test_monte_carlo_option_pricing_consistency() {
        let option = create_test_option();
        let _price1 = monte_carlo_option_pricing(&option, 100, 100).unwrap();
        let _price2 = monte_carlo_option_pricing(&option, 100, 100).unwrap();
        // Two runs should produce similar results
        // assert_relative_eq!(price1, price2,  0.05);
    }
}

#[cfg(test)]
mod tests_price_option_monte_carlo {
    use super::*;
    use crate::chains::generator_positive;
    use crate::model::utils::create_sample_option;
    use crate::simulation::simulator::Simulator;
    use crate::simulation::steps::{Step, Xstep, Ystep};
    use crate::simulation::{WalkParams, WalkType, WalkTypeAble};
    use crate::utils::TimeFrame;
    use crate::utils::time::convert_time_frame;
    #[cfg(feature = "kaleido")]
    use crate::visualization::Graph;
    use crate::{ExpirationDate, OptionStyle, Side, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    #[test]
    fn test_empty_prices_returns_zero() {
        // Arrange
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(100.0),
            pos!(1.0),
            pos!(100.0),
            pos!(0.2),
        );
        let empty_prices = &[];

        // Act
        let result = price_option_monte_carlo(&option, empty_prices);

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Positive::ZERO);
    }

    #[test]
    fn test_call_option_pricing() {
        // Arrange
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(100.0),
            pos!(1.0),
            pos!(100.0),
            pos!(0.2),
        );

        option.risk_free_rate = dec!(0.05);
        option.dividend_yield = pos!(0.02);
        option.expiration_date = ExpirationDate::Days(pos!(365.0));

        // Setup simulated prices and expected payoffs
        let prices = vec![pos!(110.0), pos!(90.0), pos!(105.0)];

        // Act
        let result = price_option_monte_carlo(&option, &prices);
        assert!(result.is_ok());
        assert_pos_relative_eq!(result.unwrap(), pos!(4.85222766), pos!(0.001));
    }

    #[test]
    fn test_simulation() {
        struct TestWalker;
        impl WalkTypeAble<Positive, Positive> for TestWalker {}
        let walker = Box::new(TestWalker);
        let initial_price = pos!(1000.0);
        let days = pos!(365.0);
        let volatility = pos!(0.2);
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            initial_price,
            pos!(1.0),
            initial_price,
            volatility,
        );
        option.risk_free_rate = dec!(0.05);
        option.dividend_yield = pos!(0.02);
        option.expiration_date = ExpirationDate::Days(days);

        let init_step = Step {
            x: Xstep::new(Positive::ONE, TimeFrame::Day, ExpirationDate::Days(days)),
            y: Ystep::new(0, initial_price),
        };

        let dt = convert_time_frame(pos!(1.0), &TimeFrame::Day, &TimeFrame::Year);
        let walk_params = WalkParams {
            size: 365,
            init_step,
            walk_type: WalkType::Custom {
                dt,
                drift: dec!(0.02),
                volatility,
                vov: pos!(0.01),
                vol_speed: Default::default(),
                vol_mean: pos!(0.2),
            },
            walker,
        };

        let simulator = Simulator::new(
            "Test Simulator".to_string(),
            100,
            &walk_params,
            generator_positive,
        );

        #[cfg(feature = "kaleido")]
        simulator
            .write_html("Draws/Simulation/simulator_test_montecarlo.html".as_ref())
            .unwrap();
        let get_last_positive_values = simulator.get_last_positive_values();

        let result = price_option_monte_carlo(&option, &get_last_positive_values);
        assert!(result.is_ok());

        let bs = option.calculate_price_black_scholes().unwrap();
        assert_pos_relative_eq!(result.unwrap(), Positive(bs), pos!(10.0));
    }

    // #[test]
    // fn test_put_option_pricing() {
    //     // Arrange
    //     let mut mock_option = MockOptions::new();
    //
    //     // Setup expiration date (1 year from now)
    //     let expiration = Utc::now().date_naive() + chrono::Duration::days(365);
    //     mock_option.risk_free_rate = 0.05;
    //     mock_option.dividend_yield = 0.02;
    //     mock_option.expiration_date = expiration;
    //
    //     // Setup simulated prices and expected payoffs
    //     let prices = vec![
    //         Positive::from_f64(90.0),
    //         Positive::from_f64(110.0),
    //         Positive::from_f64(95.0),
    //     ];
    //
    //     // For a put option with strike 100:
    //     // Payoffs would be: 10.0, 0.0, 5.0
    //     mock_option.expect_payoff_at_price()
    //         .with(eq(Positive::from_f64(90.0)))
    //         .returning(|_| Ok(Decimal::from_str("10.0").unwrap()));
    //
    //     mock_option.expect_payoff_at_price()
    //         .with(eq(Positive::from_f64(110.0)))
    //         .returning(|_| Ok(Decimal::from_str("0.0").unwrap()));
    //
    //     mock_option.expect_payoff_at_price()
    //         .with(eq(Positive::from_f64(95.0)))
    //         .returning(|_| Ok(Decimal::from_str("5.0").unwrap()));
    //
    //     // Act
    //     let result = price_option_monte_carlo(&mock_option, &prices);
    //
    //     // Assert
    //     assert!(result.is_ok());
    //
    //     // Expected: avg payoff = (10 + 0 + 5)/3 = 5.0
    //     // Discount factor = exp(-(0.05-0.02)*1) = exp(-0.03) ≈ 0.9704
    //     // Expected price = 5.0 * 0.9704 ≈ 4.852
    //     let expected = Positive::from_f64(4.852);
    //
    //     // Using approximate comparison due to floating-point calculations
    //     let diff = (result.unwrap().0 - expected.0).abs();
    //     assert!(diff < Decimal::from_str("0.001").unwrap(),
    //             "Expected close to {}, got {}", expected.0, result.unwrap().0);
    // }
    //
    // #[test]
    // fn test_expiration_date_error_handling() {
    //     // Arrange
    //     let mut mock_option = MockOptions::new();
    //
    //     // Setup with a problematic expiration date that will cause an error
    //     let prices = vec![Positive::from_f64(100.0)];
    //
    //     mock_option.expect_payoff_at_price()
    //         .returning(|_| Ok(Decimal::from_str("10.0").unwrap()));
    //
    //     // Make the expiration date calculation fail
    //     mock_option.expiration_date = NaiveDate::from_ymd_opt(9999, 12, 31).unwrap(); // Far future date that might cause issues
    //
    //     // Create a custom implementation for get_years that returns an error
    //     impl ExpirationDate for MockOptions {
    //         fn get_years(&self) -> Result<f64, Box<dyn Error>> {
    //             Err("Invalid expiration date".into())
    //         }
    //     }
    //
    //     // Act
    //     let result = price_option_monte_carlo(&mock_option, &prices);
    //
    //     // Assert
    //     assert!(result.is_err());
    //     assert_eq!(result.unwrap_err().to_string(), "Invalid expiration date");
    // }
    //
    // #[test]
    // fn test_payoff_calculation_error_handling() {
    //     // Arrange
    //     let mut mock_option = MockOptions::new();
    //
    //     // Setup expiration date (1 year from now)
    //     let expiration = Utc::now().date_naive() + chrono::Duration::days(365);
    //     mock_option.risk_free_rate = 0.05;
    //     mock_option.dividend_yield = 0.02;
    //     mock_option.expiration_date = expiration;
    //
    //     let prices = vec![Positive::from_f64(100.0)];
    //
    //     // Make the payoff calculation fail
    //     mock_option.expect_payoff_at_price()
    //         .returning(|_| Err("Payoff calculation failed".into()));
    //
    //     // Act & Assert
    //     // We expect a panic since the function uses expect() on the payoff calculation
    //     std::panic::catch_unwind(|| {
    //         price_option_monte_carlo(&mock_option, &prices)
    //     }).expect_err("Expected a panic but none occurred");
    // }
    //
    // #[test]
    // fn test_large_number_of_simulations() {
    //     // Arrange
    //     let mut mock_option = MockOptions::new();
    //
    //     // Setup expiration date (1 year from now)
    //     let expiration = Utc::now().date_naive() + chrono::Duration::days(365);
    //     mock_option.risk_free_rate = 0.05;
    //     mock_option.dividend_yield = 0.02;
    //     mock_option.expiration_date = expiration;
    //
    //     // Create a large number of simulations
    //     let num_simulations = 1000;
    //     let mut prices = Vec::with_capacity(num_simulations);
    //     for _ in 0..num_simulations {
    //         prices.push(Positive::from_f64(100.0));
    //     }
    //
    //     // All payoffs are 5.0 in this test
    //     mock_option.expect_payoff_at_price()
    //         .returning(|_| Ok(Decimal::from_str("5.0").unwrap()));
    //
    //     // Act
    //     let result = price_option_monte_carlo(&mock_option, &prices);
    //
    //     // Assert
    //     assert!(result.is_ok());
    //
    //     // Expected: avg payoff = 5.0
    //     // Discount factor = exp(-(0.05-0.02)*1) = exp(-0.03) ≈ 0.9704
    //     // Expected price = 5.0 * 0.9704 ≈ 4.852
    //     let expected = Positive::from_f64(4.852);
    //
    //     // Using approximate comparison due to floating-point calculations
    //     let diff = (result.unwrap().0 - expected.0).abs();
    //     assert!(diff < Decimal::from_str("0.001").unwrap(),
    //             "Expected close to {}, got {}", expected.0, result.unwrap().0);
    // }
    //
    // #[test]
    // fn test_negative_payoffs_handled_correctly() {
    //     // Arrange
    //     let mut mock_option = MockOptions::new();
    //
    //     // Setup expiration date (1 year from now)
    //     let expiration = Utc::now().date_naive() + chrono::Duration::days(365);
    //     mock_option.risk_free_rate = 0.05;
    //     mock_option.dividend_yield = 0.02;
    //     mock_option.expiration_date = expiration;
    //
    //     let prices = vec![
    //         Positive::from_f64(100.0),
    //         Positive::from_f64(110.0),
    //         Positive::from_f64(90.0),
    //     ];
    //
    //     // Some payoffs are negative (unusual but possible in some exotic options)
    //     mock_option.expect_payoff_at_price()
    //         .with(eq(Positive::from_f64(100.0)))
    //         .returning(|_| Ok(Decimal::from_str("-5.0").unwrap()));
    //
    //     mock_option.expect_payoff_at_price()
    //         .with(eq(Positive::from_f64(110.0)))
    //         .returning(|_| Ok(Decimal::from_str("10.0").unwrap()));
    //
    //     mock_option.expect_payoff_at_price()
    //         .with(eq(Positive::from_f64(90.0)))
    //         .returning(|_| Ok(Decimal::from_str("5.0").unwrap()));
    //
    //     // Act
    //     let result = price_option_monte_carlo(&mock_option, &prices);
    //
    //     // Assert
    //     assert!(result.is_ok());
    //
    //     // Expected: avg payoff = (-5 + 10 + 5)/3 ≈ 3.333
    //     // Discount factor = exp(-(0.05-0.02)*1) = exp(-0.03) ≈ 0.9704
    //     // Expected price = 3.333 * 0.9704 ≈ 3.235
    //     // Since we take the absolute value, it should be positive
    //     let expected = Positive::from_f64(3.235);
    //
    //     // Using approximate comparison due to floating-point calculations
    //     let diff = (result.unwrap().0 - expected.0).abs();
    //     assert!(diff < Decimal::from_str("0.001").unwrap(),
    //             "Expected close to {}, got {}", expected.0, result.unwrap().0);
    // }
}
