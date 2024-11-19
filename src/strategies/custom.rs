/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2/10/24
******************************************************************************/

use crate::chains::chain::OptionChain;
use crate::constants::{
    DARK_BLUE, DARK_GREEN, STRIKE_PRICE_LOWER_BOUND_MULTIPLIER,
    STRIKE_PRICE_UPPER_BOUND_MULTIPLIER, ZERO,
};
use crate::model::position::Position;
use crate::model::types::PositiveF64;
use crate::pos;
use crate::pricing::payoff::Profit;
use crate::strategies::base::{Optimizable, Strategies, StrategyType, Validable};
use crate::strategies::utils::{calculate_price_range, FindOptimalSide, OptimizationCriteria};
use crate::visualization::model::{ChartPoint, ChartVerticalLine};
use crate::visualization::utils::Graph;
use plotters::prelude::full_palette::ORANGE;
use plotters::prelude::{ShapeStyle, RED};
use tracing::{debug, error, info};

pub struct CustomStrategy {
    pub name: String,
    pub symbol: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<PositiveF64>,
    pub positions: Vec<Position>,
    pub underlying_price: PositiveF64,
    epsilon: f64,
    max_iterations: u32,
    step_by: f64,
    max_profit_point: Option<(PositiveF64, f64)>,
    max_loss_point: Option<(PositiveF64, f64)>,
}

impl CustomStrategy {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        symbol: String,
        description: String,
        underlying_price: PositiveF64,
        positions: Vec<Position>,
        epsilon: f64,
        max_iterations: u32,
        step_by: f64,
    ) -> Self {
        let mut strategy = CustomStrategy {
            name,
            symbol,
            kind: StrategyType::Custom,
            description,
            break_even_points: Vec::new(),
            positions,
            underlying_price,
            epsilon,
            max_iterations,
            step_by,
            max_profit_point: None,
            max_loss_point: None,
        };
        strategy.max_loss_iter();
        if !strategy.validate() {
            panic!("Invalid strategy");
        }
        strategy.calculate_break_even_points();
        strategy.max_profit_iter();
        strategy
    }

    fn calculate_break_even_points(&mut self) {
        self.break_even_points = Vec::new();
        let step = self.step_by;

        let (mut current_price, max_search_price) = self.range_to_show();
        let mut last_profit = self.calculate_profit_at(current_price);

        while current_price < max_search_price {
            let current_profit = self.calculate_profit_at(current_price);
            if last_profit.signum() != current_profit.signum() {
                let mut low = current_price - step;
                let mut high = current_price;
                let mut iterations = 0;

                'inner: while (high - low).value().abs() > self.epsilon
                    && iterations < self.max_iterations
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
            current_price += pos!(step);
        }

        if self.break_even_points.is_empty() {
            info!("No break-even points found");
        } else {
            info!(
                "Break Even Points found: {}",
                self.break_even_points
                    .iter()
                    .map(|p| format!("{:.2}", p))
                    .collect::<Vec<String>>()
                    .join(", ")
            );
        }
    }

    fn range_to_show(&self) -> (PositiveF64, PositiveF64) {
        let (first_option, last_option) = self.max_min_strikes();
        let start_price = first_option * STRIKE_PRICE_LOWER_BOUND_MULTIPLIER;
        let end_price = last_option * STRIKE_PRICE_UPPER_BOUND_MULTIPLIER;
        (start_price, end_price)
    }
}

impl Strategies for CustomStrategy {
    fn add_leg(&mut self, position: Position) {
        self.positions.push(position);
        self.max_loss_iter();
        if !self.validate() {
            panic!("Invalid position");
        }
        self.max_profit_iter();
        self.calculate_break_even_points();
    }

    fn get_legs(&self) -> Vec<Position> {
        self.positions.clone()
    }

    fn break_even(&self) -> Vec<PositiveF64> {
        if self.break_even_points.is_empty() {
            panic!("No break-even points found");
        } else {
            self.break_even_points.clone()
        }
    }

    fn max_profit_iter(&mut self) -> f64 {
        if self.positions.is_empty() {
            panic!("No positions found");
        }
        let step = self.step_by;
        let mut max_profit: f64 = f64::NEG_INFINITY;
        let (mut current_price, max_search_price) = self.range_to_show();
        while current_price < max_search_price {
            let current_profit = self.calculate_profit_at(current_price);
            if current_profit > max_profit {
                max_profit = current_profit;
                self.max_profit_point = Some((current_price, max_profit));
            }
            current_price += pos!(step);
        }
        max_profit
    }

