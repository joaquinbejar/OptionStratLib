/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2/10/24
******************************************************************************/
use super::base::{
    BreakEvenable, Optimizable, Positionable, Strategable, StrategyBasics, StrategyType, Validable,
};
use crate::{
    ExpirationDate, Options, Positive,
    chains::{OptionData, chain::OptionChain},
    error::{
        GreeksError, OperationErrorKind, position::PositionError, probability::ProbabilityError,
        strategies::StrategyError,
    },
    greeks::Greeks,
    model::{
        ProfitLossRange, Trade,
        position::Position,
        types::{Action, OptionBasicType, OptionStyle, Side},
        utils::mean_and_std,
    },
    pnl::{PnLCalculator, utils::PnL},
    pos,
    pricing::payoff::Profit,
    strategies::{
        BasicAble, DeltaAdjustment, Strategies, StrategyConstructor,
        delta_neutral::DeltaNeutrality,
        probabilities::{core::ProbabilityAnalysis, utils::VolatilityAdjustment},
        utils::{FindOptimalSide, OptimizationCriteria},
    },
    utils::process_n_times_iter,
};
use num_traits::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use tracing::{debug, error};
use utoipa::ToSchema;

/// Represents a custom options trading strategy with user-defined positions and characteristics.
///
/// The `CustomStrategy` struct allows traders to create and analyze bespoke options strategies
/// that don't fit into standard predefined patterns. It contains information about the strategy's
/// positions, risk-reward profile, break-even points, and provides methods for profit-loss analysis.
///
/// This structure supports both analytical calculations and visualization of custom strategies,
/// enabling traders to evaluate potential outcomes across different price points of the underlying asset.
///
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CustomStrategy {
    /// The name of the custom strategy.
    pub name: String,

    /// The ticker symbol of the underlying asset.
    pub symbol: String,

    /// The type of strategy, typically set to StrategyType::Custom.
    pub kind: StrategyType,

    /// A detailed description of the strategy, its purpose, and expected outcomes.
    pub description: String,

    /// The price points at which the strategy breaks even (neither profit nor loss).
    pub break_even_points: Vec<Positive>,

    /// The collection of option positions that make up the strategy.
    pub positions: Vec<Position>,

    /// The current price of the underlying asset.
    pub underlying_price: Positive,

    /// Tolerance value used in numerical calculations for finding critical points.
    epsilon: Positive,

    /// Maximum number of iterations allowed in numerical algorithms.
    max_iterations: u32,

    /// Step size used in price interval calculations.
    step_by: Positive,

    /// The price point and value of maximum profit, if calculated.
    max_profit_point: Option<(Positive, f64)>,

    /// The price point and value of maximum loss, if calculated.
    max_loss_point: Option<(Positive, f64)>,
}

