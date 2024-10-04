/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2/10/24
******************************************************************************/

use crate::constants::ZERO;
use crate::model::position::Position;
use crate::strategies::base::{Strategies, StrategyType};
use crate::visualization::model::{ChartPoint, ChartVerticalLine};
use crate::visualization::utils::Graph;
use plotters::prelude::full_palette::ORANGE;
use plotters::prelude::ShapeStyle;
use tracing::{debug, info, trace};

pub struct CustomStrategy {
    pub name: String,
    pub symbol: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<f64>,
    pub positions: Vec<Position>,
    pub underlying_price: f64,
    epsilon: f64,
    max_iterations: u32,
    step_by: f64,
}

impl CustomStrategy {
    pub fn new(
        name: String,
        symbol: String,
        description: String,
        underlying_price: f64,
        epsilon: f64,
        max_iterations: u32,
        step_by: f64,
    ) -> Self {
        CustomStrategy {
            name,
            symbol,
            kind: StrategyType::Custom,
            description,
            break_even_points: Vec::new(),
            positions: Vec::new(),
            underlying_price,
            epsilon,
            max_iterations,
            step_by,
        }
    }

    fn calculate_break_even_points(&mut self) {
        self.break_even_points = Vec::new();
        let step = self.step_by;

        let min_price = self
            .positions
            .iter()
            .map(|p| p.option.strike_price)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        let max_price = self
            .positions
            .iter()
            .map(|p| p.option.strike_price)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(100000.0);
        let range = if (max_price - min_price).abs() < f64::EPSILON {
            max_price * 2.0
        } else {
            (max_price - min_price) * 2.0
        };
        trace!("min_price: {} - max_price: {}", min_price, max_price);
        trace!("range: {}", range);

        let mut current_price = min_price - range * 0.5;
        let max_search_price = max_price + range * 0.5;
        debug!(
            "Calculating break-even points for price range: {} - {}",
            current_price, max_search_price
        );

        let mut last_profit = self.calculate_profit_at(current_price);

        while current_price < max_search_price {
            let current_profit = self.calculate_profit_at(current_price);

            if last_profit.signum() != current_profit.signum() {
                let mut low = current_price - step;
                let mut high = current_price;
                let mut iterations = 0;

                'inner: while (high - low).abs() > self.epsilon && iterations < self.max_iterations
                {
                    let mid = (low + high) / 2.0;
                    let mid_profit = self.calculate_profit_at(mid);

                    if mid_profit.abs() < self.epsilon {
                        debug!(
                            "Break-even point found at price: {} value {}",
                            mid, mid_profit
                        );
                        if !self.break_even_points.contains(&mid) {
                            self.break_even_points.push(mid);
                        }
                        break 'inner;
                    } else if mid_profit.signum() == last_profit.signum() {
                        low = mid;
                    } else {
                        high = mid;
                    }

                    iterations += 1;
                }

                if iterations == self.max_iterations {
                    debug!(
                        "Max iterations reached at price: {} value {}",
                        current_price, current_profit
                    );
                    let mid_point = (low + high) / 2.0;
                    if !self.break_even_points.contains(&mid_point) {
                        self.break_even_points.push(mid_point);
                    }
                }
            }

            last_profit = current_profit;
            current_price += step;
        }

        if self.break_even_points.is_empty() {
            info!("No break-even points found");
        } else {
            info!("Break-even points found: {:?}", self.break_even_points);
        }
    }
}

impl Strategies for CustomStrategy {
    fn add_leg(&mut self, position: Position) {
        self.positions.push(position);
        self.calculate_break_even_points();
    }

    fn break_even(&self) -> f64 {
        if self.break_even_points.is_empty() {
            panic!("No break-even points found");
        } else {
            self.break_even_points[0]
        }
    }

    fn calculate_profit_at(&self, price: f64) -> f64 {
        self.positions
            .iter()
            .map(|position| position.pnl_at_expiration(Some(price)))
            .sum()
    }

