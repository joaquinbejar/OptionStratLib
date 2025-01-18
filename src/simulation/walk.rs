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
use crate::{pos, Positive};
use num_traits::ToPrimitive;
use rand::distributions::Distribution;
use rand::thread_rng;
use rust_decimal::Decimal;
use statrs::distribution::Normal;
use std::error::Error;
use tracing::{info, trace};

pub trait Walkable {
    fn get_y_values(&mut self) -> &mut Vec<Positive>;

    fn generate_random_walk(
        &mut self,
        n_steps: usize,
        initial_price: Positive,
        mean: f64,
        std_dev: Positive,
        std_dev_change: Positive,
    ) {
        if n_steps == 0 {
            panic!("Number of steps must be greater than zero");
        }
        let mut rng = thread_rng();
        let mut current_std_dev = std_dev;
        let mut result = Vec::with_capacity(n_steps);
        result.push(initial_price);
        let mut current_value = initial_price;

        let values = self.get_y_values();
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
            let normal = Normal::new(mean, current_std_dev.to_f64()).unwrap();
            let step = normal.sample(&mut rng);
            current_value = pos!((current_value.to_f64() + step).max(ZERO));
            values.push(current_value);
            trace!("Current value: {}", current_value);
        }
    }
}

pub struct RandomWalkGraph {
    values: Vec<Positive>,
    title_text: String,
    current_index: usize,
    risk_free_rate: Option<Decimal>,
    dividend_yield: Option<Positive>,
    time_frame: TimeFrame,
    volatility_window: usize,
    initial_volatility: Option<Positive>,
}

impl RandomWalkGraph {
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

    pub fn reset_iterator(&mut self) {
        self.current_index = 0;
    }

    fn get_remaining_time(&self) -> Positive {
        pos!((self.values.len() - self.current_index) as f64)
    }
}

impl Walkable for RandomWalkGraph {
    fn get_y_values(&mut self) -> &mut Vec<Positive> {
        &mut self.values
    }
}

impl Profit for RandomWalkGraph {
    fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
        Ok(price.to_dec())
    }
}

impl Graph for RandomWalkGraph {
    fn title(&self) -> String {
        self.title_text.clone()
    }

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
        fn get_y_values(&mut self) -> &mut Vec<Positive> {
            &mut self.values
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_walk_initialization() {
        let mut walk = TestWalk::new();
        let initial_price = pos!(100.0);

        walk.generate_random_walk(10, initial_price, 0.0, pos!(1.0), pos!(0.01));

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
        walk.generate_random_walk(n_steps, initial_price, mean, std_dev, std_dev_change);
        assert_eq!(walk.values.len(), n_steps);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_random_walk_starts_at_initial_price() {
        let mut walk = TestWalk::new();

        let initial_price = pos!(100.0);
        walk.generate_random_walk(10, initial_price, 0.0, pos!(1.0), pos!(0.01));
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
        walk.generate_random_walk(n_steps, initial_price, mean, std_dev, std_dev_change);
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
        walk.generate_random_walk(n_steps, initial_price, mean, std_dev, std_dev_change);
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

        walk.generate_random_walk(n_steps, initial_price, mean, std_dev, std_dev_change);
        assert_eq!(walk.values.len(), n_steps);
        assert!(walk.values.iter().all(|x| x > 0.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_edge_cases() {
        let mut walk = TestWalk::new();

        walk.generate_random_walk(1, pos!(100.0), 0.0, pos!(1.0), pos!(0.01));
        assert_eq!(walk.values.len(), 1);
        assert_eq!(walk.values[0], 100.0);

        walk.generate_random_walk(100, pos!(0.1), 0.0, pos!(0.01), pos!(0.001));
        assert!(walk.values.iter().all(|x| x >= 0.0));

        walk.generate_random_walk(100, pos!(1e6), 0.0, pos!(100.0), pos!(1.0));
        assert!(walk.values.iter().all(|x| x >= 0.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[should_panic]
    fn test_zero_steps_should_panic() {
        let mut walk = TestWalk::new();

        walk.generate_random_walk(0, pos!(100.0), 0.0, pos!(1.0), pos!(0.01));
    }
}

#[cfg(test)]
mod tests {
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
        );

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
