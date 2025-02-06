/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2/10/24
******************************************************************************/
use crate::chains::chain::{OptionChain, OptionData};
use crate::constants::{DARK_BLUE, DARK_GREEN, ZERO};
use crate::error::position::PositionError;
use crate::error::strategies::StrategyError;
use crate::error::{GreeksError, OperationErrorKind, ProbabilityError};
use crate::greeks::Greeks;
use crate::model::utils::mean_and_std;
use crate::model::{Position, ProfitLossRange};
use crate::pricing::payoff::Profit;
use crate::strategies::base::{
    BreakEvenable, Optimizable, Positionable, Strategies, StrategyType, Validable,
};
use crate::strategies::probabilities::{ProbabilityAnalysis, VolatilityAdjustment};
use crate::strategies::utils::{FindOptimalSide, OptimizationCriteria};
use crate::utils::others::process_n_times_iter;
use crate::visualization::model::{ChartPoint, ChartVerticalLine, LabelOffsetType};
use crate::visualization::utils::Graph;
use crate::{pos, ExpirationDate, Options, Positive};
use num_traits::{FromPrimitive, ToPrimitive};
use plotters::prelude::full_palette::ORANGE;
use plotters::prelude::{ShapeStyle, RED};
use rust_decimal::Decimal;
use std::error::Error;
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
        strategy
            .update_break_even_points()
            .expect("Unable to update break even points");

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

    /// Find break-even points by analyzing profit curve inflection points
    /// Uses a combination of numerical methods to find zeros in the profit function
    fn calculate_break_even_points(&mut self) {
        self.break_even_points = Vec::new();

        // Get the visible range for our search
        let (min_price, max_price) = self.range_to_show().unwrap();

        // Create a finer grid for initial sampling
        let num_samples = 1000; // Increased for better precision
        let fine_step = (max_price - min_price) / num_samples as f64;

        // Sample profits at regular intervals to create a profit curve
        let mut price_profit_pairs: Vec<(Positive, f64)> = Vec::new();
        let mut current_price = min_price;

        while current_price <= max_price {
            let profit = self
                .calculate_profit_at(current_price)
                .unwrap()
                .to_f64()
                .unwrap();
            price_profit_pairs.push((current_price, profit));
            current_price += fine_step;
        }

        // Analyze consecutive pairs for potential break-even points
        for window in price_profit_pairs.windows(2) {
            if let [(price1, profit1), (price2, profit2)] = window {
                // Check if profits have different signs
                if (profit1.signum() != profit2.signum())
                    || profit1.abs() < self.epsilon
                    || profit2.abs() < self.epsilon
                {
                    // Handle cases where profit is very close to zero
                    if profit1.abs() < self.epsilon {
                        self.add_unique_break_even(*price1);
                        continue;
                    }
                    if profit2.abs() < self.epsilon {
                        self.add_unique_break_even(*price2);
                        continue;
                    }

                    // Linear interpolation for better initial guess
                    let t = -profit1 / (profit2 - profit1);
                    let initial_guess = *price1 + (*price2 - *price1) * t;

                    // Refine using Newton-Raphson method
                    if let Some(refined_point) = self.refine_break_even_point(initial_guess) {
                        self.add_unique_break_even(refined_point);
                    }
                }
            }
        }

        // Sort break-even points for consistency
        self.break_even_points
            .sort_by(|a, b| a.partial_cmp(b).unwrap());
    }

    /// Refine a break-even point guess using Newton-Raphson method
    fn refine_break_even_point(&self, initial_guess: Positive) -> Option<Positive> {
        let mut x = initial_guess;
        let mut iterations = 0;

        while iterations < self.max_iterations {
            let f_x = self.calculate_profit_at(x).unwrap().to_f64().unwrap();

            // Check if we're close enough to zero
            if f_x.abs() < self.epsilon {
                return Some(x);
            }

            // Calculate derivative numerically with smaller step
            let h = self.epsilon.sqrt();
            let derivative = (self.calculate_profit_at(x + h).unwrap().to_f64().unwrap() - f_x) / h;

            // Avoid division by very small numbers
            if derivative.abs() < self.epsilon {
                break;
            }

            // Newton-Raphson step
            let next_x = x - f_x / derivative;

            // Check for convergence with absolute difference
            if (next_x.to_f64() - x.to_f64()).abs() < self.epsilon {
                return Some(next_x);
            }

            x = next_x;
            iterations += 1;
        }

        None
    }

    /// Add a break-even point if it's not already in the list
    fn add_unique_break_even(&mut self, point: Positive) {
        if !self
            .break_even_points
            .iter()
            .any(|p| (p.to_f64() - point.to_f64()).abs() < self.epsilon)
        {
            self.break_even_points.push(point);
        }
    }

    pub(crate) fn get_profit_loss_zones(
        &self,
        break_even_points: &[Positive],
    ) -> Result<(Vec<ProfitLossRange>, Vec<ProfitLossRange>), ProbabilityError> {
        if break_even_points.is_empty() {
            return Ok((vec![], vec![]));
        }

        let mut profit_zones = Vec::new();
        let mut loss_zones = Vec::new();

        if break_even_points.len() == 1 {
            let break_even = break_even_points[0];
            let test_point = break_even - pos!(0.01);
            let is_profit_below = self.calculate_profit_at(test_point)? > Decimal::ZERO;

            if is_profit_below {
                profit_zones.push(ProfitLossRange::new(
                    None,
                    Some(break_even),
                    Positive::ZERO,
                )?);
                loss_zones.push(ProfitLossRange::new(
                    Some(break_even),
                    None,
                    Positive::ZERO,
                )?);
            } else {
                loss_zones.push(ProfitLossRange::new(
                    None,
                    Some(break_even),
                    Positive::ZERO,
                )?);
                profit_zones.push(ProfitLossRange::new(
                    Some(break_even),
                    None,
                    Positive::ZERO,
                )?);
            }
        } else {
            let test_point = break_even_points[0] - pos!(0.01);
            let is_first_zone_profit = self.calculate_profit_at(test_point)? > Decimal::ZERO;

            let ranges = (0..=break_even_points.len())
                .map(|i| match i {
                    0 => ProfitLossRange::new(None, Some(break_even_points[0]), Positive::ZERO),
                    i if i == break_even_points.len() => {
                        ProfitLossRange::new(Some(break_even_points[i - 1]), None, Positive::ZERO)
                    }
                    i => ProfitLossRange::new(
                        Some(break_even_points[i - 1]),
                        Some(break_even_points[i]),
                        Positive::ZERO,
                    ),
                })
                .collect::<Result<Vec<_>, _>>()?;

            for (i, range) in ranges.into_iter().enumerate() {
                if (is_first_zone_profit && i % 2 == 0) || (!is_first_zone_profit && i % 2 != 0) {
                    profit_zones.push(range);
                } else {
                    loss_zones.push(range);
                }
            }
        }

        Ok((profit_zones, loss_zones))
    }
}

