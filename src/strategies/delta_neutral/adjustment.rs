/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Extended Adjustment Actions Module
//!
//! Provides extended adjustment actions for multi-leg option strategies,
//! enabling more flexible delta management beyond simple quantity adjustments.
//!
//! ## Overview
//!
//! This module extends the basic `DeltaAdjustment` enum with additional
//! action types that support:
//!
//! - Adding new option legs at different strikes
//! - Rolling positions to different strikes or expirations
//! - Closing individual legs
//! - Adding underlying positions for delta hedging
//!
//! ## Key Types
//!
//! - [`AdjustmentAction`]: Extended enum of possible adjustment actions
//! - [`AdjustmentConfig`]: Configuration for adjustment behavior
//! - [`AdjustmentPlan`]: Result of adjustment calculation with actions and metrics


use crate::error::GreeksError;
use crate::model::types::{OptionStyle, Side};
use crate::model::{ExpirationDate, Options};
use positive::{pos_or_panic, Positive};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;

use super::portfolio::PortfolioGreeks;

/// Extended adjustment actions for multi-leg strategies.
///
/// This enum provides a comprehensive set of actions that can be taken
/// to adjust option positions for delta neutrality or other Greek targets.
///
/// ## Variants
///
/// - `ModifyQuantity`: Adjust the quantity of an existing leg
/// - `AddLeg`: Add a new option leg to the strategy
/// - `CloseLeg`: Close/remove an existing leg
/// - `RollStrike`: Roll a position to a different strike price
/// - `RollExpiration`: Roll a position to a different expiration date
/// - `AddUnderlying`: Add underlying shares for delta hedging
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum AdjustmentAction {
    /// Modify the quantity of an existing leg.
    ///
    /// This is the simplest adjustment - just changing how many contracts
    /// are held in an existing position.
    ModifyQuantity {
        /// Index of the leg in the strategy's position list
        leg_index: usize,
        /// New quantity for the position
        new_quantity: Positive,
    },

    /// Add a new option leg to the strategy.
    ///
    /// This allows adding options at different strikes or with different
    /// characteristics than existing positions.
    AddLeg {
        /// The option to add (boxed to reduce enum size)
        option: Box<Options>,
        /// Side of the new position (Long or Short)
        side: Side,
        /// Quantity to add
        quantity: Positive,
    },

    /// Close/remove an existing leg entirely.
    ///
    /// Used when a position should be completely closed rather than adjusted.
    CloseLeg {
        /// Index of the leg to close
        leg_index: usize,
    },

    /// Roll a position to a different strike price.
    ///
    /// This closes the existing position and opens a new one at a different
    /// strike, useful for adjusting delta exposure while maintaining similar
    /// time characteristics.
    RollStrike {
        /// Index of the leg to roll
        leg_index: usize,
        /// New strike price
        new_strike: Positive,
        /// Quantity for the new position
        quantity: Positive,
    },

    /// Roll a position to a different expiration date.
    ///
    /// This closes the existing position and opens a new one with a different
    /// expiration, useful for managing time decay while maintaining similar
    /// strike characteristics.
    RollExpiration {
        /// Index of the leg to roll
        leg_index: usize,
        /// New expiration date
        new_expiration: ExpirationDate,
        /// Quantity for the new position
        quantity: Positive,
    },

    /// Add underlying shares for delta hedging.
    ///
    /// Each share has delta = 1, making this a simple way to adjust
    /// portfolio delta without adding option complexity.
    AddUnderlying {
        /// Quantity of shares (negative for short)
        quantity: Decimal,
    },
}

impl fmt::Display for AdjustmentAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdjustmentAction::ModifyQuantity {
                leg_index,
                new_quantity,
            } => {
                write!(f, "Modify leg {} to quantity {}", leg_index, new_quantity)
            }
            AdjustmentAction::AddLeg {
                option,
                side,
                quantity,
            } => {
                write!(
                    f,
                    "Add {} {} {} at strike {} (qty: {})",
                    side, option.option_style, option.option_type, option.strike_price, quantity
                )
            }
            AdjustmentAction::CloseLeg { leg_index } => {
                write!(f, "Close leg {}", leg_index)
            }
            AdjustmentAction::RollStrike {
                leg_index,
                new_strike,
                quantity,
            } => {
                write!(
                    f,
                    "Roll leg {} to strike {} (qty: {})",
                    leg_index, new_strike, quantity
                )
            }
            AdjustmentAction::RollExpiration {
                leg_index,
                new_expiration,
                quantity,
            } => {
                write!(
                    f,
                    "Roll leg {} to expiration {} (qty: {})",
                    leg_index, new_expiration, quantity
                )
            }
            AdjustmentAction::AddUnderlying { quantity } => {
                if quantity.is_sign_positive() {
                    write!(f, "Buy {} shares of underlying", quantity)
                } else {
                    write!(f, "Sell {} shares of underlying", quantity.abs())
                }
            }
        }
    }
}