    fn max_profit(&self) -> f64 {
        if self.positions.is_empty() {
            return ZERO;
        }
        let step = self.step_by;
        let mut max_profit: f64 = f64::NEG_INFINITY;
        let min_price = self
            .positions
            .iter()
            .map(|p| p.option.strike_price)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        let max_price = self
            .positions
            .iter()
            .map(|p| p.option.strike_price)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(100000.0);
        let range = if (max_price - min_price).abs() < f64::EPSILON {
            max_price * 2.0
        } else {
            (max_price - min_price) * 2.0
        };
        trace!("min_price: {} - max_price: {}", min_price, max_price);
        trace!("range: {}", range);

        let mut current_price = min_price - range * 0.5;
        let max_search_price = max_price + range * 0.5;
        debug!(
            "Calculating break-even points for price range: {} - {}",
            current_price, max_search_price
        );

        while current_price < max_search_price {
            let current_profit = self.calculate_profit_at(current_price);
            if current_profit > max_profit {
                max_profit = current_profit;
            }
            current_price += step;
        }
        max_profit
    }

    fn max_loss(&self) -> f64 {
        if self.positions.is_empty() {
            return ZERO;
        }
        let step = self.step_by;
        let mut max_loss: f64 = f64::INFINITY;
        let min_price = self
            .positions
            .iter()
            .map(|p| p.option.strike_price)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        let max_price = self
            .positions
            .iter()
            .map(|p| p.option.strike_price)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(100000.0);
        let range = if (max_price - min_price).abs() < f64::EPSILON {
            max_price * 2.0
        } else {
            (max_price - min_price) * 2.0
        };
        trace!("min_price: {} - max_price: {}", min_price, max_price);
        trace!("range: {}", range);

        let mut current_price = min_price - range * 0.5;
        let max_search_price = max_price + range * 0.5;
        debug!(
            "Calculating break-even points for price range: {} - {}",
            current_price, max_search_price
        );

        while current_price < max_search_price {
            let current_profit = self.calculate_profit_at(current_price);
            if current_profit < max_loss {
                max_loss = current_profit;
            }
            current_price += step;
        }
        max_loss
    }

    fn total_cost(&self) -> f64 {
        panic!("Not implemented yet");
    }

    fn net_premium_received(&self) -> f64 {
        panic!("Not implemented yet");
    }

    fn fees(&self) -> f64 {
        panic!("Not implemented yet");
    }

    fn profit_area(&self) -> f64 {
        panic!("Not implemented yet");
    }
}

impl Graph for CustomStrategy {
    fn title(&self) -> String {
        info!(self.description);
        let strategy_title = format!("{} Strategy: {:?} on {}", self.name, self.kind, self.symbol);
        let leg_titles: Vec<String> = self
            .positions
            .iter()
            .map(|position| position.title())
            .collect();
        if leg_titles.is_empty() {
            strategy_title
        } else {
            format!("{}\n\t{}", strategy_title, leg_titles.join("\n\t"))
        }
    }

    fn get_values(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&price| self.calculate_profit_at(price))
            .collect()
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        let max_value = self.max_profit() * 1.2;
        let min_value = self.max_profit() * -1.2;

        let vertical_lines = vec![ChartVerticalLine {
            x_coordinate: self.underlying_price,
            y_range: (min_value, max_value),
            label: format!("Current Price: {:.2}", self.underlying_price),
            label_offset: (4.0, -1.0),
            line_color: ORANGE,
            label_color: ORANGE,
            line_style: ShapeStyle::from(&ORANGE).stroke_width(2),
            font_size: 18,
        }];

        vertical_lines
    }

    fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> {
        let points: Vec<ChartPoint<(f64, f64)>> = Vec::new();
        // TODO: Implement the calculation of the point of Break Even and the max profit and loss points
        points
    }
}

#[cfg(test)]
mod tests_custom_strategy {
    use super::*;
    use crate::model::types::PositiveF64;
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option;
    use crate::pos;
    use crate::utils::logger::setup_logger;
    use approx::assert_relative_eq;

    fn create_test_strategy() -> CustomStrategy {
        CustomStrategy::new(
            "Test Strategy".to_string(),
            "AAPL".to_string(),
            "Test Description".to_string(),
            100.0, // underlying_price
            1e-16, // epsilon
            1000,  // max_iterations
            0.1,   // step_by
        )
    }

