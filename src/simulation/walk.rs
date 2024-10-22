/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 22/10/24
 ******************************************************************************/
use std::error::Error;
use plotters::prelude::RGBColor;
use rand::distributions::Distribution;
use rand::thread_rng;
use statrs::distribution::Normal;
use crate::model::types::{PositiveF64, PZERO};
use crate::pos;
use crate::pricing::payoff::Profit;
use crate::visualization::model::{ChartPoint, ChartVerticalLine};
use crate::visualization::utils::Graph;

pub(crate) trait Walkable {
    fn get_values(&mut self) -> &mut Vec<PositiveF64>;

    fn generate_random_walk(&mut self, n_steps: usize, initial_price: PositiveF64, mean: f64, std_dev: PositiveF64, std_dev_change: PositiveF64) {
        let mut rng = thread_rng();
        let mut current_std_dev = std_dev.value();
        let mut result = Vec::with_capacity(n_steps);
        result.push(initial_price);
        let mut current_value = initial_price.value();

        let values = self.get_values();
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
            current_value += step;
            values.push(pos!(current_value));
        }
    }
}

pub struct RandomWalkGraph {
    values: Vec<PositiveF64>,
    title_text: String,
}

impl RandomWalkGraph {
    pub fn new(values: Vec<PositiveF64>, title: String) -> Self {
        Self {
            values,
            title_text: title,
        }
    }
}

impl Walkable for RandomWalkGraph {
    fn get_values(&mut self) -> &mut Vec<PositiveF64> {
        &mut self.values
    }
}

impl Profit for RandomWalkGraph {
    fn calculate_profit_at(&self, price: PositiveF64) -> f64 {
        // Para el random walk, el "profit" es simplemente el valor en ese punto
        price.value()
    }
}

impl Graph for RandomWalkGraph {
    fn title(&self) -> String {
        self.title_text.clone()
    }

    fn get_values(&self, _data: &[PositiveF64]) -> Vec<f64> {
        // Sobreescribimos get_values para usar nuestros propios valores en lugar
        // de calcular profits
        self.values.iter().map(|x| x.value()).collect()
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        Vec::new()  // Por defecto no incluimos líneas verticales
    }

    fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> {
        // Opcional: Podríamos marcar puntos específicos como máximos o mínimos
        let mut points = Vec::new();

        // Encontrar el máximo
        if let Some((index, &max_value)) = self.values.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.value().partial_cmp(&b.value()).unwrap()) {
            points.push(ChartPoint {
                coordinates: (index as f64, max_value.value()),
                label: "Max".to_string(),
                label_offset: (5.0, -10.0),
                point_color: RGBColor(0, 100, 0),  // Verde oscuro
                label_color: RGBColor(0, 100, 0),
                point_size: 5,
                font_size: 12,
            });
        }

        // Encontrar el mínimo
        if let Some((index, &min_value)) = self.values.iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.value().partial_cmp(&b.value()).unwrap()) {
            points.push(ChartPoint {
                coordinates: (index as f64, min_value.value()),
                label: "Min".to_string(),
                label_offset: (5.0, 10.0),
                point_color: RGBColor(100, 0, 0),  // Rojo oscuro
                label_color: RGBColor(100, 0, 0),
                point_size: 5,
                font_size: 12,
            });
        }

        points
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::pos;


    #[test]
    fn test_random_walk_graph_values() {
        let values = vec![pos!(100.0), pos!(101.0), pos!(102.0)];
        let graph = RandomWalkGraph::new(values.clone(), "Test".to_string());

        let result = graph.get_values(&[]);
        assert_eq!(result, vec![100.0, 101.0, 102.0]);
    }

    #[test]
    fn test_random_walk_points() {
        let values = vec![pos!(100.0), pos!(101.0), pos!(99.0)];
        let graph = RandomWalkGraph::new(values, "Test".to_string());

        let points = graph.get_points();
        assert!(!points.is_empty());

        // Debería haber identificado un máximo y un mínimo
        assert_eq!(points.len(), 2);
    }
}

#[cfg(test)]
mod tests_random_walk {
    use super::*;
    use statrs::statistics::Statistics;
    use crate::model::types::PZERO;

    struct TestWalk {
        values: Vec<PositiveF64>,
    }

    impl TestWalk {
        fn new() -> Self {
            Self {
                values: Vec::new(),
            }
        }
    }

    impl Walkable for TestWalk {
        fn get_values(&mut self) -> &mut Vec<PositiveF64> {
            &mut self.values
        }
    }

    #[test]
    fn test_walk_initialization() {
        let mut walk = TestWalk::new();
        let initial_price = pos!(100.0);

        walk.generate_random_walk(
            10,
            initial_price,
            0.0,
            pos!(1.0),
            pos!(0.01)
        );

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
        let result = walk.generate_random_walk(n_steps, initial_price, mean, std_dev, std_dev_change);
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
        let result = walk.generate_random_walk(n_steps, initial_price, mean, std_dev, std_dev_change);
        let changes: Vec<f64> = walk.values.windows(2)
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

        let result = walk.generate_random_walk(n_steps, initial_price, mean, std_dev, std_dev_change);
        assert_eq!(walk.values.len(), n_steps);
        assert!(walk.values.iter().all(|x| x.value() > 0.0));
    }

    #[test]
    fn test_edge_cases() {
        let mut walk = TestWalk::new();

        let result = walk.generate_random_walk(1, pos!(100.0), 0.0, pos!(1.0), pos!(0.01));
        assert_eq!(walk.values.len(), 1);
        assert_eq!(walk.values[0].value(), 100.0);

        let result = walk.generate_random_walk(100, pos!(0.1), 0.0, pos!(0.01), pos!(0.001));
        assert!(walk.values.iter().all(|x| x.value() > 0.0));

        let result = walk.generate_random_walk(100, pos!(1e6), 0.0, pos!(100.0), pos!(1.0));
        assert!(walk.values.iter().all(|x| x.value() > 0.0));
    }

    #[test]
    #[should_panic]
    fn test_zero_steps_should_panic() {
        let mut walk = TestWalk::new();

        walk.generate_random_walk(0, pos!(100.0), 0.0, pos!(1.0), pos!(0.01));
    }
}