/// Configuration for adjustment behavior.
///
/// This struct controls what types of adjustments are allowed and
/// sets constraints on the adjustment process.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdjustmentConfig {
    /// Allow adding new option legs
    pub allow_new_legs: bool,

    /// Allow using underlying shares for hedging
    pub allow_underlying: bool,

    /// Maximum number of new legs to add
    pub max_new_legs: Option<usize>,

    /// Allowed option styles for new legs
    pub allowed_styles: Vec<OptionStyle>,

    /// Strike range for new legs (min, max) relative to current price
    pub strike_range: Option<(Positive, Positive)>,

    /// Maximum cost for adjustments in currency units
    pub max_cost: Option<Positive>,

    /// Minimum option liquidity (open interest) for new legs
    pub min_liquidity: Option<u64>,

    /// Delta tolerance for considering position neutral
    pub delta_tolerance: Decimal,

    /// Whether to prefer adjusting existing legs over adding new ones
    pub prefer_existing_legs: bool,
}

impl Default for AdjustmentConfig {
    fn default() -> Self {
        Self {
            allow_new_legs: true,
            allow_underlying: false,
            max_new_legs: Some(2),
            allowed_styles: vec![OptionStyle::Call, OptionStyle::Put],
            strike_range: None,
            max_cost: None,
            min_liquidity: None,
            delta_tolerance: dec!(0.01),
            prefer_existing_legs: true,
        }
    }
}

impl AdjustmentConfig {
    /// Creates a configuration that only allows adjusting existing legs.
    #[must_use]
    pub fn existing_legs_only() -> Self {
        Self {
            allow_new_legs: false,
            allow_underlying: false,
            ..Default::default()
        }
    }

    /// Creates a configuration that allows underlying hedging.
    #[must_use]
    pub fn with_underlying() -> Self {
        Self {
            allow_underlying: true,
            ..Default::default()
        }
    }

    /// Creates a configuration for aggressive adjustment with new legs.
    #[must_use]
    pub fn aggressive() -> Self {
        Self {
            allow_new_legs: true,
            allow_underlying: true,
            max_new_legs: Some(4),
            prefer_existing_legs: false,
            ..Default::default()
        }
    }

    /// Sets the maximum cost constraint.
    #[must_use]
    pub fn with_max_cost(mut self, max_cost: Positive) -> Self {
        self.max_cost = Some(max_cost);
        self
    }

    /// Sets the delta tolerance.
    #[must_use]
    pub fn with_delta_tolerance(mut self, tolerance: Decimal) -> Self {
        self.delta_tolerance = tolerance;
        self
    }

    /// Sets the strike range for new legs.
    #[must_use]
    pub fn with_strike_range(mut self, min: Positive, max: Positive) -> Self {
        self.strike_range = Some((min, max));
        self
    }

    /// Sets whether new legs are allowed.
    #[must_use]
    pub fn with_allow_new_legs(mut self, allow: bool) -> Self {
        self.allow_new_legs = allow;
        self
    }

    /// Sets whether underlying hedging is allowed.
    #[must_use]
    pub fn with_allow_underlying(mut self, allow: bool) -> Self {
        self.allow_underlying = allow;
        self
    }

    /// Sets the maximum number of new legs.
    #[must_use]
    pub fn with_max_new_legs(mut self, max: usize) -> Self {
        self.max_new_legs = Some(max);
        self
    }

    /// Sets the minimum liquidity requirement.
    #[must_use]
    pub fn with_min_liquidity(mut self, min: u64) -> Self {
        self.min_liquidity = Some(min);
        self
    }

    /// Sets whether to prefer existing legs over new ones.
    #[must_use]
    pub fn with_prefer_existing_legs(mut self, prefer: bool) -> Self {
        self.prefer_existing_legs = prefer;
        self
    }
}

/// Result of adjustment calculation.
///
/// Contains the recommended actions to take along with metrics
/// about the expected outcome.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdjustmentPlan {
    /// Actions to execute in order
    pub actions: Vec<AdjustmentAction>,

    /// Estimated cost of adjustments in currency units
    pub estimated_cost: Decimal,

    /// Greeks after adjustment is applied
    pub resulting_greeks: PortfolioGreeks,

    /// Residual delta after adjustment
    pub residual_delta: Decimal,

    /// Quality score (lower is better)
    /// Combines residual delta and cost efficiency
    pub quality_score: Decimal,
}

impl AdjustmentPlan {
    /// Creates a new adjustment plan.
    pub fn new(
        actions: Vec<AdjustmentAction>,
        estimated_cost: Decimal,
        resulting_greeks: PortfolioGreeks,
        residual_delta: Decimal,
    ) -> Self {
        let quality_score = residual_delta.abs() + estimated_cost * dec!(0.01);
        Self {
            actions,
            estimated_cost,
            resulting_greeks,
            residual_delta,
            quality_score,
        }
    }

