/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 22/10/24
******************************************************************************/
use crate::chains::utils::OptionDataPriceParams;
use crate::constants::ZERO;
use crate::model::types::ExpirationDate;
use crate::pricing::payoff::Profit;
use crate::utils::time::TimeFrame;
use crate::visualization::utils::Graph;
use crate::volatility::adjust_volatility;
use crate::{pos, Positive};
use num_traits::ToPrimitive;
use rand::distributions::Distribution;
use rand::thread_rng;
use rust_decimal::Decimal;
use statrs::distribution::Normal;
use std::error::Error;
use tracing::{info, trace};

/// The `Walkable` trait defines a generic structure for creating and manipulating
/// entities capable of simulating or managing a random walk sequence of values.
/// Implementations of this trait must handle a vector of `Positive` values, which
/// serve as the primary storage for the y-axis values used in simulations or computations.
pub trait Walkable {
    /// Provides mutable access to the vector of y-axis values (`Positive`)
    /// associated with the structure implementing this trait.
    ///
    /// # Returns
    /// A mutable reference to the `Vec<Positive>` containing y-axis values.
    fn get_y_values(&self) -> &Vec<Positive>;

    fn get_x_values(&self) -> Vec<Positive> {
        (0..self.get_y_values().len())
            .map(|i| Positive::from(i as f64))
            .collect()
    }

    fn get_y_values_ref(&mut self) -> &mut Vec<Positive>;

    /// Generates a random walk sequence of values using a normal distribution.
    ///
    /// # Arguments
    /// * `n_steps` - The total number of steps to generate in the random walk.
    /// * `initial_price` - The starting value of the sequence, represented as a `Positive`.
    /// * `mean` - The mean value for the normal distribution of price changes.
    /// * `std_dev` - The initial standard deviation (volatility) for the normal distribution.
    /// * `std_dev_change` - The daily change in volatility (volatility of volatility or VoV).
    ///
    /// # Errors
    /// Returns an error if:
    /// * `n_steps` is zero, as a random walk requires at least one step.
    ///
    /// # Behavior
    /// The function:
    /// 1. Ensures `n_steps` is greater than zero.
    /// 2. Initializes the random number generator and prepares the vector of output values.
    /// 3. Adjusts volatility dynamically based on `std_dev_change`.
    /// 4. Calculates steps using a normal distribution with the given `mean` and `std_dev`.
    /// 5. Converts all computed values into `Positive` and updates the underlying vector.
    fn generate_random_walk(
        &mut self,
        n_steps: usize,
        initial_price: Positive,
        mean: f64,
        std_dev: Positive,
        std_dev_change: Positive,
    ) -> Result<(), Box<dyn Error>> {
        if n_steps == 0 {
            return Err(Box::from("Number of steps must be greater than zero"));
        }
        let mut rng = thread_rng();
        let mut current_std_dev = std_dev;
        let mut result = Vec::with_capacity(n_steps);
        result.push(initial_price);
        let mut current_value = initial_price;

        let values = self.get_y_values_ref();
        values.clear();
        values.reserve(n_steps);
        values.push(initial_price);

        for _ in 0..n_steps - 1 {
            if std_dev_change > Positive::ZERO {
                current_std_dev = Normal::new(std_dev.into(), std_dev_change.into())
                    .unwrap()
                    .sample(&mut rng)
                    .max(ZERO)
                    .into();
            }

            let step = if current_std_dev.is_zero() {
                mean
            } else {
                let normal = Normal::new(mean, current_std_dev.to_f64()).unwrap();
                normal.sample(&mut rng)
            };

            current_value = pos!((current_value.to_f64() + step).max(ZERO));
            values.push(current_value);
            trace!("Current value: {}", current_value);
        }
        Ok(())
    }

