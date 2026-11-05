/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 5/11/26
******************************************************************************/

//! Butterfly and calendar checks of a volatility slice or surface, on a grid.
//!
//! The two conditions are those of Gatheral and Jacquier, "Arbitrage-free SVI
//! volatility surfaces" (arXiv:1204.0646), Definition 2.1.
//!
//! # What passing means
//!
//! [`StaticArbitrageReport::passed`] says only that these two checks found
//! nothing at the grid points supplied, on the slices supplied. It does **not**
//! say that a slice or a surface is free of arbitrage:
//!
//! - A violation between two grid points is not detected. Both conditions hold
//!   for every `k` in the paper and are tested here only at the points of the
//!   grid, so a coarse grid can pass a slice a finer one fails. The caller
//!   chooses the grid, and the wings it does not reach are not tested.
//! - A violation between two expiries is not detected: only the expiries
//!   supplied are compared, and only at the points of the grid.
//! - Lemma 2.2 of the paper also requires `d+(k) -> -inf` as `k -> +inf`, so that
//!   call prices vanish in the far wing. That limit is not tested.
//! - Lemma 2.1 equates the calendar condition with monotone option prices when
//!   dividends are proportional to the stock price. With other dividends the
//!   condition is only what it says on total variance.
//!
//! # The butterfly check
//!
//! At each grid point the check evaluates equation (2.1) of the paper,
//!
//! ```text
//! g(k) = (1 - k * w'(k) / (2 * w(k)))^2 - (w'(k)^2 / 4) * (1 / w(k) + 1 / 4) + w''(k) / 2
//! ```
//!
//! whose sign is the sign of the density, by
//! `p(k) = g(k) / sqrt(2 * pi * w(k)) * exp(-d-(k)^2 / 2)`. A point with
//! `g(k) < 0` beyond the rounding of `g` is a
//! [`StaticArbitrageViolation::Butterfly`], and its `size` is `-g(k)`. `g` has
//! no unit, and it is not the density: the factor between them is left out
//! because it can be far below one in the wings, where the sign still counts.
//!
//! # The calendar check
//!
//! Total variance must not decrease with expiry at fixed log moneyness. The
//! check sorts the slices by expiry, so the order of the input does not matter,
//! and compares each with the next at every grid point. A point where the earlier
//! expiry has more total variance beyond the rounding of both is a
//! [`StaticArbitrageViolation::Calendar`], and its `size` is the difference, in
//! total variance (annualized variance times years).
//!
//! Comparing adjacent expiries is enough for the points tested only up to the
//! bands, which accumulate along the chain: every step can decrease by just less
//! than its band and report nothing, while the first expiry is above the last by
//! the sum of those bands.
//!
//! The comparison is at equal log moneyness, not at equal strike, and each slice
//! has its own forward, so the same point is two different strikes. A calendar
//! violation reports both: `earlier_strike` is `F * exp(k)` with the forward of
//! the earlier expiry, and `later_strike` with that of the later one.
//!
//! # Where `g` has no value
//!
//! `g` divides by `w`, and `w = 0` is reachable: a raw SVI slice with a zero
//! minimum touches it at its vertex `k* = m - rho * sigma / sqrt(1 - rho^2)`,
//! where `w'` is zero too, so the terms in `1 / w` read `0 / 0`.
//!
//! Near such a vertex, with `h = k - k*`, `w ~ w'' * h^2 / 2` and `w' ~ w'' * h`,
//! so `w'^2 / (4 * w)` tends to `w'' / 2` and cancels the last term of `g`. What
//! remains depends on where the vertex is. For `k* != 0` the first term grows as
//! `(k* / h)^2` and `g` tends to `+inf`. For `k* = 0` it tends to zero, and from
//! below, as `-(3 * b + b^2 * sigma / 2) * k^2 / (8 * sigma^3)` for `rho = 0` and
//! `m = 0`. Neither limit is a value of `g` at the point, so the check does not
//! substitute one:
//!
//! - Such a point is neither a violation nor a pass. It is a
//!   [`StaticArbitrageUnevaluatedPoint`] in [`StaticArbitrageReport::unevaluated`],
//!   and [`StaticArbitrageReport::passed`] is then false.
//! - It deserves the attention. A zero total variance at a strike means that an
//!   option there has no time value, while options further out have some, so the
//!   option prices are not monotone in the strike. The check finds the negative
//!   `g` this implies only at a grid point where `g` is negative beyond its band:
//!   beside the vertex when `k* = 0`, past the window below, and out in the wing
//!   when `k* != 0`.
//!
//! A point with a tiny `w` gets the same entry when its value is swamped by its
//! rounding: when the rounding [`TotalVarianceSlice::total_variance_rounding`]
//! declares exceeds `1e-12` of `w`.
//!
//! This leaves a window where a real violation goes unreported. For a raw SVI
//! slice with `a = -0.01`, `b = 0.1` and `sigma = 0.1`, which has a zero minimum at
//! `k = 0`, points closer than about `2e-7` have `w` below about `2e-14` and are
//! unevaluated, an implied volatility below about `0.00002%` at one year. Points
//! from there to about `3.2e-7` pass, although the true `g` is negative there,
//! because it is within its band: about `-4e-12` against a band of about `4e-12`.
//! Points further out are reported.
//!
//! The calendar check has the same limit on a different reference. A pair of
//! expiries is a [`StaticArbitrageUnevaluatedPoint::Calendar`] when either declared
//! rounding exceeds `1e-12` of the larger of their two total variances, because a
//! band that wide could hide a real decrease. A real decrease of up to
//! `2e-12 * max(w)` can therefore still pass. This compares neither slice with its
//! own `w`, so a zero `w` beside a large one is still a resolved comparison.
//!
//! # Tolerances
//!
//! Each tolerance is an upper bound on the rounding of the arithmetic at that point,
//! so at an evaluated point a violation larger than it is always reported. A
//! violation smaller than it is not reported, although it may be real, because the
//! bounds are conservative: rounding could explain it, but need not. Their
//! margins are choices: `1e-24` is about `1e4` units in the last place at a `w` near
//! one, `1e-26` is 100 of them, and `1e-12` moves points between evaluated and
//! unevaluated.
//!
//! - Calendar: the sum of the rounding each slice declares at the point. Equal
//!   total variances differ by zero, which is not a violation at any tolerance.
//!   For a raw SVI slice the rounding is
//!   `1e-24 * (|a| + b * R) + b * min(4e-28 / R, 0.42 * R) + 4e-28 * (1 + b)` with
//!   `R = sqrt((k - m)^2 + sigma^2)`: about `2e-25` at `a = 0.1`, `b = 0.1`, `R = 1`.
//!   The minimum peaks at about `1.3e-14 * b`, at `R` near `3e-14`.
//! - Butterfly: `g` is a sum of terms that each carry the rounding of `w`, in
//!   proportion to their size, so its band is
//!   `((1 + |q| + |k|)^2 + w'^2 / (4 * w) + w'^2 / 16 + |w''| / 2) * (2 * r + 1e-26)
//!   + |w'| * rounding / 4`
//!   with `q = k * w' / (2 * w)`, `rounding` that of `w`, and `r` the rounding of `w`
//!   over `w`. `1e-26` is 100 units in the last place of a `Decimal`, for the
//!   arithmetic of `g` itself.
//!   At a raw SVI point of ordinary size this is about `1e-23`. Where `w` is barely
//!   resolved, `r` is `1e-12` and the band is `2e-12` times the size of the terms.
//!
//! These bounds were compared with a 150-digit reference on about 250000 random raw
//! SVI points: `b` from `1e-6` to `1e21`, `sigma` from `1e-28` to `100`, `|rho|` up to
//! `0.999999`, `|m|` up to `2`, `a` from the floor to about `9 * b`, and `k` up to
//! about `9e9` from the vertex; and on about 98000 random slices declaring no rounding,
//! with `w`, `|w'|` and `|w''|` from `1e-28` to `1e28` and `|k|` up to `1e21`. No
//! evaluated `g` was outside its band, and no `w` or `w'` of a raw SVI point outside
//! its rounding. Much of that range, at a large `b` or `w'`, comes back as a typed
//! overflow rather than a value, since `w'^2` alone leaves the range of a `Decimal`.
//! Outside that range only the argument above stands.
//!
//! # Failure
//!
//! A check that cannot be evaluated never reads as clean. An empty grid, fewer
//! than two slices for a calendar check, two slices with one expiry, a slice
//! without a forward or an expiry, and any overflow of `Decimal` at extreme
//! parameters are [`StaticArbitrageError`]s, and the report is not returned.
//!
//! A strike is only computed for a finding. A grid point whose `F * exp(k)` leaves
//! the range, at `|k|` of about 65 and beyond, is an error if it has a finding to
//! name, and not an error if it has nothing to report. It should not be read as
//! tested either: out there the butterfly band grows with `k^2`. At `a = 22.272553`,
//! `b = 4.40013`, `rho = -0.5`, `m = 0`, `sigma = 8e-28`, `k = 121142260000`, the
//! true `g` is `-0.0525` and it passes within a band of `0.0588`.
//!
//! # Example
//!
//! ```rust
//! use optionstratlib_core::model::Positive;
//! use optionstratlib_pricing::volatility::{SviParameters, SviSlice, check_butterfly};
//! use rust_decimal_macros::dec;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Example 3.1 of the paper, whose density is negative in the right wing.
//! let parameters =
//!     SviParameters::try_new(dec!(-0.0410), dec!(0.1331), dec!(0.3060), dec!(0.3586), dec!(0.4153))?;
//! let slice = SviSlice::try_new(parameters, Positive::HUNDRED, Positive::ONE)?;
//!
//! let fine = [dec!(0), dec!(0.5), dec!(0.9), dec!(1.5)];
//! assert_eq!(check_butterfly(&slice, &fine)?.violations().len(), 1);
//!
//! // The same slice passes on a grid that steps over the violation, which says
//! // only that nothing was found at these points.
//! let coarse = [dec!(0), dec!(0.5), dec!(1.5)];
//! assert!(check_butterfly(&slice, &coarse)?.passed());
//! # Ok(())
//! # }
//! ```

