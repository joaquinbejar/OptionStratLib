/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2/10/24
******************************************************************************/

use super::base::{
    BreakEvenable, Optimizable, Positionable, Strategable, StrategyBasics, StrategyType, Validable,
};
use crate::error::strategies::StrategyError;
use crate::strategies::base::price_gap;
use crate::strategies::shared::{
    CachedBreakEvens, apply_contract_size, common_contract_size, edit_refreshing_break_evens,
};
use crate::strategies::shared::{measured_max_loss, measured_max_profit};
use crate::strategies::utils::calculate_price_range_bounded;
use crate::strategies::{
    BasicAble, Strategies, StrategyConstructor, combinations::process_n_times_iter,
    delta_neutral::DeltaNeutrality, probabilities::ProbabilityAnalysis,
    utils::OptimizationCriteria,
};
use num_traits::ToPrimitive;
use optionstratlib_analytics::analytics::ProfitLossRange;
use optionstratlib_analytics::analytics::VolatilityAdjustment;
use optionstratlib_analytics::error::probability::ProbabilityError;
use optionstratlib_analytics::pnl::DeltaAdjustment;
use optionstratlib_analytics::pnl::{PnLCalculator, utils::PnL};
use optionstratlib_core::error::OperationErrorKind;
use optionstratlib_core::error::position::PositionError;
use optionstratlib_core::model::ExpirationDate;
use optionstratlib_core::model::Options;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::p_sqrt;
use optionstratlib_core::model::decimal::{d_div, d_mul};
use optionstratlib_core::model::{
    Trade,
    decimal::d_add,
    position::Position,
    types::{Action, OptionBasicType, OptionStyle, Side},
    utils::mean_and_std,
};
use optionstratlib_market::chains::UpdateFromOptionData;
use optionstratlib_market::chains::utils::FindOptimalSide;
use optionstratlib_market::chains::{OptionData, chain::OptionChain};
use optionstratlib_pricing::error::GreeksError;
use optionstratlib_pricing::error::PricingError;
use optionstratlib_pricing::greeks::Greeks;
use optionstratlib_pricing::pricing::Profit;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{debug, error};

/// Build a `Positive` from a compile-time non-negative `Decimal` literal.
///
/// All call sites in this module pass `dec!(X.X)` with a statically
/// non-negative value, so the checked constructor is total; the
/// `Positive::ZERO` branch is unreachable and only exists to keep the
/// call site free of `.unwrap()`/`.expect()` per §Error Handling.
#[inline]
fn pos_lit(value: Decimal) -> Positive {
    Positive::new_decimal(value).unwrap_or(Positive::ZERO)
}

