//! Heston Ladder Pricing Module
//!
//! Ladders of everlasting options when the variance of the underlying is
//! stochastic. A ladder is the funding network in which rung `j` is funded
//! against rung `j − 1`, and rung 1 against the payoff (see
//! [`crate::pricing::funding_network`]); the price of rung `j` is the
//! European value at a random maturity, Erlang(`j`, `κ`) when funding accrues
//! continuously and `Δ` times a negative binomial count when it is paid every
//! `Δ` years.
//!
//! # Model
//!
//! The model of Heston:
//!
//! ```text
//! dS/S = μ·dt + √v·dW₁
//! dv   = λ·(θ − v)·dt + ξ·√v·dW₂        d⟨W₁, W₂⟩ = ρ·dt
//! ```
//!
//! with `v₀` the variance now, `θ` its long-run level, `λ` the speed of mean
//! reversion, `ξ` the volatility of the variance and `ρ` the correlation.
//! [`HestonModel`] holds the five numbers. The drift `μ` is chosen by the
//! caller as in a funding network: `E[S_s] = S·exp(μ·s)`, and values are
//! undiscounted. The condition of Feller is not required.
//!
//! The model is not a Lévy model: the transform of the log return over `s`
//! years is `exp(μ·z·s + C(z, s) + D(z, s)·v₀)`, not the exponential of `s`
//! times an exponent, so the maturity cannot be integrated out in closed
//! form as [`crate::pricing::funding_network`] does.
//!
//! # Method
//!
//! Each value is that of the Black-Scholes ladder of variance `v₀`, which
//! the funding network pricer gives, plus a correction:
//!
//! ```text
//! F_j = F_j^BS(v₀) + (K·e^{x/2}/π)·∫₀^∞ Re[ e^{ivx}·ΔΞ_j(v) / (−(v² + ¼)) ] dv,   x = ln(S/K)
//! ΔΞ_j(v) = ∫ w_j(s)·(φ_H − φ_BS)(v − i/2, s) ds
//! ```
//!
//! `w_j` is the law of the maturity of rung `j` and `φ_H`, `φ_BS` the two
//! transforms. The two models agree to first order in `s`, so the difference
//! is small where short maturities matter, at high frequencies, and the
//! correction converges much faster than either price. It has no pole at the
//! payoff, since both models give the same forward, and is inverted along
//! `Im u = −1/2`, where every moment involved exists whatever the
//! parameters. The same correction serves the call and the put.
//!
//! The integral over the maturity uses a fixed set of nodes. With continuous
//! funding they are 16 Gauss-Legendre points on each interval `[a, 2a]` from
//! `2⁻²⁰/κ` up to where the Erlang weights are negligible. With discrete
//! funding the first 32 payments are summed one by one, and on each later
//! block of payments `[m, 2m)` the difference is interpolated at 16
//! Chebyshev points and summed against the exact weights. The transform of
//! Heston is evaluated in the form that keeps the principal logarithm
//! continuous, and without the cancellations of its textbook form for a
//! small volatility of the variance.
//!
//! The delta uses the same correction without one factor of the payoff. The
//! sensitivity to `v₀` has no Black-Scholes value to start from: its
//! integrand is the derivative of the transform of the Black-Scholes ladder,
//! in closed form, plus the derivative of the correction.
//!
//! Against mixtures of European Heston values integrated one by one, the
//! prices agree to `1e-10·K` in the cases tested. A call costs far more than
//! one of a Lévy model, which takes a fraction of a millisecond: a few
//! milliseconds with discrete funding, tens with continuous funding, whose
//! nodes reach much shorter maturities, and up to several hundred for a
//! delta far from the money.
//!
//! # Example
//!
//! ```rust
//! use optionstratlib_core::model::Positive;
//! use optionstratlib_core::model::types::OptionStyle;
//! use optionstratlib_pricing::pricing::{
//!     FundingSchedule, HestonLadderParams, HestonModel, heston_ladder_prices,
//! };
//! use rust_decimal_macros::dec;
//! use std::num::NonZeroU32;
//!
//! let model = HestonModel::try_new(
//!     Positive::new_decimal(dec!(0.25))?,
//!     Positive::new_decimal(dec!(0.20))?,
//!     Positive::new_decimal(dec!(3))?,
//!     Positive::new_decimal(dec!(0.9))?,
//!     dec!(-0.6),
//! )?;
//! // Three rungs at a funding rate of 36.5 a year: the top one has a mean
//! // maturity of 30 days.
//! let params = HestonLadderParams::try_new(
//!     Positive::HUNDRED,
//!     Positive::HUNDRED,
//!     dec!(0),
//!     model,
//!     Positive::new_decimal(dec!(36.5))?,
//!     NonZeroU32::MIN.saturating_add(2),
//!     OptionStyle::Call,
//! )?;
//! let prices = heston_ladder_prices(&params, FundingSchedule::Continuous)?;
//!
//! // A longer maturity is worth more at the money.
//! assert_eq!(prices.len(), 3);
//! assert!(matches!((prices.first(), prices.get(2)), (Some(low), Some(top)) if low < top));
//! # Ok::<(), optionstratlib_pricing::error::PricingError>(())
//! ```

use super::funding_network::{
    Complex, FUNDING_NETWORK_MAX_ORDER, FundingNetwork, FundingNetworkParams, FundingSchedule,
    Integrand, LevyModel, Quantity, Rule, Sample, Values, exp_f64, integrate, ln_f64,
    network_values, reject_zero, sqrt_f64, to_decimal,
};
use crate::error::PricingError;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::decimal_to_f64;
use optionstratlib_core::model::types::OptionStyle;
use rust_decimal::Decimal;
use std::num::NonZeroU32;

/// Name of the method in [`PricingError::MethodError`].
const METHOD: &str = "heston_ladder";

/// Order of the exponential moment along which the correction is inverted.
const CONTOUR: f64 = 0.5;

/// Shortest maturity of the nodes of a continuous schedule, in units of
/// `1/κ`: `2⁻²⁰`. Below it the two models differ by less than the tolerance
/// of the inversion.
const SHORTEST: f64 = 9.536_743_164_062_5e-7;

/// Modulus under which a transform is taken as zero.
const NEGLIGIBLE: f64 = 1e-18;

/// Payments of a discrete schedule summed one by one before interpolation
/// takes over.
const DIRECT_PAYMENTS: f64 = 32.0;

/// Chebyshev points on each block of payments.
const POINTS: usize = 16;

/// The same as an `f64`.
const POINTS_F64: f64 = 16.0;

