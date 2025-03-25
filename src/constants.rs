/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 11/8/24
******************************************************************************/
use crate::Positive;
use plotters::style::RGBColor;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Mathematical constant representing π (pi) with high precision using Decimal type.
/// Used for circular calculations, angle conversions, and geometric computations.
pub const PI: Decimal = dec!(3.1415926535897932384626433832);

/// Represents zero as a 64-bit floating point number.
/// Used as a baseline value for numerical comparisons and calculations.
pub const ZERO: f64 = 0.0;

/// Number of seconds in a 24-hour day (86400 = 24 * 60 * 60).
/// Used for time-based calculations and conversions.
#[allow(dead_code)]
pub(crate) const SECONDS_IN_A_DAY: i64 = 86400;

/// Standard number of days in a year as a Positive decimal value.
/// Used for annualization calculations and time-based financial models.
pub(crate) const DAYS_IN_A_YEAR: Positive = Positive(dec!(365.0));

/// Small decimal value used as a threshold for convergence tests and equality comparisons.
/// Represents a general tolerance level for numerical algorithms.
pub(crate) const TOLERANCE: Decimal = dec!(1e-8);

/// Extremely small decimal value used for high-precision calculations.
/// Represents the smallest meaningful difference in numerical computations.
pub const EPSILON: Decimal = dec!(1e-16);

/// Minimum allowed volatility value as a Positive decimal.
/// Prevents numerical issues in financial calculations with near-zero volatility.
pub(crate) const MIN_VOLATILITY: Positive = Positive(dec!(1e-16));

/// Maximum allowed volatility value as a Positive decimal (100%).
/// Sets an upper bound for volatility inputs in financial models.
pub(crate) const MAX_VOLATILITY: Positive = Positive::HUNDRED;

/// RGB color definition for dark green visualization elements.
/// Used for positive indicators or upward movements in charts.
pub(crate) const DARK_GREEN: RGBColor = RGBColor(0, 150, 0);

/// RGB color definition for dark red visualization elements.
/// Used for negative indicators or downward movements in charts.
pub(crate) const DARK_RED: RGBColor = RGBColor(220, 0, 0);

/// RGB color definition for dark blue visualization elements.
/// Used for neutral or reference indicators in charts.
pub(crate) const DARK_BLUE: RGBColor = RGBColor(0, 0, 150);

/// Multiplier defining the lower bound for strike price ranges (98% of reference price).
/// Used to establish the minimum strike price in option chains or pricing models.
pub(crate) const STRIKE_PRICE_LOWER_BOUND_MULTIPLIER: f64 = 0.98;

/// Multiplier defining the upper bound for strike price ranges (102% of reference price).
/// Used to establish the maximum strike price in option chains or pricing models.
pub(crate) const STRIKE_PRICE_UPPER_BOUND_MULTIPLIER: f64 = 1.02;

/// Standard number of trading days in a year as a Positive decimal.
/// Used for business day-based financial calculations.
pub(crate) const TRADING_DAYS: Positive = Positive(dec!(252.0));

/// Standard number of trading hours in a market day as a Positive decimal.
/// Typically represents a standard U.S. market session (9:30 AM to 4:00 PM).
pub(crate) const TRADING_HOURS: Positive = Positive(dec!(6.5));

/// Number of seconds in an hour as a Positive decimal value.
/// Used for time-based conversions and calculations.
pub(crate) const SECONDS_PER_HOUR: Positive = Positive(dec!(3600.0));

/// Number of minutes in an hour as a Positive decimal value.
/// Used for time-based conversions and calculations.
pub(crate) const MINUTES_PER_HOUR: Positive = Positive(dec!(60.0));

/// Number of milliseconds in a second as a Positive decimal value.
/// Used for precise time measurements and conversions.
pub(crate) const MILLISECONDS_PER_SECOND: Positive = Positive(dec!(1000.0));

/// Number of microseconds in a second as a Positive decimal value.
/// Used for high-precision time measurements and conversions.
pub(crate) const MICROSECONDS_PER_SECOND: Positive = Positive(dec!(1_000_000.0));

/// Standard number of weeks in a year as a Positive decimal value.
/// Used for time-based financial calculations and annualization.
pub(crate) const WEEKS_PER_YEAR: Positive = Positive(dec!(52.0));

/// Number of months in a year as a Positive decimal value.
/// Used for monthly-based financial calculations and conversions.
pub(crate) const MONTHS_PER_YEAR: Positive = Positive(dec!(12.0));

/// Number of quarters in a year as a Positive decimal value.
/// Used for quarterly financial calculations and reporting periods.
pub(crate) const QUARTERS_PER_YEAR: Positive = Positive(dec!(4.0));

/// Maximum number of iterations for implied volatility calculation algorithms.
/// Prevents infinite loops in numerical methods like Newton-Raphson or bisection.
pub(crate) const MAX_ITERATIONS_IV: u32 = 1000;

/// Convergence tolerance for implied volatility calculations.
/// Determines when the implied volatility solver has reached sufficient precision.
pub(crate) const IV_TOLERANCE: Decimal = dec!(1e-5);

/// Standard deviation multiplier used for scaling graph boundaries.
///
/// This constant defines how many standard deviations from the mean should be
/// displayed on statistical graphs or charts. A value of 4.0 means the graph
/// will show data within 4 standard deviations from the mean in each direction,
/// covering approximately 99.994% of normally distributed data.
///
/// # Usage
///
/// This constant is used when setting the boundaries or range of statistical plots,
/// particularly when displaying probability distributions, confidence intervals,
/// or data with normal/Gaussian characteristics.
///
pub(crate) const STDDEV_MULTIPLAYER_GRAPH: f64 = 4.0;
