/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Adjustment Optimizer Module
//!
//! Provides optimization algorithms for finding the best adjustment plan
//! to achieve target Greeks.
//!
//! ## Overview
//!
//! The optimizer evaluates multiple adjustment strategies and selects
//! the one with the best quality score (lowest residual + cost).
//!
//! ## Strategies
//!
//! 1. **Existing legs only**: Adjust quantities of current positions
//! 2. **Add new legs**: Add options from the chain to fill gaps
//! 3. **Use underlying**: Add shares for pure delta adjustment

use crate::Positive;
use crate::chains::chain::OptionChain;
use crate::greeks::Greeks;
use crate::model::position::Position;
use crate::model::types::Side;
use positive::pos_or_panic;
use num_traits::Signed;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::{debug, trace};

use super::adjustment::{AdjustmentAction, AdjustmentConfig, AdjustmentError, AdjustmentPlan};
use super::portfolio::{AdjustmentTarget, PortfolioGreeks};

/// Portfolio-level adjustment optimizer.
///
/// Finds the optimal set of adjustments to achieve target Greeks
/// given a set of positions and available options.
pub struct AdjustmentOptimizer<'a> {
    /// Current positions to adjust
    positions: &'a [Position],
    /// Option chain for finding new legs (optional)
    chain: Option<&'a OptionChain>,
    /// Configuration for adjustment behavior
    config: AdjustmentConfig,
    /// Target Greeks to achieve
    target: AdjustmentTarget,
}

impl<'a> AdjustmentOptimizer<'a> {
    /// Creates a new optimizer with positions only (no chain for new legs).
    ///
    /// # Arguments
    ///
    /// * `positions` - Current positions to adjust
    /// * `config` - Configuration for adjustment behavior
    /// * `target` - Target Greeks to achieve
    pub fn new(
        positions: &'a [Position],
        config: AdjustmentConfig,
        target: AdjustmentTarget,
    ) -> Self {
        Self {
            positions,
            chain: None,
            config,
            target,
        }
    }

    /// Creates a new optimizer with an option chain for adding new legs.
    ///
    /// # Arguments
    ///
    /// * `positions` - Current positions to adjust
    /// * `chain` - Option chain for finding new legs
    /// * `config` - Configuration for adjustment behavior
    /// * `target` - Target Greeks to achieve
    pub fn with_chain(
        positions: &'a [Position],
        chain: &'a OptionChain,
        config: AdjustmentConfig,
        target: AdjustmentTarget,
    ) -> Self {
        Self {
            positions,
            chain: Some(chain),
            config,
            target,
        }
    }

    /// Calculates the optimal adjustment plan.
    ///
    /// Tries multiple strategies and returns the best one based on
    /// quality score (residual delta + cost).
    ///
    /// # Returns
    ///
    /// * `Ok(AdjustmentPlan)` - The optimal adjustment plan
    /// * `Err(AdjustmentError)` - If no viable plan can be found
    pub fn optimize(&self) -> Result<AdjustmentPlan, AdjustmentError> {
        if self.positions.is_empty() {
            return Err(AdjustmentError::NoPositions);
        }

        let current_greeks = PortfolioGreeks::from_positions(self.positions)?;

        // Check if already at target
        if self
            .target
            .is_satisfied(&current_greeks, self.config.delta_tolerance)
        {
            debug!("Already at target, no adjustment needed");
            return Ok(AdjustmentPlan::new(
                vec![],
                Decimal::ZERO,
                current_greeks,
                Decimal::ZERO,
            ));
        }

        let delta_gap = self.target.delta_gap(&current_greeks);
        let gamma_gap = self.target.gamma_gap(&current_greeks);

        debug!(
            "Current delta: {:.4}, gap: {:.4}",
            current_greeks.delta, delta_gap
        );

        let mut best_plan: Option<AdjustmentPlan> = None;

        // Strategy 1: Adjust existing legs only
        if self.config.prefer_existing_legs
            && let Ok(plan) = self.optimize_existing_legs(delta_gap, gamma_gap)
        {
            trace!("Existing legs plan quality: {:.4}", plan.quality_score);
            best_plan = Some(plan);
        }

        // Strategy 2: Add new legs (if allowed and chain available)
        if self.config.allow_new_legs
            && self.chain.is_some()
            && let Ok(plan) = self.optimize_with_new_legs(delta_gap, gamma_gap)
        {
            trace!("New legs plan quality: {:.4}", plan.quality_score);
            if best_plan.is_none() || plan.quality_score < best_plan.as_ref().unwrap().quality_score
            {
                best_plan = Some(plan);
            }
        }

        // Strategy 3: Use underlying (if allowed and only delta target)
        if self.config.allow_underlying
            && gamma_gap.is_none()
            && let Ok(plan) = self.optimize_with_underlying(delta_gap)
        {
            trace!("Underlying plan quality: {:.4}", plan.quality_score);
            if best_plan.is_none() || plan.quality_score < best_plan.as_ref().unwrap().quality_score
            {
                best_plan = Some(plan);
            }
        }

        best_plan.ok_or(AdjustmentError::NoViablePlan)
    }