/// Represents a custom options trading strategy with user-defined positions and characteristics.
///
/// The `CustomStrategy` struct allows traders to create and analyze bespoke options strategies
/// that don't fit into standard predefined patterns. It contains information about the strategy's
/// positions, risk-reward profile, break-even points, and provides methods for profit-loss analysis.
///
/// This structure supports both analytical calculations and visualization of custom strategies,
/// enabling traders to evaluate potential outcomes across different price points of the underlying asset.
///
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
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
    /// # Errors
    ///
    /// Returns `StrategyError::InvalidStrategy` when the assembled strategy
    /// fails its own `validate` (#696): a position fails `Position::validate`.
    ///
    /// Returns `StrategyError::OperationError` when `positions` is empty, and
    /// propagates any error from `update_break_even_points`.
    #[allow(clippy::too_many_arguments)]
    #[inline(never)]
    pub fn new(
        name: String,
        symbol: String,
        description: String,
        underlying_price: Positive,
        positions: Vec<Position>,
        epsilon: Positive,
        max_iterations: u32,
        step_by: Positive,
    ) -> Result<Self, StrategyError> {
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
            return Err(StrategyError::invalid_parameters(
                "CustomStrategy::new",
                "positions cannot be empty",
            ));
        }
        if !strategy.validate() {
            return Err(StrategyError::invalid_strategy(
                StrategyType::Custom,
                "the legs built by `new` fail validation",
            ));
        }
        strategy.update_break_even_points()?;
        Ok(strategy)
    }

    /// Fails with `reason` when the strategy no longer validates, so an edit
    /// run through [`edit_refreshing_break_evens`] is rolled back. A custom
    /// strategy has always rejected an edit that leaves it invalid (#784).
    fn ensure_valid_after_edit(&self, reason: &str) -> Result<(), PositionError> {
        if self.validate() {
            Ok(())
        } else {
            Err(PositionError::invalid_position(reason))
        }
    }

    /// Replaces the legs with `new_positions` and recomputes the break-evens
    /// with `recompute`, `update_break_even_points` outside the tests.
    ///
    /// An empty `new_positions` replaces the legs and leaves the break-evens
    /// unchanged, as it always has.
    ///
    /// # Errors
    ///
    /// Returns the failed recomputation, with the legs and break-evens
    /// restored to what they were before the call (#791).
    fn update_positions<R>(
        &mut self,
        new_positions: Vec<Position>,
        recompute: &mut R,
    ) -> Result<(), StrategyError>
    where
        R: FnMut(&mut Self) -> Result<(), StrategyError>,
    {
        let previous_positions = std::mem::replace(&mut self.positions, new_positions);
        if self.positions.is_empty() {
            tracing::warn!(
                "CustomStrategy::update_positions received empty vector; leaving break-even points unchanged"
            );
            return Ok(());
        }
        let previous_break_evens = self.break_even_points.clone();
        if let Err(err) = recompute(self) {
            self.positions = previous_positions;
            self.break_even_points = previous_break_evens;
            return Err(err);
        }
        Ok(())
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
        let strike_range = price_gap(*max_strike, *min_strike);

        // For strategies with small strike ranges, use a very focused approach
        let base_extension =
            if strike_range < self.underlying_price.checked_mul(&pos_lit(dec!(0.05)))? {
                // Very tight strikes (< 5% of underlying) - use minimal extension
                strike_range.checked_mul(&pos_lit(dec!(1.5)))? // 150% of strike range
            } else {
                // Wider strikes - use moderate extension
                strike_range.checked_mul(&Positive::ONE)? // 100% of strike range
            };

        // Center around the underlying price for better focus. Strikes spread
        // wider than the spot itself push the lower edge below zero, where the
        // underlying cannot trade.
        let center_price = self.underlying_price;
        let min_price = price_gap(center_price, base_extension);
        let max_price = center_price.checked_add(&base_extension)?;

        Ok((min_price, max_price))
    }

    /// Get the best range to show for visualization
    #[allow(dead_code)]
    fn best_range_to_show(&self, step: Positive) -> Result<Vec<Positive>, StrategyError> {
        let start = self.underlying_price.checked_mul(&pos_lit(dec!(0.5)))?;
        let end = self.underlying_price.checked_mul(&pos_lit(dec!(1.5)))?;
        calculate_price_range_bounded(start, end, step)
    }

    /// Refine a break-even point guess using Newton-Raphson method.
    ///
    /// Returns `None` instead of panicking if any per-iteration profit
    /// evaluation or `Decimal -> f64` conversion fails.
    #[allow(dead_code)]
    fn refine_break_even_point(&self, initial_guess: Positive) -> Option<Positive> {
        let mut x = initial_guess;
        let mut iterations = 0;

        while iterations < self.max_iterations {
            let f_x = self.calculate_profit_at(&x).ok()?.to_f64()?;

            // Check if we're close enough to zero
            if f_x.abs() < self.epsilon {
                return Some(x);
            }

            // Calculate derivative numerically with smaller step
            let h = p_sqrt(&self.epsilon, "strategies::custom::refine_break_even_point").ok()?;
            let f_x_h = self
                .calculate_profit_at(&x.checked_add(&h).ok()?)
                .ok()?
                .to_f64()?;
            let derivative = (f_x_h - f_x) / h;

            // Avoid division by very small numbers
            if derivative.abs() < self.epsilon {
                break;
            }

            // Newton-Raphson step. A step below zero leaves the price domain,
            // so the refinement gives up rather than aborting.
            let next_x = x.checked_sub_f64(f_x / derivative).ok()?;

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

        if let [break_even] = break_even_points {
            let break_even = *break_even;
            // A break-even inside the first cent has no probe point below it;
            // zero is the floor of the price domain, and the multi-point
            // branch below already measures the same distance that way.
            let test_point = price_gap(break_even, pos_lit(dec!(0.01)));
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
        } else if let (Some(first), Some(last)) =
            (break_even_points.first(), break_even_points.last())
        {
            // Multiple break-even points
            let test_point = price_gap(*first, pos_lit(dec!(0.01)));
            let is_profit_below = self.calculate_profit_at(&test_point)? > Decimal::ZERO;
            let is_first_zone_profit = is_profit_below;

            // Create ranges between break-even points: below the first, one
            // per adjacent pair, and above the last.
            let ranges = std::iter::once(ProfitLossRange::new(None, Some(*first), Positive::ZERO))
                .chain(
                    break_even_points
                        .iter()
                        .zip(break_even_points.iter().skip(1))
                        .map(|(lower, upper)| {
                            ProfitLossRange::new(Some(*lower), Some(*upper), Positive::ZERO)
                        }),
                )
                .chain(std::iter::once(ProfitLossRange::new(
                    Some(*last),
                    None,
                    Positive::ZERO,
                )))
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
        Self::new(
            "CustomStrategy".to_string(),
            "".to_string(),
            format!("CustomStrategy: {:?}", vec_options),
            Default::default(),
            Vec::from(vec_options),
            Default::default(),
            100,
            Default::default(),
        )
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
        let min_price = self.underlying_price.checked_mul(&pos_lit(dec!(0.5)))?;
        let max_price = self.underlying_price.checked_mul(&pos_lit(dec!(1.5)))?;
        let step = pos_lit(dec!(0.01));

        // The scan advances one cent at a time, so an underlying in the
        // billions asks for more samples than any machine will finish. That is
        // reported rather than walked.
        for current_price in calculate_price_range_bounded(min_price, max_price, step)? {
            if let Ok(profit) = self.calculate_profit_at(&current_price)
                && profit.abs() < rust_decimal::Decimal::new(1, 2)
            {
                // Close to zero
                self.break_even_points.push(current_price);
            }
        }

        Ok(())
    }
}

impl Positionable for CustomStrategy {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        edit_refreshing_break_evens(self, |strategy| {
            strategy.positions.push(position.clone());
            strategy.ensure_valid_after_edit("Strategy is not valid after adding new position")
        })
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
            1 => matching_positions.into_iter().next().ok_or_else(|| {
                PositionError::invalid_position(
                    "matching_positions length is 1 but iterator yielded none",
                )
            }),
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
        edit_refreshing_break_evens(self, |strategy| {
            let existing_position = strategy
                .get_position_unique(&position.option.option_style, &position.option.side)?;

            *existing_position = position.clone();

            strategy.ensure_valid_after_edit("Strategy is not valid after modifying position")
        })
    }

    fn replace_position(&mut self, position: &Position) -> Result<(), PositionError> {
        edit_refreshing_break_evens(self, |strategy| strategy.replace_leg(position))
    }
}

