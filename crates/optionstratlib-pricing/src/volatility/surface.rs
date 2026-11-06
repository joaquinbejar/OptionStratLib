/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 6/11/26
******************************************************************************/

//! A calibrated implied volatility surface, built from option quotes as of one
//! instant: see [`ImpliedVolatilitySurface`], whose documentation states every rule.

use super::arbitrage::{
    StaticArbitrageReport, TotalVarianceSlice, check_butterfly, check_static_arbitrage,
};
use super::svi::{
    SviCalibrationConfig, SviObservation, SviParameters, SviSlice, calibrate_svi, log_moneyness,
};
use crate::error::surface::{
    OptionQuote, QuoteRejection, QuoteRejectionReason, UnfittedExpiry, UnfittedReason,
};
use crate::error::{ImpliedVolatilitySurfaceError, StaticArbitrageError, SviError};
use crate::pricing::black_scholes_model::{black_scholes, european_price_band};
use crate::pricing::constants::MAX_ITERATIONS_IV;
use crate::pricing::iv_solver::{
    IV_BRACKET_MAX, IvOutcome, iv_initial_guess, long_residual, solve_implied_volatility,
    vega_slope,
};
use chrono::{DateTime, TimeDelta, Utc};
use optionstratlib_core::constants::MIN_VOLATILITY;
use optionstratlib_core::error::DecimalError;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::{d_add, d_div, d_exp, d_mul, d_sqrt, d_sub};
use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
use optionstratlib_core::model::{ExpirationDate, Options};
use optionstratlib_core::utils::time::years_to_expiration;
use optionstratlib_math::surfaces::{Point3D, Surface};
use rust_decimal::Decimal;
use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;
use tracing::{debug, instrument, warn};

/// Seconds in a day, for the days between two instants.
const SECONDS_PER_DAY: Decimal = Decimal::from_parts(86_400, 0, 0, false, 0);

/// Default [`ImpliedVolatilitySurfaceConfig::max_quote_age`], in seconds: five minutes.
const DEFAULT_MAX_QUOTE_AGE_SECONDS: i64 = 300;

/// Default [`ImpliedVolatilitySurfaceConfig::arbitrage_grid_points`].
const DEFAULT_ARBITRAGE_GRID_POINTS: NonZeroUsize = match NonZeroUsize::new(41) {
    Some(points) => points,
    None => NonZeroUsize::MIN,
};

/// The largest [`ImpliedVolatilitySurfaceConfig::arbitrage_grid_points`], per expiry: the
/// grid is allocated and every point costs a check on every slice.
const MAX_ARBITRAGE_GRID_POINTS: usize = 100_000;

/// The smallest [`ImpliedVolatilitySurfaceConfig::arbitrage_grid_points`], per expiry: as
/// many points as a raw SVI slice has parameters. One or two points cannot see a slice at
/// all. Five is a floor against that and no assurance: the check still covers only its
/// points.
const MIN_ARBITRAGE_GRID_POINTS: usize = 5;

/// The resolution of a time to expiry, relative to it: `Decimal::from_f64` keeps 15 to 16
/// significant digits, so the `T` of a slice is within about `6e-16` of itself as computed in
/// `Decimal`. A `T` this close to a fitted expiry reads that expiry.
const TIME_TO_EXPIRY_RESOLUTION: Decimal = Decimal::from_parts(1, 0, 0, false, 15);

/// The `theta` of the slice in the middle of a gap between two expiries: one half.
const MIDDLE_OF_GAP: Decimal = Decimal::from_parts(5, 0, 0, false, 1);

/// The spread a locked quote weighs as, relative to its mid: 0.1%. A tighter spread
/// than that weighs as this, so one quote cannot take the whole fit.
const WEIGHT_SPREAD_FLOOR: Decimal = Decimal::from_parts(1, 0, 0, false, 3);

/// Rounding of the blend of two slices, relative to the size of their total variances:
/// about `1e4` units in the last place of a `Decimal`, as the SVI band itself allows.
const BLEND_ROUNDING_BAND: Decimal = Decimal::from_parts(1, 0, 0, false, 24);

/// The same as an absolute floor, for slices so small the 28 places are what remains.
const BLEND_ROUNDING_FLOOR: Decimal = Decimal::from_parts(4, 0, 0, false, 28);

/// How far a [`SurfacePoint`] may be read outside the data before it is an error.
///
/// Within these distances the point comes back with a flag. See
/// [`ImpliedVolatilitySurface`] for the formulas read there. The default, zero on all
/// three, makes every read outside the data an error.
///
/// **A wider limit is not a safer one.** Beyond its quotes a slice is only its raw SVI formula,
/// and on few quotes its wings can explode with nothing to contain them. Measured: a fit of five
/// quotes spanning `k` from -0.1 to 0.1, within the limit on its RMSE, with noise of one
/// volatility point, reads 26% at `k = -0.1`, 36% at `-0.15` and 896% at `-0.2`. The flag,
/// [`SurfacePoint::is_extrapolated`], is the only signal: nothing bounds the value read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ExtrapolationPolicy {
    /// How far beyond the log moneyness of the quotes a slice may be read, in log
    /// moneyness.
    pub max_wing_extension: Positive,
    /// How far before the first fitted expiry a read may be, in years.
    pub max_short_extension: Positive,
    /// How far after the last fitted expiry a read may be, in years.
    pub max_long_extension: Positive,
}

/// The rules of [`ImpliedVolatilitySurface::calibrate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImpliedVolatilitySurfaceConfig {
    /// The oldest a quote may be at the as-of time. Exactly this old is still clean.
    /// Must not be negative.
    pub max_quote_age: TimeDelta,
    /// The limits of the fit of each expiry, among them
    /// [`SviCalibrationConfig::max_relative_rmse`], above which an expiry is unfitted.
    pub svi: SviCalibrationConfig,
    /// How far a read may leave the data.
    pub extrapolation: ExtrapolationPolicy,
    /// The evenly spaced points of the log moneyness grid of the arbitrage report **per
    /// expiry**, across the quotes of that expiry. The grid is the sorted union of them and of
    /// the log moneyness of every clean quote. At least 5 and at most 100000.
    ///
    /// The limit is per expiry, and there is no limit on the total. The grid grows with the
    /// expiries, and every slice is read at every point, with a slice for each expiry and each
    /// gap between two, so the cost of the check is roughly quadratic in the expiries. Measured
    /// in a release build, the whole calibration: 100000 points took about 0.9 s for one expiry,
    /// and the default of 41 about 1.8 s for 20 expiries of 61 quotes each.
    pub arbitrage_grid_points: NonZeroUsize,
}

impl Default for ImpliedVolatilitySurfaceConfig {
    /// Quotes up to five minutes old, the defaults of [`SviCalibrationConfig`], no
    /// extrapolation, and a grid of 41 points per expiry.
    fn default() -> Self {
        // The constant fits, so the fallback is not taken. Were it taken, zero would
        // mark every quote older than the as-of time stale: the failure stays loud.
        Self {
            max_quote_age: TimeDelta::try_seconds(DEFAULT_MAX_QUOTE_AGE_SECONDS)
                .unwrap_or(TimeDelta::zero()),
            svi: SviCalibrationConfig::default(),
            extrapolation: ExtrapolationPolicy::default(),
            arbitrage_grid_points: DEFAULT_ARBITRAGE_GRID_POINTS,
        }
    }
}

/// How a slice sits against one quote, by strike and expiry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct SurfaceResidual {
    /// The expiry of the quote.
    pub expiry: DateTime<Utc>,
    /// Its time to expiry, in years.
    pub time_to_expiry: Positive,
    /// Its strike.
    pub strike: Positive,
    /// Its style.
    pub style: OptionStyle,
    /// `ln(strike / forward)`.
    pub log_moneyness: Decimal,
    /// The implied volatility of its mid.
    pub observed_implied_volatility: Positive,
    /// The implied volatility of the slice at this strike.
    pub fitted_implied_volatility: Positive,
    /// Fitted minus observed implied volatility.
    pub residual: Decimal,
    /// Its weight in the fit, at most one. See [`ImpliedVolatilitySurface`].
    pub weight: Positive,
    /// Its bid.
    pub bid: Positive,
    /// Its ask.
    pub ask: Positive,
    /// The price of this option at the fitted implied volatility, per unit of underlying.
    pub fitted_price: Decimal,
    /// Whether [`SurfaceResidual::fitted_price`] is below the bid or above the ask, with no
    /// tolerance, so a locked quote is outside unless the fit prices it exactly. This
    /// ignores the weight and gates nothing.
    pub outside_bid_ask: bool,
}

/// One expiry against a flat volatility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct BaselineComparison {
    /// The expiry.
    pub expiry: DateTime<Utc>,
    /// Its time to expiry, in years.
    pub time_to_expiry: Positive,
    /// The flat volatility: `sqrt(sum(weight * iv^2) / sum(weight))`, which is the best
    /// flat fit of the objective of the slice, a weighted least squares in total variance.
    pub baseline_implied_volatility: Positive,
    /// Its weighted RMSE against the quotes, in implied volatility.
    pub baseline_rmse: Positive,
    /// The weighted RMSE of the slice against the same quotes, with the same weights.
    pub surface_rmse: Positive,
}

impl BaselineComparison {
    /// `baseline_rmse - surface_rmse`, in implied volatility: positive when the slice
    /// is closer to the quotes. Both are in sample. See [`ImpliedVolatilitySurface`].
    #[must_use]
    #[inline]
    pub fn improvement(&self) -> Decimal {
        // Both fit in a `Positive`, so their difference fits in a `Decimal`. A failure
        // would read as no improvement, the side that claims nothing.
        d_sub(
            self.baseline_rmse.to_dec(),
            self.surface_rmse.to_dec(),
            "surface::baseline::improvement",
        )
        .unwrap_or(Decimal::ZERO)
    }
}

/// One fitted expiry of a surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceExpiry {
    expiry: DateTime<Utc>,
    slice: SviSlice,
    residuals: Vec<SurfaceResidual>,
    rmse: Positive,
    relative_rmse: Positive,
    unweighted_rmse: Positive,
    quotes_outside_bid_ask: usize,
    lowest_log_moneyness: Decimal,
    highest_log_moneyness: Decimal,
    baseline: BaselineComparison,
}

impl SurfaceExpiry {
    /// Builds an expiry from its slice and its quotes, computing every figure from them.
    fn from_slice(
        expiry: DateTime<Utc>,
        slice: SviSlice,
        quotes: &[QuoteInFit],
        market: &Market,
    ) -> Result<Self, ImpliedVolatilitySurfaceError> {
        let days = time_to_expiry(market.as_of, expiry)?
            .ok_or(ImpliedVolatilitySurfaceError::ZeroTimeToExpiry)?
            .days;
        let time_to_expiry = slice.time_to_expiry();
        let mut residuals = Vec::with_capacity(quotes.len());
        for quote in quotes {
            let k = slice.log_moneyness(quote.strike)?;
            let fitted = slice.implied_volatility(k)?;
            let fitted_price = black_scholes(&european_option(
                quote.strike,
                quote.style,
                days,
                fitted,
                market,
            ))?;
            residuals.push(SurfaceResidual {
                expiry,
                time_to_expiry,
                strike: quote.strike,
                style: quote.style,
                log_moneyness: k,
                observed_implied_volatility: quote.volatility,
                fitted_implied_volatility: fitted,
                residual: d_sub(
                    fitted.to_dec(),
                    quote.volatility.to_dec(),
                    "surface::residual",
                )?,
                weight: quote.weight,
                bid: quote.bid,
                ask: quote.ask,
                fitted_price,
                outside_bid_ask: fitted_price < quote.bid.to_dec()
                    || fitted_price > quote.ask.to_dec(),
            });
        }
        let summary = FitSummary::of(&residuals)?;
        Ok(Self {
            expiry,
            slice,
            lowest_log_moneyness: summary.lowest,
            highest_log_moneyness: summary.highest,
            rmse: summary.rmse,
            relative_rmse: summary.relative_rmse,
            unweighted_rmse: summary.unweighted_rmse,
            quotes_outside_bid_ask: residuals
                .iter()
                .filter(|residual| residual.outside_bid_ask)
                .count(),
            baseline: BaselineComparison {
                expiry,
                time_to_expiry,
                baseline_implied_volatility: summary.baseline,
                baseline_rmse: summary.baseline_rmse,
                surface_rmse: summary.rmse,
            },
            residuals,
        })
    }

    /// The expiry.
    #[must_use]
    #[inline]
    pub fn expiry(&self) -> DateTime<Utc> {
        self.expiry
    }

    /// Its time to expiry, in years, Actual/365 Fixed.
    #[must_use]
    #[inline]
    pub fn time_to_expiry(&self) -> Positive {
        self.slice.time_to_expiry()
    }

    /// Its forward: `S e^((r - q) T)`.
    #[must_use]
    #[inline]
    pub fn forward(&self) -> Positive {
        self.slice.forward()
    }

    /// Its raw SVI slice. It is not claimed free of arbitrage.
    #[must_use]
    #[inline]
    pub fn slice(&self) -> &SviSlice {
        &self.slice
    }

    /// One residual per clean quote, by strike and then calls before puts.
    #[must_use]
    #[inline]
    pub fn residuals(&self) -> &[SurfaceResidual] {
        &self.residuals
    }

    /// The weighted RMSE of the slice against its quotes, in implied volatility.
    #[must_use]
    #[inline]
    pub fn rmse(&self) -> Positive {
        self.rmse
    }

    /// [`SurfaceExpiry::rmse`] over the weighted mean observed implied volatility.
    #[must_use]
    #[inline]
    pub fn relative_rmse(&self) -> Positive {
        self.relative_rmse
    }

    /// The RMSE of the slice against all its quotes with equal weights, in implied volatility:
    /// `sqrt(sum(residual^2) / n)`.
    ///
    /// [`SurfaceExpiry::rmse`] and the limit on it use the weights of the fit, so quotes with
    /// wide spreads hardly count in them. This does not, and it gates nothing: neither the fit
    /// nor the arbitrage report reads it. See [`ImpliedVolatilitySurface`].
    #[must_use]
    #[inline]
    pub fn unweighted_rmse(&self) -> Positive {
        self.unweighted_rmse
    }

    /// How many of its quotes the slice prices outside their bid and ask, whatever their
    /// weight. See [`SurfaceResidual::outside_bid_ask`]. It gates nothing: neither the fit nor
    /// the arbitrage report reads it.
    #[must_use]
    #[inline]
    pub fn quotes_outside_bid_ask(&self) -> usize {
        self.quotes_outside_bid_ask
    }

    /// The lowest log moneyness of a quote with a positive weight.
    #[must_use]
    #[inline]
    pub fn lowest_log_moneyness(&self) -> Decimal {
        self.lowest_log_moneyness
    }

    /// The highest log moneyness of a quote with a positive weight.
    #[must_use]
    #[inline]
    pub fn highest_log_moneyness(&self) -> Decimal {
        self.highest_log_moneyness
    }

    /// The comparison of this expiry with a flat volatility.
    #[must_use]
    #[inline]
    pub fn baseline(&self) -> &BaselineComparison {
        &self.baseline
    }

    /// How far `k` is beyond the quotes of this expiry, in log moneyness.
    fn wing_extension(&self, k: Decimal) -> Result<Decimal, DecimalError> {
        let below = d_sub(self.lowest_log_moneyness, k, "surface::wing::below")?;
        let above = d_sub(k, self.highest_log_moneyness, "surface::wing::above")?;
        Ok(below.max(above).max(Decimal::ZERO))
    }

    /// The quotes of this expiry, to recompute its figures against a shocked slice.
    fn quotes_in_fit(&self) -> Vec<QuoteInFit> {
        self.residuals
            .iter()
            .map(|residual| QuoteInFit {
                strike: residual.strike,
                style: residual.style,
                volatility: residual.observed_implied_volatility,
                weight: residual.weight,
                bid: residual.bid,
                ask: residual.ask,
            })
            .collect()
    }
}

/// What an expiry says about its quotes.
struct FitSummary {
    rmse: Positive,
    relative_rmse: Positive,
    unweighted_rmse: Positive,
    lowest: Decimal,
    highest: Decimal,
    baseline: Positive,
    baseline_rmse: Positive,
}

impl FitSummary {
    fn of(residuals: &[SurfaceResidual]) -> Result<Self, ImpliedVolatilitySurfaceError> {
        const OP: &str = "surface::summary";
        let mut weight_sum = Decimal::ZERO;
        let mut squares = Decimal::ZERO;
        let mut volatilities = Decimal::ZERO;
        let mut variances = Decimal::ZERO;
        let mut lowest = None;
        let mut highest = None;
        for residual in residuals {
            let weight = residual.weight.to_dec();
            if weight.is_zero() {
                continue;
            }
            let observed = residual.observed_implied_volatility.to_dec();
            weight_sum = d_add(weight_sum, weight, OP)?;
            squares = d_add(
                squares,
                d_mul(weight, d_mul(residual.residual, residual.residual, OP)?, OP)?,
                OP,
            )?;
            volatilities = d_add(volatilities, d_mul(weight, observed, OP)?, OP)?;
            variances = d_add(
                variances,
                d_mul(weight, d_mul(observed, observed, OP)?, OP)?,
                OP,
            )?;
            let k = residual.log_moneyness;
            lowest = Some(lowest.map_or(k, |value: Decimal| value.min(k)));
            highest = Some(highest.map_or(k, |value: Decimal| value.max(k)));
        }
        let rmse = d_sqrt(d_div(squares, weight_sum, OP)?, OP)?;
        let baseline = d_sqrt(d_div(variances, weight_sum, OP)?, OP)?;
        let mut baseline_squares = Decimal::ZERO;
        let mut unweighted_squares = Decimal::ZERO;
        for residual in residuals {
            unweighted_squares = d_add(
                unweighted_squares,
                d_mul(residual.residual, residual.residual, OP)?,
                OP,
            )?;
            let miss = d_sub(baseline, residual.observed_implied_volatility.to_dec(), OP)?;
            baseline_squares = d_add(
                baseline_squares,
                d_mul(residual.weight.to_dec(), d_mul(miss, miss, OP)?, OP)?,
                OP,
            )?;
        }
        let mean = d_div(volatilities, weight_sum, OP)?;
        Ok(Self {
            rmse: Positive::new_decimal(rmse)?,
            relative_rmse: Positive::new_decimal(d_div(rmse, mean, OP)?)?,
            unweighted_rmse: Positive::new_decimal(d_sqrt(
                d_div(unweighted_squares, Decimal::from(residuals.len()), OP)?,
                OP,
            )?)?,
            lowest: lowest.unwrap_or(Decimal::ZERO),
            highest: highest.unwrap_or(Decimal::ZERO),
            baseline: Positive::new_decimal(baseline)?,
            baseline_rmse: Positive::new_decimal(d_sqrt(
                d_div(baseline_squares, weight_sum, OP)?,
                OP,
            )?)?,
        })
    }
}

/// Where a [`SurfacePoint`] lies against the fitted expiries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[non_exhaustive]
pub enum SurfaceExpiryRegion {
    /// At a fitted expiry or between two of them.
    WithinExpiries,
    /// Before the first fitted expiry.
    BeforeFirstExpiry,
    /// After the last fitted expiry.
    AfterLastExpiry,
}

/// One value read from a surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct SurfacePoint {
    /// The log moneyness, `ln(K / F(T))`.
    pub log_moneyness: Decimal,
    /// The time to expiry, in years. At a fitted expiry, the `T` of its slice.
    pub time_to_expiry: Positive,
    /// The forward at that expiry: `S e^((r - q) T)`.
    pub forward: Positive,
    /// The total variance, annualized variance times years.
    pub total_variance: Positive,
    /// The annualized implied volatility, as a fraction per year.
    pub implied_volatility: Positive,
    /// How far beyond the quotes of the expiries read this point is, in log moneyness:
    /// zero inside them.
    pub wing_extension: Positive,
    /// Where it lies against the fitted expiries.
    pub expiry_region: SurfaceExpiryRegion,
    /// An expiry without a slice lies between the two this point was read between, so
    /// the data there was not used.
    pub bridges_unfitted_expiry: bool,
}

impl SurfacePoint {
    /// Whether this point was read outside the data, as the policy allows.
    #[must_use]
    #[inline]
    pub fn is_extrapolated(&self) -> bool {
        !self.wing_extension.is_zero() || self.expiry_region != SurfaceExpiryRegion::WithinExpiries
    }
}

/// A scenario applied to every expiry of a surface, by
/// [`ImpliedVolatilitySurface::shock`].
///
/// Every shock changes the raw SVI parameters of each slice and leaves its forward and
/// expiry. Parameters that are not a valid slice are a typed error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SurfaceShock {
    /// Moves the at-the-money implied volatility, at `k = 0`, of every expiry by the same
    /// amount `x`, with
    ///
    /// ```text
    /// a' = a + x * (2 * iv_atm + x) * T
    /// ```
    ///
    /// which gives `iv'(k)^2 = iv(k)^2 + x * (2 * iv_atm + x)`. Away from `k = 0` the move is
    /// about `x * iv_atm / iv(k)`, so not parallel in volatility. `b`, `rho`, `m` and `sigma`
    /// do not change.
    Level {
        /// `x`, in implied volatility (`0.01` is one volatility point).
        volatility: Decimal,
    },
    /// Tilts the smile of every expiry by the same amount `x`, with
    ///
    /// ```text
    /// rho' = rho + x
    /// ```
    ///
    /// `a`, `b`, `m` and `sigma` do not change. So `w'(k) = w(k) + b * x * (k - m)`, and
    /// at-the-money total variance moves by `-b * x * m`, which is zero only for `m = 0`.
    Skew {
        /// `x`, added to `rho`.
        rho: Decimal,
    },
    /// Moves the at-the-money implied volatility of each expiry by an amount linear in its
    /// time to expiry, with the formula of [`SurfaceShock::Level`] at
    ///
    /// ```text
    /// x(T) = slope * (T - pivot)
    /// ```
    Term {
        /// The slope, in implied volatility per year.
        slope: Decimal,
        /// The expiry with no move, in years.
        pivot: Positive,
    },
}

/// A surface evaluated on a grid, by [`ImpliedVolatilitySurface::export_surface`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SurfaceExport {
    /// `x` is log moneyness, `y` is time to expiry in years, `z` is annualized implied
    /// volatility.
    pub surface: Surface,
    /// The points of the grid read outside the data, as the policy allows. A `Surface`
    /// cannot carry this flag itself.
    pub extrapolated: Vec<SurfacePoint>,
    /// The points of the grid read across an expiry without a slice, so the data there was
    /// not used. A `Surface` cannot carry this flag either. A point can be in both lists.
    pub bridging_unfitted: Vec<SurfacePoint>,
}

