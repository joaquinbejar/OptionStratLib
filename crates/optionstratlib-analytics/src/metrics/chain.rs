//! Metric trait implementations for [`optionstratlib_market::chains::OptionChain`].
//!
//! The metric traits under `metrics::{composite, liquidity, price, risk,
//! stress, temporal}` are analytics-owned, and `OptionChain` is a market-owned
//! type, so the `impl <MetricTrait> for OptionChain` blocks live here, next to
//! the traits, rather than in `chains::chain`. Coherence allows it (local
//! trait, foreign type) and it keeps the dependency edge pointing downward:
//! analytics reads the chain through its public fields and accessors, and
//! `chains` never imports `metrics` (ADR-0001 D4, M1-12).
//!
//! The module is private: the impls need no path, and every metric is reached
//! by importing its trait from [`crate::metrics`].

use crate::metrics::{
    BidAskSpreadCurve, CharmCurve, CharmSurface, ColorCurve, ColorSurface, DeltaGammaProfileCurve,
    DeltaGammaProfileSurface, DollarGammaCurve, ImpliedVolatilityCurve, ImpliedVolatilitySurface,
    OpenInterestCurve, PriceShockCurve, PriceShockSurface, PutCallRatioCurve, RiskReversalCurve,
    SmileDynamicsCurve, SmileDynamicsSurface, StrikeConcentrationCurve, ThetaCurve, ThetaSurface,
    TimeDecayCurve, TimeDecaySurface, VannaVolgaSurface, VolatilitySensitivityCurve,
    VolatilitySensitivitySurface, VolatilitySkewCurve, VolumeProfileCurve, VolumeProfileSurface,
};
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::{d_add, d_div, d_mul, d_sqrt};
use optionstratlib_core::model::{ExpirationDate, OptionStyle, Options, Side};
use optionstratlib_market::chains::OptionChain;
use optionstratlib_math::curves::{Curve, Point2D};
use optionstratlib_math::error::CurveError;
use optionstratlib_math::error::SurfaceError;
use optionstratlib_math::geometrics::LinearInterpolation;
use optionstratlib_math::surfaces::{Point3D, Surface};
use optionstratlib_pricing::greeks::Greeks;
use optionstratlib_pricing::pricing::OptionPricing;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt::Display;

// The arithmetic left on raw operators in this module is total: a difference
// of two non-negative `Decimal`s (strike minus spot, ask minus bid, one
// volatility minus another) lies in `[-MAX, MAX]`, and a division by a
// constant of at least one cannot overflow. Every product, every sum and
// every division by chain data goes through the checked helpers (#788).
//
// No metric substitutes a value for a failed step (#639): a square root that
// fails, or a grid point that is not a valid `Positive`, is an error. Both
// used to read as 1 (or 0.01 for a volatility); for every finite input on
// which they succeed the result is unchanged.

/// Carries a checked-arithmetic failure out of a curve metric.
///
/// A metric over an extreme chain (a spot whose square leaves the `Decimal`
/// range, a zero spot under a moneyness ratio, a book whose premia are all
/// zero) has no representable value. It is reported through the curve error
/// channel instead of aborting in a raw operator (#788).
#[cold]
#[inline(never)]
fn curve_math<E: Display>(error: E) -> CurveError {
    CurveError::MetricsError(error.to_string())
}

/// Surface counterpart of [`curve_math`].
#[cold]
#[inline(never)]
fn surface_math<E: Display>(error: E) -> SurfaceError {
    SurfaceError::AnalysisError(error.to_string())
}

/// Spacing of `[range.0, range.1]` divided into `steps` equal intervals.
///
/// Zero steps keep the single point `range.0` and need no spacing, so the
/// bounds are not compared there, as before. Otherwise an upper bound below
/// the lower one has no non-negative width; it is rejected as an invalid
/// range instead of aborting in the `Positive` subtraction (#788).
///
/// # Errors
///
/// Returns [`SurfaceError::OperationError`] naming `name` when
/// `range.1 < range.0`, and [`SurfaceError::AnalysisError`] when the spacing
/// leaves the `Decimal` range.
fn grid_step(
    range: (Positive, Positive),
    steps: usize,
    name: &'static str,
) -> Result<Decimal, SurfaceError> {
    if steps == 0 {
        return Ok(Decimal::ZERO);
    }
    let width = range.1.checked_sub(&range.0).map_err(|_| {
        SurfaceError::invalid_parameters(
            name,
            &format!("upper bound {} is below lower bound {}", range.1, range.0),
        )
    })?;
    d_div(
        width.to_dec(),
        Decimal::from(steps),
        "metrics::chain::grid_step",
    )
    .map_err(surface_math)
}

/// The `index`-th point, `lower + step * index`, of an evenly spaced grid.
///
/// # Errors
///
/// Returns [`SurfaceError::AnalysisError`] when the point leaves the
/// `Decimal` range.
fn grid_point(lower: Positive, step: Decimal, index: usize) -> Result<Decimal, SurfaceError> {
    d_mul(step, Decimal::from(index), "metrics::chain::grid_point")
        .and_then(|offset| d_add(lower.to_dec(), offset, "metrics::chain::grid_point"))
        .map_err(surface_math)
}

impl VolatilitySkewCurve for OptionChain {
    /// Computes the volatility skew for the option chain.
    ///
    /// This function calculates the volatility skew by interpolating the implied
    /// volatilities for all the calculated moneyness data points in the option chain.
    /// It uses the available implied volatilities from the `options` field and
    /// performs linear interpolation to estimate missing values.
    ///
    /// # Returns
    ///
    /// A `Curve` object representing the volatility skew. The x-coordinates of the curve
    /// correspond to the moneyness, and the y-coordinates represent the corresponding
    /// implied volatilities.
    fn volatility_skew(&self) -> Result<Curve, CurveError> {
        // Build a BTreeSet with the known points (options with implied volatility)
        // A zero spot has no moneyness, and a far strike over a tiny spot
        // leaves the `Decimal` range: both come back as errors (#788).
        let mut bt_points = self
            .options
            .iter()
            .map(|option| {
                let ratio = d_div(
                    option.strike_price.to_dec(),
                    self.underlying_price.to_dec(),
                    "metrics::chain::volatility_skew",
                )
                .map_err(curve_math)?;
                // `ratio >= 0`, so `ratio - 1` cannot leave the range.
                let moneyness = d_mul(
                    ratio - Decimal::ONE,
                    Decimal::ONE_HUNDRED,
                    "metrics::chain::volatility_skew",
                )
                .map_err(curve_math)?;
                Ok(Point2D::new(moneyness, option.implied_volatility.to_dec()))
            })
            .collect::<Result<BTreeSet<_>, CurveError>>()?;

        // Create an initial Curve object using the known points
        let curve = Curve::new(bt_points.clone());

        // Interpolate missing points (options without implied volatility)
        for option in self
            .options
            .iter()
            .filter(|o| o.implied_volatility.is_zero())
        {
            // Use linear interpolation to estimate the missing implied volatility
            if let Ok(interpolated_point) = curve.linear_interpolate(option.strike_price.to_dec()) {
                bt_points.insert(interpolated_point);
            }
        }

        // Return the final Curve with all points, including interpolated ones
        // if no available points return an error
        if bt_points.is_empty() {
            return Err(CurveError::ConstructionError(
                "No options with valid volatility skew data".to_string(),
            ));
        }
        Ok(Curve::new(bt_points))
    }
}

impl ImpliedVolatilityCurve for OptionChain {
    /// Computes the implied volatility curve by strike price for this option chain.
    ///
    /// Creates a curve showing how implied volatility varies across different strike
    /// prices. This is useful for visualizing the volatility smile or skew pattern.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: A curve with strike prices on the x-axis and implied volatility
    ///   values on the y-axis
    /// - `Err(CurveError)`: If no options have valid implied volatility data
    ///
    /// # Example
    ///
    /// ```ignore
    /// use optionstratlib_market::chains::chain::OptionChain;
    /// use optionstratlib_analytics::metrics::ImpliedVolatilityCurve;
    ///
    /// let chain = OptionChain::new("SPY", pos_or_panic!(450.0), "2024-03-15".to_string(), None, None);
    /// let iv_curve = chain.iv_curve()?;
    /// ```
    fn iv_curve(&self) -> Result<Curve, CurveError> {
        let points: BTreeSet<Point2D> = self
            .options
            .iter()
            .filter(|opt| !opt.implied_volatility.is_zero())
            .map(|opt| Point2D::new(opt.strike_price.to_dec(), opt.implied_volatility.to_dec()))
            .collect();

        if points.is_empty() {
            return Err(CurveError::ConstructionError(
                "No options with valid implied volatility".to_string(),
            ));
        }

        Ok(Curve::new(points))
    }
}

