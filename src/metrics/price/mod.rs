/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 18/12/25
******************************************************************************/

/// # Price Metrics Module
///
/// Provides comprehensive price performance metrics tools for financial applications
///
/// ## Core Components
/// - `VolatilitySkew` trait: Enables plotting functionality for volatility skew curves.
///
/// ## Usage Example Volatility Skew
/// ```rust
/// use std::collections::BTreeSet;
/// use rust_decimal::Decimal;
/// use optionstratlib::curves::Curve;
/// use optionstratlib::error::greeks::CalculationErrorKind::DecimalError;
/// use optionstratlib::metrics::VolatilitySkew;
///
/// struct MySkew;
///
/// impl VolatilitySkew for MySkew {
///     fn volatility_skew(&self) -> Curve {
///         // Custom logic to build and return a Curve representing the skew
///         Curve { points: BTreeSet::new(), x_range: (Decimal::ZERO, Decimal::ZERO) }
///     }
/// }
/// ```
///
pub mod volatility_skew;