impl CustomStrategy {
    /// Replaces the leg matching `position`'s style, side and strike; the
    /// body of `replace_position` before break-evens were refreshed (#784).
    fn replace_leg(&mut self, position: &Position) -> Result<(), PositionError> {
        // Find and replace the position with matching criteria
        let slot = self
            .positions
            .iter_mut()
            .find(|p| {
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

        *slot = position.clone();

        self.ensure_valid_after_edit("Strategy is not valid after replacing position")
    }
}

impl CachedBreakEvens for CustomStrategy {
    fn break_evens_mut(&mut self) -> &mut Vec<Positive> {
        &mut self.break_even_points
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

    // `positions` is a `pub` field and the strategy derives `Deserialize`, so
    // an empty leg set reaches the getters below even though `new` rejects
    // one. The getters that have an answer without a leg take it from the
    // strategy itself (its symbol and its spot) or return an empty map; the
    // ones that need a leg (`get_type`, `one_option`) return an error. The
    // values for a strategy with legs are unchanged.
    fn get_symbol(&self) -> Result<&str, StrategyError> {
        match self.positions.first() {
            Some(position) => position.option.get_symbol(),
            None => Ok(&self.symbol),
        }
    }

    fn get_strike(&self) -> Result<HashMap<OptionBasicType<'_>, &Positive>, StrategyError> {
        match self.positions.first() {
            Some(position) => position.option.get_strike(),
            None => Ok(HashMap::new()),
        }
    }

    fn get_underlying_price(&self) -> Result<&Positive, StrategyError> {
        match self.positions.first() {
            Some(position) => position.option.get_underlying_price(),
            None => Ok(&self.underlying_price),
        }
    }

    fn get_risk_free_rate(&self) -> Result<HashMap<OptionBasicType<'_>, &Decimal>, StrategyError> {
        match self.positions.first() {
            Some(position) => position.option.get_risk_free_rate(),
            None => Ok(HashMap::new()),
        }
    }

    fn get_dividend_yield(&self) -> Result<HashMap<OptionBasicType<'_>, &Positive>, StrategyError> {
        match self.positions.first() {
            Some(position) => position.option.get_dividend_yield(),
            None => Ok(HashMap::new()),
        }
    }

