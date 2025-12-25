//! Exit policies for option trading strategies.
//!
//! This module defines various exit conditions and policies that can be used
//! to determine when to close an option position during simulations or live trading.
//!
//! # Exit Policies
//!
//! - **Percentage-based**: Exit when profit/loss reaches a percentage threshold
//! - **Fixed price**: Exit when the option premium reaches a specific price
//! - **Time-based**: Exit after a certain time period or at expiration
//! - **Delta-based**: Exit when the option's delta crosses a threshold
//! - **Underlying price**: Exit when the underlying asset reaches a price level
//! - **Combined conditions**: Exit when multiple conditions are met (AND/OR logic)

use crate::Positive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;

/// Defines exit policies for option positions.
///
/// Exit policies determine when a position should be closed based on various
/// market conditions, profit/loss levels, or time constraints.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default, ToSchema)]
pub enum ExitPolicy {
    /// Exit when profit reaches a percentage of initial premium.
    ///
    /// For short options: exit when premium decreases by this percentage.
    /// For long options: exit when premium increases by this percentage.
    ///
    /// # Example
    /// ```ignore
    /// // Exit when profit reaches 50% of initial premium
    /// ExitPolicy::ProfitPercent(dec!(0.5))
    /// ```
    ProfitPercent(Decimal),

    /// Exit when loss reaches a percentage of initial premium.
    ///
    /// For short options: exit when premium increases by this percentage.
    /// For long options: exit when premium decreases by this percentage.
    ///
    /// # Example
    /// ```ignore
    /// // Exit when loss reaches 100% of initial premium (double)
    /// ExitPolicy::LossPercent(dec!(1.0))
    /// ```
    LossPercent(Decimal),

    /// Exit when the option premium reaches a specific price.
    ///
    /// # Example
    /// ```ignore
    /// // Exit when premium reaches $50
    /// ExitPolicy::FixedPrice(pos!(50.0))
    /// ```
    FixedPrice(Positive),

    /// Exit when the option premium falls below a minimum price.
    ///
    /// Useful for taking profits on short options when premium decays.
    ///
    /// # Example
    /// ```ignore
    /// // Exit when premium falls below $5
    /// ExitPolicy::MinPrice(pos!(5.0))
    /// ```
    MinPrice(Positive),

    /// Exit when the option premium exceeds a maximum price.
    ///
    /// Useful for cutting losses on short options when premium increases.
    ///
    /// # Example
    /// ```ignore
    /// // Exit when premium exceeds $100
    /// ExitPolicy::MaxPrice(pos!(100.0))
    /// ```
    MaxPrice(Positive),

    /// Exit after a specific number of time steps.
    ///
    /// # Example
    /// ```ignore
    /// // Exit after 1000 steps (e.g., 1000 minutes)
    /// ExitPolicy::TimeSteps(1000)
    /// ```
    TimeSteps(usize),

    /// Exit when days to expiration falls below a threshold.
    ///
    /// # Example
    /// ```ignore
    /// // Exit when less than 2 days remain
    /// ExitPolicy::DaysToExpiration(pos!(2.0))
    /// ```
    DaysToExpiration(Positive),

    /// Exit when the option's delta crosses a threshold.
    ///
    /// # Example
    /// ```ignore
    /// // Exit when delta exceeds 0.5 (deep in the money)
    /// ExitPolicy::DeltaThreshold(dec!(0.5))
    /// ```
    DeltaThreshold(Decimal),

    /// Exit when the underlying price reaches a specific level.
    ///
    /// # Example
    /// ```ignore
    /// // Exit when underlying reaches $4000
    /// ExitPolicy::UnderlyingPrice(pos!(4000.0))
    /// ```
    UnderlyingPrice(Positive),

    /// Exit when the underlying price falls below a level.
    ///
    /// # Example
    /// ```ignore
    /// // Exit when underlying falls below $3900
    /// ExitPolicy::UnderlyingBelow(pos!(3900.0))
    /// ```
    UnderlyingBelow(Positive),