use super::svi::SviSlice;
use crate::error::StaticArbitrageError;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::{d_add, d_div, d_exp, d_mul, d_sub};
use rust_decimal::Decimal;
use tracing::{debug, instrument};

/// Slices the calendar check needs to compare one expiry with another.
const MIN_CALENDAR_SLICES: usize = 2;

/// The largest rounding of `w`, relative to `w`, at which a point is still evaluated:
/// `w` then carries at least 12 significant digits. Above it the point is reported
/// as unevaluated. Lowering it evaluates fewer points; raising it would let the
/// rounding of `w` grow into the terms of `g` that cancel near a vertex, and into
/// the difference a calendar comparison reads.
const MAX_RELATIVE_ROUNDING: Decimal = Decimal::from_parts(1, 0, 0, false, 12);

/// Rounding of the arithmetic of `g` itself, relative to the size of its terms:
/// 100 units in the last place of a `Decimal`, for its handful of operations.
const G_ARITHMETIC_ROUNDING: Decimal = Decimal::from_parts(1, 0, 0, false, 26);

/// The 4 in `w' * (w' / w) / 4` and in `w' / 4`.
const FOUR: Decimal = Decimal::from_parts(4, 0, 0, false, 0);

/// A slice of total implied variance, one expiry of a surface, as the static
/// arbitrage checks read it.
///
/// Total variance is annualized variance times years, a function of the log
/// moneyness `k = ln(K / F)`. An implementor gives it and its first two
/// derivatives in `k`, and says how much of it is rounding.
///
/// Every method must be stable: the same value on each call with the same argument.
/// A check reads each expiry and forward once, so a slice that breaks this cannot
/// make it panic, but what it reports about such a slice means nothing.
pub trait TotalVarianceSlice {
    /// The time to expiry of the slice, in years. Must be positive.
    fn time_to_expiry(&self) -> Positive;

    /// The forward that log moneyness is measured against. Must be positive.
    fn forward(&self) -> Positive;

    /// The total variance `w(k)`.
    ///
    /// # Errors
    ///
    /// A [`StaticArbitrageError`] if the slice cannot be evaluated at `k`. The
    /// check then fails rather than skip the point.
    fn total_variance(&self, k: Decimal) -> Result<Positive, StaticArbitrageError>;

    /// The first derivative `w'(k)`, in total variance per unit of log moneyness.
    ///
    /// # Errors
    ///
    /// A [`StaticArbitrageError`] if the slice cannot be evaluated at `k`. The
    /// check then fails rather than skip the point.
    fn total_variance_slope(&self, k: Decimal) -> Result<Decimal, StaticArbitrageError>;

    /// The second derivative `w''(k)`, in total variance per squared unit of log
    /// moneyness.
    ///
    /// # Errors
    ///
    /// A [`StaticArbitrageError`] if the slice cannot be evaluated at `k`. The
    /// check then fails rather than skip the point.
    fn total_variance_curvature(&self, k: Decimal) -> Result<Decimal, StaticArbitrageError>;

    /// An upper bound on the absolute rounding of `w(k)` and `w'(k)` as computed.
    ///
    /// This is what keeps the arithmetic from creating a violation, so it should
    /// be a true bound and a small one: too small and rounding can read as a
    /// violation, too large and a small violation can read as rounding, until a
    /// point above `1e-12` of `w` is reported as unevaluated. A slice whose values
    /// are exact returns zero.
    ///
    /// The butterfly band also assumes that the relative rounding of `w''(k)` is
    /// within the same factor it applies to the other terms of `g`:
    /// `2 * rounding / w + 1e-26`.
    ///
    /// # Errors
    ///
    /// A [`StaticArbitrageError`] if the bound cannot be computed at `k`. The
    /// check then fails rather than skip the point.
    fn total_variance_rounding(&self, k: Decimal) -> Result<Positive, StaticArbitrageError>;
}

impl<T: TotalVarianceSlice + ?Sized> TotalVarianceSlice for &T {
    #[inline]
    fn time_to_expiry(&self) -> Positive {
        (**self).time_to_expiry()
    }

    #[inline]
    fn forward(&self) -> Positive {
        (**self).forward()
    }

    fn total_variance(&self, k: Decimal) -> Result<Positive, StaticArbitrageError> {
        (**self).total_variance(k)
    }

    fn total_variance_slope(&self, k: Decimal) -> Result<Decimal, StaticArbitrageError> {
        (**self).total_variance_slope(k)
    }

    fn total_variance_curvature(&self, k: Decimal) -> Result<Decimal, StaticArbitrageError> {
        (**self).total_variance_curvature(k)
    }

    fn total_variance_rounding(&self, k: Decimal) -> Result<Positive, StaticArbitrageError> {
        (**self).total_variance_rounding(k)
    }
}

impl TotalVarianceSlice for SviSlice {
    #[inline]
    fn time_to_expiry(&self) -> Positive {
        SviSlice::time_to_expiry(self)
    }

    #[inline]
    fn forward(&self) -> Positive {
        SviSlice::forward(self)
    }