    /// Generates a random walk sequence of values over a specified timeframe,
    /// adjusting for volatility changes for the given periods.
    ///
    /// # Arguments
    /// * `n_steps` - The total number of steps in the random walk.
    /// * `initial_price` - The initial value of the sequence as a `Positive`.
    /// * `mean` - The mean of the price change distribution.
    /// * `std_dev` - Daily volatility (standard deviation) represented as `Positive`.
    /// * `std_dev_change` - Daily change in volatility (VoV) represented as `Positive`.
    /// * `time_frame` - The timeframe over which the simulation takes place.
    /// * `volatility_limits` - An optional tuple representing the minimum and maximum volatility limits.
    ///
    /// # Errors
    /// Returns an error if:
    /// * `n_steps` is zero, as a random walk requires at least one step.
    ///
    /// # Behavior
    /// 1. Converts daily volatility (`std_dev`) and its change (`std_dev_change`) to the target timeframe.
    /// 2. Dynamically adjusts volatility based on the provided volatility change (`std_dev_change`).
    /// 3. Constrains volatility within the specified limits, if provided.
    /// 4. Computes step values using a normal distribution adjusted for the timeframe's volatility.
    /// 5. Updates the internal storage of `Positive` values with the generated sequence.
    #[allow(clippy::too_many_arguments)]
    fn generate_random_walk_timeframe(
        &mut self,
        n_steps: usize,
        initial_price: Positive,
        mean: f64,
        std_dev: Positive,        // daily volatility
        std_dev_change: Positive, // daily VoV
        time_frame: TimeFrame,
        volatility_limits: Option<(Positive, Positive)>,
    ) -> Result<(), Box<dyn Error>> {
        if n_steps == 0 {
            return Err(Box::from("Number of steps must be greater than zero"));
        }

        // Convert daily volatilities to target timeframe
        let std_dev_adjusted = adjust_volatility(std_dev, TimeFrame::Day, time_frame)?;
        let std_dev_change_adjusted =
            adjust_volatility(std_dev_change, TimeFrame::Day, time_frame)?;

        // Also adjust volatility limits if provided
        let volatility_limits_adjusted = volatility_limits
            .map(|(min, max)| -> Result<_, Box<dyn Error>> {
                Ok((
                    adjust_volatility(min, TimeFrame::Day, time_frame)?,
                    adjust_volatility(max, TimeFrame::Day, time_frame)?,
                ))
            })
            .transpose()?;

        let mut rng = thread_rng();
        let mut current_std_dev = std_dev_adjusted;
        let mut result = Vec::with_capacity(n_steps);
        result.push(initial_price);
        let mut current_value = initial_price;

        let values = self.get_y_values_ref();
        values.clear();
        values.reserve(n_steps);
        values.push(initial_price);

        for _ in 0..n_steps - 1 {
            if std_dev_change_adjusted > Positive::ZERO {
                // Generate new volatility with consideration of limits
                let new_vol = Normal::new(std_dev_adjusted.into(), std_dev_change_adjusted.into())
                    .unwrap()
                    .sample(&mut rng);

                // Apply volatility limits if provided
                current_std_dev = match &volatility_limits_adjusted {
                    Some((min, max)) => pos!(new_vol.max(min.to_f64()).min(max.to_f64())),
                    None => pos!(new_vol.max(ZERO)),
                };
            }

            let step = if current_std_dev.is_zero() {
                mean
            } else {
                let normal = Normal::new(mean, current_std_dev.to_f64()).unwrap();
                normal.sample(&mut rng)
            };

            current_value = pos!((current_value.to_f64() + step).max(ZERO));
            values.push(current_value);
            trace!("Current value: {}", current_value);
        }
        Ok(())
    }
}

/// Represents a specific implementation of the `Walkable` trait called `RandomWalkGraph`.
/// It is used to simulate and store a random walk's data, along with other metadata such
/// as titles, timeframes, and optional financial parameters (e.g., risk-free rate, dividend yield).
pub struct RandomWalkGraph {
    /// Values representing the y-axis data of the random walk.
    values: Vec<Positive>,
    /// Text for the graph's title.
    title_text: String,
    /// Tracks the current index for traversing the graph.
    current_index: usize,
    /// Optional risk-free rate used in calculations.
    risk_free_rate: Option<Decimal>,
    /// Optional dividend yield percentage.
    dividend_yield: Option<Positive>,
    /// Specifies the timeframe (e.g., daily, weekly) for the graph calculations.
    time_frame: TimeFrame,
    /// Determines the window size used in volatility calculations.
    volatility_window: usize,
    /// Optional initial volatility of the random walk.
    initial_volatility: Option<Positive>,
}

