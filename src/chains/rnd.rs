//! # Risk Neutral Density (RND) Analysis Module
//!
//! This module implements functionality to calculate and analyze the Risk-Neutral Density (RND)
//! from option chains. The RND represents the market's implied probability distribution of
//! future asset prices and is a powerful tool for understanding market expectations.
//!
//! ## Theory and Background
//!
//! The Risk-Neutral Density (RND) is a probability distribution that represents the market's
//! view of possible future prices of an underlying asset, derived from option prices. It is
//! "risk-neutral" because it incorporates both the market's expectations and risk preferences
//! into a single distribution.
//!
//! Key aspects of RND:
//! - Extracted from option prices using the Breeden-Litzenberger formula
//! - Provides insights into market sentiment and expected volatility
//! - Used for pricing exotic derivatives and risk assessment
//!
//! ## Statistical Moments and Their Interpretation
//!
//! The module calculates four key statistical moments:
//!
//! 1. **Mean**: The expected future price of the underlying asset
//! 2. **Variance**: Measure of price dispersion, related to expected volatility
//! 3. **Skewness**: Indicates asymmetry in price expectations
//!    - Positive skew: Market expects upside potential
//!    - Negative skew: Market expects downside risks
//! 4. **Kurtosis**: Measures the likelihood of extreme events
//!    - High kurtosis: Market expects "fat tails" (more extreme moves)
//!    - Low kurtosis: Market expects more moderate price movements
//!
//! ## Usage Example
//!
//! ```rust
//! use rust_decimal::Decimal;
//! use rust_decimal_macros::dec;
//! use tracing::info;
//! use optionstratlib::chains::{RNDParameters, RNDAnalysis};
//! use optionstratlib::chains::chain::OptionChain;
//! use optionstratlib::{pos, spos, ExpirationDate, Positive};
//! use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
//!
//! // Create parameters for RND calculation
//! let params = RNDParameters {
//!     risk_free_rate: dec!(0.05),
//!     interpolation_points: 100,
//!     derivative_tolerance: pos!(0.001),
//! };
//! let option_chain_params = OptionChainBuildParams::new(
//!             "SP500".to_string(),
//!             None,
//!             10,
//!             spos!(1.0),
//!             dec!(-0.2),
//!             dec!(0.00001),
//!             pos!(0.02),
//!             2,
//!             OptionDataPriceParams::new(
//!                 Some(Box::new(pos!(100.0))),
//!                 Some(ExpirationDate::Days(pos!(30.0))),
//!                 Some(Decimal::ZERO),
//!                 spos!(0.05),
//!                 Some("SP500".to_string()),
//!             ),
//!             pos!(0.1),
//!         );
//!
//! let option_chain = OptionChain::build_chain(&option_chain_params);
//! // Calculate RND from option chain
//! let rnd_result = option_chain.calculate_rnd(&params).unwrap();
//!
//! // Access statistical moments
//! info!("Expected price: {}", rnd_result.statistics.mean);
//! info!("Implied volatility: {}", rnd_result.statistics.variance.sqrt());
//! info!("Market bias: {}", rnd_result.statistics.skewness);
//! info!("Tail risk: {}", rnd_result.statistics.kurtosis);
//! ```
//!
//! ## Market Insights from RND
//!
//! The RND provides several valuable insights:
//!
//! 1. **Price Expectations**
//!    - Mean indicates the market's expected future price
//!    - Variance shows uncertainty around this expectation
//!
//! 2. **Market Sentiment**
//!    - Skewness reveals directional bias
//!    - Kurtosis indicates expected market stability
//!
//! 3. **Risk Assessment**
//!    - Shape of distribution helps quantify various risks
//!    - Particularly useful for stress testing and VaR calculations
//!
//! 4. **Volatility Structure**
//!    - Implied volatility skew analysis
//!    - Term structure of market expectations
//!
//! ## Mathematical Foundation
//!
//! The RND is calculated using the Breeden-Litzenberger formula:
//!
//! ```text
//! q(K) = e^(rT) * (∂²C/∂K²)
//! ```
//!
//! Where:
//! - q(K) is the RND value at strike K
//! - r is the risk-free rate
//! - T is time to expiration
//! - C is the call option price
//! - ∂²C/∂K² is the second derivative with respect to strike
//!
//! ## Implementation Details
//!
//! The module implements:
//! - Numerical approximation of derivatives
//! - Statistical moment calculations
//! - Error handling for numerical stability
//! - Volatility skew analysis
//!
//! The implementation focuses on numerical stability and accurate moment calculations,
//! particularly for extreme market conditions.
use crate::Positive;
use crate::error::ChainError;
use pretty_simple_display::{DebugPretty, DisplaySimple};
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use utoipa::ToSchema;

/// Parameters for Risk-Neutral Density calculation
///
/// This structure holds all necessary parameters for calculating the Risk-Neutral Density (RND)
/// from option chain data.
///
/// # Parameters
/// * `risk_free_rate` - Risk-free interest rate used in the calculation
/// * `interpolation_points` - Number of points to use in interpolation between strikes
/// * `derivative_tolerance` - Numerical tolerance for derivative calculations
///
/// # Example
/// ```
/// use rust_decimal_macros::dec;
/// use optionstratlib::chains::RNDParameters;
/// use optionstratlib::pos;
/// let params = RNDParameters {
///     risk_free_rate: dec!(0.05),
///     interpolation_points: 100,
///     derivative_tolerance: pos!(0.001),
/// };
/// ```
#[derive(DebugPretty, DisplaySimple, Clone, ToSchema, Serialize, Deserialize)]
pub struct RNDParameters {
    /// Risk-free rate for calculations
    pub risk_free_rate: Decimal,
    /// Number of points to use in interpolation
    pub interpolation_points: usize,
    /// Tolerance for numerical derivatives
    pub derivative_tolerance: Positive,
}

impl Default for RNDParameters {
    fn default() -> Self {
        Self {
            risk_free_rate: Decimal::ZERO,
            interpolation_points: 100,
            derivative_tolerance: Positive::ZERO,
        }
    }
}

/// Results of Risk-Neutral Density calculation
///
/// Contains both the calculated density values and their statistical properties.
///
/// # Fields
/// * `densities` - Mapping of strike prices to their corresponding probability densities
/// * `statistics` - Statistical moments and properties of the distribution
///
/// # Notes
/// The densities represent the market's implied probability distribution of future prices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RNDResult {
    /// Mapping of strike prices to their corresponding densities
    pub densities: BTreeMap<Positive, Decimal>,
    /// Statistical moments of the distribution
    pub statistics: RNDStatistics,
}

