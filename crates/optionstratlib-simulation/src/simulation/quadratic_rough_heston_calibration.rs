/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 7/11/26
******************************************************************************/

//! Calibration of the quadratic rough Heston model to an implied volatility surface: the
//! model's implied volatilities on a fixed Monte Carlo grid and seed, and a Nelder-Mead fit of
//! `(a, b, c, lambda, eta)` and the initial factors to a set of quotes.
//! [`calibrate_quadratic_rough_heston`] documents the objective, the constraints and the cost.

use super::quadratic_rough_heston::{
    Moments, QuadraticRoughHestonParams, Schedule, State, invalid_parameter, reserve, run_paths,
};
use crate::error::QuadraticRoughHestonError;
use num_traits::ToPrimitive;
use optionstratlib_core::constants::DAYS_IN_A_YEAR;
use optionstratlib_core::model::decimal::{d_div, d_mul, d_sub, decimal_to_f64, f64_to_decimal};
use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
use optionstratlib_core::model::{ExpirationDate, Options, Positive};
use optionstratlib_pricing::error::VolatilityError;
use optionstratlib_pricing::greeks::vega;
use optionstratlib_pricing::pricing::black_scholes;
use optionstratlib_pricing::volatility::implied_volatility;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::num::{NonZeroU64, NonZeroUsize};

/// The symbol of the Black-Scholes conversions, which name nothing.
const SYMBOL: &str = "QRH";

/// The most steps of the implied volatility solver per quote.
const IV_SOLVER_STEPS: i64 = 100;

/// The bounds of a logarithmic search coordinate: `ln a`, `ln c`, `ln lambda` and `ln eta`
/// are projected on them, so a parameter stays within `e^(+/-25)`.
const LOG_BOUNDS: (f64, f64) = (-25.0, 25.0);

/// Weighted RMSE below which the quotes are matched to the precision of the arithmetic and
/// nothing is left to search for.
const EXACT_FIT_RMSE: f64 = 1e-13;

/// Nelder-Mead reflection coefficient.
const REFLECTION: f64 = 1.0;
/// Nelder-Mead expansion coefficient.
const EXPANSION: f64 = 2.0;
/// Nelder-Mead contraction and shrink coefficient.
const CONTRACTION: f64 = 0.5;

/// Default Monte Carlo paths of one objective evaluation.
const DEFAULT_PATHS: NonZeroUsize = match NonZeroUsize::new(10_000) {
    Some(paths) => paths,
    None => NonZeroUsize::MIN,
};
/// Default largest step, in years: a trading day.
const DEFAULT_MAX_STEP: Decimal = dec!(0.003968253968253968); // 1 / 252
/// Default Nelder-Mead iterations.
const DEFAULT_MAX_ITERATIONS: NonZeroU64 = match NonZeroU64::new(2_000) {
    Some(iterations) => iterations,
    None => NonZeroU64::MIN,
};
/// Default stopping tolerance, in search coordinates and in weighted RMSE.
const DEFAULT_TOLERANCE: Decimal = dec!(0.000001);
/// Default initial step of the simplex in each search coordinate.
const DEFAULT_INITIAL_STEP: Decimal = dec!(0.1);

/// One point of the surface, a maturity and a strike.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QuadraticRoughHestonSurfacePoint {
    /// The maturity, in years. Must be positive.
    pub maturity: Positive,
    /// The strike. Must be positive.
    pub strike: Positive,
}

/// One implied volatility quote a calibration fits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QuadraticRoughHestonQuote {
    /// The maturity, in years. Must be positive.
    pub maturity: Positive,
    /// The strike. Must be positive.
    pub strike: Positive,
    /// The implied volatility of the quote, as a fraction per year, under Black-Scholes with
    /// `r = q = 0` on the model's spot. Must be positive.
    pub implied_volatility: Positive,
    /// The weight of the quote's squared residual. Only ratios between weights matter. A
    /// quote of zero weight does not enter the fit and still gets its fit reported.
    pub weight: Positive,
}

/// The Monte Carlo grid and seed of every objective evaluation, and of
/// [`price_surface_quadratic_rough_heston`].
///
/// Every evaluation draws the same normal increments, so the objective is a deterministic
/// function of the parameters and the fit does not chase Monte Carlo noise: it fits the model
/// as this grid and seed discretize it. Another seed or grid gives another, equally valid, fit
/// whose distance from the first measures that noise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QuadraticRoughHestonMonteCarloConfig {
    /// The paths of one evaluation. At least two.
    pub paths: NonZeroUsize,
    /// The largest step, in years. The time between two consecutive maturities of the input is
    /// split into equal steps no longer than this, so every maturity sits on the grid.
    pub max_step: Positive,
    /// The seed of the master generator.
    pub seed: u64,
}

impl Default for QuadraticRoughHestonMonteCarloConfig {
    /// 10,000 paths, a step of at most a trading day (`1 / 252` years), seed 0.
    fn default() -> Self {
        Self {
            paths: DEFAULT_PATHS,
            // The constant is positive, so the fallback is not taken; were it taken, a zero
            // step is rejected by every function here, so the failure stays loud.
            max_step: Positive::new_decimal(DEFAULT_MAX_STEP).unwrap_or(Positive::ZERO),
            seed: 0,
        }
    }
}

/// Which parameters the fit moves. The others keep the values of the start.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct QuadraticRoughHestonFreeParameters {
    /// The feedback strength `a`, kept positive.
    pub a: bool,
    /// The asymmetry `b`.
    pub b: bool,
    /// The base variance `c`, kept positive.
    pub c: bool,
    /// The mean reversion `lambda`, kept positive.
    pub lambda: bool,
    /// The factor volatility `eta`, kept positive.
    pub eta: bool,
    /// Every initial factor `Z^i_0`.
    pub initial_factors: bool,
}

impl QuadraticRoughHestonFreeParameters {
    /// Every parameter: `(a, b, c, lambda, eta)` and the initial factors.
    pub const ALL: Self = Self {
        a: true,
        b: true,
        c: true,
        lambda: true,
        eta: true,
        initial_factors: true,
    };

    /// `(a, b, c, lambda, eta)`, with the initial factors fixed.
    pub const MODEL: Self = Self {
        a: true,
        b: true,
        c: true,
        lambda: true,
        eta: true,
        initial_factors: false,
    };

    /// The dimension of the search for a model of `factors` factors, or `None` when it does
    /// not fit a `usize`.
    #[must_use]
    pub fn dimension(&self, factors: usize) -> Option<usize> {
        let model = [self.a, self.b, self.c, self.lambda, self.eta]
            .into_iter()
            .filter(|free| *free)
            .count();
        let factors = if self.initial_factors { factors } else { 0 };
        model.checked_add(factors)
    }
}