impl PutCallRatioCurve for OptionChain {
    /// Computes the Put/Call Ratio curve by strike price for this option chain.
    ///
    /// Creates a curve showing how Put/Call Ratio varies across different strike
    /// prices. PCR is calculated as premium-weighted.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: A curve with strike prices on the x-axis and PCR values on the y-axis.
    /// - `Err(CurveError)`: If the Put/Call Ratio calculation is not successful.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use optionstratlib_market::chains::chain::OptionChain;
    /// use optionstratlib_analytics::metrics::PutCallRatioCurve;
    ///
    /// let chain = OptionChain::new("SPY", pos!(450.0), "2024-03-15".to_string(), None, None);
    /// let pcr_curve = chain.open_interest_pcr().unwrap();
    /// ```
    fn premium_weighted_pcr(&self) -> Result<Curve, CurveError> {
        let mut points = BTreeSet::new();
        for opt in self.options.iter() {
            // Calculate Put/Call Ratio premium weighted
            if let (Some(put_mid), Some(call_mid)) = (opt.put_middle, opt.call_middle) {
                let call_mid_dec = call_mid.to_dec();
                // Avoid division by zero
                if call_mid_dec.is_zero() {
                    continue;
                }
                let ratio = d_div(
                    put_mid.to_dec(),
                    call_mid_dec,
                    "metrics::chain::premium_weighted_pcr",
                )
                .map_err(curve_math)?;
                points.insert(Point2D::new(opt.strike_price.to_dec(), ratio));
            }
        }
        if points.is_empty() {
            return Err(CurveError::ConstructionError(
                "No options with valid premium weighted Put/Call ratio data".to_string(),
            ));
        }
        Ok(Curve::new(points))
    }
}

impl StrikeConcentrationCurve for OptionChain {
    /// Computes the Strike Concentration curve by strike price for this option chain.
    ///
    /// Creates a curve showing how Strike Concentration varies across different strike
    /// prices. Strike Concentration is calculated as premium-weighted.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: A curve with strike prices on the x-axis and Strike Concentration
    ///   values on the y-axis.
    /// - `Err(CurveError)`: If the Strike Concentration calculation is not successful.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use optionstratlib_market::chains::chain::OptionChain;
    /// use optionstratlib_analytics::metrics::StrikeConcentrationCurve;
    ///
    /// let chain = OptionChain::new("SPY", pos!(450.0), "2024-03-15".to_string(), None, None);
    /// let strike_concentration_curve = chain.premium_concentration().unwrap();
    /// ```
    fn premium_concentration(&self) -> Result<Curve, CurveError> {
        let mut points = BTreeSet::new();
        // Calculate total premium per each strike and total chain premium
        let mut total_chain_premium = Decimal::ZERO;
        let mut _strike_premium = Decimal::ZERO;
        let mut strike_count = Decimal::ZERO;
        for opt in self.options.iter() {
            if let (Some(put_mid), Some(call_mid)) = (opt.put_middle, opt.call_middle) {
                const OP: &str = "metrics::chain::premium_concentration";
                _strike_premium =
                    d_add(put_mid.to_dec(), call_mid.to_dec(), OP).map_err(curve_math)?;
                points.insert(Point2D::new(opt.strike_price.to_dec(), _strike_premium));
                total_chain_premium =
                    d_add(total_chain_premium, _strike_premium, OP).map_err(curve_math)?;
                strike_count = d_add(strike_count, Decimal::ONE, OP).map_err(curve_math)?;
            }
        }
        // If there are no points calculated return an error
        if points.is_empty() {
            return Err(CurveError::ConstructionError(
                "No options with valid premium weighted Strike Concentration data".to_string(),
            ));
        }
        // Calculate the average chain premium
        // `strike_count >= 1` here; a book whose premia are all zero has a
        // zero average, and no concentration to normalise by (#788).
        let average_premium = d_div(
            total_chain_premium,
            strike_count,
            "metrics::chain::premium_concentration",
        )
        .map_err(curve_math)?;
        // Normalize Strike Concentration data points based on average premium
        let mut points_normalized = BTreeSet::new();
        for point in points.iter() {
            let normalized = d_div(
                point.y,
                average_premium,
                "metrics::chain::premium_concentration",
            )
            .map_err(curve_math)?;
            points_normalized.insert(Point2D::new(point.x, normalized));
        }

        Ok(Curve::new(points_normalized))
    }
}

impl ImpliedVolatilitySurface for OptionChain {
    /// Computes the implied volatility surface (strike vs time) for this option chain.
    ///
    /// Creates a 3D surface showing how implied volatility varies across both strike
    /// prices and time to expiration. The IV is scaled using the square root of time
    /// rule to project values across different time horizons.
    ///
    /// # Parameters
    ///
    /// - `days_to_expiry`: Vector of days to expiration values to include in the surface
    ///
    /// # Returns
    ///
    /// - `Ok(Surface)`: A surface with strike on x-axis, days on y-axis, and IV on z-axis
    /// - `Err(SurfaceError)`: If no valid points can be generated
    ///
    /// # Example
    ///
    /// ```ignore
    /// use optionstratlib_market::chains::chain::OptionChain;
    /// use optionstratlib_analytics::metrics::ImpliedVolatilitySurface;
    /// use optionstratlib_core::pos_or_panic;
    ///
    /// let chain = OptionChain::new("SPY", pos_or_panic!(450.0), "2024-03-15".to_string(), None, None);
    /// let days = vec![pos_or_panic!(7.0), pos_or_panic!(14.0), pos_or_panic!(30.0), pos_or_panic!(60.0)];
    /// let iv_surface = chain.iv_surface(days)?;
    /// ```
    fn iv_surface(&self, days_to_expiry: Vec<Positive>) -> Result<Surface, SurfaceError> {
        let mut points = BTreeSet::new();

        for opt in self.options.iter() {
            if opt.implied_volatility.is_zero() {
                continue;
            }

            for days in &days_to_expiry {
                // Scale IV using square root of time rule
                // This projects the current IV to different time horizons
                // Dividing by a constant above one cannot leave the range.
                let time_factor = d_sqrt(
                    days.to_dec() / dec!(365.0),
                    "metrics::chain::iv_time_factor",
                )
                .map_err(surface_math)?;
                let adjusted_iv = d_mul(
                    opt.implied_volatility.to_dec(),
                    time_factor,
                    "metrics::chain::iv_surface",
                )
                .map_err(surface_math)?;

                points.insert(Point3D::new(
                    opt.strike_price.to_dec(),
                    days.to_dec(),
                    adjusted_iv,
                ));
            }
        }

        if points.is_empty() {
            return Err(SurfaceError::ConstructionError(
                "No valid points for IV surface".to_string(),
            ));
        }

        Ok(Surface::new(points))
    }
}

impl RiskReversalCurve for OptionChain {
    /// Computes the risk reversal curve by strike price for this option chain.
    ///
    /// Risk reversal measures the difference between call and put implied volatilities
    /// at each strike. Since this option chain stores a single IV per strike (typically
    /// the average or ATM-adjusted IV), this implementation calculates the risk reversal
    /// as the deviation from the ATM implied volatility.
    ///
    /// For strikes below ATM: negative values indicate put skew (puts more expensive)
    /// For strikes above ATM: positive values indicate call skew (calls more expensive)
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: A curve with strike prices on x-axis and risk reversal values on y-axis
    /// - `Err(CurveError)`: If insufficient data is available
    ///
    /// # Example
    ///
    /// ```ignore
    /// use optionstratlib_market::chains::chain::OptionChain;
    /// use optionstratlib_analytics::metrics::RiskReversalCurve;
    ///
    /// let chain = OptionChain::new("SPY", pos_or_panic!(450.0), "2024-03-15".to_string(), None, None);
    /// let rr_curve = chain.risk_reversal_curve()?;
    /// ```
    fn risk_reversal_curve(&self) -> Result<Curve, CurveError> {
        // Find ATM IV (closest strike to underlying price)
        let atm_iv = self
            .options
            .iter()
            .filter(|opt| !opt.implied_volatility.is_zero())
            .min_by(|a, b| {
                let diff_a = (a.strike_price.to_dec() - self.underlying_price.to_dec()).abs();
                let diff_b = (b.strike_price.to_dec() - self.underlying_price.to_dec()).abs();
                diff_a.partial_cmp(&diff_b).unwrap_or(Ordering::Equal)
            })
            .map(|opt| opt.implied_volatility.to_dec())
            .ok_or_else(|| {
                CurveError::ConstructionError(
                    "No options with valid implied volatility".to_string(),
                )
            })?;

        // Calculate risk reversal as deviation from ATM IV
        // Positive for OTM calls (strike > spot), negative for OTM puts (strike < spot)
        let points: BTreeSet<Point2D> = self
            .options
            .iter()
            .filter(|opt| !opt.implied_volatility.is_zero())
            .map(|opt| {
                let iv = opt.implied_volatility.to_dec();
                let strike = opt.strike_price.to_dec();
                let spot = self.underlying_price.to_dec();

                // Risk reversal: IV difference weighted by moneyness direction
                let rr = if strike > spot {
                    iv - atm_iv // OTM call premium
                } else if strike < spot {
                    atm_iv - iv // OTM put premium (inverted for standard RR convention)
                } else {
                    Decimal::ZERO // ATM
                };

                Point2D::new(strike, rr)
            })
            .collect();

        if points.is_empty() {
            return Err(CurveError::ConstructionError(
                "No options with valid implied volatility".to_string(),
            ));
        }

        Ok(Curve::new(points))
    }
}