    /// Exit when the underlying price exceeds a level.
    ///
    /// # Example
    /// ```ignore
    /// // Exit when underlying exceeds $4100
    /// ExitPolicy::UnderlyingAbove(pos!(4100.0))
    /// ```
    UnderlyingAbove(Positive),

    /// Exit at expiration (hold until expiration).
    ///
    /// # Example
    /// ```ignore
    /// ExitPolicy::Expiration
    /// ```
    #[default]
    Expiration,

    /// Exit when ALL conditions are met (AND logic).
    ///
    /// # Example
    /// ```ignore
    /// // Exit when profit is 50% AND at least 2 days have passed
    /// ExitPolicy::And(vec![
    ///     ExitPolicy::ProfitPercent(dec!(0.5)),
    ///     ExitPolicy::TimeSteps(2880), // 2 days in minutes
    /// ])
    /// ```
    And(Vec<ExitPolicy>),

    /// Exit when ANY condition is met (OR logic).
    ///
    /// # Example
    /// ```ignore
    /// // Exit when profit is 50% OR loss is 100%
    /// ExitPolicy::Or(vec![
    ///     ExitPolicy::ProfitPercent(dec!(0.5)),
    ///     ExitPolicy::LossPercent(dec!(1.0)),
    /// ])
    /// ```
    Or(Vec<ExitPolicy>),
}

impl ExitPolicy {
    /// Creates a standard profit target exit policy.
    ///
    /// # Arguments
    ///
    /// * `percent` - Profit percentage (e.g., 0.5 for 50%)
    ///
    /// # Returns
    ///
    /// An `ExitPolicy::ProfitPercent` variant.
    #[must_use]
    pub fn profit_target(percent: Decimal) -> Self {
        Self::ProfitPercent(percent)
    }

    /// Creates a standard stop loss exit policy.
    ///
    /// # Arguments
    ///
    /// * `percent` - Loss percentage (e.g., 1.0 for 100%)
    ///
    /// # Returns
    ///
    /// An `ExitPolicy::LossPercent` variant.
    #[must_use]
    pub fn stop_loss(percent: Decimal) -> Self {
        Self::LossPercent(percent)
    }

    /// Creates a combined profit target and stop loss policy.
    ///
    /// # Arguments
    ///
    /// * `profit_percent` - Profit percentage threshold
    /// * `loss_percent` - Loss percentage threshold
    ///
    /// # Returns
    ///
    /// An `ExitPolicy::Or` variant combining both conditions.
    #[must_use]
    pub fn profit_or_loss(profit_percent: Decimal, loss_percent: Decimal) -> Self {
        Self::Or(vec![
            Self::ProfitPercent(profit_percent),
            Self::LossPercent(loss_percent),
        ])
    }

    /// Creates a time-limited profit target policy.
    ///
    /// Exit when profit target is reached OR time limit is exceeded.
    ///
    /// # Arguments
    ///
    /// * `profit_percent` - Profit percentage threshold
    /// * `max_steps` - Maximum number of time steps
    ///
    /// # Returns
    ///
    /// An `ExitPolicy::Or` variant combining both conditions.
    #[must_use]
    pub fn profit_or_time(profit_percent: Decimal, max_steps: usize) -> Self {
        Self::Or(vec![
            Self::ProfitPercent(profit_percent),
            Self::TimeSteps(max_steps),
        ])
    }

    /// Checks if this is a composite policy (And/Or).
    ///
    /// # Returns
    ///
    /// `true` if the policy contains nested policies.
    #[must_use]
    pub const fn is_composite(&self) -> bool {
        matches!(self, Self::And(_) | Self::Or(_))
    }

    /// Returns the number of conditions in this policy.
    ///
    /// For simple policies, returns 1.
    /// For composite policies, returns the total number of leaf conditions.
    ///
    /// # Returns
    ///
    /// The count of conditions.
    #[must_use]
    pub fn condition_count(&self) -> usize {
        match self {
            Self::And(policies) | Self::Or(policies) => {
                policies.iter().map(Self::condition_count).sum()
            }
            _ => 1,
        }
    }
}