/// Statistical properties of the Risk-Neutral Density
///
/// Contains the four main statistical moments that characterize the distribution.
///
/// # Fields
/// * `mean` - First moment, represents expected future price
/// * `variance` - Second central moment, measures price dispersion
/// * `skewness` - Third standardized moment, measures asymmetry
/// * `kurtosis` - Fourth standardized moment, measures tail thickness
///
/// # Interpretation
/// * Positive skewness indicates market expects upside potential
/// * Negative skewness indicates market expects downside risks
/// * High kurtosis indicates higher probability of extreme events
/// * Low kurtosis indicates more concentrated price expectations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RNDStatistics {
    /// Mean of the distribution
    pub mean: Decimal,
    /// Variance of the distribution
    pub variance: Positive,
    /// Skewness of the distribution
    pub skewness: Decimal,
    /// Kurtosis of the distribution
    pub kurtosis: Decimal,
    /// Volatility of the distribution
    pub volatility: Positive,
}

impl RNDStatistics {
    /// Creates new RNDStatistics by calculating all moments from density values
    ///
    /// # Arguments
    /// * `densities` - Map of strike prices to their corresponding densities
    ///
    /// # Returns
    /// New RNDStatistics instance with calculated moments
    pub fn new(densities: &BTreeMap<Positive, Decimal>) -> Self {
        let mean = Self::calculate_mean(densities);
        let variance = Self::calculate_variance(densities, mean);
        let skewness = Self::calculate_skewness(densities, mean, variance);
        let kurtosis = Self::calculate_kurtosis(densities, mean, variance);

        Self {
            mean,
            variance,
            skewness,
            kurtosis,
            volatility: variance.sqrt(),
        }
    }

    /// Calculates the mean (first moment) of the distribution
    ///
    /// # Arguments
    /// * `densities` - Map of strike prices to their corresponding densities
    ///
    /// # Returns
    /// Mean value as Decimal
    fn calculate_mean(densities: &BTreeMap<Positive, Decimal>) -> Decimal {
        let mut mean = Decimal::ZERO;
        let mut total_density = Decimal::ZERO;

        for (strike, density) in densities {
            mean += strike.to_dec() * density;
            total_density += density;
        }

        if !total_density.is_zero() {
            mean / total_density
        } else {
            Decimal::ZERO
        }
    }

    /// Calculates the variance (second central moment) of the distribution
    ///
    /// # Arguments
    /// * `densities` - Map of strike prices to their corresponding densities
    /// * `mean` - Previously calculated mean of the distribution
    ///
    /// # Returns
    /// Variance as a Positive value
    fn calculate_variance(densities: &BTreeMap<Positive, Decimal>, mean: Decimal) -> Positive {
        let mut variance = Decimal::ZERO;
        let mut total_density = Decimal::ZERO;

        for (strike, density) in densities {
            let strike_dec = strike.to_dec();
            let diff = strike_dec - mean;
            variance += diff * diff * density;
            total_density += density;
        }

        if !total_density.is_zero() {
            (variance / total_density).into()
        } else {
            Positive::ZERO
        }
    }

    /// Calculates the skewness (third standardized moment) of the distribution
    ///
    /// # Arguments
    /// * `densities` - Map of strike prices to their corresponding densities
    /// * `mean` - Previously calculated mean
    /// * `variance` - Previously calculated variance
    ///
    /// # Returns
    /// Skewness as Decimal
    fn calculate_skewness(
        densities: &BTreeMap<Positive, Decimal>,
        mean: Decimal,
        variance: Positive,
    ) -> Decimal {
        if variance == Positive::ZERO {
            return Decimal::ZERO;
        }

        let std_dev = variance.sqrt();
        let mut skewness = Decimal::ZERO;
        let mut total_density = Decimal::ZERO;

        for (strike, density) in densities {
            let strike_dec = strike.to_dec();
            let normalized_diff = (strike_dec - mean) / std_dev;
            skewness += normalized_diff * normalized_diff * normalized_diff * density;
            total_density += density;
        }

        if !total_density.is_zero() {
            skewness / total_density
        } else {
            Decimal::ZERO
        }
    }

    /// Calculates the kurtosis (fourth standardized moment) of the distribution
    ///
    /// Uses the excess kurtosis formula: (m4/σ⁴) - 3, where m4 is the fourth moment
    /// and σ is the standard deviation.
    ///
    /// # Arguments
    /// * `densities` - Map of strike prices to their corresponding densities
    /// * `mean` - Previously calculated mean
    /// * `variance` - Previously calculated variance
    ///
    /// # Returns
    /// Excess kurtosis as Decimal
    fn calculate_kurtosis(
        densities: &BTreeMap<Positive, Decimal>,
        mean: Decimal,
        variance: Positive,
    ) -> Decimal {
        if variance == Positive::ZERO {
            return Decimal::ZERO;
        }

        // Convert variance to decimal and calculate std_dev
        let variance_dec = variance.to_dec();
        let std_dev = variance_dec.sqrt().unwrap();
        let std_dev_4 = std_dev.powi(4);

        let mut fourth_moment = Decimal::ZERO;
        let mut total_density = Decimal::ZERO;

        // Calculate fourth moment
        for (strike, density) in densities {
            let diff = strike.to_dec() - mean;
            let term = diff.powi(4); // Using powi instead of manual multiplication
            fourth_moment += term * density;
            total_density += density;
        }

        // Normalize by total density first
        if !total_density.is_zero() {
            let normalized_fourth_moment = fourth_moment / total_density;
            // Then divide by std_dev^4 and subtract 3
            (normalized_fourth_moment / std_dev_4) - dec!(3.0)
        } else {
            Decimal::ZERO
        }
    }
}

impl RNDResult {
    /// Create a new RNDResult with calculated statistics
    pub fn new(densities: BTreeMap<Positive, Decimal>) -> Self {
        let statistics = RNDStatistics::new(&densities);
        Self {
            densities,
            statistics,
        }
    }
}

/// Trait defining Risk-Neutral Density analysis capabilities
///
/// This trait provides methods for calculating RND and analyzing volatility skew
/// from option chain data.
pub trait RNDAnalysis {
    /// Calculates the Risk-Neutral Density from the option chain
    ///
    /// Uses the Breeden-Litzenberger formula to extract implied probabilities
    /// from option prices.
    ///
    /// # Arguments
    /// * `params` - Parameters controlling the RND calculation
    ///
    /// # Returns
    /// Result containing either RNDResult or an error
    fn calculate_rnd(&self, params: &RNDParameters) -> Result<RNDResult, ChainError>;

