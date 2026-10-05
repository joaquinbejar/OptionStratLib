/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 5/8/24
******************************************************************************/
//! Solver and lattice defaults of the pricing layer (ADR-0001 D2).

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::num::NonZeroUsize;

pub(crate) const CLAMP_MIN: Decimal = Decimal::ZERO;
pub(crate) const CLAMP_MAX: Decimal = Decimal::ONE;

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