/// Limits of a calibration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QuadraticRoughHestonCalibrationConfig {
    /// The grid and seed of every objective evaluation.
    pub monte_carlo: QuadraticRoughHestonMonteCarloConfig,
    /// The most Nelder-Mead iterations. A search that reaches it has not converged, which is
    /// an error.
    pub max_iterations: NonZeroU64,
    /// The search has converged when the simplex fits in a box of this side in the search
    /// coordinates and the spread of its weighted RMSE, in implied volatility, is at or below
    /// this value. The coordinates are `ln a`, `ln c`, `ln lambda` and `ln eta`, where the
    /// side is a relative move of the parameter, and `b` and the initial factors, where it is
    /// an absolute one in their own units. A simplex that fits the quotes to the precision of
    /// the arithmetic has also converged.
    pub tolerance: Positive,
    /// The initial step of the simplex in each search coordinate: `0.1` moves a positive
    /// parameter by about 10% and `b` or an initial factor by `0.1`.
    pub initial_step: Positive,
}

impl Default for QuadraticRoughHestonCalibrationConfig {
    /// The default Monte Carlo, 2,000 iterations, a tolerance of `1e-6` and an initial step of
    /// `0.1`.
    fn default() -> Self {
        Self {
            monte_carlo: QuadraticRoughHestonMonteCarloConfig::default(),
            max_iterations: DEFAULT_MAX_ITERATIONS,
            // Both constants are positive, so neither fallback is taken; were one taken, a zero
            // value is rejected by `calibrate_quadratic_rough_heston`.
            tolerance: Positive::new_decimal(DEFAULT_TOLERANCE).unwrap_or(Positive::ZERO),
            initial_step: Positive::new_decimal(DEFAULT_INITIAL_STEP).unwrap_or(Positive::ZERO),
        }
    }
}

/// The model's price and implied volatility at one point of the surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct QuadraticRoughHestonModelQuote {
    /// The maturity, in years.
    pub maturity: Positive,
    /// The strike.
    pub strike: Positive,
    /// The style priced: the out-of-the-money one, a call at or above the spot and a put
    /// below it.
    pub style: OptionStyle,
    /// The Monte Carlo price, undiscounted.
    pub price: Decimal,
    /// The standard error of [`price`](Self::price).
    pub standard_error: Decimal,
    /// The Black-Scholes implied volatility of the price with `r = q = 0`, or `None` when the
    /// price has none: at or below the intrinsic value, or above the spot or the strike.
    pub implied_volatility: Option<Positive>,
}

/// How the fitted model sits against one quote.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct QuadraticRoughHestonQuoteFit {
    /// The quote.
    pub quote: QuadraticRoughHestonQuote,
    /// The style priced: the out-of-the-money one, a call at or above the spot and a put
    /// below it.
    pub style: OptionStyle,
    /// The Black-Scholes price of the quote at its implied volatility, with `r = q = 0`.
    pub market_price: Decimal,
    /// The Black-Scholes vega of the quote at its implied volatility, per unit of volatility.
    pub vega: Decimal,
    /// The model's Monte Carlo price at the fitted parameters.
    pub model_price: Decimal,
    /// The standard error of [`model_price`](Self::model_price).
    pub model_standard_error: Decimal,
    /// The model's implied volatility at the fitted parameters, or `None` when the model price
    /// has none.
    pub model_implied_volatility: Option<Positive>,
    /// `(model_price - market_price) / vega`: the error in implied volatility to first order,
    /// as a fraction per year. The objective is the weighted sum of its squares.
    pub residual: Decimal,
    /// The model's implied volatility minus the quote's, when the model price has one.
    pub implied_volatility_error: Option<Decimal>,
}

/// One segment of the grid a calibration ran on: the steps up to a maturity of the input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QuadraticRoughHestonCalibrationSegment {
    /// The maturity the segment ends at, in years.
    pub maturity: Positive,
    /// The steps from the previous maturity, or from zero.
    pub steps: usize,
    /// The step, in years.
    pub dt: Decimal,
}

/// The result of [`calibrate_quadratic_rough_heston`].
#[derive(Debug, Clone, PartialEq)]
pub struct QuadraticRoughHestonCalibration {
    params: QuadraticRoughHestonParams,
    quotes: Vec<QuadraticRoughHestonQuoteFit>,
    objective: Decimal,
    rmse: Positive,
    iterations: u64,
    evaluations: u64,
    seed: u64,
    paths: usize,
    grid: Vec<QuadraticRoughHestonCalibrationSegment>,
}

impl QuadraticRoughHestonCalibration {
    /// The fitted parameters. Their [`initial_variance`](QuadraticRoughHestonParams::initial_variance)
    /// is the `V_0` of the fit.
    #[must_use]
    #[inline]
    pub fn params(&self) -> &QuadraticRoughHestonParams {
        &self.params
    }

    /// One fit per quote, in the order of the input.
    #[must_use]
    #[inline]
    pub fn quotes(&self) -> &[QuadraticRoughHestonQuoteFit] {
        &self.quotes
    }

    /// The objective at the fitted parameters: the weighted sum of squared residuals.
    #[must_use]
    #[inline]
    pub fn objective(&self) -> Decimal {
        self.objective
    }

    /// The weighted root mean square residual, in implied volatility as a fraction per year:
    /// `sqrt(objective / sum of weights)`.
    #[must_use]
    #[inline]
    pub fn rmse(&self) -> Positive {
        self.rmse
    }

    /// The Nelder-Mead iterations run.
    #[must_use]
    #[inline]
    pub fn iterations(&self) -> u64 {
        self.iterations
    }

    /// The objective evaluations run, each a Monte Carlo over the grid.
    #[must_use]
    #[inline]
    pub fn evaluations(&self) -> u64 {
        self.evaluations
    }

    /// The seed of every evaluation.
    #[must_use]
    #[inline]
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// The paths of every evaluation.
    #[must_use]
    #[inline]
    pub fn paths(&self) -> usize {
        self.paths
    }

    /// The grid of every evaluation, one segment per distinct maturity in ascending order.
    #[must_use]
    #[inline]
    pub fn grid(&self) -> &[QuadraticRoughHestonCalibrationSegment] {
        &self.grid
    }
}

/// The grid of a run: the distinct maturities in ascending order, and the schedule.
struct Grid {
    maturities: Vec<Decimal>,
    segments: Vec<QuadraticRoughHestonCalibrationSegment>,
    schedule: Schedule,
}

impl Grid {
    /// Splits the time between consecutive distinct maturities into equal steps of at most
    /// `max_step` years.
    fn build(
        maturities: &[Positive],
        max_step: Positive,
    ) -> Result<Self, QuadraticRoughHestonError> {
        if max_step.is_zero() {
            return Err(invalid_parameter(
                "max_step",
                max_step.to_dec(),
                "must be positive",
            ));
        }
        let mut sorted: Vec<Decimal> = maturities.iter().map(Positive::to_dec).collect();
        sorted.sort();
        sorted.dedup();

        let mut segments = reserve(sorted.len(), "calibration segments")?;
        let mut raw_segments = reserve(sorted.len(), "schedule segments")?;
        let mut previous = Decimal::ZERO;
        for maturity in &sorted {
            let length = d_sub(*maturity, previous, "quadratic_rough_heston::grid::length")?;
            let ratio = d_div(
                length,
                max_step.to_dec(),
                "quadratic_rough_heston::grid::ratio",
            )?;
            let steps = ratio.ceil().to_usize().filter(|steps| *steps > 0).ok_or(
                QuadraticRoughHestonError::BufferTooLarge {
                    buffer: "grid steps",
                    len: usize::MAX,
                },
            )?;
            let dt = d_div(
                length,
                Decimal::from(steps),
                "quadratic_rough_heston::grid::dt",
            )?;
            if dt.is_zero() {
                return Err(QuadraticRoughHestonError::NonPositiveStep {
                    horizon: length,
                    steps,
                });
            }
            segments.push(QuadraticRoughHestonCalibrationSegment {
                maturity: Positive::new_decimal(*maturity)
                    .map_err(|_| invalid_parameter("maturity", *maturity, "must be positive"))?,
                steps,
                dt,
            });
            raw_segments.push((dt, steps));
            previous = *maturity;
        }
        let schedule = Schedule::from_segments(&raw_segments)?;
        Ok(Self {
            maturities: sorted,
            segments,
            schedule,
        })
    }