    fn max_loss_iter(&mut self) -> f64 {
        if self.positions.is_empty() {
            return ZERO;
        }
        let step = self.step_by;
        let mut max_loss: f64 = f64::INFINITY;
        let (mut current_price, max_search_price) = self.range_to_show();
        while current_price < max_search_price {
            let current_profit = self.calculate_profit_at(current_price);
            if current_profit < max_loss {
                max_loss = current_profit;
                self.max_loss_point = Some((current_price, max_loss));
            }
            current_price += pos!(step);
        }
        debug!("Max Loss: {:.2}", max_loss);
        max_loss
    }

    fn total_cost(&self) -> f64 {
        panic!("Not implemented yet");
    }

    fn net_premium_received(&self) -> f64 {
        self.positions
            .iter()
            .map(|position| position.net_premium_received())
            .sum::<f64>()
    }

    fn fees(&self) -> f64 {
        self.positions
            .iter()
            .map(|position| position.open_fee + position.close_fee)
            .sum::<f64>()
    }

    fn profit_area(&self) -> f64 {
        if self.positions.is_empty() {
            panic!("No positions found");
        }

        let mut total_profit: f64 = ZERO;
        let (mut current_price, max_search_price) = self.range_to_show();
        while current_price < max_search_price {
            let current_profit = self.calculate_profit_at(current_price);
            if current_profit > ZERO {
                total_profit += current_profit;
            }
            current_price += pos!(self.step_by);
        }
        total_profit / current_price.value()
        // ZERO
    }

    fn profit_ratio(&self) -> f64 {
        let max_profit = self.max_profit_point.unwrap().1;
        let max_loss = self.max_loss_point.unwrap().1.abs();
        if max_loss == ZERO {
            f64::INFINITY
        } else {
            (max_profit / max_loss) * 100.0
        }
    }

    fn best_ratio(&mut self, option_chain: &OptionChain, side: FindOptimalSide) {
        self.find_optimal(option_chain, side, OptimizationCriteria::Ratio);
    }

    fn best_area(&mut self, option_chain: &OptionChain, side: FindOptimalSide) {
        self.find_optimal(option_chain, side, OptimizationCriteria::Area);
    }

    fn best_range_to_show(&self, step: PositiveF64) -> Option<Vec<PositiveF64>> {
        let (first_option, last_option) = self.max_min_strikes();

        let start_price = first_option * STRIKE_PRICE_LOWER_BOUND_MULTIPLIER;
        let end_price = last_option * STRIKE_PRICE_UPPER_BOUND_MULTIPLIER;
        Some(calculate_price_range(start_price, end_price, step))
    }
}

impl Validable for CustomStrategy {
    fn validate(&self) -> bool {
        if self.positions.is_empty() {
            error!("No positions found");
            return false;
        }

        match self.max_loss_point {
            Some(loss) => {
                if loss.1 >= ZERO {
                    error!("Max loss point is not valid");
                    return true;
                }
            }
            None => {
                error!("Max loss point not found");
                return false;
            }
        }

        self.positions.iter().all(|position| position.validate())
    }
}

impl Optimizable for CustomStrategy {
    type Strategy = CustomStrategy;

    fn find_optimal(
        &mut self,
        _option_chain: &OptionChain,
        _side: FindOptimalSide,
        _criteria: OptimizationCriteria,
    ) {
        todo!("Implement this method");
    }
}

impl Profit for CustomStrategy {
    fn calculate_profit_at(&self, price: PositiveF64) -> f64 {
        let price = Some(price);
        self.positions
            .iter()
            .map(|position| position.pnl_at_expiration(&price))
            .sum()
    }
}