impl BreakEvenable for CustomStrategy {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.calculate_break_even_points();
        Ok(())
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
            let current_profit = self.calculate_profit_at(current_price)?;
            if current_profit < Decimal::ZERO {
                current_price += step;
                continue;
            }
            if current_profit > max_profit.to_dec() {
                max_profit = current_profit.into();
                self.max_profit_point = Some((current_price, current_profit.to_f64().unwrap()));
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
            let current_profit = self.calculate_profit_at(current_price)?;
            if current_profit > Decimal::ZERO {
                current_price += step;
                continue;
            }
            if current_profit.abs() > max_loss.to_dec() {
                max_loss = current_profit.abs().into();
                self.max_loss_point = Some((current_price, current_profit.to_f64().unwrap()));
            }
            current_price += step;
        }
        debug!("Max Loss: {:.2}", max_loss);
        Ok(max_loss)
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

        let mut total_profit: Decimal = Decimal::ZERO;
        let (mut current_price, max_search_price) = self.range_to_show()?;
        while current_price < max_search_price {
            let current_profit = self.calculate_profit_at(current_price)?;
            if current_profit > Decimal::ZERO {
                total_profit += current_profit;
            }
            current_price += self.step_by;
        }
        let restult = total_profit / self.underlying_price.to_dec();
        Ok(restult)
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
    fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
        let price = Some(&price);
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

impl ProbabilityAnalysis for CustomStrategy {
    fn get_expiration(&self) -> Result<ExpirationDate, ProbabilityError> {
        match self.positions.first() {
            Some(position) => Ok(position.option.expiration_date.clone()),
            None => Err(ProbabilityError::NoPositions(
                "get_expiration: No positions found ".to_string(),
            )),
        }
    }

    fn get_risk_free_rate(&self) -> Option<Decimal> {
        self.positions
            .first()
            .map(|position| position.option.risk_free_rate)
    }

    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_points = self.get_break_even_points()?;

        let implied_volatilities = self
            .positions
            .iter()
            .map(|position| position.option.implied_volatility)
            .collect();
        let (mean_volatility, std_dev) = mean_and_std(implied_volatilities);

        let (mut profit_ranges, _) = self.get_profit_loss_zones(break_even_points)?;

        profit_ranges.iter_mut().for_each(|range| {
            range
                .calculate_probability(
                    self.get_underlying_price(),
                    Some(VolatilityAdjustment {
                        base_volatility: mean_volatility,
                        std_dev_adjustment: std_dev,
                    }),
                    None,
                    self.get_expiration().unwrap(),
                    self.get_risk_free_rate(),
                )
                .unwrap();
        });