    #[test]
    fn test_new_custom_strategy() {
        let strategy = create_test_strategy();
        assert_eq!(strategy.name, "Test Strategy");
        assert_eq!(strategy.symbol, "AAPL");
        assert_eq!(strategy.kind, StrategyType::Custom);
        assert_eq!(strategy.description, "Test Description");
        assert_eq!(strategy.underlying_price, 100.0);
        assert!(strategy.break_even_points.is_empty());
        assert!(strategy.positions.is_empty());
    }

    #[test]
    fn test_calculate_break_even_points_single_call() {
        setup_logger();
        let mut strategy = create_test_strategy();
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            100.0,     // underlying_price
            pos!(1.0), // quantity
            100.0,     // strike_price
            0.2,       // volatility
        );
        let position = Position::new(option, 5.0, Default::default(), 0.0, 0.0); // premium
        strategy.add_leg(position);
        assert_eq!(strategy.break_even_points.len(), 1);
        assert_relative_eq!(
            strategy.break_even_points[0],
            105.0,
            epsilon = strategy.epsilon
        );
    }

    #[test]
    fn test_calculate_break_even_points_single_put() {
        let mut strategy = create_test_strategy();
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            100.0,     // underlying_price
            pos!(1.0), // quantity
            100.0,     // strike_price
            0.2,       // volatility
        );
        let position = Position::new(option, 5.0, Default::default(), 0.0, 0.0); // premium
        strategy.add_leg(position);
        assert_eq!(strategy.break_even_points.len(), 1);
        assert_relative_eq!(
            strategy.break_even_points[0],
            95.0,
            epsilon = strategy.epsilon
        );
    }

    #[test]
    fn test_calculate_break_even_points_straddle() {
        setup_logger();
        let mut strategy = create_test_strategy();
        let call = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            100.0,     // underlying_price
            pos!(1.0), // quantity
            100.0,     // strike_price
            0.2,       // volatility
        );
        let put = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            100.0,     // underlying_price
            pos!(1.0), // quantity
            100.0,     // strike_price
            0.2,       // volatility
        );
        let position_call = Position::new(call, 5.0, Default::default(), 0.0, 0.0);
        let position_put = Position::new(put, 5.0, Default::default(), 0.0, 0.0);
        strategy.add_leg(position_call);
        strategy.add_leg(position_put);

        assert_eq!(strategy.positions.len(), 2);
        assert_eq!(strategy.break_even_points.len(), 2);
        assert_relative_eq!(
            strategy.break_even_points[0],
            90.0,
            epsilon = strategy.epsilon
        );
        assert_relative_eq!(
            strategy.break_even_points[1],
            110.0,
            epsilon = strategy.epsilon
        );
    }

    #[test]
    fn test_calculate_break_even_points_no_break_even() {
        let mut strategy = create_test_strategy();
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            100.0,     // underlying_price
            pos!(1.0), // quantity
            100.0,     // strike_price
            0.2,       // volatility
        );
        strategy
            .positions
            .push(Position::new(option, 5.0, Default::default(), 0.0, 0.0));
        strategy.calculate_break_even_points();

        assert_eq!(strategy.break_even_points.len(), 1);
    }
}

#[cfg(test)]
mod tests_strategy_trait {
    use super::*;
    use crate::model::types::PositiveF64;
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option;
    use crate::pos;
    use crate::utils::logger::setup_logger;
    use approx::assert_relative_eq;

    fn create_test_strategy() -> CustomStrategy {
        CustomStrategy::new(
            "Test Strategy".to_string(),
            "AAPL".to_string(),
            "Test Description".to_string(),
            100.0, // underlying_price
            1e-16, // epsilon
            1000,  // max_iterations
            0.1,   // step_by
        )
    }

    #[test]
    fn test_add_leg() {
        let mut strategy = create_test_strategy();
        let option =
            create_sample_option(OptionStyle::Call, Side::Long, 100.0, pos!(1.0), 100.0, 0.2);
        let position = Position::new(option, 1.0, Default::default(), 0.0, 0.0);

        strategy.add_leg(position.clone());

        assert_eq!(strategy.positions.len(), 1);
        assert_eq!(strategy.positions[0].option.strike_price, 100.0);
        assert!(!strategy.break_even_points.is_empty()); // Assuming calculate_break_even_points adds at least one point
    }