/// Payments after which a discrete schedule is refused: the weights of a
/// ladder this slow against its payment interval are those of continuous
/// funding to every digit the pricer returns.
const MAX_PAYMENTS: f64 = 4e6;

/// The parameters of the model of Heston.
///
/// Built with [`HestonModel::try_new`], which checks the domain of every
/// field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HestonModel {
    initial_variance: Positive,
    long_run_variance: Positive,
    mean_reversion: Positive,
    volatility_of_variance: Positive,
    correlation: Decimal,
}

impl HestonModel {
    /// Validates and builds the model.
    ///
    /// # Arguments
    ///
    /// * `initial_variance` - the variance now, `v₀`, per year.
    /// * `long_run_variance` - the level `θ` the variance reverts to.
    /// * `mean_reversion` - the speed `λ` of the reversion, per year.
    /// * `volatility_of_variance` - `ξ` in `dv = λ(θ − v)dt + ξ√v·dW`.
    /// * `correlation` - the correlation `ρ` of the two Brownian motions.
    ///
    /// # Errors
    ///
    /// [`PricingError::InvalidParameter`] when a variance, the speed or the
    /// volatility of the variance is zero, or the correlation is not
    /// strictly between −1 and 1.
    pub fn try_new(
        initial_variance: Positive,
        long_run_variance: Positive,
        mean_reversion: Positive,
        volatility_of_variance: Positive,
        correlation: Decimal,
    ) -> Result<Self, PricingError> {
        reject_zero("initial_variance", initial_variance)?;
        reject_zero("long_run_variance", long_run_variance)?;
        reject_zero("mean_reversion", mean_reversion)?;
        reject_zero("volatility_of_variance", volatility_of_variance)?;
        if correlation <= Decimal::NEGATIVE_ONE || correlation >= Decimal::ONE {
            return Err(PricingError::invalid_parameter(
                "correlation",
                correlation,
                "must be strictly between -1 and 1",
            ));
        }
        Ok(Self {
            initial_variance,
            long_run_variance,
            mean_reversion,
            volatility_of_variance,
            correlation,
        })
    }

    /// The variance now, per year.
    #[must_use]
    pub const fn initial_variance(&self) -> Positive {
        self.initial_variance
    }

    /// The level the variance reverts to.
    #[must_use]
    pub const fn long_run_variance(&self) -> Positive {
        self.long_run_variance
    }

    /// The speed of the reversion, per year.
    #[must_use]
    pub const fn mean_reversion(&self) -> Positive {
        self.mean_reversion
    }

    /// The volatility of the variance.
    #[must_use]
    pub const fn volatility_of_variance(&self) -> Positive {
        self.volatility_of_variance
    }

    /// The correlation of the price and its variance.
    #[must_use]
    pub const fn correlation(&self) -> Decimal {
        self.correlation
    }
}

/// Inputs of a call or put ladder under the model of Heston: the market,
/// the model, the ladder and the payoff shared by every rung.
///
/// Built with [`HestonLadderParams::try_new`], which checks the domain of
/// every field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HestonLadderParams {
    underlying_price: Positive,
    strike_price: Positive,
    drift: Decimal,
    model: HestonModel,
    funding_rate: Positive,
    rungs: NonZeroU32,
    option_style: OptionStyle,
}

impl HestonLadderParams {
    /// Validates and builds the inputs.
    ///
    /// # Arguments
    ///
    /// * `underlying_price` - current price `S` of the underlying.
    /// * `strike_price` - strike `K` of the payoff.
    /// * `drift` - drift `μ` of the underlying per year: `E[S_s] = S·exp(μ·s)`.
    /// * `model` - the parameters of the variance.
    /// * `funding_rate` - funding rate `κ` of every rung, per year.
    /// * `rungs` - number of rungs; rung `j` has a mean maturity of `j/κ`.
    /// * `option_style` - call or put.
    ///
    /// # Errors
    ///
    /// [`PricingError::InvalidParameter`] when a price or the funding rate is
    /// zero, or `rungs` exceeds [`FUNDING_NETWORK_MAX_ORDER`].
    pub fn try_new(
        underlying_price: Positive,
        strike_price: Positive,
        drift: Decimal,
        model: HestonModel,
        funding_rate: Positive,
        rungs: NonZeroU32,
        option_style: OptionStyle,
    ) -> Result<Self, PricingError> {
        reject_zero("underlying_price", underlying_price)?;
        reject_zero("strike_price", strike_price)?;
        // Checks the funding rate and the number of rungs.
        FundingNetwork::ladder(funding_rate, rungs)?;
        Ok(Self {
            underlying_price,
            strike_price,
            drift,
            model,
            funding_rate,
            rungs,
            option_style,
        })
    }

    /// Current price of the underlying.
    #[must_use]
    pub const fn underlying_price(&self) -> Positive {
        self.underlying_price
    }

    /// Strike of the payoff.
    #[must_use]
    pub const fn strike_price(&self) -> Positive {
        self.strike_price
    }

    /// Drift of the underlying, per year.
    #[must_use]
    pub const fn drift(&self) -> Decimal {
        self.drift
    }

    /// Parameters of the variance.
    #[must_use]
    pub const fn model(&self) -> HestonModel {
        self.model
    }

    /// Funding rate of every rung, per year.
    #[must_use]
    pub const fn funding_rate(&self) -> Positive {
        self.funding_rate
    }

    /// Number of rungs.
    #[must_use]
    pub const fn rungs(&self) -> NonZeroU32 {
        self.rungs
    }

    /// Call or put.
    #[must_use]
    pub const fn option_style(&self) -> OptionStyle {
        self.option_style
    }
}

/// Prices of every rung of a ladder under the model of Heston, per unit of
/// underlying: entry `j − 1` is rung `j`.
///
/// With [`FundingSchedule::Discrete`] the value is the price just after a
/// payment, as in
/// [`funding_network_prices_with`](crate::pricing::funding_network_prices_with).
///
/// # Errors
///
/// - [`PricingError::InvalidParameter`] when the drift is not below the
///   decay rate of the ladder under the schedule, or the payment interval is
///   zero or beyond the stability bound of the ladder.
/// - [`PricingError::MethodError`] when a Fourier inversion does not
///   converge, or the payment interval is so short against the maturity of
///   the ladder that the discrete sum is refused.
/// - [`PricingError::Decimal`] when an input has no `f64` form.
/// - [`PricingError::NonFinite`] when a result has no `Decimal` form.
pub fn heston_ladder_prices(
    params: &HestonLadderParams,
    schedule: FundingSchedule,
) -> Result<Vec<Decimal>, PricingError> {
    let values = ladder_values(params, schedule, Target::Price)?;
    collect("pricing::heston_ladder::price", &values, params.rungs)
}