    /// The index of the segment that ends at `maturity`.
    fn segment_of(&self, maturity: Positive) -> Option<usize> {
        self.maturities.binary_search(&maturity.to_dec()).ok()
    }
}

/// A point of the surface as the kernel prices it.
#[derive(Debug, Clone, Copy, PartialEq)]
struct RawPoint {
    segment: usize,
    strike: f64,
    style: OptionStyle,
}

/// Rejects a point with a zero maturity or strike, before any grid is built on it.
fn validate_point(
    index: usize,
    point: &QuadraticRoughHestonSurfacePoint,
) -> Result<(), QuadraticRoughHestonError> {
    if point.maturity.is_zero() {
        return Err(QuadraticRoughHestonError::InvalidQuote {
            index,
            reason: "the maturity must be positive",
        });
    }
    if point.strike.is_zero() {
        return Err(QuadraticRoughHestonError::InvalidQuote {
            index,
            reason: "the strike must be positive",
        });
    }
    Ok(())
}

/// The out-of-the-money style at `strike`: a call at or above the spot, a put below it.
#[inline]
fn out_of_the_money(strike: Positive, spot: Positive) -> OptionStyle {
    if strike >= spot {
        OptionStyle::Call
    } else {
        OptionStyle::Put
    }
}

/// `point` as the kernel prices it, on `grid`.
fn raw_point(
    index: usize,
    point: &QuadraticRoughHestonSurfacePoint,
    spot: Positive,
    grid: &Grid,
) -> Result<RawPoint, QuadraticRoughHestonError> {
    validate_point(index, point)?;
    let segment =
        grid.segment_of(point.maturity)
            .ok_or(QuadraticRoughHestonError::InvalidQuote {
                index,
                reason: "the maturity is not on the grid",
            })?;
    Ok(RawPoint {
        segment,
        strike: decimal_to_f64(point.strike.to_dec())?,
        style: out_of_the_money(point.strike, spot),
    })
}

/// The payoff of `style` at `strike` on `spot`.
#[inline]
fn payoff(style: OptionStyle, strike: f64, spot: f64) -> f64 {
    match style {
        OptionStyle::Call => (spot - strike).max(0.0),
        OptionStyle::Put => (strike - spot).max(0.0),
    }
}

/// The spots at the end of each segment, per path.
fn terminal_spots(
    params: &QuadraticRoughHestonParams,
    grid: &Grid,
    monte_carlo: &QuadraticRoughHestonMonteCarloConfig,
) -> Result<Vec<Vec<f64>>, QuadraticRoughHestonError> {
    let runs = run_paths(
        params,
        &grid.schedule,
        monte_carlo.paths,
        monte_carlo.seed,
        |_, state: &State, spots: &mut Vec<f64>| spots.push(state.spot),
    )?;
    Ok(runs.into_iter().map(|run| run.observed).collect())
}

/// A European option of the model's spot, with `r = q = 0`, for the Black-Scholes
/// conversions.
fn european(
    style: OptionStyle,
    strike: Positive,
    maturity: Positive,
    spot: Positive,
    implied_volatility: Positive,
) -> Result<Options, QuadraticRoughHestonError> {
    let days = d_mul(
        maturity.to_dec(),
        DAYS_IN_A_YEAR.to_dec(),
        "quadratic_rough_heston::calibration::days",
    )?;
    let days = Positive::new_decimal(days)
        .map_err(|_| invalid_parameter("maturity", maturity.to_dec(), "must be positive"))?;
    Ok(Options::new(
        OptionType::European,
        Side::Long,
        SYMBOL.to_string(),
        strike,
        ExpirationDate::Days(days),
        implied_volatility,
        Positive::ONE,
        spot,
        Decimal::ZERO,
        style,
        Positive::ZERO,
        None,
    ))
}

/// The Black-Scholes implied volatility of `price`, or `None` when it has none.
fn model_implied_volatility(
    price: Decimal,
    style: OptionStyle,
    strike: Positive,
    maturity: Positive,
    spot: Positive,
    guess: Positive,
) -> Result<Option<Positive>, QuadraticRoughHestonError> {
    let Ok(price) = Positive::new_decimal(price) else {
        return Ok(None);
    };
    let mut option = european(style, strike, maturity, spot, guess)?;
    match implied_volatility(price, &mut option, IV_SOLVER_STEPS) {
        Ok(volatility) => Ok(Some(volatility)),
        Err(VolatilityError::IvNotFound | VolatilityError::NoValidVolatility) => Ok(None),
        Err(error) => Err(QuadraticRoughHestonError::from(error)),
    }
}

