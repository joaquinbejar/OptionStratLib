/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 22/10/24
******************************************************************************/
use crate::chains::utils::OptionDataPriceParams;
use crate::constants::ZERO;
use crate::model::types::{ExpirationDate, PositiveF64, PZERO};
use crate::pricing::payoff::Profit;
use crate::utils::time::TimeFrame;
use crate::visualization::utils::Graph;
use crate::{pos, spos};
use rand::distributions::Distribution;
use rand::thread_rng;
use statrs::distribution::Normal;
use tracing::{info, trace};

pub trait Walkable {
    fn get_y_values(&mut self) -> &mut Vec<PositiveF64>;

    fn generate_random_walk(
        &mut self,
        n_steps: usize,
        initial_price: PositiveF64,
        mean: f64,
        std_dev: PositiveF64,
        std_dev_change: PositiveF64,
    ) {
        if n_steps == 0 {
            panic!("Number of steps must be greater than zero");
        }
        let mut rng = thread_rng();
        let mut current_std_dev = std_dev.value();
        let mut result = Vec::with_capacity(n_steps);
        result.push(initial_price);
        let mut current_value = initial_price.value();

        let values = self.get_y_values();
        values.clear();
        values.reserve(n_steps);
        values.push(initial_price);

        for _ in 0..n_steps - 1 {
            if std_dev_change > PZERO {
                current_std_dev = Normal::new(std_dev.value(), std_dev_change.value())
                    .unwrap()
                    .sample(&mut rng)
                    .max(0.0);
            }
            let normal = Normal::new(mean, current_std_dev).unwrap();
            let step = normal.sample(&mut rng);
            current_value = (current_value + step).max(ZERO);
            values.push(pos!(current_value));
            trace!("Current value: {}", current_value);
        }
    }
}

pub struct RandomWalkGraph {
    values: Vec<PositiveF64>,
    title_text: String,
    current_index: usize,
    risk_free_rate: Option<f64>,
    dividend_yield: Option<f64>,
    time_frame: TimeFrame,
    volatility_window: usize,
    initial_volatility: Option<PositiveF64>,
}

impl RandomWalkGraph {
    pub fn new(
        title: String,
        risk_free_rate: Option<f64>,
        dividend_yield: Option<f64>,
        time_frame: TimeFrame,
        volatility_window: usize,
        initial_volatility: Option<PositiveF64>,
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

    fn calculate_current_volatility(&self) -> Option<PositiveF64> {
        if self.current_index < 2 {
            return self.initial_volatility;
        }

        let returns: Vec<f64> = self.values[..self.current_index]
            .windows(2)
            .map(|w| (w[1].value() - w[0].value()) / w[0].value())
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
            spos!(volatility)
        }
    }

    pub fn reset_iterator(&mut self) {
        self.current_index = 0;
    }

    fn get_remaining_time(&self) -> f64 {
        (self.values.len() - self.current_index) as f64
    }
}

impl Walkable for RandomWalkGraph {
    fn get_y_values(&mut self) -> &mut Vec<PositiveF64> {
        &mut self.values
    }
}

impl Profit for RandomWalkGraph {
    fn calculate_profit_at(&self, price: PositiveF64) -> f64 {
        price.value()
    }
}

impl Graph for RandomWalkGraph {
    fn title(&self) -> String {
        self.title_text.clone()
    }

    fn get_values(&self, _data: &[PositiveF64]) -> Vec<f64> {
        info!("Number of values: {}", self.values.len());
        info!("First value: {:?}", self.values.first().unwrap());
        info!("Last value: {:?}", self.values.last().unwrap());
        self.values.iter().map(|x| x.value()).collect()
    }
}

impl Iterator for RandomWalkGraph {
    type Item = OptionDataPriceParams;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.values.len() {
            return None;
        }

        let risk_free_rate = self.risk_free_rate.unwrap_or(ZERO);
        let dividend_yield = self.dividend_yield.unwrap_or(ZERO);

        // Obtiene el precio actual
        let price = self.values[self.current_index];

        // Calcula los días restantes hasta el vencimiento
        let remaining_days = self.get_remaining_time();
        let expiration_date = ExpirationDate::Days(remaining_days);