    fn total_variance(&self, k: Decimal) -> Result<Positive, StaticArbitrageError> {
        Ok(SviSlice::total_variance(self, k)?)
    }

    fn total_variance_slope(&self, k: Decimal) -> Result<Decimal, StaticArbitrageError> {
        Ok(SviSlice::total_variance_slope(self, k)?)
    }

    fn total_variance_curvature(&self, k: Decimal) -> Result<Decimal, StaticArbitrageError> {
        Ok(SviSlice::total_variance_curvature(self, k)?)
    }

    fn total_variance_rounding(&self, k: Decimal) -> Result<Positive, StaticArbitrageError> {
        Ok(self.parameters().total_variance_rounding(k)?)
    }
}

/// One point where a check failed, beyond the rounding of the arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StaticArbitrageViolation {
    /// The density of one expiry is negative at this point: `g(k) < 0`.
    #[non_exhaustive]
    Butterfly {
        /// The expiry of the slice, in years.
        time_to_expiry: Positive,
        /// The grid point `k`.
        log_moneyness: Decimal,
        /// `F * exp(k)` with the forward of the slice.
        strike: Positive,
        /// The value of `g(k)`, which has no unit.
        g: Decimal,
        /// `-g(k)`, which has no unit.
        size: Positive,
    },
    /// Total variance decreases from one expiry to the next at this point.
    #[non_exhaustive]
    Calendar {
        /// The earlier of the two expiries, in years.
        earlier_expiry: Positive,
        /// The later of the two expiries, in years.
        later_expiry: Positive,
        /// The grid point `k`, at which both slices are compared.
        log_moneyness: Decimal,
        /// `F * exp(k)` with the forward of the earlier expiry.
        earlier_strike: Positive,
        /// `F * exp(k)` with the forward of the later expiry.
        later_strike: Positive,
        /// `w(k)` of the earlier expiry.
        earlier_total_variance: Positive,
        /// `w(k)` of the later expiry.
        later_total_variance: Positive,
        /// Earlier minus later total variance, in total variance (annualized
        /// variance times years).
        size: Positive,
    },
}

impl StaticArbitrageViolation {
    /// The grid point of the violation.
    #[must_use]
    #[inline]
    pub fn log_moneyness(&self) -> Decimal {
        match self {
            Self::Butterfly { log_moneyness, .. } | Self::Calendar { log_moneyness, .. } => {
                *log_moneyness
            }
        }
    }

    /// How far the condition is broken, in the unit its variant documents: no
    /// unit for a butterfly, total variance for a calendar.
    #[must_use]
    #[inline]
    pub fn size(&self) -> Positive {
        match self {
            Self::Butterfly { size, .. } | Self::Calendar { size, .. } => *size,
        }
    }
}

/// A grid point a check could not resolve, because the declared rounding is too
/// large against the total variance there. It is neither a violation nor a pass.
/// See the module docs for what this means and why it is reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StaticArbitrageUnevaluatedPoint {
    /// `g` has no value here: `w` is zero, or smaller than its rounding can resolve.
    /// The calendar check does not depend on `g`, and still runs at this point.
    #[non_exhaustive]
    Butterfly {
        /// The expiry of the slice, in years.
        time_to_expiry: Positive,
        /// The grid point `k`.
        log_moneyness: Decimal,
        /// `F * exp(k)` with the forward of the slice.
        strike: Positive,
        /// The total variance `w(k)`, as computed.
        total_variance: Positive,
        /// The rounding of `w(k)` the slice declares.
        rounding: Positive,
    },
    /// A decrease could be hidden within the rounding of one of these two expiries.
    #[non_exhaustive]
    Calendar {
        /// The earlier of the two expiries, in years.
        earlier_expiry: Positive,
        /// The later of the two expiries, in years.
        later_expiry: Positive,
        /// The grid point `k`, at which both slices are compared.
        log_moneyness: Decimal,
        /// `F * exp(k)` with the forward of the earlier expiry.
        earlier_strike: Positive,
        /// `F * exp(k)` with the forward of the later expiry.
        later_strike: Positive,
        /// `w(k)` of the earlier expiry, as computed.
        earlier_total_variance: Positive,
        /// `w(k)` of the later expiry, as computed.
        later_total_variance: Positive,
        /// The rounding of `w(k)` the earlier expiry declares.
        earlier_rounding: Positive,
        /// The rounding of `w(k)` the later expiry declares.
        later_rounding: Positive,
    },
}

impl StaticArbitrageUnevaluatedPoint {
    /// The grid point that could not be resolved.
    #[must_use]
    #[inline]
    pub fn log_moneyness(&self) -> Decimal {
        match self {
            Self::Butterfly { log_moneyness, .. } | Self::Calendar { log_moneyness, .. } => {
                *log_moneyness
            }
        }
    }
}

/// What the static arbitrage checks found, point by point.
///
/// An empty report means only that these checks found nothing at the grid points
/// tested: see the module docs. It does not say the slices are free of arbitrage.
///
/// Only a check builds a report, so one always says a check ran:
///
/// ```compile_fail
/// use optionstratlib_pricing::volatility::StaticArbitrageReport;
///
/// let _ = StaticArbitrageReport::default();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticArbitrageReport {
    violations: Vec<StaticArbitrageViolation>,
    unevaluated: Vec<StaticArbitrageUnevaluatedPoint>,
}

impl StaticArbitrageReport {
    /// A report of a check about to run.
    fn empty() -> Self {
        Self {
            violations: Vec::new(),
            unevaluated: Vec::new(),
        }
    }

    /// The violations found: those of the butterfly check, by expiry and then in
    /// the order of the grid, then those of the calendar check, by pair of
    /// expiries and then in the order of the grid.
    #[must_use]
    #[inline]
    pub fn violations(&self) -> &[StaticArbitrageViolation] {
        &self.violations
    }

    /// The grid points a check could not resolve: those of the butterfly check, by
    /// expiry and then in the order of the grid, then those of the calendar check,
    /// by pair of expiries and then in the order of the grid. These are not
    /// violations, and that check did not clear them either. Each entry names only
    /// its own check: the other check can still have run at the same point.
    #[must_use]
    #[inline]
    pub fn unevaluated(&self) -> &[StaticArbitrageUnevaluatedPoint] {
        &self.unevaluated
    }

    /// Whether every point was evaluated and none failed.
    ///
    /// This means only that the checks run found nothing at the grid points
    /// supplied, on the slices supplied. Between grid points and between expiries
    /// nothing is known, so this does not say a slice is free of arbitrage.
    #[must_use]
    #[inline]
    pub fn passed(&self) -> bool {
        self.violations.is_empty() && self.unevaluated.is_empty()
    }
}

/// Checks the density of one slice at each point of a log moneyness grid.
///
/// A point with `g(k) < 0` beyond its rounding is a violation; a point with a `w`
/// of zero, or too small to resolve, is reported as unevaluated. See the module
/// docs. Grid points are read in the order given, need not be sorted, and each
/// repeat of a point reports again.
///
/// # Errors
///
/// - [`StaticArbitrageError::EmptyGrid`]: the grid has no point.
/// - [`StaticArbitrageError::InvalidSlice`]: the slice has a zero forward or a
///   zero time to expiry.
/// - [`StaticArbitrageError::StrikeOutOfRange`]: a finding is at a point whose
///   strike leaves the range of a `Positive`.
/// - [`StaticArbitrageError::Decimal`], [`StaticArbitrageError::Positive`], or any
///   error of the slice: `Decimal` overflow at extreme parameters.
#[instrument(skip(slice, grid), fields(points = grid.len()), level = "debug")]
pub fn check_butterfly<S: TotalVarianceSlice>(
    slice: &S,
    grid: &[Decimal],
) -> Result<StaticArbitrageReport, StaticArbitrageError> {
    check_grid(grid)?;
    let read = read_slice(slice)?;
    let mut report = StaticArbitrageReport::empty();
    butterfly_into(&read, grid, &mut report)?;
    debug!(
        violations = report.violations.len(),
        unevaluated = report.unevaluated.len(),
        "butterfly check finished"
    );
    Ok(report)
}

