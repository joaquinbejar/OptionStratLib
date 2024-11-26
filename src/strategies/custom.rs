/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2/10/24
******************************************************************************/

use crate::chains::chain::{OptionChain, OptionData};
use crate::constants::{
    DARK_BLUE, DARK_GREEN, STRIKE_PRICE_LOWER_BOUND_MULTIPLIER,
    STRIKE_PRICE_UPPER_BOUND_MULTIPLIER, ZERO,
};
use crate::model::position::Position;
use crate::model::types::{PositiveF64, PZERO};
use crate::pos;
use crate::pricing::payoff::Profit;
use crate::strategies::base::{Optimizable, Strategies, StrategyType, Validable};
use crate::strategies::utils::{calculate_price_range, FindOptimalSide, OptimizationCriteria};
use crate::utils::others::process_n_times_iter;
use crate::visualization::model::{ChartPoint, ChartVerticalLine};
use crate::visualization::utils::Graph;
use plotters::prelude::full_palette::ORANGE;
use plotters::prelude::{ShapeStyle, RED};
use tracing::{debug, error, info};

#[derive(Clone, Debug)]
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

    fn update_positions(&mut self, new_positions: Vec<Position>) {
        self.positions = new_positions;
        if !self.validate() {
            panic!("Invalid strategy");
        }
        self.max_loss_iter();
        self.max_profit_iter();
        self.calculate_break_even_points();
    }

    fn calculate_break_even_points(&mut self) {
        self.break_even_points = Vec::new();
        let step = self.step_by;

        let mut current_price = PZERO;
        let (_, max_search_price) = self.range_to_show();
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
            debug!(
                "Break Even Points found: {}",
                self.break_even_points
                    .iter()
                    .map(|p| format!("{:.2}", p))
                    .collect::<Vec<String>>()
                    .join(", ")
            );
        }
    }
}