impl DollarGammaCurve for OptionChain {
    /// Computes the dollar gamma curve by strike price for this option chain.
    ///
    /// Dollar gamma measures gamma exposure in monetary terms, showing how much
    /// the delta will change for a 1% move in the underlying price.
    ///
    /// Formula: Dollar Gamma = Gamma × Spot² × 0.01
    ///
    /// # Parameters
    ///
    /// - `option_style`: Whether to compute for calls or puts (gamma is the same
    ///   for both at the same strike, but this allows filtering by option type)
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: A curve with strike prices on x-axis and dollar gamma on y-axis
    /// - `Err(CurveError)`: If no valid gamma values can be computed
    ///
    /// # Example
    ///
    /// ```ignore
    /// use optionstratlib_market::chains::chain::OptionChain;
    /// use optionstratlib_analytics::metrics::DollarGammaCurve;
    /// use optionstratlib_core::model::OptionStyle;
    ///
    /// let chain = OptionChain::new("SPY", pos_or_panic!(450.0), "2024-03-15".to_string(), None, None);
    /// let dg_curve = chain.dollar_gamma_curve(&OptionStyle::Call)?;
    /// ```
    fn dollar_gamma_curve(&self, option_style: &OptionStyle) -> Result<Curve, CurveError> {
        const OP: &str = "metrics::chain::dollar_gamma_curve";
        let spot = self.underlying_price;
        let spot_squared = d_mul(spot.to_dec(), spot.to_dec(), OP).map_err(curve_math)?;

        // A strike whose greek cannot be computed is skipped, as before; an
        // overflow in the dollar scaling is an error, not a missing point.
        let points: BTreeSet<Point2D> = self
            .get_single_iter()
            .filter_map(|opt| {
                let option = match option_style {
                    OptionStyle::Call => opt.get_option(Side::Long, OptionStyle::Call).ok()?,
                    OptionStyle::Put => opt.get_option(Side::Long, OptionStyle::Put).ok()?,
                };

                let gamma = option.gamma().ok()?;
                // Dollar Gamma = Gamma × Spot² × 0.01
                let dollar_gamma = d_mul(gamma, spot_squared, OP)
                    .and_then(|value| d_mul(value, dec!(0.01), OP))
                    .map_err(curve_math);

                Some(dollar_gamma.map(|value| Point2D::new(opt.strike_price.to_dec(), value)))
            })
            .collect::<Result<_, _>>()?;

        if points.is_empty() {
            return Err(CurveError::ConstructionError(
                "No valid gamma values computed".to_string(),
            ));
        }

        Ok(Curve::new(points))
    }
}

impl VannaVolgaSurface for OptionChain {
    /// Computes the Vanna-Volga hedge surface (price vs volatility) for this option chain.
    ///
    /// The Vanna-Volga method accounts for the volatility smile by computing hedge
    /// costs across different underlying prices and volatility levels. This surface
    /// helps traders understand where hedging costs are highest and how smile effects
    /// impact pricing.
    ///
    /// # Parameters
    ///
    /// - `price_range`: Tuple of (min_price, max_price) for the underlying
    /// - `vol_range`: Tuple of (min_vol, max_vol) for implied volatility
    /// - `price_steps`: Number of steps along the price axis
    /// - `vol_steps`: Number of steps along the volatility axis
    ///
    /// # Returns
    ///
    /// - `Ok(Surface)`: The Vanna-Volga surface with price on x-axis,
    ///   volatility on y-axis, and hedge cost on z-axis
    /// - `Err(SurfaceError)`: If the surface cannot be computed
    fn vanna_volga_surface(
        &self,
        price_range: (Positive, Positive),
        vol_range: (Positive, Positive),
        price_steps: usize,
        vol_steps: usize,
    ) -> Result<Surface, SurfaceError> {
        let mut points = BTreeSet::new();

        // Get ATM volatility as reference
        let atm_vol = self
            .options
            .iter()
            .filter(|opt| !opt.implied_volatility.is_zero())
            .min_by(|a, b| {
                let diff_a = (a.strike_price.to_dec() - self.underlying_price.to_dec()).abs();
                let diff_b = (b.strike_price.to_dec() - self.underlying_price.to_dec()).abs();
                diff_a.partial_cmp(&diff_b).unwrap_or(Ordering::Equal)
            })
            .map(|opt| opt.implied_volatility.to_dec())
            .unwrap_or(dec!(0.20));

        let price_step = grid_step(price_range, price_steps, "price_range")?;
        let vol_step = grid_step(vol_range, vol_steps, "vol_range")?;

        const OP: &str = "metrics::chain::vanna_volga_surface";
        for p in 0..=price_steps {
            let price = grid_point(price_range.0, price_step, p)?;

            for v in 0..=vol_steps {
                let vol = grid_point(vol_range.0, vol_step, v)?;

                // Vanna-Volga cost model:
                // Cost increases with distance from ATM and with volatility difference.
                // Both differences are of two non-negative values, so they
                // stay in range; the ratio and the products may not (#788).
                let moneyness = d_div(
                    (price - self.underlying_price.to_dec()).abs(),
                    self.underlying_price.to_dec(),
                    OP,
                )
                .map_err(surface_math)?;
                let vol_diff = (vol - atm_vol).abs();

                // Simplified Vanna-Volga cost: combines moneyness and vol effects
                // Vanna component: moneyness × vol_diff
                // Volga component: vol_diff²
                let vanna_cost = d_mul(moneyness, vol_diff, OP)
                    .and_then(|value| d_mul(value, dec!(100.0), OP))
                    .map_err(surface_math)?;
                let volga_cost = d_mul(vol_diff, vol_diff, OP)
                    .and_then(|value| d_mul(value, dec!(50.0), OP))
                    .map_err(surface_math)?;
                let vv_cost = d_add(vanna_cost, volga_cost, OP).map_err(surface_math)?;

                points.insert(Point3D::new(price, vol, vv_cost));
            }
        }

        if points.is_empty() {
            return Err(SurfaceError::ConstructionError(
                "No valid points for Vanna-Volga surface".to_string(),
            ));
        }

        Ok(Surface::new(points))
    }
}

impl DeltaGammaProfileCurve for OptionChain {
    /// Computes the delta-gamma profile curve by strike price for this option chain.
    ///
    /// Shows the combined delta and gamma exposure at each strike, helping identify
    /// where directional and convexity risks are concentrated.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The profile curve with strike on x-axis and combined
    ///   delta-gamma metric (dollar delta + dollar gamma) on y-axis
    /// - `Err(CurveError)`: If the curve cannot be computed
    fn delta_gamma_curve(&self) -> Result<Curve, CurveError> {
        const OP: &str = "metrics::chain::delta_gamma_curve";
        let spot = self.underlying_price.to_dec();
        let spot_squared = d_mul(spot, spot, OP).map_err(curve_math)?;

        // A strike whose greeks cannot be computed is skipped, as before; an
        // overflow in the dollar scaling is an error, not a missing point.
        let points: BTreeSet<Point2D> = self
            .get_single_iter()
            .filter_map(|opt| {
                // Use call options for the profile
                let option = opt.get_option(Side::Long, OptionStyle::Call).ok()?;

                let delta = option.delta().ok()?;
                let gamma = option.gamma().ok()?;

                let combined = d_mul(delta, spot, OP).and_then(|dollar_delta| {
                    // Dollar Delta = Delta × Spot
                    // Dollar Gamma = Gamma × Spot² × 0.01
                    let dollar_gamma = d_mul(gamma, spot_squared, OP)
                        .and_then(|value| d_mul(value, dec!(0.01), OP))?;
                    // Combined metric
                    d_add(dollar_delta, dollar_gamma, OP)
                });

                Some(
                    combined
                        .map(|value| Point2D::new(opt.strike_price.to_dec(), value))
                        .map_err(curve_math),
                )
            })
            .collect::<Result<_, _>>()?;

        if points.is_empty() {
            return Err(CurveError::ConstructionError(
                "No valid delta-gamma values computed".to_string(),
            ));
        }

        Ok(Curve::new(points))
    }
}