    /// The first leg's option.
    ///
    /// # Errors
    ///
    /// Returns [`StrategyError::EmptyCollection`] when the strategy has no
    /// legs: `new` rejects an empty leg set, but the `pub` `positions` field
    /// and `Deserialize` can build one.
    fn one_option(&self) -> Result<&Options, StrategyError> {
        self.positions
            .first()
            .map(|position| &position.option)
            .ok_or_else(|| StrategyError::empty_collection("CustomStrategy::one_option: no legs"))
    }

    /// The first leg's option, mutably.
    ///
    /// # Errors
    ///
    /// Returns [`StrategyError::EmptyCollection`] when the strategy has no
    /// legs, as [`Self::one_option`] does.
    fn one_option_mut(&mut self) -> Result<&mut Options, StrategyError> {
        self.positions
            .first_mut()
            .map(|position| &mut position.option)
            .ok_or_else(|| {
                StrategyError::empty_collection("CustomStrategy::one_option_mut: no legs")
            })
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
    fn get_contract_size(&self) -> Result<Positive, StrategyError> {
        let legs: Vec<&Position> = self.positions.iter().collect();
        common_contract_size(&legs, "CustomStrategy::get_contract_size")
    }
    fn set_contract_size(&mut self, contract_size: Positive) -> Result<(), StrategyError> {
        let mut legs: Vec<&mut Position> = self.positions.iter_mut().collect();
        apply_contract_size(
            &mut legs,
            contract_size,
            "CustomStrategy::set_contract_size",
        )?;
        self.update_break_even_points()
    }
}

impl Strategies for CustomStrategy {
    fn get_volume(&mut self) -> Result<Positive, StrategyError> {
        let mut total_volume = Positive::ZERO;
        for position in &self.positions {
            total_volume = total_volume.checked_add(&position.option.quantity)?;
        }
        Ok(total_volume)
    }

    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        if self.positions.is_empty() {
            return Ok(Positive::ZERO);
        }

        let (min_price, max_price) = self.range_to_show()?;
        let step = max_price
            .checked_sub(&min_price)?
            .checked_div(&pos_lit(dec!(50.0)))?; // Use 50 steps max
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
            current_price = current_price.checked_add(&step)?;
            iterations += 1;
        }