    /// Calculates the implied volatility skew
    ///
    /// Analyzes how implied volatility varies across different strike prices,
    /// providing insight into market's price expectations.
    ///
    /// # Returns
    /// Result containing vector of (strike_price, volatility) pairs or an error
    fn calculate_skew(&self) -> Result<Vec<(Positive, Decimal)>, ChainError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chains::chain::OptionChain;
    use crate::pos;
    use crate::spos;
    use rust_decimal_macros::dec;

    // Helper functions for test data creation
    fn create_test_option_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

        // Add a range of options around the money
        for strike in [80.0, 90.0, 95.0, 100.0, 105.0, 110.0, 120.0].iter() {
            chain.add_option(
                pos!(*strike),
                spos!(15.0),
                spos!(15.5),
                spos!(5.0),
                spos!(5.5),
                pos!(0.2),
                Some(dec!(-0.3)),
                Some(dec!(-0.3)),
                Some(dec!(0.3)),
                spos!(100.0),
                Some(50),
                None,
            );
        }

        chain
    }

    fn create_empty_chain() -> OptionChain {
        OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None)
    }

    mod rnd_parameters_tests {
        use super::*;

        #[test]
        fn test_default_parameters() {
            let params = RNDParameters::default();
            assert_eq!(params.risk_free_rate, Decimal::ZERO);
            assert_eq!(params.interpolation_points, 100);
            assert_eq!(params.derivative_tolerance, Positive::ZERO);
        }

        #[test]
        fn test_custom_parameters() {
            let params = RNDParameters {
                risk_free_rate: dec!(0.05),
                interpolation_points: 200,
                derivative_tolerance: pos!(0.001),
            };
            assert_eq!(params.risk_free_rate, dec!(0.05));
            assert_eq!(params.interpolation_points, 200);
            assert_eq!(params.derivative_tolerance, pos!(0.001));
        }
    }

    mod rnd_statistics_tests {
        use super::*;
        use crate::assert_decimal_eq;

        fn create_test_densities() -> BTreeMap<Positive, Decimal> {
            let mut densities = BTreeMap::new();
            densities.insert(pos!(90.0), dec!(0.2));
            densities.insert(pos!(100.0), dec!(0.5));
            densities.insert(pos!(110.0), dec!(0.3));
            densities
        }

        #[test]
        fn test_calculate_mean_normal_case() {
            let densities = create_test_densities();
            let stats = RNDStatistics::new(&densities);
            assert_eq!(stats.mean, dec!(101));
        }

        #[test]
        fn test_calculate_mean_empty_densities() {
            let densities = BTreeMap::new();
            let stats = RNDStatistics::new(&densities);
            assert_eq!(stats.mean, Decimal::ZERO);
        }

        #[test]
        fn test_calculate_variance_normal_case() {
            let densities = create_test_densities();
            let stats = RNDStatistics::new(&densities);
            assert!(stats.variance > Positive::ZERO);
        }

        #[test]
        fn test_calculate_variance_empty_densities() {
            let densities = BTreeMap::new();
            let stats = RNDStatistics::new(&densities);
            assert_eq!(stats.variance, Positive::ZERO);
        }

        #[test]
        fn test_calculate_skewness_normal_case() {
            let densities = create_test_densities();
            let stats = RNDStatistics::new(&densities);
            // Skewness should be near zero for symmetric distribution
            assert_decimal_eq!(stats.skewness.abs(), dec!(0.139941), dec!(0.00001));
        }

        #[test]
        fn test_calculate_skewness_empty_densities() {
            let densities = BTreeMap::new();
            let stats = RNDStatistics::new(&densities);
            assert_eq!(stats.skewness, Decimal::ZERO);
        }

        #[test]
        fn test_calculate_kurtosis_normal_case() {
            let densities = create_test_densities();
            let stats = RNDStatistics::new(&densities);
            // Excess kurtosis should be near zero for normal-like distribution
            assert_decimal_eq!(stats.kurtosis.abs(), dec!(0.96043315), dec!(0.00001));
        }

        #[test]
        fn test_calculate_kurtosis_empty_densities() {
            let densities = BTreeMap::new();
            let stats = RNDStatistics::new(&densities);
            assert_eq!(stats.kurtosis, Decimal::ZERO);
        }

        #[test]
        fn test_calculate_volatility_normal_case() {
            let densities = create_test_densities();
            let stats = RNDStatistics::new(&densities);
            // Excess kurtosis should be near zero for normal-like distribution
            assert_decimal_eq!(stats.volatility.to_dec(), dec!(7.0), dec!(0.00001));
        }

        #[test]
        fn test_calculate_volatility_empty_densities() {
            let densities = BTreeMap::new();
            let stats = RNDStatistics::new(&densities);
            assert_eq!(stats.volatility.to_dec(), Decimal::ZERO);
        }
    }

    mod rnd_calculation_tests {
        use super::*;

        #[test]
        fn test_calculate_rnd_normal_case() {
            let chain = create_test_option_chain();
            let params = RNDParameters {
                risk_free_rate: dec!(0.05),
                interpolation_points: 100,
                derivative_tolerance: pos!(0.001),
            };

            let result = chain.calculate_rnd(&params);
            assert!(result.is_err());

            assert!(result.unwrap_err().to_string().contains("Failed to calculate valid densities"));
        }

        #[test]
        fn test_calculate_rnd_empty_chain() {
            let chain = create_empty_chain();
            let params = RNDParameters::default();

            let result = chain.calculate_rnd(&params);
            assert!(result.is_err());
            assert!(
                result.unwrap_err().to_string().contains("Derivative tolerance must be greater than zero")
            );
        }

        #[test]
        fn test_calculate_rnd_zero_tolerance() {
            let chain = create_test_option_chain();
            let params = RNDParameters {
                derivative_tolerance: Positive::ZERO,
                ..Default::default()
            };

            let result = chain.calculate_rnd(&params);
            assert!(result.is_err());
            assert!(
                result.unwrap_err().to_string().contains("Derivative tolerance must be greater than zero")
            );
        }

        #[test]
        fn test_calculate_rnd_high_risk_free_rate() {
            let chain = create_test_option_chain();
            let params = RNDParameters {
                risk_free_rate: dec!(0.5), // 50% interest rate
                derivative_tolerance: pos!(0.001),
                ..Default::default()
            };

            let result = chain.calculate_rnd(&params);
            assert!(result.is_err());
            // Additional assertions about high interest rate effects could be added
            assert!(result.unwrap_err().to_string().contains("Failed to calculate valid densities"));
        }
    }

    mod skew_calculation_tests {
        use super::*;

        #[test]
        fn test_calculate_skew_normal_case() {
            let chain = create_test_option_chain();
            let result = chain.calculate_skew();

            assert!(result.is_ok());
            let skew = result.unwrap();
            assert!(!skew.is_empty());

            // Test for monotonicity
            for window in skew.windows(2) {
                assert!(window[0].0 < window[1].0); // Strikes should be increasing
            }
        }

        #[test]
        fn test_calculate_skew_empty_chain() {
            let chain = create_empty_chain();
            let result = chain.calculate_skew();

            assert!(result.is_err());
            assert!(
                result.unwrap_err().to_string().contains("Cannot find ATM OptionData for empty option chain: TEST")
            );
        }

        #[test]
        fn test_calculate_skew_missing_implied_volatility() {
            let mut chain = create_test_option_chain();
            // Add an option without implied volatility
            chain.add_option(
                pos!(115.0),
                spos!(5.0),
                spos!(5.5),
                spos!(15.0),
                spos!(15.5),
                pos!(0.2), // No implied volatility
                Some(dec!(0.3)),
                Some(dec!(0.3)),
                Some(dec!(0.3)),
                spos!(100.0),
                Some(50),
                None,
            );

            let result = chain.calculate_skew();
            assert!(result.is_ok()); // Should still work with partial data
        }
    }

    mod helper_method_tests {
        use super::*;

        #[test]
        fn test_get_call_price() {
            let chain = create_test_option_chain();

            // Test existing strike
            let price = chain.get_call_price(pos!(100.0));
            assert!(price.is_some());

            // Test non-existing strike
            let price = chain.get_call_price(pos!(99.0));
            assert!(price.is_none());
        }

        #[test]
        fn test_get_atm_implied_volatility() {
            let chain = create_test_option_chain();

            // Test normal case
            let vol = chain.get_atm_implied_volatility();
            assert!(vol.is_ok());

            // Test empty chain
            let empty_chain = create_empty_chain();
            let vol = empty_chain.get_atm_implied_volatility();
            assert!(vol.is_err());
        }
    }

    mod integration_tests {
        use super::*;

        #[test]
        fn test_full_rnd_workflow() {
            let chain = create_test_option_chain();
            let params = RNDParameters {
                risk_free_rate: dec!(0.05),
                interpolation_points: 100,
                derivative_tolerance: pos!(0.001),
            };

            // Calculate RND
            let rnd_result = chain.calculate_rnd(&params);

            assert!(rnd_result.is_err());

            assert!(rnd_result.unwrap_err().to_string().contains("Failed to calculate valid densities"));
        }

        #[test]
        fn test_extreme_market_conditions() {
            let mut chain =
                OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

            // Add options with extreme values
            chain.add_option(
                pos!(50.0), // Deep ITM
                spos!(50.0),
                spos!(51.0),
                spos!(0.1),
                spos!(0.2),
                pos!(0.8), // High volatility
                Some(dec!(-0.99)),
                Some(dec!(0.3)),
                Some(dec!(0.3)),
                spos!(10.0),
                Some(5),
                None,
            );

            chain.add_option(
                pos!(150.0), // Deep OTM
                spos!(0.1),
                spos!(0.2),
                spos!(50.0),
                spos!(51.0),
                pos!(0.8), // High volatility
                Some(dec!(0.99)),
                Some(dec!(0.3)),
                Some(dec!(0.3)),
                spos!(10.0),
                Some(5),
                None,
            );

            let params = RNDParameters {
                risk_free_rate: dec!(0.10), // High interest rate
                interpolation_points: 200,
                derivative_tolerance: pos!(0.001),
            };

            let rnd_result = chain.calculate_rnd(&params);
            assert!(rnd_result.is_err());
            assert!(rnd_result.unwrap_err().to_string().contains("Failed to calculate valid densities"));
        }
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    mod rnd_statistics_extended_tests {
        use super::*;
        use crate::{assert_decimal_eq, pos};

        #[test]
        fn test_asymmetric_distribution() {
            let mut densities = BTreeMap::new();
            densities.insert(pos!(90.0), dec!(0.1));
            densities.insert(pos!(100.0), dec!(0.7));
            densities.insert(pos!(110.0), dec!(0.2));

            let stats = RNDStatistics::new(&densities);
            assert_decimal_eq!(stats.skewness.abs(), dec!(0.076839), dec!(0.00001));
        }

        #[test]
        fn test_extreme_values_distribution() {
            let mut densities = BTreeMap::new();
            densities.insert(pos!(50.0), dec!(0.01));
            densities.insert(pos!(100.0), dec!(0.97));
            densities.insert(pos!(150.0), dec!(0.02));

            let stats = RNDStatistics::new(&densities);
            assert!(stats.variance > Positive::ZERO);
            assert!(stats.kurtosis.abs() > dec!(5.0));
        }

        #[test]
        fn test_uniform_distribution() {
            let mut densities = BTreeMap::new();

            densities.insert(pos!(90.0), dec!(0.2));
            densities.insert(pos!(95.0), dec!(0.2));
            densities.insert(pos!(100.0), dec!(0.2));
            densities.insert(pos!(105.0), dec!(0.2));
            densities.insert(pos!(110.0), dec!(0.2));

            let stats = RNDStatistics::new(&densities);
            assert_decimal_eq!(stats.skewness.abs(), dec!(0.0), dec!(0.00001));
            assert_decimal_eq!(stats.kurtosis, dec!(-1.2999999), dec!(0.00001));
        }

        #[test]
        fn test_bimodal_distribution() {
            let mut densities = BTreeMap::new();

            densities.insert(pos!(80.0), dec!(0.3));
            densities.insert(pos!(90.0), dec!(0.1));
            densities.insert(pos!(100.0), dec!(0.1));
            densities.insert(pos!(110.0), dec!(0.1));
            densities.insert(pos!(120.0), dec!(0.4));

            let stats = RNDStatistics::new(&densities);
            assert_decimal_eq!(stats.kurtosis, dec!(-1.69028), dec!(0.00001));
        }
    }

    mod rnd_calculation_extended_tests {
        use super::*;
        use crate::chains::chain::OptionChain;
        use crate::{pos, spos};

        fn create_test_option_chain() -> OptionChain {
            let mut chain =
                OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

            // Add a range of options around the money
            for strike in [80.0, 90.0, 95.0, 100.0, 105.0, 110.0, 120.0].iter() {
                chain.add_option(
                    pos!(*strike),
                    spos!(15.0),
                    spos!(15.5),
                    spos!(5.0),
                    spos!(5.5),
                    pos!(0.2),
                    Some(dec!(-0.3)),
                    Some(dec!(0.3)),
                    Some(dec!(0.3)),
                    spos!(100.0),
                    Some(50),
                    None,
                );
            }
            chain
        }

        fn create_wide_spread_chain() -> OptionChain {
            let mut chain =
                OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

            // Amplio rango de strikes
            for strike in [60.0, 80.0, 100.0, 120.0, 140.0].iter() {
                chain.add_option(
                    pos!(*strike),
                    spos!(15.0),
                    spos!(15.5),
                    spos!(5.0),
                    spos!(5.5),
                    pos!(0.2),
                    Some(dec!(-0.3)),
                    Some(dec!(0.3)),
                    Some(dec!(0.3)),
                    spos!(100.0),
                    Some(50),
                    None,
                );
            }
            chain
        }

        fn create_high_vol_chain() -> OptionChain {
            let mut chain =
                OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

            // Alta volatilidad
            for strike in [90.0, 95.0, 100.0, 105.0, 110.0].iter() {
                chain.add_option(
                    pos!(*strike),
                    spos!(15.0),
                    spos!(15.5),
                    spos!(5.0),
                    spos!(5.5),
                    pos!(0.5), // Alta volatilidad
                    Some(dec!(-0.3)),
                    Some(dec!(0.3)),
                    Some(dec!(0.3)),
                    spos!(100.0),
                    Some(50),
                    None,
                );
            }
            chain
        }

        #[test]
        fn test_calculate_rnd_wide_spread() {
            let chain = create_wide_spread_chain();
            let params = RNDParameters {
                risk_free_rate: dec!(0.05),
                interpolation_points: 100,
                derivative_tolerance: pos!(0.001),
            };

            let result = chain.calculate_rnd(&params);
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("Failed to calculate valid densities"));
        }

        #[test]
        fn test_calculate_rnd_high_volatility() {
            let chain = create_high_vol_chain();
            let params = RNDParameters {
                risk_free_rate: dec!(0.05),
                interpolation_points: 100,
                derivative_tolerance: pos!(0.001),
            };

            let result = chain.calculate_rnd(&params);
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("Failed to calculate valid densities"));
        }

        #[test]
        fn test_calculate_rnd_different_tolerances() {
            let chain = create_test_option_chain();

            let tolerances = [pos!(0.0001), pos!(0.001), pos!(0.01), pos!(0.1)];

            for tolerance in tolerances.iter() {
                let params = RNDParameters {
                    risk_free_rate: dec!(0.05),
                    interpolation_points: 100,
                    derivative_tolerance: *tolerance,
                };

                let result = chain.calculate_rnd(&params);
                assert!(result.is_err());
                assert!(result.unwrap_err().to_string().contains("Failed to calculate valid densities"));
            }
        }
    }

    mod numerical_stability_tests {
        use super::*;
        use crate::chains::chain::OptionChain;
        use crate::{pos, spos};

        #[test]
        fn test_numerical_stability_small_values() {
            let mut chain =
                OptionChain::new("TEST", pos!(1.0), "2024-12-31".to_string(), None, None);

            chain.add_option(
                pos!(0.9),
                spos!(0.001),
                spos!(0.002),
                spos!(0.001),
                spos!(0.002),
                pos!(0.1),
                Some(dec!(-0.3)),
                Some(dec!(0.3)),
                Some(dec!(0.3)),
                spos!(100.0),
                Some(50),
                None,
            );

            let params = RNDParameters {
                risk_free_rate: dec!(0.05),
                interpolation_points: 100,
                derivative_tolerance: pos!(0.0001),
            };

            let result = chain.calculate_rnd(&params);
            assert!(result.is_err());
        }

        #[test]
        fn test_numerical_stability_large_values() {
            let mut chain =
                OptionChain::new("TEST", pos!(10000.0), "2024-12-31".to_string(), None, None);

            chain.add_option(
                pos!(9900.0),
                spos!(1000.0),
                spos!(1001.0),
                spos!(1000.0),
                spos!(1001.0),
                pos!(0.1),
                Some(dec!(-0.3)),
                Some(dec!(0.3)),
                Some(dec!(0.3)),
                spos!(100.0),
                Some(50),
                None,
            );

            let params = RNDParameters {
                risk_free_rate: dec!(0.05),
                interpolation_points: 100,
                derivative_tolerance: pos!(0.0001),
            };

            let result = chain.calculate_rnd(&params);
            assert!(result.is_err());
        }
    }
}