impl CustomStrategy {
    /// Creates a new custom options trading strategy with the specified parameters.
    ///
    /// This constructor initializes a `CustomStrategy` instance and performs several
    /// validation and calculation steps to ensure the strategy is valid and properly
    /// analyzed before being returned to the caller.
    ///
    /// # Parameters
    /// * `name` - The name of the custom strategy
    /// * `symbol` - The ticker symbol of the underlying asset
    /// * `description` - A detailed description of the strategy's purpose and characteristics
    /// * `underlying_price` - The current price of the underlying asset
    /// * `positions` - A collection of option positions that compose the strategy
    /// * `epsilon` - Tolerance value used in numerical calculations for finding critical points
    /// * `max_iterations` - Maximum number of iterations allowed in numerical algorithms
    /// * `step_by` - Step size used in price interval calculations
    ///
    /// # Returns
    /// A fully initialized `CustomStrategy` instance with calculated break-even points,
    /// maximum profit, and maximum loss information.
    ///
    /// # Panics
    /// Panics if the strategy validation fails or if break-even points cannot be calculated.
    /// This typically occurs when the strategy has no positions or when the maximum loss point
    /// cannot be determined.
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
        // Basic validation - check positions are not empty
        if strategy.positions.is_empty() {
            panic!("Invalid strategy: No positions provided");
        }
        strategy
            .update_break_even_points()
            .expect("Unable to update break even points");
        strategy
    }

    fn update_positions(&mut self, new_positions: Vec<Position>) {
        self.positions = new_positions;
        if self.positions.is_empty() {
            panic!("Invalid strategy: No positions provided");
        }
        let _ = self.update_break_even_points();
    }

    /// Calculate the best range to show for price analysis
    fn range_to_show(&self) -> Result<(Positive, Positive), StrategyError> {
        if self.positions.is_empty() {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "range_to_show".to_string(),
                    reason: "No positions found".to_string(),
                },
            ));
        }

        let strikes: Vec<Positive> = self
            .positions
            .iter()
            .map(|position| position.option.strike_price)
            .collect();

        let min_strike = strikes.iter().min().unwrap_or(&self.underlying_price);
        let max_strike = strikes.iter().max().unwrap_or(&self.underlying_price);

        // Use a much more focused range calculation for better visualization
        let strike_range = *max_strike - *min_strike;

        // For strategies with small strike ranges, use a very focused approach
        let base_extension = if strike_range < self.underlying_price * pos!(0.05) {
            // Very tight strikes (< 5% of underlying) - use minimal extension
            strike_range * pos!(1.5) // 150% of strike range
        } else {
            // Wider strikes - use moderate extension
            strike_range * pos!(1.0) // 100% of strike range
        };

        // Center around the underlying price for better focus
        let center_price = self.underlying_price;
        let min_price = center_price - base_extension;
        let max_price = center_price + base_extension;

        Ok((min_price, max_price))
    }

    /// Get the best range to show for visualization
    #[allow(dead_code)]
    fn best_range_to_show(&self, step: Positive) -> Result<Vec<Positive>, StrategyError> {
        let mut prices = Vec::new();
        let mut current_price = self.underlying_price * pos!(0.5);
        let max_price = self.underlying_price * pos!(1.5);

        while current_price <= max_price {
            prices.push(current_price);
            current_price += step;
        }

        Ok(prices)
    }

    /// Refine a break-even point guess using Newton-Raphson method
    #[allow(dead_code)]
    fn refine_break_even_point(&self, initial_guess: Positive) -> Option<Positive> {
        let mut x = initial_guess;
        let mut iterations = 0;

        while iterations < self.max_iterations {
            let f_x = self.calculate_profit_at(&x).unwrap().to_f64().unwrap();

            // Check if we're close enough to zero
            if f_x.abs() < self.epsilon {
                return Some(x);
            }

            // Calculate derivative numerically with smaller step
            let f_x = self.calculate_profit_at(&x).unwrap().to_f64().unwrap();
            let h = self.epsilon.sqrt();
            let derivative = (self
                .calculate_profit_at(&(x + h))
                .unwrap()
                .to_f64()
                .unwrap()
                - f_x)
                / h;

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
    #[allow(dead_code)]
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
            let is_profit_below = self.calculate_profit_at(&test_point)? > Decimal::ZERO;

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
            // Multiple break-even points
            let test_point = break_even_points[0] - pos!(0.01);
            let is_profit_below = self.calculate_profit_at(&test_point)? > Decimal::ZERO;
            let is_first_zone_profit = is_profit_below;

            // Create ranges between break-even points
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

            // Classify ranges as profit or loss zones
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

impl StrategyConstructor for CustomStrategy {
    fn get_strategy(vec_options: &[Position]) -> Result<Self, StrategyError> {
        Ok(Self::new(
            "CustomStrategy".to_string(),
            "".to_string(),
            format!("CustomStrategy: {:?}", vec_options),
            Default::default(),
            Vec::from(vec_options),
            Default::default(),
            100,
            Default::default(),
        ))
    }
}

impl BreakEvenable for CustomStrategy {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        // Simple implementation - calculate break-even points by finding where profit = 0
        self.break_even_points.clear();

        // Get a reasonable price range
        let min_price = self.underlying_price * pos!(0.5);
        let max_price = self.underlying_price * pos!(1.5);
        let step = pos!(0.01);

        let mut current_price = min_price;
        while current_price <= max_price {
            if let Ok(profit) = self.calculate_profit_at(&current_price)
                && profit.abs() < rust_decimal::Decimal::new(1, 2)
            {
                // Close to zero
                self.break_even_points.push(current_price);
            }
            current_price += step;
        }

        Ok(())
    }
}

impl Positionable for CustomStrategy {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        self.positions.push(position.clone());
        if !self.validate() {
            return Err(PositionError::invalid_position(
                "Strategy is not valid after adding new position",
            ));
        }
        let _ = self.update_break_even_points();
        Ok(())
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(self.positions.iter().collect())
    }

    fn get_position(
        &mut self,
        option_style: &OptionStyle,
        side: &Side,
        strike: &Positive,
    ) -> Result<Vec<&mut Position>, PositionError> {
        let matching_positions: Vec<&mut Position> = self
            .positions
            .iter_mut()
            .filter(|position| {
                position.option.option_style == *option_style
                    && position.option.side == *side
                    && position.option.strike_price == *strike
            })
            .collect();

        if matching_positions.is_empty() {
            Err(PositionError::invalid_position(&format!(
                "Position not found: {:?} {:?} strike {}",
                option_style, side, strike
            )))
        } else {
            Ok(matching_positions)
        }
    }

    fn get_position_unique(
        &mut self,
        option_style: &OptionStyle,
        side: &Side,
    ) -> Result<&mut Position, PositionError> {
        let matching_positions: Vec<&mut Position> = self
            .positions
            .iter_mut()
            .filter(|position| {
                position.option.option_style == *option_style && position.option.side == *side
            })
            .collect();

        match matching_positions.len() {
            0 => Err(PositionError::invalid_position(&format!(
                "Position not found: {:?} {:?}",
                option_style, side
            ))),
            1 => Ok(matching_positions.into_iter().next().unwrap()),
            _ => Err(PositionError::invalid_position(&format!(
                "Multiple positions found: {:?} {:?}",
                option_style, side
            ))),
        }
    }

    fn get_option_unique(
        &mut self,
        option_style: &OptionStyle,
        side: &Side,
    ) -> Result<&mut Options, PositionError> {
        let position = self.get_position_unique(option_style, side)?;
        Ok(&mut position.option)
    }

    fn modify_position(&mut self, position: &Position) -> Result<(), PositionError> {
        let existing_position =
            self.get_position_unique(&position.option.option_style, &position.option.side)?;

        *existing_position = position.clone();

        if !self.validate() {
            return Err(PositionError::invalid_position(
                "Strategy is not valid after modifying position",
            ));
        }

        let _ = self.update_break_even_points();
        Ok(())
    }

    fn replace_position(&mut self, position: &Position) -> Result<(), PositionError> {
        // Find and replace the position with matching criteria
        let index = self
            .positions
            .iter()
            .position(|p| {
                p.option.option_style == position.option.option_style
                    && p.option.side == position.option.side
                    && p.option.strike_price == position.option.strike_price
            })
            .ok_or_else(|| {
                PositionError::invalid_position(&format!(
                    "Position not found: {:?} {:?} strike {}",
                    position.option.option_style,
                    position.option.side,
                    position.option.strike_price
                ))
            })?;

        self.positions[index] = position.clone();

        if !self.validate() {
            return Err(PositionError::invalid_position(
                "Strategy is not valid after replacing position",
            ));
        }

        let _ = self.update_break_even_points();
        Ok(())
    }
}