        // If max_profit is still zero or negative, return zero
        if max_profit <= Decimal::ZERO {
            Ok(Positive::ZERO)
        } else {
            Ok(Positive::new_decimal(max_profit)?)
        }
    }

    fn get_max_loss(&self) -> Result<Positive, StrategyError> {
        if self.positions.is_empty() {
            return Ok(Positive::ZERO);
        }

        let (min_price, max_price) = self.range_to_show()?;
        let step = max_price
            .checked_sub(&min_price)?
            .checked_div(&pos_lit(dec!(50.0)))?; // Use 50 steps max
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
            current_price = current_price.checked_add(&step)?;
            iterations += 1;
        }

        // Return absolute value of max loss
        if max_loss >= Decimal::ZERO {
            Ok(Positive::ZERO)
        } else {
            Ok(Positive::new_decimal(-max_loss)?)
        }
    }

    fn get_profit_area(&self) -> Result<Decimal, StrategyError> {
        if self.positions.is_empty() {
            return Ok(Decimal::ZERO);
        }

        let (min_price, max_price) = self.range_to_show()?;
        let step = max_price
            .checked_sub(&min_price)?
            .checked_div(&pos_lit(dec!(50.0)))?; // Use 50 steps max
        let mut total_profit = Decimal::ZERO;
        let mut current_price = min_price;

        // Limit iterations to prevent infinite loops
        let max_iterations = 100;
        let mut iterations = 0;

        while current_price <= max_price && iterations < max_iterations {
            if let Ok(current_profit) = self.calculate_profit_at(&current_price)
                && current_profit > Decimal::ZERO
            {
                total_profit = d_add(
                    total_profit,
                    current_profit,
                    "CustomStrategy::get_profit_area",
                )?;
            }
            current_price = current_price.checked_add(&step)?;
            iterations += 1;
        }

        // The area is expressed as a fraction of the spot, which a zero
        // underlying leaves undefined.
        Ok(d_div(
            total_profit,
            self.underlying_price.to_dec(),
            "CustomStrategy::get_profit_area",
        )?)
    }

    fn get_profit_ratio(&self) -> Result<Decimal, StrategyError> {
        if self.positions.is_empty() {
            return Ok(Decimal::ZERO);
        }

        let max_profit = measured_max_profit(self)?;
        let max_loss = measured_max_loss(self)?;

        if max_loss == Positive::ZERO {
            return Ok(Decimal::ZERO);
        }

        let ratio = d_mul(
            d_div(
                max_profit.to_dec(),
                max_loss.to_dec(),
                "CustomStrategy::get_profit_ratio",
            )?,
            Decimal::from(100),
            "CustomStrategy::get_profit_ratio",
        )?;
        Ok(ratio)
    }

    fn get_best_range_to_show(&self, step: Positive) -> Result<Vec<Positive>, StrategyError> {
        let (start_price, end_price) = self.range_to_show()?;
        calculate_price_range_bounded(start_price, end_price, step)
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
        self.find_optimal_with(option_chain, side, criteria, Self::update_break_even_points);
    }
}

impl CustomStrategy {
    /// The search of [`Optimizable::find_optimal`], with the break-even
    /// recomputation injected so the tests can make it fail (#791).
    ///
    /// A candidate whose break-evens cannot be recomputed is skipped. When
    /// the best positions cannot be applied at the end, the strategy is left
    /// exactly as it was before the search.
    fn find_optimal_with<R>(
        &mut self,
        option_chain: &OptionChain,
        side: FindOptimalSide,
        criteria: OptimizationCriteria,
        mut recompute: R,
    ) where
        R: FnMut(&mut Self) -> Result<(), StrategyError> + Send + Sync,
    {
        let original = self.clone();
        let positions = self.positions.clone();
        let options: Vec<&OptionData> = option_chain.filter_option_data(side);

        let mut best_value = Decimal::MIN;
        let mut best_positions = positions.clone();

        debug!("Starting optimization with {} positions", positions.len());

        let _result = process_n_times_iter(&options, positions.len(), |combination| {
            let mut current_positions = positions.clone();

            // Update each position with the new data
            for (position, option_data) in current_positions.iter_mut().zip(combination.iter()) {
                // TODO now update_from_option_data is returning a Result
                // consider the opportunity to propagate the error by adding a Result return type
                // also to the find_optimal method.
                let _ = position.update_from_option_data(option_data);
            }

            // check if the positions are valid
            for position in current_positions.iter() {
                if !position.validate() {
                    debug!("Invalid position found");
                    return vec![];
                }
            }

            // Evaluate the current combination
            if let Err(e) = self.update_positions(current_positions.clone(), &mut recompute) {
                tracing::debug!(
                    error = %e,
                    "skipping candidate whose break-even points cannot be recomputed"
                );
                return best_positions.clone();
            }
            let metric = match criteria {
                OptimizationCriteria::Ratio => self.get_profit_ratio(),
                OptimizationCriteria::Area => self.get_profit_area(),
            };
            let current_value = match metric {
                Ok(v) => v,
                Err(e) => {
                    tracing::debug!(error = %e, "skipping candidate with unscorable metric");
                    return best_positions.clone();
                }
            };

            if current_value > best_value {
                debug!("Found better value: {} > {}", current_value, best_value);
                best_value = current_value;
                best_positions = current_positions.clone();
            }

            best_positions.clone()
        });
        if let Err(e) = _result {
            tracing::warn!(error = ?e, "process_n_times_iter failed during find_optimal");
        }

        if best_value == Decimal::MIN {
            error!("No valid combinations found");
        }

        debug!("Optimization completed. Best value: {}", best_value);
        if let Err(e) = self.update_positions(best_positions, &mut recompute) {
            tracing::error!(
                error = %e,
                "CustomStrategy::find_optimal cannot recompute the break-even points of the best positions; strategy left unchanged"
            );
            *self = original;
        }
    }
}