/// The model's price and implied volatility at each point of the surface, on the grid and
/// seed of `monte_carlo`.
///
/// Each point is priced as its out-of-the-money vanilla, a call at or above the spot and a put
/// below it, undiscounted as with `r = q = 0`, by one Monte Carlo over all the maturities:
/// every point of one maturity reads the same paths, and the paths are those of
/// [`calibrate_quadratic_rough_heston`] at the same `monte_carlo`. The implied volatility is
/// Black-Scholes with `r = q = 0` on the model's spot, and `None` for a price outside the
/// no-arbitrage band, which a deep wing on few paths can give. See
/// [`QuadraticRoughHestonParams`] for the scheme and the caution on one run.
///
/// # Errors
///
/// - [`QuadraticRoughHestonError::InvalidQuote`] for a point with a zero maturity or strike.
/// - [`QuadraticRoughHestonError::InvalidParameter`] for a single path or a zero `max_step`.
/// - [`QuadraticRoughHestonError::NonPositiveStep`] when a step between maturities rounds to
///   zero.
/// - [`QuadraticRoughHestonError::NonFinite`] when a path stops being finite, with its step.
/// - [`QuadraticRoughHestonError::Volatility`] when the implied volatility solver fails for
///   another reason than a price with no implied volatility.
/// - [`QuadraticRoughHestonError::BufferTooLarge`] when the paths, the grid or the points
///   cannot be allocated.
/// - [`QuadraticRoughHestonError::Decimal`] when a value has no `f64` or `Decimal` form.
#[tracing::instrument(level = "debug", skip(params, points), fields(points = points.len()))]
pub fn price_surface_quadratic_rough_heston(
    params: &QuadraticRoughHestonParams,
    points: &[QuadraticRoughHestonSurfacePoint],
    monte_carlo: &QuadraticRoughHestonMonteCarloConfig,
) -> Result<Vec<QuadraticRoughHestonModelQuote>, QuadraticRoughHestonError> {
    if points.is_empty() {
        return Err(QuadraticRoughHestonError::NoQuotes);
    }
    for (index, point) in points.iter().enumerate() {
        validate_point(index, point)?;
    }
    let maturities: Vec<Positive> = points.iter().map(|point| point.maturity).collect();
    let grid = Grid::build(&maturities, monte_carlo.max_step)?;
    let spot = params.spot();
    let mut raw_points = reserve(points.len(), "surface points")?;
    for (index, point) in points.iter().enumerate() {
        raw_points.push(raw_point(index, point, spot, &grid)?);
    }
    let spots = terminal_spots(params, &grid, monte_carlo)?;

    let mut quotes = reserve(points.len(), "model quotes")?;
    for (point, raw) in points.iter().zip(&raw_points) {
        let mut moments = Moments::default();
        for path in &spots {
            let spot = path.get(raw.segment).copied().unwrap_or(f64::NAN);
            moments.add(payoff(raw.style, raw.strike, spot));
        }
        let (price, standard_error) = moments.estimate()?;
        let implied_volatility = model_implied_volatility(
            price,
            raw.style,
            point.strike,
            point.maturity,
            spot,
            Positive::new_decimal(DEFAULT_IV_GUESS).unwrap_or(Positive::ONE),
        )?;
        quotes.push(QuadraticRoughHestonModelQuote {
            maturity: point.maturity,
            strike: point.strike,
            style: raw.style,
            price,
            standard_error,
            implied_volatility,
        });
    }
    Ok(quotes)
}

/// The implied volatility the solver starts from when a point carries none.
const DEFAULT_IV_GUESS: Decimal = dec!(0.2);

/// A quote as the objective reads it.
#[derive(Debug, Clone, Copy, PartialEq)]
struct RawQuote {
    point: RawPoint,
    market_price: f64,
    /// `weight / vega^2`, the coefficient of the squared price error.
    coefficient: f64,
    weight: f64,
}

/// The search coordinates and what they stand for.
struct Search {
    start: QuadraticRoughHestonParams,
    free: QuadraticRoughHestonFreeParameters,
}

impl Search {
    /// Which coordinates are logarithms, in the order of the point.
    fn logarithmic(&self) -> Vec<bool> {
        let mut mask = Vec::new();
        for (free, logarithmic) in [
            (self.free.a, true),
            (self.free.b, false),
            (self.free.c, true),
            (self.free.lambda, true),
            (self.free.eta, true),
        ] {
            if free {
                mask.push(logarithmic);
            }
        }
        if self.free.initial_factors {
            mask.extend(std::iter::repeat_n(
                false,
                self.start.initial_factors().len(),
            ));
        }
        mask
    }

    /// `point` with its logarithmic coordinates projected on [`LOG_BOUNDS`], so a vertex and
    /// the parameters it stands for agree.
    fn clamp(&self, mut point: Vec<f64>) -> Vec<f64> {
        for (value, logarithmic) in point.iter_mut().zip(self.logarithmic()) {
            if logarithmic && !value.is_nan() {
                *value = value.clamp(LOG_BOUNDS.0, LOG_BOUNDS.1);
            }
        }
        point
    }

    /// The start as a point of the search space.
    fn initial_point(&self) -> Result<Vec<f64>, QuadraticRoughHestonError> {
        let raw = &self.start.raw;
        let mut point = Vec::new();
        for (free, value, logarithmic) in [
            (self.free.a, raw.a, true),
            (self.free.b, raw.b, false),
            (self.free.c, raw.c, true),
            (self.free.lambda, raw.lambda, true),
            (self.free.eta, raw.eta, true),
        ] {
            if free {
                point.push(if logarithmic {
                    value.ln() // scan-banned: allow -- f64 `ln` of a positive parameter, which `QuadraticRoughHestonParams` validated: finite, and it does not abort
                } else {
                    value
                });
            }
        }
        if self.free.initial_factors {
            point.extend_from_slice(&self.start.raw_initial.factors);
        }
        Ok(point)
    }

    /// The parameters at a point of the search space, with the logarithmic coordinates
    /// projected on [`LOG_BOUNDS`]. `None` when a coordinate is not finite.
    fn params_at(
        &self,
        point: &[f64],
    ) -> Result<Option<QuadraticRoughHestonParams>, QuadraticRoughHestonError> {
        let mut coordinates = point.iter();
        let mut next = |free: bool, fixed: Decimal, logarithmic: bool| -> Option<Decimal> {
            if !free {
                return Some(fixed);
            }
            let value = *coordinates.next()?;
            let value = if logarithmic {
                value.clamp(LOG_BOUNDS.0, LOG_BOUNDS.1).exp() // scan-banned: allow -- f64 `exp` of a value within `LOG_BOUNDS`: finite, and it does not abort
            } else {
                value
            };
            f64_to_decimal(value).ok()
        };
        let Some(a) = next(self.free.a, self.start.a(), true) else {
            return Ok(None);
        };
        let Some(b) = next(self.free.b, self.start.b(), false) else {
            return Ok(None);
        };
        let Some(c) = next(self.free.c, self.start.c(), true) else {
            return Ok(None);
        };
        let Some(lambda) = next(self.free.lambda, self.start.lambda(), true) else {
            return Ok(None);
        };
        let Some(eta) = next(self.free.eta, self.start.eta(), true) else {
            return Ok(None);
        };
        let initial_factors = if self.free.initial_factors {
            let mut factors = reserve(self.start.initial_factors().len(), "initial factors")?;
            for _ in 0..self.start.initial_factors().len() {
                let Some(factor) = coordinates
                    .next()
                    .and_then(|value| f64_to_decimal(*value).ok())
                else {
                    return Ok(None);
                };
                factors.push(factor);
            }
            factors
        } else {
            self.start.initial_factors().to_vec()
        };
        Ok(Some(QuadraticRoughHestonParams::try_new(
            a,
            b,
            c,
            lambda,
            eta,
            self.start.kernel().clone(),
            initial_factors,
            self.start.spot(),
        )?))
    }
}

/// A vertex of the simplex: a point and its objective.
#[derive(Debug, Clone, PartialEq)]
struct Vertex {
    point: Vec<f64>,
    objective: f64,
}