impl Strategable for CustomStrategy {
    fn info(&self) -> Result<StrategyBasics, StrategyError> {
        Ok(StrategyBasics {
            name: self.name.clone(),
            kind: self.kind.clone(),
            description: self.description.clone(),
        })
    }
}

impl BasicAble for CustomStrategy {
    fn get_title(&self) -> String {
        format!("{} - {} Strategy", self.symbol, self.name)
    }

    fn get_option_basic_type(&self) -> HashSet<OptionBasicType<'_>> {
        let mut types = HashSet::new();
        for position in &self.positions {
            types.insert(OptionBasicType {
                option_style: &position.option.option_style,
                side: &position.option.side,
                strike_price: &position.option.strike_price,
                expiration_date: &position.option.expiration_date,
            });
        }
        types
    }

    fn get_implied_volatility(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let mut volatilities = HashMap::new();
        for position in &self.positions {
            let basic_type = OptionBasicType {
                option_style: &position.option.option_style,
                side: &position.option.side,
                strike_price: &position.option.strike_price,
                expiration_date: &position.option.expiration_date,
            };
            volatilities.insert(basic_type, &position.option.implied_volatility);
        }
        volatilities
    }

    fn get_quantity(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let mut quantities = HashMap::new();
        for position in &self.positions {
            let basic_type = OptionBasicType {
                option_style: &position.option.option_style,
                side: &position.option.side,
                strike_price: &position.option.strike_price,
                expiration_date: &position.option.expiration_date,
            };
            quantities.insert(basic_type, &position.option.quantity);
        }
        quantities
    }

    fn one_option(&self) -> &Options {
        &self.positions[0].option
    }

    fn one_option_mut(&mut self) -> &mut Options {
        &mut self.positions[0].option
    }

    fn set_expiration_date(
        &mut self,
        expiration_date: ExpirationDate,
    ) -> Result<(), StrategyError> {
        for position in &mut self.positions {
            position.option.expiration_date = expiration_date;
        }
        Ok(())
    }

    fn set_underlying_price(&mut self, price: &Positive) -> Result<(), StrategyError> {
        self.underlying_price = *price;
        for position in &mut self.positions {
            position.option.underlying_price = *price;
        }
        Ok(())
    }

    fn set_implied_volatility(&mut self, volatility: &Positive) -> Result<(), StrategyError> {
        for position in &mut self.positions {
            position.option.implied_volatility = *volatility;
        }
        Ok(())
    }
}

