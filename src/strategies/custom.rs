/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2/10/24
******************************************************************************/
use crate::chains::chain::{OptionChain, OptionData};
use crate::constants::{DARK_BLUE, DARK_GREEN, ZERO};
use crate::error::position::PositionError;
use crate::error::strategies::{OperationErrorKind, StrategyError};
use crate::greeks::equations::{Greek, Greeks};
use crate::model::position::Position;
use crate::pricing::payoff::Profit;
use crate::strategies::base::{Optimizable, Positionable, Strategies, StrategyType, Validable};
use crate::strategies::utils::{FindOptimalSide, OptimizationCriteria};
use crate::utils::others::process_n_times_iter;
use crate::visualization::model::{ChartPoint, ChartVerticalLine, LabelOffsetType};
use crate::visualization::utils::Graph;
use crate::{f2p, Positive};
use num_traits::FromPrimitive;
use plotters::prelude::full_palette::ORANGE;
use plotters::prelude::{ShapeStyle, RED};
use rust_decimal::Decimal;
use tracing::{debug, error};

#[derive(Clone, Debug)]
pub struct CustomStrategy {
    pub name: String,
    pub symbol: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<Positive>,
    pub positions: Vec<Position>,
    pub underlying_price: Positive,
    epsilon: Positive,
    max_iterations: u32,
    step_by: Positive,
    max_profit_point: Option<(Positive, f64)>,
    max_loss_point: Option<(Positive, f64)>,
}