/// A calibrated implied volatility surface: one raw SVI slice per fitted expiry, the
/// quotes that did not enter it and why, and a report of its arbitrage checks.
///
/// [`ImpliedVolatilitySurface::calibrate`] takes bid and ask quotes across strikes and
/// expiries, the spot, a rate, a dividend yield and an as-of time. It cleans the quotes,
/// solves each for its implied volatility, fits one raw SVI slice per expiry, and checks
/// the result for butterfly and calendar arbitrage on a grid. It reads no clock of any
/// kind, so it gives the same answer in a build without the `clock` feature, and with any
/// reference datetime set.
///
/// # Time to expiry
///
/// `T` is `expiry - as_of` on Actual/365 Fixed: nanoseconds between the two
/// instants, as `Decimal` days, over 365. The solvers receive those days as an
/// `ExpirationDate::Days`, and the slices the same `T` they use, so sub-day
/// expiries keep their hours and seconds: an `ExpirationDate::Days` goes to years
/// by a division and is never rounded to whole days. That division runs in `f64`
/// and comes back through `Decimal::from_f64`, which keeps about 15 significant
/// digits, so `T` is resolved to about `1e-15` of itself: about 30 nanoseconds at
/// one year, and well below a nanosecond at one day. Two expiries closer than that have
/// one `T` and no order, and the check reports them as sharing it. A calendar day is not
/// a trading day: nothing here knows about weekends or sessions.
///
/// # Cleaning
///
/// Every quote is clean or rejected with a reason, and nothing is smoothed: a clean
/// quote enters at its own mid. A rejected quote is kept in
/// [`ImpliedVolatilitySurface::rejections`], with the first rule of this list it
/// broke as its [`QuoteRejectionReason`]:
///
/// 1. Later than the as-of time. A quote like this is seen by no later step, so it
///    cannot change the surface in any way but by being listed here.
/// 2. Its expiry is not after the as-of time.
/// 3. A zero strike.
/// 4. Crossed: the bid above the ask. A locked quote, bid equal to ask, is clean.
/// 5. A zero bid: its mid is half of an ask, with no market under it.
/// 6. Stale: older than [`ImpliedVolatilitySurfaceConfig::max_quote_age`].
/// 7. Superseded: another quote of the same expiry, strike and style that passed
///    the rules above is later, or as late and tighter, or as tight with a lower
///    bid. So the result does not depend on the order of the input. Only a quote that
///    passed them can supersede: a newer quote that is crossed, for one, is rejected and
///    leaves an older clean quote of that option in the fit, up to the age limit.
/// 8. A mid outside the no-arbitrage band of a European option:
///    `max(S e^(-qT) - K e^(-rT), 0) <= C <= S e^(-qT)` for a call and
///    `max(K e^(-rT) - S e^(-qT), 0) <= P <= K e^(-rT)` for a put.
/// 9. No implied volatility: the mid prices at or below the volatility floor, above
///    a volatility of 500%, or the solve did not converge.
/// 10. No weight: its vega is zero, or the weight below has no value in a `Decimal`.
///
/// Every quote is priced as a European option under Black-Scholes, whichever way it
/// really exercises. For an American option that bias is in the implied volatility,
/// and nothing here removes it.
///
/// # Weights
///
/// A residual is measured against the width of the quote in volatility,
/// `spread / vega`, where vega is per unit of volatility. The kernel of
/// [`calibrate_svi`] minimizes squares in total variance, where
/// `dw = 2 * iv * T * d(iv)`, so within one expiry, where `T` is common, the weight
/// is
///
/// ```text
/// weight = (vega / (iv * max(spread, 0.001 * mid)))^2 / (the largest of those squares in the expiry)
/// ```
///
/// The floor keeps a locked quote finite, as if it were 0.1% of its mid wide. A
/// weight that rounds to zero, below about `1e-28` of the largest, keeps its residual
/// but does not enter the fit. These weights also weigh the RMSE that
/// [`SviCalibrationConfig::max_relative_rmse`] limits, so a quote with little vega or a wide
/// spread can sit far from the slice within the limit.
///
/// [`SurfaceExpiry::quotes_outside_bid_ask`] and [`SurfaceExpiry::unweighted_rmse`] ignore the
/// weights, and show that. They are figures to read: neither gates the fit, and neither
/// enters the arbitrage report.
///
/// # An expiry that cannot be fitted
///
/// An expiry with a quote at or before the as-of time either has a slice or is an
/// [`UnfittedExpiry`] with its [`UnfittedReason`]: expired at the as-of time, too few clean
/// quotes, too few distinct strikes, a best fit above the limit, or a failed calibration. It never
/// disappears, and the rest of the surface is still built. A
/// [`SurfacePoint`] read between two expiries with an unfitted one between them says
/// so in [`SurfacePoint::bridges_unfitted_expiry`]. Only a surface with no slice at
/// all is an error, [`ImpliedVolatilitySurfaceError::NoFittedExpiry`], and it carries
/// both lists.
///
/// [`calibrate_svi`] has no flat fallback. A raw SVI slice is convex in `k`, so it cannot
/// follow a frown, and with even spreads such an expiry comes back unfitted. **That is not a
/// guarantee**: the limit is on a weighted RMSE. Measured: nine quotes of a frown from 50% down
/// to 30%, at the money 0.1% wide and the rest 60% wide, fit at a relative RMSE of 0.0046 with
/// residuals up to 21 volatility points and 5 of the 9 fitted prices outside their bid and ask.
/// That expiry has a slice and the report can pass. [`SurfaceExpiry::quotes_outside_bid_ask`]
/// and [`SurfaceExpiry::unweighted_rmse`] are what show it.
///
/// [`SviCalibrationConfig::max_relative_rmse`], through
/// [`ImpliedVolatilitySurfaceConfig::svi`], is what the caller tightens or loosens. It is
/// relative to the level of volatility, so the noise it admits scales with that level. On the
/// default of 3%, measured on five expiries of 15 quotes each: at a volatility of 20%, noise of
/// 0.5 volatility points (2.5% of the level) fits all of them, and at 0.8 points (4%) three are
/// lost; at 60%, 0.8 points (1.3%) fits all of them.
///
/// # Interpolation
///
/// Between two fitted expiries `T1 < T < T2`, total variance is linear in `T` at
/// fixed log moneyness, `k = ln(K / F(T))`, with `F(T) = S e^((r - q) T)`:
///
/// ```text
/// w(k, T) = (1 - theta) * w1(k) + theta * w2(k),  theta = (T - T1) / (T2 - T1)
/// ```
///
/// This keeps the calendar condition between `T1` and `T2` at each `k` where the two
/// slices meet it. Nothing proves it keeps the butterfly condition, which is why the
/// check below reads a slice between each two expiries too.
///
/// # Extrapolation
///
/// [`ExtrapolationPolicy`], in the configuration, declares how far a value may be read
/// outside the data. Within it a [`SurfacePoint`] comes back with an explicit flag,
/// [`SurfacePoint::is_extrapolated`]. Beyond it a read is a typed error. The default
/// allows nothing, so every read outside the data is an error until the caller says
/// otherwise.
///
/// - Wings. A slice is read beyond the log moneyness of its quotes in the fit by
///   continuing its raw SVI formula. Between two expiries `k` must be within the quotes
///   of both for the point to be inside the data.
/// - Short and long expiries. Before the first expiry and after the last, implied
///   volatility at fixed `k` is that of the nearest slice: `w(k, T) = w_ref(k) * T / T_ref`.
///   This keeps total variance increasing in `T`. It is a declared convention, not an
///   estimate: short-dated variance need not behave like this, and any event between
///   today and the first expiry is not in it.
///
/// # What the arbitrage report means
///
/// [`check_static_arbitrage`] runs on the slices of the fitted expiries and on a slice
/// at the middle of each gap between them, all on one log moneyness grid: the sorted
/// union of [`ImpliedVolatilitySurfaceConfig::arbitrage_grid_points`] points across the
/// quotes of each expiry and the log moneyness of every clean quote. So a short expiry with
/// narrow quotes has that many points inside them, and dense quotes keep their own points when
/// a far quote stretches the spacing. A surface with one expiry has no calendar check, and
/// [`ImpliedVolatilitySurface::calendar_checked`] says so.
///
/// [`ImpliedVolatilitySurface::arbitrage_report`] is a finding, not a verdict:
///
/// - A surface with violations is still returned, and so is one with points the check
///   could not resolve, which are `StaticArbitrageReport::unevaluated`. Neither passes.
/// - `StaticArbitrageReport::passed` means only that these two checks found nothing at
///   those grid points, on those slices. **This surface is not claimed free of
///   arbitrage.** Between grid points, at other expiries and in the far wings nothing is
///   known. [`ImpliedVolatilitySurface::check_on_grid`] repeats the check on a grid of
///   the caller's.
/// - The grid is one for all the slices, so it also reads a short expiry far beyond its
///   quotes where a long expiry has quotes. A finding names its `k` and expiry, which is
///   how to tell.
/// - The check also aborts on a `Decimal` overflow. A surface whose check cannot finish
///   is an error, [`ImpliedVolatilitySurfaceError::StaticArbitrage`], since it has no
///   report to carry.
///
/// # Baseline
///
/// Each expiry is compared with one flat volatility, the weighted least squares of the
/// same objective, in [`BaselineComparison`]. Both figures are in sample, and a slice
/// has five parameters against one, so a slice is expected to win by some margin without
/// that saying how it predicts.
///
/// # Tolerances
///
/// On noiseless quotes of a known SSVI surface with a rate and a dividend yield, the
/// tests recover the implied volatility of every slice within `1e-8` at its quotes and
/// its raw parameters within `1e-6`, and its report has nothing to say. Real quotes are
/// not an exact SVI: the residuals say how far they are.
///
/// # Example
///
/// ```rust
/// use chrono::{TimeDelta, TimeZone, Utc};
/// use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
/// use optionstratlib_core::model::{ExpirationDate, Options, Positive};
/// use optionstratlib_pricing::pricing::black_scholes_model::black_scholes;
/// use optionstratlib_pricing::volatility::{
///     ImpliedVolatilitySurface, ImpliedVolatilitySurfaceConfig, OptionQuote, SviParameters,
///     SviSlice,
/// };
/// use rust_decimal::Decimal;
/// use rust_decimal_macros::dec;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let as_of = Utc.with_ymd_and_hms(2030, 1, 1, 12, 0, 0).single().ok_or("date")?;
/// let spot = Positive::HUNDRED;
/// let parameters =
///     SviParameters::try_new(dec!(0.02), dec!(0.1), dec!(-0.4), dec!(0.05), dec!(0.2))?;
/// // A slice with no rate or dividend, so the forward at each expiry is the spot.
/// let known = SviSlice::try_new(parameters, spot, Positive::ONE)?;
///
/// let mut quotes = Vec::new();
/// for strike in [70, 80, 90, 100, 110, 120, 135] {
///     let strike = Positive::new_decimal(Decimal::from(strike))?;
///     let option = Options::new(
///         OptionType::European,
///         Side::Long,
///         String::new(),
///         strike,
///         ExpirationDate::Days(Positive::new_decimal(dec!(365))?),
///         known.implied_volatility_at_strike(strike)?,
///         Positive::ONE,
///         spot,
///         Decimal::ZERO,
///         OptionStyle::Call,
///         Positive::ZERO,
///         None,
///     );
///     let price = Positive::new_decimal(black_scholes(&option)?)?;
///     quotes.push(OptionQuote {
///         expiry: as_of + TimeDelta::try_days(365).ok_or("days")?,
///         strike,
///         style: OptionStyle::Call,
///         bid: price,
///         ask: price,
///         quoted_at: as_of,
///     });
/// }
///
/// let surface = ImpliedVolatilitySurface::calibrate(
///     &quotes,
///     spot,
///     Decimal::ZERO,
///     Positive::ZERO,
///     as_of,
///     &ImpliedVolatilitySurfaceConfig::default(),
/// )?;
/// assert_eq!(surface.expiries().len(), 1);
/// assert!(surface.rejections().is_empty());
///
/// let point = surface.point(dec!(0), Positive::ONE)?;
/// assert!(!point.is_extrapolated());
/// let difference = point.implied_volatility.to_dec() - known.implied_volatility(dec!(0))?.to_dec();
/// assert!(difference.abs() < dec!(0.000001));
///
/// // Beyond the quotes the default policy refuses to answer.
/// assert!(surface.point(dec!(2), Positive::ONE).is_err());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImpliedVolatilitySurface {
    market: Market,
    extrapolation: ExtrapolationPolicy,
    expiries: Vec<SurfaceExpiry>,
    unfitted: Vec<UnfittedExpiry>,
    rejections: Vec<QuoteRejection>,
    arbitrage_grid: Vec<Decimal>,
    arbitrage_report: StaticArbitrageReport,
    calendar_checked: bool,
    shocks: Vec<SurfaceShock>,
}

impl ImpliedVolatilitySurface {
    /// Calibrates a surface from quotes as of one instant.
    ///
    /// `spot` is the underlying at the as-of time, `rate` the continuously compounded
    /// rate per year, and `dividend_yield` the continuous yield per year. Both are flat
    /// across expiries, and each expiry has its forward `S e^((r - q) T)`. The result does
    /// not depend on the order of `quotes`, except for the order of
    /// [`ImpliedVolatilitySurface::rejections`], which is theirs.
    ///
    /// # Errors
    ///
    /// - [`ImpliedVolatilitySurfaceError::ZeroSpot`]: a zero spot.
    /// - [`ImpliedVolatilitySurfaceError::NegativeQuoteAgeLimit`],
    ///   [`ImpliedVolatilitySurfaceError::ArbitrageGridPointsOutOfRange`], and
    ///   [`ImpliedVolatilitySurfaceError::Svi`] holding [`SviError::InvalidConfig`]: an
    ///   invalid configuration.
    /// - [`ImpliedVolatilitySurfaceError::NoFittedExpiry`]: no expiry has a slice.
    /// - [`ImpliedVolatilitySurfaceError::StaticArbitrage`]: the check could not finish, so
    ///   there is no report. Findings of a check that finished are not an error.
    /// - [`ImpliedVolatilitySurfaceError::Decimal`],
    ///   [`ImpliedVolatilitySurfaceError::Positive`],
    ///   [`ImpliedVolatilitySurfaceError::ExpirationDate`]: a figure leaves the range of its
    ///   type.
    #[instrument(skip(quotes, config), fields(quotes = quotes.len()), level = "debug")]
    pub fn calibrate(
        quotes: &[OptionQuote],
        spot: Positive,
        rate: Decimal,
        dividend_yield: Positive,
        as_of: DateTime<Utc>,
        config: &ImpliedVolatilitySurfaceConfig,
    ) -> Result<Self, ImpliedVolatilitySurfaceError> {
        if spot.is_zero() {
            return Err(ImpliedVolatilitySurfaceError::ZeroSpot);
        }
        check_config(config)?;
        let market = Market {
            spot,
            rate,
            dividend_yield,
            as_of,
        };

        let Cleaned {
            by_expiry,
            rejections,
        } = clean_quotes(quotes, &market, config)?;

        let mut expiries = Vec::with_capacity(by_expiry.len());
        let mut unfitted = Vec::new();
        for (expiry, group) in &by_expiry {
            match fit_expiry(*expiry, group, &market, &config.svi) {
                Ok(fitted) => expiries.push(fitted),
                Err(FitFailure::Surface(error)) => return Err(error),
                Err(FitFailure::Expiry(reason)) => {
                    warn!(%expiry, ?reason, "expiry has no slice");
                    unfitted.push(UnfittedExpiry {
                        expiry: *expiry,
                        time_to_expiry: group.years,
                        clean_quotes: group.clean.len(),
                        reason,
                    });
                }
            }
        }
        if expiries.is_empty() {
            return Err(ImpliedVolatilitySurfaceError::NoFittedExpiry {
                unfitted,
                rejections,
            });
        }

        let arbitrage_grid = union_across_quotes(&expiries, config.arbitrage_grid_points)?;
        let (arbitrage_report, calendar_checked) =
            check_expiries(&expiries, &market, &arbitrage_grid)?;
        log_report(&arbitrage_report);
        debug!(
            expiries = expiries.len(),
            unfitted = unfitted.len(),
            rejections = rejections.len(),
            "surface calibrated"
        );
        Ok(Self {
            market,
            extrapolation: config.extrapolation,
            expiries,
            unfitted,
            rejections,
            arbitrage_grid,
            arbitrage_report,
            calendar_checked,
            shocks: Vec::new(),
        })
    }

    /// The instant the surface describes. No quote later than it entered.
    #[must_use]
    #[inline]
    pub fn as_of(&self) -> DateTime<Utc> {
        self.market.as_of
    }

    /// The spot at the as-of time.
    #[must_use]
    #[inline]
    pub fn spot(&self) -> Positive {
        self.market.spot
    }

    /// The continuously compounded rate per year.
    #[must_use]
    #[inline]
    pub fn rate(&self) -> Decimal {
        self.market.rate
    }

    /// The continuous dividend yield per year.
    #[must_use]
    #[inline]
    pub fn dividend_yield(&self) -> Positive {
        self.market.dividend_yield
    }

    /// The fitted expiries, by increasing expiry. There is at least one.
    #[must_use]
    #[inline]
    pub fn expiries(&self) -> &[SurfaceExpiry] {
        &self.expiries
    }

    /// The expiries with a quote at or before the as-of time and no slice, by increasing
    /// expiry, each with its reason.
    #[must_use]
    #[inline]
    pub fn unfitted(&self) -> &[UnfittedExpiry] {
        &self.unfitted
    }

    /// The quotes that did not enter the surface, each with its reason, in the order of
    /// the input.
    #[must_use]
    #[inline]
    pub fn rejections(&self) -> &[QuoteRejection] {
        &self.rejections
    }

    /// Every residual, by expiry and then by strike.
    pub fn residuals(&self) -> impl Iterator<Item = &SurfaceResidual> {
        self.expiries
            .iter()
            .flat_map(|expiry| expiry.residuals.iter())
    }

    /// The comparison of every fitted expiry with a flat volatility, by expiry.
    pub fn baseline_comparison(&self) -> impl Iterator<Item = &BaselineComparison> {
        self.expiries.iter().map(|expiry| &expiry.baseline)
    }

    /// The findings of the checks run on this surface: see [`ImpliedVolatilitySurface`]
    /// for what they cover. `StaticArbitrageReport::passed` says only that those checks
    /// found nothing on [`ImpliedVolatilitySurface::arbitrage_grid`].
    #[must_use]
    #[inline]
    pub fn arbitrage_report(&self) -> &StaticArbitrageReport {
        &self.arbitrage_report
    }

    /// The log moneyness grid of [`ImpliedVolatilitySurface::arbitrage_report`], strictly
    /// sorted: the union of [`ImpliedVolatilitySurfaceConfig::arbitrage_grid_points`] points
    /// across the quotes of each expiry and the log moneyness of every clean quote.
    #[must_use]
    #[inline]
    pub fn arbitrage_grid(&self) -> &[Decimal] {
        &self.arbitrage_grid
    }

    /// Whether the report includes a calendar check: false for a surface with one expiry.
    #[must_use]
    #[inline]
    pub fn calendar_checked(&self) -> bool {
        self.calendar_checked
    }

    /// The shocks applied to reach this surface from a calibrated one, in order. A surface
    /// with any no longer fits its quotes by construction: its residuals and RMSE compare the
    /// shocked slices with the quotes they came from.
    #[must_use]
    #[inline]
    pub fn shocks(&self) -> &[SurfaceShock] {
        &self.shocks
    }

    /// How old the surface is at `instant`.
    ///
    /// # Errors
    ///
    /// Returns [`ImpliedVolatilitySurfaceError::InstantBeforeAsOf`] for an instant before
    /// the as-of time.
    pub fn age_at(
        &self,
        instant: DateTime<Utc>,
    ) -> Result<TimeDelta, ImpliedVolatilitySurfaceError> {
        if instant < self.market.as_of {
            return Err(ImpliedVolatilitySurfaceError::InstantBeforeAsOf {
                instant,
                as_of: self.market.as_of,
            });
        }
        Ok(instant.signed_duration_since(self.market.as_of))
    }

    /// Reads the surface at a strike and an expiry: `k = ln(K / F(T))`, with the forward of
    /// that expiry. See [`ImpliedVolatilitySurface::point`].
    ///
    /// # Errors
    ///
    /// [`ImpliedVolatilitySurfaceError::ExpiryNotAfterAsOf`] for an expiry not after the
    /// as-of time, [`ImpliedVolatilitySurfaceError::Svi`] for a zero strike, and the errors
    /// of [`ImpliedVolatilitySurface::point`].
    pub fn point_at_strike(
        &self,
        strike: Positive,
        expiry: DateTime<Utc>,
    ) -> Result<SurfacePoint, ImpliedVolatilitySurfaceError> {
        let Some(TimeToExpiry { years, .. }) = time_to_expiry(self.market.as_of, expiry)? else {
            return Err(ImpliedVolatilitySurfaceError::ExpiryNotAfterAsOf {
                expiry,
                as_of: self.market.as_of,
            });
        };
        let k = log_moneyness(strike, self.market.forward(years)?)?;
        self.point(k, years)
    }