impl DeltaGammaProfileSurface for OptionChain {
    /// Computes the delta-gamma profile surface (price vs time) for this option chain.
    ///
    /// Shows how delta exposure varies across both underlying price and time to
    /// expiration, providing a complete view of risk evolution.
    ///
    /// # Parameters
    ///
    /// - `price_range`: Tuple of (min_price, max_price) for the underlying
    /// - `days_to_expiry`: Vector of days to expiration values
    /// - `price_steps`: Number of steps along the price axis
    ///
    /// # Returns
    ///
    /// - `Ok(Surface)`: The profile surface with price on x-axis,
    ///   days on y-axis, and delta exposure on z-axis
    /// - `Err(SurfaceError)`: If the surface cannot be computed
    fn delta_gamma_surface(
        &self,
        price_range: (Positive, Positive),
        days_to_expiry: Vec<Positive>,
        price_steps: usize,
    ) -> Result<Surface, SurfaceError> {
        let mut points = BTreeSet::new();

        let price_step = grid_step(price_range, price_steps, "price_range")?;

        // Get a representative option to use as template
        let template_opt = self
            .get_single_iter()
            .find_map(|opt| opt.get_option(Side::Long, OptionStyle::Call).ok());

        let template = match template_opt {
            Some(opt) => opt,
            None => {
                return Err(SurfaceError::ConstructionError(
                    "No valid options in chain".to_string(),
                ));
            }
        };

        for days in &days_to_expiry {
            for p in 0..=price_steps {
                let price = grid_point(price_range.0, price_step, p)?;
                let price_pos = Positive::new_decimal(price).map_err(surface_math)?;

                // Create option with modified price and expiration
                let modified_option = Options::new(
                    template.option_type.clone(),
                    template.side,
                    template.underlying_symbol.clone(),
                    template.strike_price,
                    ExpirationDate::Days(*days),
                    template.implied_volatility,
                    template.quantity,
                    price_pos,
                    template.risk_free_rate,
                    template.option_style,
                    template.dividend_yield,
                    template.exotic_params.clone(),
                )
                .with_contract_size(template.contract_size);

                if let Ok(delta) = modified_option.delta() {
                    points.insert(Point3D::new(price, days.to_dec(), delta));
                }
            }
        }

        if points.is_empty() {
            return Err(SurfaceError::ConstructionError(
                "No valid points for delta-gamma surface".to_string(),
            ));
        }

        Ok(Surface::new(points))
    }
}

impl SmileDynamicsCurve for OptionChain {
    /// Computes the smile dynamics curve by strike price for this option chain.
    ///
    /// Shows the current shape of the volatility smile, representing how implied
    /// volatility varies across strike prices.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The smile curve with strike on x-axis and IV on y-axis
    /// - `Err(CurveError)`: If the curve cannot be computed
    fn smile_dynamics_curve(&self) -> Result<Curve, CurveError> {
        let points: BTreeSet<Point2D> = self
            .options
            .iter()
            .filter(|opt| !opt.implied_volatility.is_zero())
            .map(|opt| Point2D::new(opt.strike_price.to_dec(), opt.implied_volatility.to_dec()))
            .collect();

        if points.is_empty() {
            return Err(CurveError::ConstructionError(
                "No options with valid implied volatility".to_string(),
            ));
        }

        Ok(Curve::new(points))
    }
}

impl SmileDynamicsSurface for OptionChain {
    /// Computes the smile dynamics surface (strike vs time) for this option chain.
    ///
    /// Shows how the volatility smile evolves across different time horizons,
    /// providing insights into term structure and smile dynamics.
    ///
    /// # Parameters
    ///
    /// - `days_to_expiry`: Vector of days to expiration values to include
    ///
    /// # Returns
    ///
    /// - `Ok(Surface)`: The smile surface with strike on x-axis,
    ///   days on y-axis, and IV on z-axis
    /// - `Err(SurfaceError)`: If the surface cannot be computed
    fn smile_dynamics_surface(
        &self,
        days_to_expiry: Vec<Positive>,
    ) -> Result<Surface, SurfaceError> {
        let mut points = BTreeSet::new();

        // Get ATM volatility for reference
        let atm_vol = self
            .options
            .iter()
            .filter(|opt| !opt.implied_volatility.is_zero())
            .min_by(|a, b| {
                let diff_a = (a.strike_price.to_dec() - self.underlying_price.to_dec()).abs();
                let diff_b = (b.strike_price.to_dec() - self.underlying_price.to_dec()).abs();
                diff_a.partial_cmp(&diff_b).unwrap_or(Ordering::Equal)
            })
            .map(|opt| opt.implied_volatility.to_dec())
            .unwrap_or(dec!(0.20));

        for opt in self.options.iter() {
            if opt.implied_volatility.is_zero() {
                continue;
            }

            let strike = opt.strike_price.to_dec();
            let base_iv = opt.implied_volatility.to_dec();

            // Calculate skew from current smile
            let skew = base_iv - atm_vol;

            for days in &days_to_expiry {
                // Smile dynamics: skew steepens for shorter expirations
                let time_factor = d_sqrt(
                    days.to_dec() / dec!(30.0),
                    "metrics::chain::skew_time_factor",
                )
                .map_err(surface_math)?;
                let adjusted_skew = if time_factor > Decimal::ZERO {
                    d_div(skew, time_factor, "metrics::chain::smile_dynamics_surface")
                        .map_err(surface_math)?
                } else {
                    skew
                };

                let adjusted_iv = d_add(
                    atm_vol,
                    adjusted_skew,
                    "metrics::chain::smile_dynamics_surface",
                )
                .map_err(surface_math)?;
                let final_iv = adjusted_iv.max(dec!(0.01)); // Ensure positive IV

                points.insert(Point3D::new(strike, days.to_dec(), final_iv));
            }
        }

        if points.is_empty() {
            return Err(SurfaceError::ConstructionError(
                "No valid points for smile dynamics surface".to_string(),
            ));
        }

        Ok(Surface::new(points))
    }
}

impl BidAskSpreadCurve for OptionChain {
    /// Computes the bid-ask spread curve by strike price for this option chain.
    ///
    /// Shows how liquidity varies across different strike prices by calculating
    /// the relative spread (spread / mid price) at each strike.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The spread curve with strike on x-axis and relative spread on y-axis
    /// - `Err(CurveError)`: If no valid bid/ask data is available
    fn bid_ask_spread_curve(&self) -> Result<Curve, CurveError> {
        let mut points = BTreeSet::new();

        for opt in self.options.iter() {
            // Calculate call spread if available, else the put spread
            let quote = match (opt.call_bid, opt.call_ask) {
                (Some(bid), Some(ask)) => Some((bid, ask)),
                _ => opt.put_bid.zip(opt.put_ask),
            };
            if let Some((bid, ask)) = quote
                && let Some(spread) = relative_spread(opt.strike_price, bid, ask)?
            {
                points.insert(Point2D::new(opt.strike_price.to_dec(), spread));
            }
        }

        if points.is_empty() {
            return Err(CurveError::ConstructionError(
                "No options with valid bid/ask data".to_string(),
            ));
        }

        Ok(Curve::new(points))
    }
}

/// Relative spread `(ask - bid) / mid` of one quote, `None` at a zero mid.
///
/// # Errors
///
/// Returns [`CurveError::OperationError`] for a crossed quote (`ask < bid`),
/// which has no non-negative spread, and [`CurveError::MetricsError`] when
/// `bid + ask` leaves the `Decimal` range. Both used to abort in the
/// `Positive` operators (#788).
fn relative_spread(
    strike: Positive,
    bid: Positive,
    ask: Positive,
) -> Result<Option<Decimal>, CurveError> {
    let mid = bid.checked_add(&ask).map_err(curve_math)? / Positive::TWO;
    if mid == Positive::ZERO {
        return Ok(None);
    }
    let width = ask.checked_sub(&bid).map_err(|_| {
        CurveError::invalid_parameters(
            "bid_ask_spread_curve",
            &format!("crossed quote at strike {strike}: ask {ask} is below bid {bid}"),
        )
    })?;
    d_div(
        width.to_dec(),
        mid.to_dec(),
        "metrics::chain::bid_ask_spread_curve",
    )
    .map(Some)
    .map_err(curve_math)
}

impl VolumeProfileCurve for OptionChain {
    /// Computes the volume profile curve by strike price for this option chain.
    ///
    /// Shows how trading activity is distributed across different strike prices.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The volume curve with strike on x-axis and volume on y-axis
    /// - `Err(CurveError)`: If no valid volume data is available
    fn volume_profile_curve(&self) -> Result<Curve, CurveError> {
        let points: BTreeSet<Point2D> = self
            .options
            .iter()
            .filter_map(|opt| {
                opt.volume
                    .map(|vol| Point2D::new(opt.strike_price.to_dec(), vol.to_dec()))
            })
            .collect();

        if points.is_empty() {
            return Err(CurveError::ConstructionError(
                "No options with valid volume data".to_string(),
            ));
        }

        Ok(Curve::new(points))
    }
}