impl CustomStrategy {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        symbol: String,
        description: String,
        underlying_price: Positive,
        positions: Vec<Position>,
        epsilon: Positive,
        max_iterations: u32,
        step_by: Positive,
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
        let _ = strategy.max_loss_iter();
        if !strategy.validate() {
            panic!("Invalid strategy");
        }
        strategy.calculate_break_even_points();
        let _ = strategy.max_profit_iter();
        strategy
    }

    fn update_positions(&mut self, new_positions: Vec<Position>) {
        self.positions = new_positions;
        if !self.validate() {
            panic!("Invalid strategy");
        }
        let _ = self.max_loss_iter();
        let _ = self.max_profit_iter();
        self.calculate_break_even_points();
    }

    fn calculate_break_even_points(&mut self) {
        self.break_even_points = Vec::new();
        let step = self.step_by;

        let mut current_price = Positive::ZERO;
        let (_, max_search_price) = self.range_to_show().unwrap();
        let mut last_profit = self.calculate_profit_at(current_price);

        while current_price < max_search_price {
            let current_profit = self.calculate_profit_at(current_price);
            if last_profit.signum() != current_profit.signum() {
                let mut low = current_price - step;
                let mut high = current_price;
                let mut iterations = 0;

                'inner: while (high - low).to_f64().abs() > self.epsilon
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
            current_price += step;
        }

        if self.break_even_points.is_empty() {
            debug!("No break-even points found");
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

impl Positionable for CustomStrategy {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        self.positions.push(position.clone());
        let _ = self.max_loss_iter();
        if !self.validate() {
            return Err(PositionError::invalid_position(
                "Strategy is not valid after adding new position",
            ));
        }
        let _ = self.max_profit_iter();
        self.calculate_break_even_points();
        Ok(())
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(self.positions.iter().collect())
    }
}

impl Strategies for CustomStrategy {
    fn get_underlying_price(&self) -> Positive {
        self.underlying_price
    }

    fn max_profit_iter(&mut self) -> Result<Positive, StrategyError> {
        if self.positions.is_empty() {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "max_loss_iter".to_string(),
                    reason: "No positions found".to_string(),
                },
            ));
        }
        let step = self.step_by;
        let mut max_profit: Positive = Positive::ZERO;
        let (mut current_price, max_search_price) = self.range_to_show()?;

        while current_price < max_search_price {
            let current_profit = self.calculate_profit_at(current_price);
            if current_profit < ZERO {
                current_price += step;
                continue;
            }
            if current_profit > max_profit {
                max_profit = f2p!(current_profit);
                self.max_profit_point = Some((current_price, current_profit));
            }
            current_price += step;
        }
        Ok(max_profit)
    }

    fn max_loss_iter(&mut self) -> Result<Positive, StrategyError> {
        if self.positions.is_empty() {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "max_loss_iter".to_string(),
                    reason: "No positions found".to_string(),
                },
            ));
        }
        let step = self.step_by;
        let mut max_loss: Positive = Positive::ZERO;
        let (mut current_price, max_search_price) = self.range_to_show()?;
        while current_price < max_search_price {
            let current_profit = self.calculate_profit_at(current_price);
            if current_profit > ZERO {
                current_price += step;
                continue;
            }
            if current_profit.abs() > max_loss {
                max_loss = f2p!(current_profit.abs());
                self.max_loss_point = Some((current_price, current_profit));
            }
            current_price += step;
        }
        debug!("Max Loss: {:.2}", max_loss);
        Ok(max_loss)
    }

    fn total_cost(&self) -> Positive {
        if self.positions.is_empty() {
            return Positive::ZERO;
        }
        self.positions.iter().map(Position::total_cost).sum()
    }

    fn net_premium_received(&self) -> Result<Decimal, StrategyError> {
        let restult = self
            .positions
            .iter()
            .map(|position| position.net_premium_received())
            .sum::<f64>();
        Ok(Decimal::from_f64(restult).unwrap())
    }

    fn fees(&self) -> Result<Decimal, StrategyError> {
        let restult = self
            .positions
            .iter()
            .map(|position| position.open_fee + position.close_fee)
            .sum::<f64>();
        Ok(Decimal::from_f64(restult).unwrap())
    }

    fn profit_area(&self) -> Result<Decimal, StrategyError> {
        if self.positions.is_empty() {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "profit_area".to_string(),
                    reason: "No positions found".to_string(),
                },
            ));
        }

        let mut total_profit: f64 = ZERO;
        let (mut current_price, max_search_price) = self.range_to_show()?;
        while current_price < max_search_price {
            let current_profit = self.calculate_profit_at(current_price);
            if current_profit > ZERO {
                total_profit += current_profit;
            }
            current_price += self.step_by;
        }
        let restult = total_profit / self.underlying_price.to_f64();
        Ok(Decimal::from_f64(restult).unwrap())
    }

    fn profit_ratio(&self) -> Result<Decimal, StrategyError> {
        match (self.max_profit_point, self.max_loss_point) {
            (None, _) => Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "profit_ratio".to_string(),
                    reason: "Max profit point not found".to_string(),
                },
            )),
            (_, None) => Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "profit_ratio".to_string(),
                    reason: "Max loss point not found".to_string(),
                },
            )),
            (Some(max_profit_point), Some(max_loss_point)) => {
                let result = (max_profit_point.1 / max_loss_point.1).abs() * 100.0;
                Ok(Decimal::from_f64(result).unwrap())
            }
        }
    }

    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
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
        option_chain: &OptionChain,
        side: FindOptimalSide,
        criteria: OptimizationCriteria,
    ) {
        let positions = self.positions.clone();
        let options: Vec<&OptionData> = option_chain.filter_option_data(side);

        let mut best_value = Decimal::MIN;
        let mut best_positions = positions.clone();

        debug!("Starting optimization with {} positions", positions.len());

        let _result = process_n_times_iter(&options, positions.len(), |combination| {
            let mut current_positions = positions.clone();

            // Update each position with the new data
            for (position, option_data) in current_positions.iter_mut().zip(combination.iter()) {
                position.update_from_option_data(option_data)
            }

            // check if the positions are valid
            for position in current_positions.iter() {
                if !position.validate() {
                    debug!("Invalid position found");
                    return vec![];
                }
            }

            // Evaluate the current combination
            self.update_positions(current_positions.clone());
            let current_value = match criteria {
                OptimizationCriteria::Ratio => self.profit_ratio().unwrap(),
                OptimizationCriteria::Area => self.profit_area().unwrap(),
            };

            if current_value > best_value {
                debug!("Found better value: {} > {}", current_value, best_value);
                best_value = current_value;
                best_positions = current_positions.clone();
            }

            best_positions.clone()
        })
        .unwrap();

        if best_value == Decimal::MIN {
            error!("No valid combinations found");
        }

        debug!("Optimization completed. Best value: {}", best_value);
        self.update_positions(best_positions);
    }
}