    /// Reads the surface at a log moneyness and a time to expiry in years.
    ///
    /// See [`ImpliedVolatilitySurface`] for the interpolation and for what is read outside
    /// the data. A read beyond [`ExtrapolationPolicy`] is an error, and one within it says so in
    /// [`SurfacePoint::is_extrapolated`].
    ///
    /// A `time_to_expiry` within `1e-15` of a fitted expiry, relative to it, reads that slice,
    /// and [`SurfacePoint::time_to_expiry`] is then the `T` of the slice. That is the resolution
    /// of `T`, so `91 / 365` computed in `Decimal` reads the expiry 91 days out, whose own `T`
    /// went through `f64`, rather than falling just outside it. Within that of two fitted
    /// expiries, it reads the nearer. Anything further is a different time to expiry.
    ///
    /// # Errors
    ///
    /// - [`ImpliedVolatilitySurfaceError::ZeroTimeToExpiry`]: a zero time to expiry.
    /// - [`ImpliedVolatilitySurfaceError::WingOutOfPolicy`],
    ///   [`ImpliedVolatilitySurfaceError::ShortExpiryOutOfPolicy`],
    ///   [`ImpliedVolatilitySurfaceError::LongExpiryOutOfPolicy`]: beyond the policy.
    /// - [`ImpliedVolatilitySurfaceError::Svi`], [`ImpliedVolatilitySurfaceError::Decimal`],
    ///   [`ImpliedVolatilitySurfaceError::Positive`]: a figure leaves the range of its type.
    pub fn point(
        &self,
        k: Decimal,
        time_to_expiry: Positive,
    ) -> Result<SurfacePoint, ImpliedVolatilitySurfaceError> {
        const OP: &str = "surface::point";
        if time_to_expiry.is_zero() {
            return Err(ImpliedVolatilitySurfaceError::ZeroTimeToExpiry);
        }
        let located = self.locate(time_to_expiry)?;
        // Within the resolution of `T` a read is of that slice, at its own `T`.
        let time_to_expiry = match located {
            Located::Fitted(expiry) => expiry.time_to_expiry(),
            _ => time_to_expiry,
        };
        let wing_extension = self.wing_extension(k, time_to_expiry, &located)?;
        let total_variance = match located {
            Located::Fitted(expiry) => expiry.slice.total_variance(k)?,
            Located::Between {
                earlier,
                later,
                theta,
            } => Positive::new_decimal(blend(
                earlier.slice.total_variance(k)?.to_dec(),
                later.slice.total_variance(k)?.to_dec(),
                theta,
            )?)?,
            Located::Before(expiry) | Located::After(expiry) => Positive::new_decimal(d_mul(
                expiry.slice.total_variance(k)?.to_dec(),
                d_div(
                    time_to_expiry.to_dec(),
                    expiry.time_to_expiry().to_dec(),
                    OP,
                )?,
                OP,
            )?)?,
        };
        let implied_volatility = Positive::new_decimal(d_sqrt(
            d_div(total_variance.to_dec(), time_to_expiry.to_dec(), OP)?,
            OP,
        )?)?;
        let (expiry_region, bridges_unfitted_expiry) = match located {
            Located::Fitted(_) => (SurfaceExpiryRegion::WithinExpiries, false),
            Located::Between { earlier, later, .. } => {
                let (first, last) = (earlier.time_to_expiry(), later.time_to_expiry());
                (
                    SurfaceExpiryRegion::WithinExpiries,
                    self.unfitted.iter().any(|expiry| {
                        expiry
                            .time_to_expiry
                            .is_some_and(|years| years > first && years < last)
                    }),
                )
            }
            Located::Before(_) => (SurfaceExpiryRegion::BeforeFirstExpiry, false),
            Located::After(_) => (SurfaceExpiryRegion::AfterLastExpiry, false),
        };
        Ok(SurfacePoint {
            log_moneyness: k,
            time_to_expiry,
            forward: self.market.forward(time_to_expiry)?,
            total_variance,
            implied_volatility,
            wing_extension,
            expiry_region,
            bridges_unfitted_expiry,
        })
    }

    /// Where a time to expiry lies against the fitted expiries, or the error the policy
    /// gives there.
    fn locate(
        &self,
        time_to_expiry: Positive,
    ) -> Result<Located<'_>, ImpliedVolatilitySurfaceError> {
        const OP: &str = "surface::locate";
        let after = self
            .expiries
            .partition_point(|expiry| expiry.time_to_expiry() <= time_to_expiry);
        let earlier = after
            .checked_sub(1)
            .and_then(|index| self.expiries.get(index));
        let later = self.expiries.get(after);
        // The two neighbours are the nearest of all the expiries, since they are sorted, and
        // can both be within the resolution: the nearer one is the read, and on a tie the earlier.
        let mut nearest: Option<(Decimal, &SurfaceExpiry)> = None;
        for expiry in [earlier, later].into_iter().flatten() {
            let fitted = expiry.time_to_expiry().to_dec();
            let distance = d_sub(time_to_expiry.to_dec(), fitted, OP)?.abs();
            if distance <= d_mul(fitted, TIME_TO_EXPIRY_RESOLUTION, OP)?
                && nearest.is_none_or(|(best, _)| distance < best)
            {
                nearest = Some((distance, expiry));
            }
        }
        if let Some((_, expiry)) = nearest {
            return Ok(Located::Fitted(expiry));
        }
        match (earlier, later) {
            (Some(earlier), Some(later)) => {
                let first = earlier.time_to_expiry().to_dec();
                let theta = d_div(
                    d_sub(time_to_expiry.to_dec(), first, OP)?,
                    d_sub(later.time_to_expiry().to_dec(), first, OP)?,
                    OP,
                )?;
                Ok(Located::Between {
                    earlier,
                    later,
                    theta,
                })
            }
            (Some(last), None) => {
                let limit = self.extrapolation.max_long_extension;
                let beyond = d_sub(time_to_expiry.to_dec(), last.time_to_expiry().to_dec(), OP)?;
                if beyond > limit.to_dec() {
                    return Err(ImpliedVolatilitySurfaceError::LongExpiryOutOfPolicy {
                        time_to_expiry,
                        last: last.time_to_expiry(),
                        limit,
                    });
                }
                Ok(Located::After(last))
            }
            (None, Some(first)) => {
                let limit = self.extrapolation.max_short_extension;
                let before = d_sub(first.time_to_expiry().to_dec(), time_to_expiry.to_dec(), OP)?;
                if before > limit.to_dec() {
                    return Err(ImpliedVolatilitySurfaceError::ShortExpiryOutOfPolicy {
                        time_to_expiry,
                        first: first.time_to_expiry(),
                        limit,
                    });
                }
                Ok(Located::Before(first))
            }
            // A surface always has an expiry, and with one of them `after` finds it on
            // one side or the other. Kept as an error, not a panic.
            (None, None) => Err(ImpliedVolatilitySurfaceError::NoFittedExpiry {
                unfitted: self.unfitted.clone(),
                rejections: self.rejections.clone(),
            }),
        }
    }

    /// How far `k` is beyond the quotes of the expiries read, or the error the policy gives.
    fn wing_extension(
        &self,
        k: Decimal,
        time_to_expiry: Positive,
        located: &Located<'_>,
    ) -> Result<Positive, ImpliedVolatilitySurfaceError> {
        let (one, other) = match *located {
            Located::Fitted(expiry) | Located::Before(expiry) | Located::After(expiry) => {
                (expiry, expiry)
            }
            Located::Between { earlier, later, .. } => (earlier, later),
        };
        let (one_extension, other_extension) = (one.wing_extension(k)?, other.wing_extension(k)?);
        let (furthest, extension) = if other_extension > one_extension {
            (other, other_extension)
        } else {
            (one, one_extension)
        };
        let extension = Positive::new_decimal(extension)?;
        let limit = self.extrapolation.max_wing_extension;
        if extension > limit {
            return Err(ImpliedVolatilitySurfaceError::WingOutOfPolicy {
                log_moneyness: k,
                time_to_expiry,
                lowest: furthest.lowest_log_moneyness,
                highest: furthest.highest_log_moneyness,
                extension,
                limit,
            });
        }
        Ok(extension)
    }

    /// Runs the check of the report on a grid of the caller's, on the same slices.
    ///
    /// # Errors
    ///
    /// [`ImpliedVolatilitySurfaceError::StaticArbitrage`] if the check cannot finish, an
    /// empty grid among the reasons, and [`ImpliedVolatilitySurfaceError::Decimal`] or
    /// [`ImpliedVolatilitySurfaceError::Positive`] if a forward leaves the range of its type.
    pub fn check_on_grid(
        &self,
        grid: &[Decimal],
    ) -> Result<StaticArbitrageReport, ImpliedVolatilitySurfaceError> {
        Ok(check_expiries(&self.expiries, &self.market, grid)?.0)
    }

    /// Applies a shock and returns the shocked surface, with its own arbitrage report on the
    /// same grid. See [`SurfaceShock`] for each formula. Its rejections and unfitted expiries
    /// are those of this surface, and the limit on the RMSE does not apply to it.
    ///
    /// # Errors
    ///
    /// - [`ImpliedVolatilitySurfaceError::ShockedVolatilityNotPositive`]: a level or term
    ///   shock takes an at-the-money volatility to zero or below.
    /// - [`ImpliedVolatilitySurfaceError::InvalidShockedParameters`]: the parameters are not
    ///   a valid raw SVI slice.
    /// - [`ImpliedVolatilitySurfaceError::StaticArbitrage`]: the check of the shocked surface
    ///   could not finish. Findings of a check that finished are not an error.
    /// - [`ImpliedVolatilitySurfaceError::Pricing`]: an option cannot be priced at its shocked
    ///   implied volatility, which a volatility of exactly zero at a quote is enough for.
    /// - [`ImpliedVolatilitySurfaceError::Svi`], [`ImpliedVolatilitySurfaceError::Decimal`],
    ///   [`ImpliedVolatilitySurfaceError::Positive`]: a figure leaves the range of its type.
    #[instrument(skip(self), level = "debug")]
    pub fn shock(&self, shock: SurfaceShock) -> Result<Self, ImpliedVolatilitySurfaceError> {
        let mut expiries = Vec::with_capacity(self.expiries.len());
        for expiry in &self.expiries {
            let parameters = shocked_parameters(expiry, shock)?;
            let slice = SviSlice::try_new(parameters, expiry.forward(), expiry.time_to_expiry())?;
            expiries.push(SurfaceExpiry::from_slice(
                expiry.expiry,
                slice,
                &expiry.quotes_in_fit(),
                &self.market,
            )?);
        }
        let (arbitrage_report, calendar_checked) =
            check_expiries(&expiries, &self.market, &self.arbitrage_grid)?;
        log_report(&arbitrage_report);
        let mut shocks = self.shocks.clone();
        shocks.push(shock);
        Ok(Self {
            market: self.market,
            extrapolation: self.extrapolation,
            expiries,
            unfitted: self.unfitted.clone(),
            rejections: self.rejections.clone(),
            arbitrage_grid: self.arbitrage_grid.clone(),
            arbitrage_report,
            calendar_checked,
            shocks,
        })
    }

    /// Evaluates the surface at every pair of log moneyness and time to expiry in years, and
    /// exports it as a generic [`Surface`].
    ///
    /// Every point goes through [`ImpliedVolatilitySurface::point`], so the policy applies:
    /// one point beyond it fails the export, those within it are listed in
    /// [`SurfaceExport::extrapolated`], and those read across an expiry without a slice in
    /// [`SurfaceExport::bridging_unfitted`].
    ///
    /// # Errors
    ///
    /// [`ImpliedVolatilitySurfaceError::EmptyExportGrid`] if either axis is empty, and the
    /// errors of [`ImpliedVolatilitySurface::point`].
    pub fn export_surface(
        &self,
        log_moneyness: &[Decimal],
        times_to_expiry: &[Positive],
    ) -> Result<SurfaceExport, ImpliedVolatilitySurfaceError> {
        if log_moneyness.is_empty() || times_to_expiry.is_empty() {
            return Err(ImpliedVolatilitySurfaceError::EmptyExportGrid);
        }
        let mut points = BTreeSet::new();
        let mut extrapolated = Vec::new();
        let mut bridging_unfitted = Vec::new();
        for time_to_expiry in times_to_expiry {
            for k in log_moneyness {
                let point = self.point(*k, *time_to_expiry)?;
                points.insert(Point3D::new(
                    *k,
                    time_to_expiry.to_dec(),
                    point.implied_volatility.to_dec(),
                ));
                if point.is_extrapolated() {
                    extrapolated.push(point);
                }
                if point.bridges_unfitted_expiry {
                    bridging_unfitted.push(point);
                }
            }
        }
        Ok(SurfaceExport {
            surface: Surface::new(points),
            extrapolated,
            bridging_unfitted,
        })
    }
}

/// Rejects the limits of a configuration that could not be applied. Those of the fit are
/// [`calibrate_svi`]'s to check.
fn check_config(
    config: &ImpliedVolatilitySurfaceConfig,
) -> Result<(), ImpliedVolatilitySurfaceError> {
    if config.max_quote_age < TimeDelta::zero() {
        return Err(ImpliedVolatilitySurfaceError::NegativeQuoteAgeLimit {
            limit: config.max_quote_age,
        });
    }
    let points = config.arbitrage_grid_points.get();
    if !(MIN_ARBITRAGE_GRID_POINTS..=MAX_ARBITRAGE_GRID_POINTS).contains(&points) {
        return Err(
            ImpliedVolatilitySurfaceError::ArbitrageGridPointsOutOfRange {
                points,
                minimum: MIN_ARBITRAGE_GRID_POINTS,
                maximum: MAX_ARBITRAGE_GRID_POINTS,
            },
        );
    }
    Ok(())
}

/// Where a time to expiry lies against the fitted expiries.
#[derive(Clone, Copy)]
enum Located<'a> {
    Fitted(&'a SurfaceExpiry),
    Between {
        earlier: &'a SurfaceExpiry,
        later: &'a SurfaceExpiry,
        theta: Decimal,
    },
    Before(&'a SurfaceExpiry),
    After(&'a SurfaceExpiry),
}

/// The inputs shared by every expiry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Market {
    spot: Positive,
    rate: Decimal,
    dividend_yield: Positive,
    as_of: DateTime<Utc>,
}

impl Market {
    /// The forward at a time to expiry: `S e^((r - q) T)`.
    fn forward(&self, years: Positive) -> Result<Positive, ImpliedVolatilitySurfaceError> {
        const OP: &str = "surface::forward";
        let carry = d_mul(
            d_sub(self.rate, self.dividend_yield.to_dec(), OP)?,
            years.to_dec(),
            OP,
        )?;
        let forward = d_mul(self.spot.to_dec(), d_exp(carry, OP)?, OP)?;
        Ok(Positive::new_decimal(forward)?)
    }
}

/// The time from the as-of time to an expiry after it.
#[derive(Debug, Clone, Copy)]
struct TimeToExpiry {
    /// Fractional days, in the form the solvers read.
    days: Positive,
    /// Actual/365 Fixed years, through the same conversion the solvers run.
    years: Positive,
}

/// The time from `as_of` to `expiry`, or `None` if the expiry is not after it.
///
/// Measured here and never by `get_years` on a `DateTime`, which reads a clock. See
/// [`ImpliedVolatilitySurface`] for its precision.
fn time_to_expiry(
    as_of: DateTime<Utc>,
    expiry: DateTime<Utc>,
) -> Result<Option<TimeToExpiry>, ImpliedVolatilitySurfaceError> {
    const OP: &str = "surface::time_to_expiry";
    let elapsed = expiry.signed_duration_since(as_of);
    if elapsed <= TimeDelta::zero() {
        return Ok(None);
    }
    let seconds = d_add(
        Decimal::from(elapsed.num_seconds()),
        Decimal::new(i64::from(elapsed.subsec_nanos()), 9),
        OP,
    )?;
    let days = Positive::new_decimal(d_div(seconds, SECONDS_PER_DAY, OP)?)?;
    let years = years_to_expiration(&ExpirationDate::Days(days))?;
    if years.is_zero() {
        return Ok(None);
    }
    Ok(Some(TimeToExpiry { days, years }))
}

/// A quote that passed every rule.
#[derive(Debug, Clone, Copy)]
struct CleanQuote {
    quote: OptionQuote,
    volatility: Positive,
    raw_weight: Decimal,
}

/// A quote as a slice is measured against it.
#[derive(Debug, Clone, Copy)]
struct QuoteInFit {
    strike: Positive,
    style: OptionStyle,
    volatility: Positive,
    weight: Positive,
    bid: Positive,
    ask: Positive,
}

/// The quotes of one expiry that are at or before the as-of time.
#[derive(Debug, Clone, Default)]
struct ExpiryGroup {
    years: Option<Positive>,
    clean: Vec<CleanQuote>,
}

/// What the cleaning leaves.
struct Cleaned {
    by_expiry: BTreeMap<DateTime<Utc>, ExpiryGroup>,
    rejections: Vec<QuoteRejection>,
}

/// A quote through the rules that need no pricing.
#[derive(Debug, Clone, Copy)]
struct Screened {
    index: usize,
    quote: OptionQuote,
    spread: Decimal,
    time: TimeToExpiry,
}

impl Screened {
    /// Later, as late and tighter, as tight with a lower bid, or first in the input: the
    /// quote of an instrument that stands is the first in this order. It is total, so the
    /// quote that stands does not depend on the order of the input unless two are identical.
    fn order(&self, other: &Self) -> std::cmp::Ordering {
        instrument(&self.quote)
            .cmp(&instrument(&other.quote))
            .then_with(|| other.quote.quoted_at.cmp(&self.quote.quoted_at))
            .then_with(|| self.spread.cmp(&other.spread))
            .then_with(|| self.quote.bid.cmp(&other.quote.bid))
            .then_with(|| self.index.cmp(&other.index))
    }
}

/// What identifies an option: calls before puts at each strike.
fn instrument(quote: &OptionQuote) -> (DateTime<Utc>, Positive, bool) {
    (
        quote.expiry,
        quote.strike,
        matches!(quote.style, OptionStyle::Put),
    )
}

/// Applies the rules of [`ImpliedVolatilitySurface`] in their order.
fn clean_quotes(
    quotes: &[OptionQuote],
    market: &Market,
    config: &ImpliedVolatilitySurfaceConfig,
) -> Result<Cleaned, ImpliedVolatilitySurfaceError> {
    let mut rejected = Vec::new();
    let mut by_expiry: BTreeMap<DateTime<Utc>, ExpiryGroup> = BTreeMap::new();
    let mut times: BTreeMap<DateTime<Utc>, Option<TimeToExpiry>> = BTreeMap::new();
    let mut screened = Vec::with_capacity(quotes.len());
    for (index, quote) in quotes.iter().enumerate() {
        // First, and before anything below reads the quote: this is what keeps a later
        // quote from changing the surface.
        if quote.quoted_at > market.as_of {
            rejected.push((index, QuoteRejectionReason::LaterThanAsOf));
            continue;
        }
        let time = match times.get(&quote.expiry) {
            Some(time) => *time,
            None => {
                let time = time_to_expiry(market.as_of, quote.expiry)?;
                times.insert(quote.expiry, time);
                time
            }
        };
        by_expiry.entry(quote.expiry).or_default().years = time.map(|time| time.years);
        let Some(time) = time else {
            rejected.push((index, QuoteRejectionReason::ExpiredAtAsOf));
            continue;
        };
        if let Some(reason) = screen_quote(quote, market, config) {
            rejected.push((index, reason));
            continue;
        }
        screened.push(Screened {
            index,
            quote: *quote,
            spread: d_sub(quote.ask.to_dec(), quote.bid.to_dec(), "surface::spread")?,
            time,
        });
    }

    screened.sort_by(Screened::order);
    let mut previous = None;
    for candidate in &screened {
        let current = instrument(&candidate.quote);
        if previous == Some(current) {
            rejected.push((candidate.index, QuoteRejectionReason::Superseded));
            continue;
        }
        previous = Some(current);
        match extract_quote(candidate, market) {
            Ok(clean) => by_expiry
                .entry(candidate.quote.expiry)
                .or_default()
                .clean
                .push(clean),
            Err(reason) => rejected.push((candidate.index, reason)),
        }
    }

    rejected.sort_by_key(|(index, _)| *index);
    let mut rejections = Vec::with_capacity(rejected.len());
    for (index, reason) in rejected {
        if let Some(quote) = quotes.get(index) {
            rejections.push(QuoteRejection {
                quote: *quote,
                reason,
            });
        }
    }
    Ok(Cleaned {
        by_expiry,
        rejections,
    })
}

/// The rules that read only the quote.
fn screen_quote(
    quote: &OptionQuote,
    market: &Market,
    config: &ImpliedVolatilitySurfaceConfig,
) -> Option<QuoteRejectionReason> {
    if quote.strike.is_zero() {
        return Some(QuoteRejectionReason::ZeroStrike);
    }
    if quote.bid > quote.ask {
        return Some(QuoteRejectionReason::Crossed);
    }
    if quote.bid.is_zero() {
        return Some(QuoteRejectionReason::ZeroBid);
    }
    let age = market.as_of.signed_duration_since(quote.quoted_at);
    if age > config.max_quote_age {
        return Some(QuoteRejectionReason::Stale {
            age,
            limit: config.max_quote_age,
        });
    }
    None
}

/// One long European option, as a quote is priced.
fn european_option(
    strike: Positive,
    style: OptionStyle,
    days: Positive,
    volatility: Positive,
    market: &Market,
) -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        String::new(),
        strike,
        ExpirationDate::Days(days),
        volatility,
        Positive::ONE,
        market.spot,
        market.rate,
        style,
        market.dividend_yield,
        None,
    )
}

/// The rules that price the quote: its band, its implied volatility, its weight.
fn extract_quote(screened: &Screened, market: &Market) -> Result<CleanQuote, QuoteRejectionReason> {
    const OP: &str = "surface::quote";
    let quote = &screened.quote;
    let mid = d_add(quote.bid.to_dec(), quote.ask.to_dec(), OP)
        .and_then(|sum| d_div(sum, Decimal::TWO, OP))
        .map_err(numerical_failure)?;
    let mut option = european_option(
        quote.strike,
        quote.style,
        screened.time.days,
        *MIN_VOLATILITY,
        market,
    );
    let (lower, upper) = european_price_band(&option).map_err(numerical_failure)?;
    if mid < lower || mid > upper {
        return Err(QuoteRejectionReason::OutsideNoArbitrageBand { mid, lower, upper });
    }
    let guess = iv_initial_guess(&option, mid).unwrap_or(IV_BRACKET_MAX);
    let outcome = solve_implied_volatility(
        *MIN_VOLATILITY,
        IV_BRACKET_MAX,
        guess,
        MAX_ITERATIONS_IV,
        |sigma, with_slope| long_residual(&option, mid, sigma, with_slope),
    )
    .map_err(numerical_failure)?;
    let volatility = solved_volatility(outcome)?;
    option.implied_volatility = volatility;
    let vega = vega_slope(&option);
    if vega.is_zero() {
        return Err(QuoteRejectionReason::NoWeight);
    }
    let floor = d_mul(mid, WEIGHT_SPREAD_FLOOR, OP).map_err(numerical_failure)?;
    let raw_weight = d_mul(volatility.to_dec(), screened.spread.max(floor), OP)
        .and_then(|width| d_div(vega, width, OP))
        .map_err(|_| QuoteRejectionReason::NoWeight)?;
    Ok(CleanQuote {
        quote: *quote,
        volatility,
        raw_weight,
    })
}