#[cfg(test)]
mod statistical_validation_tests {
    use super::*;
    use crate::{assert_decimal_eq, pos};
    use rust_decimal_macros::dec;

    mod moments_tests {
        use super::*;
        use num_traits::{FromPrimitive, ToPrimitive};
        use tracing::info;

        #[test]
        fn test_simple_mean() {
            let mut densities = BTreeMap::new();
            densities.insert(pos!(100.0), dec!(0.5));
            densities.insert(pos!(200.0), dec!(0.5));

            let stats = RNDStatistics::new(&densities);
            assert_decimal_eq!(stats.mean, dec!(150.0), dec!(0.00001));
        }

        #[test]
        fn test_normal_distribution_step_by_step() {
            let mut densities = BTreeMap::new();
            densities.insert(pos!(80.0), dec!(0.1));
            densities.insert(pos!(90.0), dec!(0.2));
            densities.insert(pos!(100.0), dec!(0.4));
            densities.insert(pos!(110.0), dec!(0.2));
            densities.insert(pos!(120.0), dec!(0.1));

            // Step 1: Calculate mean
            let mut mean = Decimal::ZERO;
            let mut total = Decimal::ZERO;
            for (x, p) in densities.iter() {
                mean += x.to_dec() * *p;
                total += *p;
            }
            mean /= total;
            info!("Step-by-step mean: {}", mean);

            // Step 2: Calculate variance
            let mut variance = Decimal::ZERO;
            for (x, p) in densities.iter() {
                let diff = x.to_dec() - mean;
                variance += diff * diff * (*p);
            }
            variance /= total;
            info!("Step-by-step variance: {}", variance);

            // Step 3: Calculate kurtosis
            let std_dev = Decimal::from_f64(variance.to_f64().unwrap().sqrt()).unwrap();
            let std_dev_4 = std_dev.powi(4);
            let mut kurtosis = Decimal::ZERO;
            for (x, p) in densities.iter() {
                let diff = x.to_dec() - mean;
                kurtosis += (diff.powi(4) * (*p)) / std_dev_4;
            }
            kurtosis = (kurtosis / total) - dec!(3.0);
            info!("Step-by-step kurtosis: {}", kurtosis);

            // Verify with structure
            let stats = RNDStatistics::new(&densities);
            info!("Structure values:");
            info!("Mean: {}", stats.mean);
            info!("Variance: {}", stats.variance);
            info!("Kurtosis: {}", stats.kurtosis);

            assert_decimal_eq!(stats.kurtosis, kurtosis, dec!(0.00001));
        }