impl Profit for CustomStrategy {
    fn calculate_profit_at(&self, price: Positive) -> f64 {
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
            x_coordinate: self.underlying_price.to_f64(),
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
                coordinates: (point.to_f64(), 0.0),
                label: format!("Break Even {:.2}", point),
                label_offset: LabelOffsetType::Relative(-26.0, 2.0),
                point_color: DARK_BLUE,
                label_color: DARK_BLUE,
                point_size: 5,
                font_size: 18,
            });
        }

        points.push(self.get_point_at_price(self.underlying_price));

        points.push(ChartPoint {
            coordinates: (
                self.max_profit_point.unwrap().0.to_f64(),
                self.max_profit_point.unwrap().1,
            ),
            label: format!("Max Profit {:.2}", self.max_profit_point.unwrap().1),
            label_offset: LabelOffsetType::Relative(2.0, 1.0),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (
                self.max_loss_point.unwrap().0.to_f64(),
                self.max_loss_point.unwrap().1,
            ),
            label: format!("Max Loss {:.2}", self.max_loss_point.unwrap().1),
            label_offset: LabelOffsetType::Relative(-30.0, 2.0),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        points
    }
}

impl Greeks for CustomStrategy {
    fn greeks(&self) -> Greek {
        let mut greek = Greek {
            delta: Decimal::ZERO,
            gamma: Decimal::ZERO,
            theta: Decimal::ZERO,
            vega: Decimal::ZERO,
            rho: Decimal::ZERO,
            rho_d: Decimal::ZERO,
        };
        for position in self.positions.iter() {
            let current_greek = position.greeks();
            greek.delta += current_greek.delta;
            greek.gamma += current_greek.gamma;
            greek.theta += current_greek.theta;
            greek.vega += current_greek.vega;
            greek.rho += current_greek.rho;
            greek.rho_d += current_greek.rho_d;
        }
        greek
    }
}

#[cfg(test)]
mod tests_custom_strategy {
    use super::*;
    use crate::f2p;
    use crate::model::types::{ExpirationDate, OptionType};
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option;
    use crate::utils::logger::setup_logger;
    use crate::Options;
    use approx::assert_relative_eq;
    use chrono::Utc;