/// Checks that total variance does not decrease with expiry, at each point of a
/// log moneyness grid.
///
/// The slices may come in any order: they are sorted by expiry, and each is
/// compared with the next. A pair whose declared rounding is too large against
/// its total variance at a point is reported as unevaluated there. See the module
/// docs.
///
/// # Errors
///
/// - [`StaticArbitrageError::EmptyGrid`]: the grid has no point.
/// - [`StaticArbitrageError::TooFewSlices`]: fewer than two slices.
/// - [`StaticArbitrageError::DuplicateExpiry`]: two slices share an expiry, even
///   with equal total variances.
/// - [`StaticArbitrageError::InvalidSlice`]: a slice has a zero forward or a zero
///   time to expiry.
/// - [`StaticArbitrageError::StrikeOutOfRange`]: a finding is at a point whose
///   strike leaves the range of a `Positive`.
/// - [`StaticArbitrageError::Decimal`], [`StaticArbitrageError::Positive`], or any
///   error of a slice: `Decimal` overflow at extreme parameters.
#[instrument(skip(slices, grid), fields(slices = slices.len(), points = grid.len()), level = "debug")]
pub fn check_calendar<S: TotalVarianceSlice>(
    slices: &[S],
    grid: &[Decimal],
) -> Result<StaticArbitrageReport, StaticArbitrageError> {
    check_grid(grid)?;
    let sorted = sorted_by_expiry(slices)?;
    let mut report = StaticArbitrageReport::empty();
    calendar_into(&sorted, grid, &mut report)?;
    debug!(
        violations = report.violations.len(),
        unevaluated = report.unevaluated.len(),
        "calendar check finished"
    );
    Ok(report)
}

/// Runs the butterfly check on every slice and the calendar check across them,
/// on one log moneyness grid.
///
/// The slices may come in any order. A single slice has no calendar check to
/// run, so it is an error here rather than a report that reads as if it had one:
/// call [`check_butterfly`] for it. See the module docs for what the result means
/// and does not mean.
///
/// # Errors
///
/// The errors of [`check_butterfly`] and [`check_calendar`].
#[instrument(skip(slices, grid), fields(slices = slices.len(), points = grid.len()), level = "debug")]
pub fn check_static_arbitrage<S: TotalVarianceSlice>(
    slices: &[S],
    grid: &[Decimal],
) -> Result<StaticArbitrageReport, StaticArbitrageError> {
    check_grid(grid)?;
    let sorted = sorted_by_expiry(slices)?;
    let mut report = StaticArbitrageReport::empty();
    for read in &sorted {
        butterfly_into(read, grid, &mut report)?;
    }
    calendar_into(&sorted, grid, &mut report)?;
    debug!(
        violations = report.violations.len(),
        unevaluated = report.unevaluated.len(),
        "static arbitrage check finished"
    );
    Ok(report)
}

/// Rejects a grid that would test nothing.
fn check_grid(grid: &[Decimal]) -> Result<(), StaticArbitrageError> {
    if grid.is_empty() {
        return Err(StaticArbitrageError::EmptyGrid);
    }
    Ok(())
}

/// A slice with the expiry and the forward it gave when read, once, so that sorting
/// and reporting cannot see two different values of either.
struct ReadSlice<'a, S> {
    expiry: Positive,
    forward: Positive,
    slice: &'a S,
}

/// Reads a slice, rejecting one whose strikes or expiry would mean nothing.
fn read_slice<S: TotalVarianceSlice>(slice: &S) -> Result<ReadSlice<'_, S>, StaticArbitrageError> {
    let expiry = slice.time_to_expiry();
    let forward = slice.forward();
    let invalid = |reason| StaticArbitrageError::InvalidSlice {
        time_to_expiry: expiry,
        forward,
        reason,
    };
    if forward.is_zero() {
        return Err(invalid("the forward must be positive"));
    }
    if expiry.is_zero() {
        return Err(invalid("the time to expiry must be positive"));
    }
    Ok(ReadSlice {
        expiry,
        forward,
        slice,
    })
}

/// The slices by increasing expiry, each of them read once, checked, and no two
/// sharing an expiry.
fn sorted_by_expiry<S: TotalVarianceSlice>(
    slices: &[S],
) -> Result<Vec<ReadSlice<'_, S>>, StaticArbitrageError> {
    if slices.len() < MIN_CALENDAR_SLICES {
        return Err(StaticArbitrageError::TooFewSlices {
            got: slices.len(),
            required: MIN_CALENDAR_SLICES,
        });
    }
    let mut sorted = Vec::with_capacity(slices.len());
    for slice in slices {
        sorted.push(read_slice(slice)?);
    }
    sorted.sort_by_key(|read| read.expiry);
    for pair in sorted.windows(2) {
        if let [earlier, later] = pair
            && earlier.expiry == later.expiry
        {
            return Err(StaticArbitrageError::DuplicateExpiry {
                time_to_expiry: earlier.expiry,
            });
        }
    }
    Ok(sorted)
}

/// `forward * exp(k)`. A strike that overflows or rounds to zero is not one.
fn strike_at(forward: Positive, k: Decimal) -> Result<Positive, StaticArbitrageError> {
    const OP: &str = "arbitrage::strike";
    let out_of_range = || StaticArbitrageError::StrikeOutOfRange {
        forward,
        log_moneyness: k,
    };
    let growth = d_exp(k, OP).map_err(|_| out_of_range())?;
    let strike = d_mul(forward.to_dec(), growth, OP).map_err(|_| out_of_range())?;
    if strike.is_zero() {
        return Err(out_of_range());
    }
    Ok(Positive::new_decimal(strike)?)
}

/// `g(k)` and the bound on its rounding.
#[derive(Debug, Clone, Copy)]
struct DensityFactor {
    g: Decimal,
    band: Decimal,
}

