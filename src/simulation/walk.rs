/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 22/10/24
******************************************************************************/
use crate::constants::ZERO;
use crate::model::types::{PositiveF64, PZERO};
use crate::pos;
use crate::pricing::payoff::Profit;
use crate::visualization::utils::Graph;
use rand::distributions::Distribution;
use rand::thread_rng;
use statrs::distribution::Normal;
use tracing::{debug, info};

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
            debug!("Current value: {}", current_value);
        }
    }
}

pub struct RandomWalkGraph {
    values: Vec<PositiveF64>,
    title_text: String,
}

impl RandomWalkGraph {
    pub fn new(title: String) -> Self {
        Self {
            values: Vec::new(),
            title_text: title,
        }
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
        assert!(walk.values.iter().all(|x| x.value() > 0.0));

        walk.generate_random_walk(100, pos!(1e6), 0.0, pos!(100.0), pos!(1.0));
        assert!(walk.values.iter().all(|x| x.value() > 0.0));
    }

    #[test]
    #[should_panic]
    fn test_zero_steps_should_panic() {
        let mut walk = TestWalk::new();

        walk.generate_random_walk(0, pos!(100.0), 0.0, pos!(1.0), pos!(0.01));
    }
}