impl fmt::Display for ExitPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProfitPercent(pct) => {
                write!(f, "Profit Target: {:.1}%", pct * Decimal::from(100))
            }
            Self::LossPercent(pct) => write!(f, "Stop Loss: {:.1}%", pct * Decimal::from(100)),
            Self::FixedPrice(price) => write!(f, "Fixed Price: ${price}"),
            Self::MinPrice(price) => write!(f, "Min Price: ${price}"),
            Self::MaxPrice(price) => write!(f, "Max Price: ${price}"),
            Self::TimeSteps(steps) => write!(f, "Time Steps: {steps}"),
            Self::DaysToExpiration(days) => write!(f, "Days to Expiration: {days}"),
            Self::DeltaThreshold(delta) => write!(f, "Delta Threshold: {delta}"),
            Self::UnderlyingPrice(price) => write!(f, "Underlying Price: ${price}"),
            Self::UnderlyingBelow(price) => write!(f, "Underlying Below: ${price}"),
            Self::UnderlyingAbove(price) => write!(f, "Underlying Above: ${price}"),
            Self::Expiration => write!(f, "Hold to Expiration"),
            Self::And(policies) => {
                write!(f, "AND(")?;
                for (i, policy) in policies.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{policy}")?;
                }
                write!(f, ")")
            }
            Self::Or(policies) => {
                write!(f, "OR(")?;
                for (i, policy) in policies.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{policy}")?;
                }
                write!(f, ")")
            }
        }
    }
}

