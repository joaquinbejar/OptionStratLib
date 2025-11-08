/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/2/25
******************************************************************************/
use crate::{Positive, pos};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Represents various risk metrics for the options strategy
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RiskMetricsSimulation {
    /// Value at Risk (VaR) at 95% confidence level
    pub var_95: Decimal,

    /// Value at Risk (VaR) at 99% confidence level
    pub var_99: Decimal,

    /// Conditional Value at Risk (CVaR) at 95% confidence level (Expected Shortfall)
    pub cvar_95: Decimal,

    /// Probability of losing more than 50% of maximum investment
    pub severe_loss_probability: Positive,

    /// Maximum drawdown observed in simulations
    pub max_drawdown: Positive,

    /// Sharpe ratio (risk-adjusted return)
    pub sharpe_ratio: Decimal,
}

impl Default for RiskMetricsSimulation {
    fn default() -> Self {
        Self {
            var_95: dec!(0.0),
            var_99: dec!(0.0),
            cvar_95: dec!(0.0),
            severe_loss_probability: pos!(0.01),
            max_drawdown: pos!(0.01),
            sharpe_ratio: dec!(0.0),
        }
    }
}

/// Risk categories for options strategies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskCategory {
    /// Low risk strategies (e.g., covered calls with strong underlying)
    Low,

    /// Medium risk strategies (e.g., credit spreads)
    Medium,

    /// High risk strategies (e.g., naked options, speculative positions)
    High,
}

#[cfg(test)]
mod tests_risk_metrics {
    use super::*;
    use crate::pos;
    use rust_decimal_macros::dec;
    use serde_json;

    #[test]
    fn test_risk_metrics_simulation_default() {
        let metrics = RiskMetricsSimulation::default();

        assert_eq!(metrics.var_95, dec!(0.0));
        assert_eq!(metrics.var_99, dec!(0.0));
        assert_eq!(metrics.cvar_95, dec!(0.0));
        assert_eq!(metrics.severe_loss_probability, pos!(0.01));
        assert_eq!(metrics.max_drawdown, pos!(0.01));
        assert_eq!(metrics.sharpe_ratio, dec!(0.0));
    }

    #[test]
    fn test_risk_metrics_simulation_clone() {
        let metrics = RiskMetricsSimulation {
            var_95: dec!(-100.0),
            var_99: dec!(-150.0),
            cvar_95: dec!(-120.0),
            severe_loss_probability: pos!(0.15),
            max_drawdown: pos!(0.25),
            sharpe_ratio: dec!(1.2),
        };

        let cloned = metrics.clone();

        assert_eq!(cloned.var_95, dec!(-100.0));
        assert_eq!(cloned.var_99, dec!(-150.0));
        assert_eq!(cloned.cvar_95, dec!(-120.0));
        assert_eq!(cloned.severe_loss_probability, pos!(0.15));
        assert_eq!(cloned.max_drawdown, pos!(0.25));
        assert_eq!(cloned.sharpe_ratio, dec!(1.2));
    }

    #[test]
    fn test_risk_metrics_simulation_custom_values() {
        let metrics = RiskMetricsSimulation {
            var_95: dec!(-75.5),
            var_99: dec!(-120.25),
            cvar_95: dec!(-90.75),
            severe_loss_probability: pos!(0.22),
            max_drawdown: pos!(0.35),
            sharpe_ratio: dec!(0.88),
        };

        assert_eq!(metrics.var_95, dec!(-75.5));
        assert_eq!(metrics.var_99, dec!(-120.25));
        assert_eq!(metrics.cvar_95, dec!(-90.75));
        assert_eq!(metrics.severe_loss_probability, pos!(0.22));
        assert_eq!(metrics.max_drawdown, pos!(0.35));
        assert_eq!(metrics.sharpe_ratio, dec!(0.88));
    }

    #[test]
    fn test_risk_metrics_simulation_debug() {
        let metrics = RiskMetricsSimulation::default();
        let debug_string = format!("{metrics:?}");

        // Verify the debug output contains all relevant fields
        assert!(debug_string.contains("var_95"));
        assert!(debug_string.contains("var_99"));
        assert!(debug_string.contains("cvar_95"));
        assert!(debug_string.contains("severe_loss_probability"));
        assert!(debug_string.contains("max_drawdown"));
        assert!(debug_string.contains("sharpe_ratio"));
    }

    #[test]
    fn test_risk_metrics_simulation_serialization() {
        let metrics = RiskMetricsSimulation {
            var_95: dec!(-50.0),
            var_99: dec!(-75.0),
            cvar_95: dec!(-60.0),
            severe_loss_probability: pos!(0.1),
            max_drawdown: pos!(0.2),
            sharpe_ratio: dec!(1.5),
        };

        let serialized = serde_json::to_string(&metrics).expect("Failed to serialize");
        let deserialized: RiskMetricsSimulation =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(deserialized.var_95, dec!(-50.0));
        assert_eq!(deserialized.var_99, dec!(-75.0));
        assert_eq!(deserialized.cvar_95, dec!(-60.0));
        assert_eq!(deserialized.severe_loss_probability, pos!(0.1));
        assert_eq!(deserialized.max_drawdown, pos!(0.2));
        assert_eq!(deserialized.sharpe_ratio, dec!(1.5));
    }

    #[test]
    fn test_risk_metrics_simulation_field_access() {
        let metrics = RiskMetricsSimulation {
            var_95: dec!(-30.0),
            var_99: dec!(-45.0),
            cvar_95: dec!(-35.0),
            severe_loss_probability: pos!(0.05),
            max_drawdown: pos!(0.15),
            sharpe_ratio: dec!(1.0),
        };

        // Verify modifications
        assert_eq!(metrics.var_95, dec!(-30.0));
        assert_eq!(metrics.var_99, dec!(-45.0));
        assert_eq!(metrics.cvar_95, dec!(-35.0));
        assert_eq!(metrics.severe_loss_probability, pos!(0.05));
        assert_eq!(metrics.max_drawdown, pos!(0.15));
        assert_eq!(metrics.sharpe_ratio, dec!(1.0));
    }