    fn create_test_strategy() -> CustomStrategy {
        let underlying_price = f2p!(5780.0);
        let underlying_symbol = "SP500".to_string();
        let expiration = ExpirationDate::Days(60.0);
        let implied_volatility = 0.18;
        let risk_free_rate = 0.05;
        let dividend_yield = 0.0;

        // Short Call
        let short_strike = f2p!(5800.0);
        let short_quantity = f2p!(2.0);
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
            f2p!(100.0), // underlying_price
            vec![short_call],
            f2p!(1e-2), // epsilon
            2,          // max_iterations
            f2p!(0.1),  // step_by
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
            strategy.break_even_points[0].to_f64(),
            5851.48,
            epsilon = strategy.epsilon.to_f64()
        );
    }

    #[test]
    fn test_calculate_break_even_points_single_put() {
        let mut strategy = create_test_strategy();
        let underlying_price = f2p!(5780.0);
        let underlying_symbol = "SP500".to_string();
        let expiration = ExpirationDate::Days(60.0);
        let implied_volatility = 0.18;
        let risk_free_rate = 0.05;
        let dividend_yield = 0.0;

        // Long Call ITM
        let long_strike_itm = f2p!(5750.0);
        let long_quantity = f2p!(1.0);
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

        strategy.add_position(&position).expect("Invalid position");
        assert_eq!(strategy.break_even_points.len(), 1);
        assert_relative_eq!(
            strategy.break_even_points[0].to_f64(),
            5866.36,
            epsilon = strategy.epsilon.to_f64()
        );
    }

    #[test]
    fn test_calculate_break_even_points_straddle() {
        setup_logger();

        let mut strategy = create_test_strategy();
        let underlying_price = f2p!(5780.0);
        let underlying_symbol = "SP500".to_string();
        let expiration = ExpirationDate::Days(60.0);
        let implied_volatility = 0.18;
        let risk_free_rate = 0.05;
        let dividend_yield = 0.0;

        // Long Call ITM
        let long_strike_itm = f2p!(5750.0);
        let long_quantity = f2p!(1.0);
        let premium_long_itm = 85.04;
        let open_fee_long = 0.78;
        let close_fee_long = 0.78;

        // Long Call OTM
        let long_strike_otm = f2p!(5850.0);
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

        strategy.add_position(&position).expect("Invalid position");

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
        strategy.add_position(&position).expect("Invalid position");

        assert_eq!(strategy.positions.len(), 3);
        assert_eq!(strategy.break_even_points.len(), 2);
        assert_relative_eq!(
            strategy.break_even_points[0].to_f64(),
            5766.85,
            epsilon = strategy.epsilon.to_f64()
        );
        assert_relative_eq!(
            strategy.break_even_points[1].to_f64(),
            5833.15,
            epsilon = strategy.epsilon.to_f64()
        );
    }

    #[test]
    fn test_calculate_break_even_points_no_get_break_even_points() {
        let mut strategy = create_test_strategy();
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            f2p!(100.0), // underlying_price
            f2p!(1.0),   // quantity
            f2p!(100.0), // strike_price
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
    use crate::f2p;
    use crate::model::types::{ExpirationDate, OptionType};
    use crate::model::types::{OptionStyle, Side};
    use crate::utils::logger::setup_logger;
    use crate::Options;
    use chrono::Utc;

    fn create_test_strategy() -> CustomStrategy {
        let underlying_price = f2p!(5780.0);
        let underlying_symbol = "SP500".to_string();
        let expiration = ExpirationDate::Days(60.0);
        let implied_volatility = 0.18;
        let risk_free_rate = 0.05;
        let dividend_yield = 0.0;

        // Short Call
        let short_strike = f2p!(5800.0);
        let short_quantity = f2p!(2.0);
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
            f2p!(100.0), // underlying_price
            vec![short_call],
            f2p!(1e-2), // epsilon
            10,         // max_iterations
            f2p!(0.1),  // step_by
        )
    }

    #[test]
    fn test_max_profit_single_long_call() {
        let mut strategy = create_test_strategy();
        let max_profit = strategy.max_profit_iter().unwrap();
        assert!(max_profit > Positive::ZERO);
    }

    #[test]
    fn test_max_profit_multi_leg_strategy() {
        setup_logger();

        let mut strategy = create_test_strategy();
        let underlying_price = f2p!(5780.0);
        let underlying_symbol = "SP500".to_string();
        let expiration = ExpirationDate::Days(60.0);
        let implied_volatility = 0.18;
        let risk_free_rate = 0.05;
        let dividend_yield = 0.0;

        // Long Call ITM
        let long_strike_itm = f2p!(5750.0);
        let long_quantity = f2p!(1.0);
        let premium_long_itm = 85.04;
        let open_fee_long = 0.78;
        let close_fee_long = 0.78;

        // Long Call OTM
        let long_strike_otm = f2p!(5850.0);
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

        strategy.add_position(&position).expect("Invalid position");

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
        strategy.add_position(&position).expect("Invalid position");

        let max_profit = strategy.max_profit_iter().unwrap();
        assert!(max_profit > Positive::ZERO);
    }
}