impl Profit for CustomStrategy {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, PricingError> {
        let price = Some(price);
        self.positions
            .iter()
            .try_fold(Decimal::ZERO, |acc, position| {
                let pnl = position.pnl_at_expiration(&price)?;
                d_add(acc, pnl, "strategies::custom::profit_at").map_err(PricingError::from)
            })
    }
}

// `Graph` is implemented in `optionstratlib-visualization`
// (`visualization::strategies`, through `impl_graph_for_payoff_strategy!`).

impl ProbabilityAnalysis for CustomStrategy {
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_points = self.get_break_even_points()?;

        let implied_volatilities = self
            .positions
            .iter()
            .map(|position| position.option.implied_volatility)
            .collect();
        let (mean_volatility, std_dev) = mean_and_std(implied_volatilities)?;

        let (mut profit_ranges, _) = self.get_profit_loss_zones(break_even_points)?;

        let expiration = match self.positions.first() {
            Some(position) => position.option.expiration_date,
            None => return Ok(profit_ranges),
        };
        let risk_free_rate = self
            .positions
            .first()
            .map(|position| position.option.risk_free_rate);

        for range in profit_ranges.iter_mut() {
            range.calculate_probability(
                &self.underlying_price,
                VolatilityAdjustment {
                    base_volatility: mean_volatility,
                    std_dev_adjustment: std_dev,
                },
                None, // PriceTrend
                &expiration,
                risk_free_rate,
            )?;
        }

        Ok(profit_ranges)
    }

    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_points = self.get_break_even_points()?;

        let implied_volatilities = self
            .positions
            .iter()
            .map(|position| position.option.implied_volatility)
            .collect();
        let (mean_volatility, std_dev) = mean_and_std(implied_volatilities)?;

        let (_, mut loss_ranges) = self.get_profit_loss_zones(break_even_points)?;

        let expiration = match self.positions.first() {
            Some(position) => position.option.expiration_date,
            None => return Ok(loss_ranges),
        };
        let risk_free_rate = self
            .positions
            .first()
            .map(|position| position.option.risk_free_rate);

        for range in loss_ranges.iter_mut() {
            range.calculate_probability(
                &self.underlying_price,
                VolatilityAdjustment {
                    base_volatility: mean_volatility,
                    std_dev_adjustment: std_dev,
                },
                None, // PriceTrend
                &expiration,
                risk_free_rate,
            )?;
        }

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
    ) -> Result<PnL, PricingError> {
        let mut total = PnL::default();
        for position in &self.positions {
            total = total.try_add(&position.calculate_pnl(
                market_price,
                expiration_date,
                implied_volatility,
            )?)?;
        }
        Ok(total)
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<PnL, PricingError> {
        let mut total = PnL::default();
        for position in &self.positions {
            total = total.try_add(&position.calculate_pnl_at_expiration(underlying_price)?)?;
        }
        Ok(total)
    }

    fn adjustments_pnl(&self, adjustment: &DeltaAdjustment) -> Result<PnL, PricingError> {
        let mut total_pnl = PnL::default();

        for position in &self.positions {
            let position_pnl = position.adjustments_pnl(adjustment)?;
            total_pnl = total_pnl.try_add(&position_pnl)?;
        }

        Ok(total_pnl)
    }
}

#[cfg(test)]
mod tests_find_optimal_break_evens {
    //! `find_optimal` with a break-even recomputation that fails (#791).

    use super::*;
    use chrono::{DateTime, Utc};
    use optionstratlib_core::model::types::OptionType;
    use optionstratlib_core::{pos_or_panic, spos};
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn leg(side: Side, style: OptionStyle, strike: f64, premium: f64) -> Position {
        let option = Options::new(
            OptionType::European,
            side,
            "TEST".to_string(),
            pos_or_panic!(strike),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            Positive::ONE,
            Positive::HUNDRED,
            dec!(0.05),
            style,
            Positive::ZERO,
            None,
        );
        Position::new(
            option,
            pos_or_panic!(premium),
            DateTime::<Utc>::UNIX_EPOCH,
            Positive::ZERO,
            Positive::ZERO,
            None,
            None,
        )
    }