/// The objective: the weighted sum of squared price errors over vega, by Monte Carlo on the
/// grid. `None` for parameters whose paths or prices are not finite, which the search treats
/// as an infinite objective.
fn objective(
    params: &QuadraticRoughHestonParams,
    quotes: &[RawQuote],
    grid: &Grid,
    monte_carlo: &QuadraticRoughHestonMonteCarloConfig,
    strict: bool,
) -> Result<Option<f64>, QuadraticRoughHestonError> {
    let spots = match terminal_spots(params, grid, monte_carlo) {
        Ok(spots) => spots,
        Err(QuadraticRoughHestonError::NonFinite { quantity, step }) if !strict => {
            tracing::debug!(quantity, step, "a path of a trial point is not finite");
            return Ok(None);
        }
        Err(error) => return Err(error),
    };
    let count = spots.len() as f64;
    let mut total = 0.0;
    for quote in quotes {
        if quote.weight == 0.0 {
            continue;
        }
        let sum: f64 = spots
            .iter()
            .map(|path| {
                payoff(
                    quote.point.style,
                    quote.point.strike,
                    path.get(quote.point.segment).copied().unwrap_or(f64::NAN),
                )
            })
            .sum();
        let error = sum / count - quote.market_price;
        total += quote.coefficient * error * error;
    }
    Ok(total.is_finite().then_some(total))
}

/// Orders a simplex by objective. Stable, and `total_cmp` is a total order, so the result is
/// fully determined by the input.
#[inline]
fn sort_simplex(simplex: &mut [Vertex]) {
    simplex.sort_by(|left, right| left.objective.total_cmp(&right.objective));
}

/// The stopping rule of [`QuadraticRoughHestonCalibrationConfig::tolerance`], with `rmse`
/// turning an objective into a weighted RMSE.
fn has_converged(simplex: &[Vertex], tolerance: f64, rmse: impl Fn(f64) -> f64) -> bool {
    let (Some(best), Some(worst)) = (simplex.first(), simplex.last()) else {
        return false;
    };
    let worst_rmse = rmse(worst.objective);
    if worst_rmse <= EXACT_FIT_RMSE {
        return true;
    }
    let spread = worst_rmse - rmse(best.objective);
    let diameter = simplex
        .iter()
        .flat_map(|vertex| {
            vertex
                .point
                .iter()
                .zip(&best.point)
                .map(|(value, reference)| (value - reference).abs())
        })
        // A NaN distance, from a coordinate that is not finite, is not a converged one.
        .map(|distance| {
            if distance.is_nan() {
                f64::INFINITY
            } else {
                distance
            }
        })
        .fold(0.0_f64, f64::max);
    spread <= tolerance && diameter <= tolerance
}

/// `base + coefficient * (base - other)`, coordinate by coordinate.
fn along(base: &[f64], other: &[f64], coefficient: f64) -> Vec<f64> {
    base.iter()
        .zip(other)
        .map(|(from, to)| from + coefficient * (from - to))
        .collect()
}

/// The search: the evaluated objective behind a point, with its count.
struct Evaluator<'a> {
    search: &'a Search,
    quotes: &'a [RawQuote],
    grid: &'a Grid,
    monte_carlo: &'a QuadraticRoughHestonMonteCarloConfig,
    evaluations: u64,
}

impl Evaluator<'_> {
    /// The vertex at `point`. Parameters whose paths are not finite get an infinite objective;
    /// `strict` turns that into the error of the path instead, for the initial simplex.
    fn vertex(
        &mut self,
        point: Vec<f64>,
        strict: bool,
    ) -> Result<Vertex, QuadraticRoughHestonError> {
        self.evaluations =
            self.evaluations
                .checked_add(1)
                .ok_or(QuadraticRoughHestonError::BufferTooLarge {
                    buffer: "objective evaluations",
                    len: usize::MAX,
                })?;
        let point = self.search.clamp(point);
        let objective = match self.search.params_at(&point)? {
            Some(params) => objective(&params, self.quotes, self.grid, self.monte_carlo, strict)?,
            None => None,
        };
        Ok(Vertex {
            point,
            objective: objective.unwrap_or(f64::INFINITY),
        })
    }
}