    /// Returns true if this plan achieves delta neutrality within tolerance.
    #[must_use]
    pub fn is_delta_neutral(&self, tolerance: Decimal) -> bool {
        self.residual_delta.abs() <= tolerance
    }

    /// Returns true if this plan has no actions.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    /// Returns the number of actions in the plan.
    #[must_use]
    pub fn action_count(&self) -> usize {
        self.actions.len()
    }
}

impl fmt::Display for AdjustmentPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Adjustment Plan:")?;
        writeln!(f, "  Actions: {}", self.actions.len())?;
        for (i, action) in self.actions.iter().enumerate() {
            writeln!(f, "    {}: {}", i + 1, action)?;
        }
        writeln!(f, "  Estimated Cost: {:.2}", self.estimated_cost)?;
        writeln!(f, "  Residual Delta: {:.4}", self.residual_delta)?;
        writeln!(f, "  Quality Score: {:.4}", self.quality_score)?;
        Ok(())
    }
}

/// Error types specific to adjustment operations.
#[derive(Debug, Clone, PartialEq)]
pub enum AdjustmentError {
    /// No viable adjustment plan could be found
    NoViablePlan,
    /// Cost constraint exceeded
    CostExceeded,
    /// No positions to adjust
    NoPositions,
    /// Invalid leg index
    InvalidLegIndex(usize),
    /// Greeks calculation failed
    GreeksError(String),
    /// Configuration constraint violated
    ConfigurationViolation(String),
}

impl fmt::Display for AdjustmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdjustmentError::NoViablePlan => write!(f, "No viable adjustment plan found"),
            AdjustmentError::CostExceeded => write!(f, "Adjustment cost exceeds maximum"),
            AdjustmentError::NoPositions => write!(f, "No positions to adjust"),
            AdjustmentError::InvalidLegIndex(idx) => write!(f, "Invalid leg index: {}", idx),
            AdjustmentError::GreeksError(msg) => write!(f, "Greeks calculation error: {}", msg),
            AdjustmentError::ConfigurationViolation(msg) => {
                write!(f, "Configuration violation: {}", msg)
            }
        }
    }
}

impl std::error::Error for AdjustmentError {}

impl From<GreeksError> for AdjustmentError {
    fn from(err: GreeksError) -> Self {
        AdjustmentError::GreeksError(err.to_string())
    }
}

#[cfg(test)]
mod tests_adjustment {
    use super::*;

    #[test]
    fn test_adjustment_config_default() {
        let config = AdjustmentConfig::default();
        assert!(config.allow_new_legs);
        assert!(!config.allow_underlying);
        assert_eq!(config.max_new_legs, Some(2));
        assert!(config.prefer_existing_legs);
    }

    #[test]
    fn test_adjustment_config_existing_legs_only() {
        let config = AdjustmentConfig::existing_legs_only();
        assert!(!config.allow_new_legs);
        assert!(!config.allow_underlying);
    }

    #[test]
    fn test_adjustment_config_with_underlying() {
        let config = AdjustmentConfig::with_underlying();
        assert!(config.allow_underlying);
    }

    #[test]
    fn test_adjustment_config_aggressive() {
        let config = AdjustmentConfig::aggressive();
        assert!(config.allow_new_legs);
        assert!(config.allow_underlying);
        assert_eq!(config.max_new_legs, Some(4));
        assert!(!config.prefer_existing_legs);
    }

    #[test]
    fn test_adjustment_action_display() {
        let modify = AdjustmentAction::ModifyQuantity {
            leg_index: 0,
            new_quantity: pos_or_panic!(5.0),
        };
        assert!(modify.to_string().contains("Modify leg 0"));

        let close = AdjustmentAction::CloseLeg { leg_index: 1 };
        assert!(close.to_string().contains("Close leg 1"));

        let underlying = AdjustmentAction::AddUnderlying {
            quantity: dec!(100.0),
        };
        assert!(underlying.to_string().contains("Buy 100"));

        let short_underlying = AdjustmentAction::AddUnderlying {
            quantity: dec!(-50.0),
        };
        assert!(short_underlying.to_string().contains("Sell 50"));
    }

    #[test]
    fn test_adjustment_plan_is_delta_neutral() {
        let plan = AdjustmentPlan::new(
            vec![],
            Decimal::ZERO,
            PortfolioGreeks::default(),
            dec!(0.005),
        );

        assert!(plan.is_delta_neutral(dec!(0.01)));
        assert!(!plan.is_delta_neutral(dec!(0.001)));
    }

    #[test]
    fn test_adjustment_error_display() {
        let err = AdjustmentError::NoViablePlan;
        assert!(err.to_string().contains("No viable"));

        let err = AdjustmentError::InvalidLegIndex(5);
        assert!(err.to_string().contains("5"));
    }
}