        #[test]
        fn test_kurtosis_calculation_comparison() {
            let mut densities = BTreeMap::new();
            densities.insert(pos!(80.0), dec!(0.1));
            densities.insert(pos!(90.0), dec!(0.2));
            densities.insert(pos!(100.0), dec!(0.4));
            densities.insert(pos!(110.0), dec!(0.2));
            densities.insert(pos!(120.0), dec!(0.1));

            // Manual calculation
            info!("Manual Calculation:");
            let mut mean = Decimal::ZERO;
            let mut total = Decimal::ZERO;
            for (x, p) in densities.iter() {
                mean += x.to_dec() * *p;
                total += *p;
            }
            mean /= total;
            info!("Mean: {}", mean);

            let mut variance_dec = Decimal::ZERO;
            for (x, p) in densities.iter() {
                let diff = x.to_dec() - mean;
                variance_dec += diff * diff * (*p);
            }
            variance_dec /= total;
            info!("Variance as Decimal: {}", variance_dec);

            let std_dev_manual = Decimal::from_f64(variance_dec.to_f64().unwrap().sqrt()).unwrap();
            info!("Std Dev (manual): {}", std_dev_manual);
            let std_dev_4_manual = std_dev_manual.powi(4);
            info!("Std Dev^4 (manual): {}", std_dev_4_manual);

            let mut fourth_moment = Decimal::ZERO;
            for (x, p) in densities.iter() {
                let diff = x.to_dec() - mean;
                let term = diff.powi(4);
                fourth_moment += term * (*p);
            }
            fourth_moment /= total;
            info!("Fourth Moment: {}", fourth_moment);

            let kurtosis_manual = (fourth_moment / std_dev_4_manual) - dec!(3.0);
            info!("Kurtosis (manual): {}", kurtosis_manual);

            // Structure calculation
            info!("\nStructure Calculation:");
            let stats = RNDStatistics::new(&densities);
            info!("Mean: {}", stats.mean);
            info!("Variance: {}", stats.variance);
            info!("Kurtosis: {}", stats.kurtosis);

            // Compare values
            assert_decimal_eq!(stats.mean, mean, dec!(0.00001));
            assert_decimal_eq!(stats.variance.to_dec(), variance_dec, dec!(0.00001));
            assert_decimal_eq!(stats.kurtosis, kurtosis_manual, dec!(0.00001));
        }