/// Fits the model to weighted implied volatility quotes.
///
/// # Objective
///
/// Each quote is priced as its out-of-the-money vanilla, a call at or above the spot and a put
/// below it, undiscounted as with `r = q = 0`, by one Monte Carlo over all the maturities on
/// the grid and seed of [`QuadraticRoughHestonCalibrationConfig::monte_carlo`]. The fit
/// minimizes
///
/// ```text
/// sum_i w_i ((P_i(theta) - P_i^market) / vega_i)^2
/// ```
///
/// where `P_i^market` and `vega_i` are the Black-Scholes price and vega of quote `i` at its
/// implied volatility, per unit of volatility. The residual `(P_i - P_i^market) / vega_i` is
/// the implied volatility error to first order, so the objective is a weighted squared implied
/// volatility error that stays defined and smooth where a Monte Carlo price has no implied
/// volatility, as a deep wing on few paths can give. [`QuadraticRoughHestonQuoteFit`] reports
/// both the residual and, where the model price has one, the exact implied volatility error.
/// [`QuadraticRoughHestonCalibration::rmse`] is the weighted RMSE of the residuals.
///
/// # Constraints
///
/// `a`, `c`, `lambda` and `eta` are searched as logarithms, so they stay positive; `b` and
/// the initial factors are free. The fixed parameters keep the values of `start`, as do the
/// kernel and the spot, which are never fitted. The result's
/// [`initial_variance`](QuadraticRoughHestonParams::initial_variance) is the `V_0 = a (Z_0 -
/// b)^2 + c` of the fit.
///
/// # Search
///
/// Nelder-Mead from `start`, with a simplex of one vertex per free coordinate at
/// [`initial_step`](QuadraticRoughHestonCalibrationConfig::initial_step) from the start. Every
/// evaluation draws the same increments, so the objective is deterministic and the search
/// converges on the fit of the discretized model rather than on noise; the result reports the
/// seed, the paths and the grid it used. Parameters whose paths stop being finite get an
/// infinite objective and are moved away from, except at the initial simplex, where that is
/// reported as the error of the path.
///
/// # Cost
///
/// One evaluation simulates `paths` paths over the steps to the last maturity, with `n`
/// factors per step, and prices every quote off them. A Nelder-Mead search in `d` dimensions
/// takes on the order of hundreds of evaluations per unit of `d` to converge. With the
/// defaults and five free parameters, expect a few thousand evaluations of 10,000 paths each.
///
/// # What the fit does not say
///
/// It is the fit of the model as this grid and seed discretize it, found by a local search
/// from `start`: another seed or grid gives another fit, and another start may give another
/// local minimum. Compare seeds, grids and starts before reading parameters off one fit.
///
/// # Errors
///
/// - [`QuadraticRoughHestonError::NoQuotes`] when no quote has a positive weight.
/// - [`QuadraticRoughHestonError::InvalidQuote`] for a quote with a zero maturity, strike or
///   implied volatility, or a zero Black-Scholes vega.
/// - [`QuadraticRoughHestonError::NoFreeParameters`] when `free` names no parameter.
/// - [`QuadraticRoughHestonError::InvalidParameter`] for a single path, a zero `max_step`,
///   `tolerance` or `initial_step`, or a free `a` that starts at zero.
/// - [`QuadraticRoughHestonError::NonPositiveStep`] when a step between maturities rounds to
///   zero.
/// - [`QuadraticRoughHestonError::NonFinite`] when a path of the initial simplex stops being
///   finite, with its step.
/// - [`QuadraticRoughHestonError::NoConvergence`] when the search reaches `max_iterations`.
/// - [`QuadraticRoughHestonError::Pricing`], [`QuadraticRoughHestonError::Greeks`] and
///   [`QuadraticRoughHestonError::Volatility`] when a Black-Scholes conversion fails.
/// - [`QuadraticRoughHestonError::BufferTooLarge`] when a buffer cannot be allocated.
/// - [`QuadraticRoughHestonError::Decimal`] when a value has no `f64` or `Decimal` form.
#[tracing::instrument(level = "debug", skip(quotes, start, config), fields(quotes = quotes.len()))]
pub fn calibrate_quadratic_rough_heston(
    quotes: &[QuadraticRoughHestonQuote],
    start: &QuadraticRoughHestonParams,
    free: QuadraticRoughHestonFreeParameters,
    config: &QuadraticRoughHestonCalibrationConfig,
) -> Result<QuadraticRoughHestonCalibration, QuadraticRoughHestonError> {
    if quotes.iter().all(|quote| quote.weight.is_zero()) {
        return Err(QuadraticRoughHestonError::NoQuotes);
    }
    let dimension = free.dimension(start.kernel().factors()).ok_or(
        QuadraticRoughHestonError::BufferTooLarge {
            buffer: "search coordinates",
            len: usize::MAX,
        },
    )?;
    if dimension == 0 {
        return Err(QuadraticRoughHestonError::NoFreeParameters);
    }
    if config.tolerance.is_zero() {
        return Err(invalid_parameter(
            "tolerance",
            config.tolerance.to_dec(),
            "must be positive",
        ));
    }
    if config.initial_step.is_zero() {
        return Err(invalid_parameter(
            "initial_step",
            config.initial_step.to_dec(),
            "must be positive",
        ));
    }
    if free.a && start.a().is_zero() {
        return Err(invalid_parameter(
            "a",
            start.a(),
            "must be positive to be calibrated: it is searched as a logarithm",
        ));
    }
    let tolerance = decimal_to_f64(config.tolerance.to_dec())?;
    let initial_step = decimal_to_f64(config.initial_step.to_dec())?;
    let monte_carlo = &config.monte_carlo;

    for (index, quote) in quotes.iter().enumerate() {
        validate_point(
            index,
            &QuadraticRoughHestonSurfacePoint {
                maturity: quote.maturity,
                strike: quote.strike,
            },
        )?;
        if quote.implied_volatility.is_zero() {
            return Err(QuadraticRoughHestonError::InvalidQuote {
                index,
                reason: "the implied volatility must be positive",
            });
        }
    }
    let maturities: Vec<Positive> = quotes.iter().map(|quote| quote.maturity).collect();
    let grid = Grid::build(&maturities, monte_carlo.max_step)?;
    let spot = start.spot();

    // The quotes as the objective reads them, with the Black-Scholes constants of each.
    let mut raw_quotes = reserve(quotes.len(), "calibration quotes")?;
    let mut market = reserve(quotes.len(), "market prices")?;
    let mut total_weight = 0.0_f64;
    for (index, quote) in quotes.iter().enumerate() {
        let point = raw_point(
            index,
            &QuadraticRoughHestonSurfacePoint {
                maturity: quote.maturity,
                strike: quote.strike,
            },
            spot,
            &grid,
        )?;
        let option = european(
            point.style,
            quote.strike,
            quote.maturity,
            spot,
            quote.implied_volatility,
        )?;
        let market_price = black_scholes(&option)?;
        // `vega` is per volatility point; the residual is per unit of volatility.
        let vega_per_unit = d_mul(
            vega(&option)?,
            Decimal::ONE_HUNDRED,
            "quadratic_rough_heston::calibration::vega",
        )?;
        if vega_per_unit <= Decimal::ZERO {
            return Err(QuadraticRoughHestonError::InvalidQuote {
                index,
                reason: "the Black-Scholes vega is zero",
            });
        }
        let raw_vega = decimal_to_f64(vega_per_unit)?;
        let weight = decimal_to_f64(quote.weight.to_dec())?;
        total_weight += weight;
        raw_quotes.push(RawQuote {
            point,
            market_price: decimal_to_f64(market_price)?,
            coefficient: weight / (raw_vega * raw_vega),
            weight,
        });
        market.push((market_price, vega_per_unit));
    }
    let rmse_of = |objective: f64| (objective / total_weight).sqrt(); // scan-banned: allow -- f64 `sqrt` of a non-negative ratio, or NaN for an infinite objective, which the stopping rule reads as not converged; it does not abort

    // The simplex: the start, and the start moved by `initial_step` along each coordinate.
    let search = Search {
        start: start.clone(),
        free,
    };
    let mut evaluator = Evaluator {
        search: &search,
        quotes: &raw_quotes,
        grid: &grid,
        monte_carlo,
        evaluations: 0,
    };
    let origin = search.initial_point()?;
    let vertices = dimension
        .checked_add(1)
        .ok_or(QuadraticRoughHestonError::BufferTooLarge {
            buffer: "simplex",
            len: dimension,
        })?;
    let mut simplex = reserve(vertices, "simplex")?;
    simplex.push(evaluator.vertex(origin.clone(), true)?);
    for index in 0..dimension {
        let mut point = origin.clone();
        if let Some(coordinate) = point.get_mut(index) {
            *coordinate += initial_step;
        }
        simplex.push(evaluator.vertex(point, true)?);
    }
    sort_simplex(&mut simplex);

    let mut iterations = 0_u64;
    let mut converged = has_converged(&simplex, tolerance, rmse_of);
    while !converged && iterations < config.max_iterations.get() {
        iterations = iterations
            .checked_add(1)
            .ok_or(QuadraticRoughHestonError::NoConvergence { iterations })?;
        nelder_mead_step(&mut simplex, &mut evaluator)?;
        sort_simplex(&mut simplex);
        converged = has_converged(&simplex, tolerance, rmse_of);
        if let Some(best) = simplex.first() {
            tracing::debug!(
                iteration = iterations,
                objective = best.objective,
                rmse = rmse_of(best.objective),
                "nelder-mead"
            );
        }
    }
    if !converged {
        return Err(QuadraticRoughHestonError::NoConvergence { iterations });
    }
    let best = simplex
        .first()
        .ok_or(QuadraticRoughHestonError::NoFreeParameters)?;
    // A converged best vertex has a finite objective, so finite coordinates and parameters;
    // the fallback is for the type, not a reachable case.
    let params = search
        .params_at(&best.point)?
        .ok_or(QuadraticRoughHestonError::NoConvergence { iterations })?;

    // The fit per quote at the best parameters, with the standard errors and the implied
    // volatilities the objective does not need.
    let spots = terminal_spots(&params, &grid, monte_carlo)?;
    let mut fits = reserve(quotes.len(), "quote fits")?;
    let mut objective_total = 0.0_f64;
    for ((quote, raw), (market_price, vega_per_unit)) in quotes.iter().zip(&raw_quotes).zip(&market)
    {
        let mut moments = Moments::default();
        for path in &spots {
            let spot = path.get(raw.point.segment).copied().unwrap_or(f64::NAN);
            moments.add(payoff(raw.point.style, raw.point.strike, spot));
        }
        let (model_price, model_standard_error) = moments.estimate()?;
        let residual = d_div(
            d_sub(
                model_price,
                *market_price,
                "quadratic_rough_heston::calibration::price_error",
            )?,
            *vega_per_unit,
            "quadratic_rough_heston::calibration::residual",
        )?;
        let raw_residual = decimal_to_f64(residual)?;
        objective_total += raw.weight * raw_residual * raw_residual;
        let model_implied_volatility = model_implied_volatility(
            model_price,
            raw.point.style,
            quote.strike,
            quote.maturity,
            spot,
            quote.implied_volatility,
        )?;
        let implied_volatility_error = model_implied_volatility
            .map(|volatility| {
                d_sub(
                    volatility.to_dec(),
                    quote.implied_volatility.to_dec(),
                    "quadratic_rough_heston::calibration::iv_error",
                )
            })
            .transpose()?;
        fits.push(QuadraticRoughHestonQuoteFit {
            quote: *quote,
            style: raw.point.style,
            market_price: *market_price,
            vega: *vega_per_unit,
            model_price,
            model_standard_error,
            model_implied_volatility,
            residual,
            implied_volatility_error,
        });
    }
    // The root of a non-negative finite ratio: `new_decimal` cannot reject it, and a NaN was
    // reported by `f64_to_decimal` above.
    let rmse =
        Positive::new_decimal(f64_to_decimal(rmse_of(objective_total))?).unwrap_or(Positive::ZERO);

    Ok(QuadraticRoughHestonCalibration {
        params,
        quotes: fits,
        objective: f64_to_decimal(objective_total)?,
        rmse,
        iterations,
        evaluations: evaluator.evaluations,
        seed: monte_carlo.seed,
        paths: monte_carlo.paths.get(),
        grid: grid.segments,
    })
}