/// The volatility of a solve, or the rejection of a quote it found none for. The floor is
/// not a volatility here: a quote at or under it has none.
fn solved_volatility(outcome: IvOutcome) -> Result<Positive, QuoteRejectionReason> {
    match outcome {
        IvOutcome::Found(volatility) => Ok(volatility),
        IvOutcome::BelowBracket => Err(QuoteRejectionReason::BelowVolatilityFloor),
        IvOutcome::AboveBracket => Err(QuoteRejectionReason::AboveVolatilityCap),
        IvOutcome::NoConvergence(last_volatility) => {
            Err(QuoteRejectionReason::NoConvergence { last_volatility })
        }
    }
}

/// A failure of the arithmetic of one quote, which costs that quote and not the surface.
#[cold]
#[inline(never)]
fn numerical_failure(error: impl std::fmt::Display) -> QuoteRejectionReason {
    QuoteRejectionReason::NumericalFailure {
        reason: error.to_string(),
    }
}

/// Why an expiry did not give a slice: the whole surface, or only that expiry.
enum FitFailure {
    Surface(ImpliedVolatilitySurfaceError),
    Expiry(UnfittedReason),
}

impl From<SviError> for FitFailure {
    fn from(error: SviError) -> Self {
        match error {
            // The same configuration fails every expiry, and is the caller's to fix.
            config @ SviError::InvalidConfig { .. } => Self::Surface(config.into()),
            other => Self::Expiry(other.into()),
        }
    }
}

impl From<ImpliedVolatilitySurfaceError> for FitFailure {
    fn from(error: ImpliedVolatilitySurfaceError) -> Self {
        Self::Expiry(UnfittedReason::CalibrationFailed {
            reason: error.to_string(),
        })
    }
}

/// Fits one expiry on its clean quotes.
fn fit_expiry(
    expiry: DateTime<Utc>,
    group: &ExpiryGroup,
    market: &Market,
    config: &SviCalibrationConfig,
) -> Result<SurfaceExpiry, FitFailure> {
    let Some(years) = group.years else {
        return Err(FitFailure::Expiry(UnfittedReason::ExpiredAtAsOf));
    };
    let forward = market.forward(years)?;
    let quotes = quotes_with_weights(&group.clean)?;
    let observations: Vec<SviObservation> = quotes
        .iter()
        .map(|quote| SviObservation {
            strike: quote.strike,
            implied_volatility: quote.volatility,
            weight: quote.weight,
        })
        .collect();
    let fit = calibrate_svi(&observations, forward, years, config)?;
    Ok(SurfaceExpiry::from_slice(
        expiry,
        *fit.slice(),
        &quotes,
        market,
    )?)
}

/// The weight of each quote: see [`ImpliedVolatilitySurface`]. Normalized by the largest before it is
/// squared, so a large ratio cannot overflow.
fn quotes_with_weights(
    clean: &[CleanQuote],
) -> Result<Vec<QuoteInFit>, ImpliedVolatilitySurfaceError> {
    const OP: &str = "surface::weight";
    let mut largest = Decimal::ZERO;
    for quote in clean {
        largest = largest.max(quote.raw_weight);
    }
    let mut quotes = Vec::with_capacity(clean.len());
    for quote in clean {
        let ratio = if largest.is_zero() {
            Decimal::ZERO
        } else {
            d_div(quote.raw_weight, largest, OP)?
        };
        quotes.push(QuoteInFit {
            strike: quote.quote.strike,
            style: quote.quote.style,
            volatility: quote.volatility,
            weight: Positive::new_decimal(d_mul(ratio, ratio, OP)?)?,
            bid: quote.quote.bid,
            ask: quote.quote.ask,
        });
    }
    Ok(quotes)
}

/// The strictly sorted union, over the expiries, of `points` evenly spaced points across the
/// quotes of each, both ends included, and the log moneyness of each of its clean quotes.
///
/// One grid spaced by the widest quotes would leave a short expiry with a point or two inside
/// its own. A step for each expiry still leaves few among dense quotes when a far quote
/// stretches it, so every quote is a point in itself: where there is data, there is a check.
fn union_across_quotes(
    expiries: &[SurfaceExpiry],
    points: NonZeroUsize,
) -> Result<Vec<Decimal>, DecimalError> {
    const OP: &str = "surface::grid";
    let intervals = d_sub(Decimal::from(points.get()), Decimal::ONE, OP)?;
    // Only a hint: a size that does not fit in a `usize` allocates as it goes.
    let capacity = expiries
        .iter()
        .try_fold(0_usize, |total, expiry| {
            total
                .checked_add(points.get())?
                .checked_add(expiry.residuals.len())
        })
        .unwrap_or(0);
    let mut grid = Vec::with_capacity(capacity);
    for expiry in expiries {
        let (lowest, highest) = (expiry.lowest_log_moneyness, expiry.highest_log_moneyness);
        let step = d_div(d_sub(highest, lowest, OP)?, intervals, OP)?;
        grid.push(lowest);
        for index in 1..points.get() {
            grid.push(d_add(lowest, d_mul(step, Decimal::from(index), OP)?, OP)?);
        }
        // The step is rounded, and the last quote is in the grid exactly.
        if let Some(end) = grid.last_mut() {
            *end = highest;
        }
        grid.extend(
            expiry
                .residuals
                .iter()
                .map(|residual| residual.log_moneyness),
        );
    }
    grid.sort_unstable();
    grid.dedup();
    Ok(grid)
}

/// A slice as the check reads it: a fitted one, or a blend of two at an expiry between them.
struct GridSlice {
    earlier: SviSlice,
    later: SviSlice,
    theta: Decimal,
    time_to_expiry: Positive,
    forward: Positive,
}

/// `(1 - theta) * earlier + theta * later`: a sum of two non-negative terms for non-negative
/// inputs, so rounding cannot take it below zero.
fn blend(earlier: Decimal, later: Decimal, theta: Decimal) -> Result<Decimal, DecimalError> {
    const OP: &str = "surface::blend";
    if theta.is_zero() {
        return Ok(earlier);
    }
    d_add(
        d_mul(d_sub(Decimal::ONE, theta, OP)?, earlier, OP)?,
        d_mul(theta, later, OP)?,
        OP,
    )
}

impl TotalVarianceSlice for GridSlice {
    fn time_to_expiry(&self) -> Positive {
        self.time_to_expiry
    }

    fn forward(&self) -> Positive {
        self.forward
    }

    fn total_variance(&self, k: Decimal) -> Result<Positive, StaticArbitrageError> {
        Ok(Positive::new_decimal(blend(
            self.earlier.total_variance(k)?.to_dec(),
            self.later.total_variance(k)?.to_dec(),
            self.theta,
        )?)?)
    }

    fn total_variance_slope(&self, k: Decimal) -> Result<Decimal, StaticArbitrageError> {
        Ok(blend(
            self.earlier.total_variance_slope(k)?,
            self.later.total_variance_slope(k)?,
            self.theta,
        )?)
    }

    fn total_variance_curvature(&self, k: Decimal) -> Result<Decimal, StaticArbitrageError> {
        Ok(blend(
            self.earlier.total_variance_curvature(k)?,
            self.later.total_variance_curvature(k)?,
            self.theta,
        )?)
    }

    fn total_variance_rounding(&self, k: Decimal) -> Result<Positive, StaticArbitrageError> {
        const OP: &str = "surface::blend_rounding";
        let rounding = blend(
            TotalVarianceSlice::total_variance_rounding(&self.earlier, k)?.to_dec(),
            TotalVarianceSlice::total_variance_rounding(&self.later, k)?.to_dec(),
            self.theta,
        )?;
        if self.theta.is_zero() {
            return Ok(Positive::new_decimal(rounding)?);
        }
        let size = d_add(
            self.earlier.total_variance(k)?.to_dec(),
            self.later.total_variance(k)?.to_dec(),
            OP,
        )?;
        let band = d_add(
            d_mul(BLEND_ROUNDING_BAND, size, OP)?,
            BLEND_ROUNDING_FLOOR,
            OP,
        )?;
        Ok(Positive::new_decimal(d_add(rounding, band, OP)?)?)
    }
}

/// The slices the check reads: each fitted expiry, and the middle of each gap.
fn slices_to_check(
    expiries: &[SurfaceExpiry],
    market: &Market,
) -> Result<Vec<GridSlice>, ImpliedVolatilitySurfaceError> {
    const OP: &str = "surface::slices_to_check";
    let mut slices = Vec::with_capacity(expiries.len());
    for expiry in expiries {
        slices.push(GridSlice {
            earlier: expiry.slice,
            later: expiry.slice,
            theta: Decimal::ZERO,
            time_to_expiry: expiry.time_to_expiry(),
            forward: expiry.forward(),
        });
    }
    for pair in expiries.windows(2) {
        if let [earlier, later] = pair {
            let middle = Positive::new_decimal(d_div(
                d_add(
                    earlier.time_to_expiry().to_dec(),
                    later.time_to_expiry().to_dec(),
                    OP,
                )?,
                Decimal::TWO,
                OP,
            )?)?;
            slices.push(GridSlice {
                earlier: earlier.slice,
                later: later.slice,
                theta: MIDDLE_OF_GAP,
                time_to_expiry: middle,
                forward: market.forward(middle)?,
            });
        }
    }
    Ok(slices)
}

/// Runs the checks of the report, and says whether they include a calendar check.
fn check_expiries(
    expiries: &[SurfaceExpiry],
    market: &Market,
    grid: &[Decimal],
) -> Result<(StaticArbitrageReport, bool), ImpliedVolatilitySurfaceError> {
    let slices = slices_to_check(expiries, market)?;
    match slices.as_slice() {
        [only] => Ok((check_butterfly(only, grid)?, false)),
        all => Ok((check_static_arbitrage(all, grid)?, true)),
    }
}

/// A report with findings is worth a warning: a surface is returned with them.
fn log_report(report: &StaticArbitrageReport) {
    if !report.passed() {
        warn!(
            violations = report.violations().len(),
            unevaluated = report.unevaluated().len(),
            "surface has static arbitrage findings"
        );
    }
}

/// The parameters of a slice under a shock.
fn shocked_parameters(
    expiry: &SurfaceExpiry,
    shock: SurfaceShock,
) -> Result<SviParameters, ImpliedVolatilitySurfaceError> {
    const OP: &str = "surface::shock";
    let time_to_expiry = expiry.time_to_expiry();
    match shock {
        SurfaceShock::Level { volatility } => shifted_level(expiry, volatility),
        SurfaceShock::Term { slope, pivot } => {
            let distance = d_sub(time_to_expiry.to_dec(), pivot.to_dec(), OP)?;
            shifted_level(expiry, d_mul(slope, distance, OP)?)
        }
        SurfaceShock::Skew { rho } => {
            let parameters = expiry.slice.parameters();
            SviParameters::try_new(
                parameters.a(),
                parameters.b(),
                d_add(parameters.rho(), rho, OP)?,
                parameters.m(),
                parameters.sigma(),
            )
            .map_err(|source| {
                ImpliedVolatilitySurfaceError::InvalidShockedParameters {
                    time_to_expiry,
                    source,
                }
            })
        }
    }
}

/// `a' = a + x * (2 * iv_atm + x) * T`, which moves the at-the-money volatility by `x`.
fn shifted_level(
    expiry: &SurfaceExpiry,
    shift: Decimal,
) -> Result<SviParameters, ImpliedVolatilitySurfaceError> {
    const OP: &str = "surface::shock_level";
    let time_to_expiry = expiry.time_to_expiry();
    let at_the_money = expiry.slice.implied_volatility(Decimal::ZERO)?;
    if d_add(at_the_money.to_dec(), shift, OP)? <= Decimal::ZERO {
        return Err(
            ImpliedVolatilitySurfaceError::ShockedVolatilityNotPositive {
                time_to_expiry,
                at_the_money,
                shift,
            },
        );
    }
    let doubled = d_mul(Decimal::TWO, at_the_money.to_dec(), OP)?;
    let variance_shift = d_mul(
        d_mul(shift, d_add(doubled, shift, OP)?, OP)?,
        time_to_expiry.to_dec(),
        OP,
    )?;
    let parameters = expiry.slice.parameters();
    SviParameters::try_new(
        d_add(parameters.a(), variance_shift, OP)?,
        parameters.b(),
        parameters.rho(),
        parameters.m(),
        parameters.sigma(),
    )
    .map_err(
        |source| ImpliedVolatilitySurfaceError::InvalidShockedParameters {
            time_to_expiry,
            source,
        },
    )
}

#[cfg(test)]
mod tests_implied_volatility_surface {
    use super::*;
    use crate::pricing::black_scholes_model::black_scholes;
    use chrono::TimeZone;
    use rust_decimal_macros::dec;

    const RATE: Decimal = Decimal::from_parts(3, 0, 0, false, 2);
    const YIELD: Decimal = Decimal::from_parts(1, 0, 0, false, 2);

    fn pos(value: Decimal) -> Positive {
        Positive::new_decimal(value).unwrap()
    }