    /// Optimizes by adjusting existing leg quantities only.
    fn optimize_existing_legs(
        &self,
        delta_gap: Decimal,
        _gamma_gap: Option<Decimal>,
    ) -> Result<AdjustmentPlan, AdjustmentError> {
        let mut actions = Vec::new();
        let mut remaining_delta = delta_gap;

        // Sort legs by delta contribution (highest first)
        let mut legs_with_delta: Vec<(usize, Decimal, Decimal)> = self
            .positions
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                p.option.delta().ok().map(|d| {
                    let sign = if p.option.is_long() {
                        dec!(1)
                    } else {
                        dec!(-1)
                    };
                    let delta_per_contract = d / p.option.quantity.to_dec();
                    (i, d * sign, delta_per_contract * sign)
                })
            })
            .collect();

        legs_with_delta.sort_by(|a, b| b.2.abs().partial_cmp(&a.2.abs()).unwrap());

        // Greedily adjust quantities
        for (idx, _leg_delta, delta_per_contract) in legs_with_delta {
            if remaining_delta.abs() < self.config.delta_tolerance {
                break;
            }

            if delta_per_contract.abs() < dec!(0.001) {
                continue;
            }

            let current_qty = self.positions[idx].option.quantity.to_dec();
            let adjustment_qty = remaining_delta / delta_per_contract;

            // Calculate new quantity
            let new_qty = current_qty + adjustment_qty;

            // Can't have negative quantity
            if new_qty > Decimal::ZERO
                && let Ok(new_quantity) = Positive::new_decimal(new_qty)
            {
                actions.push(AdjustmentAction::ModifyQuantity {
                    leg_index: idx,
                    new_quantity,
                });
                remaining_delta -= adjustment_qty * delta_per_contract;
            }
        }

        self.build_plan(actions, remaining_delta)
    }

    /// Optimizes by adding new option legs from the chain.
    fn optimize_with_new_legs(
        &self,
        delta_gap: Decimal,
        _gamma_gap: Option<Decimal>,
    ) -> Result<AdjustmentPlan, AdjustmentError> {
        let chain = self.chain.ok_or(AdjustmentError::NoViablePlan)?;
        let mut actions = Vec::new();
        let mut remaining_delta = delta_gap;

        // Find candidate options
        let candidates = self.find_candidate_options(chain, delta_gap)?;

        let max_legs = self.config.max_new_legs.unwrap_or(2);

        for (legs_added, (option, quantity, option_delta)) in candidates.into_iter().enumerate() {
            if remaining_delta.abs() < self.config.delta_tolerance {
                break;
            }
            if legs_added >= max_legs {
                break;
            }

            let side = if delta_gap > Decimal::ZERO {
                Side::Long
            } else {
                Side::Short
            };

            actions.push(AdjustmentAction::AddLeg {
                option: Box::new(option),
                side,
                quantity,
            });

            remaining_delta -= option_delta * quantity.to_dec();
        }

        self.build_plan(actions, remaining_delta)
    }

    /// Optimizes using underlying shares only.
    fn optimize_with_underlying(
        &self,
        delta_gap: Decimal,
    ) -> Result<AdjustmentPlan, AdjustmentError> {
        // Each share has delta = 1
        let shares_needed = delta_gap;

        let actions = vec![AdjustmentAction::AddUnderlying {
            quantity: shares_needed,
        }];

        self.build_plan(actions, Decimal::ZERO)
    }

    /// Finds candidate options for delta adjustment.
    fn find_candidate_options(
        &self,
        chain: &OptionChain,
        delta_gap: Decimal,
    ) -> Result<Vec<(crate::Options, Positive, Decimal)>, AdjustmentError> {
        let mut candidates = Vec::new();
        let target_delta_sign = delta_gap.signum();

        for opt_data in chain.get_single_iter() {
            // Filter by liquidity
            if let Some(min_oi) = self.config.min_liquidity
                && opt_data.open_interest.unwrap_or(0) < min_oi
            {
                continue;
            }

            // Filter by strike range
            if let Some((min_strike, max_strike)) = &self.config.strike_range
                && (opt_data.strike_price < *min_strike || opt_data.strike_price > *max_strike)
            {
                continue;
            }

            // Try both call and put options
            for option_style in &self.config.allowed_styles {
                if let Ok(position) =
                    opt_data.get_position(Side::Long, *option_style, None, None, None)
                {
                    let option = position.option;
                    if let Ok(option_delta) = option.delta() {
                        // Check if this option helps reduce the gap
                        if option_delta.signum() == target_delta_sign {
                            let quantity_needed =
                                (delta_gap.abs() / option_delta.abs()).min(dec!(100));
                            if let Ok(qty) = Positive::new_decimal(quantity_needed) {
                                candidates.push((option, qty, option_delta));
                            }
                        }
                    }
                }
            }
        }

        // Sort by efficiency (delta per dollar cost)
        candidates.sort_by(|a, b| {
            let eff_a = a.2.abs()
                / a.0
                    .calculate_price_black_scholes()
                    .unwrap_or(dec!(1))
                    .max(dec!(0.01));
            let eff_b = b.2.abs()
                / b.0
                    .calculate_price_black_scholes()
                    .unwrap_or(dec!(1))
                    .max(dec!(0.01));
            eff_b.partial_cmp(&eff_a).unwrap()
        });

        Ok(candidates)
    }

    /// Builds an adjustment plan from actions.
    fn build_plan(
        &self,
        actions: Vec<AdjustmentAction>,
        residual_delta: Decimal,
    ) -> Result<AdjustmentPlan, AdjustmentError> {
        let cost = self.estimate_cost(&actions)?;

        // Validate cost constraint
        if let Some(max_cost) = &self.config.max_cost
            && cost > max_cost.to_dec()
        {
            return Err(AdjustmentError::CostExceeded);
        }

        // Calculate resulting Greeks
        let new_positions = self.apply_actions_preview(&actions)?;
        let resulting_greeks = PortfolioGreeks::from_positions(&new_positions)?;

        Ok(AdjustmentPlan::new(
            actions,
            cost,
            resulting_greeks,
            residual_delta,
        ))
    }

    /// Estimates the cost of a set of actions.
    fn estimate_cost(&self, actions: &[AdjustmentAction]) -> Result<Decimal, AdjustmentError> {
        let mut cost = Decimal::ZERO;

        for action in actions {
            match action {
                AdjustmentAction::AddLeg {
                    option, quantity, ..
                } => {
                    let price = option
                        .calculate_price_black_scholes()
                        .map_err(|e| AdjustmentError::GreeksError(e.to_string()))?;
                    cost += price * quantity.to_dec();
                }
                AdjustmentAction::AddUnderlying { quantity } => {
                    let spot = self
                        .positions
                        .first()
                        .map(|p| p.option.underlying_price.to_dec())
                        .unwrap_or(Decimal::ZERO);
                    cost += (spot * quantity).abs();
                }
                AdjustmentAction::RollStrike {
                    leg_index,
                    new_strike,
                    quantity,
                } => {
                    if let Some(pos) = self.positions.get(*leg_index) {
                        // Cost is the difference in option prices
                        let old_price = pos
                            .option
                            .calculate_price_black_scholes()
                            .unwrap_or(Decimal::ZERO);
                        // Approximate new price (simplified)
                        let new_price = old_price
                            * (dec!(1)
                                + (new_strike.to_dec() - pos.option.strike_price.to_dec())
                                    / pos.option.underlying_price.to_dec()
                                    * dec!(0.5));
                        cost += (new_price - old_price).abs() * quantity.to_dec();
                    }
                }
                AdjustmentAction::RollExpiration { quantity, .. } => {
                    // Approximate cost for rolling expiration
                    cost += quantity.to_dec() * dec!(0.10); // Simplified estimate
                }
                _ => {}
            }
        }

        Ok(cost)
    }

    /// Applies actions to positions for preview (doesn't modify original).
    fn apply_actions_preview(
        &self,
        actions: &[AdjustmentAction],
    ) -> Result<Vec<Position>, AdjustmentError> {
        let mut positions = self.positions.to_vec();

        for action in actions {
            match action {
                AdjustmentAction::ModifyQuantity {
                    leg_index,
                    new_quantity,
                } => {
                    if let Some(pos) = positions.get_mut(*leg_index) {
                        pos.option.quantity = *new_quantity;
                    }
                }
                AdjustmentAction::AddLeg {
                    option, quantity, ..
                } => {
                    let mut new_option = *option.clone();
                    new_option.quantity = *quantity;
                    positions.push(Position::new(
                        new_option,
                        Positive::ZERO,
                        chrono::Utc::now(),
                        Positive::ZERO,
                        Positive::ZERO,
                        None,
                        None,
                    ));
                }
                AdjustmentAction::CloseLeg { leg_index } => {
                    if *leg_index < positions.len() {
                        positions.remove(*leg_index);
                    }
                }
                _ => {}
            }
        }

        Ok(positions)
    }
}

