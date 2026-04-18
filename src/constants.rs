/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 11/8/24
******************************************************************************/
use positive::Positive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::LazyLock;

/// Mathematical constant representing π (pi) with high precision using Decimal type.
/// Used for circular calculations, angle conversions, and geometric computations.
pub const PI: Decimal = dec!(3.1415926535897932384626433832);

/// Represents zero as a 64-bit floating point number.
/// Used as a baseline value for numerical comparisons and calculations.
pub const ZERO: f64 = 0.0;

/// Small decimal value used as a threshold for convergence tests and equality comparisons.
/// Represents a general tolerance level for numerical algorithms.
#[allow(dead_code)]
pub(crate) const TOLERANCE: Decimal = dec!(1e-8);

/// Extremely small decimal value used for high-precision calculations.
/// Represents the smallest meaningful difference in numerical computations.
pub const EPSILON: Decimal = dec!(1e-16);

/// Minimum allowed volatility value as a `Positive` decimal.
///
/// Prevents numerical issues in financial calculations with near-zero volatility.
/// Initialized once via `LazyLock`. `Positive::new_decimal(dec!(1e-16))` is
/// total because `dec!()` produces a compile-time non-negative `Decimal`, so
/// the `Err` arm is unreachable by construction; it trips `unreachable!` if
/// the invariant is ever broken (fail-fast instead of silent `Positive::ZERO`).
pub(crate) static MIN_VOLATILITY: LazyLock<Positive> = LazyLock::new(|| {
    Positive::new_decimal(dec!(1e-16))
        .unwrap_or_else(|e| unreachable!("MIN_VOLATILITY literal is positive and finite: {e}"))
});

/// Maximum allowed volatility value as a `Positive` decimal (100%).
///
/// Sets an upper bound for volatility inputs in financial models.
pub(crate) const MAX_VOLATILITY: Positive = Positive::HUNDRED;

/// Multiplier defining the lower bound for strike price ranges (98% of reference price).
/// Used to establish the minimum strike price in option chains or pricing models.
pub(crate) const STRIKE_PRICE_LOWER_BOUND_MULTIPLIER: f64 = 0.98;

/// Multiplier defining the upper bound for strike price ranges (102% of reference price).
/// Used to establish the maximum strike price in option chains or pricing models.
pub(crate) const STRIKE_PRICE_UPPER_BOUND_MULTIPLIER: f64 = 1.02;

/// Standard number of trading days in a year as a `Positive` decimal.
///
/// Used for business day-based financial calculations. See `MIN_VOLATILITY`
/// for the `LazyLock` rationale.
pub(crate) static TRADING_DAYS: LazyLock<Positive> = LazyLock::new(|| {
    Positive::new_decimal(dec!(252.0))
        .unwrap_or_else(|e| unreachable!("TRADING_DAYS literal is positive and finite: {e}"))
});

/// Standard number of trading hours in a market day as a `Positive` decimal.
///
/// Typically represents a standard U.S. market session (9:30 AM to 4:00 PM).
pub(crate) static TRADING_HOURS: LazyLock<Positive> = LazyLock::new(|| {
    Positive::new_decimal(dec!(6.5))
        .unwrap_or_else(|e| unreachable!("TRADING_HOURS literal is positive and finite: {e}"))
});

/// Number of seconds in an hour as a `Positive` decimal value.
///
/// Used for time-based conversions and calculations.
pub(crate) static SECONDS_PER_HOUR: LazyLock<Positive> = LazyLock::new(|| {
    Positive::new_decimal(dec!(3600.0))
        .unwrap_or_else(|e| unreachable!("SECONDS_PER_HOUR literal is positive and finite: {e}"))
});

/// Number of minutes in an hour as a `Positive` decimal value.
///
/// Aliased to `positive::constants::SIXTY`, which already exists upstream —
/// no runtime initialization required.
pub(crate) const MINUTES_PER_HOUR: Positive = positive::constants::SIXTY;

/// Number of milliseconds in a second as a `Positive` decimal value.
///
/// Aliased to `positive::constants::THOUSAND`, which already exists upstream.
pub(crate) const MILLISECONDS_PER_SECOND: Positive = positive::constants::THOUSAND;

/// Number of microseconds in a second as a `Positive` decimal value.
///
/// No matching `positive::constants::*` entry for `1_000_000`, so the value
/// is built once via `LazyLock`.
pub(crate) static MICROSECONDS_PER_SECOND: LazyLock<Positive> = LazyLock::new(|| {
    Positive::new_decimal(dec!(1_000_000.0)).unwrap_or_else(|e| {
        unreachable!("MICROSECONDS_PER_SECOND literal is positive and finite: {e}")
    })
});

/// Standard number of weeks in a year as a `Positive` decimal value.
///
/// Used for time-based financial calculations and annualization.
pub(crate) static WEEKS_PER_YEAR: LazyLock<Positive> = LazyLock::new(|| {
    Positive::new_decimal(dec!(52.0))
        .unwrap_or_else(|e| unreachable!("WEEKS_PER_YEAR literal is positive and finite: {e}"))
});

/// Number of months in a year as a `Positive` decimal value.
///
/// Upstream `positive::constants` skips 11/12 in its integer ladder, so the
/// value is built once via `LazyLock`.
pub(crate) static MONTHS_PER_YEAR: LazyLock<Positive> = LazyLock::new(|| {
    Positive::new_decimal(dec!(12.0))
        .unwrap_or_else(|e| unreachable!("MONTHS_PER_YEAR literal is positive and finite: {e}"))
});

/// Number of quarters in a year as a `Positive` decimal value.
///
/// Aliased to `positive::constants::FOUR`, which already exists upstream.
pub(crate) const QUARTERS_PER_YEAR: Positive = positive::constants::FOUR;

/// Maximum number of iterations for implied volatility calculation algorithms.
/// Prevents infinite loops in numerical methods like Newton-Raphson or bisection.
pub(crate) const MAX_ITERATIONS_IV: u32 = 1000;

/// Convergence tolerance for implied volatility calculations.
/// Determines when the implied volatility solver has reached sufficient precision.
pub(crate) const IV_TOLERANCE: Decimal = dec!(1e-5);