impl VolumeProfileSurface for OptionChain {
    /// Computes the volume profile surface (strike vs time) for this option chain.
    ///
    /// Projects volume across different time horizons, simulating how volume
    /// typically increases closer to expiration.
    ///
    /// # Parameters
    ///
    /// - `days`: Vector of time points (in days) to include in the surface
    ///
    /// # Returns
    ///
    /// - `Ok(Surface)`: The volume surface with strike on x-axis, days on y-axis,
    ///   and volume on z-axis
    /// - `Err(SurfaceError)`: If no valid volume data is available
    fn volume_profile_surface(&self, days: Vec<Positive>) -> Result<Surface, SurfaceError> {
        let mut points = BTreeSet::new();

        for opt in self.options.iter() {
            if let Some(base_vol) = opt.volume {
                for day in &days {
                    // Volume typically increases closer to expiration
                    // Using a simple model: volume scales inversely with sqrt(time)
                    const OP: &str = "metrics::chain::volume_profile_surface";
                    let time_factor = if day.to_dec() > Decimal::ZERO {
                        d_sqrt(
                            d_div(dec!(30.0), day.to_dec(), OP).map_err(surface_math)?,
                            "metrics::chain::volume_time_factor",
                        )
                        .map_err(surface_math)?
                    } else {
                        Decimal::ONE
                    };

                    let adjusted_vol =
                        d_mul(base_vol.to_dec(), time_factor, OP).map_err(surface_math)?;
                    points.insert(Point3D::new(
                        opt.strike_price.to_dec(),
                        day.to_dec(),
                        adjusted_vol,
                    ));
                }
            }
        }

        if points.is_empty() {
            return Err(SurfaceError::ConstructionError(
                "No valid points for volume profile surface".to_string(),
            ));
        }

        Ok(Surface::new(points))
    }
}

impl OpenInterestCurve for OptionChain {
    /// Computes the open interest distribution curve by strike price for this option chain.
    ///
    /// Shows how outstanding contracts are distributed across different strike prices,
    /// helping identify key levels and market positioning.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The OI curve with strike on x-axis and open interest on y-axis
    /// - `Err(CurveError)`: If no valid open interest data is available
    fn open_interest_curve(&self) -> Result<Curve, CurveError> {
        let points: BTreeSet<Point2D> = self
            .options
            .iter()
            .filter_map(|opt| {
                opt.open_interest
                    .map(|oi| Point2D::new(opt.strike_price.to_dec(), Decimal::from(oi)))
            })
            .collect();

        if points.is_empty() {
            return Err(CurveError::ConstructionError(
                "No options with valid open interest data".to_string(),
            ));
        }

        Ok(Curve::new(points))
    }
}

impl VolatilitySensitivityCurve for OptionChain {
    /// Computes the volatility sensitivity curve by strike price for this option chain.
    ///
    /// Shows vega exposure at each strike, helping identify where volatility
    /// risk is concentrated.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The vega curve with strike on x-axis and vega on y-axis
    /// - `Err(CurveError)`: If no valid vega data is available
    fn volatility_sensitivity_curve(&self) -> Result<Curve, CurveError> {
        let points: BTreeSet<Point2D> = self
            .get_single_iter()
            .filter_map(|opt| {
                let option = opt.get_option(Side::Long, OptionStyle::Call).ok()?;
                let vega = option.vega().ok()?;
                Some(Point2D::new(opt.strike_price.to_dec(), vega))
            })
            .collect();

        if points.is_empty() {
            return Err(CurveError::ConstructionError(
                "No options with valid vega data".to_string(),
            ));
        }

        Ok(Curve::new(points))
    }
}

impl VolatilitySensitivitySurface for OptionChain {
    /// Computes the volatility sensitivity surface (price vs volatility).
    ///
    /// Shows how option value changes across both underlying price and
    /// volatility levels.
    ///
    /// # Parameters
    ///
    /// - `price_range`: Tuple of (min_price, max_price) for the underlying
    /// - `vol_range`: Tuple of (min_vol, max_vol) for implied volatility
    /// - `price_steps`: Number of steps along the price axis
    /// - `vol_steps`: Number of steps along the volatility axis
    ///
    /// # Returns
    ///
    /// - `Ok(Surface)`: The sensitivity surface
    /// - `Err(SurfaceError)`: If the surface cannot be computed
    fn volatility_sensitivity_surface(
        &self,
        price_range: (Positive, Positive),
        vol_range: (Positive, Positive),
        price_steps: usize,
        vol_steps: usize,
    ) -> Result<Surface, SurfaceError> {
        let mut points = BTreeSet::new();

        let price_step = grid_step(price_range, price_steps, "price_range")?;
        let vol_step = grid_step(vol_range, vol_steps, "vol_range")?;

        // Get a representative option to use as template
        let template_opt = self
            .get_single_iter()
            .find_map(|opt| opt.get_option(Side::Long, OptionStyle::Call).ok());

        let template = match template_opt {
            Some(opt) => opt,
            None => {
                return Err(SurfaceError::ConstructionError(
                    "No valid options in chain".to_string(),
                ));
            }
        };

        // Grid points are `lower + step * i` with both terms non-negative, so
        // the `Positive` conversions below cannot fail on a finite grid; they
        // used to fall back to a made-up spot of 1 and volatility of 0.01,
        // and now report instead (#639, #788).
        for p in 0..=price_steps {
            let price = grid_point(price_range.0, price_step, p)?;
            let price_pos = Positive::new_decimal(price).map_err(surface_math)?;

            for v in 0..=vol_steps {
                let vol = grid_point(vol_range.0, vol_step, v)?;
                let vol_pos = Positive::new_decimal(vol).map_err(surface_math)?;

                let modified_option = Options::new(
                    template.option_type.clone(),
                    template.side,
                    template.underlying_symbol.clone(),
                    template.strike_price,
                    template.expiration_date,
                    vol_pos,
                    template.quantity,
                    price_pos,
                    template.risk_free_rate,
                    template.option_style,
                    template.dividend_yield,
                    template.exotic_params.clone(),
                )
                .with_contract_size(template.contract_size);

                if let Ok(option_price) = modified_option.calculate_price_black_scholes() {
                    points.insert(Point3D::new(price, vol, option_price));
                }
            }
        }

        if points.is_empty() {
            return Err(SurfaceError::ConstructionError(
                "No valid points for volatility sensitivity surface".to_string(),
            ));
        }

        Ok(Surface::new(points))
    }
}

impl TimeDecayCurve for OptionChain {
    /// Computes the time decay profile curve by strike price for this option chain.
    ///
    /// Shows theta at each strike, helping identify where time decay
    /// exposure is concentrated.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The theta curve with strike on x-axis and theta on y-axis
    /// - `Err(CurveError)`: If no valid theta data is available
    fn time_decay_curve(&self) -> Result<Curve, CurveError> {
        let points: BTreeSet<Point2D> = self
            .get_single_iter()
            .filter_map(|opt| {
                let option = opt.get_option(Side::Long, OptionStyle::Call).ok()?;
                let theta = option.theta().ok()?;
                Some(Point2D::new(opt.strike_price.to_dec(), theta))
            })
            .collect();

        if points.is_empty() {
            return Err(CurveError::ConstructionError(
                "No options with valid theta data".to_string(),
            ));
        }

        Ok(Curve::new(points))
    }
}

impl TimeDecaySurface for OptionChain {
    /// Computes the time decay profile surface (price vs time).
    ///
    /// Shows how option value evolves across both underlying price and
    /// time to expiration.
    ///
    /// # Parameters
    ///
    /// - `price_range`: Tuple of (min_price, max_price) for the underlying
    /// - `days_to_expiry`: Vector of days to expiration values
    /// - `price_steps`: Number of steps along the price axis
    ///
    /// # Returns
    ///
    /// - `Ok(Surface)`: The decay surface
    /// - `Err(SurfaceError)`: If the surface cannot be computed
    fn time_decay_surface(
        &self,
        price_range: (Positive, Positive),
        days_to_expiry: Vec<Positive>,
        price_steps: usize,
    ) -> Result<Surface, SurfaceError> {
        let mut points = BTreeSet::new();

        let price_step = grid_step(price_range, price_steps, "price_range")?;

        // Get a representative option to use as template
        let template_opt = self
            .get_single_iter()
            .find_map(|opt| opt.get_option(Side::Long, OptionStyle::Call).ok());

        let template = match template_opt {
            Some(opt) => opt,
            None => {
                return Err(SurfaceError::ConstructionError(
                    "No valid options in chain".to_string(),
                ));
            }
        };

        for days in &days_to_expiry {
            for p in 0..=price_steps {
                let price = grid_point(price_range.0, price_step, p)?;
                let price_pos = Positive::new_decimal(price).map_err(surface_math)?;

                let modified_option = Options::new(
                    template.option_type.clone(),
                    template.side,
                    template.underlying_symbol.clone(),
                    template.strike_price,
                    ExpirationDate::Days(*days),
                    template.implied_volatility,
                    template.quantity,
                    price_pos,
                    template.risk_free_rate,
                    template.option_style,
                    template.dividend_yield,
                    template.exotic_params.clone(),
                )
                .with_contract_size(template.contract_size);

                if let Ok(option_price) = modified_option.calculate_price_black_scholes() {
                    points.insert(Point3D::new(price, days.to_dec(), option_price));
                }
            }
        }

        if points.is_empty() {
            return Err(SurfaceError::ConstructionError(
                "No valid points for time decay surface".to_string(),
            ));
        }

        Ok(Surface::new(points))
    }
}