/// Deltas of every rung of a ladder under the model of Heston: the
/// derivative of each price of [`heston_ladder_prices`] with respect to the
/// underlying price, the variance held fixed.
///
/// # Errors
///
/// As [`heston_ladder_prices`].
pub fn heston_ladder_deltas(
    params: &HestonLadderParams,
    schedule: FundingSchedule,
) -> Result<Vec<Decimal>, PricingError> {
    let values = ladder_values(params, schedule, Target::Delta)?;
    collect("pricing::heston_ladder::delta", &values, params.rungs)
}

/// Sensitivities of every rung of a ladder to the variance: the derivative
/// of each price of [`heston_ladder_prices`] with respect to the initial
/// variance `v₀`, the underlying price held fixed. The same for the call and
/// the put.
///
/// # Errors
///
/// As [`heston_ladder_prices`].
pub fn heston_ladder_variance_sensitivities(
    params: &HestonLadderParams,
    schedule: FundingSchedule,
) -> Result<Vec<Decimal>, PricingError> {
    let values = ladder_values(params, schedule, Target::Variance)?;
    collect(
        "pricing::heston_ladder::variance_sensitivity",
        &values,
        params.rungs,
    )
}

/// The first values, one per rung, as `Decimal`s.
fn collect(
    context: &'static str,
    values: &Values,
    rungs: NonZeroU32,
) -> Result<Vec<Decimal>, PricingError> {
    values
        .iter()
        .take(order(rungs))
        .map(|value| to_decimal(context, *value))
        .collect()
}

/// The number of rungs as an index bound. [`HestonLadderParams::try_new`]
/// has checked that it fits.
fn order(rungs: NonZeroU32) -> usize {
    usize::try_from(rungs.get()).map_or(FUNDING_NETWORK_MAX_ORDER, |order| {
        order.min(FUNDING_NETWORK_MAX_ORDER)
    })
}

/// What the pricer returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Target {
    Price,
    Delta,
    Variance,
}

/// A maturity at which the transforms are evaluated, with the weight it
/// carries for every rung: the law of the maturity times the weight of the
/// quadrature.
#[derive(Debug, Clone, Copy)]
struct Node {
    time: f64,
    weights: Values,
}

/// Sorts the nodes by maturity: the evaluation stops at the first maturities
/// at which the transforms are negligible.
fn sorted(mut nodes: Vec<Node>) -> Vec<Node> {
    nodes.sort_by(|a, b| a.time.total_cmp(&b.time));
    nodes
}

/// Nodes of the Erlang laws of a ladder of funding rate `rate`: 16
/// Gauss-Legendre points on each `[a, 2a]`.
fn continuous_nodes(rate: f64, rungs: NonZeroU32) -> Vec<Node> {
    let rule = Rule::new();
    let size = f64::from(rungs.get());
    // Twice the point beyond which the Erlang law of the top rung has a mass
    // of `e^{−45}`: the moments of order one half grow at half the drift,
    // which is below the funding rate.
    let longest = 2.0 * (size + 12.0 * sqrt_f64(size) + 45.0) / rate;
    let log_rate = ln_f64(rate);
    let mut nodes = Vec::new();
    let mut low = SHORTEST / rate;
    while low < longest {
        let half_width = 0.5 * low;
        let middle = 1.5 * low;
        for (node, weight) in rule.nodes.iter().zip(rule.weights.iter()) {
            let time = middle + half_width * node;
            let mut weights = [0.0; FUNDING_NETWORK_MAX_ORDER];
            // Density of rung `j`: `κ^j·s^{j−1}·e^{−κs}/(j−1)!`.
            let mut log_density = log_rate - rate * time;
            let step = ln_f64(rate * time);
            let mut rung = 1.0;
            for slot in weights.iter_mut().take(order(rungs)) {
                *slot = weight * half_width * exp_f64(log_density);
                log_density += step - ln_f64(rung);
                rung += 1.0;
            }
            nodes.push(Node { time, weights });
        }
        low *= 2.0;
    }
    sorted(nodes)
}

/// Moves the law of the number of payments one payment forward: rung `j`
/// matures at this payment with probability `p` if rung `j − 1` matured at
/// the one before, and otherwise waits. `source` is the mass of the payoff,
/// one before the first payment and zero after.
fn advance(mass: &mut Values, rungs: NonZeroU32, share: f64, source: f64) {
    let mut below = source;
    for slot in mass.iter_mut().take(order(rungs)) {
        let own = *slot;
        *slot = (1.0 - share) * own + share * below;
        below = own;
    }
}

/// Nodes of the negative binomial laws of a ladder that pays the share
/// `share = κ·Δ` of its premium every `interval` years.
///
/// The first payments are nodes themselves. On each later block of payments
/// the function summed is interpolated at Chebyshev points, so the weight of
/// a point is the sum over the payments of the block of the law times the
/// Lagrange polynomial of the point.
fn discrete_nodes(share: f64, interval: f64, rungs: NonZeroU32) -> Result<Vec<Node>, PricingError> {
    let size = f64::from(rungs.get());
    // Payments until the law of the top rung has a mass of `e^{−45}` left.
    if (size + 12.0 * sqrt_f64(size) + 45.0) / share > MAX_PAYMENTS {
        return Err(PricingError::method_error(
            METHOD,
            "the payment interval is too short against the maturity of the ladder: price it with continuous funding",
        ));
    }
    let mut nodes = Vec::new();
    let mut mass = [0.0; FUNDING_NETWORK_MAX_ORDER];
    let mut payment = 1.0;
    advance(&mut mass, rungs, share, 1.0);
    while payment <= DIRECT_PAYMENTS {
        nodes.push(Node {
            time: payment * interval,
            weights: mass,
        });
        payment += 1.0;
        advance(&mut mass, rungs, share, 0.0);
    }

    // Chebyshev points of the first kind on `[−1, 1]` and their barycentric
    // weights.
    let mut points = [0.0; POINTS];
    let mut signs = [0.0; POINTS];
    let mut index = 0.0;
    let mut sign = 1.0;
    for (point, weight) in points.iter_mut().zip(signs.iter_mut()) {
        let angle = std::f64::consts::PI * (2.0 * index + 1.0) / (2.0 * POINTS_F64);
        let (sine, cosine) = angle.sin_cos();
        *point = cosine;
        *weight = sign * sine;
        sign = -sign;
        index += 1.0;
    }

    // `payment` is the first payment of the block and `mass` its law.
    loop {
        let last = 2.0 * payment - 1.0;
        let middle = 0.5 * (payment + last);
        let half_width = 0.5 * (last - payment) + 0.5;
        let mut block = [[0.0; FUNDING_NETWORK_MAX_ORDER]; POINTS];
        while payment <= last {
            let mut basis = [0.0; POINTS];
            let mut total = 0.0;
            let mut exact = None;
            for (slot, (value, (point, weight))) in basis
                .iter_mut()
                .zip(points.iter().zip(signs.iter()))
                .enumerate()
            {
                let distance = payment - (middle + half_width * point);
                if distance == 0.0 {
                    exact = Some(slot);
                    break;
                }
                *value = weight / distance;
                total += *value;
            }
            for (slot, (weights, value)) in block.iter_mut().zip(basis.iter()).enumerate() {
                let lagrange = match exact {
                    Some(hit) if hit == slot => 1.0,
                    Some(_) => 0.0,
                    None => value / total,
                };
                for (target, law) in weights.iter_mut().zip(mass.iter()) {
                    *target += law * lagrange;
                }
            }
            payment += 1.0;
            advance(&mut mass, rungs, share, 0.0);
        }
        for (weights, point) in block.iter().zip(points.iter()) {
            nodes.push(Node {
                time: (middle + half_width * point) * interval,
                weights: *weights,
            });
        }
        // Past the mode of the top rung the laws decay: stop when what is
        // left of every one of them is negligible.
        let largest = mass.iter().fold(0.0_f64, |a, b| a.max(b.abs()));
        if payment * share > size && largest * payment < NEGLIGIBLE {
            break;
        }
        if payment > 4.0 * MAX_PAYMENTS {
            return Err(PricingError::method_error(
                METHOD,
                "the law of the maturity did not decay",
            ));
        }
    }
    Ok(sorted(nodes))
}