    #[test]
    fn test_calculate_profit_at_i() {
        let mut strategy = create_test_strategy();

        // Add a long call
        let call_option =
            create_sample_option(OptionStyle::Call, Side::Long, 100.0, pos!(1.0), 100.0, 0.2);
        strategy.add_leg(Position::new(
            call_option,
            1.0,
            Default::default(),
            0.0,
            0.0,
        ));

        assert_relative_eq!(
            strategy.calculate_profit_at(80.0),
            -1.0,
            epsilon = strategy.epsilon
        );
        assert_relative_eq!(
            strategy.calculate_profit_at(120.0),
            19.0,
            epsilon = strategy.epsilon
        );

        // Test break-even point (this might be approximate)
        let break_even = strategy.break_even_points[0];
        let profit_at_break_even = strategy.calculate_profit_at(break_even);
        assert!(profit_at_break_even.abs() < 0.01); // Should be very close to zero
    }

    #[test]
    fn test_calculate_profit_at_ii() {
        setup_logger();
        let mut strategy = create_test_strategy();

        // Add a long call
        let call_option =
            create_sample_option(OptionStyle::Call, Side::Long, 100.0, pos!(1.0), 100.0, 0.2);
        strategy.add_leg(Position::new(
            call_option,
            1.0,
            Default::default(),
            0.0,
            0.0,
        ));

        let call_option =
            create_sample_option(OptionStyle::Call, Side::Long, 90.0, pos!(1.0), 90.0, 0.2);
        strategy.add_leg(Position::new(
            call_option,
            1.0,
            Default::default(),
            0.0,
            0.0,
        ));

        assert_relative_eq!(
            strategy.calculate_profit_at(80.0),
            -2.0,
            epsilon = strategy.epsilon
        );
        assert_relative_eq!(
            strategy.calculate_profit_at(95.0),
            3.0,
            epsilon = strategy.epsilon
        );
        assert_relative_eq!(
            strategy.calculate_profit_at(120.0),
            48.0,
            epsilon = strategy.epsilon
        );

        // Test break-even point (this might be approximate)
        let break_even = strategy.break_even_points[0];
        let profit_at_break_even = strategy.calculate_profit_at(break_even);
        assert_relative_eq!(break_even, 92.0, epsilon = strategy.epsilon);
        assert_relative_eq!(profit_at_break_even, 0.0, epsilon = strategy.epsilon);
    }

    #[test]
    fn test_calculate_profit_at_iii() {
        setup_logger();
        let mut strategy = create_test_strategy();

        // Add a long call
        let call_option =
            create_sample_option(OptionStyle::Call, Side::Long, 80.0, pos!(1.0), 80.0, 0.2);
        strategy.add_leg(Position::new(
            call_option,
            11.0,
            Default::default(),
            0.0,
            0.0,
        ));

        // Add a short put
        let put_option =
            create_sample_option(OptionStyle::Put, Side::Long, 100.0, pos!(1.0), 100.0, 0.2);
        strategy.add_leg(Position::new(
            put_option,
            11.0,
            Default::default(),
            0.0,
            0.0,
        ));

        assert_relative_eq!(
            strategy.calculate_profit_at(70.0),
            8.0,
            epsilon = strategy.epsilon
        );
        assert_relative_eq!(
            strategy.calculate_profit_at(80.0),
            -2.0,
            epsilon = strategy.epsilon
        );
        assert_relative_eq!(
            strategy.calculate_profit_at(90.0),
            -2.0,
            epsilon = strategy.epsilon
        );
        assert_relative_eq!(
            strategy.calculate_profit_at(110.0),
            8.0,
            epsilon = strategy.epsilon
        );

        assert_eq!(strategy.break_even_points.len(), 2);
        assert_relative_eq!(
            strategy.break_even_points[0],
            78.0,
            epsilon = strategy.epsilon
        );
        assert_relative_eq!(
            strategy.break_even_points[1],
            102.0,
            epsilon = strategy.epsilon
        );

        let break_even = strategy.break_even_points[0];
        let profit_at_break_even = strategy.calculate_profit_at(break_even);
        assert_relative_eq!(profit_at_break_even, 0.0, epsilon = strategy.epsilon);
    }