        #[test]
        fn test_normal_distribution_detailed() {
            let mut densities = BTreeMap::new();
            densities.insert(pos!(80.0), dec!(0.1));
            densities.insert(pos!(90.0), dec!(0.2));
            densities.insert(pos!(100.0), dec!(0.4));
            densities.insert(pos!(110.0), dec!(0.2));
            densities.insert(pos!(120.0), dec!(0.1));

            // Step 1: Calculate mean
            let mut mean = Decimal::ZERO;
            let mut total = Decimal::ZERO;
            for (x, p) in densities.iter() {
                mean += x.to_dec() * *p;
                total += *p;
            }
            mean /= total;
            info!("Step 1 - Mean: {}", mean);

            // Step 2: Calculate variance
            let mut variance = Decimal::ZERO;
            for (x, p) in densities.iter() {
                let diff = x.to_dec() - mean;
                variance += diff * diff * (*p);
            }
            variance /= total;
            info!("Step 2 - Variance: {}", variance);

            // Step 3: Calculate fourth moment
            let std_dev = Decimal::from_f64(variance.to_f64().unwrap().sqrt()).unwrap();
            let std_dev_4 = std_dev * std_dev * std_dev * std_dev;
            info!("Step 3a - Std Dev: {}", std_dev);
            info!("Step 3b - Std Dev^4: {}", std_dev_4);

            let mut fourth_moment = Decimal::ZERO;
            for (x, p) in densities.iter() {
                let diff = x.to_dec() - mean;
                let term = diff * diff * diff * diff;
                info!("x: {}, diff^4: {}", x, term);
                fourth_moment += term * (*p);
            }
            fourth_moment /= total;
            info!("Step 3c - Fourth Moment: {}", fourth_moment);

            // Step 4: Calculate kurtosis
            let kurtosis = (fourth_moment / std_dev_4) - dec!(3.0);
            info!("Step 4 - Final Kurtosis: {}", kurtosis);

            // Verify with structure
            let stats = RNDStatistics::new(&densities);
            info!("\nStructure values:");
            info!("Mean: {}", stats.mean);
            info!("Variance: {}", stats.variance);
            info!("Kurtosis: {}", stats.kurtosis);

            assert_decimal_eq!(mean, stats.mean, dec!(0.00001));
            assert_decimal_eq!(variance, stats.variance.to_dec(), dec!(0.00001));
            assert_decimal_eq!(kurtosis, stats.kurtosis, dec!(0.00001));
        }

        #[test]
        fn test_simple_variance() {
            let mut densities = BTreeMap::new();
            densities.insert(pos!(90.0), dec!(0.5));
            densities.insert(pos!(110.0), dec!(0.5));

            let stats = RNDStatistics::new(&densities);

            assert_decimal_eq!(stats.mean, dec!(100.0), dec!(0.00001));

            // Variance = 100 ((90-100)²*0.5 + (110-100)²*0.5)
            assert_decimal_eq!(stats.variance.to_dec(), dec!(100.0), dec!(0.00001));
        }

        #[test]
        fn test_discrete_uniform() {
            let mut densities = BTreeMap::new();
            for i in 1..=5 {
                densities.insert(pos!(i as f64), dec!(0.2));
            }

            let stats = RNDStatistics::new(&densities);

            assert_decimal_eq!(stats.mean, dec!(3.0), dec!(0.00001));
            assert_decimal_eq!(stats.variance.to_dec(), dec!(2.0), dec!(0.00001));
            assert_decimal_eq!(stats.skewness, dec!(0.0), dec!(0.00001));
        }

        #[test]
        fn test_normalization() {
            let mut densities = BTreeMap::new();
            densities.insert(pos!(100.0), dec!(2.0));
            densities.insert(pos!(200.0), dec!(3.0));
            let stats = RNDStatistics::new(&densities);
            assert_decimal_eq!(stats.mean, dec!(160.0), dec!(0.00001));
        }