/// Evaluates `g` at a point whose `w` is positive and resolved.
///
/// `w'` is never halved before it is used: a value near the last place would round
/// away, and `w' / w` can be large however small `w'` is. So `p = w' / w` comes
/// first, `q = k * p / 2`, and `w'^2 / (4 * w)` is `w' * p / 4`.
///
/// Each step then rounds by at most about one unit in the last place, or one part
/// in `1e28` of a larger value. `p` being that far off moves `q` by `|k|` units and
/// `w' * p / 4` by `|w'|` units plus a part in `1e28` of itself, and a term that
/// rounds to zero was below one unit. So all of it is within `1e-26`, 100 units, of
/// the first term of `magnitude`, which is never below one, using that `|w'| / 2` is
/// at most `1 + w'^2 / 16` by the arithmetic-geometric mean.
///
/// The rounding the slice declares comes on top. Relative to `w`, called `r` here,
/// it moves `q` by up to `(|q| + |k| / 2) * r`, since it also bounds that of `w'`,
/// which gives the first term of `magnitude` again, and it is within `magnitude` for
/// `w' * p / 4` by the same mean. `w'^2 / 16` has no `1 / w` to put its `w'` rounding
/// in terms of `r`, so that one is added on its own. Any product here that rounds to
/// zero is below one unit, which is within the `1e-26` too.
fn density_factor(
    k: Decimal,
    w: Decimal,
    slope: Decimal,
    curvature: Decimal,
    rounding: Decimal,
) -> Result<DensityFactor, StaticArbitrageError> {
    const OP: &str = "arbitrage::g";
    let ratio = d_div(slope, w, OP)?;
    let q = d_div(d_mul(k, ratio, OP)?, Decimal::TWO, OP)?;
    let one_minus_q = d_sub(Decimal::ONE, q, OP)?;
    let first = d_mul(one_minus_q, one_minus_q, OP)?;
    let over_w = d_div(d_mul(slope, ratio, OP)?, FOUR, OP)?;
    let quarter_slope = d_div(slope, FOUR, OP)?;
    let over_four = d_mul(quarter_slope, quarter_slope, OP)?;
    let half_curvature = d_div(curvature, Decimal::TWO, OP)?;
    let g = d_add(
        d_sub(d_sub(first, over_w, OP)?, over_four, OP)?,
        half_curvature,
        OP,
    )?;

    let spread = d_add(d_add(Decimal::ONE, q.abs(), OP)?, k.abs(), OP)?;
    let magnitude = d_add(
        d_add(d_mul(spread, spread, OP)?, over_w, OP)?,
        d_add(over_four, half_curvature.abs(), OP)?,
        OP,
    )?;
    let relative = d_add(
        d_mul(Decimal::TWO, d_div(rounding, w, OP)?, OP)?,
        G_ARITHMETIC_ROUNDING,
        OP,
    )?;
    let over_four_rounding = d_mul(quarter_slope.abs(), rounding, OP)?;
    Ok(DensityFactor {
        g,
        band: d_add(d_mul(magnitude, relative, OP)?, over_four_rounding, OP)?,
    })
}

/// Adds what one slice shows on the grid to the report.
fn butterfly_into<S: TotalVarianceSlice>(
    read: &ReadSlice<'_, S>,
    grid: &[Decimal],
    report: &mut StaticArbitrageReport,
) -> Result<(), StaticArbitrageError> {
    let slice = read.slice;
    for &k in grid {
        let total_variance = slice.total_variance(k)?;
        let rounding = slice.total_variance_rounding(k)?;
        let w = total_variance.to_dec();
        let limit = d_mul(MAX_RELATIVE_ROUNDING, w, "arbitrage::g::limit")?;
        if w.is_zero() || rounding.to_dec() > limit {
            report
                .unevaluated
                .push(StaticArbitrageUnevaluatedPoint::Butterfly {
                    time_to_expiry: read.expiry,
                    log_moneyness: k,
                    strike: strike_at(read.forward, k)?,
                    total_variance,
                    rounding,
                });
            continue;
        }
        let factor = density_factor(
            k,
            w,
            slice.total_variance_slope(k)?,
            slice.total_variance_curvature(k)?,
            rounding.to_dec(),
        )?;
        if factor.g < -factor.band {
            report.violations.push(StaticArbitrageViolation::Butterfly {
                time_to_expiry: read.expiry,
                log_moneyness: k,
                strike: strike_at(read.forward, k)?,
                g: factor.g,
                size: Positive::new_decimal(-factor.g)?,
            });
        }
    }
    Ok(())
}

