//! Metric trait implementations for [`OptionChain`].
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

use crate::chains::OptionChain;
use crate::curves::{Curve, Point2D};
use crate::error::{CurveError, SurfaceError};
use crate::geometrics::LinearInterpolation;
use crate::greeks::Greeks;
use crate::metrics::{
    BidAskSpreadCurve, CharmCurve, CharmSurface, ColorCurve, ColorSurface, DeltaGammaProfileCurve,
    DeltaGammaProfileSurface, DollarGammaCurve, ImpliedVolatilityCurve, ImpliedVolatilitySurface,
    OpenInterestCurve, PriceShockCurve, PriceShockSurface, PutCallRatioCurve, RiskReversalCurve,
    SmileDynamicsCurve, SmileDynamicsSurface, StrikeConcentrationCurve, ThetaCurve, ThetaSurface,
    TimeDecayCurve, TimeDecaySurface, VannaVolgaSurface, VolatilitySensitivityCurve,
    VolatilitySensitivitySurface, VolatilitySkewCurve, VolumeProfileCurve, VolumeProfileSurface,
};
use crate::model::{ExpirationDate, OptionStyle, Options, Side};
use crate::surfaces::{Point3D, Surface};
use positive::Positive;
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use std::cmp::Ordering;
use std::collections::BTreeSet;

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
        let mut bt_points = self
            .options
            .iter()
            .map(|option| {
                Point2D::new(
                    (option.strike_price.to_dec() / self.underlying_price.to_dec() - Decimal::ONE)
                        * Decimal::ONE_HUNDRED,
                    option.implied_volatility.to_dec(),
                )
            })
            .collect::<BTreeSet<_>>();

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
    /// use optionstratlib::chains::chain::OptionChain;
    /// use optionstratlib::metrics::ImpliedVolatilityCurve;
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
    /// use optionstratlib::chains::chain::OptionChain;
    /// use optionstratlib::metrics::PutCallRatioCurve;
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
                points.insert(Point2D::new(
                    opt.strike_price.to_dec(),
                    put_mid.to_dec() / call_mid_dec,
                ));
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
    /// use optionstratlib::chains::chain::OptionChain;
    /// use optionstratlib::metrics::StrikeConcentrationCurve;
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
                _strike_premium = put_mid.to_dec() + call_mid.to_dec();
                points.insert(Point2D::new(opt.strike_price.to_dec(), _strike_premium));
                total_chain_premium += _strike_premium;
                strike_count += Decimal::ONE;
            }
        }
        // If there are no points calculated return an error
        if points.is_empty() {
            return Err(CurveError::ConstructionError(
                "No options with valid premium weighted Strike Concentration data".to_string(),
            ));
        }
        // Calculate the average chain premium
        let average_premium = total_chain_premium / strike_count;
        // Normalize Strike Concentration data points based on average premium
        let mut points_normalized = BTreeSet::new();
        for point in points.iter() {
            points_normalized.insert(Point2D::new(point.x, point.y / average_premium));
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
    /// use optionstratlib::chains::chain::OptionChain;
    /// use optionstratlib::metrics::ImpliedVolatilitySurface;
    /// use positive::pos_or_panic;
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
                let time_factor = (days.to_dec() / dec!(365.0)).sqrt().unwrap_or(Decimal::ONE);
                let adjusted_iv = opt.implied_volatility.to_dec() * time_factor;

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
    /// use optionstratlib::chains::chain::OptionChain;
    /// use optionstratlib::metrics::RiskReversalCurve;
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
    /// use optionstratlib::chains::chain::OptionChain;
    /// use optionstratlib::metrics::DollarGammaCurve;
    /// use optionstratlib::model::OptionStyle;
    ///
    /// let chain = OptionChain::new("SPY", pos_or_panic!(450.0), "2024-03-15".to_string(), None, None);
    /// let dg_curve = chain.dollar_gamma_curve(&OptionStyle::Call)?;
    /// ```
    fn dollar_gamma_curve(&self, option_style: &OptionStyle) -> Result<Curve, CurveError> {
        let spot = self.underlying_price;
        let spot_squared = spot.to_dec() * spot.to_dec();

        let points: BTreeSet<Point2D> = self
            .get_single_iter()
            .filter_map(|opt| {
                let option = match option_style {
                    OptionStyle::Call => opt.get_option(Side::Long, OptionStyle::Call).ok()?,
                    OptionStyle::Put => opt.get_option(Side::Long, OptionStyle::Put).ok()?,
                };

                let gamma = option.gamma().ok()?;
                // Dollar Gamma = Gamma × Spot² × 0.01
                let dollar_gamma = gamma * spot_squared * dec!(0.01);

                Some(Point2D::new(opt.strike_price.to_dec(), dollar_gamma))
            })
            .collect();

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

        let price_step = if price_steps > 0 {
            (price_range.1 - price_range.0).to_dec() / Decimal::from(price_steps)
        } else {
            Decimal::ZERO
        };

        let vol_step = if vol_steps > 0 {
            (vol_range.1 - vol_range.0).to_dec() / Decimal::from(vol_steps)
        } else {
            Decimal::ZERO
        };

        for p in 0..=price_steps {
            let price = price_range.0.to_dec() + price_step * Decimal::from(p);

            for v in 0..=vol_steps {
                let vol = vol_range.0.to_dec() + vol_step * Decimal::from(v);

                // Vanna-Volga cost model:
                // Cost increases with distance from ATM and with volatility difference
                let moneyness =
                    (price - self.underlying_price.to_dec()).abs() / self.underlying_price.to_dec();
                let vol_diff = (vol - atm_vol).abs();

                // Simplified Vanna-Volga cost: combines moneyness and vol effects
                // Vanna component: moneyness × vol_diff
                // Volga component: vol_diff²
                let vanna_cost = moneyness * vol_diff * dec!(100.0);
                let volga_cost = vol_diff * vol_diff * dec!(50.0);
                let vv_cost = vanna_cost + volga_cost;

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
        let spot = self.underlying_price.to_dec();
        let spot_squared = spot * spot;

        let points: BTreeSet<Point2D> = self
            .get_single_iter()
            .filter_map(|opt| {
                // Use call options for the profile
                let option = opt.get_option(Side::Long, OptionStyle::Call).ok()?;

                let delta = option.delta().ok()?;
                let gamma = option.gamma().ok()?;

                // Dollar Delta = Delta × Spot
                let dollar_delta = delta * spot;
                // Dollar Gamma = Gamma × Spot² × 0.01
                let dollar_gamma = gamma * spot_squared * dec!(0.01);

                // Combined metric
                let combined = dollar_delta + dollar_gamma;

                Some(Point2D::new(opt.strike_price.to_dec(), combined))
            })
            .collect();

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

        let price_step = if price_steps > 0 {
            (price_range.1 - price_range.0).to_dec() / Decimal::from(price_steps)
        } else {
            Decimal::ZERO
        };

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
                let price = price_range.0.to_dec() + price_step * Decimal::from(p);
                let price_pos = Positive::new_decimal(price).unwrap_or(Positive::ONE);

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
                );

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
                let time_factor = (days.to_dec() / dec!(30.0)).sqrt().unwrap_or(Decimal::ONE);
                let adjusted_skew = if time_factor > Decimal::ZERO {
                    skew / time_factor
                } else {
                    skew
                };

                let adjusted_iv = atm_vol + adjusted_skew;
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
            // Calculate call spread if available
            if let (Some(bid), Some(ask)) = (opt.call_bid, opt.call_ask) {
                let mid = (bid + ask) / Positive::TWO;
                if mid > Positive::ZERO {
                    let spread = (ask - bid).to_dec() / mid.to_dec();
                    points.insert(Point2D::new(opt.strike_price.to_dec(), spread));
                }
            }
            // If no call data, try put spread
            else if let (Some(bid), Some(ask)) = (opt.put_bid, opt.put_ask) {
                let mid = (bid + ask) / Positive::TWO;
                if mid > Positive::ZERO {
                    let spread = (ask - bid).to_dec() / mid.to_dec();
                    points.insert(Point2D::new(opt.strike_price.to_dec(), spread));
                }
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
                    let time_factor = if day.to_dec() > Decimal::ZERO {
                        (dec!(30.0) / day.to_dec()).sqrt().unwrap_or(Decimal::ONE)
                    } else {
                        Decimal::ONE
                    };

                    let adjusted_vol = base_vol.to_dec() * time_factor;
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

        let price_step = if price_steps > 0 {
            (price_range.1 - price_range.0).to_dec() / Decimal::from(price_steps)
        } else {
            Decimal::ZERO
        };

        let vol_step = if vol_steps > 0 {
            (vol_range.1 - vol_range.0).to_dec() / Decimal::from(vol_steps)
        } else {
            Decimal::ZERO
        };

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

        // `dec!(0.01)` is a compile-time positive literal; the checked
        // constructor is total, so the `Positive::ZERO` branch is
        // unreachable. Hoisted out of the nested loop so the fallback
        // isn't rebuilt for every (price, vol) pair.
        let vol_fallback = Positive::new_decimal(dec!(0.01)).unwrap_or(Positive::ZERO);
        for p in 0..=price_steps {
            let price = price_range.0.to_dec() + price_step * Decimal::from(p);
            let price_pos = Positive::new_decimal(price).unwrap_or(Positive::ONE);

            for v in 0..=vol_steps {
                let vol = vol_range.0.to_dec() + vol_step * Decimal::from(v);
                let vol_pos = Positive::new_decimal(vol).unwrap_or(vol_fallback);

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
                );

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

        let price_step = if price_steps > 0 {
            (price_range.1 - price_range.0).to_dec() / Decimal::from(price_steps)
        } else {
            Decimal::ZERO
        };

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
                let price = price_range.0.to_dec() + price_step * Decimal::from(p);
                let price_pos = Positive::new_decimal(price).unwrap_or(Positive::ONE);

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
                );

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
        let spot = self.underlying_price.to_dec();
        let price_move = spot * shock_pct;

        let points: BTreeSet<Point2D> = self
            .get_single_iter()
            .filter_map(|opt| {
                let option = opt.get_option(Side::Long, OptionStyle::Call).ok()?;
                let delta = option.delta().ok()?;
                let gamma = option.gamma().ok()?;

                // P&L = Delta × ΔS + 0.5 × Gamma × ΔS²
                let pnl = delta * price_move + dec!(0.5) * gamma * price_move * price_move;

                Some(Point2D::new(opt.strike_price.to_dec(), pnl))
            })
            .collect();

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

        let price_step = if price_steps > 0 {
            (price_range.1 - price_range.0).to_dec() / Decimal::from(price_steps)
        } else {
            Decimal::ZERO
        };

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
                let price = price_range.0.to_dec() + price_step * Decimal::from(p);
                let price_pos = Positive::new_decimal(price).unwrap_or(Positive::ONE);

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
                );

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

        let price_step = if price_steps > 0 {
            (price_range.1 - price_range.0).to_dec() / Decimal::from(price_steps)
        } else {
            Decimal::ZERO
        };

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
                let price = price_range.0.to_dec() + price_step * Decimal::from(p);
                let price_pos = Positive::new_decimal(price).unwrap_or(Positive::ONE);

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
                );

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

        let price_step = if price_steps > 0 {
            (price_range.1 - price_range.0).to_dec() / Decimal::from(price_steps)
        } else {
            Decimal::ZERO
        };

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
                let price = price_range.0.to_dec() + price_step * Decimal::from(p);
                let price_pos = Positive::new_decimal(price).unwrap_or(Positive::ONE);

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
                );

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
    use crate::assert_decimal_eq;

    #[test]
    fn test_put_call_ratio_premium_weighted_trait() {
        let result = OptionChain::load_from_json("examples/Chains/SP500-18-oct-2024-5781.88.json");
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
        let result = OptionChain::load_from_json("examples/Chains/SP500-18-oct-2024-5781.88.json");
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
    use positive::pos_or_panic;

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