impl PriceShockCurve for OptionChain {
    /// Computes the price shock impact curve by strike price for this option chain.
    ///
    /// Shows P&L impact from a price shock at each strike.
    ///
    /// # Parameters
    ///
    /// - `shock_pct`: Price shock as a decimal (e.g., -0.10 for -10%)
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The shock curve with strike on x-axis and P&L on y-axis
    /// - `Err(CurveError)`: If no valid delta/gamma data is available
    fn price_shock_curve(&self, shock_pct: Decimal) -> Result<Curve, CurveError> {
        const OP: &str = "metrics::chain::price_shock_curve";
        let spot = self.underlying_price.to_dec();
        let price_move = d_mul(spot, shock_pct, OP).map_err(curve_math)?;

        // A strike whose greeks cannot be computed is skipped, as before; an
        // overflow in the Taylor expansion is an error, not a missing point.
        let points: BTreeSet<Point2D> = self
            .get_single_iter()
            .filter_map(|opt| {
                let option = opt.get_option(Side::Long, OptionStyle::Call).ok()?;
                let delta = option.delta().ok()?;
                let gamma = option.gamma().ok()?;

                // P&L = Delta × ΔS + 0.5 × Gamma × ΔS²
                let pnl = d_mul(delta, price_move, OP).and_then(|first_order| {
                    let second_order = d_mul(dec!(0.5), gamma, OP)
                        .and_then(|value| d_mul(value, price_move, OP))
                        .and_then(|value| d_mul(value, price_move, OP))?;
                    d_add(first_order, second_order, OP)
                });

                Some(
                    pnl.map(|value| Point2D::new(opt.strike_price.to_dec(), value))
                        .map_err(curve_math),
                )
            })
            .collect::<Result<_, _>>()?;

        if points.is_empty() {
            return Err(CurveError::ConstructionError(
                "No options with valid delta/gamma data".to_string(),
            ));
        }

        Ok(Curve::new(points))
    }
}

impl PriceShockSurface for OptionChain {
    /// Computes the price shock impact surface (price vs volatility).
    ///
    /// Shows option value across combined price and volatility scenarios.
    ///
    /// # Parameters
    ///
    /// - `price_range`: Tuple of (min_price, max_price) for the underlying
    /// - `vol_range`: Tuple of (min_vol, max_vol) for implied volatility
    /// - `price_steps`: Number of steps along the price axis
    /// - `vol_steps`: Number of steps along the volatility axis
    ///
    /// # Returns
    ///
    /// - `Ok(Surface)`: The shock surface
    /// - `Err(SurfaceError)`: If the surface cannot be computed
    fn price_shock_surface(
        &self,
        price_range: (Positive, Positive),
        vol_range: (Positive, Positive),
        price_steps: usize,
        vol_steps: usize,
    ) -> Result<Surface, SurfaceError> {
        // This is essentially the same as volatility_sensitivity_surface
        // but conceptually represents stress scenarios
        self.volatility_sensitivity_surface(price_range, vol_range, price_steps, vol_steps)
    }
}

impl ThetaCurve for OptionChain {
    /// Computes the theta curve by strike price for this option chain.
    ///
    /// Shows theta (time decay) at each strike using the existing Greeks calculations.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The theta curve with strike on x-axis and theta on y-axis
    /// - `Err(CurveError)`: If no valid theta data is available
    fn theta_curve(&self) -> Result<Curve, CurveError> {
        let points: BTreeSet<Point2D> = self
            .get_single_iter()
            .filter_map(|opt| {
                let option = opt.get_option(Side::Long, OptionStyle::Call).ok()?;
                let theta = option.theta().ok()?;
                Some(Point2D::new(opt.strike_price.to_dec(), theta))
            })
            .collect();

        if points.is_empty() {
            return Err(CurveError::ConstructionError(
                "No options with valid theta data".to_string(),
            ));
        }

        Ok(Curve::new(points))
    }
}

impl ThetaSurface for OptionChain {
    /// Computes the theta surface (price vs time) for this option chain.
    ///
    /// Shows how theta evolves across both underlying price and time to expiration.
    ///
    /// # Parameters
    ///
    /// - `price_range`: Tuple of (min_price, max_price) for the underlying
    /// - `days_to_expiry`: Vector of days to expiration values
    /// - `price_steps`: Number of steps along the price axis
    ///
    /// # Returns
    ///
    /// - `Ok(Surface)`: The theta surface
    /// - `Err(SurfaceError)`: If the surface cannot be computed
    fn theta_surface(
        &self,
        price_range: (Positive, Positive),
        days_to_expiry: Vec<Positive>,
        price_steps: usize,
    ) -> Result<Surface, SurfaceError> {
        let mut points = BTreeSet::new();

        let price_step = grid_step(price_range, price_steps, "price_range")?;

        let template_opt = self
            .get_single_iter()
            .find_map(|opt| opt.get_option(Side::Long, OptionStyle::Call).ok());

        let template = match template_opt {
            Some(opt) => opt,
            None => {
                return Err(SurfaceError::ConstructionError(
                    "No valid options in chain".to_string(),
                ));
            }
        };

        for days in &days_to_expiry {
            for p in 0..=price_steps {
                let price = grid_point(price_range.0, price_step, p)?;
                let price_pos = Positive::new_decimal(price).map_err(surface_math)?;

                let modified_option = Options::new(
                    template.option_type.clone(),
                    template.side,
                    template.underlying_symbol.clone(),
                    template.strike_price,
                    ExpirationDate::Days(*days),
                    template.implied_volatility,
                    template.quantity,
                    price_pos,
                    template.risk_free_rate,
                    template.option_style,
                    template.dividend_yield,
                    template.exotic_params.clone(),
                )
                .with_contract_size(template.contract_size);

                if let Ok(theta) = modified_option.theta() {
                    points.insert(Point3D::new(price, days.to_dec(), theta));
                }
            }
        }

        if points.is_empty() {
            return Err(SurfaceError::ConstructionError(
                "No valid points for theta surface".to_string(),
            ));
        }

        Ok(Surface::new(points))
    }
}

impl CharmCurve for OptionChain {
    /// Computes the charm curve by strike price for this option chain.
    ///
    /// Shows charm (delta decay) at each strike using the existing Greeks calculations.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The charm curve with strike on x-axis and charm on y-axis
    /// - `Err(CurveError)`: If no valid charm data is available
    fn charm_curve(&self) -> Result<Curve, CurveError> {
        let points: BTreeSet<Point2D> = self
            .get_single_iter()
            .filter_map(|opt| {
                let option = opt.get_option(Side::Long, OptionStyle::Call).ok()?;
                let charm = option.charm().ok()?;
                Some(Point2D::new(opt.strike_price.to_dec(), charm))
            })
            .collect();

        if points.is_empty() {
            return Err(CurveError::ConstructionError(
                "No options with valid charm data".to_string(),
            ));
        }

        Ok(Curve::new(points))
    }
}

impl CharmSurface for OptionChain {
    /// Computes the charm surface (price vs time) for this option chain.
    ///
    /// Shows how charm evolves across both underlying price and time to expiration.
    ///
    /// # Parameters
    ///
    /// - `price_range`: Tuple of (min_price, max_price) for the underlying
    /// - `days_to_expiry`: Vector of days to expiration values
    /// - `price_steps`: Number of steps along the price axis
    ///
    /// # Returns
    ///
    /// - `Ok(Surface)`: The charm surface
    /// - `Err(SurfaceError)`: If the surface cannot be computed
    fn charm_surface(
        &self,
        price_range: (Positive, Positive),
        days_to_expiry: Vec<Positive>,
        price_steps: usize,
    ) -> Result<Surface, SurfaceError> {
        let mut points = BTreeSet::new();

        let price_step = grid_step(price_range, price_steps, "price_range")?;

        let template_opt = self
            .get_single_iter()
            .find_map(|opt| opt.get_option(Side::Long, OptionStyle::Call).ok());

        let template = match template_opt {
            Some(opt) => opt,
            None => {
                return Err(SurfaceError::ConstructionError(
                    "No valid options in chain".to_string(),
                ));
            }
        };

        for days in &days_to_expiry {
            for p in 0..=price_steps {
                let price = grid_point(price_range.0, price_step, p)?;
                let price_pos = Positive::new_decimal(price).map_err(surface_math)?;

                let modified_option = Options::new(
                    template.option_type.clone(),
                    template.side,
                    template.underlying_symbol.clone(),
                    template.strike_price,
                    ExpirationDate::Days(*days),
                    template.implied_volatility,
                    template.quantity,
                    price_pos,
                    template.risk_free_rate,
                    template.option_style,
                    template.dividend_yield,
                    template.exotic_params.clone(),
                )
                .with_contract_size(template.contract_size);

                if let Ok(charm) = modified_option.charm() {
                    points.insert(Point3D::new(price, days.to_dec(), charm));
                }
            }
        }

        if points.is_empty() {
            return Err(SurfaceError::ConstructionError(
                "No valid points for charm surface".to_string(),
            ));
        }

        Ok(Surface::new(points))
    }
}