        // Calcula la volatilidad implícita usando el histórico
        let implied_volatility = self.calculate_current_volatility();

        // Incrementa el índice para la siguiente iteración
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
    use crate::model::types::PZERO;
    use statrs::statistics::Statistics;

    struct TestWalk {
        values: Vec<PositiveF64>,
    }

    impl TestWalk {
        fn new() -> Self {
            Self { values: Vec::new() }
        }
    }

    impl Walkable for TestWalk {
        fn get_y_values(&mut self) -> &mut Vec<PositiveF64> {
            &mut self.values
        }
    }

    #[test]
    fn test_walk_initialization() {
        let mut walk = TestWalk::new();
        let initial_price = pos!(100.0);

        walk.generate_random_walk(10, initial_price, 0.0, pos!(1.0), pos!(0.01));

        assert_eq!(walk.values.len(), 10);
        assert_eq!(walk.values[0], initial_price);
    }

    #[test]
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
    fn test_random_walk_starts_at_initial_price() {
        let mut walk = TestWalk::new();

        let initial_price = pos!(100.0);
        walk.generate_random_walk(10, initial_price, 0.0, pos!(1.0), pos!(0.01));
        assert_eq!(walk.values[0], initial_price);
    }

    #[test]
    fn test_all_values_are_positive() {
        let mut walk = TestWalk::new();
        let n_steps = 1000;
        let initial_price = pos!(100.0);
        let mean = 0.0;
        let std_dev = pos!(1.0);
        let std_dev_change = pos!(0.01);
        walk.generate_random_walk(n_steps, initial_price, mean, std_dev, std_dev_change);
        assert!(walk.values.iter().all(|x| x.value() > 0.0));
    }

    #[test]
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
            .map(|w| w[1].value() - w[0].value())
            .collect();

        let empirical_mean = changes.mean();
        assert!((empirical_mean - mean).abs() < 0.1);
    }

    #[test]
    fn test_zero_std_dev_change() {
        let mut walk = TestWalk::new();

        let n_steps = 100;
        let initial_price = pos!(100.0);
        let mean = 0.0;
        let std_dev = pos!(1.0);
        let std_dev_change = PZERO;

        walk.generate_random_walk(n_steps, initial_price, mean, std_dev, std_dev_change);
        assert_eq!(walk.values.len(), n_steps);
        assert!(walk.values.iter().all(|x| x.value() > 0.0));
    }

    #[test]
    fn test_edge_cases() {
        let mut walk = TestWalk::new();

        walk.generate_random_walk(1, pos!(100.0), 0.0, pos!(1.0), pos!(0.01));
        assert_eq!(walk.values.len(), 1);
        assert_eq!(walk.values[0].value(), 100.0);

        walk.generate_random_walk(100, pos!(0.1), 0.0, pos!(0.01), pos!(0.001));
        assert!(walk.values.iter().all(|x| x.value() >= 0.0));

        walk.generate_random_walk(100, pos!(1e6), 0.0, pos!(100.0), pos!(1.0));
        assert!(walk.values.iter().all(|x| x.value() >= 0.0));
    }

    #[test]
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
    use tracing::debug;

    #[test]
    fn test_random_walk_iterator() {
        setup_logger_with_level("debug");
        let years = 3.0;
        let steps = 252 * years as usize; // 2 years

        let mut walk = RandomWalkGraph::new(
            "Test Walk".to_string(),
            Some(0.05),     // risk_free_rate
            Some(0.02),     // dividend_yield
            TimeFrame::Day, // time_frame (2 years)
            4,              // volatility_window
            spos!(0.2),     // initial_volatility
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

            assert!(params.underlying_price > PZERO, "Price should be positive");
            assert!(
                params.risk_free_rate >= ZERO,
                "Risk-free rate should be non-negative"
            );
            assert!(
                params.dividend_yield >= ZERO,
                "Dividend yield should be non-negative"
            );

            if let Some(iv) = params.implied_volatility {
                assert!(iv >= PZERO, "Implied volatility should be positive");
            }
        }
    }
}