/// The integrand of the correction, at every frequency of the contour
/// `Im u = −1/2`.
struct Correction<'a> {
    nodes: &'a [Node],
    rungs: NonZeroU32,
    target: Target,
    log_moneyness: f64,
    drift: f64,
    /// `v₀`.
    variance: f64,
    /// `λ·θ`.
    pull: f64,
    /// `λ`.
    reversion: f64,
    /// `ξ`.
    spread: f64,
    /// `ρ`.
    correlation: f64,
    funding_rate: f64,
    /// Years between two payments; `None` for continuous funding.
    interval: Option<f64>,
}

impl Correction<'_> {
    /// The derivative with respect to the variance of the transform of the
    /// Black-Scholes ladder, per rung: `d/dv₀ R^j` with `R = κ/(κ − χ)` for
    /// continuous funding and `R = p·y/(1 − (1 − p)·y)`, `y = e^{Δχ}`, for
    /// payments every `Δ`. `half_square` is `dχ/dv₀`.
    fn black_scholes_slope(
        &self,
        exponent: Complex,
        half_square: f64,
    ) -> [Complex; FUNDING_NETWORK_MAX_ORDER] {
        let (ratio, slope) = match self.interval {
            None => {
                let gap = Complex::real(self.funding_rate).minus(exponent);
                (
                    Complex::real(self.funding_rate).over(gap),
                    Complex::real(half_square).over(gap),
                )
            }
            Some(interval) => {
                let share = self.funding_rate * interval;
                let growth = exponent.scale(interval).exponential();
                let gap = Complex::real(1.0).minus(growth.scale(1.0 - share));
                (
                    growth.scale(share).over(gap),
                    Complex::real(half_square * interval).over(gap),
                )
            }
        };
        let mut slopes = [Complex::default(); FUNDING_NETWORK_MAX_ORDER];
        let mut power = ratio;
        let mut rung = 1.0;
        for target in slopes.iter_mut().take(order(self.rungs)) {
            *target = power.times(slope).scale(rung);
            power = power.times(ratio);
            rung += 1.0;
        }
        slopes
    }
}

impl Integrand for Correction<'_> {
    fn sample(&mut self, v: f64) -> Option<Sample> {
        // On the contour `z = iu = 1/2 + iv` and `z·(z − 1) = −(v² + 1/4)`.
        let square = v * v + 0.25;
        let half_square = -0.5 * square;
        let spread_square = self.spread * self.spread;
        // `b = λ − ρ·ξ·z`, `d = √(b² − ξ²·z·(z − 1))`.
        let tilt = Complex::new(
            self.reversion - self.correlation * self.spread * CONTOUR,
            -self.correlation * self.spread * v,
        );
        let root = tilt
            .times(tilt)
            .plus(Complex::real(spread_square * square))
            .square_root();
        let sum = tilt.plus(root);
        // `(b − d)/ξ²` and `g = (b − d)/(b + d)`, written with `b + d` only:
        // `b − d = ξ²·z·(z − 1)/(b + d)`.
        let level = Complex::real(2.0 * half_square).over(sum);
        let ratio = level.scale(spread_square).over(sum);
        let one_minus_ratio = Complex::real(1.0).minus(ratio);
        // Exponent of the Black-Scholes model of variance `v₀`.
        let exponent = Complex::new(
            self.drift * CONTOUR + half_square * self.variance,
            self.drift * v,
        );

        let mut totals = [Complex::default(); FUNDING_NETWORK_MAX_ORDER];
        let mut quiet = false;
        for node in self.nodes {
            let time = node.time;
            // `e^{−d·s} − 1`.
            let decay = root.scale(-time).exp_m1();
            let one_minus_decay = decay.scale(-1.0);
            let remaining = Complex::new(1.0 + decay.re, decay.im);
            // `D(s) = level·(1 − e^{−ds})/(1 − g·e^{−ds})`.
            let slope =
                level.times(one_minus_decay.over(Complex::real(1.0).minus(ratio.times(remaining))));
            // `ln((1 − g·e^{−ds})/(1 − g)) = ln(1 + g·(1 − e^{−ds})/(1 − g))`.
            let logarithm = ratio.times(one_minus_decay).over(one_minus_ratio).ln_1p();
            // `C(s) = λθ·(level·s − (2/ξ²)·ln(…))`.
            let constant = level
                .scale(time)
                .minus(logarithm.scale(2.0 / spread_square))
                .scale(self.pull);
            // What the exponent of Heston adds to that of Black-Scholes.
            let shift = constant
                .plus(Complex::new(slope.re - half_square * time, slope.im).scale(self.variance));
            let base = exponent.scale(time);
            let black_scholes = base.exponential();
            let heston = base.plus(shift).exponential();
            let term = match self.target {
                Target::Price | Target::Delta => {
                    if shift.norm() < 0.5 {
                        black_scholes.times(shift.exp_m1())
                    } else {
                        heston.minus(black_scholes)
                    }
                }
                Target::Variance => slope
                    .times(heston)
                    .minus(black_scholes.scale(half_square * time)),
            };
            if !term.re.is_finite() || !term.im.is_finite() {
                return None;
            }
            for (total, weight) in totals.iter_mut().zip(node.weights.iter()) {
                *total = total.plus(term.scale(*weight));
            }
            // Both transforms decay with the maturity: two negligible nodes
            // in a row end the sum.
            if black_scholes.norm() < NEGLIGIBLE && heston.norm() < NEGLIGIBLE {
                if quiet {
                    break;
                }
                quiet = true;
            } else {
                quiet = false;
            }
        }
        if self.target == Target::Variance {
            let slopes = self.black_scholes_slope(exponent, half_square);
            for (total, slope) in totals.iter_mut().zip(slopes.iter()) {
                *total = total.plus(*slope);
            }
        }

        let (sine, cosine) = (v * self.log_moneyness).sin_cos();
        let phase = Complex::new(cosine, sine);
        // The transform of the payoff: `1/(z·(z − 1))`, and `1/(z − 1)` for
        // the delta.
        let factor = match self.target {
            Target::Price | Target::Variance => phase.scale(-1.0 / square),
            Target::Delta => phase.over(Complex::new(CONTOUR - 1.0, v)),
        };
        let mut sample = Sample {
            real: [0.0; FUNDING_NETWORK_MAX_ORDER],
            modulus: [0.0; FUNDING_NETWORK_MAX_ORDER],
        };
        for ((real, modulus), total) in sample
            .real
            .iter_mut()
            .zip(sample.modulus.iter_mut())
            .zip(totals.iter())
            .take(order(self.rungs))
        {
            let value = total.times(factor);
            if !value.re.is_finite() || !value.im.is_finite() {
                return None;
            }
            *real = value.re;
            *modulus = value.norm();
        }
        Some(sample)
    }
}