#[cfg(test)]
mod tests_max_loss {
    use super::*;
    use crate::f2p;
    use crate::model::types::{ExpirationDate, OptionType};
    use crate::model::types::{OptionStyle, Side};
    use crate::utils::logger::setup_logger;
    use crate::Options;
    use chrono::Utc;

    fn create_test_strategy() -> CustomStrategy {
        let underlying_price = f2p!(5780.0);
        let underlying_symbol = "SP500".to_string();
        let expiration = ExpirationDate::Days(60.0);
        let implied_volatility = 0.18;
        let risk_free_rate = 0.05;
        let dividend_yield = 0.0;

        // Short Call
        let short_strike = f2p!(5800.0);
        let short_quantity = f2p!(2.0);
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
            f2p!(100.0), // underlying_price
            vec![short_call],
            f2p!(1e-16), // epsilon
            1000,        // max_iterations
            f2p!(0.1),   // step_by
        )
    }

    #[test]
    fn test_max_loss_single_long_call() {
        let mut strategy = create_test_strategy();
        let max_loss = strategy.max_loss_iter().unwrap();
        assert!(max_loss > Positive::ZERO);
    }

    #[test]
    fn test_max_loss_multi_leg_strategy() {
        setup_logger();

        let mut strategy = create_test_strategy();
        let underlying_price = f2p!(5780.0);
        let underlying_symbol = "SP500".to_string();
        let expiration = ExpirationDate::Days(60.0);
        let implied_volatility = 0.18;
        let risk_free_rate = 0.05;
        let dividend_yield = 0.0;

        // Long Call ITM
        let long_strike_itm = f2p!(5750.0);
        let long_quantity = f2p!(1.0);
        let premium_long_itm = 85.04;
        let open_fee_long = 0.78;
        let close_fee_long = 0.78;

        // Long Call OTM
        let long_strike_otm = f2p!(5850.0);
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

        strategy.add_position(&position).expect("Invalid position");

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
        strategy.add_position(&position).expect("Invalid position");

        let max_loss = strategy.max_loss_iter().unwrap();
        assert!(max_loss > Positive::ZERO);
    }
}

#[cfg(test)]
mod tests_total_cost {
    use super::*;
    use crate::f2p;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
    use crate::utils::setup_logger;
    use crate::Options;
    use chrono::Utc;

    fn create_test_position(side: Side, premium: f64, fees: f64) -> Position {
        Position::new(
            Options::new(
                OptionType::European,
                side,
                "TEST".to_string(),
                f2p!(100.0),
                ExpirationDate::Days(30.0),
                0.2,
                f2p!(1.0),
                f2p!(100.0),
                0.01,
                OptionStyle::Call,
                0.0,
                None,
            ),
            premium,
            Utc::now(),
            fees, // open fees
            fees, // closing fees
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
            f2p!(100.0),
            positions,
            f2p!(0.001),
            100,
            f2p!(1.0),
        );

        assert_eq!(strategy.total_cost(), 10.0); // 6.0 + 4.0
    }

    #[test]
    fn test_total_cost_only_short_positions() {
        setup_logger();
        let position_1 = create_test_position(Side::Short, 5.0, 0.5);
        let position_2 = create_test_position(Side::Short, 3.0, 0.5);

        assert_eq!(position_1.total_cost(), 1.0);
        assert_eq!(position_2.total_cost(), 1.0);

        let positions = vec![position_1, position_2];

        let strategy = CustomStrategy::new(
            "Test".to_string(),
            "TEST".to_string(),
            "Test description".to_string(),
            f2p!(120.0),
            positions,
            f2p!(0.001),
            10,
            f2p!(1.0),
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
            f2p!(100.0),
            positions,
            f2p!(0.001),
            100,
            f2p!(1.0),
        );

        assert_eq!(strategy.total_cost(), 13.5); // 6.0 + 1.0 + 5.0 + 1.5
    }