impl Graph for CustomStrategy {
    fn title(&self) -> String {
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

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        let vertical_lines = vec![ChartVerticalLine {
            x_coordinate: self.underlying_price.value(),
            y_range: (-1e8, 1e8),
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
        let mut points: Vec<ChartPoint<(f64, f64)>> = Vec::new();

        for point in self.break_even_points.iter() {
            points.push(ChartPoint {
                coordinates: (point.value(), 0.0),
                label: format!("Break Even {:.2}", point),
                label_offset: (-26.0, 2.0),
                point_color: DARK_BLUE,
                label_color: DARK_BLUE,
                point_size: 5,
                font_size: 18,
            });
        }

        points.push(self.get_point_at_price(self.underlying_price));

        points.push(ChartPoint {
            coordinates: (
                self.max_profit_point.unwrap().0.value(),
                self.max_profit_point.unwrap().1,
            ),
            label: format!("Max Profit {:.2}", self.max_profit_point.unwrap().1),
            label_offset: (2.0, 1.0),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (
                self.max_loss_point.unwrap().0.value(),
                self.max_loss_point.unwrap().1,
            ),
            label: format!("Max Loss {:.2}", self.max_loss_point.unwrap().1),
            label_offset: (-30.0, 2.0),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        points
    }
}

#[cfg(test)]
mod tests_custom_strategy {
    use super::*;
    use crate::model::option::Options;
    use crate::model::types::{ExpirationDate, OptionType, PositiveF64};
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option;
    use crate::pos;
    use crate::utils::logger::setup_logger;
    use approx::assert_relative_eq;
    use chrono::Utc;

    fn create_test_strategy() -> CustomStrategy {
        let underlying_price = pos!(5780.0);
        let underlying_symbol = "SP500".to_string();
        let expiration = ExpirationDate::Days(60.0);
        let implied_volatility = 0.18;
        let risk_free_rate = 0.05;
        let dividend_yield = 0.0;

        // Short Call
        let short_strike = pos!(5800.0);
        let short_quantity = pos!(2.0);
        let premium_short = 53.04;
        let open_fee_short = 0.78;
        let close_fee_short = 0.78;

        let short_call = Position::new(
            Options::new(
                OptionType::European,
                Side::Short,
                underlying_symbol.clone(),
                short_strike,
                expiration.clone(),
                implied_volatility,
                short_quantity,
                underlying_price,
                risk_free_rate,
                OptionStyle::Call,
                dividend_yield,
                None,
            ),
            premium_short,
            Utc::now(),
            open_fee_short,
            close_fee_short,
        );

        CustomStrategy::new(
            "Test Strategy".to_string(),
            "AAPL".to_string(),
            "Test Description".to_string(),
            pos!(100.0), // underlying_price
            vec![short_call],
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
        assert!(!strategy.break_even_points.is_empty());
        assert_eq!(strategy.positions.len(), 1);
    }

    #[test]
    fn test_calculate_break_even_points_single_call() {
        setup_logger();
        let strategy = create_test_strategy();
        assert_eq!(strategy.break_even_points.len(), 1);
        assert_relative_eq!(
            strategy.break_even_points[0].value(),
            5851.48,
            epsilon = strategy.epsilon
        );
    }

    #[test]
    fn test_calculate_break_even_points_single_put() {
        let mut strategy = create_test_strategy();
        let underlying_price = pos!(5780.0);
        let underlying_symbol = "SP500".to_string();
        let expiration = ExpirationDate::Days(60.0);
        let implied_volatility = 0.18;
        let risk_free_rate = 0.05;
        let dividend_yield = 0.0;

        // Long Call ITM
        let long_strike_itm = pos!(5750.0);
        let long_quantity = pos!(1.0);
        let premium_long_itm = 85.04;
        let open_fee_long = 0.78;
        let close_fee_long = 0.78;

        let position = Position::new(
            Options::new(
                OptionType::European,
                Side::Long,
                underlying_symbol.clone(),
                long_strike_itm,
                expiration.clone(),
                implied_volatility,
                long_quantity,
                underlying_price,
                risk_free_rate,
                OptionStyle::Call,
                dividend_yield,
                None,
            ),
            premium_long_itm,
            Utc::now(),
            open_fee_long,
            close_fee_long,
        );

        strategy.add_leg(position);
        assert_eq!(strategy.break_even_points.len(), 1);
        assert_relative_eq!(
            strategy.break_even_points[0].value(),
            5866.36,
            epsilon = strategy.epsilon
        );
    }

    #[test]
    fn test_calculate_break_even_points_straddle() {
        setup_logger();

        let mut strategy = create_test_strategy();
        let underlying_price = pos!(5780.0);
        let underlying_symbol = "SP500".to_string();
        let expiration = ExpirationDate::Days(60.0);
        let implied_volatility = 0.18;
        let risk_free_rate = 0.05;
        let dividend_yield = 0.0;

        // Long Call ITM
        let long_strike_itm = pos!(5750.0);
        let long_quantity = pos!(1.0);
        let premium_long_itm = 85.04;
        let open_fee_long = 0.78;
        let close_fee_long = 0.78;

        // Long Call OTM
        let long_strike_otm = pos!(5850.0);
        let premium_long_otm = 31.65;
        let position = Position::new(
            Options::new(
                OptionType::European,
                Side::Long,
                underlying_symbol.clone(),
                long_strike_itm,
                expiration.clone(),
                implied_volatility,
                long_quantity,
                underlying_price,
                risk_free_rate,
                OptionStyle::Call,
                dividend_yield,
                None,
            ),
            premium_long_itm,
            Utc::now(),
            open_fee_long,
            close_fee_long,
        );

        strategy.add_leg(position);

        let position = Position::new(
            Options::new(
                OptionType::European,
                Side::Long,
                underlying_symbol.clone(),
                long_strike_otm,
                expiration.clone(),
                implied_volatility,
                long_quantity,
                underlying_price,
                risk_free_rate,
                OptionStyle::Call,
                dividend_yield,
                None,
            ),
            premium_long_otm,
            Utc::now(),
            open_fee_long,
            close_fee_long,
        );
        strategy.add_leg(position);

        assert_eq!(strategy.positions.len(), 3);
        assert_eq!(strategy.break_even_points.len(), 2);
        assert_relative_eq!(
            strategy.break_even_points[0].value(),
            5766.85,
            epsilon = strategy.epsilon
        );
        assert_relative_eq!(
            strategy.break_even_points[1].value(),
            5833.15,
            epsilon = strategy.epsilon
        );
    }

    #[test]
    fn test_calculate_break_even_points_no_break_even() {
        let mut strategy = create_test_strategy();
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            pos!(100.0), // underlying_price
            pos!(1.0),   // quantity
            pos!(100.0), // strike_price
            0.2,         // volatility
        );
        strategy
            .positions
            .push(Position::new(option, 5.0, Default::default(), 0.0, 0.0));
        strategy.calculate_break_even_points();

        assert_eq!(strategy.break_even_points.len(), 1);
    }
}

#[cfg(test)]
mod tests_max_profit {
    use super::*;
    use crate::model::option::Options;
    use crate::model::types::{ExpirationDate, OptionType, PositiveF64};
    use crate::model::types::{OptionStyle, Side};
    use crate::pos;
    use crate::utils::logger::setup_logger;
    use chrono::Utc;

    fn create_test_strategy() -> CustomStrategy {
        let underlying_price = pos!(5780.0);
        let underlying_symbol = "SP500".to_string();
        let expiration = ExpirationDate::Days(60.0);
        let implied_volatility = 0.18;
        let risk_free_rate = 0.05;
        let dividend_yield = 0.0;

        // Short Call
        let short_strike = pos!(5800.0);
        let short_quantity = pos!(2.0);
        let premium_short = 53.04;
        let open_fee_short = 0.78;
        let close_fee_short = 0.78;

        let short_call = Position::new(
            Options::new(
                OptionType::European,
                Side::Short,
                underlying_symbol.clone(),
                short_strike,
                expiration.clone(),
                implied_volatility,
                short_quantity,
                underlying_price,
                risk_free_rate,
                OptionStyle::Call,
                dividend_yield,
                None,
            ),
            premium_short,
            Utc::now(),
            open_fee_short,
            close_fee_short,
        );

        CustomStrategy::new(
            "Test Strategy".to_string(),
            "AAPL".to_string(),
            "Test Description".to_string(),
            pos!(100.0), // underlying_price
            vec![short_call],
            1e-16, // epsilon
            1000,  // max_iterations
            0.1,   // step_by
        )
    }

    #[test]
    fn test_max_profit_single_long_call() {
        let mut strategy = create_test_strategy();
        let max_profit = strategy.max_profit_iter();
        assert!(max_profit > 0.0);
    }

    #[test]
    fn test_max_profit_multi_leg_strategy() {
        setup_logger();

        let mut strategy = create_test_strategy();
        let underlying_price = pos!(5780.0);
        let underlying_symbol = "SP500".to_string();
        let expiration = ExpirationDate::Days(60.0);
        let implied_volatility = 0.18;
        let risk_free_rate = 0.05;
        let dividend_yield = 0.0;

        // Long Call ITM
        let long_strike_itm = pos!(5750.0);
        let long_quantity = pos!(1.0);
        let premium_long_itm = 85.04;
        let open_fee_long = 0.78;
        let close_fee_long = 0.78;

        // Long Call OTM
        let long_strike_otm = pos!(5850.0);
        let premium_long_otm = 31.65;
        let position = Position::new(
            Options::new(
                OptionType::European,
                Side::Long,
                underlying_symbol.clone(),
                long_strike_itm,
                expiration.clone(),
                implied_volatility,
                long_quantity,
                underlying_price,
                risk_free_rate,
                OptionStyle::Call,
                dividend_yield,
                None,
            ),
            premium_long_itm,
            Utc::now(),
            open_fee_long,
            close_fee_long,
        );

        strategy.add_leg(position);

        let position = Position::new(
            Options::new(
                OptionType::European,
                Side::Long,
                underlying_symbol.clone(),
                long_strike_otm,
                expiration.clone(),
                implied_volatility,
                long_quantity,
                underlying_price,
                risk_free_rate,
                OptionStyle::Call,
                dividend_yield,
                None,
            ),
            premium_long_otm,
            Utc::now(),
            open_fee_long,
            close_fee_long,
        );
        strategy.add_leg(position);

        let max_profit = strategy.max_profit_iter();
        assert!(max_profit > 0.0);
    }
}

#[cfg(test)]
mod tests_max_loss {
    use super::*;
    use crate::model::option::Options;
    use crate::model::types::{ExpirationDate, OptionType, PositiveF64};
    use crate::model::types::{OptionStyle, Side};
    use crate::pos;
    use crate::utils::logger::setup_logger;
    use chrono::Utc;

    fn create_test_strategy() -> CustomStrategy {
        let underlying_price = pos!(5780.0);
        let underlying_symbol = "SP500".to_string();
        let expiration = ExpirationDate::Days(60.0);
        let implied_volatility = 0.18;
        let risk_free_rate = 0.05;
        let dividend_yield = 0.0;

        // Short Call
        let short_strike = pos!(5800.0);
        let short_quantity = pos!(2.0);
        let premium_short = 53.04;
        let open_fee_short = 0.78;
        let close_fee_short = 0.78;

        let short_call = Position::new(
            Options::new(
                OptionType::European,
                Side::Short,
                underlying_symbol.clone(),
                short_strike,
                expiration.clone(),
                implied_volatility,
                short_quantity,
                underlying_price,
                risk_free_rate,
                OptionStyle::Call,
                dividend_yield,
                None,
            ),
            premium_short,
            Utc::now(),
            open_fee_short,
            close_fee_short,
        );

        CustomStrategy::new(
            "Test Strategy".to_string(),
            "AAPL".to_string(),
            "Test Description".to_string(),
            pos!(100.0), // underlying_price
            vec![short_call],
            1e-16, // epsilon
            1000,  // max_iterations
            0.1,   // step_by
        )
    }

    #[test]
    fn test_max_loss_single_long_call() {
        let mut strategy = create_test_strategy();
        let max_loss = strategy.max_loss_iter();
        assert!(max_loss < ZERO);
    }

    #[test]
    fn test_max_loss_multi_leg_strategy() {
        setup_logger();

        let mut strategy = create_test_strategy();
        let underlying_price = pos!(5780.0);
        let underlying_symbol = "SP500".to_string();
        let expiration = ExpirationDate::Days(60.0);
        let implied_volatility = 0.18;
        let risk_free_rate = 0.05;
        let dividend_yield = 0.0;

        // Long Call ITM
        let long_strike_itm = pos!(5750.0);
        let long_quantity = pos!(1.0);
        let premium_long_itm = 85.04;
        let open_fee_long = 0.78;
        let close_fee_long = 0.78;

        // Long Call OTM
        let long_strike_otm = pos!(5850.0);
        let premium_long_otm = 31.65;
        let position = Position::new(
            Options::new(
                OptionType::European,
                Side::Long,
                underlying_symbol.clone(),
                long_strike_itm,
                expiration.clone(),
                implied_volatility,
                long_quantity,
                underlying_price,
                risk_free_rate,
                OptionStyle::Call,
                dividend_yield,
                None,
            ),
            premium_long_itm,
            Utc::now(),
            open_fee_long,
            close_fee_long,
        );

        strategy.add_leg(position);

        let position = Position::new(
            Options::new(
                OptionType::European,
                Side::Long,
                underlying_symbol.clone(),
                long_strike_otm,
                expiration.clone(),
                implied_volatility,
                long_quantity,
                underlying_price,
                risk_free_rate,
                OptionStyle::Call,
                dividend_yield,
                None,
            ),
            premium_long_otm,
            Utc::now(),
            open_fee_long,
            close_fee_long,
        );
        strategy.add_leg(position);

        let max_loss = strategy.max_loss_iter();
        assert!(max_loss < ZERO);
    }
}