        Ok(profit_ranges)
    }

    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_points = self.get_break_even_points()?;

        let implied_volatilities = self
            .positions
            .iter()
            .map(|position| position.option.implied_volatility)
            .collect();
        let (mean_volatility, std_dev) = mean_and_std(implied_volatilities);

        let (_, mut loss_ranges) = self.get_profit_loss_zones(break_even_points)?;

        loss_ranges.iter_mut().for_each(|range| {
            range
                .calculate_probability(
                    self.get_underlying_price(),
                    Some(VolatilityAdjustment {
                        base_volatility: mean_volatility,
                        std_dev_adjustment: std_dev,
                    }),
                    None,
                    self.get_expiration().unwrap(),
                    self.get_risk_free_rate(),
                )
                .unwrap();
        });

        Ok(loss_ranges)
    }
}

impl Greeks for CustomStrategy {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(self
            .positions
            .iter()
            .map(|position| &position.option)
            .collect())
    }
}

#[cfg(test)]
fn create_test_strategy() -> CustomStrategy {
    use crate::{OptionStyle, OptionType, Options, Side};
    use chrono::Utc;
    use rust_decimal_macros::dec;

    let underlying_price = pos!(2340.0);
    let underlying_symbol = "GAS".to_string();
    let expiration = ExpirationDate::Days(pos!(6.0));
    let implied_volatility = pos!(0.73);
    let risk_free_rate = dec!(0.05);
    let dividend_yield = Positive::ZERO;

    // Short Call 1
    let short_strike_1_strike = pos!(2050.0);
    let short_strike_1_quantity = pos!(3.0);
    let short_strike_1_premium = pos!(192.0);
    let short_strike_1_open_fee = pos!(7.51);
    let short_strike_1_close_fee = pos!(7.51);

    // Short Call 2
    let short_strike_2_strike = pos!(2250.0);
    let short_strike_2_quantity = pos!(2.0);
    let short_strike_2_premium = pos!(88.0);
    let short_strike_2_open_fee = pos!(6.68);
    let short_strike_2_close_fee = pos!(6.68);

    // Short Put
    let short_put_strike = pos!(2400.0);
    let short_put_premium = pos!(55.0);
    let short_put_quantity = pos!(1.0);
    let short_put_open_fee = pos!(6.68);
    let short_put_close_fee = pos!(6.68);

    let short_strike_1 = Position::new(
        Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            short_strike_1_strike,
            expiration.clone(),
            implied_volatility,
            short_strike_1_quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        ),
        short_strike_1_premium,
        Utc::now(),
        short_strike_1_open_fee,
        short_strike_1_close_fee,
    );

    let short_strike_2 = Position::new(
        Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            short_strike_2_strike,
            expiration.clone(),
            implied_volatility,
            short_strike_2_quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        ),
        short_strike_2_premium,
        Utc::now(),
        short_strike_2_open_fee,
        short_strike_2_close_fee,
    );

    let short_put = Position::new(
        Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            short_put_strike,
            expiration.clone(),
            implied_volatility,
            short_put_quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Put,
            dividend_yield,
            None,
        ),
        short_put_premium,
        Utc::now(),
        short_put_open_fee,
        short_put_close_fee,
    );

    let extra_strike = pos!(2160.0);
    let extra_quantity = pos!(2.5);
    let extra_premium = pos!(21.0);
    let extra_open_fee = pos!(4.91);
    let extra_close_fee = pos!(4.91);

    let extra = Position::new(
        Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            extra_strike,
            expiration.clone(),
            implied_volatility,
            extra_quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Put,
            dividend_yield,
            None,
        ),
        extra_premium,
        Utc::now(),
        extra_open_fee,
        extra_close_fee,
    );

    let positions: Vec<Position> = vec![short_strike_1, short_strike_2, short_put, extra];

    CustomStrategy::new(
        "Custom Strategy".to_string(),
        underlying_symbol,
        "Example of a custom strategy".to_string(),
        underlying_price,
        positions,
        pos!(0.01),
        200,
        pos!(0.01),
    )
}