    fn base() -> CustomStrategy {
        CustomStrategy::new(
            "Custom".to_string(),
            "TEST".to_string(),
            "Long call and short put".to_string(),
            Positive::HUNDRED,
            vec![
                leg(Side::Long, OptionStyle::Call, 105.0, 1.2),
                leg(Side::Short, OptionStyle::Put, 95.0, 6.6),
            ],
            pos_or_panic!(0.001),
            100,
            pos_or_panic!(0.1),
        )
        .unwrap_or_else(|e| panic!("{e}"))
    }

    /// Five strikes around 100, optionally leaving one out.
    fn chain(without: Option<f64>) -> OptionChain {
        let mut chain = OptionChain::new(
            "TEST",
            Positive::HUNDRED,
            "2024-12-31".to_string(),
            None,
            None,
        );
        let quotes = [
            (85.0, 16.0, 16.2, 0.4, 0.5),
            (90.0, 11.5, 11.7, 1.1, 1.3),
            (95.0, 7.0, 7.2, 2.6, 2.8),
            (100.0, 3.5, 3.7, 4.1, 4.3),
            (105.0, 1.0, 1.2, 6.6, 6.8),
        ];
        for (strike, call_bid, call_ask, put_bid, put_ask) in quotes {
            if without == Some(strike) {
                continue;
            }
            chain.add_option(
                pos_or_panic!(strike),
                spos!(call_bid),
                spos!(call_ask),
                spos!(put_bid),
                spos!(put_ask),
                pos_or_panic!(0.2),
                Some(dec!(0.5)),
                Some(dec!(0.2)),
                Some(dec!(0.2)),
                spos!(100.0),
                Some(50),
                None,
            );
        }
        chain
    }

    fn legs(strategy: &CustomStrategy) -> Vec<(Positive, Positive)> {
        strategy
            .positions
            .iter()
            .map(|p| (p.option.strike_price, p.premium))
            .collect()
    }

    fn snapshot(strategy: &CustomStrategy) -> serde_json::Value {
        serde_json::to_value(strategy).unwrap_or_else(|e| panic!("{e}"))
    }

    fn recompute_failure() -> StrategyError {
        StrategyError::invalid_parameters("test", "break-evens cannot be recomputed")
    }

    #[test]
    fn test_custom_find_optimal_normal_run_refreshes_break_evens() {
        let mut strategy = base();
        let before = legs(&strategy);
        strategy.find_optimal(
            &chain(None),
            FindOptimalSide::All,
            OptimizationCriteria::Ratio,
        );

        assert_ne!(legs(&strategy), before);
        let rebuilt = CustomStrategy::new(
            "Custom".to_string(),
            "TEST".to_string(),
            "Long call and short put".to_string(),
            Positive::HUNDRED,
            strategy.positions.clone(),
            pos_or_panic!(0.001),
            100,
            pos_or_panic!(0.1),
        )
        .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(strategy.break_even_points, rebuilt.break_even_points);
    }

    #[test]
    fn test_custom_find_optimal_failed_candidate_skipped() {
        // The strike the unrestricted search picks for the long call.
        let mut unrestricted = base();
        unrestricted.find_optimal(
            &chain(None),
            FindOptimalSide::All,
            OptimizationCriteria::Ratio,
        );
        let chosen = unrestricted.positions[0].option.strike_price;

        // The same search where every candidate using that strike fails to
        // recompute its break-evens picks what a chain without it picks.
        let mut skipping = base();
        skipping.find_optimal_with(
            &chain(None),
            FindOptimalSide::All,
            OptimizationCriteria::Ratio,
            |strategy: &mut CustomStrategy| {
                if strategy
                    .positions
                    .iter()
                    .any(|p| p.option.strike_price == chosen)
                {
                    Err(recompute_failure())
                } else {
                    strategy.update_break_even_points()
                }
            },
        );
        let mut reference = base();
        reference.find_optimal(
            &chain(Some(chosen.to_f64())),
            FindOptimalSide::All,
            OptimizationCriteria::Ratio,
        );

        assert!(
            skipping
                .positions
                .iter()
                .all(|p| p.option.strike_price != chosen)
        );
        assert_eq!(legs(&skipping), legs(&reference));
        assert_eq!(skipping.break_even_points, reference.break_even_points);
    }