    #[test]
    fn test_risk_metrics_simulation_decimal_precision() {
        // Test with high precision decimal values
        let metrics = RiskMetricsSimulation {
            var_95: dec!(-123.45678),
            var_99: dec!(-234.56789),
            cvar_95: dec!(-345.67890),
            severe_loss_probability: pos!(0.12345),
            max_drawdown: pos!(0.23456),
            sharpe_ratio: dec!(1.23456),
        };

        // Verify precision is maintained
        assert_eq!(metrics.var_95, dec!(-123.45678));
        assert_eq!(metrics.var_99, dec!(-234.56789));
        assert_eq!(metrics.cvar_95, dec!(-345.67890));
        assert_eq!(metrics.severe_loss_probability, pos!(0.12345));
        assert_eq!(metrics.max_drawdown, pos!(0.23456));
        assert_eq!(metrics.sharpe_ratio, dec!(1.23456));
    }

    #[test]
    fn test_risk_metrics_simulation_zero_and_negative_values() {
        // Test with zero and negative values where appropriate
        let metrics = RiskMetricsSimulation {
            var_95: dec!(0.0),
            var_99: dec!(-200.0),
            cvar_95: dec!(-100.0),
            severe_loss_probability: pos!(0.0001), // Near-zero positive value
            max_drawdown: pos!(0.0001),            // Near-zero positive value
            sharpe_ratio: dec!(-0.5),              // Negative Sharpe ratio (poor performance)
        };

        assert_eq!(metrics.var_95, dec!(0.0));
        assert_eq!(metrics.var_99, dec!(-200.0));
        assert_eq!(metrics.cvar_95, dec!(-100.0));
        assert_eq!(metrics.severe_loss_probability, pos!(0.0001));
        assert_eq!(metrics.max_drawdown, pos!(0.0001));
        assert_eq!(metrics.sharpe_ratio, dec!(-0.5));
    }

    #[test]
    fn test_risk_category_basic() {
        // Test basic enum values
        let low = RiskCategory::Low;
        let medium = RiskCategory::Medium;
        let high = RiskCategory::High;

        // Verify equality
        assert_eq!(low, RiskCategory::Low);
        assert_eq!(medium, RiskCategory::Medium);
        assert_eq!(high, RiskCategory::High);

        // Verify inequality
        assert_ne!(low, medium);
        assert_ne!(medium, high);
        assert_ne!(low, high);
    }

    #[test]
    fn test_risk_category_clone() {
        let original = RiskCategory::Medium;
        let cloned = original.clone();

        assert_eq!(original, cloned);
    }

    #[test]
    fn test_risk_category_debug() {
        assert_eq!(format!("{:?}", RiskCategory::Low), "Low");
        assert_eq!(format!("{:?}", RiskCategory::Medium), "Medium");
        assert_eq!(format!("{:?}", RiskCategory::High), "High");
    }

    #[test]
    fn test_risk_category_serialization() {
        // Test all variants
        let categories = vec![RiskCategory::Low, RiskCategory::Medium, RiskCategory::High];

        for category in categories {
            let serialized = serde_json::to_string(&category).expect("Failed to serialize");
            let deserialized: RiskCategory =
                serde_json::from_str(&serialized).expect("Failed to deserialize");

            assert_eq!(category, deserialized);
        }
    }

    #[test]
    fn test_risk_category_specific_serialization_format() {
        let serialized = serde_json::to_string(&RiskCategory::Low).unwrap();
        assert_eq!(serialized, "\"Low\"");

        let serialized = serde_json::to_string(&RiskCategory::Medium).unwrap();
        assert_eq!(serialized, "\"Medium\"");

        let serialized = serde_json::to_string(&RiskCategory::High).unwrap();
        assert_eq!(serialized, "\"High\"");
    }

    #[test]
    fn test_risk_category_deserialization() {
        let json_low = "\"Low\"";
        let json_medium = "\"Medium\"";
        let json_high = "\"High\"";

        let deserialized_low: RiskCategory = serde_json::from_str(json_low).unwrap();
        let deserialized_medium: RiskCategory = serde_json::from_str(json_medium).unwrap();
        let deserialized_high: RiskCategory = serde_json::from_str(json_high).unwrap();

        assert_eq!(deserialized_low, RiskCategory::Low);
        assert_eq!(deserialized_medium, RiskCategory::Medium);
        assert_eq!(deserialized_high, RiskCategory::High);
    }

    #[test]
    fn test_risk_category_invalid_deserialization() {
        let result = serde_json::from_str::<RiskCategory>("\"InvalidCategory\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_risk_category_match_patterns() {
        let category = RiskCategory::Medium;

        let description = match category {
            RiskCategory::Low => "low risk",
            RiskCategory::Medium => "medium risk",
            RiskCategory::High => "high risk",
        };

        assert_eq!(description, "medium risk");
    }

    #[test]
    fn test_risk_category_exhaustive_matching() {
        // Ensure all variants can be correctly matched
        fn get_numeric_risk_level(category: &RiskCategory) -> u8 {
            match category {
                RiskCategory::Low => 1,
                RiskCategory::Medium => 2,
                RiskCategory::High => 3,
            }
        }

        assert_eq!(get_numeric_risk_level(&RiskCategory::Low), 1);
        assert_eq!(get_numeric_risk_level(&RiskCategory::Medium), 2);
        assert_eq!(get_numeric_risk_level(&RiskCategory::High), 3);
    }
}