    fn as_of() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2030, 1, 1, 12, 0, 0).unwrap()
    }

    fn in_days(days: i64) -> DateTime<Utc> {
        as_of() + TimeDelta::try_days(days).unwrap()
    }

    fn seconds(count: i64) -> TimeDelta {
        TimeDelta::try_seconds(count).unwrap()
    }

    fn years(days: i64) -> Positive {
        years_to_expiration(&ExpirationDate::Days(pos(Decimal::from(days)))).unwrap()
    }

    fn market() -> Market {
        Market {
            spot: Positive::HUNDRED,
            rate: RATE,
            dividend_yield: pos(YIELD),
            as_of: as_of(),
        }
    }

    /// An SSVI slice in raw parameters, by (3.3) of Gatheral and Jacquier with
    /// `theta = 0.04 * T`, `eta = 1` and `rho = -0.4`, which Theorems 4.1 and 4.2 clear.
    fn known(days: i64) -> SviSlice {
        known_scaled(days, dec!(0.04))
    }

    fn known_scaled(days: i64, variance: Decimal) -> SviSlice {
        let t = years(days);
        let theta = variance * t.to_dec();
        let rho = dec!(-0.4);
        let phi = Decimal::ONE / d_sqrt(theta, "test").unwrap();
        let room = d_sqrt(Decimal::ONE - rho * rho, "test").unwrap();
        SviSlice::try_new(
            SviParameters::try_new(
                theta / dec!(2) * (Decimal::ONE - rho * rho),
                theta * phi / dec!(2),
                rho,
                -rho / phi,
                room / phi,
            )
            .unwrap(),
            market().forward(t).unwrap(),
            t,
        )
        .unwrap()
    }

    fn price(
        strike: Positive,
        style: OptionStyle,
        days: Positive,
        volatility: Positive,
    ) -> Decimal {
        black_scholes(&Options::new(
            OptionType::European,
            Side::Long,
            String::new(),
            strike,
            ExpirationDate::Days(days),
            volatility,
            Positive::ONE,
            Positive::HUNDRED,
            RATE,
            style,
            pos(YIELD),
            None,
        ))
        .unwrap()
    }

    /// A quote 1% either side of the closed form at `volatility`.
    fn quote_at_volatility(
        days: i64,
        strike: Decimal,
        style: OptionStyle,
        volatility: Positive,
    ) -> OptionQuote {
        let strike = pos(strike);
        let mid = price(strike, style, pos(Decimal::from(days)), volatility);
        OptionQuote {
            expiry: in_days(days),
            strike,
            style,
            bid: pos(mid * dec!(0.99)),
            ask: pos(mid * dec!(1.01)),
            quoted_at: as_of(),
        }
    }

    fn quote_of(slice: &SviSlice, days: i64, strike: Decimal, style: OptionStyle) -> OptionQuote {
        let volatility = slice.implied_volatility_at_strike(pos(strike)).unwrap();
        quote_at_volatility(days, strike, style, volatility)
    }

    /// Out-of-the-money quotes of a slice, 15 strikes across two standard deviations.
    fn chain(slice: &SviSlice, days: i64) -> Vec<OptionQuote> {
        let forward = slice.forward().to_dec();
        let width = d_sqrt(years(days).to_dec(), "test").unwrap() * dec!(0.1);
        (-7_i32..=7)
            .map(|step| {
                let k = width * Decimal::from(step) / dec!(3.5);
                let strike = (forward * d_exp(k, "test").unwrap()).round_dp(2);
                let style = if strike >= forward {
                    OptionStyle::Call
                } else {
                    OptionStyle::Put
                };
                quote_of(slice, days, strike, style)
            })
            .collect()
    }

    fn quotes_of(expiries: &[i64]) -> Vec<OptionQuote> {
        expiries
            .iter()
            .flat_map(|days| chain(&known(*days), *days))
            .collect()
    }

    fn calibrate_with(
        quotes: &[OptionQuote],
        config: &ImpliedVolatilitySurfaceConfig,
    ) -> Result<ImpliedVolatilitySurface, ImpliedVolatilitySurfaceError> {
        ImpliedVolatilitySurface::calibrate(
            quotes,
            Positive::HUNDRED,
            RATE,
            pos(YIELD),
            as_of(),
            config,
        )
    }

    fn calibrate(
        quotes: &[OptionQuote],
    ) -> Result<ImpliedVolatilitySurface, ImpliedVolatilitySurfaceError> {
        calibrate_with(quotes, &ImpliedVolatilitySurfaceConfig::default())
    }

    fn flexible() -> ImpliedVolatilitySurfaceConfig {
        ImpliedVolatilitySurfaceConfig {
            extrapolation: ExtrapolationPolicy {
                max_wing_extension: pos(dec!(0.5)),
                max_short_extension: pos(dec!(0.05)),
                max_long_extension: Positive::ONE,
            },
            ..ImpliedVolatilitySurfaceConfig::default()
        }
    }

    /// Why one quote among a clean chain was rejected.
    fn rejection_of(extra: OptionQuote) -> QuoteRejectionReason {
        let mut quotes = chain(&known(365), 365);
        quotes.push(extra);
        let surface = calibrate(&quotes).unwrap();
        assert_eq!(surface.rejections().len(), 1, "{:?}", surface.rejections());
        let rejection = surface.rejections().first().unwrap();
        assert_eq!(rejection.quote, extra);
        rejection.reason.clone()
    }

    /// A quote that is clean on its own, away from the strikes of `chain`.
    fn good_quote() -> OptionQuote {
        quote_of(&known(365), 365, dec!(101.5), OptionStyle::Call)
    }

    fn assert_close(actual: Decimal, expected: Decimal, tolerance: Decimal) {
        assert!(
            (actual - expected).abs() <= tolerance,
            "{actual} is not within {tolerance} of {expected}"
        );
    }

    // Time to expiry.

    #[test]
    fn test_time_to_expiry_is_actual_365_with_its_hours_and_seconds() {
        let six_hours = time_to_expiry(as_of(), as_of() + seconds(21_600))
            .unwrap()
            .unwrap();
        assert_eq!(six_hours.days, pos(dec!(0.25)));
        assert_close(
            six_hours.years.to_dec(),
            dec!(0.25) / dec!(365),
            dec!(0.000000000000000001),
        );
        let one_second = time_to_expiry(as_of(), as_of() + seconds(1))
            .unwrap()
            .unwrap();
        assert_close(
            one_second.years.to_dec(),
            Decimal::ONE / dec!(31536000),
            dec!(0.0000000000000000000001),
        );
    }

    #[test]
    fn test_time_to_expiry_resolves_nanoseconds_at_one_day_and_30_at_one_year() {
        let one_day = in_days(1);
        let day = time_to_expiry(as_of(), one_day).unwrap().unwrap();
        let day_and_nanosecond = time_to_expiry(as_of(), one_day + TimeDelta::nanoseconds(1))
            .unwrap()
            .unwrap();
        assert!(day_and_nanosecond.years > day.years);
        let one_year = in_days(365);
        let year = time_to_expiry(as_of(), one_year).unwrap().unwrap();
        let later = time_to_expiry(as_of(), one_year + TimeDelta::nanoseconds(30))
            .unwrap()
            .unwrap();
        assert!(later.years > year.years);
    }

    #[test]
    fn test_time_to_expiry_is_none_when_the_expiry_is_not_after_as_of() {
        assert!(time_to_expiry(as_of(), as_of()).unwrap().is_none());
        assert!(
            time_to_expiry(as_of(), as_of() - seconds(1))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn test_expiries_closer_than_the_resolution_of_t_are_reported_as_sharing_it() {
        let one = chain(&known(365), 365);
        let other: Vec<OptionQuote> = one
            .iter()
            .map(|quote| OptionQuote {
                expiry: quote.expiry + TimeDelta::nanoseconds(1),
                ..*quote
            })
            .collect();
        assert!(matches!(
            calibrate(&[one, other].concat()),
            Err(ImpliedVolatilitySurfaceError::StaticArbitrage(
                StaticArbitrageError::DuplicateExpiry { .. }
            ))
        ));
    }

    #[test]
    fn test_calibrate_sub_day_expiries_use_their_own_hours() {
        // Quotes of one implied volatility at six and at seven hours: a time rounded to a
        // whole day, or to zero, would give each a different volatility or none.
        let volatility = pos(dec!(0.3));
        let mut quotes = Vec::new();
        for (hours, days) in [(6_i64, dec!(0.25)), (7, dec!(7) / dec!(24))] {
            for strike in [dec!(99), dec!(99.5), dec!(100), dec!(100.5), dec!(101)] {
                let strike = pos(strike);
                let style = OptionStyle::Call;
                let mid = price(strike, style, pos(days), volatility);
                quotes.push(OptionQuote {
                    expiry: as_of() + TimeDelta::try_hours(hours).unwrap(),
                    strike,
                    style,
                    bid: pos(mid),
                    ask: pos(mid),
                    quoted_at: as_of(),
                });
            }
        }
        let surface = calibrate(&quotes).unwrap();
        assert!(
            surface.rejections().is_empty(),
            "{:?}",
            surface.rejections()
        );
        let [six, seven] = surface.expiries() else {
            panic!("two expiries: {:?}", surface.unfitted());
        };
        assert_close(
            six.time_to_expiry().to_dec(),
            dec!(0.25) / dec!(365),
            dec!(0.000000000000000001),
        );
        assert!(seven.time_to_expiry() > six.time_to_expiry());
        for residual in surface.residuals() {
            assert_close(
                residual.observed_implied_volatility.to_dec(),
                dec!(0.3),
                dec!(0.00001),
            );
        }
    }

    // Cleaning, one rule at a time.

    #[test]
    fn test_quote_later_than_as_of_is_rejected_and_at_as_of_is_clean() {
        let at = good_quote();
        assert_eq!(
            calibrate(&[chain(&known(365), 365), vec![at]].concat())
                .unwrap()
                .rejections(),
            []
        );
        let later = OptionQuote {
            quoted_at: as_of() + TimeDelta::nanoseconds(1),
            ..at
        };
        assert_eq!(rejection_of(later), QuoteRejectionReason::LaterThanAsOf);
    }

    #[test]
    fn test_quote_whose_expiry_is_not_after_as_of_is_rejected() {
        for expiry in [as_of(), as_of() - seconds(1)] {
            let quote = OptionQuote {
                expiry,
                ..good_quote()
            };
            assert_eq!(rejection_of(quote), QuoteRejectionReason::ExpiredAtAsOf);
        }
    }

    #[test]
    fn test_quote_with_zero_strike_is_rejected() {
        let quote = OptionQuote {
            strike: Positive::ZERO,
            ..good_quote()
        };
        assert_eq!(rejection_of(quote), QuoteRejectionReason::ZeroStrike);
    }

    #[test]
    fn test_crossed_quote_is_rejected_and_a_locked_one_is_clean() {
        let good = good_quote();
        let crossed = OptionQuote {
            bid: good.ask,
            ask: good.bid,
            ..good
        };
        assert_eq!(rejection_of(crossed), QuoteRejectionReason::Crossed);
        let locked = OptionQuote {
            ask: good.bid,
            ..good
        };
        let quotes = [chain(&known(365), 365), vec![locked]].concat();
        assert_eq!(calibrate(&quotes).unwrap().rejections(), []);
    }

    #[test]
    fn test_quote_with_zero_bid_is_rejected() {
        let quote = OptionQuote {
            bid: Positive::ZERO,
            ..good_quote()
        };
        assert_eq!(rejection_of(quote), QuoteRejectionReason::ZeroBid);
    }

    #[test]
    fn test_quote_older_than_the_limit_is_stale_and_exactly_that_old_is_clean() {
        let limit = ImpliedVolatilitySurfaceConfig::default().max_quote_age;
        let old = OptionQuote {
            quoted_at: as_of() - limit,
            ..good_quote()
        };
        let quotes = [chain(&known(365), 365), vec![old]].concat();
        assert_eq!(calibrate(&quotes).unwrap().rejections(), []);
        let older = OptionQuote {
            quoted_at: as_of() - limit - TimeDelta::nanoseconds(1),
            ..good_quote()
        };
        assert_eq!(
            rejection_of(older),
            QuoteRejectionReason::Stale {
                age: limit + TimeDelta::nanoseconds(1),
                limit,
            }
        );
    }

    /// The quote of two for one instrument that stands, in either order.
    fn standing(one: OptionQuote, other: OptionQuote) -> OptionQuote {
        let mut stands = None;
        for pair in [[one, other], [other, one]] {
            let quotes = [chain(&known(365), 365), pair.to_vec()].concat();
            let surface = calibrate(&quotes).unwrap();
            let rejected = surface.rejections().first().unwrap();
            assert_eq!(surface.rejections().len(), 1);
            assert_eq!(rejected.reason, QuoteRejectionReason::Superseded);
            let kept = if rejected.quote == one { other } else { one };
            assert_eq!(*stands.get_or_insert(kept), kept);
        }
        stands.unwrap()
    }

    #[test]
    fn test_later_quote_of_an_instrument_supersedes_the_older_one() {
        let older = OptionQuote {
            quoted_at: as_of() - seconds(10),
            ..good_quote()
        };
        assert_eq!(standing(older, good_quote()), good_quote());
    }

    #[test]
    fn test_tighter_quote_of_the_same_instant_supersedes_the_wider_one() {
        let good = good_quote();
        let wider = OptionQuote {
            ask: pos(good.ask.to_dec() * dec!(1.01)),
            ..good
        };
        assert_eq!(standing(wider, good), good);
    }

    #[test]
    fn test_lower_bid_of_the_same_instant_and_spread_supersedes_the_higher_one() {
        let good = good_quote();
        let higher = OptionQuote {
            bid: pos(good.bid.to_dec() + dec!(0.01)),
            ask: pos(good.ask.to_dec() + dec!(0.01)),
            ..good
        };
        assert_eq!(standing(higher, good), good);
    }

    #[test]
    fn test_newer_quote_a_rule_rejects_leaves_the_older_one_in_the_fit() {
        let older: Vec<OptionQuote> = chain(&known(365), 365)
            .into_iter()
            .map(|quote| OptionQuote {
                quoted_at: as_of() - seconds(200),
                ..quote
            })
            .collect();
        let base = calibrate(&older).unwrap();
        let first = *older.first().unwrap();
        let crossed = OptionQuote {
            bid: first.ask,
            ask: first.bid,
            quoted_at: as_of(),
            ..first
        };
        let with = calibrate(&[older, vec![crossed]].concat()).unwrap();
        assert_eq!(with.expiries(), base.expiries());
        let rejection = with.rejections().first().unwrap();
        assert_eq!(with.rejections().len(), 1);
        assert_eq!(rejection.quote, crossed);
        assert_eq!(rejection.reason, QuoteRejectionReason::Crossed);
    }

    #[test]
    fn test_call_and_put_at_one_strike_are_different_instruments() {
        let good = good_quote();
        let put = quote_of(&known(365), 365, dec!(101.5), OptionStyle::Put);
        let quotes = [chain(&known(365), 365), vec![good, put]].concat();
        assert_eq!(calibrate(&quotes).unwrap().rejections(), []);
    }

    /// The band of a European call at a strike of 100 and one year.
    fn call_band() -> (Decimal, Decimal) {
        let t = years(365).to_dec();
        let spot_pv = dec!(100) * d_exp(-YIELD * t, "test").unwrap();
        let strike_pv = dec!(100) * d_exp(-RATE * t, "test").unwrap();
        (spot_pv - strike_pv, spot_pv)
    }

    fn call_at_mid(mid: Decimal) -> OptionQuote {
        OptionQuote {
            expiry: in_days(365),
            strike: Positive::HUNDRED,
            style: OptionStyle::Call,
            bid: pos(mid),
            ask: pos(mid),
            quoted_at: as_of(),
        }
    }

    #[test]
    fn test_mid_above_the_band_is_rejected_with_the_band() {
        let (lower, upper) = call_band();
        let mid = upper + dec!(0.01);
        match rejection_of(call_at_mid(mid)) {
            QuoteRejectionReason::OutsideNoArbitrageBand {
                mid: found,
                lower: low,
                upper: high,
            } => {
                assert_eq!(found, mid);
                assert_close(low, lower, dec!(0.0000000001));
                assert_close(high, upper, dec!(0.0000000001));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn test_mid_exactly_at_the_top_of_the_band_is_inside_it_and_above_the_cap() {
        let quote = call_at_mid(dec!(50));
        let time = time_to_expiry(as_of(), quote.expiry).unwrap().unwrap();
        let (_, upper) = european_price_band(&Options::new(
            OptionType::European,
            Side::Long,
            String::new(),
            quote.strike,
            ExpirationDate::Days(time.days),
            *MIN_VOLATILITY,
            Positive::ONE,
            Positive::HUNDRED,
            RATE,
            quote.style,
            pos(YIELD),
            None,
        ))
        .unwrap();
        assert_eq!(
            rejection_of(call_at_mid(upper)),
            QuoteRejectionReason::AboveVolatilityCap
        );
    }

    #[test]
    fn test_mid_below_the_band_is_rejected() {
        let (lower, _) = call_band();
        assert!(matches!(
            rejection_of(call_at_mid(lower - dec!(0.01))),
            QuoteRejectionReason::OutsideNoArbitrageBand { .. }
        ));
    }

    #[test]
    fn test_put_mid_above_the_band_of_a_put_is_rejected() {
        // The discounted strike is 97.04, far under the top of the band of a call.
        let quote = OptionQuote {
            style: OptionStyle::Put,
            ..call_at_mid(dec!(98))
        };
        assert!(matches!(
            rejection_of(quote),
            QuoteRejectionReason::OutsideNoArbitrageBand { .. }
        ));
    }

    #[test]
    fn test_mid_inside_the_band_at_a_price_no_volatility_reaches_is_above_the_cap() {
        // At 500% a call at the money of one year prices below 98: above that nothing does.
        let (_, upper) = call_band();
        assert_eq!(
            rejection_of(call_at_mid(upper - dec!(0.01))),
            QuoteRejectionReason::AboveVolatilityCap
        );
    }

    #[test]
    fn test_mid_at_the_bottom_of_the_band_is_below_the_floor() {
        // Exactly at the discounted intrinsic value: every volatility near zero prices it
        // too, so none is implied, and the floor must not be returned as if it were one.
        let t = years(365).to_dec();
        let spot_pv = dec!(100) * d_exp(-YIELD * t, "test").unwrap();
        let strike = dec!(80);
        let strike_pv = strike * d_exp(-RATE * t, "test").unwrap();
        let quote = OptionQuote {
            strike: pos(strike),
            ..call_at_mid(spot_pv - strike_pv)
        };
        assert_eq!(
            rejection_of(quote),
            QuoteRejectionReason::BelowVolatilityFloor
        );
    }

    #[test]
    fn test_solver_outcomes_map_to_a_volatility_or_a_reason() {
        let last = pos(dec!(0.7));
        assert_eq!(solved_volatility(IvOutcome::Found(last)), Ok(last));
        assert_eq!(
            solved_volatility(IvOutcome::BelowBracket),
            Err(QuoteRejectionReason::BelowVolatilityFloor)
        );
        assert_eq!(
            solved_volatility(IvOutcome::AboveBracket),
            Err(QuoteRejectionReason::AboveVolatilityCap)
        );
        assert_eq!(
            solved_volatility(IvOutcome::NoConvergence(last)),
            Err(QuoteRejectionReason::NoConvergence {
                last_volatility: last
            })
        );
    }

    #[test]
    fn test_locked_quote_too_small_for_its_spread_floor_has_no_weight() {
        // Its floor rounds to nothing and its spread is zero, so its weight has no value.
        let tick = Decimal::new(1, 27);
        let quote = OptionQuote {
            strike: pos(dec!(150)),
            ..call_at_mid(tick)
        };
        assert_eq!(rejection_of(quote), QuoteRejectionReason::NoWeight);
    }

    #[test]
    fn test_mid_that_overflows_costs_the_quote_and_not_the_surface() {
        let huge = Positive::new_decimal(Decimal::MAX).unwrap();
        let quote = OptionQuote {
            bid: huge,
            ask: huge,
            ..good_quote()
        };
        assert!(matches!(
            rejection_of(quote),
            QuoteRejectionReason::NumericalFailure { .. }
        ));
    }

    #[test]
    fn test_rejections_are_in_the_order_of_the_input_and_the_first_rule_wins() {
        let good = good_quote();
        let later_and_crossed = OptionQuote {
            quoted_at: as_of() + seconds(1),
            bid: good.ask,
            ask: good.bid,
            ..good
        };
        let crossed_and_stale = OptionQuote {
            quoted_at: as_of() - seconds(1_000_000),
            bid: good.ask,
            ask: good.bid,
            ..good
        };
        let quotes = [
            vec![crossed_and_stale],
            chain(&known(365), 365),
            vec![later_and_crossed],
        ]
        .concat();
        let surface = calibrate(&quotes).unwrap();
        let reasons: Vec<_> = surface
            .rejections()
            .iter()
            .map(|r| r.reason.clone())
            .collect();
        assert_eq!(
            reasons,
            vec![
                QuoteRejectionReason::Crossed,
                QuoteRejectionReason::LaterThanAsOf
            ]
        );
    }

    // A quote later than the as-of time.

    #[test]
    fn test_later_quotes_cannot_change_the_surface_in_any_figure() {
        let quotes = quotes_of(&[91, 365]);
        let base = calibrate(&quotes).unwrap();

        let later = as_of() + seconds(1);
        // The same instruments at other prices, which would supersede them; a crossed quote;
        // a whole new expiry that would fit; and a few at a fitted expiry.
        let mut extra: Vec<OptionQuote> = quotes
            .iter()
            .map(|quote| OptionQuote {
                bid: pos(quote.bid.to_dec() * dec!(1.2)),
                ask: pos(quote.ask.to_dec() * dec!(1.2)),
                quoted_at: later,
                ..*quote
            })
            .collect();
        extra.extend(
            chain(&known(182), 182)
                .into_iter()
                .map(|quote| OptionQuote {
                    quoted_at: later,
                    ..quote
                }),
        );
        extra.push(OptionQuote {
            bid: good_quote().ask,
            ask: good_quote().bid,
            quoted_at: later,
            ..good_quote()
        });
        let mixed = [extra.clone(), quotes].concat();
        let with = calibrate(&mixed).unwrap();

        // Everything but the list that says they were seen is identical, not close.
        let mut without_them = with.clone();
        without_them
            .rejections
            .retain(|rejection| rejection.reason != QuoteRejectionReason::LaterThanAsOf);
        assert_eq!(without_them, base);
        assert_eq!(with.rejections().len(), extra.len());
        assert!(with.unfitted().is_empty());
    }

    #[test]
    fn test_expiry_with_only_later_quotes_is_not_reported_as_unfitted() {
        let mut quotes = chain(&known(365), 365);
        let mut lone = good_quote();
        lone.expiry = in_days(91);
        lone.quoted_at = as_of() + seconds(1);
        quotes.push(lone);
        let surface = calibrate(&quotes).unwrap();
        assert!(surface.unfitted().is_empty());
        assert_eq!(surface.rejections().len(), 1);
    }

    /// Quotes with a rival for many of their instruments: an identical one, an older one, and
    /// one as late and as tight with a higher bid, so that `Screened::order` runs on every
    /// tie it has.
    fn quotes_with_rivals() -> Vec<OptionQuote> {
        let quotes = quotes_of(&[91, 365]);
        let mut all = quotes.clone();
        for (index, quote) in quotes.iter().enumerate() {
            all.push(match index % 4 {
                0 => *quote,
                1 => OptionQuote {
                    quoted_at: as_of() - seconds(10),
                    bid: pos(quote.bid.to_dec() * dec!(1.01)),
                    ask: pos(quote.ask.to_dec() * dec!(1.01)),
                    ..*quote
                },
                2 => OptionQuote {
                    bid: pos(quote.bid.to_dec() + dec!(0.001)),
                    ask: pos(quote.ask.to_dec() + dec!(0.001)),
                    ..*quote
                },
                _ => OptionQuote {
                    ask: pos(quote.ask.to_dec() * dec!(1.05)),
                    ..*quote
                },
            });
        }
        all
    }

    /// A Fisher-Yates shuffle on a fixed generator.
    fn shuffled(quotes: &[OptionQuote], seed: &mut u64) -> Vec<OptionQuote> {
        let mut shuffled = quotes.to_vec();
        for index in (1..shuffled.len()).rev() {
            *seed ^= *seed << 13;
            *seed ^= *seed >> 7;
            *seed ^= *seed << 17;
            let other = usize::try_from(*seed % (index as u64 + 1)).unwrap();
            shuffled.swap(index, other);
        }
        shuffled
    }

    #[test]
    fn test_rivals_of_one_instrument_leave_the_quote_each_tie_says() {
        let clean = quotes_of(&[91, 365]);
        let all = quotes_with_rivals();
        let surface = calibrate(&all).unwrap();
        assert_eq!(surface.rejections().len(), clean.len());
        assert!(
            surface
                .rejections()
                .iter()
                .all(|rejection| rejection.reason == QuoteRejectionReason::Superseded)
        );
        // In every tie the original stands, so the fit is that of the originals alone.
        assert_eq!(surface.expiries(), calibrate(&clean).unwrap().expiries());
    }

    #[test]
    fn test_calibrate_does_not_depend_on_the_order_of_the_quotes() {
        let all = quotes_with_rivals();
        let base = calibrate(&all).unwrap();
        let sorted_rejections = |surface: &ImpliedVolatilitySurface| {
            let mut rejections: Vec<String> = surface
                .rejections()
                .iter()
                .map(|rejection| format!("{rejection:?}"))
                .collect();
            rejections.sort();
            rejections
        };
        let mut seed = 0x9E37_79B9_7F4A_7C15_u64;
        for _ in 0..8 {
            let surface = calibrate(&shuffled(&all, &mut seed)).unwrap();
            // Exactly the same but for the order of the rejections, which is that of the input.
            assert_eq!(surface.expiries(), base.expiries());
            assert_eq!(surface.unfitted(), base.unfitted());
            assert_eq!(surface.arbitrage_report(), base.arbitrage_report());
            assert_eq!(surface.arbitrage_grid(), base.arbitrage_grid());
            assert_eq!(sorted_rejections(&surface), sorted_rejections(&base));
        }
    }

    #[test]
    fn test_calibrate_is_the_same_under_any_reference_datetime() {
        let quotes = quotes_of(&[91, 365]);
        let base = calibrate(&quotes).unwrap();
        ExpirationDate::set_reference_datetime(Some(as_of() + seconds(1_000_000)));
        let referenced = calibrate(&quotes);
        ExpirationDate::set_reference_datetime(None);
        assert_eq!(referenced.unwrap(), base);
    }

    // Weights.

    /// The Black-Scholes vega per unit of volatility, in closed form.
    fn vega(strike: f64, volatility: f64, t: f64) -> f64 {
        let (spot, rate, dividend) = (100.0_f64, 0.03_f64, 0.01_f64);
        let d1 = ((spot / strike).ln() + (rate - dividend + volatility * volatility / 2.0) * t)
            / (volatility * t.sqrt());
        spot * (-dividend * t).exp() * (-d1 * d1 / 2.0).exp() * t.sqrt()
            / (2.0 * std::f64::consts::PI).sqrt()
    }

    fn f(value: Decimal) -> f64 {
        optionstratlib_core::model::decimal::decimal_to_f64(value).unwrap()
    }

    #[test]
    fn test_weights_are_vega_over_volatility_and_spread_squared_relative_to_the_largest() {
        let surface = calibrate(&chain(&known(365), 365)).unwrap();
        let expiry = surface.expiries().first().unwrap();
        let spreads: Vec<f64> = chain(&known(365), 365)
            .iter()
            .map(|quote| f(quote.ask.to_dec() - quote.bid.to_dec()))
            .collect();
        let t = f(expiry.time_to_expiry().to_dec());
        let raws: Vec<f64> = expiry
            .residuals()
            .iter()
            .zip(&spreads)
            .map(|(residual, spread)| {
                let volatility = f(residual.observed_implied_volatility.to_dec());
                let vega = vega(f(residual.strike.to_dec()), volatility, t);
                vega / (volatility * spread)
            })
            .collect();
        let largest = raws.iter().cloned().fold(0.0_f64, f64::max);
        for (residual, raw) in expiry.residuals().iter().zip(&raws) {
            let expected = (raw / largest) * (raw / largest);
            assert!(
                (f(residual.weight.to_dec()) - expected).abs() < 1e-9,
                "{residual:?} against {expected}"
            );
        }
        assert!(expiry.residuals().iter().any(|r| r.weight == Positive::ONE));
    }

    fn raw_weight_at_spread(spread: Decimal) -> Decimal {
        let mid = dec!(10);
        let half = spread / dec!(2);
        let quote = OptionQuote {
            bid: pos(mid - half),
            ask: pos(mid + half),
            ..call_at_mid(mid)
        };
        let time = time_to_expiry(as_of(), quote.expiry).unwrap().unwrap();
        extract_quote(
            &Screened {
                index: 0,
                quote,
                spread,
                time,
            },
            &market(),
        )
        .unwrap()
        .raw_weight
    }

    #[test]
    fn test_spread_below_the_floor_weighs_as_the_floor_and_above_it_as_itself() {
        // 0.1% of a mid of 10 is 0.01.
        let at_floor = raw_weight_at_spread(dec!(0.01));
        assert_eq!(raw_weight_at_spread(dec!(0)), at_floor);
        assert_eq!(raw_weight_at_spread(dec!(0.005)), at_floor);
        assert_close(
            raw_weight_at_spread(dec!(0.02)) * dec!(2),
            at_floor,
            dec!(0.000000000001),
        );
    }

    #[test]
    fn test_zero_weight_quote_keeps_its_residual_and_leaves_the_range() {
        let clean = [
            (dec!(90), dec!(1)),
            (dec!(95), dec!(1)),
            (dec!(100), dec!(1)),
            (dec!(105), dec!(1)),
            (dec!(110), dec!(1)),
            (dec!(200), dec!(0)),
        ];
        let slice = known(365);
        let quotes: Vec<QuoteInFit> = clean
            .iter()
            .map(|(strike, weight)| QuoteInFit {
                strike: pos(*strike),
                style: OptionStyle::Call,
                volatility: slice.implied_volatility_at_strike(pos(*strike)).unwrap(),
                weight: pos(*weight),
                bid: Positive::ONE,
                ask: Positive::TWO,
            })
            .collect();
        let expiry = SurfaceExpiry::from_slice(in_days(365), slice, &quotes, &market()).unwrap();
        assert_eq!(expiry.residuals().len(), 6);
        let widest = slice.log_moneyness(pos(dec!(110))).unwrap();
        assert_eq!(expiry.highest_log_moneyness(), widest);
    }

    // Figures of a fit.

    #[test]
    fn test_rmse_and_baseline_match_their_closed_forms() {
        // A flat slice at 20% against quotes of known volatility and weight.
        let slice = SviSlice::try_new(
            SviParameters::try_new(dec!(0.04), dec!(0), dec!(0), dec!(0), dec!(1)).unwrap(),
            Positive::HUNDRED,
            Positive::ONE,
        )
        .unwrap();
        let quote = |strike, volatility, weight| QuoteInFit {
            strike: pos(strike),
            style: OptionStyle::Call,
            volatility: pos(volatility),
            weight: pos(weight),
            bid: Positive::ONE,
            ask: Positive::TWO,
        };
        let quotes = [
            quote(dec!(90), dec!(0.1), dec!(1)),
            quote(dec!(110), dec!(0.3), dec!(0.25)),
        ];
        let expiry = SurfaceExpiry::from_slice(in_days(365), slice, &quotes, &market()).unwrap();
        // Residuals of +0.1 and -0.1 under any weights have an rmse of 0.1.
        assert_close(expiry.rmse().to_dec(), dec!(0.1), dec!(0.0000000000001));
        // The weighted mean volatility is (0.1 + 0.075) / 1.25 = 0.14.
        assert_close(
            expiry.relative_rmse().to_dec(),
            dec!(0.1) / dec!(0.14),
            dec!(0.0000000000001),
        );
        // sqrt((0.01 + 0.0225) / 1.25) = sqrt(0.026) = 0.16124515...
        let flat = d_sqrt(dec!(0.026), "test").unwrap();
        let baseline = expiry.baseline();
        assert_close(
            baseline.baseline_implied_volatility.to_dec(),
            flat,
            dec!(0.0000000000001),
        );
        let misses = (flat - dec!(0.1)) * (flat - dec!(0.1))
            + dec!(0.25) * (flat - dec!(0.3)) * (flat - dec!(0.3));
        assert_close(
            baseline.baseline_rmse.to_dec(),
            d_sqrt(misses / dec!(1.25), "test").unwrap(),
            dec!(0.0000000000001),
        );
        assert_eq!(baseline.surface_rmse, expiry.rmse());
        assert_eq!(
            baseline.improvement(),
            baseline.baseline_rmse.to_dec() - expiry.rmse().to_dec()
        );
        assert_eq!(
            expiry.lowest_log_moneyness(),
            slice.log_moneyness(pos(dec!(90))).unwrap()
        );
    }

    #[test]
    fn test_unweighted_rmse_ignores_the_weights() {
        let slice = SviSlice::try_new(
            SviParameters::try_new(dec!(0.04), dec!(0), dec!(0), dec!(0), dec!(1)).unwrap(),
            Positive::HUNDRED,
            Positive::ONE,
        )
        .unwrap();
        let quote = |strike, volatility, weight| QuoteInFit {
            strike: pos(strike),
            style: OptionStyle::Call,
            volatility: pos(volatility),
            weight: pos(weight),
            bid: Positive::ONE,
            ask: Positive::TWO,
        };
        // Residuals of 0, 0.1 and 0.2, the largest with no weight at all.
        let quotes = [
            quote(dec!(90), dec!(0.2), dec!(1)),
            quote(dec!(100), dec!(0.1), dec!(0.5)),
            quote(dec!(110), dec!(0.4), dec!(0)),
        ];
        let expiry = SurfaceExpiry::from_slice(in_days(365), slice, &quotes, &market()).unwrap();
        // sqrt((0 + 0.01 + 0.04) / 3).
        assert_close(
            expiry.unweighted_rmse().to_dec(),
            d_sqrt(dec!(0.05) / dec!(3), "test").unwrap(),
            dec!(0.0000000000001),
        );
        // sqrt(0.5 * 0.01 / 1.5).
        assert_close(
            expiry.rmse().to_dec(),
            d_sqrt(dec!(0.005) / dec!(1.5), "test").unwrap(),
            dec!(0.0000000000001),
        );
    }

    /// The price of the residual's own option at its fitted volatility, in closed form.
    fn closed_form_price(residual: &SurfaceResidual) -> Decimal {
        price(
            residual.strike,
            residual.style,
            pos(residual.time_to_expiry.to_dec() * dec!(365)),
            residual.fitted_implied_volatility,
        )
    }

    #[test]
    fn test_fitted_price_is_black_scholes_at_the_fitted_volatility() {
        let surface = calibrate(&quotes_of(&[365])).unwrap();
        for residual in surface.residuals() {
            assert_close(
                residual.fitted_price,
                closed_form_price(residual),
                dec!(0.00000000001),
            );
            assert!(!residual.outside_bid_ask, "{residual:?}");
        }
        let expiry = surface.expiries().first().unwrap();
        assert_eq!(expiry.quotes_outside_bid_ask(), 0);
        assert!(expiry.unweighted_rmse().to_dec() < dec!(0.00000001));
    }

    #[test]
    fn test_price_below_the_bid_or_above_the_ask_is_outside_and_at_either_is_not() {
        let slice = known(365);
        let strike = pos(dec!(100));
        let volatility = slice.implied_volatility_at_strike(strike).unwrap();
        let fitted = price(strike, OptionStyle::Call, pos(dec!(365)), volatility);
        let cent = dec!(0.01);
        let cases = [
            (fitted + cent, fitted + dec!(2) * cent, true),
            (fitted - dec!(2) * cent, fitted - cent, true),
            (fitted, fitted + cent, false),
            (fitted - cent, fitted, false),
        ];
        for (bid, ask, outside) in cases {
            let quotes = [QuoteInFit {
                strike,
                style: OptionStyle::Call,
                volatility,
                weight: Positive::ONE,
                bid: pos(bid),
                ask: pos(ask),
            }];
            let expiry =
                SurfaceExpiry::from_slice(in_days(365), slice, &quotes, &market()).unwrap();
            let residual = expiry.residuals().first().unwrap();
            assert_eq!(
                residual.outside_bid_ask, outside,
                "{bid} {ask}: {residual:?}"
            );
            assert_eq!(expiry.quotes_outside_bid_ask(), usize::from(outside));
        }
    }

    /// A frown with the same volatilities as `frown`, the centre `centre` either side and the
    /// rest `wing` either side, around their closed form price.
    fn frown_with_spreads(days: i64, centre: Decimal, wing: Decimal) -> Vec<OptionQuote> {
        (-4_i32..=4)
            .map(|step| {
                let strike = pos(dec!(100) + Decimal::from(step) * dec!(2));
                let style = if step >= 0 {
                    OptionStyle::Call
                } else {
                    OptionStyle::Put
                };
                let volatility = pos(dec!(0.5) - dec!(0.05) * Decimal::from(step.abs()));
                let mid = price(strike, style, pos(Decimal::from(days)), volatility);
                let half = if step == 0 { centre } else { wing };
                OptionQuote {
                    expiry: in_days(days),
                    strike,
                    style,
                    bid: pos(mid * (Decimal::ONE - half)),
                    ask: pos(mid * (Decimal::ONE + half)),
                    quoted_at: as_of(),
                }
            })
            .collect()
    }

    #[test]
    fn test_frown_with_a_tight_centre_fits_within_the_limit_and_the_figures_show_it() {
        // Even spreads: the limit sees it.
        let even = frown_with_spreads(182, dec!(0.01), dec!(0.01));
        assert!(matches!(
            calibrate(&even),
            Err(ImpliedVolatilitySurfaceError::NoFittedExpiry { .. })
        ));

        // A tight centre and wide wings: the weights hide it from the limit.
        let quotes = frown_with_spreads(182, dec!(0.0005), dec!(0.3));
        let surface = calibrate(&quotes).unwrap();
        let expiry = surface.expiries().first().unwrap();
        assert!(expiry.relative_rmse().to_dec() < dec!(0.03), "{expiry:?}");
        let worst = expiry
            .residuals()
            .iter()
            .map(|residual| residual.residual.abs())
            .max()
            .unwrap();
        assert!(worst > dec!(0.2), "{worst}");

        // Against the same residuals, the figures that ignore the weights.
        assert!(expiry.unweighted_rmse().to_dec() > dec!(0.1), "{expiry:?}");
        assert!(expiry.unweighted_rmse().to_dec() > dec!(10) * expiry.rmse().to_dec());
        assert!(expiry.quotes_outside_bid_ask() >= 5, "{expiry:?}");
        for residual in expiry.residuals() {
            assert_eq!(
                residual.outside_bid_ask,
                residual.fitted_price < residual.bid.to_dec()
                    || residual.fitted_price > residual.ask.to_dec()
            );
        }

        // They gate nothing: the expiry is fitted, and the report does not read them.
        assert_eq!(surface.expiries().len(), 1);
        assert!(surface.unfitted().is_empty());
        assert!(surface.arbitrage_report().passed());
    }

    #[test]
    fn test_noise_limit_is_relative_to_the_level_of_volatility() {
        // One noise of 0.8 volatility points on the same smile shape at 20% and at 60%: the
        // limit is on a ratio to the level, so it is lost at one and not at the other.
        let noisy = |variance: Decimal| -> Vec<OptionQuote> {
            let mut seed = 42_u64;
            [30_i64, 91, 182, 365, 730]
                .iter()
                .flat_map(|days| {
                    let slice = known_scaled(*days, variance);
                    let forward = slice.forward().to_dec();
                    let width = d_sqrt(years(*days).to_dec() * variance / dec!(0.04), "test")
                        .unwrap()
                        * dec!(0.1);
                    (-7_i32..=7)
                        .map(|step| {
                            let k = width * Decimal::from(step) / dec!(3.5);
                            let strike = (forward * d_exp(k, "test").unwrap()).round_dp(2);
                            let level = slice.implied_volatility_at_strike(pos(strike)).unwrap();
                            let noise: f64 = (0..12)
                                .map(|_| {
                                    seed = seed
                                        .wrapping_mul(6_364_136_223_846_793_005)
                                        .wrapping_add(1_442_695_040_888_963_407);
                                    f64::from((seed >> 33) as u32) / 2_147_483_648.0 - 0.5
                                })
                                .sum();
                            let bumped = level.to_dec() + Decimal::try_from(0.008 * noise).unwrap();
                            let style = if strike >= forward {
                                OptionStyle::Call
                            } else {
                                OptionStyle::Put
                            };
                            quote_at_volatility(*days, strike, style, pos(bumped))
                        })
                        .collect::<Vec<_>>()
                })
                .collect()
        };
        let low = calibrate(&noisy(dec!(0.04))).unwrap();
        assert!(!low.unfitted().is_empty());
        let high = calibrate(&noisy(dec!(0.36))).unwrap();
        assert!(high.unfitted().is_empty(), "{:?}", high.unfitted());
    }

    #[test]
    fn test_baseline_loses_to_a_slice_on_a_smile_and_ties_on_flat_quotes() {
        let surface = calibrate(&chain(&known(365), 365)).unwrap();
        let comparison = surface.baseline_comparison().next().unwrap();
        assert!(comparison.improvement() > dec!(0.01), "{comparison:?}");

        let flat = SviSlice::try_new(
            SviParameters::try_new(dec!(0.04), dec!(0), dec!(0), dec!(0), dec!(1)).unwrap(),
            known(365).forward(),
            years(365),
        )
        .unwrap();
        let surface = calibrate(&chain(&flat, 365)).unwrap();
        let comparison = surface.baseline_comparison().next().unwrap();
        assert!(
            comparison.improvement().abs() < dec!(0.0000001),
            "{comparison:?}"
        );
    }

    #[test]
    fn test_residuals_are_by_expiry_and_strike_and_count_every_clean_quote() {
        let quotes = quotes_of(&[91, 365]);
        let surface = calibrate(&quotes).unwrap();
        assert_eq!(surface.residuals().count(), quotes.len());
        let keys: Vec<_> = surface
            .residuals()
            .map(|residual| (residual.expiry, residual.strike))
            .collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted);
        for expiry in surface.expiries() {
            for residual in expiry.residuals() {
                assert_eq!(residual.expiry, expiry.expiry());
                assert_eq!(residual.time_to_expiry, expiry.time_to_expiry());
                assert_eq!(
                    residual.residual,
                    residual.fitted_implied_volatility.to_dec()
                        - residual.observed_implied_volatility.to_dec()
                );
                assert_eq!(
                    residual.log_moneyness,
                    expiry.slice().log_moneyness(residual.strike).unwrap()
                );
            }
        }
    }

    // Reading the surface.

    fn two_expiries(config: &ImpliedVolatilitySurfaceConfig) -> ImpliedVolatilitySurface {
        calibrate_with(&quotes_of(&[91, 365]), config).unwrap()
    }

    #[test]
    fn test_point_at_a_fitted_expiry_is_its_slice() {
        let surface = two_expiries(&ImpliedVolatilitySurfaceConfig::default());
        let expiry = surface.expiries().first().unwrap();
        let k = dec!(0.02);
        let point = surface.point(k, expiry.time_to_expiry()).unwrap();
        assert_eq!(
            point.total_variance,
            expiry.slice().total_variance(k).unwrap()
        );
        assert_eq!(
            point.implied_volatility,
            expiry.slice().implied_volatility(k).unwrap()
        );
        assert_eq!(point.forward, expiry.forward());
        assert!(!point.is_extrapolated());
        assert!(!point.bridges_unfitted_expiry);
        assert_eq!(point.expiry_region, SurfaceExpiryRegion::WithinExpiries);
    }

    #[test]
    fn test_point_between_expiries_is_linear_in_total_variance_at_fixed_k() {
        let surface = two_expiries(&ImpliedVolatilitySurfaceConfig::default());
        let [earlier, later] = surface.expiries() else {
            panic!("two expiries");
        };
        let (t1, t2) = (
            earlier.time_to_expiry().to_dec(),
            later.time_to_expiry().to_dec(),
        );
        let k = dec!(0.03);
        // A quarter of the way, so that a formula swapping the two ends gets it wrong.
        let t = t1 + (t2 - t1) / dec!(4);
        let point = surface.point(k, pos(t)).unwrap();
        let (w1, w2) = (
            earlier.slice().total_variance(k).unwrap().to_dec(),
            later.slice().total_variance(k).unwrap().to_dec(),
        );
        let expected = w1 * dec!(0.75) + w2 * dec!(0.25);
        assert_close(
            point.total_variance.to_dec(),
            expected,
            dec!(0.00000000000000000001),
        );
        assert_close(
            point.implied_volatility.to_dec(),
            d_sqrt(expected / t, "test").unwrap(),
            dec!(0.00000000000000000001),
        );
        assert_eq!(point.forward, market().forward(pos(t)).unwrap());
        assert!(!point.is_extrapolated());
    }

    #[test]
    fn test_point_at_strike_uses_the_forward_of_its_own_expiry() {
        let surface = two_expiries(&ImpliedVolatilitySurfaceConfig::default());
        let strike = pos(dec!(103));
        let expiry = in_days(200);
        let t = time_to_expiry(as_of(), expiry).unwrap().unwrap().years;
        let forward = market().forward(t).unwrap();
        let point = surface.point_at_strike(strike, expiry).unwrap();
        assert_eq!(point.log_moneyness, log_moneyness(strike, forward).unwrap());
        assert_eq!(point.time_to_expiry, t);
        assert_eq!(point.forward, forward);
    }

    #[test]
    fn test_forward_carries_the_rate_less_the_yield() {
        let forward = market().forward(Positive::TWO).unwrap();
        assert_close(
            forward.to_dec(),
            dec!(100) * d_exp(dec!(0.04), "test").unwrap(),
            dec!(0.0000000000000000001),
        );
    }

    #[test]
    fn test_time_within_the_resolution_of_a_fitted_expiry_reads_that_slice() {
        let surface = calibrate(&quotes_of(&[91, 182, 365])).unwrap();
        let k = dec!(0.01);
        let [first, middle, last] = surface.expiries() else {
            panic!("three expiries");
        };

        // Computed in `Decimal`, just below the `T` of the slice, which went through `f64`.
        let computed = dec!(91) / dec!(365);
        assert!(computed < first.time_to_expiry().to_dec());
        // Just above the last, at both ends of what a default policy would refuse.
        let above = last.time_to_expiry().to_dec() + dec!(0.00000000000000000001);
        // Close to the middle one, which is otherwise a read between two slices.
        let near_middle = middle.time_to_expiry().to_dec() * dec!(1.0000000000000005);
        for (time, expiry) in [(computed, first), (above, last), (near_middle, middle)] {
            let point = surface.point(k, pos(time)).unwrap();
            assert_eq!(point.time_to_expiry, expiry.time_to_expiry());
            assert_eq!(point.forward, expiry.forward());
            assert_eq!(
                point.total_variance,
                expiry.slice().total_variance(k).unwrap()
            );
            assert_eq!(
                point.implied_volatility,
                expiry.slice().implied_volatility(k).unwrap()
            );
            assert!(!point.is_extrapolated());
        }
    }

    /// Flat quotes of one expiry at a given volatility.
    fn flat_chain(expiry: DateTime<Utc>, days: i64, volatility: Decimal) -> Vec<OptionQuote> {
        [90, 95, 100, 105, 110, 115]
            .into_iter()
            .map(|strike| {
                let strike = Decimal::from(strike);
                OptionQuote {
                    expiry,
                    ..quote_at_volatility(days, strike, OptionStyle::Call, pos(volatility))
                }
            })
            .collect()
    }

    #[test]
    fn test_time_within_the_resolution_of_two_expiries_reads_the_nearer() {
        // Five nanoseconds apart, within `1e-15` of each other, at 20% and 40%.
        let earlier = in_days(91);
        let later = earlier + TimeDelta::nanoseconds(5);
        let quotes = [
            flat_chain(earlier, 91, dec!(0.2)),
            flat_chain(later, 91, dec!(0.4)),
        ]
        .concat();
        let surface = calibrate(&quotes).unwrap();
        let [first, last] = surface.expiries() else {
            panic!("two expiries: {:?}", surface.unfitted());
        };
        let (t1, t2) = (
            first.time_to_expiry().to_dec(),
            last.time_to_expiry().to_dec(),
        );
        let gap = t2 - t1;
        assert!(gap > dec!(0));
        assert!(gap <= t1 * TIME_TO_EXPIRY_RESOLUTION);

        let read = |time: Decimal| surface.point(dec!(0), pos(time)).unwrap();
        for (time, expiry) in [
            (t1, first),
            (t2, last),
            (t1 + gap * dec!(0.01), first),
            (t1 + gap * dec!(0.49), first),
            (t2 - gap * dec!(0.49), last),
            (t2 - gap * dec!(0.01), last),
            // Exactly in the middle the earlier is as near.
            (t1 + gap / dec!(2), first),
        ] {
            let point = read(time);
            assert_eq!(point.time_to_expiry, expiry.time_to_expiry(), "{time}");
            assert_eq!(
                point.implied_volatility,
                expiry.slice().implied_volatility(dec!(0)).unwrap(),
                "{time}"
            );
        }
        assert_close(
            read(t2 - gap * dec!(0.01)).implied_volatility.to_dec(),
            dec!(0.4),
            dec!(0.000001),
        );
        assert_close(
            read(t1 + gap * dec!(0.01)).implied_volatility.to_dec(),
            dec!(0.2),
            dec!(0.000001),
        );
    }

    #[test]
    fn test_time_really_outside_the_fitted_expiries_still_errors() {
        let surface = calibrate(&quotes_of(&[91, 365])).unwrap();
        let [first, last] = surface.expiries() else {
            panic!("two expiries");
        };
        let before = first.time_to_expiry().to_dec() * dec!(0.99999999999999);
        assert!(matches!(
            surface.point(dec!(0), pos(before)),
            Err(ImpliedVolatilitySurfaceError::ShortExpiryOutOfPolicy { .. })
        ));
        let after = last.time_to_expiry().to_dec() * dec!(1.00000000000001);
        assert!(matches!(
            surface.point(dec!(0), pos(after)),
            Err(ImpliedVolatilitySurfaceError::LongExpiryOutOfPolicy { .. })
        ));
        // The same distance inside a gap is a read between two slices at its own time.
        let inside = pos(first.time_to_expiry().to_dec() * dec!(1.00000000000001));
        assert_eq!(
            surface.point(dec!(0), inside).unwrap().time_to_expiry,
            inside
        );
    }

    #[test]
    fn test_read_beyond_the_wings_of_the_default_policy_is_a_typed_error() {
        let surface = two_expiries(&ImpliedVolatilitySurfaceConfig::default());
        let expiry = surface.expiries().last().unwrap();
        let t = expiry.time_to_expiry();
        let beyond = expiry.highest_log_moneyness() + dec!(0.0001);
        match surface.point(beyond, t).unwrap_err() {
            ImpliedVolatilitySurfaceError::WingOutOfPolicy {
                extension, limit, ..
            } => {
                assert_close(extension.to_dec(), dec!(0.0001), dec!(0.0000000000001));
                assert_eq!(limit, Positive::ZERO);
            }
            other => panic!("{other:?}"),
        }
        let below = expiry.lowest_log_moneyness() - dec!(0.0001);
        assert!(matches!(
            surface.point(below, t),
            Err(ImpliedVolatilitySurfaceError::WingOutOfPolicy { .. })
        ));
        // Exactly at the last quote is inside the data.
        let at = surface.point(expiry.highest_log_moneyness(), t).unwrap();
        assert!(!at.is_extrapolated());
    }

    #[test]
    fn test_read_between_expiries_must_be_inside_the_quotes_of_both() {
        let surface = two_expiries(&ImpliedVolatilitySurfaceConfig::default());
        let [earlier, later] = surface.expiries() else {
            panic!("two expiries");
        };
        // Inside the quotes of the long expiry, and outside those of the short one.
        let k = earlier.highest_log_moneyness() + dec!(0.01);
        assert!(k < later.highest_log_moneyness());
        let t = pos(dec!(0.5));
        match surface.point(k, t).unwrap_err() {
            ImpliedVolatilitySurfaceError::WingOutOfPolicy { highest, .. } => {
                assert_eq!(highest, earlier.highest_log_moneyness());
            }
            other => panic!("{other:?}"),
        }
        assert!(
            !surface
                .point(k, later.time_to_expiry())
                .unwrap()
                .is_extrapolated()
        );
    }

    /// A wide short expiry and a long one with only its five middle strikes, so the later
    /// slice is the one with the narrower quotes.
    fn narrower_later_expiry(config: &ImpliedVolatilitySurfaceConfig) -> ImpliedVolatilitySurface {
        let middle: Vec<OptionQuote> = chain(&known(365), 365)
            .into_iter()
            .skip(5)
            .take(5)
            .collect();
        calibrate_with(&[chain(&known(182), 182), middle].concat(), config).unwrap()
    }

    #[test]
    fn test_wing_extension_between_expiries_is_measured_from_the_later_when_it_is_further() {
        let surface = narrower_later_expiry(&ImpliedVolatilitySurfaceConfig::default());
        let [earlier, later] = surface.expiries() else {
            panic!("two expiries");
        };
        let k = later.highest_log_moneyness() + dec!(0.005);
        assert!(k < earlier.highest_log_moneyness());
        let t = pos(dec!(0.75));
        match surface.point(k, t).unwrap_err() {
            ImpliedVolatilitySurfaceError::WingOutOfPolicy {
                extension, highest, ..
            } => {
                assert_eq!(highest, later.highest_log_moneyness());
                assert_close(extension.to_dec(), dec!(0.005), dec!(0.0000000000001));
            }
            other => panic!("{other:?}"),
        }
        let flagged = narrower_later_expiry(&flexible());
        let point = flagged.point(k, t).unwrap();
        assert_close(
            point.wing_extension.to_dec(),
            dec!(0.005),
            dec!(0.0000000000001),
        );
    }

    #[test]
    fn test_read_before_the_first_or_after_the_last_expiry_of_the_default_policy_is_an_error() {
        let surface = two_expiries(&ImpliedVolatilitySurfaceConfig::default());
        assert!(matches!(
            surface.point(dec!(0), pos(dec!(0.1))),
            Err(ImpliedVolatilitySurfaceError::ShortExpiryOutOfPolicy { .. })
        ));
        assert!(matches!(
            surface.point(dec!(0), Positive::TWO),
            Err(ImpliedVolatilitySurfaceError::LongExpiryOutOfPolicy { .. })
        ));
    }

    #[test]
    fn test_read_within_the_policy_is_flagged_and_beyond_it_is_an_error() {
        let surface = two_expiries(&flexible());
        let [first, last] = surface.expiries() else {
            panic!("two expiries");
        };
        let (t1, tn) = (
            first.time_to_expiry().to_dec(),
            last.time_to_expiry().to_dec(),
        );

        // Short: flat volatility at fixed k.
        let t = pos(t1 - dec!(0.05));
        let point = surface.point(dec!(0.01), t).unwrap();
        assert!(point.is_extrapolated());
        assert_eq!(point.expiry_region, SurfaceExpiryRegion::BeforeFirstExpiry);
        assert_close(
            point.implied_volatility.to_dec(),
            first
                .slice()
                .implied_volatility(dec!(0.01))
                .unwrap()
                .to_dec(),
            dec!(0.00000000000000000001),
        );
        assert!(matches!(
            surface.point(dec!(0.01), pos(t1 - dec!(0.0500001))),
            Err(ImpliedVolatilitySurfaceError::ShortExpiryOutOfPolicy { .. })
        ));

        // Long: the same.
        let t = pos(tn + dec!(1));
        let point = surface.point(dec!(0.01), t).unwrap();
        assert!(point.is_extrapolated());
        assert_eq!(point.expiry_region, SurfaceExpiryRegion::AfterLastExpiry);
        assert_close(
            point.total_variance.to_dec(),
            last.slice().total_variance(dec!(0.01)).unwrap().to_dec() * (tn + dec!(1)) / tn,
            dec!(0.00000000000000000001),
        );
        assert!(matches!(
            surface.point(dec!(0.01), pos(tn + dec!(1.0000001))),
            Err(ImpliedVolatilitySurfaceError::LongExpiryOutOfPolicy { .. })
        ));

        // Wings: the raw SVI formula.
        let k = last.highest_log_moneyness() + dec!(0.5);
        let point = surface.point(k, last.time_to_expiry()).unwrap();
        assert!(point.is_extrapolated());
        assert_close(
            point.wing_extension.to_dec(),
            dec!(0.5),
            dec!(0.0000000000001),
        );
        assert_eq!(
            point.total_variance,
            last.slice().total_variance(k).unwrap()
        );
        assert!(matches!(
            surface.point(k + dec!(0.0000001), last.time_to_expiry()),
            Err(ImpliedVolatilitySurfaceError::WingOutOfPolicy { .. })
        ));
    }

    #[test]
    fn test_read_before_first_expiry_keeps_total_variance_increasing() {
        let surface = two_expiries(&flexible());
        let first = surface.expiries().first().unwrap().time_to_expiry();
        let short = surface
            .point(dec!(0), pos(first.to_dec() - dec!(0.05)))
            .unwrap();
        let shorter = surface
            .point(dec!(0), pos(first.to_dec() - dec!(0.01)))
            .unwrap();
        assert!(short.total_variance < shorter.total_variance);
    }

    #[test]
    fn test_point_refuses_a_zero_time_and_an_expiry_not_after_as_of() {
        let surface = two_expiries(&flexible());
        assert!(matches!(
            surface.point(dec!(0), Positive::ZERO),
            Err(ImpliedVolatilitySurfaceError::ZeroTimeToExpiry)
        ));
        assert!(matches!(
            surface.point_at_strike(Positive::HUNDRED, as_of()),
            Err(ImpliedVolatilitySurfaceError::ExpiryNotAfterAsOf { .. })
        ));
        assert!(matches!(
            surface.point_at_strike(Positive::ZERO, in_days(365)),
            Err(ImpliedVolatilitySurfaceError::Svi(
                SviError::ZeroStrike { .. }
            ))
        ));
    }

    // An expiry that cannot be fitted.

    #[test]
    fn test_expiry_with_too_few_clean_quotes_is_reported_and_the_rest_is_built() {
        let mut quotes = chain(&known(365), 365);
        quotes.extend(chain(&known(91), 91).into_iter().take(4));
        let surface = calibrate(&quotes).unwrap();
        assert_eq!(surface.expiries().len(), 1);
        assert_eq!(
            surface.unfitted(),
            [UnfittedExpiry {
                expiry: in_days(91),
                time_to_expiry: Some(years(91)),
                clean_quotes: 4,
                reason: UnfittedReason::TooFewCleanQuotes {
                    clean: 4,
                    required: 5
                },
            }]
        );
    }

    #[test]
    fn test_expiry_whose_quotes_all_fail_is_reported_with_none_clean() {
        let mut quotes = chain(&known(365), 365);
        let good = good_quote();
        quotes.extend((0..3).map(|step| OptionQuote {
            expiry: in_days(91),
            strike: pos(Decimal::from(90 + step)),
            bid: good.ask,
            ask: good.bid,
            ..good
        }));
        let surface = calibrate(&quotes).unwrap();
        assert_eq!(surface.rejections().len(), 3);
        assert_eq!(
            surface.unfitted().first().unwrap().reason,
            UnfittedReason::TooFewCleanQuotes {
                clean: 0,
                required: 5
            }
        );
    }

    #[test]
    fn test_expired_expiry_is_reported_without_a_time_to_expiry() {
        let mut quotes = chain(&known(365), 365);
        quotes.push(OptionQuote {
            expiry: as_of() - seconds(1),
            ..good_quote()
        });
        let surface = calibrate(&quotes).unwrap();
        let unfitted = surface.unfitted().first().unwrap();
        assert_eq!(unfitted.time_to_expiry, None);
        assert_eq!(unfitted.clean_quotes, 0);
        assert_eq!(unfitted.reason, UnfittedReason::ExpiredAtAsOf);
    }

    #[test]
    fn test_expiry_with_too_few_distinct_strikes_is_reported() {
        let mut quotes = chain(&known(365), 365);
        let slice = known(91);
        for strike in [dec!(95), dec!(98), dec!(102), dec!(105)] {
            for style in [OptionStyle::Call, OptionStyle::Put] {
                quotes.push(quote_of(&slice, 91, strike, style));
            }
        }
        let surface = calibrate(&quotes).unwrap();
        assert_eq!(
            surface.unfitted().first().unwrap().reason,
            UnfittedReason::TooFewDistinctStrikes {
                distinct: 4,
                required: 5
            }
        );
    }

    /// A frown: lowest in the wings, which no raw SVI slice can fit.
    fn frown(days: i64) -> Vec<OptionQuote> {
        (-4_i32..=4)
            .map(|step| {
                let distance = Decimal::from(step.abs());
                let volatility = pos(dec!(0.5) - dec!(0.05) * distance);
                quote_at_volatility(
                    days,
                    dec!(100) + Decimal::from(step) * dec!(2),
                    OptionStyle::Call,
                    volatility,
                )
            })
            .collect()
    }

    #[test]
    fn test_frown_expiry_is_unfitted_with_its_rmse_and_a_loose_limit_admits_it() {
        let quotes = [quotes_of(&[91, 365]), frown(182)].concat();
        let surface = calibrate(&quotes).unwrap();
        assert_eq!(surface.expiries().len(), 2);
        let unfitted = surface.unfitted().first().unwrap();
        assert_eq!(unfitted.expiry, in_days(182));
        let UnfittedReason::RmseAboveLimit {
            relative_rmse,
            limit,
            ..
        } = unfitted.reason
        else {
            panic!("{:?}", unfitted.reason);
        };
        assert!(relative_rmse > limit);
        assert_eq!(limit, SviCalibrationConfig::default().max_relative_rmse);

        // The caller sets the limit, and a fit within it comes back flat.
        let mut loose = ImpliedVolatilitySurfaceConfig::default();
        loose.svi.max_relative_rmse = Positive::ONE;
        let surface = calibrate_with(&quotes, &loose).unwrap();
        assert_eq!(surface.expiries().len(), 3);
        assert!(surface.unfitted().is_empty());
    }

    #[test]
    fn test_point_bridging_an_unfitted_expiry_says_so() {
        let quotes = [quotes_of(&[91, 365]), frown(182)].concat();
        let surface = calibrate(&quotes).unwrap();
        let t = years(182);
        let point = surface.point(dec!(0), t).unwrap();
        assert!(point.bridges_unfitted_expiry);
        assert!(!point.is_extrapolated());

        // Not at the fitted expiries, and not across a gap without one.
        let at = surface.point(dec!(0), years(91)).unwrap();
        assert!(!at.bridges_unfitted_expiry);
        let clean = calibrate(&quotes_of(&[91, 365])).unwrap();
        assert!(!clean.point(dec!(0), t).unwrap().bridges_unfitted_expiry);

        // Nor when the unfitted one is outside this gap.
        let outside = calibrate(&[quotes_of(&[91, 182]), frown(365)].concat()).unwrap();
        let gap = pos((years(91).to_dec() + years(182).to_dec()) / dec!(2));
        assert!(!outside.point(dec!(0), gap).unwrap().bridges_unfitted_expiry);
    }

    #[test]
    fn test_unfitted_expiry_sharing_the_time_of_a_fitted_one_is_not_bridged() {
        // One nanosecond after each fitted expiry, below the resolution of their `T`: such
        // an unfitted expiry sits at an end of the gap and not inside it.
        let near = |days: i64| {
            chain(&known(days), days)
                .into_iter()
                .take(3)
                .map(move |quote| OptionQuote {
                    expiry: in_days(days) + TimeDelta::nanoseconds(1),
                    ..quote
                })
        };
        let quotes: Vec<OptionQuote> = quotes_of(&[91, 365])
            .into_iter()
            .chain(near(91))
            .chain(near(365))
            .collect();
        let surface = calibrate(&quotes).unwrap();
        let [first, second] = surface.unfitted() else {
            panic!("two unfitted: {:?}", surface.unfitted());
        };
        assert_eq!(first.time_to_expiry, Some(years(91)));
        assert_eq!(second.time_to_expiry, Some(years(365)));
        let point = surface.point(dec!(0), pos(dec!(0.5))).unwrap();
        assert!(!point.bridges_unfitted_expiry);
    }

    #[test]
    fn test_no_expiry_fitted_is_an_error_carrying_both_lists() {
        let good = good_quote();
        let crossed = OptionQuote {
            bid: good.ask,
            ask: good.bid,
            ..good
        };
        match calibrate(&[crossed]).unwrap_err() {
            ImpliedVolatilitySurfaceError::NoFittedExpiry {
                unfitted,
                rejections,
            } => {
                assert_eq!(unfitted.len(), 1);
                assert_eq!(rejections.len(), 1);
                assert_eq!(
                    rejections.first().unwrap().reason,
                    QuoteRejectionReason::Crossed
                );
            }
            other => panic!("{other:?}"),
        }
        assert!(matches!(
            calibrate(&[]),
            Err(ImpliedVolatilitySurfaceError::NoFittedExpiry { .. })
        ));
    }

    #[test]
    fn test_option_with_no_volatility_at_a_quote_is_a_pricing_error_of_that_expiry() {
        // A zero minimum at `k = 0`, read at the strike there: a slice can be valid and have
        // a volatility of exactly zero at a quote, and an option has no price at that. A public
        // input does not reach it, since no fit or shock lands on a quote at the minimum with
        // no rounding, so this builds the expiry directly.
        let forward = market().forward(years(365)).unwrap();
        let slice = SviSlice::try_new(
            SviParameters::try_new(dec!(-0.01), dec!(0.1), dec!(0), dec!(0), dec!(0.1)).unwrap(),
            forward,
            years(365),
        )
        .unwrap();
        let quotes = [QuoteInFit {
            strike: forward,
            style: OptionStyle::Call,
            volatility: pos(dec!(0.2)),
            weight: Positive::ONE,
            bid: Positive::ONE,
            ask: Positive::TWO,
        }];
        let error = SurfaceExpiry::from_slice(in_days(365), slice, &quotes, &market()).unwrap_err();
        assert!(
            matches!(error, ImpliedVolatilitySurfaceError::Pricing(_)),
            "{error:?}"
        );
        // In a calibration that costs the expiry, not the surface.
        match FitFailure::from(error) {
            FitFailure::Expiry(UnfittedReason::CalibrationFailed { reason }) => {
                assert!(!reason.is_empty());
            }
            FitFailure::Expiry(other) => panic!("{other:?}"),
            FitFailure::Surface(other) => panic!("{other:?}"),
        }
    }

    #[test]
    fn test_unfitted_reason_keeps_any_other_calibration_failure_as_its_message() {
        let reason = UnfittedReason::from(SviError::NoConvergence {
            iterations: 7,
            limit: 1,
        });
        match reason {
            UnfittedReason::CalibrationFailed { reason } => {
                assert!(reason.contains("did not converge"), "{reason}");
            }
            other => panic!("{other:?}"),
        }
    }

    // Configuration and inputs.

    #[test]
    fn test_calibrate_refuses_a_zero_spot() {
        assert!(matches!(
            ImpliedVolatilitySurface::calibrate(
                &quotes_of(&[365]),
                Positive::ZERO,
                RATE,
                pos(YIELD),
                as_of(),
                &ImpliedVolatilitySurfaceConfig::default(),
            ),
            Err(ImpliedVolatilitySurfaceError::ZeroSpot)
        ));
    }

    #[test]
    fn test_calibrate_refuses_a_negative_age_limit_and_takes_a_zero_one() {
        let mut config = ImpliedVolatilitySurfaceConfig {
            max_quote_age: TimeDelta::nanoseconds(-1),
            ..ImpliedVolatilitySurfaceConfig::default()
        };
        assert!(matches!(
            calibrate_with(&quotes_of(&[365]), &config),
            Err(ImpliedVolatilitySurfaceError::NegativeQuoteAgeLimit { .. })
        ));
        config.max_quote_age = TimeDelta::zero();
        assert!(calibrate_with(&quotes_of(&[365]), &config).is_ok());
    }

    #[test]
    fn test_calibrate_refuses_a_grid_below_the_minimum() {
        for points in [1, MIN_ARBITRAGE_GRID_POINTS - 1] {
            let config = ImpliedVolatilitySurfaceConfig {
                arbitrage_grid_points: NonZeroUsize::new(points).unwrap(),
                ..ImpliedVolatilitySurfaceConfig::default()
            };
            match calibrate_with(&quotes_of(&[365]), &config).unwrap_err() {
                ImpliedVolatilitySurfaceError::ArbitrageGridPointsOutOfRange {
                    points: found,
                    minimum,
                    maximum,
                } => {
                    assert_eq!(found, points);
                    assert_eq!(minimum, MIN_ARBITRAGE_GRID_POINTS);
                    assert_eq!(maximum, MAX_ARBITRAGE_GRID_POINTS);
                }
                other => panic!("{other:?}"),
            }
        }
        let config = ImpliedVolatilitySurfaceConfig {
            arbitrage_grid_points: NonZeroUsize::new(MIN_ARBITRAGE_GRID_POINTS).unwrap(),
            ..ImpliedVolatilitySurfaceConfig::default()
        };
        assert!(calibrate_with(&quotes_of(&[365]), &config).is_ok());
    }

    #[test]
    fn test_calibrate_refuses_a_grid_above_the_limit() {
        let config = ImpliedVolatilitySurfaceConfig {
            arbitrage_grid_points: NonZeroUsize::new(MAX_ARBITRAGE_GRID_POINTS + 1).unwrap(),
            ..ImpliedVolatilitySurfaceConfig::default()
        };
        assert!(matches!(
            calibrate_with(&quotes_of(&[365]), &config),
            Err(ImpliedVolatilitySurfaceError::ArbitrageGridPointsOutOfRange { .. })
        ));
    }

    #[test]
    fn test_grid_of_exactly_the_limit_is_allowed() {
        let config = ImpliedVolatilitySurfaceConfig {
            arbitrage_grid_points: NonZeroUsize::new(MAX_ARBITRAGE_GRID_POINTS).unwrap(),
            ..ImpliedVolatilitySurfaceConfig::default()
        };
        assert!(check_config(&config).is_ok());
    }

    #[test]
    fn test_calibrate_refuses_an_invalid_svi_configuration() {
        let mut config = ImpliedVolatilitySurfaceConfig::default();
        config.svi.tolerance = Positive::ONE;
        assert!(matches!(
            calibrate_with(&quotes_of(&[365]), &config),
            Err(ImpliedVolatilitySurfaceError::Svi(
                SviError::InvalidConfig { .. }
            ))
        ));
    }

    #[test]
    fn test_default_config_declares_its_numbers() {
        let config = ImpliedVolatilitySurfaceConfig::default();
        assert_eq!(config.max_quote_age, seconds(300));
        assert_eq!(config.svi, SviCalibrationConfig::default());
        assert_eq!(config.extrapolation, ExtrapolationPolicy::default());
        assert_eq!(config.extrapolation.max_wing_extension, Positive::ZERO);
        assert_eq!(config.arbitrage_grid_points.get(), 41);
    }

    // Accessors.

    #[test]
    fn test_surface_states_its_inputs() {
        let surface = two_expiries(&ImpliedVolatilitySurfaceConfig::default());
        assert_eq!(surface.as_of(), as_of());
        assert_eq!(surface.spot(), Positive::HUNDRED);
        assert_eq!(surface.rate(), RATE);
        assert_eq!(surface.dividend_yield(), pos(YIELD));
        assert_eq!(surface.shocks(), []);
    }

    #[test]
    fn test_age_at_measures_from_as_of_and_refuses_an_earlier_instant() {
        let surface = two_expiries(&ImpliedVolatilitySurfaceConfig::default());
        assert_eq!(surface.age_at(as_of()).unwrap(), TimeDelta::zero());
        assert_eq!(surface.age_at(as_of() + seconds(90)).unwrap(), seconds(90));
        assert!(matches!(
            surface.age_at(as_of() - TimeDelta::nanoseconds(1)),
            Err(ImpliedVolatilitySurfaceError::InstantBeforeAsOf { .. })
        ));
    }

    // Arbitrage report.

    #[test]
    fn test_grid_is_the_sorted_union_of_points_across_the_quotes_of_each_expiry() {
        let surface = two_expiries(&ImpliedVolatilitySurfaceConfig::default());
        let grid = surface.arbitrage_grid();
        let mut sorted = grid.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(grid, sorted);
        for expiry in surface.expiries() {
            let (lowest, highest) = (
                expiry.lowest_log_moneyness(),
                expiry.highest_log_moneyness(),
            );
            assert!(grid.contains(&lowest));
            assert!(grid.contains(&highest));
            let inside = grid
                .iter()
                .filter(|k| **k >= lowest && **k <= highest)
                .count();
            assert!(
                inside >= 41,
                "{inside} points inside the quotes of {expiry:?}"
            );
            for index in 0..=40 {
                let point = lowest + (highest - lowest) * Decimal::from(index) / dec!(40);
                let nearest = grid.iter().map(|k| (*k - point).abs()).min().unwrap();
                assert!(
                    nearest < dec!(0.000000000000000000001),
                    "{point}: {nearest}"
                );
            }
        }
    }

    #[test]
    fn test_grid_holds_every_clean_quote_with_no_repeat_and_exact_ends() {
        let surface = two_expiries(&ImpliedVolatilitySurfaceConfig::default());
        let grid = surface.arbitrage_grid();
        assert!(grid.windows(2).all(|pair| pair.first() < pair.last()));
        for residual in surface.residuals() {
            assert!(grid.contains(&residual.log_moneyness), "{residual:?}");
        }
        let lowest = surface.residuals().map(|r| r.log_moneyness).min();
        let highest = surface.residuals().map(|r| r.log_moneyness).max();
        assert_eq!(grid.first().copied(), lowest);
        assert_eq!(grid.last().copied(), highest);
    }

    #[test]
    fn test_grid_of_two_equal_expiries_has_one_copy_of_each_point() {
        let surface = calibrate(&quotes_of(&[365])).unwrap();
        let expiry = surface.expiries().first().unwrap().clone();
        // 41 evenly spaced points and 15 quotes, of which two are the ends.
        assert_eq!(surface.arbitrage_grid().len(), 41 + 15 - 2);
        let grid =
            union_across_quotes(&[expiry.clone(), expiry], NonZeroUsize::new(41).unwrap()).unwrap();
        assert_eq!(grid, surface.arbitrage_grid());
    }

    /// Dense quotes of a day-long expiry with a butterfly violation in them, and six far quotes
    /// that stretch its spacing to 0.015, beside a clean year-long expiry. Spaced points alone
    /// put five inside the dense quotes, step over the violation, and pass.
    #[test]
    fn test_butterfly_among_dense_quotes_stretched_by_far_ones_is_found() {
        let t = years(1);
        let short = SviSlice::try_new(
            SviParameters::try_new(
                dec!(0.00006660529139128362),
                dec!(0.013301456431887263),
                dec!(-0.4091625325933133),
                dec!(-0.0022509189316787075),
                dec!(0.00013376743036954395),
            )
            .unwrap(),
            market().forward(t).unwrap(),
            t,
        )
        .unwrap();
        let forward = short.forward().to_dec();
        let quote_at_k = |k: Decimal| {
            let strike = (forward * d_exp(k, "test").unwrap()).round_dp(4);
            let style = if strike >= forward {
                OptionStyle::Call
            } else {
                OptionStyle::Put
            };
            quote_of(&short, 1, strike, style)
        };
        let mut quotes: Vec<OptionQuote> = (-15_i32..=15)
            .map(|step| quote_at_k(dec!(0.002) * Decimal::from(step)))
            .collect();
        for k in [-0.3, -0.2, -0.1, 0.1, 0.2, 0.3] {
            quotes.push(quote_at_k(Decimal::try_from(k).unwrap()));
        }
        let long = known(365);
        let long_forward = long.forward().to_dec();
        quotes.extend((-20_i32..=20).map(|step| {
            let k = dec!(0.0375) * Decimal::from(step);
            let strike = (long_forward * d_exp(k, "test").unwrap()).round_dp(2);
            let style = if strike >= long_forward {
                OptionStyle::Call
            } else {
                OptionStyle::Put
            };
            quote_of(&long, 365, strike, style)
        }));

        let surface = calibrate(&quotes).unwrap();
        assert!(surface.unfitted().is_empty(), "{:?}", surface.unfitted());
        assert!(
            surface.rejections().is_empty(),
            "{:?}",
            surface.rejections()
        );
        let expiry = surface.expiries().first().unwrap();
        assert_eq!(expiry.time_to_expiry(), t);

        // Every violation of the slice at its own quotes is in the report.
        let at_quotes: Vec<Decimal> = expiry
            .residuals()
            .iter()
            .map(|residual| residual.log_moneyness)
            .collect();
        let own = check_butterfly(expiry.slice(), &at_quotes).unwrap();
        assert!(!own.violations().is_empty());
        let report = surface.arbitrage_report();
        assert!(!report.passed());
        for violation in own.violations() {
            assert!(report.violations().contains(violation), "{violation:?}");
        }
    }

    /// A slice a day from expiry with a butterfly violation inside quotes 0.03 wide, and a
    /// clean year-long slice with quotes 0.75 wide. One grid spaced by the long expiry puts
    /// one point inside the short quotes, at the money, and passes.
    #[test]
    fn test_butterfly_inside_the_narrow_quotes_of_a_short_expiry_is_found() {
        let bad = (
            dec!(0.00004075917963554839),
            dec!(0.023277104032127877),
            dec!(0.5928072869710899),
            dec!(0.0007647429485032747),
            dec!(0.001079933017846628),
        );
        let t = years(1);
        let short = SviSlice::try_new(
            SviParameters::try_new(bad.0, bad.1, bad.2, bad.3, bad.4).unwrap(),
            market().forward(t).unwrap(),
            t,
        )
        .unwrap();
        let forward = short.forward().to_dec();
        let mut quotes: Vec<OptionQuote> = (-15_i32..=15)
            .map(|step| {
                let k = dec!(0.002) * Decimal::from(step);
                let strike = (forward * d_exp(k, "test").unwrap()).round_dp(2);
                let style = if strike >= forward {
                    OptionStyle::Call
                } else {
                    OptionStyle::Put
                };
                quote_of(&short, 1, strike, style)
            })
            .collect();
        let long = known(365);
        let long_forward = long.forward().to_dec();
        quotes.extend((-20_i32..=20).map(|step| {
            let k = dec!(0.0375) * Decimal::from(step);
            let strike = (long_forward * d_exp(k, "test").unwrap()).round_dp(2);
            let style = if strike >= long_forward {
                OptionStyle::Call
            } else {
                OptionStyle::Put
            };
            quote_of(&long, 365, strike, style)
        }));

        let surface = calibrate(&quotes).unwrap();
        assert!(surface.unfitted().is_empty(), "{:?}", surface.unfitted());
        assert!(
            surface.rejections().is_empty(),
            "{:?}",
            surface.rejections()
        );
        let report = surface.arbitrage_report();
        assert!(!report.passed());
        let expiry = surface.expiries().first().unwrap();
        let (lowest, highest) = (
            expiry.lowest_log_moneyness(),
            expiry.highest_log_moneyness(),
        );
        let found = report
            .violations()
            .iter()
            .filter(|violation| {
                matches!(
                    violation,
                    crate::volatility::StaticArbitrageViolation::Butterfly { time_to_expiry, .. }
                        if *time_to_expiry == t
                ) && violation.log_moneyness() >= lowest
                    && violation.log_moneyness() <= highest
            })
            .count();
        assert!(
            found >= 10,
            "{found} violations inside the short quotes: {report:?}"
        );
    }

    #[test]
    fn test_known_surface_has_nothing_to_report() {
        let surface = calibrate(&quotes_of(&[30, 91, 182, 365, 730])).unwrap();
        let report = surface.arbitrage_report();
        assert!(report.passed(), "{report:?}");
        assert!(surface.calendar_checked());
    }

    #[test]
    fn test_one_expiry_has_no_calendar_check_and_says_so() {
        let surface = calibrate(&quotes_of(&[365])).unwrap();
        assert!(!surface.calendar_checked());
        assert!(surface.arbitrage_report().passed());
    }

    /// Vogt's slice of Example 3.1 of Gatheral and Jacquier, with butterfly arbitrage.
    fn vogt() -> SviSlice {
        SviSlice::try_new(
            SviParameters::try_new(
                dec!(-0.0410),
                dec!(0.1331),
                dec!(0.3060),
                dec!(0.3586),
                dec!(0.4153),
            )
            .unwrap(),
            market().forward(years(365)).unwrap(),
            years(365),
        )
        .unwrap()
    }

    #[test]
    fn test_butterfly_finding_is_reported_and_the_surface_still_returned() {
        let slice = vogt();
        let forward = slice.forward().to_dec();
        let quotes: Vec<OptionQuote> = (-4_i32..=14)
            .map(|step| {
                let k = dec!(0.1) * Decimal::from(step);
                let strike = (forward * d_exp(k, "test").unwrap()).round_dp(2);
                quote_of(&slice, 365, strike, OptionStyle::Call)
            })
            .collect();
        let surface = calibrate(&quotes).unwrap();
        assert!(
            surface.rejections().is_empty(),
            "{:?}",
            surface.rejections()
        );
        let report = surface.arbitrage_report();
        assert!(!report.passed());
        assert!(report.violations().iter().any(|violation| matches!(
            violation,
            crate::volatility::StaticArbitrageViolation::Butterfly { .. }
        )));
    }

    #[test]
    fn test_calendar_finding_is_reported_and_the_surface_still_returned() {
        // More total variance at the short expiry than at the long one.
        let short = known_scaled(91, dec!(0.5));
        let long = known_scaled(182, dec!(0.04));
        let quotes = [chain(&short, 91), chain(&long, 182)].concat();
        let surface = calibrate(&quotes).unwrap();
        assert_eq!(surface.expiries().len(), 2);
        let report = surface.arbitrage_report();
        assert!(!report.passed());
        assert!(report.violations().iter().any(|violation| matches!(
            violation,
            crate::volatility::StaticArbitrageViolation::Calendar { .. }
        )));
    }

    #[test]
    fn test_check_reads_the_slice_in_the_middle_of_each_gap() {
        let surface = two_expiries(&ImpliedVolatilitySurfaceConfig::default());
        let slices = slices_to_check(surface.expiries(), &market()).unwrap();
        assert_eq!(slices.len(), 3);
        let middle = slices.get(2).unwrap();
        let [first, last] = surface.expiries() else {
            panic!("two expiries");
        };
        let t = pos((first.time_to_expiry().to_dec() + last.time_to_expiry().to_dec()) / dec!(2));
        assert_eq!(middle.time_to_expiry(), t);
        assert_eq!(middle.forward(), market().forward(t).unwrap());
        let k = dec!(0.02);
        assert_eq!(
            TotalVarianceSlice::total_variance(middle, k).unwrap(),
            surface.point(k, t).unwrap().total_variance
        );
        let (s1, s2) = (first.slice(), last.slice());
        assert_eq!(
            middle.total_variance_slope(k).unwrap(),
            (s1.total_variance_slope(k).unwrap() + s2.total_variance_slope(k).unwrap()) / dec!(2)
        );
        assert_eq!(
            middle.total_variance_curvature(k).unwrap(),
            (s1.total_variance_curvature(k).unwrap() + s2.total_variance_curvature(k).unwrap())
                / dec!(2)
        );
        let rounding = TotalVarianceSlice::total_variance_rounding(middle, k).unwrap();
        let sides = TotalVarianceSlice::total_variance_rounding(s1, k)
            .unwrap()
            .to_dec()
            .max(
                TotalVarianceSlice::total_variance_rounding(s2, k)
                    .unwrap()
                    .to_dec(),
            );
        assert!(rounding.to_dec() > sides / dec!(2));
    }

    #[test]
    fn test_blended_slice_is_read_by_the_check_as_a_slice_of_its_own() {
        let middle = GridSlice {
            earlier: vogt(),
            later: vogt(),
            theta: dec!(0.5),
            time_to_expiry: years(365),
            forward: vogt().forward(),
        };
        let report = check_butterfly(&middle, &[dec!(0.9)]).unwrap();
        assert!(!report.violations().is_empty(), "{report:?}");
    }

    #[test]
    fn test_unevaluated_point_is_a_finding_and_not_a_pass() {
        // A zero minimum at k = 0, where g has no value.
        let slice = SviSlice::try_new(
            SviParameters::try_new(dec!(-0.01), dec!(0.1), dec!(0), dec!(0), dec!(0.1)).unwrap(),
            Positive::HUNDRED,
            Positive::ONE,
        )
        .unwrap();
        let quotes = [QuoteInFit {
            strike: pos(dec!(110)),
            style: OptionStyle::Call,
            volatility: pos(dec!(0.2)),
            weight: Positive::ONE,
            bid: Positive::ONE,
            ask: Positive::TWO,
        }];
        let expiry = SurfaceExpiry::from_slice(in_days(365), slice, &quotes, &market()).unwrap();
        let (report, calendar) = check_expiries(&[expiry], &market(), &[dec!(0)]).unwrap();
        assert!(!calendar);
        assert!(!report.unevaluated().is_empty());
        assert!(!report.passed());
    }

    #[test]
    fn test_check_on_grid_runs_on_the_caller_grid_and_reports_a_check_it_cannot_finish() {
        let surface = two_expiries(&ImpliedVolatilitySurfaceConfig::default());
        assert!(
            surface
                .check_on_grid(&[dec!(0), dec!(0.1)])
                .unwrap()
                .passed()
        );
        assert!(matches!(
            surface.check_on_grid(&[]),
            Err(ImpliedVolatilitySurfaceError::StaticArbitrage(
                StaticArbitrageError::EmptyGrid
            ))
        ));
        assert!(matches!(
            surface.check_on_grid(&[Decimal::MAX]),
            Err(ImpliedVolatilitySurfaceError::StaticArbitrage(_))
        ));
    }

    // Shocks.

    #[test]
    fn test_level_shock_moves_at_the_money_by_its_size_in_every_expiry() {
        let surface = two_expiries(&ImpliedVolatilitySurfaceConfig::default());
        let x = dec!(0.015);
        let shocked = surface
            .shock(SurfaceShock::Level { volatility: x })
            .unwrap();
        for (before, after) in surface.expiries().iter().zip(shocked.expiries()) {
            let atm = before.slice().implied_volatility(dec!(0)).unwrap().to_dec();
            assert_close(
                after.slice().implied_volatility(dec!(0)).unwrap().to_dec(),
                atm + x,
                dec!(0.00000000000000000001),
            );
            let (p, q) = (before.slice().parameters(), after.slice().parameters());
            assert_eq!(
                q.a(),
                p.a() + x * (dec!(2) * atm + x) * before.time_to_expiry().to_dec()
            );
            assert_eq!(
                (q.b(), q.rho(), q.m(), q.sigma()),
                (p.b(), p.rho(), p.m(), p.sigma())
            );
            // iv'(k)^2 = iv(k)^2 + x * (2 * iv_atm + x), away from the money too.
            let k = dec!(0.05);
            let iv = before.slice().implied_volatility(k).unwrap().to_dec();
            let iv_shocked = after.slice().implied_volatility(k).unwrap().to_dec();
            assert_close(
                iv_shocked * iv_shocked,
                iv * iv + x * (dec!(2) * atm + x),
                dec!(0.00000000000000000001),
            );
        }
    }

    #[test]
    fn test_skew_shock_adds_to_rho_and_nothing_else() {
        let surface = two_expiries(&ImpliedVolatilitySurfaceConfig::default());
        let x = dec!(-0.1);
        let shocked = surface.shock(SurfaceShock::Skew { rho: x }).unwrap();
        for (before, after) in surface.expiries().iter().zip(shocked.expiries()) {
            let (p, q) = (before.slice().parameters(), after.slice().parameters());
            assert_eq!(q.rho(), p.rho() + x);
            assert_eq!(
                (q.a(), q.b(), q.m(), q.sigma()),
                (p.a(), p.b(), p.m(), p.sigma())
            );
            let k = dec!(0.05);
            assert_close(
                after.slice().total_variance(k).unwrap().to_dec(),
                before.slice().total_variance(k).unwrap().to_dec() + p.b() * x * (k - p.m()),
                dec!(0.00000000000000000001),
            );
        }
    }

    #[test]
    fn test_term_shock_moves_at_the_money_linearly_in_expiry_about_its_pivot() {
        let surface = two_expiries(&ImpliedVolatilitySurfaceConfig::default());
        let (slope, pivot) = (dec!(0.02), dec!(0.5));
        let shocked = surface
            .shock(SurfaceShock::Term {
                slope,
                pivot: pos(pivot),
            })
            .unwrap();
        for (before, after) in surface.expiries().iter().zip(shocked.expiries()) {
            let move_here = slope * (before.time_to_expiry().to_dec() - pivot);
            assert_close(
                after.slice().implied_volatility(dec!(0)).unwrap().to_dec(),
                before.slice().implied_volatility(dec!(0)).unwrap().to_dec() + move_here,
                dec!(0.00000000000000000001),
            );
        }
        // The short expiry goes down and the long one up, about the pivot.
        let [short_before, long_before] = surface.expiries() else {
            panic!("two expiries");
        };
        let [short_after, long_after] = shocked.expiries() else {
            panic!("two expiries");
        };
        assert!(short_after.slice().parameters().a() < short_before.slice().parameters().a());
        assert!(long_after.slice().parameters().a() > long_before.slice().parameters().a());
    }

    #[test]
    fn test_shock_at_its_pivot_or_of_size_zero_changes_nothing_in_the_slices() {
        let surface = calibrate(&quotes_of(&[365])).unwrap();
        let t = surface.expiries().first().unwrap().time_to_expiry();
        for shock in [
            SurfaceShock::Level {
                volatility: dec!(0),
            },
            SurfaceShock::Skew { rho: dec!(0) },
            SurfaceShock::Term {
                slope: dec!(0.3),
                pivot: t,
            },
        ] {
            let shocked = surface.shock(shock).unwrap();
            assert_eq!(shocked.expiries(), surface.expiries());
            assert_eq!(shocked.arbitrage_report(), surface.arbitrage_report());
            assert_eq!(shocked.shocks(), [shock]);
        }
    }

    #[test]
    fn test_level_shock_to_a_volatility_at_or_below_zero_is_a_typed_error() {
        let surface = calibrate(&quotes_of(&[365])).unwrap();
        let atm = surface
            .expiries()
            .first()
            .unwrap()
            .slice()
            .implied_volatility(dec!(0))
            .unwrap();
        for volatility in [-atm.to_dec(), -atm.to_dec() - dec!(0.5)] {
            match surface
                .shock(SurfaceShock::Level { volatility })
                .unwrap_err()
            {
                ImpliedVolatilitySurfaceError::ShockedVolatilityNotPositive {
                    at_the_money,
                    shift,
                    ..
                } => {
                    assert_eq!(at_the_money, atm);
                    assert_eq!(shift, volatility);
                }
                other => panic!("{other:?}"),
            }
        }
    }

    #[test]
    fn test_level_shock_that_breaks_the_minimum_variance_is_a_typed_error() {
        let surface = calibrate(&quotes_of(&[365])).unwrap();
        let slice = *surface.expiries().first().unwrap().slice();
        let p = slice.parameters();
        let t = slice.time_to_expiry().to_dec();
        let minimum =
            p.a() + p.b() * p.sigma() * d_sqrt(Decimal::ONE - p.rho() * p.rho(), "test").unwrap();
        let w0 = slice.total_variance(dec!(0)).unwrap().to_dec();
        // Halfway from the at-the-money variance to the gap above the minimum: the
        // at-the-money volatility stays positive and the minimum goes below zero.
        let atm = slice.implied_volatility(dec!(0)).unwrap().to_dec();
        let target = d_sqrt((w0 - minimum) / dec!(2) / t, "test").unwrap();
        match surface
            .shock(SurfaceShock::Level {
                volatility: target - atm,
            })
            .unwrap_err()
        {
            ImpliedVolatilitySurfaceError::InvalidShockedParameters {
                time_to_expiry,
                source,
            } => {
                assert_eq!(time_to_expiry, slice.time_to_expiry());
                assert!(matches!(source, SviError::NegativeMinimumVariance { .. }));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn test_skew_shock_past_one_is_a_typed_error_naming_rho() {
        let surface = calibrate(&quotes_of(&[365])).unwrap();
        match surface
            .shock(SurfaceShock::Skew { rho: dec!(2) })
            .unwrap_err()
        {
            ImpliedVolatilitySurfaceError::InvalidShockedParameters { source, .. } => {
                assert!(matches!(
                    source,
                    SviError::InvalidParameter {
                        parameter: "rho",
                        ..
                    }
                ));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn test_term_shock_past_zero_is_a_typed_error_at_the_expiry_it_breaks() {
        let surface = two_expiries(&ImpliedVolatilitySurfaceConfig::default());
        match surface
            .shock(SurfaceShock::Term {
                slope: dec!(-100),
                pivot: Positive::ZERO,
            })
            .unwrap_err()
        {
            ImpliedVolatilitySurfaceError::ShockedVolatilityNotPositive {
                time_to_expiry, ..
            } => {
                assert_eq!(
                    time_to_expiry,
                    surface.expiries().first().unwrap().time_to_expiry()
                );
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn test_shocked_surface_has_its_own_report() {
        // A term shock that makes the short end high enough to cross the long one.
        let surface = two_expiries(&ImpliedVolatilitySurfaceConfig::default());
        assert!(surface.arbitrage_report().passed());
        let shocked = surface
            .shock(SurfaceShock::Term {
                slope: dec!(-0.5),
                pivot: Positive::ONE,
            })
            .unwrap();
        assert!(!shocked.arbitrage_report().passed());
        assert_eq!(shocked.arbitrage_grid(), surface.arbitrage_grid());
        assert!(surface.arbitrage_report().passed());
    }

    #[test]
    fn test_shocked_surface_recomputes_its_residuals_against_the_same_quotes() {
        let quotes = [quotes_of(&[91, 365]), frown(182)].concat();
        let surface = calibrate(&quotes).unwrap();
        let shocked = surface
            .shock(SurfaceShock::Level {
                volatility: dec!(0.01),
            })
            .unwrap();
        assert_eq!(shocked.rejections(), surface.rejections());
        assert_eq!(shocked.unfitted(), surface.unfitted());
        assert_eq!(shocked.as_of(), surface.as_of());
        for (before, after) in surface.residuals().zip(shocked.residuals()) {
            assert_eq!(
                after.observed_implied_volatility,
                before.observed_implied_volatility
            );
            assert_eq!(after.weight, before.weight);
            assert_eq!(after.log_moneyness, before.log_moneyness);
            assert!(after.fitted_implied_volatility > before.fitted_implied_volatility);
        }
        let expiry = shocked.expiries().first().unwrap();
        assert!(expiry.rmse() > surface.expiries().first().unwrap().rmse());
        assert_eq!(expiry.baseline().surface_rmse, expiry.rmse());
        assert_eq!(
            expiry.baseline().baseline_rmse,
            surface.expiries().first().unwrap().baseline().baseline_rmse
        );
        let twice = shocked
            .shock(SurfaceShock::Skew { rho: dec!(0.01) })
            .unwrap();
        assert_eq!(twice.shocks().len(), 2);
    }

    // Export.

    #[test]
    fn test_export_gives_one_point_per_pair_with_its_axes() {
        let surface = two_expiries(&ImpliedVolatilitySurfaceConfig::default());
        let grid = [dec!(-0.02), dec!(0), dec!(0.02)];
        let times = [
            surface.expiries().first().unwrap().time_to_expiry(),
            pos(dec!(0.5)),
        ];
        let export = surface.export_surface(&grid, &times).unwrap();
        assert!(export.extrapolated.is_empty());
        assert_eq!(export.surface.points.len(), 6);
        for point in &export.surface.points {
            let read = surface.point(point.x, pos(point.y)).unwrap();
            assert_eq!(point.z, read.implied_volatility.to_dec());
        }
        assert_eq!(export.surface.x_range, (dec!(-0.02), dec!(0.02)));
    }

    #[test]
    fn test_export_lists_what_it_read_outside_the_data() {
        let surface = two_expiries(&flexible());
        let last = surface.expiries().last().unwrap().time_to_expiry();
        let export = surface
            .export_surface(
                &[
                    dec!(0),
                    surface.expiries().last().unwrap().highest_log_moneyness() + dec!(0.1),
                ],
                &[last],
            )
            .unwrap();
        assert_eq!(export.surface.points.len(), 2);
        assert_eq!(export.extrapolated.len(), 1);
        assert!(export.extrapolated.first().unwrap().is_extrapolated());
    }

    #[test]
    fn test_export_lists_what_it_read_across_an_unfitted_expiry() {
        // The middle expiry has quotes and all of them are stale, so it has no slice.
        let stale = quotes_of(&[182]).into_iter().map(|quote| OptionQuote {
            quoted_at: as_of() - TimeDelta::try_hours(2).unwrap(),
            ..quote
        });
        let quotes: Vec<OptionQuote> = quotes_of(&[91, 365]).into_iter().chain(stale).collect();
        let surface = calibrate(&quotes).unwrap();
        assert_eq!(surface.unfitted().len(), 1);
        let across = years(182);
        let at_a_slice = years(91);
        let export = surface
            .export_surface(&[dec!(0), dec!(0.01)], &[at_a_slice, across])
            .unwrap();
        assert_eq!(export.surface.points.len(), 4);
        assert!(export.extrapolated.is_empty());
        assert_eq!(export.bridging_unfitted.len(), 2);
        for point in &export.bridging_unfitted {
            assert_eq!(point.time_to_expiry, across);
            assert!(point.bridges_unfitted_expiry);
        }
        // With no gap to cross, nothing is listed.
        let clean = calibrate(&quotes_of(&[91, 365])).unwrap();
        let export = clean.export_surface(&[dec!(0)], &[across]).unwrap();
        assert!(export.bridging_unfitted.is_empty());
    }

    #[test]
    fn test_export_fails_beyond_the_policy_and_on_an_empty_axis() {
        let surface = two_expiries(&ImpliedVolatilitySurfaceConfig::default());
        assert!(matches!(
            surface.export_surface(&[dec!(5)], &[Positive::ONE]),
            Err(ImpliedVolatilitySurfaceError::WingOutOfPolicy { .. })
        ));
        assert!(matches!(
            surface.export_surface(&[], &[Positive::ONE]),
            Err(ImpliedVolatilitySurfaceError::EmptyExportGrid)
        ));
        assert!(matches!(
            surface.export_surface(&[dec!(0)], &[]),
            Err(ImpliedVolatilitySurfaceError::EmptyExportGrid)
        ));
    }

    #[test]
    fn test_blend_is_convex_and_exact_at_zero() {
        assert_eq!(blend(dec!(3), dec!(9), dec!(0)).unwrap(), dec!(3));
        assert_eq!(blend(dec!(3), dec!(9), dec!(1)).unwrap(), dec!(9));
        assert_eq!(blend(dec!(3), dec!(9), dec!(0.25)).unwrap(), dec!(4.5));
    }
}