impl Strategies for CustomStrategy {
    fn get_volume(&mut self) -> Result<Positive, StrategyError> {
        let mut total_volume = Positive::ZERO;
        for position in &self.positions {
            total_volume += position.option.quantity;
        }
        Ok(total_volume)
    }

    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        if self.positions.is_empty() {
            return Ok(Positive::ZERO);
        }

        let (min_price, max_price) = self.range_to_show()?;
        let step = (max_price - min_price) / pos!(50.0); // Use 50 steps max
        let mut max_profit = Decimal::ZERO;
        let mut current_price = min_price;

        // Limit iterations to prevent infinite loops
        let max_iterations = 100;
        let mut iterations = 0;

        while current_price <= max_price && iterations < max_iterations {
            if let Ok(current_profit) = self.calculate_profit_at(&current_price)
                && current_profit > max_profit
            {
                max_profit = current_profit;
            }
            current_price += step;
            iterations += 1;
        }

        // If max_profit is still zero or negative, return zero
        if max_profit <= Decimal::ZERO {
            Ok(Positive::ZERO)
        } else {
            Ok(max_profit.into())
        }
    }

    fn get_max_loss(&self) -> Result<Positive, StrategyError> {
        if self.positions.is_empty() {
            return Ok(Positive::ZERO);
        }

        let (min_price, max_price) = self.range_to_show()?;
        let step = (max_price - min_price) / pos!(50.0); // Use 50 steps max
        let mut max_loss = Decimal::ZERO;
        let mut current_price = min_price;

        // Limit iterations to prevent infinite loops
        let max_iterations = 100;
        let mut iterations = 0;

        while current_price <= max_price && iterations < max_iterations {
            if let Ok(current_profit) = self.calculate_profit_at(&current_price)
                && current_profit < max_loss
            {
                max_loss = current_profit;
            }
            current_price += step;
            iterations += 1;
        }

        // Return absolute value of max loss
        if max_loss >= Decimal::ZERO {
            Ok(Positive::ZERO)
        } else {
            Ok((-max_loss).into())
        }
    }

    fn get_profit_area(&self) -> Result<Decimal, StrategyError> {
        if self.positions.is_empty() {
            return Ok(Decimal::ZERO);
        }

        let (min_price, max_price) = self.range_to_show()?;
        let step = (max_price - min_price) / pos!(50.0); // Use 50 steps max
        let mut total_profit = Decimal::ZERO;
        let mut current_price = min_price;

        // Limit iterations to prevent infinite loops
        let max_iterations = 100;
        let mut iterations = 0;

        while current_price <= max_price && iterations < max_iterations {
            if let Ok(current_profit) = self.calculate_profit_at(&current_price)
                && current_profit > Decimal::ZERO
            {
                total_profit += current_profit;
            }
            current_price += step;
            iterations += 1;
        }

        Ok(total_profit / self.underlying_price.to_dec())
    }

    fn get_profit_ratio(&self) -> Result<Decimal, StrategyError> {
        if self.positions.is_empty() {
            return Ok(Decimal::ZERO);
        }

        let max_profit = self.get_max_profit().unwrap_or(Positive::ZERO);
        let max_loss = self.get_max_loss().unwrap_or(Positive::ZERO);

        if max_loss == Positive::ZERO {
            return Ok(Decimal::ZERO);
        }

        let ratio = (max_profit.to_dec() / max_loss.to_dec()) * Decimal::from(100);
        Ok(ratio)
    }

    fn get_best_range_to_show(&self, step: Positive) -> Result<Vec<Positive>, StrategyError> {
        let (start_price, end_price) = self.range_to_show()?;
        let mut prices = Vec::new();
        let mut current_price = start_price;

        while current_price <= end_price {
            prices.push(current_price);
            current_price += step;
        }

        Ok(prices)
    }

    fn roll_in(&mut self, _position: &Position) -> Result<HashMap<Action, Trade>, StrategyError> {
        // Custom strategy doesn't support rolling operations by default
        Ok(HashMap::new())
    }

    fn roll_out(&mut self, _position: &Position) -> Result<HashMap<Action, Trade>, StrategyError> {
        // Custom strategy doesn't support rolling operations by default
        Ok(HashMap::new())
    }
}