    #[test]
    fn test_total_cost_with_different_premiums_and_fees() {
        let positions = vec![
            create_test_position(Side::Long, 10.0, 1.0),
            create_test_position(Side::Short, 5.0, 0.5),
            create_test_position(Side::Short, 3.0, 0.25),
            create_test_position(Side::Long, 7.0, 0.75),
        ];

        let strategy = CustomStrategy::new(
            "Test".to_string(),
            "TEST".to_string(),
            "Test description".to_string(),
            f2p!(100.0),
            positions,
            f2p!(0.001),
            100,
            f2p!(1.0),
        );
        assert_eq!(strategy.total_cost(), 22.0); // 12.0 + 1.0 + 8.5 + 0.5
    }
}

#[cfg(test)]
mod tests_best_range_to_show {
    use super::*;
    use crate::constants::STRIKE_PRICE_LOWER_BOUND_MULTIPLIER;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
    use crate::Options;
    use chrono::Utc;

    fn create_test_position(strike: Positive, side: Side) -> Position {
        Position::new(
            Options::new(
                OptionType::European,
                side,
                "SP500".to_string(),
                strike,
                ExpirationDate::Days(60.0),
                0.18,
                f2p!(2.0),
                f2p!(5780.0),
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

    fn create_test_strategy_with_strikes(strikes: Vec<Positive>) -> CustomStrategy {
        let positions: Vec<Position> = strikes
            .into_iter()
            .map(|strike| create_test_position(strike, Side::Short))
            .collect();

        CustomStrategy::new(
            "Test Strategy".to_string(),
            "SP500".to_string(),
            "Test Description".to_string(),
            f2p!(5780.0),
            positions,
            f2p!(1e-16),
            1000,
            f2p!(0.1),
        )
    }

    #[test]
    fn test_best_range_single_strike() {
        let strategy = create_test_strategy_with_strikes(vec![f2p!(5800.0)]);
        let step = f2p!(10.0);
        let range = strategy.best_range_to_show(step).unwrap();

        assert_eq!(range.first().unwrap().to_f64(), 5644.8);
        assert_eq!(range.last().unwrap().to_f64(), 5974.8);

        // Check step size
        for i in 0..range.len() - 1 {
            assert_eq!((range[i + 1] - range[i]).to_f64(), step.to_f64());
        }
    }

    #[test]
    fn test_best_range_multiple_strikes() {
        let strategy =
            create_test_strategy_with_strikes(vec![f2p!(5700.0), f2p!(5800.0), f2p!(5900.0)]);
        let step = f2p!(50.0);
        let range = strategy.best_range_to_show(step).unwrap();

        assert_eq!(range.first().unwrap().to_f64(), 5546.8);
        assert_eq!(range.last().unwrap().to_f64(), 6046.8);

        // Verify step size
        for i in 0..range.len() - 1 {
            assert_eq!((range[i + 1] - range[i]).to_f64(), step.to_f64());
        }
    }

    #[test]
    fn test_best_range_with_small_step() {
        let strategy = create_test_strategy_with_strikes(vec![f2p!(5800.0), f2p!(5850.0)]);
        let step = f2p!(5.0);
        let range = strategy.best_range_to_show(step).unwrap();

        // Verify granular steps
        for i in 0..range.len() - 1 {
            assert_eq!((range[i + 1] - range[i]).to_f64(), step.to_f64());
        }
    }

    #[test]
    fn test_best_range_with_underlying() {
        let strategy = create_test_strategy_with_strikes(vec![f2p!(5700.0), f2p!(5900.0)]);
        let range = strategy.best_range_to_show(f2p!(10.0)).unwrap();

        // Verify range includes underlying price (5780.0)
        assert!(range.iter().any(|&price| price <= f2p!(5780.0)));
        assert!(range.iter().any(|&price| price >= f2p!(5780.0)));
    }

    #[test]
    fn test_best_range_with_large_step() {
        let strategy = create_test_strategy_with_strikes(vec![f2p!(5600.0), f2p!(6000.0)]);
        let step = f2p!(100.0);
        let range = strategy.best_range_to_show(step).unwrap();

        // Verify minimum points
        assert!(range.len() >= 3);

        // Verify step size
        for i in 0..range.len() - 1 {
            assert_eq!((range[i + 1] - range[i]).to_f64(), step.to_f64());
        }
    }

    #[test]
    fn test_best_range_strike_bounds() {
        let min_strike = f2p!(5600.0);
        let max_strike = f2p!(6000.0);
        let strategy = create_test_strategy_with_strikes(vec![min_strike, max_strike]);
        let range = strategy.best_range_to_show(f2p!(50.0)).unwrap();

        let expected_min = (min_strike * STRIKE_PRICE_LOWER_BOUND_MULTIPLIER).to_f64();

        assert!(range.first().unwrap().to_f64() <= expected_min);
        assert!(range.last().unwrap().to_f64() >= max_strike.to_f64());
    }

    #[test]
    fn test_best_range_unordered_strikes() {
        let strategy =
            create_test_strategy_with_strikes(vec![f2p!(5600.0), f2p!(5700.0), f2p!(5100.0)]);
        let range = strategy.best_range_to_show(f2p!(50.0)).unwrap();

        assert_eq!(range.first().unwrap().to_f64(), 4998.0);
        assert_eq!(range.last().unwrap().to_f64(), 6598.0);
    }
}

#[cfg(test)]
mod tests_greeks {
    use super::*;
    use crate::assert_decimal_eq;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
    use crate::utils::setup_logger;
    use crate::Options;
    use chrono::Utc;
    use rust_decimal_macros::dec;

    const EPSILON: Decimal = dec!(1e-10);

    // Helper function to create a test position
    fn create_test_position(strike: Positive, side: Side, option_style: OptionStyle) -> Position {
        Position::new(
            Options::new(
                OptionType::European,
                side,
                "TEST".to_string(),
                strike,
                ExpirationDate::Days(30.0),
                0.2,         // volatility
                f2p!(1.0),   // quantity
                f2p!(100.0), // underlying_price
                0.05,        // risk_free_rate
                option_style,
                0.02, // dividend_yield
                None,
            ),
            10.0, // premium
            Utc::now(),
            1.0, // open_fee
            1.0, // close_fee
        )
    }

    #[test]
    fn test_greeks_single_long_call() {
        let position = create_test_position(f2p!(100.0), Side::Long, OptionStyle::Call);
        let strategy = CustomStrategy::new(
            "Long Call".to_string(),
            "TEST".to_string(),
            "Test Description".to_string(),
            f2p!(100.0),
            vec![position.clone()],
            f2p!(0.001),
            100,
            Positive::ONE,
        );

        let strategy_greeks = strategy.greeks();
        let position_greeks = position.greeks();

        assert_decimal_eq!(strategy_greeks.delta, position_greeks.delta, EPSILON);
        assert_decimal_eq!(strategy_greeks.gamma, position_greeks.gamma, EPSILON);
        assert_decimal_eq!(strategy_greeks.theta, position_greeks.theta, EPSILON);
        assert_decimal_eq!(strategy_greeks.vega, position_greeks.vega, EPSILON);
        assert_decimal_eq!(strategy_greeks.rho, position_greeks.rho, EPSILON);
        assert_decimal_eq!(strategy_greeks.rho_d, position_greeks.rho_d, EPSILON);
    }

    #[test]
    fn test_greeks_single_short_put() {
        setup_logger();
        let position = create_test_position(f2p!(100.0), Side::Short, OptionStyle::Put);
        let strategy = CustomStrategy::new(
            "Short Put".to_string(),
            "TEST".to_string(),
            "Test Description".to_string(),
            f2p!(90.0),
            vec![position.clone()],
            f2p!(0.001),
            100,
            Positive::ONE,
        );

        let strategy_greeks = strategy.greeks();
        let position_greeks = position.greeks();

        assert_decimal_eq!(strategy_greeks.delta, position_greeks.delta, EPSILON);
        assert_decimal_eq!(strategy_greeks.gamma, position_greeks.gamma, EPSILON);
        assert_decimal_eq!(strategy_greeks.theta, position_greeks.theta, EPSILON);
        assert_decimal_eq!(strategy_greeks.vega, position_greeks.vega, EPSILON);
        assert_decimal_eq!(strategy_greeks.rho, position_greeks.rho, EPSILON);
        assert_decimal_eq!(strategy_greeks.rho_d, position_greeks.rho_d, EPSILON);
    }

    #[test]
    fn test_greeks_multiple_positions() {
        let long_call = create_test_position(f2p!(100.0), Side::Long, OptionStyle::Call);
        let short_put = create_test_position(f2p!(95.0), Side::Short, OptionStyle::Put);
        let long_put = create_test_position(f2p!(105.0), Side::Long, OptionStyle::Put);

        let strategy = CustomStrategy::new(
            "Multiple".to_string(),
            "TEST".to_string(),
            "Test Description".to_string(),
            f2p!(100.0),
            vec![long_call.clone(), short_put.clone(), long_put.clone()],
            f2p!(0.001),
            100,
            Positive::ONE,
        );

        let strategy_greeks = strategy.greeks();
        let long_call_greeks = long_call.greeks();
        let short_put_greeks = short_put.greeks();
        let long_put_greeks = long_put.greeks();

        assert_decimal_eq!(
            strategy_greeks.delta,
            long_call_greeks.delta + short_put_greeks.delta + long_put_greeks.delta,
            EPSILON
        );
        assert_decimal_eq!(
            strategy_greeks.gamma,
            long_call_greeks.gamma + short_put_greeks.gamma + long_put_greeks.gamma,
            EPSILON
        );
        assert_decimal_eq!(
            strategy_greeks.theta,
            long_call_greeks.theta + short_put_greeks.theta + long_put_greeks.theta,
            EPSILON
        );
        assert_decimal_eq!(
            strategy_greeks.vega,
            long_call_greeks.vega + short_put_greeks.vega + long_put_greeks.vega,
            EPSILON
        );
        assert_decimal_eq!(
            strategy_greeks.rho,
            long_call_greeks.rho + short_put_greeks.rho + long_put_greeks.rho,
            EPSILON
        );
        assert_decimal_eq!(
            strategy_greeks.rho_d,
            long_call_greeks.rho_d + short_put_greeks.rho_d + long_put_greeks.rho_d,
            EPSILON
        );
    }

    #[test]
    fn test_greeks_straddle() {
        let long_call = create_test_position(f2p!(100.0), Side::Long, OptionStyle::Call);
        let long_put = create_test_position(f2p!(100.0), Side::Long, OptionStyle::Put);

        let strategy = CustomStrategy::new(
            "Straddle".to_string(),
            "TEST".to_string(),
            "Test Description".to_string(),
            f2p!(100.0),
            vec![long_call.clone(), long_put.clone()],
            f2p!(0.001),
            100,
            Positive::ONE,
        );

        let strategy_greeks = strategy.greeks();
        let call_greeks = long_call.greeks();
        let put_greeks = long_put.greeks();

        // Straddle specific assertions
        assert_decimal_eq!(
            strategy_greeks.delta,
            call_greeks.delta + put_greeks.delta,
            EPSILON
        );
        // Gamma should be positive and roughly double the individual option's gamma
        assert!(strategy_greeks.gamma > Decimal::ZERO);
        assert_decimal_eq!(
            strategy_greeks.gamma,
            call_greeks.gamma + put_greeks.gamma,
            EPSILON
        );
        // Theta should be negative for long straddle
        assert!(strategy_greeks.theta < Decimal::ZERO);
        // Vega should be positive and roughly double the individual option's vega
        assert!(strategy_greeks.vega > Decimal::ZERO);
    }
}