impl RandomWalkGraph {
    /// Creates a new `RandomWalkGraph` instance.
    ///
    /// # Arguments
    /// * `title` - A string representing the graph's title.
    /// * `risk_free_rate` - An optional risk-free rate as a `Decimal`.
    /// * `dividend_yield` - An optional dividend yield value as a `Positive`.
    /// * `time_frame` - The timeframe for the graph's data (e.g., daily, weekly).
    /// * `volatility_window` - The window size for calculating historical volatility.
    /// * `initial_volatility` - Optional initial volatility for the graph's random walk.
    pub fn new(
        title: String,
        risk_free_rate: Option<Decimal>,
        dividend_yield: Option<Positive>,
        time_frame: TimeFrame,
        volatility_window: usize,
        initial_volatility: Option<Positive>,
    ) -> Self {
        Self {
            values: Vec::new(),
            title_text: title,
            current_index: 0,
            risk_free_rate,
            dividend_yield,
            time_frame,
            volatility_window,
            initial_volatility,
        }
    }

    /// Calculates the current volatility of the random walk based on historical
    /// returns. This considers a moving window of data or defaults to the
    /// initial volatility if insufficient data exists.
    ///
    /// # Returns
    /// * The computed volatility as an optional `Positive`.
    /// * Returns `None` if the volatility cannot be computed or is invalid.
    fn calculate_current_volatility(&self) -> Option<Positive> {
        if self.current_index < 2 {
            return self.initial_volatility;
        }

        let returns: Vec<f64> = self.values[..self.current_index]
            .windows(2)
            .map(|w| ((w[1].to_dec() - w[0]) / w[0]).to_f64().unwrap())
            .collect();

        if returns.is_empty() {
            return self.initial_volatility;
        }

        let window_size = self.volatility_window.min(returns.len());
        let recent_returns = if returns.len() > window_size {
            &returns[returns.len() - window_size..]
        } else {
            &returns
        };

        let mean = recent_returns.iter().sum::<f64>() / recent_returns.len() as f64;
        let variance = recent_returns
            .iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>()
            / (recent_returns.len() - 1) as f64;

        let volatility = (variance * self.time_frame.periods_per_year()).sqrt();
        if volatility < ZERO || volatility.is_nan() {
            None
        } else {
            Some(volatility.into())
        }
    }

    /// Resets the iterator for traversing the graph's values to its starting position.
    pub fn reset_iterator(&mut self) {
        self.current_index = 0;
    }

    /// Calculates the remaining time, in steps, for the graph's iteration.
    ///
    /// # Returns
    /// The time remaining as a `Positive`.
    fn get_remaining_time(&self) -> Positive {
        pos!((self.values.len() - self.current_index) as f64)
    }
}

/// Implements the `Walkable` trait for the `RandomWalkGraph` structure. This provides
/// the graph with an interface for managing its y-values and performing random walk simulations.
impl Walkable for RandomWalkGraph {
    fn get_y_values(&self) -> &Vec<Positive> {
        &self.values
    }

    fn get_y_values_ref(&mut self) -> &mut Vec<Positive> {
        &mut self.values
    }
}

/// Implements a `Profit` trait for `RandomWalkGraph`, providing functionality
/// for calculating potential profit at a given price level.
impl Profit for RandomWalkGraph {
    /// Calculates the profit at a specified price, returning it as a `Decimal`.
    ///
    /// # Arguments
    /// * `price` - The price at which the profit is being calculated, represented as a `Positive`.
    ///
    /// # Returns
    /// A `Decimal` value representing the calculated profit.
    fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
        Ok(price.to_dec())
    }
}

/// Implements a `Graph` trait for `RandomWalkGraph`, allowing it to manage
/// the graphical data, such as titles and y-axis values.
impl Graph for RandomWalkGraph {
    /// Retrieves the title text for the graph.
    fn title(&self) -> String {
        self.title_text.clone()
    }

    /// Processes y-axis values from the graph and converts them into a vector of `f64`.
    ///
    /// # Arguments
    /// * `_data` - A slice of `Positive` values, optionally usable during processing.
    ///
    /// # Returns
    /// A vector of `f64` values as the processed y-axis data.
    fn get_values(&self, _data: &[Positive]) -> Vec<f64> {
        info!("Number of values: {}", self.values.len());
        info!("First value: {:?}", self.values.first().unwrap());
        info!("Last value: {:?}", self.values.last().unwrap());
        self.values.iter().map(|x| x.to_f64()).collect()
    }
}