impl Validable for CustomStrategy {
    fn validate(&self) -> bool {
        if self.positions.is_empty() {
            error!("No positions found");
            return false;
        }

        // Validate individual positions
        if !self.positions.iter().all(|position| position.validate()) {
            error!("One or more positions are invalid");
            return false;
        }

        // Max loss point validation is optional during construction
        if let Some(loss) = self.max_loss_point
            && loss.1 >= 0.0
        {
            error!("Max loss point is not valid");
            return false;
        }

        true
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
                OptimizationCriteria::Ratio => self.get_profit_ratio().unwrap(),
                OptimizationCriteria::Area => self.get_profit_area().unwrap(),
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
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, Box<dyn Error>> {
        let price = Some(price);
        self.positions
            .iter()
            .map(|position| position.pnl_at_expiration(&price))
            .try_fold(Decimal::ZERO, |acc, pnl| Ok(acc + pnl?))
    }
}

// Graph trait implementation is provided by the impl_graph_for_payoff_strategy! macro in graph.rs

impl ProbabilityAnalysis for CustomStrategy {
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_points = self.get_break_even_points()?;

        let implied_volatilities = self
            .positions
            .iter()
            .map(|position| position.option.implied_volatility)
            .collect();
        let (mean_volatility, std_dev) = mean_and_std(implied_volatilities);

        let (mut profit_ranges, _) = self.get_profit_loss_zones(break_even_points)?;

        let expiration = match self.positions.first() {
            Some(position) => position.option.expiration_date,
            None => return Ok(profit_ranges),
        };
        let risk_free_rate = self
            .positions
            .first()
            .map(|position| position.option.risk_free_rate);

        profit_ranges.iter_mut().for_each(|range| {
            range
                .calculate_probability(
                    &self.underlying_price,
                    Some(VolatilityAdjustment {
                        base_volatility: mean_volatility,
                        std_dev_adjustment: std_dev,
                    }),
                    None, // PriceTrend
                    &expiration,
                    risk_free_rate,
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

        let expiration = match self.positions.first() {
            Some(position) => position.option.expiration_date,
            None => return Ok(loss_ranges),
        };
        let risk_free_rate = self
            .positions
            .first()
            .map(|position| position.option.risk_free_rate);

        loss_ranges.iter_mut().for_each(|range| {
            range
                .calculate_probability(
                    &self.underlying_price,
                    Some(VolatilityAdjustment {
                        base_volatility: mean_volatility,
                        std_dev_adjustment: std_dev,
                    }),
                    None, // PriceTrend
                    &expiration,
                    risk_free_rate,
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

impl DeltaNeutrality for CustomStrategy {}

impl PnLCalculator for CustomStrategy {
    fn calculate_pnl(
        &self,
        market_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        Ok(self
            .positions
            .iter()
            .map(|position| {
                position
                    .calculate_pnl(market_price, expiration_date, implied_volatility)
                    .unwrap()
            })
            .sum())
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        Ok(self
            .positions
            .iter()
            .map(|position| {
                position
                    .calculate_pnl_at_expiration(underlying_price)
                    .unwrap()
            })
            .sum())
    }

    fn adjustments_pnl(&self, adjustment: &DeltaAdjustment) -> Result<PnL, Box<dyn Error>> {
        let mut total_pnl = PnL::default();

        for position in &self.positions {
            let position_pnl = position.adjustments_pnl(adjustment)?;
            total_pnl = total_pnl + position_pnl;
        }

        Ok(total_pnl)
    }
}