impl ColorCurve for OptionChain {
    /// Computes the color curve by strike price for this option chain.
    ///
    /// Shows color (gamma decay) at each strike using the existing Greeks calculations.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The color curve with strike on x-axis and color on y-axis
    /// - `Err(CurveError)`: If no valid color data is available
    fn color_curve(&self) -> Result<Curve, CurveError> {
        let points: BTreeSet<Point2D> = self
            .get_single_iter()
            .filter_map(|opt| {
                let option = opt.get_option(Side::Long, OptionStyle::Call).ok()?;
                let color = option.color().ok()?;
                Some(Point2D::new(opt.strike_price.to_dec(), color))
            })
            .collect();

        if points.is_empty() {
            return Err(CurveError::ConstructionError(
                "No options with valid color data".to_string(),
            ));
        }

        Ok(Curve::new(points))
    }
}

impl ColorSurface for OptionChain {
    /// Computes the color surface (price vs time) for this option chain.
    ///
    /// Shows how color evolves across both underlying price and time to expiration.
    ///
    /// # Parameters
    ///
    /// - `price_range`: Tuple of (min_price, max_price) for the underlying
    /// - `days_to_expiry`: Vector of days to expiration values
    /// - `price_steps`: Number of steps along the price axis
    ///
    /// # Returns
    ///
    /// - `Ok(Surface)`: The color surface
    /// - `Err(SurfaceError)`: If the surface cannot be computed
    fn color_surface(
        &self,
        price_range: (Positive, Positive),
        days_to_expiry: Vec<Positive>,
        price_steps: usize,
    ) -> Result<Surface, SurfaceError> {
        let mut points = BTreeSet::new();

        let price_step = grid_step(price_range, price_steps, "price_range")?;

        let template_opt = self
            .get_single_iter()
            .find_map(|opt| opt.get_option(Side::Long, OptionStyle::Call).ok());

        let template = match template_opt {
            Some(opt) => opt,
            None => {
                return Err(SurfaceError::ConstructionError(
                    "No valid options in chain".to_string(),
                ));
            }
        };

        for days in &days_to_expiry {
            for p in 0..=price_steps {
                let price = grid_point(price_range.0, price_step, p)?;
                let price_pos = Positive::new_decimal(price).map_err(surface_math)?;

                let modified_option = Options::new(
                    template.option_type.clone(),
                    template.side,
                    template.underlying_symbol.clone(),
                    template.strike_price,
                    ExpirationDate::Days(*days),
                    template.implied_volatility,
                    template.quantity,
                    price_pos,
                    template.risk_free_rate,
                    template.option_style,
                    template.dividend_yield,
                    template.exotic_params.clone(),
                )
                .with_contract_size(template.contract_size);

                if let Ok(color) = modified_option.color() {
                    points.insert(Point3D::new(price, days.to_dec(), color));
                }
            }
        }

        if points.is_empty() {
            return Err(SurfaceError::ConstructionError(
                "No valid points for color surface".to_string(),
            ));
        }

        Ok(Surface::new(points))
    }
}

#[cfg(test)]
mod tests_price_metrics_traits {
    #![allow(clippy::indexing_slicing)]
    use super::*;
    use optionstratlib_core::assert_decimal_eq;

    #[test]
    fn test_put_call_ratio_premium_weighted_trait() {
        let result = OptionChain::load_from_json(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/Chains/SP500-18-oct-2024-5781.88.json"
        ));
        assert!(result.is_ok());
        let chain = result.unwrap();
        let put_call_ratio_curve = chain.premium_weighted_pcr().unwrap();
        let pcr_vec: Vec<_> = put_call_ratio_curve.points.iter().collect();

        // Check specific points for correct put call ratio calculation
        let epsilon = dec!(1e-5);
        assert_decimal_eq!(pcr_vec[0].y, dec!(0.05109), epsilon);
        assert_decimal_eq!(pcr_vec[1].y, dec!(0.05324), epsilon);
        assert_decimal_eq!(pcr_vec[2].y, dec!(0.05552), epsilon);
        assert_decimal_eq!(pcr_vec[3].y, dec!(0.06046), epsilon);
        assert_decimal_eq!(pcr_vec[4].y, dec!(0.06593), epsilon);
    }

    #[test]
    fn test_strike_concentration_premium_weighted_trait() {
        let result = OptionChain::load_from_json(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/Chains/SP500-18-oct-2024-5781.88.json"
        ));
        assert!(result.is_ok());
        let chain = result.unwrap();
        let strike_concentration_curve = chain.premium_concentration().unwrap();
        let strike_concentration_vec: Vec<_> = strike_concentration_curve.points.iter().collect();

        // Check specific points for correct strike concentration calculation
        let epsilon = dec!(1e-5);
        assert_decimal_eq!(strike_concentration_vec[0].y, dec!(1.44624), epsilon);
        assert_decimal_eq!(strike_concentration_vec[1].y, dec!(1.42477), epsilon);
        assert_decimal_eq!(strike_concentration_vec[2].y, dec!(1.40347), epsilon);
        assert_decimal_eq!(strike_concentration_vec[3].y, dec!(1.36114), epsilon);
        assert_decimal_eq!(strike_concentration_vec[4].y, dec!(1.31928), epsilon);
    }
}

#[cfg(test)]
mod tests_volatility_skew {
    #![allow(clippy::indexing_slicing)]
    use super::*;
    use optionstratlib_core::pos_or_panic;

    fn create_chain_with_options() -> OptionChain {
        let mut chain = OptionChain::new(
            "TEST",
            Positive::HUNDRED,
            "2030-01-01".to_string(),
            None,
            None,
        );

        chain.add_option(
            pos_or_panic!(90.0),
            Some(pos_or_panic!(11.0)),
            Some(pos_or_panic!(11.5)),
            Some(pos_or_panic!(0.5)),
            Some(Positive::ONE),
            pos_or_panic!(0.28),
            Some(dec!(0.85)),
            Some(dec!(-0.15)),
            Some(dec!(0.015)),
            None,
            None,
            None,
        );

        chain.add_option(
            pos_or_panic!(95.0),
            Some(pos_or_panic!(6.5)),
            Some(pos_or_panic!(7.0)),
            Some(Positive::ONE),
            Some(pos_or_panic!(1.5)),
            pos_or_panic!(0.24),
            Some(dec!(0.72)),
            Some(dec!(-0.28)),
            Some(dec!(0.020)),
            None,
            None,
            None,
        );

        chain.add_option(
            Positive::HUNDRED,
            Some(pos_or_panic!(3.5)),
            Some(pos_or_panic!(4.0)),
            Some(pos_or_panic!(3.0)),
            Some(pos_or_panic!(3.5)),
            pos_or_panic!(0.20),
            Some(dec!(0.52)),
            Some(dec!(-0.48)),
            Some(dec!(0.025)),
            None,
            None,
            None,
        );

        chain.add_option(
            pos_or_panic!(105.0),
            Some(pos_or_panic!(1.5)),
            Some(Positive::TWO),
            Some(pos_or_panic!(6.0)),
            Some(pos_or_panic!(6.5)),
            pos_or_panic!(0.22),
            Some(dec!(0.32)),
            Some(dec!(-0.68)),
            Some(dec!(0.020)),
            None,
            None,
            None,
        );

        chain.add_option(
            pos_or_panic!(110.0),
            Some(pos_or_panic!(0.5)),
            Some(Positive::ONE),
            Some(pos_or_panic!(10.0)),
            Some(pos_or_panic!(10.5)),
            pos_or_panic!(0.26),
            Some(dec!(0.18)),
            Some(dec!(-0.82)),
            Some(dec!(0.015)),
            None,
            None,
            None,
        );

        chain
    }

    #[test]
    fn test_volatility_skew_returns_curve_with_correct_points() {
        let chain = create_chain_with_options();
        let skew = chain.volatility_skew().unwrap();

        assert_eq!(skew.points.len(), 5);
    }

    #[test]
    fn test_volatility_skew_moneyness_as_x_axis() {
        let chain = create_chain_with_options();
        let skew = chain.volatility_skew().unwrap();

        let points: Vec<_> = skew.points.iter().collect();

        // Moneyness = (strike/underlying - 1) * 100
        // For strike 90, underlying 100: (90/100 - 1) * 100 = -10
        assert_eq!(points[0].x, dec!(-10.0));
        // For strike 100, underlying 100: (100/100 - 1) * 100 = 0
        assert_eq!(points[2].x, dec!(0.0));
        // For strike 110, underlying 100: (110/100 - 1) * 100 = 10
        assert_eq!(points[4].x, dec!(10.0));
    }

    #[test]
    fn test_volatility_skew_iv_as_y_axis() {
        let chain = create_chain_with_options();
        let skew = chain.volatility_skew().unwrap();

        let points: Vec<_> = skew.points.iter().collect();

        assert_eq!(points[0].y, dec!(0.28)); // OTM put
        assert_eq!(points[2].y, dec!(0.20)); // ATM
        assert_eq!(points[4].y, dec!(0.26)); // OTM call
    }