/// Iterator implementation for `RandomWalkGraph` which generates `OptionDataPriceParams`.
///
/// This iterator traverses through a `RandomWalkGraph` object, producing
/// `OptionDataPriceParams` for each element in the underlying vector of price values.
///
/// # Type Alias
///
/// * `type Item` - Specifies the type of item produced by the iterator,
///   which is `OptionDataPriceParams`.
///
/// # Methods
///
/// * `next(&mut self) -> Option<Self::Item>` - Advances the iterator and
///   returns the next set of option data parameters. If all values have been
///   processed, it returns `None`.
///
///   - Checks if the `current_index` surpasses the length of the `values` vector.
///     If true, iteration stops by returning `None`.
///   - Extracts risk-free rate and dividend yield from their respective options,
///     defaulting to zero if not available.
///   - Retrieves the current price and calculates the remaining days using
///     `get_remaining_time()`.
///   - Determines the expiration date based on the remaining days available until expiration.
///   - Computes the current implied volatility using `calculate_current_volatility()`.
///   - Increments `current_index` to progress through the `values`.
///   - Returns a wrapped `Some()` with fields populated in `OptionDataPriceParams`.
///
/// # Fields
///
/// - `underlying_price`: Current price of the asset.
/// - `expiration_date`: Date at which the option expires, computed as
///   a number of days from the current index.
/// - `implied_volatility`: Estimated volatility of the asset over the
///   remaining time period.
/// - `risk_free_rate`: Interest rate assumed for risk-free investments.
/// - `dividend_yield`: Expected return from dividends, if applicable.
///
/// This design is useful for simulations or models where price and
/// volatility data need to be processed in a time-series format.
impl Iterator for RandomWalkGraph {
    type Item = OptionDataPriceParams;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.values.len() {
            return None;
        }

        let risk_free_rate: Decimal = self.risk_free_rate.unwrap_or(Decimal::ZERO);
        let dividend_yield: Positive = self.dividend_yield.unwrap_or(Positive::ZERO);
        let price = self.values[self.current_index];
        let remaining_days = self.get_remaining_time();
        let expiration_date = ExpirationDate::Days(remaining_days);
        let implied_volatility = self.calculate_current_volatility();
        self.current_index += 1;

        Some(OptionDataPriceParams {
            underlying_price: price,
            expiration_date,
            implied_volatility,
            risk_free_rate,
            dividend_yield,
            underlying_symbol: None,
        })
    }
}

#[cfg(test)]
mod tests_random_walk {
    use super::*;
    use crate::pos;
    use num_traits::ToPrimitive;
    use statrs::statistics::Statistics;

    struct TestWalk {
        values: Vec<Positive>,
    }

    impl TestWalk {
        fn new() -> Self {
            Self { values: Vec::new() }
        }
    }

    impl Walkable for TestWalk {
        fn get_y_values(&self) -> &Vec<Positive> {
            &self.values
        }