        #[test]
        fn test_small_values() {
            let mut densities = BTreeMap::new();
            densities.insert(pos!(1.0), dec!(0.001));
            densities.insert(pos!(2.0), dec!(0.002));
            densities.insert(pos!(3.0), dec!(0.001));

            let stats = RNDStatistics::new(&densities);

            assert_decimal_eq!(stats.mean, dec!(2.0), dec!(0.00001));
            assert!(stats.variance > Positive::ZERO);
        }

        #[test]
        fn test_extreme_values() {
            let mut densities = BTreeMap::new();
            densities.insert(pos!(1000000.0), dec!(0.3));
            densities.insert(pos!(2000000.0), dec!(0.4));
            densities.insert(pos!(3000000.0), dec!(0.3));

            let stats = RNDStatistics::new(&densities);

            assert_decimal_eq!(stats.mean, dec!(2000000.0), dec!(0.00001));
            assert!(stats.variance > Positive::ZERO);
        }

        #[test]
        fn test_gap_distribution() {
            let mut densities = BTreeMap::new();
            densities.insert(pos!(10.0), dec!(0.45));
            densities.insert(pos!(90.0), dec!(0.55));

            let stats = RNDStatistics::new(&densities);

            assert_decimal_eq!(stats.mean, dec!(54.0), dec!(0.00001));

            assert!(stats.kurtosis < dec!(0.0));
        }

        #[test]
        fn test_gap_distribution_detailed() {
            let mut densities = BTreeMap::new();
            densities.insert(pos!(10.0), dec!(0.45));
            densities.insert(pos!(90.0), dec!(0.55));

            // Step 1: Calculate mean manually
            let mut mean = Decimal::ZERO;
            let mut total = Decimal::ZERO;
            for (x, p) in densities.iter() {
                mean += x.to_dec() * *p;
                total += *p;
            }
            mean /= total;
            info!("Step 1 - Mean: {}", mean);
            // Should be: (10 * 0.45 + 90 * 0.55) = 54.0

            // Step 2: Calculate variance manually
            let mut variance = Decimal::ZERO;
            for (x, p) in densities.iter() {
                let diff = x.to_dec() - mean;
                variance += diff * diff * (*p);
            }
            variance /= total;
            info!("Step 2 - Variance: {}", variance);

            // Step 3: Calculate fourth moment
            let std_dev = Decimal::from_f64(variance.to_f64().unwrap().sqrt()).unwrap();
            let std_dev_4 = std_dev.powi(4);
            info!("Step 3a - Std Dev: {}", std_dev);
            info!("Step 3b - Std Dev^4: {}", std_dev_4);

            let mut fourth_moment = Decimal::ZERO;
            for (x, p) in densities.iter() {
                let diff = x.to_dec() - mean;
                let term = diff.powi(4);
                info!("x: {}, diff: {}, diff^4: {}, p: {}", x, diff, term, p);
                fourth_moment += term * (*p);
            }
            fourth_moment /= total;
            info!("Step 3c - Fourth Moment: {}", fourth_moment);

            // Step 4: Calculate kurtosis
            let kurtosis = (fourth_moment / std_dev_4) - dec!(3.0);
            info!("Step 4 - Final Kurtosis: {}", kurtosis);

            // Compare with structure calculation
            let stats = RNDStatistics::new(&densities);
            info!("\nStructure values:");
            info!("Mean: {}", stats.mean);
            info!("Variance: {}", stats.variance);
            info!("Kurtosis: {}", stats.kurtosis);

            assert_decimal_eq!(mean, stats.mean, dec!(0.00001));
            assert_decimal_eq!(variance, stats.variance.to_dec(), dec!(0.00001));
            assert_decimal_eq!(kurtosis, stats.kurtosis, dec!(0.00001));
            assert_decimal_eq!(kurtosis, dec!(-1.9595959595), dec!(0.00001));
        }

        #[test]
        fn test_moment_properties() {
            let mut densities = BTreeMap::new();
            densities.insert(pos!(95.0), dec!(0.3));
            densities.insert(pos!(100.0), dec!(0.4));
            densities.insert(pos!(105.0), dec!(0.3));

            let stats = RNDStatistics::new(&densities);

            assert!(stats.variance > Positive::ZERO);
            assert_decimal_eq!(stats.skewness.abs(), dec!(0.0), dec!(0.00001));
            assert!(stats.kurtosis > dec!(-3.0));
        }
    }

    mod validation_utils {
        use super::*;

        fn calculate_raw_moment(densities: &BTreeMap<Positive, Decimal>, order: i32) -> Decimal {
            let mut moment = Decimal::ZERO;
            let mut total_density = Decimal::ZERO;

            for (strike, density) in densities {
                moment += strike.to_dec().powi(order as i64) * density;
                total_density += density;
            }

            if !total_density.is_zero() {
                moment / total_density
            } else {
                Decimal::ZERO
            }
        }

        #[test]
        fn test_raw_moments() {
            let mut densities = BTreeMap::new();
            densities.insert(pos!(90.0), dec!(0.2));
            densities.insert(pos!(100.0), dec!(0.6));
            densities.insert(pos!(110.0), dec!(0.2));

            // Primer momento (media)
            let mean = calculate_raw_moment(&densities, 1);
            assert_decimal_eq!(mean, dec!(100.0), dec!(0.00001));

            // Segundo momento
            let second_moment = calculate_raw_moment(&densities, 2);
            assert!(second_moment > mean.powi(2)); // Varianza positiva
        }
    }
}

#[cfg(test)]
mod chain_test {
    use crate::chains::chain::OptionChain;
    use crate::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
    use crate::chains::{RNDAnalysis, RNDParameters};
    use crate::{ExpirationDate, assert_decimal_eq, pos, spos};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use tracing::debug;