/// Prices, deltas or sensitivities to the variance of every rung, in `f64`.
fn ladder_values(
    params: &HestonLadderParams,
    schedule: FundingSchedule,
    target: Target,
) -> Result<Values, PricingError> {
    let spot = decimal_to_f64(params.underlying_price.to_dec())?;
    let strike = decimal_to_f64(params.strike_price.to_dec())?;
    let drift = decimal_to_f64(params.drift)?;
    let funding_rate = decimal_to_f64(params.funding_rate.to_dec())?;
    let variance = decimal_to_f64(params.model.initial_variance.to_dec())?;
    let long_run = decimal_to_f64(params.model.long_run_variance.to_dec())?;
    let reversion = decimal_to_f64(params.model.mean_reversion.to_dec())?;
    let spread = decimal_to_f64(params.model.volatility_of_variance.to_dec())?;
    let correlation = decimal_to_f64(params.model.correlation)?;

    let network = FundingNetwork::ladder(params.funding_rate, params.rungs)?;
    let decay_rate = network.scheduled_decay(schedule)?;
    if drift >= decay_rate {
        return Err(PricingError::invalid_parameter(
            "drift",
            params.drift,
            "must be below the decay rate of the ladder",
        ));
    }
    let interval = match schedule {
        FundingSchedule::Continuous => None,
        FundingSchedule::Discrete { interval } => Some(decimal_to_f64(interval.to_dec())?),
    };
    let nodes = match interval {
        None => continuous_nodes(funding_rate, params.rungs),
        Some(interval) => discrete_nodes(funding_rate * interval, interval, params.rungs)?,
    };

    let log_moneyness = ln_f64(spot / strike);
    let mut correction = Correction {
        nodes: &nodes,
        rungs: params.rungs,
        target,
        log_moneyness,
        drift,
        variance,
        pull: reversion * long_run,
        reversion,
        spread,
        correlation,
        funding_rate,
        interval,
    };
    let integral = integrate(&mut correction, log_moneyness.abs(), CONTOUR)?;
    let scale = match target {
        Target::Price | Target::Variance => {
            strike * exp_f64(CONTOUR * log_moneyness) / std::f64::consts::PI
        }
        Target::Delta => exp_f64((CONTOUR - 1.0) * log_moneyness) / std::f64::consts::PI,
    };
    let mut values = [0.0; FUNDING_NETWORK_MAX_ORDER];
    for (target, value) in values.iter_mut().zip(integral.iter()) {
        *target = scale * value;
    }
    let quantity = match target {
        Target::Price => Quantity::Price,
        Target::Delta => Quantity::Delta,
        Target::Variance => return Ok(values),
    };

    // The Black-Scholes ladder of variance `v₀`, which the correction is
    // measured from.
    let volatility = Positive::new_decimal(to_decimal(
        "pricing::heston_ladder::volatility",
        sqrt_f64(variance),
    )?)?;
    let black_scholes = FundingNetworkParams::try_new(
        params.underlying_price,
        params.strike_price,
        params.drift,
        LevyModel::BlackScholes { volatility },
        params.option_style,
    )?;
    let base = network_values(&network, &black_scholes, schedule, quantity)?;
    for (target, value) in values.iter_mut().zip(base.iter()) {
        *target += value;
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pricing::everlasting::{EverlastingParams, everlasting_price};
    use optionstratlib_core::pos_or_panic;
    use rust_decimal::prelude::ToPrimitive;
    use rust_decimal_macros::dec;

    /// A variance well above its long-run level, strongly mean reverting,
    /// with a large volatility of the variance: `(v₀, θ, λ, ξ, ρ)`.
    const CRYPTO: [f64; 5] = [0.25, 0.20, 3.0, 0.9, -0.6];
    /// A variance below its long-run level that reverts slowly.
    const EQUITY: [f64; 5] = [0.04, 0.09, 1.5, 0.5, -0.7];

    fn decimal(value: f64) -> Decimal {
        Decimal::try_from(value).unwrap()
    }

    fn model(values: [f64; 5]) -> HestonModel {
        HestonModel::try_new(
            pos_or_panic!(values[0]),
            pos_or_panic!(values[1]),
            pos_or_panic!(values[2]),
            pos_or_panic!(values[3]),
            decimal(values[4]),
        )
        .unwrap()
    }

    fn params(
        spot: f64,
        mu: f64,
        values: [f64; 5],
        rate: f64,
        rungs: u32,
        style: OptionStyle,
    ) -> HestonLadderParams {
        HestonLadderParams::try_new(
            pos_or_panic!(spot),
            Positive::HUNDRED,
            decimal(mu),
            model(values),
            pos_or_panic!(rate),
            NonZeroU32::new(rungs).unwrap(),
            style,
        )
        .unwrap()
    }

    fn discrete(interval: f64) -> FundingSchedule {
        FundingSchedule::Discrete {
            interval: pos_or_panic!(interval),
        }
    }

    fn floats(values: &[Decimal]) -> Vec<f64> {
        values.iter().map(|value| value.to_f64().unwrap()).collect()
    }

    fn assert_close(actual: &[f64], expected: &[f64], tolerance: f64) {
        assert_eq!(actual.len(), expected.len());
        for (a, e) in actual.iter().zip(expected) {
            assert!((a - e).abs() < tolerance, "{a} against {e}");
        }
    }

    // Reference values: mixtures of European Heston values, each from the
    // formula of Lewis by adaptive quadrature, integrated against the Erlang
    // density or summed against the negative binomial law (SciPy).

    #[test]
    fn test_heston_ladder_prices_continuous_match_reference() {
        let cases: [(f64, [f64; 3]); 3] = [
            (
                80.0,
                [0.028_309_701_110, 0.102_321_910_142, 0.220_894_486_720],
            ),
            (
                100.0,
                [2.887_007_998_561, 4.294_444_256_857, 5.325_732_814_994],
            ),
            (
                125.0,
                [25.107_803_593_854, 25.341_775_991_267, 25.665_740_733_860],
            ),
        ];
        for (spot, expected) in cases {
            let prices = heston_ladder_prices(
                &params(spot, 0.0, CRYPTO, 36.5, 3, OptionStyle::Call),
                FundingSchedule::Continuous,
            )
            .unwrap();
            assert_close(&floats(&prices), &expected, 1e-7);
        }
    }

    #[test]
    fn test_heston_ladder_prices_continuous_with_drift_match_reference() {
        let cases: [(f64, [f64; 3]); 3] = [
            (
                80.0,
                [0.004_039_835_651, 0.020_901_715_845, 0.061_178_349_169],
            ),
            (
                100.0,
                [2.336_916_383_050, 3.692_233_540_628, 4.818_364_175_055],
            ),
            (
                125.0,
                [25.575_883_064_442, 26.229_567_374_405, 26.951_238_110_418],
            ),
        ];
        for (spot, expected) in cases {
            let prices = heston_ladder_prices(
                &params(spot, 0.05, EQUITY, 12.0, 3, OptionStyle::Call),
                FundingSchedule::Continuous,
            )
            .unwrap();
            assert_close(&floats(&prices), &expected, 1e-7);
        }
        let puts = heston_ladder_prices(
            &params(100.0, 0.05, EQUITY, 12.0, 3, OptionStyle::Put),
            FundingSchedule::Continuous,
        )
        .unwrap();
        assert_close(
            &floats(&puts),
            &[1.918_506_341_209, 2.853_662_787_315, 3.557_874_715_661],
            1e-7,
        );
    }

    #[test]
    fn test_heston_ladder_prices_discrete_match_reference() {
        let daily: [(f64, [f64; 3]); 3] = [
            (
                80.0,
                [0.026_006_727_647, 0.097_795_298_204, 0.215_606_870_610],
            ),
            (
                100.0,
                [2.944_215_401_460, 4.330_154_535_083, 5.353_538_706_878],
            ),
            (
                125.0,
                [25.101_385_052_662, 25.332_627_165_245, 25.658_173_045_707],
            ),
        ];
        for (spot, expected) in daily {
            let prices = heston_ladder_prices(
                &params(spot, 0.0, CRYPTO, 36.5, 3, OptionStyle::Call),
                discrete(1.0 / 365.0),
            )
            .unwrap();
            assert_close(&floats(&prices), &expected, 1e-7);
        }
        let weekly: [(f64, [f64; 3]); 3] = [
            (
                80.0,
                [0.002_670_006_985, 0.015_656_127_917, 0.049_906_865_556],
            ),
            (
                100.0,
                [2.413_325_896_266, 3.739_353_944_078, 4.853_576_234_692],
            ),
            (
                125.0,
                [25.566_679_753_427, 26.213_306_717_288, 26.934_271_396_332],
            ),
        ];
        for (spot, expected) in weekly {
            let prices = heston_ladder_prices(
                &params(spot, 0.05, EQUITY, 12.0, 3, OptionStyle::Call),
                discrete(7.0 / 365.0),
            )
            .unwrap();
            assert_close(&floats(&prices), &expected, 1e-7);
        }
    }
    #[test]
    fn test_heston_ladder_sensitivities_match_reference() {
        // Central differences of the reference prices, at the money. The
        // third derivative of the first rung is large at the strike, and a
        // difference over 0.1 % of the spot sees it.
        let at_the_money = params(100.0, 0.0, CRYPTO, 36.5, 3, OptionStyle::Call);
        let deltas = heston_ladder_deltas(&at_the_money, FundingSchedule::Continuous).unwrap();
        assert_close(
            &floats(&deltas),
            &[0.529_891_634, 0.544_288_811, 0.554_657_282],
            1e-5,
        );
        let slopes =
            heston_ladder_variance_sensitivities(&at_the_money, FundingSchedule::Continuous)
                .unwrap();
        assert_close(
            &floats(&slopes),
            &[5.547_619_24, 8.030_980_56, 9.689_191_88],
            1e-5,
        );
    }

    #[test]
    fn test_heston_ladder_sensitivities_match_finite_differences() {
        for (schedule, spot, style) in [
            (FundingSchedule::Continuous, 90.0, OptionStyle::Put),
            (discrete(1.0 / 365.0), 115.0, OptionStyle::Call),
        ] {
            {
                let price = |spot: f64, variance: f64| {
                    let mut values = EQUITY;
                    values[0] = variance;
                    floats(
                        &heston_ladder_prices(
                            &params(spot, 0.03, values, 12.0, 4, style),
                            schedule,
                        )
                        .unwrap(),
                    )
                };
                let centre = params(spot, 0.03, EQUITY, 12.0, 4, style);
                let deltas = floats(&heston_ladder_deltas(&centre, schedule).unwrap());
                let slopes =
                    floats(&heston_ladder_variance_sensitivities(&centre, schedule).unwrap());
                let (up, down) = (
                    price(spot * 1.001, EQUITY[0]),
                    price(spot * 0.999, EQUITY[0]),
                );
                let (above, below) = (price(spot, EQUITY[0] + 1e-4), price(spot, EQUITY[0] - 1e-4));
                for rung in 0..4 {
                    let delta = (up[rung] - down[rung]) / (0.002 * spot);
                    let slope = (above[rung] - below[rung]) / 2e-4;
                    assert!((deltas[rung] - delta).abs() < 1e-5, "delta of rung {rung}");
                    assert!((slopes[rung] - slope).abs() < 1e-4, "slope of rung {rung}");
                }
            }
        }
    }

    #[test]
    fn test_heston_ladder_prices_without_volatility_of_variance_are_black_scholes() {
        // With `v₀ = θ` and a variance that barely moves, the ladder is the
        // Black-Scholes one, which has a closed form. The correlation leaves
        // a skew of the order of the volatility of the variance.
        let flat = [0.16, 0.16, 2.0, 1e-6, -0.5];
        for spot in [85.0, 100.0, 120.0] {
            let prices = floats(
                &heston_ladder_prices(
                    &params(spot, 0.02, flat, 20.0, 4, OptionStyle::Call),
                    FundingSchedule::Continuous,
                )
                .unwrap(),
            );
            for (index, price) in prices.iter().enumerate() {
                let closed = everlasting_price(
                    &EverlastingParams::try_new(
                        pos_or_panic!(spot),
                        Positive::HUNDRED,
                        pos_or_panic!(0.4),
                        dec!(0.02),
                        pos_or_panic!(20.0),
                        NonZeroU32::new(u32::try_from(index).unwrap() + 1).unwrap(),
                        OptionStyle::Call,
                    )
                    .unwrap(),
                )
                .unwrap()
                .to_f64()
                .unwrap();
                assert!((price - closed).abs() < 1e-6, "{price} against {closed}");
            }
        }
    }

    #[test]
    fn test_heston_ladder_prices_parity_gives_the_forward_of_each_rung() {
        // `C_j − P_j = S·(κ/(κ − μ))^j − K`, whatever the variance does.
        let (rate, mu, spot) = (12.0, 0.05, 110.0);
        let calls = floats(
            &heston_ladder_prices(
                &params(spot, mu, EQUITY, rate, 3, OptionStyle::Call),
                FundingSchedule::Continuous,
            )
            .unwrap(),
        );
        let puts = floats(
            &heston_ladder_prices(
                &params(spot, mu, EQUITY, rate, 3, OptionStyle::Put),
                FundingSchedule::Continuous,
            )
            .unwrap(),
        );
        let mut forward = 1.0;
        for (call, put) in calls.iter().zip(&puts) {
            forward *= rate / (rate - mu);
            assert!((call - put - (spot * forward - 100.0)).abs() < 1e-9);
        }
    }

    #[test]
    fn test_heston_ladder_prices_with_shrinking_interval_tend_to_continuous() {
        let inputs = params(100.0, 0.0, CRYPTO, 36.5, 2, OptionStyle::Call);
        let limit = floats(&heston_ladder_prices(&inputs, FundingSchedule::Continuous).unwrap());
        let mut previous = f64::INFINITY;
        for hours in [24.0, 8.0, 1.0] {
            let prices = floats(&heston_ladder_prices(&inputs, discrete(hours / 8760.0)).unwrap());
            let gap = (prices[1] - limit[1]).abs();
            assert!(gap < previous, "{hours} hours: {gap}");
            previous = gap;
        }
        assert!(previous < 2e-3);
    }

    #[test]
    fn test_heston_ladder_prices_far_from_the_money_stay_small_and_finite() {
        let prices = floats(
            &heston_ladder_prices(
                &params(30.0, 0.0, CRYPTO, 36.5, 3, OptionStyle::Call),
                FundingSchedule::Continuous,
            )
            .unwrap(),
        );
        for price in prices {
            assert!(price.abs() < 1e-4, "{price}");
        }
        let prices = floats(
            &heston_ladder_prices(
                &params(300.0, 0.0, CRYPTO, 36.5, 3, OptionStyle::Put),
                FundingSchedule::Continuous,
            )
            .unwrap(),
        );
        for price in prices {
            assert!(price.abs() < 1e-3, "{price}");
        }
    }

    #[test]
    fn test_continuous_nodes_carry_the_erlang_laws() {
        // Total mass one and mean `j/κ`, less what lies under the shortest
        // node.
        let rate = 12.0;
        let nodes = continuous_nodes(rate, NonZeroU32::new(4).unwrap());
        for rung in 0..4 {
            let mass: f64 = nodes.iter().map(|node| node.weights[rung]).sum();
            let mean: f64 = nodes
                .iter()
                .map(|node| node.weights[rung] * node.time)
                .sum();
            assert!((mass - 1.0).abs() < 2e-6, "mass of rung {rung}: {mass}");
            let expected = (rung as f64 + 1.0) / rate;
            assert!(
                (mean - expected).abs() < 1e-10,
                "mean of rung {rung}: {mean}"
            );
        }
        assert!(nodes.windows(2).all(|pair| pair[0].time < pair[1].time));
    }

    #[test]
    fn test_discrete_nodes_sum_a_decaying_function_as_the_payments_do() {
        // `Σ_k P(τ_j = k)·f(k·Δ)` for `f(s) = e^{−cs}·cos(ωs)`, against the
        // sum over every payment.
        let (share, interval) = (0.004, 1.0 / 1000.0);
        let rungs = NonZeroU32::new(3).unwrap();
        let nodes = discrete_nodes(share, interval, rungs).unwrap();
        for (decay, turn) in [(0.5, 0.0), (8.0, 3.0), (60.0, 25.0), (400.0, 90.0)] {
            let function = |time: f64| (-decay * time).exp() * (turn * time).cos();
            let mut exact = [0.0; 3];
            let mut mass = [0.0; FUNDING_NETWORK_MAX_ORDER];
            advance(&mut mass, rungs, share, 1.0);
            for payment in 1..200_000 {
                let value = function(f64::from(payment) * interval);
                for rung in 0..3 {
                    exact[rung] += mass[rung] * value;
                }
                advance(&mut mass, rungs, share, 0.0);
            }
            for rung in 0..3 {
                let quadrature: f64 = nodes
                    .iter()
                    .map(|node| node.weights[rung] * function(node.time))
                    .sum();
                assert!(
                    (quadrature - exact[rung]).abs() < 1e-11,
                    "rung {rung}, decay {decay}: {quadrature} against {}",
                    exact[rung]
                );
            }
        }
    }

    #[test]
    fn test_heston_model_rejects_values_outside_the_domain() {
        let half = pos_or_panic!(0.5);
        let build = |variance, level, speed, spread, correlation| {
            HestonModel::try_new(variance, level, speed, spread, correlation)
        };
        assert!(build(Positive::ZERO, half, half, half, dec!(0)).is_err());
        assert!(build(half, Positive::ZERO, half, half, dec!(0)).is_err());
        assert!(build(half, half, Positive::ZERO, half, dec!(0)).is_err());
        assert!(build(half, half, half, Positive::ZERO, dec!(0)).is_err());
        assert!(build(half, half, half, half, dec!(1)).is_err());
        assert!(build(half, half, half, half, dec!(-1)).is_err());
        let model = build(half, half, half, half, dec!(-0.9)).unwrap();
        assert_eq!(model.initial_variance(), half);
        assert_eq!(model.long_run_variance(), half);
        assert_eq!(model.mean_reversion(), half);
        assert_eq!(model.volatility_of_variance(), half);
        assert_eq!(model.correlation(), dec!(-0.9));
    }

    #[test]
    fn test_heston_ladder_params_rejects_values_outside_the_domain() {
        let build = |spot, strike, rate, rungs: u32| {
            HestonLadderParams::try_new(
                spot,
                strike,
                dec!(0),
                model(CRYPTO),
                rate,
                NonZeroU32::new(rungs).unwrap(),
                OptionStyle::Call,
            )
        };
        let rate = pos_or_panic!(12.0);
        assert!(build(Positive::ZERO, Positive::HUNDRED, rate, 2).is_err());
        assert!(build(Positive::HUNDRED, Positive::ZERO, rate, 2).is_err());
        assert!(build(Positive::HUNDRED, Positive::HUNDRED, Positive::ZERO, 2).is_err());
        assert!(build(Positive::HUNDRED, Positive::HUNDRED, rate, 33).is_err());
        let inputs = build(Positive::HUNDRED, Positive::HUNDRED, rate, 16).unwrap();
        assert_eq!(inputs.underlying_price(), Positive::HUNDRED);
        assert_eq!(inputs.strike_price(), Positive::HUNDRED);
        assert_eq!(inputs.drift(), dec!(0));
        assert_eq!(inputs.model(), model(CRYPTO));
        assert_eq!(inputs.funding_rate(), rate);
        assert_eq!(inputs.rungs().get(), 16);
        assert_eq!(inputs.option_style(), OptionStyle::Call);
    }

    #[test]
    fn test_heston_ladder_prices_rejects_schedules_and_drifts_outside_the_domain() {
        // A drift at the funding rate: the forward of every rung is infinite.
        let fast = params(100.0, 12.0, EQUITY, 12.0, 2, OptionStyle::Call);
        assert!(matches!(
            heston_ladder_prices(&fast, FundingSchedule::Continuous),
            Err(PricingError::InvalidParameter {
                parameter: "drift",
                ..
            })
        ));
        assert!(heston_ladder_variance_sensitivities(&fast, FundingSchedule::Continuous).is_err());
        // Payments of more than twice the premium.
        let inputs = params(100.0, 0.0, EQUITY, 12.0, 2, OptionStyle::Call);
        assert!(heston_ladder_prices(&inputs, discrete(0.2)).is_err());
        // A payment interval of a few milliseconds.
        assert!(matches!(
            heston_ladder_prices(&inputs, discrete(1e-10)),
            Err(PricingError::MethodError { .. })
        ));
    }

    #[test]
    fn test_heston_ladder_prices_longer_ladder_keeps_the_lower_rungs() {
        // Rungs added on top of a ladder at the same rate leave the others
        // where they were, up to the largest order.
        let rate = 16.0 * 365.0 / 30.0;
        for schedule in [FundingSchedule::Continuous, discrete(1.0 / 8760.0)] {
            let short = floats(
                &heston_ladder_prices(
                    &params(96.0, 0.0, CRYPTO, rate, 16, OptionStyle::Call),
                    schedule,
                )
                .unwrap(),
            );
            for rungs in [18_u32, 32] {
                let long = floats(
                    &heston_ladder_prices(
                        &params(96.0, 0.0, CRYPTO, rate, rungs, OptionStyle::Call),
                        schedule,
                    )
                    .unwrap(),
                );
                assert_eq!(long.len(), rungs as usize);
                for (lower, same) in short.iter().zip(&long) {
                    assert!((lower - same).abs() < 1e-9, "{lower} against {same}");
                }
                assert!(long.windows(2).all(|pair| pair[0] < pair[1]));
            }
        }
    }

    #[test]
    fn test_heston_ladder_prices_thirty_two_rungs_without_volatility_of_variance_are_black_scholes()
    {
        // The largest order against the closed form of the Black-Scholes
        // ladder, with a variance that barely moves.
        let flat = [0.16, 0.16, 2.0, 1e-6, -0.5];
        let rate = 32.0 * 365.0 / 30.0;
        for spot in [85.0, 100.0, 120.0] {
            let prices = floats(
                &heston_ladder_prices(
                    &params(spot, 0.0, flat, rate, 32, OptionStyle::Call),
                    FundingSchedule::Continuous,
                )
                .unwrap(),
            );
            assert_eq!(prices.len(), 32);
            for (index, price) in prices.iter().enumerate() {
                let closed = everlasting_price(
                    &EverlastingParams::try_new(
                        pos_or_panic!(spot),
                        Positive::HUNDRED,
                        pos_or_panic!(0.4),
                        dec!(0),
                        pos_or_panic!(rate),
                        NonZeroU32::new(u32::try_from(index).unwrap() + 1).unwrap(),
                        OptionStyle::Call,
                    )
                    .unwrap(),
                )
                .unwrap()
                .to_f64()
                .unwrap();
                assert!((price - closed).abs() < 1e-6, "{price} against {closed}");
            }
        }
    }

    #[test]
    fn test_heston_ladder_prices_sixteen_rungs_hourly_funding_are_ordered_at_the_money() {
        // The largest ladder at the shortest interval in use: a longer
        // maturity is worth more at the money.
        let inputs = params(
            100.0,
            0.0,
            CRYPTO,
            16.0 * 365.0 / 30.0,
            16,
            OptionStyle::Call,
        );
        let prices = floats(&heston_ladder_prices(&inputs, discrete(1.0 / 8760.0)).unwrap());
        assert_eq!(prices.len(), 16);
        assert!(prices.windows(2).all(|pair| pair[0] < pair[1]));
        let continuous =
            floats(&heston_ladder_prices(&inputs, FundingSchedule::Continuous).unwrap());
        // Hourly payments move the top rungs by a few thousandths (the error
        // grows with `j·κ·Δ`).
        for (hourly, limit) in prices.iter().zip(&continuous) {
            assert!((hourly - limit).abs() < 2e-2, "{hourly} against {limit}");
        }
    }
}