        fn get_y_values_ref(&mut self) -> &mut Vec<Positive> {
            &mut self.values
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_walk_initialization() {
        let mut walk = TestWalk::new();
        let initial_price = pos!(100.0);

        walk.generate_random_walk(10, initial_price, 0.0, pos!(1.0), pos!(0.01))
            .unwrap();

        assert_eq!(walk.values.len(), 10);
        assert_eq!(walk.values[0], initial_price);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_random_walk_length() {
        let mut walk = TestWalk::new();
        let n_steps = 100;
        let initial_price = pos!(100.0);
        let mean = 0.0;
        let std_dev = pos!(1.0);
        let std_dev_change = pos!(0.01);
        walk.generate_random_walk(n_steps, initial_price, mean, std_dev, std_dev_change)
            .unwrap();
        assert_eq!(walk.values.len(), n_steps);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_random_walk_starts_at_initial_price() {
        let mut walk = TestWalk::new();

        let initial_price = pos!(100.0);
        walk.generate_random_walk(10, initial_price, 0.0, pos!(1.0), pos!(0.01))
            .unwrap();
        assert_eq!(walk.values[0], initial_price);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_all_values_are_positive() {
        let mut walk = TestWalk::new();
        let n_steps = 1000;
        let initial_price = pos!(100.0);
        let mean = 0.0;
        let std_dev = pos!(1.0);
        let std_dev_change = pos!(0.01);
        walk.generate_random_walk(n_steps, initial_price, mean, std_dev, std_dev_change)
            .unwrap();
        assert!(walk.values.iter().all(|x| x > 0.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_statistical_properties() {
        let mut walk = TestWalk::new();

        let n_steps = 10000;
        let initial_price = pos!(100.0);
        let mean = 0.1;
        let std_dev = pos!(1.0);
        let std_dev_change = pos!(0.01);
        walk.generate_random_walk(n_steps, initial_price, mean, std_dev, std_dev_change)
            .unwrap();
        let changes: Vec<f64> = walk
            .values
            .windows(2)
            .map(|w| (w[1].to_dec() - w[0]).to_f64().unwrap())
            .collect();

        let empirical_mean = changes.mean();
        assert!((empirical_mean - mean).abs() < 0.1);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_zero_std_dev_change() {
        let mut walk = TestWalk::new();

        let n_steps = 100;
        let initial_price = pos!(100.0);
        let mean = 0.0;
        let std_dev = pos!(1.0);
        let std_dev_change = Positive::ZERO;

        walk.generate_random_walk(n_steps, initial_price, mean, std_dev, std_dev_change)
            .unwrap();
        assert_eq!(walk.values.len(), n_steps);
        assert!(walk.values.iter().all(|x| x > 0.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_edge_cases() {
        let mut walk = TestWalk::new();

        walk.generate_random_walk(1, pos!(100.0), 0.0, pos!(1.0), pos!(0.01))
            .unwrap();
        assert_eq!(walk.values.len(), 1);
        assert_eq!(walk.values[0], 100.0);

        walk.generate_random_walk(100, pos!(0.1), 0.0, pos!(0.01), pos!(0.001))
            .unwrap();
        assert!(walk.values.iter().all(|x| x >= 0.0));

        walk.generate_random_walk(100, pos!(1e6), 0.0, pos!(100.0), pos!(1.0))
            .unwrap();
        assert!(walk.values.iter().all(|x| x >= 0.0));
    }

    #[test]
    fn test_zero_steps_should_error() {
        let mut walk = TestWalk::new();
        let result = walk.generate_random_walk(0, pos!(100.0), 0.0, pos!(1.0), pos!(0.01));
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod tests_iterator {
    use super::*;
    use crate::utils::logger::setup_logger_with_level;
    use crate::{pos, spos};
    use rust_decimal_macros::dec;
    use tracing::debug;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_random_walk_iterator() {
        setup_logger_with_level("debug");
        let years = 3.0;
        let steps = 252 * years as usize; // 2 years

        let mut walk = RandomWalkGraph::new(
            "Test Walk".to_string(),
            Some(dec!(0.05)), // risk_free_rate
            Some(pos!(0.02)), // dividend_yield
            TimeFrame::Day,   // time_frame (2 years)
            4,                // volatility_window
            spos!(0.2),       // initial_volatility
        );

        walk.generate_random_walk(
            steps,       // n_steps
            pos!(100.0), // initial_price
            0.0,         // mean
            pos!(0.2),   // std_dev
            pos!(0.01),  // std_dev_change
        )
        .unwrap();

        for (i, params) in walk.enumerate() {
            debug!(
                "Step {}: Price={}, IV={:?}, Days to Exp={:?}",
                i, params.underlying_price, params.implied_volatility, params.expiration_date
            );

            assert!(
                params.underlying_price > Positive::ZERO,
                "Price should be positive"
            );
            assert!(
                params.risk_free_rate >= Decimal::ZERO,
                "Risk-free rate should be non-negative"
            );
            assert!(
                params.dividend_yield >= Positive::ZERO,
                "Dividend yield should be non-negative"
            );

            if let Some(iv) = params.implied_volatility {
                assert!(
                    iv >= Positive::ZERO,
                    "Implied volatility should be positive"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests_random_walk_timeframe {
    use super::*;
    use crate::pos;

    // Helper function to calculate volatility of a series
    fn calculate_volatility(values: &[Positive]) -> f64 {
        let returns: Vec<f64> = values
            .windows(2)
            .map(|w| (w[1].to_f64() / w[0].to_f64()).ln())
            .collect();

        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance =
            returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (returns.len() - 1) as f64;

        variance.sqrt()
    }

    // Mock struct for testing
    struct TestWalker {
        values: Vec<Positive>,
    }

    impl TestWalker {
        fn new() -> Self {
            Self { values: Vec::new() }
        }
    }

    impl Walkable for TestWalker {
        fn get_y_values(&self) -> &Vec<Positive> {
            &self.values
        }

        fn get_y_values_ref(&mut self) -> &mut Vec<Positive> {
            &mut self.values
        }

        // Implementing required generate_random_walk
        fn generate_random_walk(
            &mut self,
            n_steps: usize,
            initial_price: Positive,
            _mean: f64,
            _std_dev: Positive,
            _std_dev_change: Positive,
        ) -> Result<(), Box<dyn Error>> {
            // Simple implementation for testing
            self.values = vec![initial_price; n_steps];
            Ok(())
        }
    }

    #[test]
    fn test_zero_steps() {
        let mut walker = TestWalker::new();
        let result = walker.generate_random_walk_timeframe(
            0,
            pos!(100.0),
            0.0,
            pos!(0.2),
            pos!(0.1),
            TimeFrame::Day,
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_initial_price() {
        let mut walker = TestWalker::new();
        let initial_price = pos!(100.0);

        walker
            .generate_random_walk_timeframe(
                100,
                initial_price,
                0.0,
                pos!(0.2),
                pos!(0.0), // No volatility change
                TimeFrame::Day,
                None,
            )
            .unwrap();

        assert!(!walker.values.is_empty());
        assert_eq!(walker.values[0], initial_price);
    }

    #[test]
    fn test_correct_number_of_steps() {
        let mut walker = TestWalker::new();
        let n_steps = 100;

        // Usando valores más realistas para la volatilidad
        walker
            .generate_random_walk_timeframe(
                n_steps,
                pos!(100.0), // initial price
                0.0,         // mean
                pos!(0.01),  // std_dev (1% volatilidad diaria)
                pos!(0.001), // std_dev_change (0.1% VoV)
                TimeFrame::Day,
                None,
            )
            .unwrap();

        assert_eq!(walker.values.len(), n_steps);
    }

    #[test]
    fn test_volatility_limits() {
        let mut walker = TestWalker::new();
        let min_vol = pos!(0.1);
        let max_vol = pos!(0.3);

        walker
            .generate_random_walk_timeframe(
                1000,
                pos!(100.0),
                0.0,
                pos!(0.2),
                pos!(0.1),
                TimeFrame::Day,
                Some((min_vol, max_vol)),
            )
            .unwrap();

        // Check that values don't change too extremely
        let max_allowed_change = max_vol.to_f64() * 4.0; // 3 standard deviations
        for i in 1..walker.values.len() {
            let change = (walker.values[i].to_f64() - walker.values[i - 1].to_f64()).abs();
            assert!(
                change <= max_allowed_change,
                "Change between steps too large: {}",
                change
            );
        }
    }

    #[test]
    fn test_small_valid_volatility() {
        let mut walker = TestWalker::new();
        let n_steps = 100;

        walker
            .generate_random_walk_timeframe(
                n_steps,
                pos!(100.0),
                0.0,
                pos!(0.0001),
                pos!(0.00001),
                TimeFrame::Day,
                Some((pos!(0.00001), pos!(0.001))),
            )
            .unwrap();

        assert_eq!(walker.values.len(), n_steps);
    }

    #[test]
    fn test_different_timeframes() {
        let mut walker = TestWalker::new();
        let initial_price = pos!(100.0);
        let daily_vol = pos!(0.2);

        // Run with daily timeframe
        walker
            .generate_random_walk_timeframe(
                100,
                initial_price,
                0.0,
                daily_vol,
                pos!(0.0),
                TimeFrame::Day,
                None,
            )
            .unwrap();
        let daily_values = walker.values.clone();

        // Run with minute timeframe
        walker
            .generate_random_walk_timeframe(
                100,
                initial_price,
                0.0,
                daily_vol,
                pos!(0.0),
                TimeFrame::Minute,
                None,
            )
            .unwrap();
        let minute_values = walker.values.clone();

        // Minute values should have smaller changes between steps
        let daily_changes: Vec<f64> = daily_values
            .windows(2)
            .map(|w| (w[1].to_f64() - w[0].to_f64()).abs())
            .collect();
        let minute_changes: Vec<f64> = minute_values
            .windows(2)
            .map(|w| (w[1].to_f64() - w[0].to_f64()).abs())
            .collect();

        let avg_daily_change: f64 = daily_changes.iter().sum::<f64>() / daily_changes.len() as f64;
        let avg_minute_change: f64 =
            minute_changes.iter().sum::<f64>() / minute_changes.len() as f64;

        assert!(
            avg_minute_change < avg_daily_change,
            "Average minute change ({}) should be less than average daily change ({})",
            avg_minute_change,
            avg_daily_change
        );
    }

    #[test]
    fn test_no_negative_values() {
        let mut walker = TestWalker::new();

        walker
            .generate_random_walk_timeframe(
                1000,
                pos!(100.0),
                -0.1,      // Negative drift
                pos!(0.5), // High volatility
                pos!(0.1),
                TimeFrame::Day,
                None,
            )
            .unwrap();

        // Check that no values are negative
        assert!(walker.values.iter().all(|&v| v >= Positive::ZERO));
    }

    #[test]
    fn test_volatility_of_volatility() {
        let mut walker = TestWalker::new();

        // Test with no VoV
        walker
            .generate_random_walk_timeframe(
                1000,
                pos!(100.0),
                0.0,
                pos!(0.2),
                pos!(0.0), // No VoV
                TimeFrame::Day,
                None,
            )
            .unwrap();
        let no_vov_values = walker.values.clone();

        // Test with high VoV
        walker
            .generate_random_walk_timeframe(
                1000,
                pos!(100.0),
                0.0,
                pos!(0.2),
                pos!(0.2), // High VoV
                TimeFrame::Day,
                None,
            )
            .unwrap();
        let high_vov_values = walker.values;

        // Calculate price changes volatility
        let no_vov_volatility = calculate_volatility(&no_vov_values);
        let high_vov_volatility = calculate_volatility(&high_vov_values);

        // High VoV should result in more variable price changes
        assert!(
            high_vov_volatility > no_vov_volatility,
            "High VoV volatility ({}) should be greater than no VoV volatility ({})",
            high_vov_volatility,
            no_vov_volatility
        );
    }

    #[test]
    fn test_mean_drift() {
        let mut walker = TestWalker::new();
        let initial_price = pos!(100.0);
        let positive_drift = 0.01;

        walker
            .generate_random_walk_timeframe(
                1000,
                initial_price,
                positive_drift,
                pos!(0.1), // Low volatility for clearer trend
                pos!(0.0),
                TimeFrame::Day,
                None,
            )
            .unwrap();

        let final_value = walker.values.last().unwrap();
        assert!(
            *final_value > initial_price,
            "With positive drift, final value ({}) should be greater than initial value ({})",
            final_value,
            initial_price
        );
    }

    #[test]
    fn test_zero_volatility() {
        let mut walker = TestWalker::new();
        let n_steps = 100;
        let initial_price = pos!(100.0);
        let mean = 0.01; // positive drift

        walker
            .generate_random_walk_timeframe(
                n_steps,
                initial_price,
                mean,
                pos!(0.0), // zero volatility
                pos!(0.0), // zero VoV
                TimeFrame::Day,
                None,
            )
            .unwrap();

        // With zero volatility, each step should exactly equal the mean
        // So the price should follow a deterministic path
        let mut expected_price = initial_price;
        for i in 0..n_steps {
            assert_eq!(walker.values[i], expected_price);
            expected_price = pos!((expected_price.to_f64() + mean).max(0.0));
        }
    }

    #[test]
    fn test_zero_volatility_negative_drift() {
        let mut walker = TestWalker::new();
        let n_steps = 100;
        let initial_price = pos!(100.0);
        let mean = -0.01; // negative drift

        walker
            .generate_random_walk_timeframe(
                n_steps,
                initial_price,
                mean,
                pos!(0.0), // zero volatility
                pos!(0.0), // zero VoV
                TimeFrame::Day,
                None,
            )
            .unwrap();

        // With zero volatility and negative drift, price should decrease deterministically
        // until it hits zero
        let mut expected_price = initial_price;
        for i in 0..n_steps {
            assert_eq!(walker.values[i], expected_price);
            expected_price = pos!((expected_price.to_f64() + mean).max(0.0));
        }
    }
}