/// One iteration of Nelder-Mead on a sorted simplex (Lagarias et al., 1998).
fn nelder_mead_step(
    simplex: &mut [Vertex],
    evaluator: &mut Evaluator<'_>,
) -> Result<(), QuadraticRoughHestonError> {
    let Some((worst, rest)) = simplex.split_last() else {
        return Ok(());
    };
    let (Some(best), Some(second_worst)) = (rest.first(), rest.last()) else {
        return Ok(());
    };
    let dimension = worst.point.len();
    let mut centroid = vec![0.0_f64; dimension];
    for vertex in rest {
        for (sum, value) in centroid.iter_mut().zip(&vertex.point) {
            *sum += value / rest.len() as f64;
        }
    }

    let reflected = evaluator.vertex(along(&centroid, &worst.point, REFLECTION), false)?;
    let replacement = if reflected.objective < best.objective {
        let expanded = evaluator.vertex(along(&centroid, &worst.point, EXPANSION), false)?;
        if expanded.objective < reflected.objective {
            expanded
        } else {
            reflected
        }
    } else if reflected.objective < second_worst.objective {
        reflected
    } else {
        let outside = reflected.objective < worst.objective;
        let coefficient = if outside { CONTRACTION } else { -CONTRACTION };
        let contracted = evaluator.vertex(along(&centroid, &worst.point, coefficient), false)?;
        // Strict on the inside: on a plateau of equal objectives the simplex then shrinks
        // instead of collapsing on an edge.
        let accepted = if outside {
            contracted.objective <= reflected.objective
        } else {
            contracted.objective < worst.objective
        };
        if accepted {
            contracted
        } else {
            let best_point = best.point.clone();
            for vertex in simplex.iter_mut().skip(1) {
                let shrunk = along(&best_point, &vertex.point, -CONTRACTION);
                *vertex = evaluator.vertex(shrunk, false)?;
            }
            return Ok(());
        }
    };
    if let Some(last) = simplex.last_mut() {
        *last = replacement;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::QuadraticRoughHestonKernel;
    use optionstratlib_core::pos_or_panic;

    fn params() -> QuadraticRoughHestonParams {
        let kernel = QuadraticRoughHestonKernel::try_new(dec!(0.6), 2, dec!(5)).unwrap();
        QuadraticRoughHestonParams::try_new(
            dec!(0.4),
            dec!(0.1),
            dec!(0.02),
            dec!(1.5),
            dec!(0.8),
            kernel,
            vec![dec!(-0.1), dec!(-0.2)],
            Positive::HUNDRED,
        )
        .unwrap()
    }

    #[test]
    fn test_free_parameters_count_their_dimension() {
        assert_eq!(
            QuadraticRoughHestonFreeParameters::ALL.dimension(3),
            Some(8)
        );
        assert_eq!(
            QuadraticRoughHestonFreeParameters::MODEL.dimension(3),
            Some(5)
        );
        assert_eq!(
            QuadraticRoughHestonFreeParameters::default().dimension(3),
            Some(0)
        );
        assert_eq!(
            QuadraticRoughHestonFreeParameters::ALL.dimension(usize::MAX),
            None
        );
        let only_factors = QuadraticRoughHestonFreeParameters {
            initial_factors: true,
            ..Default::default()
        };
        assert_eq!(only_factors.dimension(4), Some(4));
    }

    #[test]
    fn test_defaults_are_the_documented_values() {
        let config = QuadraticRoughHestonCalibrationConfig::default();
        assert_eq!(config.monte_carlo.paths.get(), 10_000);
        assert_eq!(config.monte_carlo.seed, 0);
        assert!(
            (config.monte_carlo.max_step.to_dec() - Decimal::ONE / dec!(252)).abs() < dec!(1e-15)
        );
        assert_eq!(config.max_iterations.get(), 2_000);
        assert_eq!(config.tolerance, pos_or_panic!(1e-6));
        assert_eq!(config.initial_step, pos_or_panic!(0.1));
    }

    /// Maturities `0.1`, `0.25`, `0.5` and a step of at most `0.02`: `5`, `8` and `13` steps.
    #[test]
    fn test_grid_splits_each_gap_into_equal_steps_at_most_the_maximum() {
        let maturities = [
            pos_or_panic!(0.5),
            pos_or_panic!(0.1),
            pos_or_panic!(0.25),
            pos_or_panic!(0.1),
        ];
        let grid = Grid::build(&maturities, pos_or_panic!(0.02)).unwrap();
        assert_eq!(grid.maturities, vec![dec!(0.1), dec!(0.25), dec!(0.5)]);
        let steps: Vec<usize> = grid.segments.iter().map(|segment| segment.steps).collect();
        assert_eq!(steps, vec![5, 8, 13]);
        for segment in &grid.segments {
            assert!(segment.dt <= dec!(0.02), "{segment:?}");
            assert!(segment.dt > Decimal::ZERO);
        }
        assert_eq!(grid.segments[1].dt, dec!(0.15) / dec!(8));
        assert_eq!(grid.segment_of(pos_or_panic!(0.25)), Some(1));
        assert_eq!(grid.segment_of(pos_or_panic!(0.3)), None);
    }

    #[test]
    fn test_grid_rejects_a_zero_step() {
        assert!(matches!(
            Grid::build(&[pos_or_panic!(0.5)], Positive::ZERO),
            Err(QuadraticRoughHestonError::InvalidParameter {
                parameter: "max_step",
                ..
            })
        ));
    }

    #[test]
    fn test_out_of_the_money_style_is_a_call_at_or_above_the_spot() {
        assert_eq!(
            out_of_the_money(Positive::HUNDRED, Positive::HUNDRED),
            OptionStyle::Call
        );
        assert_eq!(
            out_of_the_money(pos_or_panic!(110.0), Positive::HUNDRED),
            OptionStyle::Call
        );
        assert_eq!(
            out_of_the_money(pos_or_panic!(90.0), Positive::HUNDRED),
            OptionStyle::Put
        );
    }

    #[test]
    fn test_search_round_trips_the_start_through_its_coordinates() {
        let start = params();
        for free in [
            QuadraticRoughHestonFreeParameters::ALL,
            QuadraticRoughHestonFreeParameters::MODEL,
            QuadraticRoughHestonFreeParameters {
                b: true,
                initial_factors: true,
                ..Default::default()
            },
        ] {
            let search = Search {
                start: start.clone(),
                free,
            };
            let point = search.initial_point().unwrap();
            assert_eq!(Some(point.len()), free.dimension(2));
            let back = search.params_at(&point).unwrap().unwrap();
            let close =
                |actual: Decimal, expected: Decimal| (actual - expected).abs() < dec!(1e-12);
            assert!(close(back.a(), start.a()));
            assert!(close(back.b(), start.b()));
            assert!(close(back.c(), start.c()));
            assert!(close(back.lambda(), start.lambda()));
            assert!(close(back.eta(), start.eta()));
            for (actual, expected) in back.initial_factors().iter().zip(start.initial_factors()) {
                assert!(close(*actual, *expected));
            }
        }
    }

    /// A logarithmic coordinate beyond the bounds is projected on them, and a coordinate that
    /// is not finite gives no parameters.
    #[test]
    fn test_search_projects_logarithms_on_their_bounds_and_rejects_non_finite_points() {
        let search = Search {
            start: params(),
            free: QuadraticRoughHestonFreeParameters {
                a: true,
                ..Default::default()
            },
        };
        let projected = search.params_at(&[1e9]).unwrap().unwrap();
        assert!((decimal_to_f64(projected.a()).unwrap() - LOG_BOUNDS.1.exp()).abs() < 1e-3);
        assert!(search.params_at(&[f64::NAN]).unwrap().is_none());
    }

    #[test]
    fn test_search_clamps_logarithmic_coordinates_and_leaves_the_others() {
        let search = Search {
            start: params(),
            free: QuadraticRoughHestonFreeParameters::ALL,
        };
        assert_eq!(
            search.logarithmic(),
            vec![true, false, true, true, true, false, false]
        );
        let clamped = search.clamp(vec![1e9, 1e9, -1e9, 3.0, f64::NAN, 1e9, -1e9]);
        assert_eq!(clamped[0], LOG_BOUNDS.1);
        assert_eq!(clamped[1], 1e9);
        assert_eq!(clamped[2], LOG_BOUNDS.0);
        assert_eq!(clamped[3], 3.0);
        assert!(clamped[4].is_nan());
        assert_eq!(clamped[5], 1e9);
        assert_eq!(clamped[6], -1e9);
    }

    #[test]
    fn test_has_converged_treats_a_nan_distance_as_not_converged() {
        let rmse = |objective: f64| objective.sqrt();
        let simplex = vec![
            Vertex {
                point: vec![f64::NEG_INFINITY, 0.0],
                objective: 1e-4,
            },
            Vertex {
                point: vec![f64::NEG_INFINITY, 1e-9],
                objective: 1e-4,
            },
        ];
        assert!(!has_converged(&simplex, 1e-6, rmse));
    }

    #[test]
    fn test_has_converged_reads_the_spread_and_the_diameter() {
        let rmse = |objective: f64| objective.sqrt();
        let tight = vec![
            Vertex {
                point: vec![0.0, 0.0],
                objective: 1e-4,
            },
            Vertex {
                point: vec![1e-7, 0.0],
                objective: 1e-4 + 1e-9,
            },
            Vertex {
                point: vec![0.0, 1e-7],
                objective: 1e-4 + 2e-9,
            },
        ];
        assert!(has_converged(&tight, 1e-6, rmse));
        let wide = vec![
            Vertex {
                point: vec![0.0, 0.0],
                objective: 1e-4,
            },
            Vertex {
                point: vec![1e-3, 0.0],
                objective: 1e-4,
            },
        ];
        assert!(!has_converged(&wide, 1e-6, rmse));
        let exact = vec![
            Vertex {
                point: vec![0.0],
                objective: 0.0,
            },
            Vertex {
                point: vec![5.0],
                objective: 1e-30,
            },
        ];
        assert!(has_converged(&exact, 1e-6, rmse));
        assert!(!has_converged(&[], 1e-6, rmse));
    }

    #[test]
    fn test_along_moves_from_the_base_away_from_the_other_point() {
        assert_eq!(along(&[1.0, 2.0], &[0.0, 4.0], 1.0), vec![2.0, 0.0]);
        assert_eq!(along(&[1.0, 2.0], &[0.0, 4.0], -0.5), vec![0.5, 3.0]);
    }

    #[test]
    fn test_payoff_is_the_intrinsic_value() {
        assert_eq!(payoff(OptionStyle::Call, 100.0, 110.0), 10.0);
        assert_eq!(payoff(OptionStyle::Call, 100.0, 90.0), 0.0);
        assert_eq!(payoff(OptionStyle::Put, 100.0, 90.0), 10.0);
        assert_eq!(payoff(OptionStyle::Put, 100.0, 110.0), 0.0);
    }

    #[test]
    fn test_model_implied_volatility_is_none_for_a_price_without_one() {
        let spot = Positive::HUNDRED;
        let strike = pos_or_panic!(110.0);
        let maturity = pos_or_panic!(0.25);
        for price in [Decimal::ZERO, dec!(-1), dec!(200)] {
            assert_eq!(
                model_implied_volatility(
                    price,
                    OptionStyle::Call,
                    strike,
                    maturity,
                    spot,
                    pos_or_panic!(0.2)
                )
                .unwrap(),
                None,
                "{price}"
            );
        }
        // The Black-Scholes price at 20% inverts to 20%.
        let option = european(
            OptionStyle::Call,
            strike,
            maturity,
            spot,
            pos_or_panic!(0.2),
        )
        .unwrap();
        let price = black_scholes(&option).unwrap();
        let volatility = model_implied_volatility(
            price,
            OptionStyle::Call,
            strike,
            maturity,
            spot,
            pos_or_panic!(0.3),
        )
        .unwrap()
        .unwrap();
        assert!(
            (volatility.to_dec() - dec!(0.2)).abs() < dec!(1e-6),
            "{volatility}"
        );
    }
}