    fn create_test_option_chain() -> OptionChain {
        let option_chain_params = OptionChainBuildParams::new(
            "SP500".to_string(),
            None,
            10,
            spos!(1.0),
            dec!(-0.2),
            dec!(0.1),
            pos!(0.02),
            2,
            OptionDataPriceParams::new(
                Some(Box::new(pos!(100.0))),
                Some(ExpirationDate::Days(pos!(30.0))),
                Some(Decimal::ZERO),
                spos!(0.05),
                Some("SP500".to_string()),
            ),
            pos!(0.2),
        );

        OptionChain::build_chain(&option_chain_params)
    }
    #[test]
    fn test_chain_creation() {
        let option_chain_params = OptionChainBuildParams::new(
            "SP500".to_string(),
            None,
            10,
            spos!(1.0),
            dec!(-0.2),
            dec!(0.1),
            pos!(0.02),
            2,
            OptionDataPriceParams::new(
                Some(Box::new(pos!(100.0))),
                Some(ExpirationDate::Days(pos!(30.0))),
                Some(Decimal::ZERO),
                spos!(0.0),
                Some("SP500".to_string()),
            ),
            pos!(0.2),
        );

        let chain = OptionChain::build_chain(&option_chain_params);

        let params = RNDParameters {
            risk_free_rate: dec!(0.05),
            interpolation_points: 100,
            derivative_tolerance: pos!(0.01),
        };
        // Calculate RND from option chain
        let rnd_result = chain.calculate_rnd(&params).unwrap();
        assert!(!rnd_result.densities.is_empty());

        // Updated expected values to match correct chain_size=10 behavior
        // (previously the chain was incorrectly generating more strikes than requested)
        assert_decimal_eq!(rnd_result.statistics.mean, dec!(99.96667), dec!(0.001));
        assert_decimal_eq!(rnd_result.statistics.skewness, dec!(0.04974), dec!(0.001));
        assert_decimal_eq!(rnd_result.statistics.kurtosis, dec!(-0.8346), dec!(0.001));
        assert_decimal_eq!(
            rnd_result.statistics.variance.to_dec(),
            dec!(20.4989),
            dec!(0.001)
        );
    }

    #[test]
    fn test_rnd_calculation_debug() {
        let option_chain_params = OptionChainBuildParams::new(
            "SP500".to_string(),
            None,
            10,
            spos!(1.0),
            dec!(-0.2),
            dec!(0.1),
            pos!(0.02),
            2,
            OptionDataPriceParams::new(
                Some(Box::new(pos!(100.0))),
                Some(ExpirationDate::Days(pos!(30.0))),
                Some(Decimal::ZERO),
                spos!(0.05),
                Some("SP500".to_string()),
            ),
            pos!(0.2),
        );

        let chain = OptionChain::build_chain(&option_chain_params);
        let params = RNDParameters {
            risk_free_rate: dec!(0.05),
            interpolation_points: 100,
            derivative_tolerance: pos!(1.0), // Using larger step size for testing
        };

        debug!("Initial option chain:");
        for opt in &chain.options {
            debug!("Strike: {}, Call Ask: {:?}", opt.strike_price, opt.call_ask);
        }

        let result = chain.calculate_rnd(&params);
        match result {
            Ok(rnd) => {
                debug!("\nCalculated densities:");
                for (k, d) in rnd.densities {
                    debug!("Strike: {}, Density: {}", k, d);
                }
            }
            Err(e) => debug!("Error: {}", e),
        }
    }

    #[test]
    fn test_rnd_calculation_tolerance_comparison() {
        let chain = create_test_option_chain();

        // Test with h = 1.0
        let params_1 = RNDParameters {
            risk_free_rate: dec!(0.05),
            interpolation_points: 100,
            derivative_tolerance: pos!(1.0),
        };

        // Test with h = 0.1
        let params_2 = RNDParameters {
            risk_free_rate: dec!(0.05),
            interpolation_points: 100,
            derivative_tolerance: pos!(0.1),
        };

        debug!("Testing with h = 1.0:");
        for opt in &chain.options {
            let k = opt.strike_price;
            debug!(
                "Strike {}: Found neighbors: k-h={}, k+h={}",
                k,
                chain.get_call_price(k - pos!(1.0)).is_some(),
                chain.get_call_price(k + pos!(1.0)).is_some()
            );
            assert!(
                chain.get_call_price(k - pos!(1.0)).is_some()
                    || chain.get_call_price(k + pos!(1.0)).is_some()
            );
        }
        assert!(chain.calculate_rnd(&params_1).is_ok());

        debug!("\nTesting with h = 0.1:");
        for opt in &chain.options {
            let k = opt.strike_price;
            debug!(
                "Strike {}: Found neighbors: k-h={}, k+h={}",
                k,
                chain.get_call_price(k - pos!(0.1)).is_some(),
                chain.get_call_price(k + pos!(0.1)).is_some()
            );
            assert!(
                chain.get_call_price(k - pos!(1.0)).is_some()
                    || chain.get_call_price(k + pos!(1.0)).is_some()
            );
        }
        assert!(chain.calculate_rnd(&params_2).is_ok());
    }
}

#[cfg(test)]
mod rnd_coverage_tests {
    use super::*;
    use crate::chains::OptionChain;
    use crate::chains::RNDAnalysis;
    use crate::chains::RNDResult;
    use crate::{pos, spos};
    use std::collections::BTreeMap;

    // Test for line 322 in rnd.rs
    #[test]
    fn test_rnd_result_new() {
        // Create a simple densities map
        let mut densities = BTreeMap::new();
        densities.insert(pos!(90.0), dec!(0.2));
        densities.insert(pos!(100.0), dec!(0.6));
        densities.insert(pos!(110.0), dec!(0.2));

        // Create a new RNDResult
        let result = RNDResult::new(densities);

        // Check that statistics were calculated
        assert_eq!(result.statistics.mean, dec!(100.0));
        assert!(result.statistics.variance > Positive::ZERO);
        assert!(result.statistics.volatility > Positive::ZERO);
    }

    // Test for line 369 in rnd.rs
    #[test]
    fn test_calculate_skew_with_custom_chain() {
        // Create a custom chain with specific volatility pattern
        let mut chain = OptionChain::new(
            "TEST",
            pos!(100.0),
            "2024-06-30".to_string(),
            Some(dec!(0.05)),
            spos!(0.0),
        );

        // Add options with volatility smile pattern
        let strikes = [80.0, 90.0, 100.0, 110.0, 120.0];
        let vols = [0.25, 0.20, 0.17, 0.20, 0.25]; // Smile pattern

        for (i, strike) in strikes.iter().enumerate() {
            chain.add_option(
                pos!(*strike),
                spos!(10.0),
                spos!(10.5),
                spos!(10.0),
                spos!(10.5),
                pos!(vols[i]),
                None,
                None,
                None,
                spos!(1000.0),
                None,
                None,
            );
        }

        // Calculate skew
        let result = chain.calculate_skew();
        assert!(result.is_ok());

        let skew = result.unwrap();

        // Confirm we got the right number of data points
        assert_eq!(skew.len(), 5);

        // With a symmetric smile, the skew around ATM should be symmetric
        let atm_index = skew.iter().position(|(k, _)| *k == pos!(1.0)).unwrap();
        let lower = skew[atm_index - 1].1;
        let higher = skew[atm_index + 1].1;

        // The absolute skew values should be similar in a smile
        assert!((lower.abs() - higher.abs()).abs() < dec!(0.05));
    }
}