    #[test]
    fn test_multiple_legs() {
        let mut strategy = create_test_strategy();

        // Add multiple legs
        strategy.add_leg(Position::new(
            create_sample_option(OptionStyle::Call, Side::Long, 100.0, pos!(1.0), 100.0, 0.2),
            1.0,
            Default::default(),
            0.0,
            0.0,
        ));
        strategy.add_leg(Position::new(
            create_sample_option(OptionStyle::Call, Side::Short, 100.0, pos!(1.0), 110.0, 0.2),
            1.0,
            Default::default(),
            0.0,
            0.0,
        ));
        strategy.add_leg(Position::new(
            create_sample_option(OptionStyle::Put, Side::Long, 90.0, pos!(1.0), 90.0, 0.2),
            1.0,
            Default::default(),
            0.0,
            0.0,
        ));

        assert_eq!(strategy.positions.len(), 3);

        // Test profit calculation with multiple legs
        let profit = strategy.calculate_profit_at(105.0);
        assert_relative_eq!(profit, 4.0, epsilon = strategy.epsilon);
    }
}

#[cfg(test)]
mod tests_max_profit {
    use super::*;
    use crate::model::types::PositiveF64;
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option;
    use crate::pos;

    fn create_test_strategy() -> CustomStrategy {
        CustomStrategy::new(
            "Test Strategy".to_string(),
            "AAPL".to_string(),
            "Test Description".to_string(),
            100.0, // underlying_price
            1e-16, // epsilon
            1000,  // max_iterations
            0.1,   // step_by
        )
    }

    #[test]
    fn test_max_profit_single_long_call() {
        let mut strategy = create_test_strategy();

        let call_option =
            create_sample_option(OptionStyle::Call, Side::Long, 100.0, pos!(1.0), 100.0, 0.2);
        strategy.add_leg(Position::new(
            call_option,
            1.0,
            Default::default(),
            0.0,
            0.0,
        ));

        let max_profit = strategy.max_profit();
        assert!(max_profit > 0.0);
    }

    #[test]
    fn test_max_profit_single_short_put() {
        let mut strategy = create_test_strategy();

        let put_option =
            create_sample_option(OptionStyle::Put, Side::Short, 90.0, pos!(1.0), 90.0, 0.2);
        strategy.add_leg(Position::new(put_option, 1.0, Default::default(), 0.0, 0.0));

        let max_profit = strategy.max_profit();
        assert!(max_profit > 0.0);
    }

    #[test]
    fn test_max_profit_multi_leg_strategy() {
        let mut strategy = create_test_strategy();

        let call_option =
            create_sample_option(OptionStyle::Call, Side::Long, 100.0, pos!(1.0), 100.0, 0.2);
        strategy.add_leg(Position::new(
            call_option,
            1.0,
            Default::default(),
            0.0,
            0.0,
        ));

        let put_option =
            create_sample_option(OptionStyle::Put, Side::Short, 90.0, pos!(1.0), 90.0, 0.2);
        strategy.add_leg(Position::new(put_option, 1.0, Default::default(), 0.0, 0.0));

        let max_profit = strategy.max_profit();
        assert!(max_profit > 0.0);
    }

    #[test]
    fn test_max_profit_no_positions() {
        let strategy = create_test_strategy();

        let max_profit = strategy.max_profit();
        assert_eq!(max_profit, ZERO);
    }

    #[test]
    fn test_max_profit_with_small_step() {
        let mut strategy = create_test_strategy();
        strategy.step_by = 0.01;

        let call_option =
            create_sample_option(OptionStyle::Call, Side::Long, 100.0, pos!(1.0), 100.0, 0.2);
        strategy.add_leg(Position::new(
            call_option,
            1.0,
            Default::default(),
            0.0,
            0.0,
        ));

        let max_profit = strategy.max_profit();
        assert!(max_profit > 0.0);
    }