    #[test]
    fn test_volatility_skew_empty_chain() {
        let chain = OptionChain::new(
            "TEST",
            Positive::HUNDRED,
            "2030-01-01".to_string(),
            None,
            None,
        );
        let skew = chain.volatility_skew();

        assert!(skew.is_err());
    }
}

/// Inputs that used to abort the chain metrics in a raw `Decimal` or
/// `Positive` operator, each pinned to the error it now returns (#788).
#[cfg(test)]
mod tests_panic_freedom {
    use super::*;
    use optionstratlib_core::pos_or_panic;

    /// Strike, bid, ask (both styles) and volume of one chain row.
    type Row = (
        Positive,
        Option<Positive>,
        Option<Positive>,
        Option<Positive>,
    );

    fn chain_with(spot: Positive, rows: &[Row]) -> OptionChain {
        let mut chain = OptionChain::new("TEST", spot, "2030-01-01".to_string(), None, None);
        for (strike, bid, ask, volume) in rows {
            chain.add_option(
                *strike,
                *bid,
                *ask,
                *bid,
                *ask,
                pos_or_panic!(0.2),
                None,
                None,
                None,
                *volume,
                Some(10),
                None,
            );
        }
        chain
    }

    fn quoted_chain() -> OptionChain {
        chain_with(
            Positive::HUNDRED,
            &[
                (
                    pos_or_panic!(90.0),
                    Some(pos_or_panic!(11.0)),
                    Some(pos_or_panic!(12.0)),
                    Some(Positive::HUNDRED),
                ),
                (
                    Positive::HUNDRED,
                    Some(pos_or_panic!(4.0)),
                    Some(pos_or_panic!(5.0)),
                    Some(Positive::HUNDRED),
                ),
                (
                    pos_or_panic!(110.0),
                    Some(Positive::ONE),
                    Some(Positive::TWO),
                    Some(Positive::HUNDRED),
                ),
            ],
        )
    }

    fn reversed() -> (Positive, Positive) {
        (pos_or_panic!(200.0), Positive::HUNDRED)
    }

    fn is_invalid_range<T>(result: Result<T, SurfaceError>) -> bool {
        matches!(
            result,
            Err(SurfaceError::OperationError(
                optionstratlib_core::error::OperationErrorKind::InvalidParameters { .. }
            ))
        )
    }

    #[test]
    fn test_volatility_skew_zero_spot_returns_error() {
        let chain = chain_with(
            Positive::ZERO,
            &[(
                Positive::HUNDRED,
                Some(Positive::ONE),
                Some(Positive::TWO),
                None,
            )],
        );
        assert!(matches!(
            chain.volatility_skew(),
            Err(CurveError::MetricsError(_))
        ));
    }

    #[test]
    fn test_premium_concentration_zero_premia_returns_error() {
        let chain = chain_with(
            Positive::HUNDRED,
            &[(
                Positive::HUNDRED,
                Some(Positive::ZERO),
                Some(Positive::ZERO),
                None,
            )],
        );
        assert!(matches!(
            chain.premium_concentration(),
            Err(CurveError::MetricsError(_))
        ));
    }

    #[test]
    fn test_grid_surfaces_reversed_range_return_error() {
        let chain = quoted_chain();
        let iv = pos_or_panic!(0.2);
        let days = vec![pos_or_panic!(30.0)];
        assert!(is_invalid_range(chain.vanna_volga_surface(
            reversed(),
            (iv, iv),
            2,
            2
        )));
        assert!(is_invalid_range(chain.vanna_volga_surface(
            (Positive::HUNDRED, Positive::HUNDRED),
            (pos_or_panic!(0.5), iv),
            2,
            2
        )));
        assert!(is_invalid_range(chain.delta_gamma_surface(
            reversed(),
            days.clone(),
            2
        )));
        assert!(is_invalid_range(chain.volatility_sensitivity_surface(
            reversed(),
            (iv, iv),
            2,
            2
        )));
        assert!(is_invalid_range(chain.price_shock_surface(
            reversed(),
            (iv, iv),
            2,
            2
        )));
        assert!(is_invalid_range(chain.time_decay_surface(
            reversed(),
            days.clone(),
            2
        )));
        assert!(is_invalid_range(chain.theta_surface(
            reversed(),
            days.clone(),
            2
        )));
        assert!(is_invalid_range(chain.charm_surface(
            reversed(),
            days.clone(),
            2
        )));
        assert!(is_invalid_range(chain.color_surface(reversed(), days, 2)));
    }

    #[test]
    fn test_grid_surfaces_reversed_range_without_steps_is_a_single_point() {
        // Zero steps never formed the width, so a reversed range was, and
        // stays, the single point at its lower bound.
        let chain = quoted_chain();
        let iv = pos_or_panic!(0.2);
        let surface = chain
            .vanna_volga_surface(reversed(), (iv, iv), 0, 0)
            .expect("zero steps need no width");
        assert_eq!(surface.points.len(), 1);
    }

    #[test]
    fn test_vanna_volga_surface_overflow_and_zero_spot_return_error() {
        let iv = pos_or_panic!(0.2);
        assert!(matches!(
            quoted_chain().vanna_volga_surface(
                (Positive::ZERO, Positive::MAX),
                (Positive::ZERO, Positive::MAX),
                2,
                2
            ),
            Err(SurfaceError::AnalysisError(_))
        ));
        let zero_spot = chain_with(
            Positive::ZERO,
            &[(
                Positive::HUNDRED,
                Some(Positive::ONE),
                Some(Positive::TWO),
                None,
            )],
        );
        assert!(matches!(
            zero_spot.vanna_volga_surface((Positive::HUNDRED, Positive::HUNDRED), (iv, iv), 1, 1),
            Err(SurfaceError::AnalysisError(_))
        ));
    }

    #[test]
    fn test_bid_ask_spread_curve_crossed_quote_returns_error() {
        let chain = chain_with(
            Positive::HUNDRED,
            &[(
                Positive::HUNDRED,
                Some(pos_or_panic!(5.0)),
                Some(pos_or_panic!(4.0)),
                None,
            )],
        );
        assert!(matches!(
            chain.bid_ask_spread_curve(),
            Err(CurveError::OperationError(_))
        ));
    }

    #[test]
    fn test_bid_ask_spread_curve_overflowing_mid_returns_error() {
        let chain = chain_with(
            Positive::HUNDRED,
            &[(
                Positive::HUNDRED,
                Some(Positive::MAX),
                Some(Positive::MAX),
                None,
            )],
        );
        assert!(matches!(
            chain.bid_ask_spread_curve(),
            Err(CurveError::MetricsError(_))
        ));
    }

    #[test]
    fn test_bid_ask_spread_curve_matches_reference() {
        // (ask - bid) / mid: 1 / 11.5, 1 / 4.5 and 1 / 1.5.
        let curve = quoted_chain()
            .bid_ask_spread_curve()
            .expect("well-formed quotes");
        let spreads: Vec<Decimal> = curve.points.iter().map(|p| p.y).collect();
        assert_eq!(
            spreads,
            vec![
                Decimal::ONE / dec!(11.5),
                Decimal::ONE / dec!(4.5),
                Decimal::ONE / dec!(1.5)
            ]
        );
    }

    #[test]
    fn test_dollar_gamma_and_delta_gamma_huge_spot_return_error() {
        let huge = pos_or_panic!(1e20);
        let chain = chain_with(
            huge,
            &[(huge, Some(Positive::ONE), Some(Positive::TWO), None)],
        );
        assert!(matches!(
            chain.dollar_gamma_curve(&OptionStyle::Call),
            Err(CurveError::MetricsError(_))
        ));
        assert!(matches!(
            chain.delta_gamma_curve(),
            Err(CurveError::MetricsError(_))
        ));
        assert!(matches!(
            chain.price_shock_curve(dec!(1e10)),
            Err(CurveError::MetricsError(_))
        ));
    }

    #[test]
    fn test_price_shock_curve_overflowing_shock_returns_error() {
        assert!(matches!(
            quoted_chain().price_shock_curve(Decimal::MAX),
            Err(CurveError::MetricsError(_))
        ));
    }

    #[test]
    fn test_volume_and_iv_surfaces_overflow_return_error() {
        let chain = chain_with(
            Positive::HUNDRED,
            &[(
                Positive::HUNDRED,
                Some(Positive::ONE),
                Some(Positive::TWO),
                Some(Positive::MAX),
            )],
        );
        let tiny_day = Positive::new_decimal(Decimal::new(1, 28)).expect("positive literal");
        assert!(matches!(
            chain.volume_profile_surface(vec![tiny_day]),
            Err(SurfaceError::AnalysisError(_))
        ));
        let mut max_iv = OptionChain::new(
            "TEST",
            Positive::HUNDRED,
            "2030-01-01".to_string(),
            None,
            None,
        );
        max_iv.add_option(
            Positive::HUNDRED,
            None,
            None,
            None,
            None,
            Positive::MAX,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert!(matches!(
            max_iv.iv_surface(vec![Positive::MAX]),
            Err(SurfaceError::AnalysisError(_))
        ));
    }
}