    #[test]
    fn test_custom_find_optimal_failed_final_application_leaves_strategy_unchanged() {
        // Count the recomputations of a run: every candidate, then the best.
        let calls = AtomicUsize::new(0);
        let mut counted = base();
        counted.find_optimal_with(
            &chain(None),
            FindOptimalSide::All,
            OptimizationCriteria::Ratio,
            |strategy: &mut CustomStrategy| {
                calls.fetch_add(1, Ordering::SeqCst);
                strategy.update_break_even_points()
            },
        );
        let total = calls.load(Ordering::SeqCst);
        assert!(total > 1);

        let mut strategy = base();
        let before = snapshot(&strategy);
        let seen = AtomicUsize::new(0);
        strategy.find_optimal_with(
            &chain(None),
            FindOptimalSide::All,
            OptimizationCriteria::Ratio,
            |strategy: &mut CustomStrategy| {
                if seen.fetch_add(1, Ordering::SeqCst) + 1 == total {
                    Err(recompute_failure())
                } else {
                    strategy.update_break_even_points()
                }
            },
        );

        assert_eq!(seen.load(Ordering::SeqCst), total);
        assert_eq!(snapshot(&strategy), before);
    }

    #[test]
    fn test_custom_find_optimal_every_recomputation_failing_leaves_strategy_unchanged() {
        let mut strategy = base();
        let before = snapshot(&strategy);
        strategy.find_optimal_with(
            &chain(None),
            FindOptimalSide::All,
            OptimizationCriteria::Ratio,
            |_: &mut CustomStrategy| Err(recompute_failure()),
        );

        assert_eq!(snapshot(&strategy), before);
    }

    #[test]
    fn test_custom_update_positions_failed_recompute_rolls_back() {
        let mut strategy = base();
        let before = snapshot(&strategy);
        let result = strategy.update_positions(
            vec![leg(Side::Long, OptionStyle::Call, 90.0, 11.7)],
            &mut |_: &mut CustomStrategy| Err(recompute_failure()),
        );

        assert!(result.is_err());
        assert_eq!(snapshot(&strategy), before);
    }
}

#[cfg(test)]
mod tests_empty_legs {
    //! A `CustomStrategy` whose legs were removed through the `pub`
    //! `positions` field (#788).

    use super::*;
    use optionstratlib_core::model::types::OptionType;

    fn emptied() -> CustomStrategy {
        let leg = Position::new(
            Options::new(
                OptionType::European,
                Side::Long,
                "TEST".to_string(),
                Positive::HUNDRED,
                ExpirationDate::Days(Positive::new(30.0).unwrap()),
                Positive::new(0.2).unwrap(),
                Positive::ONE,
                Positive::HUNDRED,
                dec!(0.05),
                OptionStyle::Call,
                Positive::ZERO,
                None,
            ),
            Positive::ONE,
            chrono::Utc::now(),
            Positive::ZERO,
            Positive::ZERO,
            None,
            None,
        );
        let mut strategy = CustomStrategy::new(
            "Custom".to_string(),
            "TEST".to_string(),
            "One long call".to_string(),
            Positive::HUNDRED,
            vec![leg],
            Positive::new(0.01).unwrap(),
            100,
            Positive::ONE,
        )
        .unwrap();
        strategy.positions.clear();
        strategy
    }

    #[test]
    fn test_one_option_without_legs_returns_err() {
        let mut strategy = emptied();
        assert!(matches!(
            strategy.one_option(),
            Err(StrategyError::EmptyCollection { .. })
        ));
        assert!(matches!(
            strategy.one_option_mut(),
            Err(StrategyError::EmptyCollection { .. })
        ));
        assert!(matches!(
            strategy.get_type(),
            Err(StrategyError::EmptyCollection { .. })
        ));
    }

    #[test]
    fn test_getters_without_legs_answer_from_the_strategy() {
        let strategy = emptied();
        assert_eq!(strategy.get_symbol().unwrap(), "TEST");
        assert_eq!(*strategy.get_underlying_price().unwrap(), Positive::HUNDRED);
        assert!(strategy.get_strike().unwrap().is_empty());
        assert!(strategy.get_risk_free_rate().unwrap().is_empty());
        assert!(strategy.get_dividend_yield().unwrap().is_empty());
    }
}