#[cfg(test)]
mod tests_optimizer {
    use super::*;
    use crate::model::ExpirationDate;
    use crate::model::types::{OptionStyle, OptionType};

    fn create_test_option(
        strike: Positive,
        option_style: OptionStyle,
        side: Side,
        quantity: Positive,
    ) -> crate::Options {
        crate::Options::new(
            OptionType::European,
            side,
            "TEST".to_string(),
            strike,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.20),
            quantity,
            Positive::HUNDRED,
            dec!(0.05),
            option_style,
            Positive::ZERO,
            None,
        )
    }

    fn create_test_position(
        strike: Positive,
        option_style: OptionStyle,
        side: Side,
        quantity: Positive,
    ) -> Position {
        let option = create_test_option(strike, option_style, side, quantity);
        Position::new(
            option,
            Positive::TWO,
            chrono::Utc::now(),
            Positive::ZERO,
            Positive::ZERO,
            None,
            None,
        )
    }

    #[test]
    fn test_optimizer_no_positions() {
        let positions: Vec<Position> = vec![];
        let config = AdjustmentConfig::default();
        let target = AdjustmentTarget::delta_neutral();

        let optimizer = AdjustmentOptimizer::new(&positions, config, target);
        let result = optimizer.optimize();

        assert!(matches!(result, Err(AdjustmentError::NoPositions)));
    }

    #[test]
    fn test_optimizer_already_neutral() {
        // Create a delta-neutral position (long call + short call at same strike)
        let pos1 = create_test_position(
            Positive::HUNDRED,
            OptionStyle::Call,
            Side::Long,
            Positive::ONE,
        );
        let pos2 = create_test_position(
            Positive::HUNDRED,
            OptionStyle::Call,
            Side::Short,
            Positive::ONE,
        );
        let positions = vec![pos1, pos2];

        let config = AdjustmentConfig::default();
        let target = AdjustmentTarget::delta_neutral();

        let optimizer = AdjustmentOptimizer::new(&positions, config, target);
        let result = optimizer.optimize();

        assert!(result.is_ok());
        let plan = result.unwrap();
        assert!(plan.actions.is_empty() || plan.is_delta_neutral(dec!(0.01)));
    }

    #[test]
    fn test_optimizer_with_underlying() {
        let pos1 = create_test_position(
            Positive::HUNDRED,
            OptionStyle::Call,
            Side::Long,
            Positive::ONE,
        );
        let positions = vec![pos1];

        let config = AdjustmentConfig::with_underlying();
        let target = AdjustmentTarget::delta_neutral();

        let optimizer = AdjustmentOptimizer::new(&positions, config, target);
        let result = optimizer.optimize();

        assert!(result.is_ok());
    }

    #[test]
    fn test_optimizer_existing_legs_only() {
        let pos1 = create_test_position(
            Positive::HUNDRED,
            OptionStyle::Call,
            Side::Long,
            Positive::TWO,
        );
        let pos2 = create_test_position(
            pos_or_panic!(110.0),
            OptionStyle::Put,
            Side::Long,
            Positive::ONE,
        );
        let positions = vec![pos1, pos2];

        let config = AdjustmentConfig::existing_legs_only();
        let target = AdjustmentTarget::delta_neutral();

        let optimizer = AdjustmentOptimizer::new(&positions, config, target);
        let result = optimizer.optimize();

        // Should either succeed or fail gracefully
        assert!(result.is_ok() || matches!(result, Err(AdjustmentError::NoViablePlan)));
    }

    #[test]
    fn test_adjustment_config_builder() {
        let config = AdjustmentConfig::default()
            .with_max_cost(pos_or_panic!(1000.0))
            .with_delta_tolerance(dec!(0.05))
            .with_strike_range(pos_or_panic!(90.0), pos_or_panic!(110.0));

        assert_eq!(config.max_cost, Some(pos_or_panic!(1000.0)));
        assert_eq!(config.delta_tolerance, dec!(0.05));
        assert_eq!(
            config.strike_range,
            Some((pos_or_panic!(90.0), pos_or_panic!(110.0)))
        );
    }
}