/// Checks if an exit policy is triggered for an option position.
///
/// # Parameters
///
/// * `policy` - The exit policy to evaluate
/// * `initial_premium` - The initial premium (paid for long, received for short)
/// * `current_premium` - The current premium value
/// * `step_num` - Current step number
/// * `days_left` - Days remaining to expiration
/// * `underlying_price` - Current underlying price
/// * `is_long` - `true` for long positions (bought), `false` for short positions (sold)
///
/// # Returns
///
/// `Some(policy)` if the policy is triggered, `None` otherwise
///
/// # Logic
///
/// For **Long positions** (bought options):
/// - Profit: current_premium > initial_premium (premium increases)
/// - Loss: current_premium < initial_premium (premium decreases)
///
/// For **Short positions** (sold options):
/// - Profit: current_premium < initial_premium (premium decreases)
/// - Loss: current_premium > initial_premium (premium increases)
#[allow(clippy::only_used_in_recursion)]
pub fn check_exit_policy(
    policy: &ExitPolicy,
    initial_premium: Decimal,
    current_premium: Decimal,
    step_num: usize,
    days_left: Positive,
    underlying_price: Positive,
    is_long: bool,
) -> Option<ExitPolicy> {
    match policy {
        ExitPolicy::ProfitPercent(pct) => {
            if is_long {
                // For long: profit when premium increases
                let target = initial_premium * (Decimal::ONE + pct);
                if current_premium >= target {
                    Some(ExitPolicy::ProfitPercent(*pct))
                } else {
                    None
                }
            } else {
                // For short: profit when premium decreases
                let target = initial_premium * (Decimal::ONE - pct);
                if current_premium <= target {
                    Some(ExitPolicy::ProfitPercent(*pct))
                } else {
                    None
                }
            }
        }
        ExitPolicy::LossPercent(pct) => {
            if is_long {
                // For long: loss when premium decreases
                let limit = initial_premium * (Decimal::ONE - pct);
                if current_premium <= limit {
                    Some(ExitPolicy::LossPercent(*pct))
                } else {
                    None
                }
            } else {
                // For short: loss when premium increases
                let limit = initial_premium * (Decimal::ONE + pct);
                if current_premium >= limit {
                    Some(ExitPolicy::LossPercent(*pct))
                } else {
                    None
                }
            }
        }
        ExitPolicy::FixedPrice(price) => {
            if (current_premium - price.to_dec()).abs() < dec!(0.01) {
                Some(ExitPolicy::FixedPrice(*price))
            } else {
                None
            }
        }
        ExitPolicy::MinPrice(price) => {
            if current_premium <= price.to_dec() {
                Some(ExitPolicy::MinPrice(*price))
            } else {
                None
            }
        }
        ExitPolicy::MaxPrice(price) => {
            if current_premium >= price.to_dec() {
                Some(ExitPolicy::MaxPrice(*price))
            } else {
                None
            }
        }
        ExitPolicy::TimeSteps(steps) => {
            if step_num >= *steps {
                Some(ExitPolicy::TimeSteps(*steps))
            } else {
                None
            }
        }
        ExitPolicy::DaysToExpiration(days) => {
            if days_left <= *days {
                Some(ExitPolicy::DaysToExpiration(*days))
            } else {
                None
            }
        }
        ExitPolicy::UnderlyingPrice(price) => {
            if (underlying_price.to_dec() - price.to_dec()).abs() < dec!(0.01) {
                Some(ExitPolicy::UnderlyingPrice(*price))
            } else {
                None
            }
        }
        ExitPolicy::UnderlyingBelow(price) => {
            if underlying_price < *price {
                Some(ExitPolicy::UnderlyingBelow(*price))
            } else {
                None
            }
        }
        ExitPolicy::UnderlyingAbove(price) => {
            if underlying_price > *price {
                Some(ExitPolicy::UnderlyingAbove(*price))
            } else {
                None
            }
        }
        ExitPolicy::Expiration => None,        // Handled separately
        ExitPolicy::DeltaThreshold(_) => None, // Not implemented in this example
        ExitPolicy::And(policies) => {
            let mut triggered = Vec::new();
            for p in policies {
                if let Some(triggered_policy) = check_exit_policy(
                    p,
                    initial_premium,
                    current_premium,
                    step_num,
                    days_left,
                    underlying_price,
                    is_long,
                ) {
                    triggered.push(triggered_policy);
                } else {
                    return None; // All must be true for AND
                }
            }
            Some(ExitPolicy::And(triggered))
        }
        ExitPolicy::Or(policies) => {
            for p in policies {
                if let Some(triggered_policy) = check_exit_policy(
                    p,
                    initial_premium,
                    current_premium,
                    step_num,
                    days_left,
                    underlying_price,
                    is_long,
                ) {
                    return Some(triggered_policy);
                }
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rust_decimal_macros::dec;

    #[test]
    fn test_profit_target_creation() {
        let policy = ExitPolicy::profit_target(dec!(0.5));
        assert_eq!(policy, ExitPolicy::ProfitPercent(dec!(0.5)));
    }

    #[test]
    fn test_stop_loss_creation() {
        let policy = ExitPolicy::stop_loss(dec!(1.0));
        assert_eq!(policy, ExitPolicy::LossPercent(dec!(1.0)));
    }

    #[test]
    fn test_profit_or_loss_creation() {
        let policy = ExitPolicy::profit_or_loss(dec!(0.5), dec!(1.0));
        match policy {
            ExitPolicy::Or(policies) => {
                assert_eq!(policies.len(), 2);
                assert_eq!(policies[0], ExitPolicy::ProfitPercent(dec!(0.5)));
                assert_eq!(policies[1], ExitPolicy::LossPercent(dec!(1.0)));
            }
            _ => panic!("Expected Or variant"),
        }
    }

    #[test]
    fn test_profit_or_time_creation() {
        let policy = ExitPolicy::profit_or_time(dec!(0.5), 1000);
        match policy {
            ExitPolicy::Or(policies) => {
                assert_eq!(policies.len(), 2);
                assert_eq!(policies[0], ExitPolicy::ProfitPercent(dec!(0.5)));
                assert_eq!(policies[1], ExitPolicy::TimeSteps(1000));
            }
            _ => panic!("Expected Or variant"),
        }
    }

    #[test]
    fn test_is_composite() {
        let simple = ExitPolicy::ProfitPercent(dec!(0.5));
        assert!(!simple.is_composite());

        let composite = ExitPolicy::Or(vec![
            ExitPolicy::ProfitPercent(dec!(0.5)),
            ExitPolicy::LossPercent(dec!(1.0)),
        ]);
        assert!(composite.is_composite());
    }

    #[test]
    fn test_condition_count() {
        let simple = ExitPolicy::ProfitPercent(dec!(0.5));
        assert_eq!(simple.condition_count(), 1);

        let composite = ExitPolicy::Or(vec![
            ExitPolicy::ProfitPercent(dec!(0.5)),
            ExitPolicy::LossPercent(dec!(1.0)),
        ]);
        assert_eq!(composite.condition_count(), 2);

        let nested = ExitPolicy::And(vec![
            ExitPolicy::Or(vec![
                ExitPolicy::ProfitPercent(dec!(0.5)),
                ExitPolicy::LossPercent(dec!(1.0)),
            ]),
            ExitPolicy::TimeSteps(1000),
        ]);
        assert_eq!(nested.condition_count(), 3);
    }

    #[test]
    fn test_display_profit_percent() {
        let policy = ExitPolicy::ProfitPercent(dec!(0.5));
        assert_eq!(format!("{policy}"), "Profit Target: 50.0%");
    }

    #[test]
    fn test_display_loss_percent() {
        let policy = ExitPolicy::LossPercent(dec!(1.0));
        assert_eq!(format!("{policy}"), "Stop Loss: 100.0%");
    }

    #[test]
    fn test_display_fixed_price() {
        let policy = ExitPolicy::FixedPrice(pos!(50.0));
        assert_eq!(format!("{policy}"), "Fixed Price: $50");
    }

    #[test]
    fn test_display_time_steps() {
        let policy = ExitPolicy::TimeSteps(1000);
        assert_eq!(format!("{policy}"), "Time Steps: 1000");
    }

    #[test]
    fn test_display_expiration() {
        let policy = ExitPolicy::Expiration;
        assert_eq!(format!("{policy}"), "Hold to Expiration");
    }

    #[test]
    fn test_display_or_composite() {
        let policy = ExitPolicy::Or(vec![
            ExitPolicy::ProfitPercent(dec!(0.5)),
            ExitPolicy::LossPercent(dec!(1.0)),
        ]);
        let display = format!("{policy}");
        assert!(display.contains("OR("));
        assert!(display.contains("Profit Target: 50.0%"));
        assert!(display.contains("Stop Loss: 100.0%"));
    }

    #[test]
    fn test_display_and_composite() {
        let policy = ExitPolicy::And(vec![
            ExitPolicy::ProfitPercent(dec!(0.5)),
            ExitPolicy::TimeSteps(1000),
        ]);
        let display = format!("{policy}");
        assert!(display.contains("AND("));
        assert!(display.contains("Profit Target: 50.0%"));
        assert!(display.contains("Time Steps: 1000"));
    }

    #[test]
    fn test_serialization() {
        let policy = ExitPolicy::profit_or_loss(dec!(0.5), dec!(1.0));
        let json = serde_json::to_string(&policy).unwrap();
        assert!(json.contains("Or"));
        assert!(json.contains("ProfitPercent"));
        assert!(json.contains("LossPercent"));
    }

    #[test]
    fn test_deserialization() {
        let json = r#"{"ProfitPercent":"0.5"}"#;
        let policy: ExitPolicy = serde_json::from_str(json).unwrap();
        assert_eq!(policy, ExitPolicy::ProfitPercent(dec!(0.5)));
    }

    #[test]
    fn test_all_variants() {
        let policies = vec![
            ExitPolicy::ProfitPercent(dec!(0.5)),
            ExitPolicy::LossPercent(dec!(1.0)),
            ExitPolicy::FixedPrice(pos!(50.0)),
            ExitPolicy::MinPrice(pos!(5.0)),
            ExitPolicy::MaxPrice(pos!(100.0)),
            ExitPolicy::TimeSteps(1000),
            ExitPolicy::DaysToExpiration(pos!(2.0)),
            ExitPolicy::DeltaThreshold(dec!(0.5)),
            ExitPolicy::UnderlyingPrice(pos!(4000.0)),
            ExitPolicy::UnderlyingBelow(pos!(3900.0)),
            ExitPolicy::UnderlyingAbove(pos!(4100.0)),
            ExitPolicy::Expiration,
        ];

        for policy in policies {
            // Ensure all variants can be displayed
            let _ = format!("{policy}");
            // Ensure all variants can be cloned
            let _ = policy.clone();
        }
    }
}