#[cfg(test)]
mod tests_custom_strategy {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionType};
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option;
    use crate::pos;
    use crate::utils::logger::setup_logger;
    use crate::Options;
    use approx::assert_relative_eq;
    use chrono::Utc;
    use rust_decimal_macros::dec;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_new_custom_strategy() {
        let strategy = create_test_strategy();
        assert_eq!(strategy.name, "Custom Strategy");
        assert_eq!(strategy.symbol, "GAS");
        assert_eq!(strategy.kind, StrategyType::Custom);
        assert_eq!(strategy.description, "Example of a custom strategy");
        assert_eq!(strategy.underlying_price, 2340.0);
        assert!(!strategy.break_even_points.is_empty());
        assert_eq!(strategy.positions.len(), 4);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_break_even_points_single_call() {
        setup_logger();
        let strategy = create_test_strategy();
        assert_eq!(strategy.break_even_points.len(), 2);
        assert_relative_eq!(
            strategy.break_even_points[0].to_f64(),
            2014.34,
            epsilon = strategy.epsilon.to_f64()
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_break_even_points_single_put() {
        let mut strategy = create_test_strategy();
        let underlying_price = pos!(5780.0);
        let underlying_symbol = "SP500".to_string();
        let expiration = ExpirationDate::Days(pos!(60.0));
        let implied_volatility = pos!(0.18);
        let risk_free_rate = dec!(0.05);
        let dividend_yield = Positive::ZERO;

        // Long Call ITM
        let long_strike_itm = pos!(5750.0);
        let long_quantity = pos!(1.0);
        let premium_long_itm = pos!(85.04);
        let open_fee_long = pos!(0.78);
        let close_fee_long = pos!(0.78);

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
        assert_eq!(strategy.break_even_points.len(), 2);
        assert_relative_eq!(
            strategy.break_even_points[0].to_f64(),
            2039.08285,
            epsilon = strategy.epsilon.to_f64()
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_break_even_points_straddle() {
        setup_logger();

        let mut strategy = create_test_strategy();
        let underlying_price = pos!(5780.0);
        let underlying_symbol = "SP500".to_string();
        let expiration = ExpirationDate::Days(pos!(60.0));
        let implied_volatility = pos!(0.18);
        let risk_free_rate = dec!(0.05);
        let dividend_yield = Positive::ZERO;

        // Long Call ITM
        let long_strike_itm = pos!(5750.0);
        let long_quantity = pos!(1.0);
        let premium_long_itm = pos!(85.04);
        let open_fee_long = pos!(0.78);
        let close_fee_long = pos!(0.78);

        // Long Call OTM
        let long_strike_otm = pos!(5850.0);
        let premium_long_otm = pos!(31.65);
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

        assert_eq!(strategy.positions.len(), 6);
        assert_eq!(strategy.break_even_points.len(), 2);
        assert_relative_eq!(
            strategy.break_even_points[0].to_f64(),
            2048.5714,
            epsilon = strategy.epsilon.to_f64()
        );
        assert_relative_eq!(
            strategy.break_even_points[1].to_f64(),
            2190.0,
            epsilon = strategy.epsilon.to_f64()
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_break_even_points_no_get_break_even_points() {
        let mut strategy = create_test_strategy();
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            pos!(100.0), // underlying_price
            pos!(1.0),   // quantity
            pos!(100.0), // strike_price
            pos!(0.2),   // volatility
        );
        strategy.positions.push(Position::new(
            option,
            pos!(5.0),
            Default::default(),
            Positive::ZERO,
            Positive::ZERO,
        ));
        strategy.calculate_break_even_points();

        assert_eq!(strategy.break_even_points.len(), 0);
    }

    #[test]
    fn test_new_with_empty_positions() {
        let result = std::panic::catch_unwind(|| {
            CustomStrategy::new(
                "Test Strategy".to_string(),
                "TEST".to_string(),
                "Test Description".to_string(),
                pos!(100.0),
                vec![],
                pos!(0.001),
                100,
                pos!(1.0),
            )
        });
        assert!(result.is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_add_position_invalid_strategy() {
        let mut strategy = create_test_strategy();
        let invalid_position = Position::new(
            create_sample_option(
                OptionStyle::Put,
                Side::Short,
                pos!(0.0),
                pos!(1.0),
                pos!(100.0),
                pos!(0.2),
            ),
            pos!(5.0),
            Default::default(),
            Positive::ZERO,
            Positive::ZERO,
        );

        let result = strategy.add_position(&invalid_position);
        assert!(result.is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_ratio_missing_points() {
        let mut strategy = create_test_strategy();
        strategy.max_profit_point = None;
        let result = strategy.profit_ratio();
        assert!(result.is_err());

        strategy.max_profit_point = Some((pos!(100.0), 50.0));
        strategy.max_loss_point = None;
        let result = strategy.profit_ratio();
        assert!(result.is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_break_even_points_edge_cases() {
        let strategy = create_test_strategy();

        let result = strategy.refine_break_even_point(pos!(0.0001));
        assert!(result.is_some());

        let result = strategy.refine_break_even_point(pos!(100.0));
        assert!(result.is_some());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_add_duplicate_break_even_point() {
        let mut strategy = create_test_strategy();
        let point = pos!(100.0);

        strategy.add_unique_break_even(point);
        let initial_len = strategy.break_even_points.len();

        strategy.add_unique_break_even(point);
        assert_eq!(strategy.break_even_points.len(), initial_len);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_graph_implementation() {
        let strategy = create_test_strategy();

        // Test title
        let title = strategy.title();
        assert!(title.contains(&strategy.name));
        assert!(title.contains(&strategy.symbol));

        // Test vertical lines
        let lines = strategy.get_vertical_lines();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].x_coordinate, strategy.underlying_price.to_f64());

        // Test points
        let points = strategy.get_points();
        assert!(!points.is_empty());
    }
}

#[cfg(test)]
mod tests_max_profit {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionType};
    use crate::model::types::{OptionStyle, Side};
    use crate::pos;
    use crate::utils::logger::setup_logger;
    use crate::Options;
    use chrono::Utc;
    use rust_decimal_macros::dec;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_max_profit_single_long_call() {
        let mut strategy = create_test_strategy();
        let max_profit = strategy.max_profit_iter().unwrap();
        assert!(max_profit > Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_max_profit_multi_leg_strategy() {
        setup_logger();

        let mut strategy = create_test_strategy();
        let underlying_price = pos!(5780.0);
        let underlying_symbol = "SP500".to_string();
        let expiration = ExpirationDate::Days(pos!(60.0));
        let implied_volatility = pos!(0.18);
        let risk_free_rate = dec!(0.05);
        let dividend_yield = Positive::ZERO;

        // Long Call ITM
        let long_strike_itm = pos!(5750.0);
        let long_quantity = pos!(1.0);
        let premium_long_itm = pos!(85.04);
        let open_fee_long = pos!(0.78);
        let close_fee_long = pos!(0.78);

        // Long Call OTM
        let long_strike_otm = pos!(5850.0);
        let premium_long_otm = pos!(31.65);
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
    use crate::model::types::{ExpirationDate, OptionType};
    use crate::model::types::{OptionStyle, Side};
    use crate::pos;
    use crate::utils::logger::setup_logger;
    use crate::Options;
    use chrono::Utc;
    use rust_decimal_macros::dec;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_max_loss_single_long_call() {
        let mut strategy = create_test_strategy();
        let max_loss = strategy.max_loss_iter().unwrap();
        assert!(max_loss > Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_max_loss_multi_leg_strategy() {
        setup_logger();

        let mut strategy = create_test_strategy();
        let underlying_price = pos!(5780.0);
        let underlying_symbol = "SP500".to_string();
        let expiration = ExpirationDate::Days(pos!(60.0));
        let implied_volatility = pos!(0.18);
        let risk_free_rate = dec!(0.05);
        let dividend_yield = Positive::ZERO;

        // Long Call ITM
        let long_strike_itm = pos!(5750.0);
        let long_quantity = pos!(1.0);
        let premium_long_itm = pos!(85.04);
        let open_fee_long = pos!(0.78);
        let close_fee_long = pos!(0.78);

        // Long Call OTM
        let long_strike_otm = pos!(5850.0);
        let premium_long_otm = pos!(31.65);
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
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
    use crate::pos;
    use crate::utils::setup_logger;
    use crate::Options;
    use chrono::Utc;
    use rust_decimal_macros::dec;

    fn create_test_position(side: Side, premium: Positive, fees: Positive) -> Position {
        Position::new(
            Options::new(
                OptionType::European,
                side,
                "TEST".to_string(),
                pos!(100.0),
                ExpirationDate::Days(pos!(30.0)),
                pos!(0.2),
                pos!(1.0),
                pos!(100.0),
                dec!(0.01),
                OptionStyle::Call,
                Positive::ZERO,
                None,
            ),
            premium,
            Utc::now(),
            fees, // open fees
            fees, // closing fees
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_total_cost_only_long_positions() {
        let positions = vec![
            create_test_position(Side::Long, pos!(5.0), pos!(0.5)), // net cost = 6.0 (premium + fees)
            create_test_position(Side::Long, pos!(3.0), pos!(0.5)), // net cost = 4.0 (premium + fees)
        ];

        let strategy = CustomStrategy::new(
            "Test".to_string(),
            "TEST".to_string(),
            "Test description".to_string(),
            pos!(100.0),
            positions,
            pos!(0.001),
            100,
            pos!(1.0),
        );

        assert_eq!(strategy.total_cost().unwrap(), 10.0); // 6.0 + 4.0
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_total_cost_only_short_positions() {
        setup_logger();
        let position_1 = create_test_position(Side::Short, pos!(5.0), pos!(0.5));
        let position_2 = create_test_position(Side::Short, pos!(3.0), pos!(0.5));

        assert_eq!(position_1.total_cost().unwrap(), 1.0);
        assert_eq!(position_2.total_cost().unwrap(), 1.0);

        let positions = vec![position_1, position_2];

        let strategy = CustomStrategy::new(
            "Test".to_string(),
            "TEST".to_string(),
            "Test description".to_string(),
            pos!(120.0),
            positions,
            pos!(0.001),
            10,
            pos!(1.0),
        );

        assert_eq!(strategy.total_cost().unwrap(), 2.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_total_cost_mixed_positions() {
        let positions = vec![
            create_test_position(Side::Long, pos!(5.0), pos!(0.5)), // net cost = 6.0
            create_test_position(Side::Short, pos!(3.0), pos!(0.5)), // net cost = 1.0
            create_test_position(Side::Long, pos!(4.0), pos!(0.5)), // net cost = 5.0
            create_test_position(Side::Short, pos!(2.0), pos!(0.75)), // net cost = 1.5
        ];

        let strategy = CustomStrategy::new(
            "Test".to_string(),
            "TEST".to_string(),
            "Test description".to_string(),
            pos!(100.0),
            positions,
            pos!(0.001),
            100,
            pos!(1.0),
        );

        assert_eq!(strategy.total_cost().unwrap(), 13.5); // 6.0 + 1.0 + 5.0 + 1.5
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_total_cost_with_different_premiums_and_fees() {
        let positions = vec![
            create_test_position(Side::Long, pos!(10.0), pos!(1.0)),
            create_test_position(Side::Short, pos!(5.0), pos!(0.5)),
            create_test_position(Side::Short, pos!(3.0), pos!(0.25)),
            create_test_position(Side::Long, pos!(7.0), pos!(0.75)),
        ];

        let strategy = CustomStrategy::new(
            "Test".to_string(),
            "TEST".to_string(),
            "Test description".to_string(),
            pos!(100.0),
            positions,
            pos!(0.001),
            100,
            pos!(1.0),
        );
        assert_eq!(strategy.total_cost().unwrap(), 22.0); // 12.0 + 1.0 + 8.5 + 0.5
    }
}

#[cfg(test)]
mod tests_best_range_to_show {
    use super::*;
    use crate::constants::STRIKE_PRICE_LOWER_BOUND_MULTIPLIER;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
    use crate::{pos, Options};
    use chrono::Utc;
    use rust_decimal_macros::dec;

    fn create_test_position(strike: Positive, side: Side) -> Position {
        Position::new(
            Options::new(
                OptionType::European,
                side,
                "SP500".to_string(),
                strike,
                ExpirationDate::Days(pos!(60.0)),
                pos!(0.18),
                pos!(2.0),
                pos!(5780.0),
                dec!(0.05),
                OptionStyle::Call,
                Positive::ZERO,
                None,
            ),
            pos!(53.04),
            Utc::now(),
            pos!(0.78),
            pos!(0.78),
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
            pos!(5780.0),
            positions,
            pos!(1e-16),
            1000,
            pos!(0.1),
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_best_range_single_strike() {
        let strategy = create_test_strategy_with_strikes(vec![pos!(5800.0)]);
        let step = pos!(10.0);
        let range = strategy.best_range_to_show(step).unwrap();

        assert_eq!(range.first().unwrap().to_f64(), 5644.8);
        assert_eq!(range.last().unwrap().to_f64(), 5974.8);

        // Check step size
        for i in 0..range.len() - 1 {
            assert_eq!((range[i + 1] - range[i]).to_f64(), step.to_f64());
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_best_range_multiple_strikes() {
        let strategy =
            create_test_strategy_with_strikes(vec![pos!(5700.0), pos!(5800.0), pos!(5900.0)]);
        let step = pos!(50.0);
        let range = strategy.best_range_to_show(step).unwrap();

        assert_eq!(range.first().unwrap().to_f64(), 5546.8);
        assert_eq!(range.last().unwrap().to_f64(), 6046.8);

        // Verify step size
        for i in 0..range.len() - 1 {
            assert_eq!((range[i + 1] - range[i]).to_f64(), step.to_f64());
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_best_range_with_small_step() {
        let strategy = create_test_strategy_with_strikes(vec![pos!(5800.0), pos!(5850.0)]);
        let step = pos!(5.0);
        let range = strategy.best_range_to_show(step).unwrap();

        // Verify granular steps
        for i in 0..range.len() - 1 {
            assert_eq!((range[i + 1] - range[i]).to_f64(), step.to_f64());
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_best_range_with_underlying() {
        let strategy = create_test_strategy_with_strikes(vec![pos!(5700.0), pos!(5900.0)]);
        let range = strategy.best_range_to_show(pos!(10.0)).unwrap();

        // Verify range includes underlying price (5780.0)
        assert!(range.iter().any(|&price| price <= pos!(5780.0)));
        assert!(range.iter().any(|&price| price >= pos!(5780.0)));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_best_range_with_large_step() {
        let strategy = create_test_strategy_with_strikes(vec![pos!(5600.0), pos!(6000.0)]);
        let step = pos!(100.0);
        let range = strategy.best_range_to_show(step).unwrap();

        // Verify minimum points
        assert!(range.len() >= 3);

        // Verify step size
        for i in 0..range.len() - 1 {
            assert_eq!((range[i + 1] - range[i]).to_f64(), step.to_f64());
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_best_range_strike_bounds() {
        let min_strike = pos!(5600.0);
        let max_strike = pos!(6000.0);
        let strategy = create_test_strategy_with_strikes(vec![min_strike, max_strike]);
        let range = strategy.best_range_to_show(pos!(50.0)).unwrap();

        let expected_min = (min_strike * STRIKE_PRICE_LOWER_BOUND_MULTIPLIER).to_f64();

        assert!(range.first().unwrap().to_f64() <= expected_min);
        assert!(range.last().unwrap().to_f64() >= max_strike.to_f64());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_best_range_unordered_strikes() {
        let strategy =
            create_test_strategy_with_strikes(vec![pos!(5600.0), pos!(5700.0), pos!(5100.0)]);
        let range = strategy.best_range_to_show(pos!(50.0)).unwrap();

        assert_eq!(range.first().unwrap().to_f64(), 4998.0);
        assert_eq!(range.last().unwrap().to_f64(), 6598.0);
    }
}

#[cfg(test)]
mod tests_greeks {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
    use crate::utils::setup_logger;
    use crate::Options;
    use crate::{assert_decimal_eq, pos};
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
                ExpirationDate::Days(pos!(30.0)),
                pos!(0.2),   // volatility
                pos!(1.0),   // quantity
                pos!(100.0), // underlying_price
                dec!(0.05),  // risk_free_rate
                option_style,
                pos!(0.02), // dividend_yield
                None,
            ),
            pos!(10.0), // premium
            Utc::now(),
            Positive::ONE, // open_fee
            Positive::ONE, // close_fee
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_greeks_single_long_call() {
        let position = create_test_position(pos!(100.0), Side::Long, OptionStyle::Call);
        let strategy = CustomStrategy::new(
            "Long Call".to_string(),
            "TEST".to_string(),
            "Test Description".to_string(),
            pos!(100.0),
            vec![position.clone()],
            pos!(0.001),
            100,
            Positive::ONE,
        );

        let strategy_greeks = strategy.greeks().unwrap();
        let position_greeks = position.greeks().unwrap();

        assert_decimal_eq!(strategy_greeks.delta, position_greeks.delta, EPSILON);
        assert_decimal_eq!(strategy_greeks.gamma, position_greeks.gamma, EPSILON);
        assert_decimal_eq!(strategy_greeks.theta, position_greeks.theta, EPSILON);
        assert_decimal_eq!(strategy_greeks.vega, position_greeks.vega, EPSILON);
        assert_decimal_eq!(strategy_greeks.rho, position_greeks.rho, EPSILON);
        assert_decimal_eq!(strategy_greeks.rho_d, position_greeks.rho_d, EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_greeks_single_short_put() {
        setup_logger();
        let position = create_test_position(pos!(100.0), Side::Short, OptionStyle::Put);
        let strategy = CustomStrategy::new(
            "Short Put".to_string(),
            "TEST".to_string(),
            "Test Description".to_string(),
            pos!(90.0),
            vec![position.clone()],
            pos!(0.001),
            100,
            Positive::ONE,
        );

        let strategy_greeks = strategy.greeks().unwrap();
        let position_greeks = position.greeks().unwrap();

        assert_decimal_eq!(strategy_greeks.delta, position_greeks.delta, EPSILON);
        assert_decimal_eq!(strategy_greeks.gamma, position_greeks.gamma, EPSILON);
        assert_decimal_eq!(strategy_greeks.theta, position_greeks.theta, EPSILON);
        assert_decimal_eq!(strategy_greeks.vega, position_greeks.vega, EPSILON);
        assert_decimal_eq!(strategy_greeks.rho, position_greeks.rho, EPSILON);
        assert_decimal_eq!(strategy_greeks.rho_d, position_greeks.rho_d, EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_greeks_multiple_positions() {
        let long_call = create_test_position(pos!(100.0), Side::Long, OptionStyle::Call);
        let short_put = create_test_position(pos!(95.0), Side::Short, OptionStyle::Put);
        let long_put = create_test_position(pos!(105.0), Side::Long, OptionStyle::Put);

        let strategy = CustomStrategy::new(
            "Multiple".to_string(),
            "TEST".to_string(),
            "Test Description".to_string(),
            pos!(100.0),
            vec![long_call.clone(), short_put.clone(), long_put.clone()],
            pos!(0.001),
            100,
            Positive::ONE,
        );

        let strategy_greeks = strategy.greeks().unwrap();
        let long_call_greeks = long_call.greeks().unwrap();
        let short_put_greeks = short_put.greeks().unwrap();
        let long_put_greeks = long_put.greeks().unwrap();

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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_greeks_straddle() {
        let long_call = create_test_position(pos!(100.0), Side::Long, OptionStyle::Call);
        let long_put = create_test_position(pos!(100.0), Side::Long, OptionStyle::Put);

        let strategy = CustomStrategy::new(
            "Straddle".to_string(),
            "TEST".to_string(),
            "Test Description".to_string(),
            pos!(100.0),
            vec![long_call.clone(), long_put.clone()],
            pos!(0.001),
            100,
            Positive::ONE,
        );

        let strategy_greeks = strategy.greeks().unwrap();
        let call_greeks = long_call.greeks().unwrap();
        let put_greeks = long_put.greeks().unwrap();

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

#[cfg(test)]
mod tests_custom_strategy_probability {
    use super::*;
    use crate::strategies::probabilities::utils::PriceTrend;
    use crate::{assert_pos_relative_eq, pos};
    use num_traits::ToPrimitive;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_profit_loss_zones() {
        let strategy = create_test_strategy();
        let (profit_zones, loss_zones) = strategy
            .get_profit_loss_zones(&strategy.break_even_points)
            .unwrap();

        assert_eq!(profit_zones.len() + loss_zones.len(), 3);
        assert_eq!(profit_zones.len(), 1);
        assert_eq!(loss_zones.len(), 2);

        for zone in profit_zones.iter() {
            assert_eq!(zone.probability, Positive::ZERO);
        }

        for zone in loss_zones.iter() {
            assert_eq!(zone.probability, Positive::ZERO);
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_expiration() {
        let strategy = create_test_strategy();
        let result = strategy.get_expiration();

        assert!(result.is_ok());
        match result.unwrap() {
            ExpirationDate::Days(days) => assert_eq!(days, 6.0),
            _ => panic!("Expected ExpirationDate::Days"),
        }
    }

    #[test]
    #[should_panic]
    fn test_get_expiration_empty_positions() {
        let _ = CustomStrategy::new(
            "Empty Strategy".to_string(),
            "TEST".to_string(),
            "Empty strategy for testing".to_string(),
            pos!(100.0),
            vec![],
            pos!(0.01),
            200,
            pos!(0.01),
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_risk_free_rate() {
        let strategy = create_test_strategy();
        let rate = strategy.get_risk_free_rate();

        assert!(rate.is_some());
        assert_eq!(rate.unwrap().to_f64().unwrap(), 0.05);
    }

    #[test]
    #[should_panic]
    fn test_get_risk_free_rate_empty_positions() {
        let _ = CustomStrategy::new(
            "Empty Strategy".to_string(),
            "TEST".to_string(),
            "Empty strategy for testing".to_string(),
            pos!(100.0),
            vec![],
            pos!(0.01),
            200,
            pos!(0.01),
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_profit_ranges() {
        let strategy = create_test_strategy();
        let result = strategy.get_profit_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();

        assert!(!ranges.is_empty());

        for range in ranges {
            assert!(range.probability >= Positive::ZERO);
            assert!(range.probability <= pos!(1.0));
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_loss_ranges() {
        let strategy = create_test_strategy();
        let result = strategy.get_loss_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();

        assert!(!ranges.is_empty());

        for range in ranges {
            assert!(range.probability >= Positive::ZERO);
            assert!(range.probability <= pos!(1.0));
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_loss_ranges_consistency() {
        let strategy = create_test_strategy();
        let profit_ranges = strategy.get_profit_ranges().unwrap();
        let loss_ranges = strategy.get_loss_ranges().unwrap();

        let total_profit_prob: Positive = profit_ranges.iter().map(|r| r.probability).sum();
        let total_loss_prob: Positive = loss_ranges.iter().map(|r| r.probability).sum();

        assert_pos_relative_eq!(total_profit_prob + total_loss_prob, pos!(1.0), pos!(0.0001));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_ranges_ordering() {
        let strategy = create_test_strategy();
        let profit_ranges = strategy.get_profit_ranges().unwrap();
        let loss_ranges = strategy.get_loss_ranges().unwrap();
        for ranges in [profit_ranges, loss_ranges] {
            for i in 0..ranges.len().saturating_sub(1) {
                if let (Some(upper), Some(lower)) =
                    (ranges[i].upper_bound, ranges[i + 1].lower_bound)
                {
                    assert!(upper <= lower);
                }
            }
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_volatility_calculation() {
        let strategy = create_test_strategy();
        let implied_volatilities: Vec<Positive> = strategy
            .positions
            .iter()
            .map(|position| position.option.implied_volatility)
            .collect();

        assert!(!implied_volatilities.is_empty());
        assert_eq!(implied_volatilities[0], pos!(0.73));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_probability_calculation_with_trend() {
        let strategy = create_test_strategy();
        let trend = Some(PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        });

        let prob = strategy.probability_of_profit(None, trend);
        assert!(prob.is_ok());
        let probability = prob.unwrap();
        assert!(probability > Positive::ZERO);
        assert!(probability <= pos!(1.0));
    }
}
