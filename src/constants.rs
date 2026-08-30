/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 11/8/24
******************************************************************************/
use positive::Positive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::num::NonZeroUsize;
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
/// Initialized once via `LazyLock`, like the six cells below it.
///
/// # Why the `Err` arm cannot fire
///
/// Each of these initialisers is a closure over nothing: its only input is a
/// `dec!(…)` literal, expanded to a fixed mantissa and scale at compile time.
/// [`Positive::new_decimal`] rejects exactly one thing, a value below the
/// lower bound — zero without the `non-zero` feature, which this crate does
/// not enable — and every literal here is strictly positive. There is no
/// runtime value that reaches the `Err` arm, and no `Positive` constant to
/// return in its place: `positive` keeps `from_decimal_const` crate-private,
/// so a `const` alternative does not exist for values outside its own
/// constant ladder. The alternative to aborting would be a stand-in like
/// `Positive::ZERO`, which would silently make every annualisation divide by
/// zero — worse than the abort it replaced.
pub(crate) static MIN_VOLATILITY: LazyLock<Positive> = LazyLock::new(|| {
    Positive::new_decimal(dec!(1e-16))
        .unwrap_or_else(|e| unreachable!("MIN_VOLATILITY literal is positive and finite: {e}")) // scan-banned: allow -- `dec!()` literal, `Err` arm has no reachable input; see MIN_VOLATILITY
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
        .unwrap_or_else(|e| unreachable!("TRADING_DAYS literal is positive and finite: {e}")) // scan-banned: allow -- `dec!()` literal, `Err` arm has no reachable input; see MIN_VOLATILITY
});

/// Standard number of trading hours in a market day as a `Positive` decimal.
///
/// Typically represents a standard U.S. market session (9:30 AM to 4:00 PM).
pub(crate) static TRADING_HOURS: LazyLock<Positive> = LazyLock::new(|| {
    Positive::new_decimal(dec!(6.5))
        .unwrap_or_else(|e| unreachable!("TRADING_HOURS literal is positive and finite: {e}")) // scan-banned: allow -- `dec!()` literal, `Err` arm has no reachable input; see MIN_VOLATILITY
});

/// Number of seconds in an hour as a `Positive` decimal value.
///
/// Used for time-based conversions and calculations.
pub(crate) static SECONDS_PER_HOUR: LazyLock<Positive> = LazyLock::new(|| {
    Positive::new_decimal(dec!(3600.0))
        .unwrap_or_else(|e| unreachable!("SECONDS_PER_HOUR literal is positive and finite: {e}")) // scan-banned: allow -- `dec!()` literal, `Err` arm has no reachable input; see MIN_VOLATILITY
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
        unreachable!("MICROSECONDS_PER_SECOND literal is positive and finite: {e}") // scan-banned: allow -- `dec!()` literal, `Err` arm has no reachable input; see MIN_VOLATILITY
    })
});

/// Standard number of weeks in a year as a `Positive` decimal value.
///
/// Used for time-based financial calculations and annualization.
pub(crate) static WEEKS_PER_YEAR: LazyLock<Positive> = LazyLock::new(|| {
    Positive::new_decimal(dec!(52.0))
        .unwrap_or_else(|e| unreachable!("WEEKS_PER_YEAR literal is positive and finite: {e}")) // scan-banned: allow -- `dec!()` literal, `Err` arm has no reachable input; see MIN_VOLATILITY
});

/// Number of months in a year as a `Positive` decimal value.
///
/// Upstream `positive::constants` skips 11/12 in its integer ladder, so the
/// value is built once via `LazyLock`.
pub(crate) static MONTHS_PER_YEAR: LazyLock<Positive> = LazyLock::new(|| {
    Positive::new_decimal(dec!(12.0))
        .unwrap_or_else(|e| unreachable!("MONTHS_PER_YEAR literal is positive and finite: {e}")) // scan-banned: allow -- `dec!()` literal, `Err` arm has no reachable input; see MIN_VOLATILITY
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

/// Default number of binomial-tree steps for `calculate_price_binomial` and
/// related lattice-based pricers.
///
/// Typed as `NonZeroUsize` so the type system enforces the non-zero invariant
/// at call sites, matching the public signatures migrated in #337.
///
/// # Why the `None` arm cannot fire
///
/// This and the three `NonZeroUsize` constants below are `const` items, so
/// the `match` is evaluated by the compiler, not at run time. `std` offers no
/// safe `const` literal for a non-zero integer, so the `Option` has to be
/// destructured; if the literal were ever changed to zero the build would
/// fail with `evaluation of constant value failed`, which is the outcome
/// wanted. No caller can reach the arm, because there is no run time at
/// which it exists.
pub const DEFAULT_BINOMIAL_STEPS: NonZeroUsize = match NonZeroUsize::new(100) {
    Some(n) => n,
    None => unreachable!(), // scan-banned: allow -- `const` context: an unreachable arm here fails compilation, it cannot abort at run time
};

/// Default number of Monte-Carlo simulation paths used by
/// `monte_carlo_option_pricing` and related samplers.
pub const DEFAULT_MC_PATHS: NonZeroUsize = match NonZeroUsize::new(10_000) {
    Some(n) => n,
    None => unreachable!(), // scan-banned: allow -- `const` context: an unreachable arm here fails compilation, it cannot abort at run time
};

/// Default number of time steps per Monte-Carlo path.
pub const DEFAULT_MC_STEPS: NonZeroUsize = match NonZeroUsize::new(252) {
    Some(n) => n,
    None => unreachable!(), // scan-banned: allow -- `const` context: an unreachable arm here fails compilation, it cannot abort at run time
};

/// Maximum Newton-Raphson iterations used by implied-volatility solvers.
///
/// Typed as `NonZeroUsize`; the crate-private `MAX_ITERATIONS_IV` constant
/// holds the `u32` diagnostic counterpart surfaced through
/// `VolatilityError`.
pub const MAX_NEWTON_ITER: NonZeroUsize = match NonZeroUsize::new(100) {
    Some(n) => n,
    None => unreachable!(), // scan-banned: allow -- `const` context: an unreachable arm here fails compilation, it cannot abort at run time
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Forces every `LazyLock` in this file.
    ///
    /// The `unreachable!` arms above are justified by the literals beside
    /// them being strictly positive and finite, which is true of the literals
    /// as written and of nothing else: `LazyLock` initialises at first
    /// access, at run time, so an edit that makes one of these literals zero
    /// or negative compiles cleanly and aborts on whichever request touches
    /// it first. Unlike the `const` items below, the compiler does not check
    /// the premise.
    ///
    /// Dereferencing each one here moves that failure into the test suite,
    /// where a bad literal is a red run rather than a dead worker. It is what
    /// makes the markers honest rather than merely reviewed.
    #[test]
    fn test_every_lazy_constant_initialises() {
        for (name, value) in [
            ("MIN_VOLATILITY", *MIN_VOLATILITY),
            ("TRADING_DAYS", *TRADING_DAYS),
            ("TRADING_HOURS", *TRADING_HOURS),
            ("SECONDS_PER_HOUR", *SECONDS_PER_HOUR),
            ("MICROSECONDS_PER_SECOND", *MICROSECONDS_PER_SECOND),
            ("WEEKS_PER_YEAR", *WEEKS_PER_YEAR),
            ("MONTHS_PER_YEAR", *MONTHS_PER_YEAR),
        ] {
            assert!(
                value > Positive::ZERO,
                "{name} must be strictly positive, or its unreachable arm is reachable"
            );
        }
    }
}