/// Adds what each pair of adjacent expiries shows on the grid to the report.
///
/// Each rounding is held to the limit before the two are summed, so any declared
/// rounding, however large, ends here as an unevaluated point. The reference is the
/// larger `w`: a comparison of a zero `w` with a large one is resolved.
fn calendar_into<S: TotalVarianceSlice>(
    sorted: &[ReadSlice<'_, S>],
    grid: &[Decimal],
    report: &mut StaticArbitrageReport,
) -> Result<(), StaticArbitrageError> {
    const OP: &str = "arbitrage::calendar";
    for pair in sorted.windows(2) {
        let [earlier, later] = pair else {
            continue;
        };
        for &k in grid {
            let earlier_w = earlier.slice.total_variance(k)?;
            let later_w = later.slice.total_variance(k)?;
            let earlier_rounding = earlier.slice.total_variance_rounding(k)?;
            let later_rounding = later.slice.total_variance_rounding(k)?;
            let limit = d_mul(
                MAX_RELATIVE_ROUNDING,
                earlier_w.to_dec().max(later_w.to_dec()),
                OP,
            )?;
            if earlier_rounding.to_dec() > limit || later_rounding.to_dec() > limit {
                report
                    .unevaluated
                    .push(StaticArbitrageUnevaluatedPoint::Calendar {
                        earlier_expiry: earlier.expiry,
                        later_expiry: later.expiry,
                        log_moneyness: k,
                        earlier_strike: strike_at(earlier.forward, k)?,
                        later_strike: strike_at(later.forward, k)?,
                        earlier_total_variance: earlier_w,
                        later_total_variance: later_w,
                        earlier_rounding,
                        later_rounding,
                    });
                continue;
            }
            let decrease = d_sub(earlier_w.to_dec(), later_w.to_dec(), OP)?;
            let band = d_add(earlier_rounding.to_dec(), later_rounding.to_dec(), OP)?;
            if decrease > band {
                report.violations.push(StaticArbitrageViolation::Calendar {
                    earlier_expiry: earlier.expiry,
                    later_expiry: later.expiry,
                    log_moneyness: k,
                    earlier_strike: strike_at(earlier.forward, k)?,
                    later_strike: strike_at(later.forward, k)?,
                    earlier_total_variance: earlier_w,
                    later_total_variance: later_w,
                    size: Positive::new_decimal(decrease)?,
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::volatility::SviParameters;
    use rust_decimal_macros::dec;

    fn pos(value: Decimal) -> Positive {
        Positive::new_decimal(value).unwrap()
    }

    /// A slice that returns what it is told at every point, to place `g` and the
    /// bands exactly where a test wants them.
    #[derive(Debug, Clone, Copy)]
    struct Fixed {
        expiry: Positive,
        forward: Positive,
        w: Decimal,
        slope: Decimal,
        curvature: Decimal,
        rounding: Decimal,
    }

    impl Fixed {
        fn new(w: Decimal, slope: Decimal, curvature: Decimal, rounding: Decimal) -> Self {
            Self {
                expiry: Positive::ONE,
                forward: Positive::HUNDRED,
                w,
                slope,
                curvature,
                rounding,
            }
        }

        /// A slice with `g(0) = 1 + curvature / 2` exactly, since `w' = 0`.
        fn with_g_at_zero(g: Decimal, rounding: Decimal) -> Self {
            Self::new(
                Decimal::ONE,
                Decimal::ZERO,
                (g - Decimal::ONE) * Decimal::TWO,
                rounding,
            )
        }
    }

    impl TotalVarianceSlice for Fixed {
        fn time_to_expiry(&self) -> Positive {
            self.expiry
        }
        fn forward(&self) -> Positive {
            self.forward
        }
        fn total_variance(&self, _k: Decimal) -> Result<Positive, StaticArbitrageError> {
            Ok(pos(self.w))
        }
        fn total_variance_slope(&self, _k: Decimal) -> Result<Decimal, StaticArbitrageError> {
            Ok(self.slope)
        }
        fn total_variance_curvature(&self, _k: Decimal) -> Result<Decimal, StaticArbitrageError> {
            Ok(self.curvature)
        }
        fn total_variance_rounding(&self, _k: Decimal) -> Result<Positive, StaticArbitrageError> {
            Ok(pos(self.rounding))
        }
    }

    #[test]
    fn test_density_factor_each_term_of_equation_2_1() {
        // k = 1, w = 0.5, w' = 0.2, w'' = 0.1:
        // (1 - 0.2)^2 - 0.01 * (2 + 0.25) + 0.05 = 0.64 - 0.0225 + 0.05.
        let factor =
            density_factor(dec!(1), dec!(0.5), dec!(0.2), dec!(0.1), Decimal::ZERO).unwrap();
        assert_eq!(factor.g, dec!(0.6675));
    }

    #[test]
    fn test_density_factor_matches_references_on_the_positive_side() {
        // Example 3.1 of the paper, against finite differences of `w` with 80
        // significant digits and a step of 1e-15, from a script sharing no code
        // with this crate. A report only ever shows the negative side.
        let parameters = SviParameters::try_new(
            dec!(-0.0410),
            dec!(0.1331),
            dec!(0.3060),
            dec!(0.3586),
            dec!(0.4153),
        )
        .unwrap();
        let slice = SviSlice::try_new(parameters, Positive::HUNDRED, Positive::ONE).unwrap();
        for (k, expected) in [
            (dec!(-1), dec!(0.2678703587364597144938115851)),
            (dec!(0), dec!(1.0386497312817974641928775294)),
            (dec!(0.5), dec!(0.0693792113008288652036289243)),
            (dec!(0.6), dec!(0.0146894171485162655184938540)),
            (dec!(1.5), dec!(0.0291238656581985846785593587)),
            (dec!(2), dec!(0.0769961673097418127281712935)),
        ] {
            let factor = density_factor(
                k,
                SviSlice::total_variance(&slice, k).unwrap().to_dec(),
                SviSlice::total_variance_slope(&slice, k).unwrap(),
                SviSlice::total_variance_curvature(&slice, k).unwrap(),
                TotalVarianceSlice::total_variance_rounding(&slice, k)
                    .unwrap()
                    .to_dec(),
            )
            .unwrap();
            let difference = (factor.g - expected).abs();
            assert!(
                difference < dec!(0.000000000000000000000001),
                "k = {k}: {difference}"
            );
            assert!(
                factor.band < dec!(0.0000000000000000000001),
                "k = {k}: {}",
                factor.band
            );
        }
    }

    #[test]
    fn test_density_factor_at_a_tiny_root_matches_the_reference() {
        // k - m = 4.72e-12 and sigma = 3.667e-13, where rounding the root at 28
        // places would cost w'' eleven of its digits. Reference from the same
        // script, with the analytic derivatives and 150 significant digits.
        let parameters = SviParameters::try_new(
            dec!(1.3699999999999999986223081),
            dec!(0.000003757),
            dec!(0),
            dec!(0.847),
            dec!(0.0000000000003667),
        )
        .unwrap();
        let slice = SviSlice::try_new(parameters, Positive::HUNDRED, Positive::ONE).unwrap();
        let k = dec!(0.84700000000472);
        let factor = density_factor(
            k,
            SviSlice::total_variance(&slice, k).unwrap().to_dec(),
            SviSlice::total_variance_slope(&slice, k).unwrap(),
            SviSlice::total_variance_curvature(&slice, k).unwrap(),
            TotalVarianceSlice::total_variance_rounding(&slice, k)
                .unwrap()
                .to_dec(),
        )
        .unwrap();
        let difference = (factor.g - dec!(2381.6022677796875342688299535)).abs();
        assert!(difference < factor.band, "{difference} vs {}", factor.band);
        assert!(difference < dec!(0.00000000000000000001), "{difference}");
    }

    #[test]
    fn test_density_factor_far_from_a_tiny_sigma_matches_the_reference() {
        // k - m = -1.04e-20 and sigma = 6.64e-26, where `sigma / R` keeps only 23
        // digits at 28 places and would put g outside its band. Same script.
        let parameters = SviParameters::try_new(
            dec!(0.684),
            dec!(0.000001652),
            dec!(0),
            dec!(-1.153),
            dec!(0.0000000000000000000000000664),
        )
        .unwrap();
        let slice = SviSlice::try_new(parameters, Positive::HUNDRED, Positive::ONE).unwrap();
        let k = dec!(-1.1530000000000000000104);
        let factor = density_factor(
            k,
            SviSlice::total_variance(&slice, k).unwrap().to_dec(),
            SviSlice::total_variance_slope(&slice, k).unwrap(),
            SviSlice::total_variance_curvature(&slice, k).unwrap(),
            TotalVarianceSlice::total_variance_rounding(&slice, k)
                .unwrap()
                .to_dec(),
        )
        .unwrap();
        let difference = (factor.g - dec!(3238.5477894615521609485374)).abs();
        assert!(difference < factor.band, "{difference} vs {}", factor.band);
    }

    /// `g` and its band, the way a check computes them.
    fn factor_of(slice: &SviSlice, k: Decimal) -> DensityFactor {
        density_factor(
            k,
            SviSlice::total_variance(slice, k).unwrap().to_dec(),
            SviSlice::total_variance_slope(slice, k).unwrap(),
            SviSlice::total_variance_curvature(slice, k).unwrap(),
            TotalVarianceSlice::total_variance_rounding(slice, k)
                .unwrap()
                .to_dec(),
        )
        .unwrap()
    }

    #[test]
    fn test_density_factor_large_b_is_not_a_false_violation() {
        // The true g is +9.2e-22, from the same script. Without the rounding of `w'`
        // in `w'^2 / 16`, and with `b` applied after the divisions of the curvature,
        // this read as a violation of 4.55e-21.
        let parameters = SviParameters::try_new(
            dec!(10198617),
            dec!(39781903),
            dec!(-0.999999),
            dec!(0),
            dec!(0.000011),
        )
        .unwrap();
        let slice = SviSlice::try_new(parameters, Positive::HUNDRED, Positive::ONE).unwrap();
        let k = dec!(0.030430593442154205258362455);
        let truth = dec!(0.0000000000000000000009199435);
        let factor = factor_of(&slice, k);
        let difference = (factor.g - truth).abs();
        assert!(difference <= factor.band, "{difference} vs {}", factor.band);
        assert!(
            check_butterfly(&slice, &[k])
                .unwrap()
                .violations()
                .is_empty()
        );
    }

    #[test]
    fn test_density_factor_large_b_stays_within_its_band_with_b_applied_first() {
        // With `b` applied after the two divisions of the curvature, g here is 7.5e-21
        // off against a band of 1.3e-21. Same script.
        let parameters = SviParameters::try_new(
            dec!(1066743530.278020518029584448),
            dec!(354400000),
            dec!(-0.999999),
            dec!(0),
            dec!(0.0009372),
        )
        .unwrap();
        let slice = SviSlice::try_new(parameters, Positive::HUNDRED, Positive::ONE).unwrap();
        let factor = factor_of(&slice, dec!(0.662699978302572854858704983));
        let difference = (factor.g - dec!(535.7811679546582669017279115)).abs();
        assert!(difference <= factor.band, "{difference} vs {}", factor.band);
    }

    #[test]
    fn test_butterfly_slope_at_the_last_place_is_not_lost_to_a_false_pass() {
        // Halving w' = 1e-28 rounds to zero, while w' / (2 * w) is 0.5 at w = 1e-28,
        // so k = 1 gives g = 0.25 - 0.5 = -0.25 and not 0.5.
        let last_place = Decimal::new(1, 28);
        let exact = Fixed::new(last_place, last_place, dec!(-1), dec!(0));
        let report = check_butterfly(&exact, &[Decimal::ONE]).unwrap();
        assert!(!report.passed(), "{report:?}");
        assert!(report.unevaluated().is_empty(), "{report:?}");
        match report.violations() {
            [violation] => assert_eq!(violation.size(), pos(dec!(0.25))),
            other => panic!("expected one violation, got {other:?}"),
        }

        // An odd last place at the same w, whose half would round to an even one.
        let factor = density_factor(
            Decimal::ONE,
            last_place,
            dec!(0.0000000000000000000000000003),
            dec!(0),
            dec!(0),
        )
        .unwrap();
        assert_eq!(factor.g, dec!(0.25) - dec!(0.0000000000000000000000000002));
    }

    #[test]
    fn test_density_factor_small_slope_is_not_lost_below_the_last_place() {
        // w' = 1e-15 squares to 1e-30, below the last place of a `Decimal`.
        // w'^2 / (4 * w) must still be 2.5e-17 at w = 1e-14, so g = 1 - 2.5e-17.
        let factor = density_factor(
            dec!(0),
            dec!(0.00000000000001),
            dec!(0.000000000000001),
            dec!(0),
            dec!(0),
        )
        .unwrap();
        assert_eq!(factor.g, dec!(0.999999999999999975));
    }

    #[test]
    fn test_butterfly_band_is_pinned_from_both_sides_without_rounding() {
        // At k = 0 with w = 1, w' = 0 and g near zero, the terms are 1 and about
        // 1, so the band is 2 * 1e-26.
        for (g, reported) in [
            (dec!(-0.00000000000000000000000001), false),
            (dec!(-0.00000000000000000000000002), false),
            (dec!(-0.00000000000000000000000003), true),
            (dec!(-0.0000000000000000000000001), true),
        ] {
            let report =
                check_butterfly(&Fixed::with_g_at_zero(g, Decimal::ZERO), &[dec!(0)]).unwrap();
            assert_eq!(report.violations().len(), usize::from(reported), "g = {g}");
        }
    }

    #[test]
    fn test_butterfly_band_grows_with_the_rounding_of_w() {
        // With a rounding of 1e-24 on w = 1, the band is 2 * (2e-24 + 1e-26).
        let rounding = dec!(0.000000000000000000000001);
        for (g, reported) in [
            (dec!(-0.000000000000000000000004), false),
            (dec!(-0.00000000000000000000000403), true),
        ] {
            let report = check_butterfly(&Fixed::with_g_at_zero(g, rounding), &[dec!(0)]).unwrap();
            assert_eq!(report.violations().len(), usize::from(reported), "g = {g}");
        }
    }

    #[test]
    fn test_butterfly_resolution_limit_is_pinned_from_both_sides() {
        // A rounding of 1e-24 is exactly 1e-12 of w = 1e-12, and is evaluated.
        // A rounding one unit above is not.
        let at = Fixed::new(
            dec!(0.000000000001),
            dec!(0),
            dec!(0),
            dec!(0.000000000000000000000001),
        );
        let report = check_butterfly(&at, &[dec!(0)]).unwrap();
        assert!(report.passed(), "{report:?}");

        let above = Fixed {
            rounding: dec!(0.0000000000000000000000010001),
            ..at
        };
        let report = check_butterfly(&above, &[dec!(0)]).unwrap();
        assert!(report.violations().is_empty(), "{report:?}");
        assert_eq!(report.unevaluated().len(), 1, "{report:?}");
        assert!(matches!(
            report.unevaluated(),
            [StaticArbitrageUnevaluatedPoint::Butterfly { rounding, .. }] if *rounding == pos(above.rounding)
        ));
    }

    #[test]
    fn test_butterfly_zero_variance_with_a_slope_is_unevaluated() {
        // Not reachable from a valid SVI slice, but a trait implementor can.
        let slice = Fixed::new(dec!(0), dec!(0.5), dec!(1), dec!(0));
        let report = check_butterfly(&slice, &[dec!(0), dec!(0.5)]).unwrap();
        assert!(report.violations().is_empty());
        assert_eq!(report.unevaluated().len(), 2);
        assert!(!report.passed());
    }

    #[test]
    fn test_calendar_resolution_limit_is_pinned_from_both_sides() {
        // At a larger w of 1, each rounding may reach 1e-12. The band is then 2e-12,
        // so a decrease of 3e-12 is reported and one of 1e-12 passes.
        let at = dec!(0.000000000001);
        let later = |rounding| Fixed {
            expiry: pos(dec!(2)),
            ..Fixed::new(dec!(1), dec!(0), dec!(0), rounding)
        };
        for (w, reported) in [(dec!(1.000000000001), false), (dec!(1.000000000003), true)] {
            let earlier = Fixed::new(w, dec!(0), dec!(0), at);
            let report = check_calendar(&[earlier, later(at)], &[dec!(0)]).unwrap();
            assert!(report.unevaluated().is_empty(), "w = {w}: {report:?}");
            assert_eq!(report.violations().len(), usize::from(reported), "w = {w}");
        }

        // Just above the limit, on either slice, the same comparison is unevaluated.
        let above = dec!(0.00000000000100001);
        for (earlier_rounding, later_rounding) in [(above, at), (at, above)] {
            let earlier = Fixed::new(dec!(1), dec!(0), dec!(0), earlier_rounding);
            let report = check_calendar(&[earlier, later(later_rounding)], &[dec!(0)]).unwrap();
            assert!(report.violations().is_empty(), "{report:?}");
            assert!(!report.passed());
            assert!(matches!(
                report.unevaluated(),
                [StaticArbitrageUnevaluatedPoint::Calendar {
                    earlier_rounding: found_earlier,
                    later_rounding: found_later,
                    ..
                }] if found_earlier.to_dec() == earlier_rounding && found_later.to_dec() == later_rounding
            ));
        }
    }

    #[test]
    fn test_calendar_large_declared_rounding_is_unevaluated_not_a_pass() {
        // These hid a decrease from 0.5 to 0.1, while the butterfly check marked the
        // same slices unevaluated. The largest one must not overflow in the sum.
        for rounding in [dec!(0.5), dec!(1000000), Decimal::MAX] {
            let earlier = Fixed {
                expiry: pos(dec!(0.5)),
                ..Fixed::new(dec!(0.5), dec!(0), dec!(0), rounding)
            };
            let later = Fixed::new(dec!(0.1), dec!(0), dec!(0), rounding);
            let report = check_calendar(&[earlier, later], &[dec!(0.25)]).unwrap();
            assert!(!report.passed(), "{report:?}");
            assert!(report.violations().is_empty(), "{report:?}");
            match report.unevaluated() {
                [point] => assert_eq!(point.log_moneyness(), dec!(0.25)),
                other => panic!("expected one unevaluated point, got {other:?}"),
            }

            let both = check_static_arbitrage(&[earlier, later], &[dec!(0)]).unwrap();
            assert_eq!(both.unevaluated().len(), 3, "{both:?}");
            assert!(matches!(
                both.unevaluated().last(),
                Some(StaticArbitrageUnevaluatedPoint::Calendar { .. })
            ));
        }
    }

    #[test]
    fn test_calendar_exact_zero_total_variances_are_resolved() {
        // Declaring no rounding at all, there is nothing to swamp, and zero against
        // zero is not a decrease.
        let earlier = Fixed::new(dec!(0), dec!(0), dec!(0), dec!(0));
        let later = Fixed {
            expiry: pos(dec!(2)),
            ..earlier
        };
        assert!(
            check_calendar(&[earlier, later], &[dec!(0)])
                .unwrap()
                .passed()
        );
    }

    #[test]
    fn test_calendar_zero_beside_a_large_total_variance_is_resolved() {
        // The rounding is small against the larger `w`, in either direction.
        let tiny = dec!(0.000000000000000000000000001);
        let zero = Fixed::new(dec!(0), dec!(0), dec!(0), tiny);
        let one = Fixed::new(dec!(1), dec!(0), dec!(0), tiny);
        let later = |slice| Fixed {
            expiry: pos(dec!(2)),
            ..slice
        };
        assert!(
            check_calendar(&[zero, later(one)], &[dec!(0)])
                .unwrap()
                .passed()
        );
        let report = check_calendar(&[one, later(zero)], &[dec!(0)]).unwrap();
        assert_eq!(report.violations().len(), 1, "{report:?}");
        assert!(report.unevaluated().is_empty(), "{report:?}");
    }

    #[test]
    fn test_calendar_band_is_pinned_from_both_sides_without_rounding() {
        let later = Fixed {
            expiry: pos(dec!(2)),
            ..Fixed::new(dec!(1), dec!(0), dec!(0), dec!(0))
        };
        let earlier = Fixed::new(dec!(1), dec!(0), dec!(0), dec!(0));

        let equal = check_calendar(&[earlier, later], &[dec!(0)]).unwrap();
        assert!(equal.passed(), "{equal:?}");

        let last_place = Fixed {
            w: dec!(1.0000000000000000000000000001),
            ..earlier
        };
        let report = check_calendar(&[last_place, later], &[dec!(0)]).unwrap();
        assert_eq!(report.violations().len(), 1, "{report:?}");
    }

    #[test]
    fn test_calendar_band_is_the_sum_of_both_roundings() {
        let earlier_rounding = dec!(0.000000000000000000000001);
        let later_rounding = dec!(0.000000000000000000000002);
        let earlier = |w| Fixed::new(w, dec!(0), dec!(0), earlier_rounding);
        let later = Fixed {
            expiry: pos(dec!(2)),
            ..Fixed::new(dec!(1), dec!(0), dec!(0), later_rounding)
        };
        for (w, reported) in [
            (dec!(1.000000000000000000000003), false),
            (dec!(1.000000000000000000000004), true),
        ] {
            let report = check_calendar(&[earlier(w), later], &[dec!(0)]).unwrap();
            assert_eq!(report.violations().len(), usize::from(reported), "w = {w}");
        }
    }

    #[test]
    fn test_check_slice_zero_forward_or_expiry_is_a_typed_error() {
        let no_forward = Fixed {
            forward: Positive::ZERO,
            ..Fixed::new(dec!(1), dec!(0), dec!(0), dec!(0))
        };
        assert!(matches!(
            check_butterfly(&no_forward, &[dec!(0)]),
            Err(StaticArbitrageError::InvalidSlice { reason, .. }) if reason.contains("forward")
        ));
        let no_expiry = Fixed {
            expiry: Positive::ZERO,
            ..Fixed::new(dec!(1), dec!(0), dec!(0), dec!(0))
        };
        let good = Fixed::new(dec!(1), dec!(0), dec!(0), dec!(0));
        assert!(matches!(
            check_calendar(&[good, no_expiry], &[dec!(0)]),
            Err(StaticArbitrageError::InvalidSlice { reason, .. }) if reason.contains("expiry")
        ));
    }

    #[test]
    fn test_strike_at_edges_of_the_range_are_typed_errors() {
        assert_eq!(
            strike_at(Positive::HUNDRED, dec!(0)).unwrap(),
            Positive::HUNDRED
        );
        for k in [dec!(1000), dec!(-1000), Decimal::MAX, Decimal::MIN] {
            assert!(matches!(
                strike_at(Positive::HUNDRED, k),
                Err(StaticArbitrageError::StrikeOutOfRange { .. })
            ));
        }
    }

    #[test]
    fn test_read_slice_reads_the_expiry_and_the_forward_once() {
        use std::cell::Cell;

        /// Counts how often the two are read.
        struct Counting<'a> {
            expiry: Positive,
            reads: &'a Cell<u32>,
        }
        impl TotalVarianceSlice for Counting<'_> {
            fn time_to_expiry(&self) -> Positive {
                self.reads.set(self.reads.get() + 1);
                self.expiry
            }
            fn forward(&self) -> Positive {
                self.reads.set(self.reads.get() + 1);
                Positive::HUNDRED
            }
            fn total_variance(&self, _k: Decimal) -> Result<Positive, StaticArbitrageError> {
                // Each later expiry has less: a violation at every point.
                Ok(pos(dec!(10) - self.expiry.to_dec()))
            }
            fn total_variance_slope(&self, _k: Decimal) -> Result<Decimal, StaticArbitrageError> {
                Ok(dec!(0))
            }
            fn total_variance_curvature(
                &self,
                _k: Decimal,
            ) -> Result<Decimal, StaticArbitrageError> {
                Ok(dec!(-4))
            }
            fn total_variance_rounding(
                &self,
                _k: Decimal,
            ) -> Result<Positive, StaticArbitrageError> {
                Ok(Positive::ZERO)
            }
        }

        let reads = Cell::new(0);
        let slices: Vec<Counting> = (1..=5)
            .rev()
            .map(|expiry| Counting {
                expiry: pos(Decimal::from(expiry)),
                reads: &reads,
            })
            .collect();
        let report = check_static_arbitrage(&slices, &[dec!(0), dec!(0.1)]).unwrap();
        // g = 1 - 2 = -1 on each of the 5 slices, and each of the 4 steps decreases.
        assert_eq!(report.violations().len(), 2 * 5 + 2 * 4);
        assert_eq!(reads.get(), 2 * 5);
    }

    #[test]
    fn test_references_to_slices_are_slices_themselves() {
        let earlier = Fixed::new(dec!(0.5), dec!(0), dec!(0), dec!(0));
        let later = Fixed {
            expiry: pos(dec!(2)),
            ..Fixed::new(dec!(0.1), dec!(0), dec!(0), dec!(0))
        };
        let by_value = check_calendar(&[earlier, later], &[dec!(0)]).unwrap();
        assert_eq!(by_value.violations().len(), 1);

        let by_reference = check_calendar(&[&earlier, &later], &[dec!(0)]).unwrap();
        assert_eq!(by_reference, by_value);
        let by_object: [&dyn TotalVarianceSlice; 2] = [&earlier, &later];
        assert_eq!(check_calendar(&by_object, &[dec!(0)]).unwrap(), by_value);
        assert_eq!(
            check_butterfly(&&earlier, &[dec!(0)]).unwrap(),
            check_butterfly(&earlier, &[dec!(0)]).unwrap()
        );
    }

    #[test]
    fn test_svi_slice_implements_the_trait_from_its_own_values() {
        let parameters =
            SviParameters::try_new(dec!(0.04), dec!(0.2), dec!(-0.4), dec!(0.05), dec!(0.3))
                .unwrap();
        let slice = SviSlice::try_new(parameters, pos(dec!(90)), pos(dec!(0.5))).unwrap();
        let k = dec!(0.1);
        let read: &dyn TotalVarianceSlice = &slice;
        assert_eq!(read.time_to_expiry(), pos(dec!(0.5)));
        assert_eq!(read.forward(), pos(dec!(90)));
        assert_eq!(
            read.total_variance(k).unwrap(),
            slice.total_variance(k).unwrap()
        );
        assert_eq!(
            read.total_variance_slope(k).unwrap(),
            slice.total_variance_slope(k).unwrap()
        );
        assert_eq!(
            read.total_variance_curvature(k).unwrap(),
            slice.total_variance_curvature(k).unwrap()
        );
        assert!(read.total_variance_rounding(k).unwrap() < pos(dec!(0.000000000000000000000001)));
    }
}