impl Strategies for CustomStrategy {
    fn get_underlying_price(&self) -> PositiveF64 {
        self.underlying_price
    }

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
        if self.positions.is_empty() {
            return ZERO;
        }
        self.positions.iter().map(Position::total_cost).sum()
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
        let mut break_even_points = self.break_even_points.clone();
        break_even_points.push(first_option);
        break_even_points.push(last_option);
        break_even_points.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let start_price = *break_even_points.first().unwrap() * STRIKE_PRICE_LOWER_BOUND_MULTIPLIER;
        let end_price = *break_even_points.last().unwrap() * STRIKE_PRICE_UPPER_BOUND_MULTIPLIER;
        Some(calculate_price_range(start_price, end_price, step))
    }

    fn get_break_even_points(&self) -> Vec<PositiveF64> {
        self.break_even_points.clone()
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

    // fn find_optimal(
    //     &mut self,
    //     option_chain: &OptionChain,
    //     _side: FindOptimalSide,
    //     criteria: OptimizationCriteria,
    // ) {
    //     let positions = self.positions.clone();
    //     let options: Vec<&OptionData> = option_chain.options.iter().collect();
    //     let mut best_value = f64::NEG_INFINITY;
    //     let mut best_positions = positions.clone();
    //
    //     let new_positions: Vec<Position> =
    //         process_n_times_iter(&options, positions.len(), |vec_option_data| {
    //             // TODO: Implement the optimization algorithm
    //             // en vec_option_data tengo un vector de OptionData de longitud igual a la cantidad de posiciones
    //             // en positions tengo las posiciones actuales que tengo que reemplazar por las nuevas
    //             // y devolver un nuevo vector de posiciones
    //             // como convierto un OptionData en Position?
    //             // si la Position original es ej: Short Call, tengo que crear una nueva Position con el mismo
    //             // Side, Symbol, Quantity, etc. pero el strike y los precios del OptionData
    //             // puedo usar el metodo get_option() de OptionData para obtener el Option y luego usar
    //             // Position::new con premium: f64,
    //             //         date: DateTime<Utc>,
    //             //         open_fee: f64,
    //             //         close_fee: f64,
    //             for (mut position, &option_data) in positions.iter().zip(vec_option_data.iter()) {
    //                 position.update_from_option_data(option_data);
    //             }
    //             self.update_positions(positions.clone());
    //             let current_value = match criteria {
    //                 OptimizationCriteria::Ratio => self.profit_ratio(),
    //                 OptimizationCriteria::Area => self.profit_area(),
    //             };
    //
    //             if current_value > best_value {
    //                 best_value = current_value;
    //                 best_positions = positions.clone();
    //             }
    //             best_positions
    //         })
    //         .unwrap();
    //     // info!("New Positions: {:?}", new_positions);
    //     let _ = new_positions
    //         .iter()
    //         .map(|position| info!("Position: {:?}", position));
    //     self.update_positions(new_positions);
    // }

    fn find_optimal(
        &mut self,
        option_chain: &OptionChain,
        side: FindOptimalSide,
        criteria: OptimizationCriteria,
    ) {
        let positions = self.positions.clone();
        let options: Vec<&OptionData> = option_chain.filter_option_data(side);

        let mut best_value = f64::NEG_INFINITY;
        let mut best_positions = positions.clone();

        debug!("Starting optimization with {} positions", positions.len());

        let result = process_n_times_iter(&options, positions.len(), |combination| {
            let mut current_positions = positions.clone();

            // Update each position with the new data
            for (position, option_data) in current_positions.iter_mut().zip(combination.iter()) {
                position.update_from_option_data(option_data)
            }

            // Evaluate the current combination
            self.update_positions(current_positions.clone());
            let current_value = match criteria {
                OptimizationCriteria::Ratio => self.profit_ratio(),
                OptimizationCriteria::Area => self.profit_area(),
            };


            if current_value > best_value {
                info!("Found better value: {} > {}", current_value, best_value);
                best_value = current_value;
                best_positions = current_positions.clone();
            } else { 
                info!("Current value: {} <= {}", current_value, best_value); 
            }

            best_positions.clone()
        })
        .unwrap();

        if best_value == f64::NEG_INFINITY {
            error!("No valid combinations found");
        }

        debug!("Optimization completed. Best value: {}", best_value);
        info!("Best positions length: {:?}", result.len());
        self.update_positions(best_positions);
    }

    fn create_strategy(
        &self,
        _chain: &OptionChain,
        _call: &OptionData,
        _put: &OptionData,
    ) -> Self::Strategy {
        panic!("Create strategy is not applicable for this strategy");
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

#[cfg(test)]
mod tests_total_cost {
    use super::*;
    use crate::model::option::Options;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
    use crate::pos;
    use chrono::Utc;

    fn create_test_position(side: Side, premium: f64, fees: f64) -> Position {
        Position::new(
            Options::new(
                OptionType::European,
                side,
                "TEST".to_string(),
                pos!(100.0),
                ExpirationDate::Days(30.0),
                0.2,
                pos!(1.0),
                pos!(100.0),
                0.01,
                OptionStyle::Call,
                0.0,
                None,
            ),
            premium,
            Utc::now(),
            fees, // open fee
            fees, // close fee
        )
    }

    #[test]
    fn test_total_cost_only_long_positions() {
        let positions = vec![
            create_test_position(Side::Long, 5.0, 0.5), // net cost = 6.0 (premium + fees)
            create_test_position(Side::Long, 3.0, 0.5), // net cost = 4.0 (premium + fees)
        ];

        let strategy = CustomStrategy::new(
            "Test".to_string(),
            "TEST".to_string(),
            "Test description".to_string(),
            pos!(100.0),
            positions,
            0.001,
            100,
            1.0,
        );

        assert_eq!(strategy.total_cost(), 10.0); // 6.0 + 4.0
    }

    #[test]
    fn test_total_cost_only_short_positions() {
        let position_1 = create_test_position(Side::Short, 5.0, 0.5);
        let position_2 = create_test_position(Side::Short, 3.0, 0.5);

        assert_eq!(position_1.total_cost(), 1.0);
        assert_eq!(position_2.total_cost(), 1.0);

        let positions = vec![position_1, position_2];

        let strategy = CustomStrategy::new(
            "Test".to_string(),
            "TEST".to_string(),
            "Test description".to_string(),
            pos!(100.0),
            positions,
            0.001,
            100,
            1.0,
        );

        assert_eq!(strategy.total_cost(), 2.0);
    }

    #[test]
    fn test_total_cost_mixed_positions() {
        let positions = vec![
            create_test_position(Side::Long, 5.0, 0.5), // net cost = 6.0
            create_test_position(Side::Short, 3.0, 0.5), // net cost = 1.0
            create_test_position(Side::Long, 4.0, 0.5), // net cost = 5.0
            create_test_position(Side::Short, 2.0, 0.75), // net cost = 1.5
        ];

        let strategy = CustomStrategy::new(
            "Test".to_string(),
            "TEST".to_string(),
            "Test description".to_string(),
            pos!(100.0),
            positions,
            0.001,
            100,
            1.0,
        );

        assert_eq!(strategy.total_cost(), 13.5); // 6.0 + 1.0 + 5.0 + 1.5
    }

    #[test]
    fn test_total_cost_with_different_premiums_and_fees() {
        let positions = vec![
            create_test_position(Side::Long, 10.0, 1.0), // net cost = 12.0
            create_test_position(Side::Short, 5.0, 0.5), // net cost = 1.0
            create_test_position(Side::Long, 7.0, 0.75), // net cost = 8.5
            create_test_position(Side::Short, 3.0, 0.25), // net cost = 0.5
        ];

        let strategy = CustomStrategy::new(
            "Test".to_string(),
            "TEST".to_string(),
            "Test description".to_string(),
            pos!(100.0),
            positions,
            0.001,
            100,
            1.0,
        );

        assert_eq!(strategy.total_cost(), 22.0); // 12.0 + 1.0 + 8.5 + 0.5
    }
}

#[cfg(test)]
mod tests_best_range_to_show {
    use super::*;
    use crate::model::option::Options;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
    use chrono::Utc;

    fn create_test_position(strike: PositiveF64, side: Side) -> Position {
        Position::new(
            Options::new(
                OptionType::European,
                side,
                "SP500".to_string(),
                strike,
                ExpirationDate::Days(60.0),
                0.18,
                pos!(2.0),
                pos!(5780.0),
                0.05,
                OptionStyle::Call,
                0.0,
                None,
            ),
            53.04,
            Utc::now(),
            0.78,
            0.78,
        )
    }

    fn create_test_strategy_with_strikes(strikes: Vec<PositiveF64>) -> CustomStrategy {
        let positions: Vec<Position> = strikes
            .into_iter()
            .map(|strike| create_test_position(strike, Side::Short))
            .collect();

        CustomStrategy::new(
            "Test Strategy".to_string(),
            "SP500".to_string(),
            "Test Description".to_string(),
            pos!(5780.0),
            positions,
            1e-16,
            1000,
            0.1,
        )
    }

    #[test]
    fn test_best_range_single_strike() {
        let strategy = create_test_strategy_with_strikes(vec![pos!(5800.0)]);
        let step = pos!(10.0);
        let range = strategy.best_range_to_show(step).unwrap();

        let expected_start = (pos!(5780.0) * STRIKE_PRICE_LOWER_BOUND_MULTIPLIER).value();

        assert_eq!(range.first().unwrap().value(), expected_start);
        assert_eq!(range.last().unwrap().value(), 5964.4);

        // Check step size
        for i in 0..range.len() - 1 {
            assert_eq!((range[i + 1] - range[i]).value(), step.value());
        }
    }

    #[test]
    fn test_best_range_multiple_strikes() {
        let strategy =
            create_test_strategy_with_strikes(vec![pos!(5700.0), pos!(5800.0), pos!(5900.0)]);
        let step = pos!(50.0);
        let range = strategy.best_range_to_show(step).unwrap();

        let expected_start = (pos!(5700.0) * STRIKE_PRICE_LOWER_BOUND_MULTIPLIER).value();

        assert_eq!(range.first().unwrap().value(), expected_start);
        assert_eq!(range.last().unwrap().value(), 5986.0);

        // Verify step size
        for i in 0..range.len() - 1 {
            assert_eq!((range[i + 1] - range[i]).value(), step.value());
        }
    }

    #[test]
    fn test_best_range_with_small_step() {
        let strategy = create_test_strategy_with_strikes(vec![pos!(5800.0), pos!(5850.0)]);
        let step = pos!(5.0);
        let range = strategy.best_range_to_show(step).unwrap();

        // Verify granular steps
        for i in 0..range.len() - 1 {
            assert_eq!((range[i + 1] - range[i]).value(), step.value());
        }
    }

    #[test]
    fn test_best_range_with_underlying() {
        let strategy = create_test_strategy_with_strikes(vec![pos!(5700.0), pos!(5900.0)]);
        let range = strategy.best_range_to_show(pos!(10.0)).unwrap();

        // Verify range includes underlying price (5780.0)
        assert!(range.iter().any(|&price| price <= pos!(5780.0)));
        assert!(range.iter().any(|&price| price >= pos!(5780.0)));
    }

    #[test]
    fn test_best_range_with_large_step() {
        let strategy = create_test_strategy_with_strikes(vec![pos!(5600.0), pos!(6000.0)]);
        let step = pos!(100.0);
        let range = strategy.best_range_to_show(step).unwrap();

        // Verify minimum points
        assert!(range.len() >= 3);

        // Verify step size
        for i in 0..range.len() - 1 {
            assert_eq!((range[i + 1] - range[i]).value(), step.value());
        }
    }

    #[test]
    fn test_best_range_strike_bounds() {
        let min_strike = pos!(5600.0);
        let max_strike = pos!(6000.0);
        let strategy = create_test_strategy_with_strikes(vec![min_strike, max_strike]);
        let range = strategy.best_range_to_show(pos!(50.0)).unwrap();

        let expected_min = (min_strike * STRIKE_PRICE_LOWER_BOUND_MULTIPLIER).value();

        assert!(range.first().unwrap().value() <= expected_min);
        assert!(range.last().unwrap().value() >= max_strike.value());
    }

    #[test]
    fn test_best_range_unordered_strikes() {
        let strategy =
            create_test_strategy_with_strikes(vec![pos!(5600.0), pos!(5700.0), pos!(5100.0)]);
        let range = strategy.best_range_to_show(pos!(50.0)).unwrap();

        let expected_start = (pos!(5100.0) * STRIKE_PRICE_LOWER_BOUND_MULTIPLIER).value();

        assert_eq!(range.first().unwrap().value(), expected_start);
        assert_eq!(range.last().unwrap().value(), 5848.0);
    }
}