    #[test]
    fn test_max_profit_large_range() {
        let mut strategy = create_test_strategy();

        let call_option_1 =
            create_sample_option(OptionStyle::Call, Side::Long, 50.0, pos!(1.0), 50.0, 0.2);
        let call_option_2 =
            create_sample_option(OptionStyle::Call, Side::Short, 50.0, pos!(1.0), 150.0, 0.2);
        strategy.add_leg(Position::new(
            call_option_1,
            1.0,
            Default::default(),
            0.0,
            0.0,
        ));
        strategy.add_leg(Position::new(
            call_option_2,
            1.0,
            Default::default(),
            0.0,
            0.0,
        ));

        let max_profit = strategy.max_profit();
        assert!(max_profit > 0.0); // A pesar de la distancia de strikes, debería haber un beneficio positivo
    }
}

#[cfg(test)]
mod tests_max_loss {
    use super::*;
    use crate::model::types::PositiveF64;
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option;
    use crate::pos;

    fn create_test_strategy() -> CustomStrategy {
        CustomStrategy::new(
            "Test Strategy".to_string(),
            "AAPL".to_string(),
            "Test Description".to_string(),
            100.0, // underlying_price
            1e-16, // epsilon
            1000,  // max_iterations
            0.1,   // step_by
        )
    }

    #[test]
    fn test_max_loss_single_long_call() {
        let mut strategy = create_test_strategy();

        let call_option =
            create_sample_option(OptionStyle::Call, Side::Long, 100.0, pos!(1.0), 100.0, 0.2);
        strategy.add_leg(Position::new(
            call_option,
            1.0,
            Default::default(),
            0.0,
            0.0,
        ));

        let max_loss = strategy.max_loss();
        assert!(max_loss < 0.0);
    }

    #[test]
    fn test_max_loss_single_short_put() {
        let mut strategy = create_test_strategy();

        let put_option =
            create_sample_option(OptionStyle::Put, Side::Short, 90.0, pos!(1.0), 90.0, 0.2);
        strategy.add_leg(Position::new(put_option, 1.0, Default::default(), 0.0, 0.0));

        let max_loss = strategy.max_loss();
        assert!(max_loss < 0.0); // Para una opción corta, la pérdida máxima debería ser negativa
    }

    #[test]
    fn test_max_loss_multi_leg_strategy() {
        let mut strategy = create_test_strategy();

        let call_option =
            create_sample_option(OptionStyle::Call, Side::Long, 100.0, pos!(1.0), 100.0, 0.2);
        strategy.add_leg(Position::new(
            call_option,
            1.0,
            Default::default(),
            0.0,
            0.0,
        ));

        let put_option =
            create_sample_option(OptionStyle::Put, Side::Short, 90.0, pos!(1.0), 90.0, 0.2);
        strategy.add_leg(Position::new(put_option, 1.0, Default::default(), 0.0, 0.0));

        let max_loss = strategy.max_loss();
        assert!(max_loss < 0.0);
    }

    #[test]
    fn test_max_loss_no_positions() {
        let strategy = create_test_strategy();

        let max_loss = strategy.max_loss();
        assert_eq!(max_loss, ZERO);
    }

    #[test]
    fn test_max_loss_with_small_step() {
        let mut strategy = create_test_strategy();
        strategy.step_by = 0.01;

        let call_option =
            create_sample_option(OptionStyle::Call, Side::Long, 100.0, pos!(1.0), 100.0, 0.2);
        strategy.add_leg(Position::new(
            call_option,
            1.0,
            Default::default(),
            0.0,
            0.0,
        ));

        let max_loss = strategy.max_loss();
        assert!(max_loss < 0.0);
    }

    #[test]
    fn test_max_loss_large_range() {
        let mut strategy = create_test_strategy();

        let call_option_1 =
            create_sample_option(OptionStyle::Call, Side::Long, 50.0, pos!(1.0), 50.0, 0.2);
        let call_option_2 =
            create_sample_option(OptionStyle::Call, Side::Short, 50.0, pos!(1.0), 150.0, 0.2);
        strategy.add_leg(Position::new(
            call_option_1,
            1.0,
            Default::default(),
            0.1,
            0.0,
        ));
        strategy.add_leg(Position::new(
            call_option_2,
            1.0,
            Default::default(),
            0.1,
            0.0,
        ));

        let max_loss = strategy.max_loss();
        assert!(max_loss < 0.0);
    }
}
