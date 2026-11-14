//! Funding Network Pricing Module
//!
//! A funding network is a set of `n` perpetual contracts on the same
//! underlying and the same payoff `φ` whose funding rates are linear in each
//! other's prices:
//!
//! ```text
//! f = A·(F − φ(S)·1)        f_i = Σ_j A_ij·(F_j − φ(S))
//! ```
//!
//! `F` is the vector of prices, `f` the funding each long side pays per year
//! and `A` the funding matrix. The everlasting option is the network of order
//! one, `A = (κ)`. A ladder is the bidiagonal network in which rung `j` is
//! funded against rung `j − 1`
//! (see [`crate::pricing::everlasting`]).
//!
//! # Prices
//!
//! A network is *stable* when every eigenvalue of `A` has a positive real
//! part. The smallest of those real parts is its decay rate `η(A)`. In a
//! stable network the prices are mixtures of European values,
//!
//! ```text
//! F_i = ∫ w_i(s)·E[φ(S_s)] ds        w(s) = exp(−A·s)·A·1
//! ```
//!
//! and each `w_i` has total mass one. When the cross-funding weights are not
//! negative (`A_ij ≤ 0` off the diagonal, `A·1 ≥ 0`) every `w_i` is a
//! phase-type density, the law of the maturity of contract `i`. With signed
//! weights a `w_i` may still be a density, or it may change sign: this module
//! prices the mixture in both cases and does not check the sign of `w_i`.
//!
//! # Model
//!
//! The log price follows a Lévy process `X` with `E[exp(iuX_s)] =
//! exp(s·χ(u))`, with a drift `μ` chosen by the caller so that
//! `E[S_s] = S·exp(μ·s)`; values are undiscounted. Pass the risk-neutral
//! drift of the underlying (zero for a futures or perpetual underlying) for
//! prices, or a real-world drift for expectations. [`LevyModel`] lists the
//! models: Black-Scholes and the jump diffusion of Merton.
//!
//! Integrating the maturity out gives the characteristic function of the log
//! return at the maturity of contract `i` as a rational function of `χ`:
//!
//! ```text
//! Ξ_i(u) = e_iᵀ·(A − χ(u)·I)⁻¹·A·1
//! ```
//!
//! which on a ladder is `(κ/(κ − χ(u)))^j`.
//!
//! # Method
//!
//! Prices follow by Fourier inversion along the line `Im u = −p`:
//!
//! ```text
//! V_i = (K/π)·∫₀^∞ Re[ exp(iu·ln(S/K))·Ξ_i(u) / ((iu)·(iu − 1)) ] dv,   u = v − ip
//! ```
//!
//! which is the call for `p > 1` and the put for `p < 0`. The side that is
//! out of the money is computed directly, the other one by parity,
//! `C_i − P_i = S·Ξ_i(−i) − K`. The delta uses the same integral without the
//! factor `iu`. Each frequency costs one complex linear system of size `n`,
//! and returns every contract of the network at once.
//!
//! The integral runs over panels that double in width until each holds two
//! cycles of the oscillation, with a 16 point Gauss-Legendre rule refined by
//! bisection, and stops when a bound on the tail falls below `1e-12`. The
//! damping `p` is chosen so that the exponential moment of order `p` stays
//! inside the decay rate, `χ(−ip) < η(A)`. Against the closed form of a
//! ladder the absolute error stays below `1e-12·K` in the cases tested, from
//! hourly to yearly funding rates and up to sixteen rungs, and the relative
//! error near the money below `1e-9`.
//!
//! The spectrum of `A` comes from a shifted QR iteration in complex
//! arithmetic, or from the diagonal when `A` is triangular. The complex
//! arithmetic and the dense solver are private to this module: no linear
//! algebra dependency is involved, and the order is bounded by
//! [`FUNDING_NETWORK_MAX_ORDER`].
//!
//! # Example
//!
//! ```rust
//! use optionstratlib_core::model::Positive;
//! use optionstratlib_core::model::types::OptionStyle;
//! use optionstratlib_pricing::pricing::{
//!     EverlastingParams, FundingNetwork, FundingNetworkParams, LevyModel,
//!     everlasting_price, funding_network_prices,
//! };
//! use rust_decimal_macros::dec;
//! use std::num::NonZeroU32;
//!
//! let rate = Positive::new_decimal(dec!(36.5))?;
//! let volatility = Positive::new_decimal(dec!(0.5))?;
//! let three = NonZeroU32::MIN.saturating_add(2);
//!
//! // A ladder of three rungs: the top one has a mean maturity of 30 days.
//! let ladder = FundingNetwork::ladder(rate, three)?;
//! let params = FundingNetworkParams::try_new(
//!     Positive::HUNDRED,
//!     Positive::HUNDRED,
//!     dec!(0),
//!     LevyModel::BlackScholes { volatility },
//!     OptionStyle::Call,
//! )?;
//! let prices = funding_network_prices(&ladder, &params)?;
//!
//! // Under Black-Scholes the top rung has a closed form.
//! let closed = everlasting_price(&EverlastingParams::try_new(
//!     Positive::HUNDRED,
//!     Positive::HUNDRED,
//!     volatility,
//!     dec!(0),
//!     rate,
//!     three,
//!     OptionStyle::Call,
//! )?)?;
//! assert!(matches!(prices.get(2), Some(top) if (*top - closed).abs() < dec!(0.000001)));
//! # Ok::<(), optionstratlib_pricing::error::PricingError>(())
//! ```

use crate::error::PricingError;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::{decimal_to_f64, finite_decimal};
use optionstratlib_core::model::types::OptionStyle;
use rust_decimal::Decimal;
use std::num::NonZeroU32;

/// Largest number of contracts in a funding network.
pub const FUNDING_NETWORK_MAX_ORDER: usize = 16;

/// Method name carried by the errors of this module.
const METHOD: &str = "funding_network";

/// Points of the Gauss-Legendre rule used on every panel.
const NODES: usize = 16;

/// `NODES` as a float, for the Legendre recurrence.
const NODES_F64: f64 = 16.0;

/// Absolute tolerance of one panel of the Fourier integral.
const PANEL_TOLERANCE: f64 = 1e-13;

/// Bound on the tail of the Fourier integral at which it is truncated.
const TAIL_TOLERANCE: f64 = 1e-12;

/// Panels after which the Fourier integral is declared not convergent.
const MAX_PANELS: u32 = 1 << 15;

/// Bisections of one panel before it is declared not integrable.
const MAX_DEPTH: u32 = 12;

/// Smallest decay rate accepted as positive, relative to the largest entry
/// of the funding matrix.
const STABILITY_MARGIN: f64 = 1e-12;

/// QR sweeps allowed for one eigenvalue.
const MAX_SWEEPS: u32 = 200;

/// One value per contract; entries past the order of the network are zero.
type Values = [f64; FUNDING_NETWORK_MAX_ORDER];

/// Funding matrix of a stable funding network.
///
/// Row `i` holds the weights with which contract `i` is funded against every
/// contract of the network: `f_i = Σ_j A_ij·(F_j − φ(S))`. Contracts are
/// numbered from zero. Built with [`FundingNetwork::try_new`] or
/// [`FundingNetwork::ladder`], which check that the matrix is square, of
/// bounded order and stable.
#[derive(Debug, Clone, PartialEq)]
pub struct FundingNetwork {
    entries: Vec<Vec<Decimal>>,
    values: Vec<Vec<f64>>,
    anchor: Vec<f64>,
    decay_rate: f64,
    /// Eigenvalues of the funding matrix, as (real part, imaginary part).
    spectrum: Vec<(f64, f64)>,
    non_negative: bool,
}

impl FundingNetwork {
    /// Builds a funding network from the rows of its matrix, in units of one
    /// over years.
    ///
    /// # Errors
    ///
    /// - [`PricingError::InvalidParameter`] when the matrix is empty, has
    ///   more than [`FUNDING_NETWORK_MAX_ORDER`] rows, is not square, has a
    ///   row whose sum leaves the `Decimal` range, or is not stable (an
    ///   eigenvalue has a real part that is not positive to working
    ///   precision).
    /// - [`PricingError::Decimal`] when an entry has no `f64` form.
    /// - [`PricingError::MethodError`] when the eigenvalue iteration does
    ///   not converge.
    pub fn try_new(rows: &[Vec<Decimal>]) -> Result<Self, PricingError> {
        let order = rows.len();
        if order == 0 || order > FUNDING_NETWORK_MAX_ORDER {
            return Err(PricingError::invalid_parameter(
                "order",
                Decimal::from(order),
                "must be between 1 and FUNDING_NETWORK_MAX_ORDER",
            ));
        }
        let mut values = Vec::with_capacity(order);
        let mut anchor = Vec::with_capacity(order);
        let mut non_negative = true;
        for (index, row) in rows.iter().enumerate() {
            if row.len() != order {
                return Err(PricingError::invalid_parameter(
                    "row_length",
                    Decimal::from(row.len()),
                    "every row needs as many entries as the matrix has rows",
                ));
            }
            let mut converted = Vec::with_capacity(order);
            let mut row_sum = Decimal::ZERO;
            for (column, entry) in row.iter().enumerate() {
                converted.push(decimal_to_f64(*entry)?);
                row_sum = row_sum.checked_add(*entry).ok_or_else(|| {
                    PricingError::invalid_parameter(
                        "funding_matrix",
                        *entry,
                        "the sum of a row leaves the Decimal range",
                    )
                })?;
                if column != index && *entry > Decimal::ZERO {
                    non_negative = false;
                }
            }
            if row_sum < Decimal::ZERO {
                non_negative = false;
            }
            anchor.push(decimal_to_f64(row_sum)?);
            values.push(converted);
        }
        let spectrum: Vec<(f64, f64)> = eigenvalues(&values)?
            .iter()
            .map(|eigenvalue| (eigenvalue.re, eigenvalue.im))
            .collect();
        let decay_rate = spectrum
            .iter()
            .map(|(real, _)| *real)
            .fold(f64::INFINITY, f64::min);
        // An eigenvalue on the imaginary axis is computed with a real part
        // of the order of the rounding error, of either sign.
        let largest = values
            .iter()
            .flat_map(|row| row.iter())
            .fold(0.0_f64, |a, b| a.max(b.abs()));
        if decay_rate.is_nan() || decay_rate <= STABILITY_MARGIN * largest {
            return Err(PricingError::invalid_parameter(
                "funding_matrix",
                finite_decimal(decay_rate).unwrap_or_default(),
                "must be stable: every eigenvalue needs a positive real part",
            ));
        }
        Ok(Self {
            entries: rows.to_vec(),
            values,
            anchor,
            decay_rate,
            spectrum,
            non_negative,
        })
    }

    /// The ladder with `rungs` rungs and funding rate `funding_rate`: rung
    /// `j` is contract `j − 1`, funded against the payoff for `j = 1` and
    /// against rung `j − 1` otherwise. Its maturity is Erlang(`j`, `κ`).
    ///
    /// # Errors
    ///
    /// [`PricingError::InvalidParameter`] when the funding rate is zero or
    /// `rungs` exceeds [`FUNDING_NETWORK_MAX_ORDER`].
    pub fn ladder(funding_rate: Positive, rungs: NonZeroU32) -> Result<Self, PricingError> {
        reject_zero("funding_rate", funding_rate)?;
        let order = usize::try_from(rungs.get())
            .ok()
            .filter(|order| *order <= FUNDING_NETWORK_MAX_ORDER)
            .ok_or_else(|| {
                PricingError::invalid_parameter(
                    "rungs",
                    Decimal::from(rungs.get()),
                    "exceeds FUNDING_NETWORK_MAX_ORDER",
                )
            })?;
        let rate = funding_rate.to_dec();
        let rows: Vec<Vec<Decimal>> = (0..order)
            .map(|row| {
                (0..order)
                    .map(|column| {
                        if column == row {
                            rate
                        } else if column.checked_add(1) == Some(row) {
                            -rate
                        } else {
                            Decimal::ZERO
                        }
                    })
                    .collect()
            })
            .collect();
        Self::try_new(&rows)
    }

    /// Number of contracts.
    #[must_use]
    pub fn order(&self) -> usize {
        self.entries.len()
    }

    /// Entry `A_ij` of the funding matrix, or `None` outside it.
    #[must_use]
    pub fn entry(&self, row: usize, column: usize) -> Option<Decimal> {
        self.entries.get(row)?.get(column).copied()
    }

    /// `true` when no cross-funding weight is negative: `A_ij ≤ 0` off the
    /// diagonal and `A·1 ≥ 0`. Every contract of such a network has a
    /// phase-type maturity law.
    #[must_use]
    pub const fn is_non_negative(&self) -> bool {
        self.non_negative
    }

    /// The funding matrix in `f64`, row by row.
    pub(super) fn rows(&self) -> &[Vec<f64>] {
        &self.values
    }

    /// `A·1` in `f64`: the weight of the payoff in the funding of each
    /// contract.
    pub(super) fn anchor(&self) -> &[f64] {
        &self.anchor
    }

    /// Decay rate under a funding schedule, in `f64`: `η(A)` when funding
    /// accrues continuously, and `−ln(max |1 − Δ·λ|)/Δ` over the eigenvalues
    /// `λ` of `A` when it is paid every `Δ` years. Infinite when every
    /// `Δ·λ` is one.
    pub(super) fn scheduled_decay(&self, schedule: FundingSchedule) -> Result<f64, PricingError> {
        let interval = match schedule {
            FundingSchedule::Continuous => return Ok(self.decay_rate),
            FundingSchedule::Discrete { interval } => interval,
        };
        reject_zero("interval", interval)?;
        let years = decimal_to_f64(interval.to_dec())?;
        // `|1 − Δ·λ|² − 1 = Δ·(Δ·|λ|² − 2·Re λ)`, the largest over the
        // spectrum: negative when the network is stable at this interval.
        let excess = self
            .spectrum
            .iter()
            .map(|(real, imaginary)| {
                years * (years * (real * real + imaginary * imaginary) - 2.0 * real)
            })
            .fold(f64::NEG_INFINITY, f64::max);
        if excess.is_nan() || excess >= 0.0 {
            return Err(PricingError::invalid_parameter(
                "interval",
                interval.to_dec(),
                "must be below the stability bound of the network",
            ));
        }
        Ok(-0.5 * excess.ln_1p() / years)
    }

    /// Decay rate of the network under a funding schedule, per year.
    ///
    /// With continuous funding it is [`FundingNetwork::decay_rate`]. With a
    /// payment every `Δ` years a mispricing is multiplied by `I − Δ·A` at
    /// each payment, so the rate is `−ln(max |1 − Δ·λ|)/Δ` over the
    /// eigenvalues `λ` of `A`. Exponential moments of the underlying up to
    /// this rate are finite at every maturity of the network.
    ///
    /// # Errors
    ///
    /// - [`PricingError::InvalidParameter`] when the interval is zero, or at
    ///   or above [`FundingNetwork::stable_interval_bound`].
    /// - [`PricingError::Decimal`] when the interval has no `f64` form.
    /// - [`PricingError::NonFinite`] when the rate has no `Decimal` form:
    ///   it is infinite when a single payment closes the whole gap.
    pub fn decay_rate_with(&self, schedule: FundingSchedule) -> Result<Decimal, PricingError> {
        to_decimal(
            "pricing::funding_network::decay_rate_with",
            self.scheduled_decay(schedule)?,
        )
    }

    /// Payment interval below which discrete funding is stable, in years:
    /// the smallest `2·Re λ/|λ|²` over the eigenvalues `λ` of the funding
    /// matrix.
    ///
    /// Funding paid every `Δ` years multiplies a mispricing by `I − Δ·A` at
    /// each payment; it clears only when every `|1 − Δ·λ|` is below one. For
    /// a single contract or a ladder of rate `κ` the bound is `2/κ`; complex
    /// eigenvalues, which only signed weights produce, lower it.
    ///
    /// # Errors
    ///
    /// [`PricingError::NonFinite`] when the bound has no `Decimal` form.
    pub fn stable_interval_bound(&self) -> Result<Decimal, PricingError> {
        let bound = self
            .spectrum
            .iter()
            .map(|(real, imaginary)| 2.0 * real / (real * real + imaginary * imaginary))
            .fold(f64::INFINITY, f64::min);
        to_decimal("pricing::funding_network::stable_interval_bound", bound)
    }

    /// Decay rate `η(A)`: the smallest real part of an eigenvalue of the
    /// funding matrix, per year. Exponential moments of the underlying up to
    /// this rate are finite at every maturity of the network, and under a
    /// linear response it is the rate at which a mispricing clears.
    ///
    /// # Errors
    ///
    /// [`PricingError::NonFinite`] when the rate has no `Decimal` form.
    pub fn decay_rate(&self) -> Result<Decimal, PricingError> {
        to_decimal("pricing::funding_network::decay_rate", self.decay_rate)
    }

    /// Mean of the maturity weights of contract `contract`, in years:
    /// `e_iᵀ·A⁻¹·1`.
    ///
    /// # Errors
    ///
    /// - [`PricingError::InvalidParameter`] when `contract` is not a
    ///   contract of the network.
    /// - [`PricingError::MethodError`] when the funding matrix is singular
    ///   to working precision.
    /// - [`PricingError::NonFinite`] when the result has no `Decimal` form.
    pub fn mean_maturity(&self, contract: usize) -> Result<Decimal, PricingError> {
        let (means, _) = self.maturity_moments()?;
        to_decimal(
            "pricing::funding_network::mean_maturity",
            pick(&means, self.order(), contract)?,
        )
    }

    /// Variance of the maturity weights of contract `contract`, in years
    /// squared: `2·e_iᵀ·A⁻²·1` minus the squared mean. Divided by the
    /// squared mean it is the squared coefficient of variation, `1/j` on
    /// rung `j` of a ladder.
    ///
    /// # Errors
    ///
    /// As [`FundingNetwork::mean_maturity`].
    pub fn maturity_variance(&self, contract: usize) -> Result<Decimal, PricingError> {
        let (means, second_halves) = self.maturity_moments()?;
        let mean = pick(&means, self.order(), contract)?;
        let second = 2.0 * pick(&second_halves, self.order(), contract)?;
        to_decimal(
            "pricing::funding_network::maturity_variance",
            second - mean * mean,
        )
    }

    /// `A⁻¹·1` and `A⁻²·1`: the first moment and half the second moment of
    /// the weights of every contract.
    fn maturity_moments(&self) -> Result<(Values, Values), PricingError> {
        let ones = vec![1.0; self.order()];
        let means = self.solve_real(0.0, &ones)?;
        let second_halves = self.solve_real(0.0, &means)?;
        Ok((means, second_halves))
    }

    /// Solves `(A − shift·I)·x = rhs` in real arithmetic.
    fn solve_real(&self, shift: f64, rhs: &[f64]) -> Result<Values, PricingError> {
        let mut workspace = Workspace::new(self.order());
        workspace.load(&self.values, Complex::real(shift), rhs);
        workspace.solve().ok_or_else(|| {
            PricingError::method_error(METHOD, "the funding matrix is singular at this shift")
        })?;
        let mut solution = [0.0; FUNDING_NETWORK_MAX_ORDER];
        for (target, value) in solution.iter_mut().zip(workspace.rhs.iter()) {
            *target = value.re;
        }
        Ok(solution)
    }
}

/// How the funding of a network is paid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FundingSchedule {
    /// Funding accrues continuously, at the rate `A·(F − φ(S)·1)` per year.
    Continuous,
    /// Funding is paid every `interval` years, each payment being
    /// `interval·A·(F − φ(S)·1)` on the prices at the payment time.
    Discrete {
        /// Time between two payments, in years.
        interval: Positive,
    },
}

/// Exponential Lévy model of the underlying.
///
/// The drift is not part of the model: it is an input of
/// [`FundingNetworkParams`], and the model is compensated so that
/// `E[S_s] = S·exp(μ·s)` whatever its jumps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LevyModel {
    /// Geometric Brownian motion.
    BlackScholes {
        /// Volatility of the diffusion, per square root of a year.
        volatility: Positive,
    },
    /// Jump diffusion of Merton: a geometric Brownian motion with compound
    /// Poisson jumps whose logarithms are normal.
    Merton {
        /// Volatility of the diffusion, per square root of a year.
        volatility: Positive,
        /// Jumps per year.
        jump_intensity: Positive,
        /// Mean of the logarithm of one jump.
        jump_mean: Decimal,
        /// Standard deviation of the logarithm of one jump.
        jump_volatility: Positive,
    },
}

impl LevyModel {
    /// Volatility of the diffusion.
    #[must_use]
    pub const fn volatility(&self) -> Positive {
        match self {
            Self::BlackScholes { volatility } | Self::Merton { volatility, .. } => *volatility,
        }
    }
}

/// Inputs of a call or put funding network: the market, the model and the
/// payoff shared by every contract.
///
/// Built with [`FundingNetworkParams::try_new`], which checks the domain of
/// every field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FundingNetworkParams {
    underlying_price: Positive,
    strike_price: Positive,
    drift: Decimal,
    model: LevyModel,
    option_style: OptionStyle,
}

impl FundingNetworkParams {
    /// Validates and builds the inputs.
    ///
    /// # Arguments
    ///
    /// * `underlying_price` - current price `S` of the underlying.
    /// * `strike_price` - strike `K` of the payoff.
    /// * `drift` - drift `μ` of the underlying per year: `E[S_s] = S·exp(μ·s)`.
    /// * `model` - the Lévy model of the log price.
    /// * `option_style` - call or put.
    ///
    /// # Errors
    ///
    /// [`PricingError::InvalidParameter`] when a price or the volatility of
    /// the diffusion is zero.
    pub fn try_new(
        underlying_price: Positive,
        strike_price: Positive,
        drift: Decimal,
        model: LevyModel,
        option_style: OptionStyle,
    ) -> Result<Self, PricingError> {
        reject_zero("underlying_price", underlying_price)?;
        reject_zero("strike_price", strike_price)?;
        reject_zero("volatility", model.volatility())?;
        Ok(Self {
            underlying_price,
            strike_price,
            drift,
            model,
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

    /// Model of the log price.
    #[must_use]
    pub const fn model(&self) -> LevyModel {
        self.model
    }

    /// Call or put.
    #[must_use]
    pub const fn option_style(&self) -> OptionStyle {
        self.option_style
    }
}

/// Prices of every contract of a funding network, per unit of underlying,
/// in the order of the rows of the funding matrix.
///
/// The prices are not clamped at zero: with signed funding weights a mixture
/// may be negative, and far out of the money a price of a network with
/// maturity laws can fall below zero by the absolute error of the inversion.
///
/// # Errors
///
/// - [`PricingError::InvalidParameter`] when the drift is not below the
///   decay rate of the network: the forward of some contract is infinite.
/// - [`PricingError::MethodError`] when the Fourier inversion does not
///   converge, or no damping keeps the exponential moment finite.
/// - [`PricingError::Decimal`] when an input has no `f64` form.
/// - [`PricingError::NonFinite`] when a result has no `Decimal` form.
pub fn funding_network_prices(
    network: &FundingNetwork,
    params: &FundingNetworkParams,
) -> Result<Vec<Decimal>, PricingError> {
    funding_network_prices_with(network, params, FundingSchedule::Continuous)
}

/// Prices of every contract of a funding network under a funding schedule,
/// per unit of underlying, in the order of the rows of the funding matrix.
///
/// With [`FundingSchedule::Continuous`] this is [`funding_network_prices`].
/// With [`FundingSchedule::Discrete`] funding is paid every `Δ` years, each
/// payment being `Δ·A·(F − φ(S)·1)` on the prices at the payment time, and
/// the value is the price just after a payment:
///
/// ```text
/// F = Σ_{k≥1} (I − Δ·A)^{k−1}·Δ·A·1·E[φ(S_{kΔ})]
/// ```
///
/// Its transform is the continuous one with the characteristic exponent
/// `χ(u)` replaced by `(1 − e^{−Δ·χ(u)})/Δ`, so the same inversion applies.
/// For a single contract of rate `κ` the weights are `κΔ·(1 − κΔ)^{k−1}`;
/// with `κΔ = 1` the contract is the European option that expires at the
/// next payment.
///
/// # Errors
///
/// As [`funding_network_prices`], with the decay rate of the schedule
/// ([`FundingNetwork::decay_rate_with`]) in place of that of the network,
/// and [`PricingError::InvalidParameter`] when the interval is zero or not
/// below [`FundingNetwork::stable_interval_bound`].
pub fn funding_network_prices_with(
    network: &FundingNetwork,
    params: &FundingNetworkParams,
    schedule: FundingSchedule,
) -> Result<Vec<Decimal>, PricingError> {
    let values = network_values(network, params, schedule, Quantity::Price)?;
    collect("pricing::funding_network::price", &values, network.order())
}

/// Deltas of every contract of a funding network: the derivative of each
/// price with respect to the underlying price.
///
/// # Errors
///
/// As [`funding_network_prices`]. The integrand of the delta decays one
/// power slower than that of the price, so the inversion takes more panels.
pub fn funding_network_deltas(
    network: &FundingNetwork,
    params: &FundingNetworkParams,
) -> Result<Vec<Decimal>, PricingError> {
    funding_network_deltas_with(network, params, FundingSchedule::Continuous)
}

/// Deltas of every contract of a funding network under a funding schedule:
/// the derivative of each price of [`funding_network_prices_with`] with
/// respect to the underlying price.
///
/// # Errors
///
/// As [`funding_network_prices_with`].
pub fn funding_network_deltas_with(
    network: &FundingNetwork,
    params: &FundingNetworkParams,
    schedule: FundingSchedule,
) -> Result<Vec<Decimal>, PricingError> {
    let values = network_values(network, params, schedule, Quantity::Delta)?;
    collect("pricing::funding_network::delta", &values, network.order())
}

/// Funding paid per year by the long side of every contract, per unit of
/// underlying: `A·(F − φ(S)·1)` with `F` the prices of
/// [`funding_network_prices`] and `φ(S)` the payoff at the current price.
///
/// The funding rule is the pricing equation, so this is also the generator
/// of the model applied to each price.
///
/// # Errors
///
/// As [`funding_network_prices`].
pub fn funding_network_funding(
    network: &FundingNetwork,
    params: &FundingNetworkParams,
) -> Result<Vec<Decimal>, PricingError> {
    let prices = network_values(
        network,
        params,
        FundingSchedule::Continuous,
        Quantity::Price,
    )?;
    let spot = decimal_to_f64(params.underlying_price.to_dec())?;
    let strike = decimal_to_f64(params.strike_price.to_dec())?;
    let payoff = match params.option_style {
        OptionStyle::Call => (spot - strike).max(0.0),
        OptionStyle::Put => (strike - spot).max(0.0),
    };
    let mut funding = [0.0; FUNDING_NETWORK_MAX_ORDER];
    for (target, row) in funding.iter_mut().zip(network.values.iter()) {
        *target = row
            .iter()
            .zip(prices.iter())
            .map(|(weight, price)| weight * (price - payoff))
            .sum();
    }
    collect(
        "pricing::funding_network::funding",
        &funding,
        network.order(),
    )
}

/// Rejects a zero value of a parameter that must be strictly positive.
pub(super) fn reject_zero(parameter: &'static str, value: Positive) -> Result<(), PricingError> {
    if value == Positive::ZERO {
        return Err(PricingError::invalid_parameter(
            parameter,
            value.to_dec(),
            "must be strictly positive",
        ));
    }
    Ok(())
}

/// Converts a kernel result to `Decimal`, rejecting non-finite values.
pub(super) fn to_decimal(context: &'static str, value: f64) -> Result<Decimal, PricingError> {
    finite_decimal(value).ok_or_else(|| PricingError::non_finite(context, value))
}

/// The first `order` values as `Decimal`s.
fn collect(
    context: &'static str,
    values: &Values,
    order: usize,
) -> Result<Vec<Decimal>, PricingError> {
    values
        .iter()
        .take(order)
        .map(|value| to_decimal(context, *value))
        .collect()
}

/// The value of contract `contract`, or the error that names it.
fn pick(values: &Values, order: usize, contract: usize) -> Result<f64, PricingError> {
    values
        .iter()
        .take(order)
        .nth(contract)
        .copied()
        .ok_or_else(|| {
            PricingError::invalid_parameter(
                "contract",
                Decimal::from(contract),
                "is not a contract of the network",
            )
        })
}

#[inline]
pub(super) fn exp_f64(x: f64) -> f64 {
    x.exp() // scan-banned: allow -- f64 `exp`: returns inf on overflow, it does not abort; the non-finite value is rejected at the `Decimal` boundary
}

#[inline]
pub(super) fn ln_f64(x: f64) -> f64 {
    x.ln() // scan-banned: allow -- f64 `ln`: returns -inf or NaN outside its domain, it does not abort; the non-finite value is rejected at the `Decimal` boundary
}

#[inline]
pub(super) fn sqrt_f64(x: f64) -> f64 {
    x.sqrt() // scan-banned: allow -- f64 `sqrt`: returns NaN for negative input, it does not abort; the non-finite value is rejected at the `Decimal` boundary
}

#[inline]
fn cos_f64(x: f64) -> f64 {
    x.cos() // scan-banned: allow -- f64 `cos`: total on the floats, it does not abort
}

/// A complex number in `f64`. The operations are named, not operators, so
/// that the arithmetic lints of the crate keep applying to everything else.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Complex {
    re: f64,
    im: f64,
}

impl Complex {
    const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    const fn real(re: f64) -> Self {
        Self { re, im: 0.0 }
    }

    fn plus(self, other: Self) -> Self {
        Self::new(self.re + other.re, self.im + other.im)
    }

    fn minus(self, other: Self) -> Self {
        Self::new(self.re - other.re, self.im - other.im)
    }

    fn times(self, other: Self) -> Self {
        Self::new(
            self.re * other.re - self.im * other.im,
            self.re * other.im + self.im * other.re,
        )
    }

    fn scale(self, factor: f64) -> Self {
        Self::new(self.re * factor, self.im * factor)
    }

    fn conj(self) -> Self {
        Self::new(self.re, -self.im)
    }

    fn norm(self) -> f64 {
        self.re.hypot(self.im)
    }

    fn norm_sqr(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    /// Quotient by the algorithm of Smith, which does not overflow when the
    /// square of the divisor would.
    fn over(self, other: Self) -> Self {
        if other.re.abs() >= other.im.abs() {
            let ratio = other.im / other.re;
            let denominator = other.re + other.im * ratio;
            Self::new(
                (self.re + self.im * ratio) / denominator,
                (self.im - self.re * ratio) / denominator,
            )
        } else {
            let ratio = other.re / other.im;
            let denominator = other.re * ratio + other.im;
            Self::new(
                (self.re * ratio + self.im) / denominator,
                (self.im * ratio - self.re) / denominator,
            )
        }
    }

    fn exponential(self) -> Self {
        let modulus = exp_f64(self.re);
        // `sin_cos` is the f64 method: total on the floats.
        let (sine, cosine) = self.im.sin_cos();
        Self::new(modulus * cosine, modulus * sine)
    }

    /// Principal square root.
    fn square_root(self) -> Self {
        let modulus = self.norm();
        let re = sqrt_f64((0.5 * (modulus + self.re)).max(0.0));
        let im = sqrt_f64((0.5 * (modulus - self.re)).max(0.0));
        Self::new(re, if self.im < 0.0 { -im } else { im })
    }
}

/// Entry `(row, column)` of a complex matrix, zero outside it.
fn entry(matrix: &[Vec<Complex>], row: usize, column: usize) -> Complex {
    matrix
        .get(row)
        .and_then(|line| line.get(column))
        .copied()
        .unwrap_or_default()
}

/// A dense complex linear system, reused across frequencies.
struct Workspace {
    rows: Vec<Vec<Complex>>,
    rhs: Vec<Complex>,
}

impl Workspace {
    fn new(order: usize) -> Self {
        Self {
            rows: vec![vec![Complex::default(); order]; order],
            rhs: vec![Complex::default(); order],
        }
    }

    /// Loads `(A − shift·I)·x = rhs`.
    fn load(&mut self, values: &[Vec<f64>], shift: Complex, rhs: &[f64]) {
        for (index, (target, source)) in self.rows.iter_mut().zip(values.iter()).enumerate() {
            for (column, (cell, value)) in target.iter_mut().zip(source.iter()).enumerate() {
                *cell = if column == index {
                    Complex::real(*value).minus(shift)
                } else {
                    Complex::real(*value)
                };
            }
        }
        for (target, value) in self.rhs.iter_mut().zip(rhs.iter()) {
            *target = Complex::real(*value);
        }
    }

    /// Gaussian elimination with partial pivoting; the solution replaces the
    /// right-hand side. `None` when a pivot is zero or not finite.
    fn solve(&mut self) -> Option<()> {
        let order = self.rows.len();
        for column in 0..order {
            let mut pivot = column;
            let mut largest = 0.0;
            for (index, row) in self.rows.iter().enumerate().skip(column) {
                let size = row.get(column)?.norm_sqr();
                if size > largest {
                    largest = size;
                    pivot = index;
                }
            }
            if !largest.is_finite() || largest <= 0.0 {
                return None;
            }
            self.rows.swap(column, pivot);
            self.rhs.swap(column, pivot);
            let mut lines = self.rows.iter_mut().zip(self.rhs.iter_mut()).skip(column);
            let (pivot_row, pivot_rhs) = lines.next()?;
            let pivot_value = *pivot_row.get(column)?;
            for (row, value) in lines {
                let factor = row.get(column)?.over(pivot_value);
                for (target, source) in row.iter_mut().zip(pivot_row.iter()).skip(column) {
                    *target = target.minus(factor.times(*source));
                }
                *value = value.minus(factor.times(*pivot_rhs));
            }
        }
        for column in (0..order).rev() {
            let row = self.rows.get(column)?;
            let mut accumulated = *self.rhs.get(column)?;
            for (index, coefficient) in row.iter().enumerate().skip(column).skip(1) {
                accumulated = accumulated.minus(coefficient.times(*self.rhs.get(index)?));
            }
            *self.rhs.get_mut(column)? = accumulated.over(*row.get(column)?);
        }
        Some(())
    }
}

/// One plane rotation of a QR sweep, acting on lines `column` and `row`.
#[derive(Clone, Copy)]
struct Rotation {
    column: usize,
    row: usize,
    cosine: Complex,
    sine: Complex,
}

/// Eigenvalues of a real square matrix.
///
/// A triangular matrix returns its diagonal, which is exact: this covers
/// ladders, whose single eigenvalue of full multiplicity an iteration would
/// only resolve to the `n`-th root of the rounding error. Any other matrix
/// goes through a shifted QR iteration in complex arithmetic, deflating one
/// eigenvalue at a time from the bottom.
fn eigenvalues(values: &[Vec<f64>]) -> Result<Vec<Complex>, PricingError> {
    let order = values.len();
    let diagonal = || -> Vec<Complex> {
        values
            .iter()
            .enumerate()
            .map(|(index, row)| Complex::real(row.get(index).copied().unwrap_or_default()))
            .collect()
    };
    let upper_is_zero = values
        .iter()
        .enumerate()
        .all(|(index, row)| row.iter().skip(index).skip(1).all(|value| *value == 0.0));
    let lower_is_zero = values
        .iter()
        .enumerate()
        .all(|(index, row)| row.iter().take(index).all(|value| *value == 0.0));
    if upper_is_zero || lower_is_zero {
        return Ok(diagonal());
    }

    let mut matrix: Vec<Vec<Complex>> = values
        .iter()
        .map(|row| row.iter().map(|value| Complex::real(*value)).collect())
        .collect();
    let norm = sqrt_f64(
        values
            .iter()
            .flat_map(|row| row.iter())
            .map(|value| value * value)
            .sum::<f64>(),
    );
    let tolerance = 16.0 * f64::EPSILON * norm;
    let mut spectrum = Vec::with_capacity(order);
    let mut rotations: Vec<Rotation> = Vec::new();
    let mut active = order;
    let mut sweeps = 0_u32;
    while let Some(last) = active.checked_sub(1) {
        let corner = entry(&matrix, last, last);
        let Some(previous) = last.checked_sub(1) else {
            spectrum.push(corner);
            break;
        };
        let off_diagonal: f64 = matrix
            .get(last)
            .map(|row| row.iter().take(last).map(|value| value.norm()).sum())
            .unwrap_or_default();
        if off_diagonal <= tolerance {
            spectrum.push(corner);
            active = last;
            sweeps = 0;
            continue;
        }
        sweeps = sweeps
            .checked_add(1)
            .filter(|count| *count <= MAX_SWEEPS)
            .ok_or_else(|| {
                PricingError::method_error(METHOD, "the eigenvalue iteration did not converge")
            })?;
        let shift = if sweeps.is_multiple_of(10) {
            // An exceptional shift breaks a cycle of the iteration.
            corner.plus(Complex::new(off_diagonal, 0.5 * off_diagonal))
        } else {
            wilkinson_shift(
                entry(&matrix, previous, previous),
                entry(&matrix, previous, last),
                entry(&matrix, last, previous),
                corner,
            )
        };
        qr_sweep(&mut matrix, active, shift, &mut rotations);
    }
    Ok(spectrum)
}

/// The eigenvalue of `[[a, b], [c, d]]` closest to `d`.
fn wilkinson_shift(a: Complex, b: Complex, c: Complex, d: Complex) -> Complex {
    let mean = a.plus(d).scale(0.5);
    let half_gap = a.minus(d).scale(0.5);
    let root = half_gap.times(half_gap).plus(b.times(c)).square_root();
    let first = mean.plus(root);
    let second = mean.minus(root);
    if first.minus(d).norm_sqr() <= second.minus(d).norm_sqr() {
        first
    } else {
        second
    }
}

/// Adds `shift` to the diagonal of the leading block of size `active`.
fn shift_diagonal(matrix: &mut [Vec<Complex>], active: usize, shift: Complex) {
    for (index, row) in matrix.iter_mut().enumerate().take(active) {
        if let Some(cell) = row.get_mut(index) {
            *cell = cell.plus(shift);
        }
    }
}

/// One QR sweep with shift on the leading block of size `active`:
/// `H − μI = QR`, then `H ← RQ + μI`.
fn qr_sweep(
    matrix: &mut [Vec<Complex>],
    active: usize,
    shift: Complex,
    rotations: &mut Vec<Rotation>,
) {
    shift_diagonal(matrix, active, shift.scale(-1.0));
    rotations.clear();
    for column in 0..active {
        for row in (0..active).skip(column).skip(1) {
            let (head, tail) = matrix.split_at_mut(row);
            let (Some(upper), Some(lower)) = (head.get_mut(column), tail.first_mut()) else {
                continue;
            };
            let a = upper.get(column).copied().unwrap_or_default();
            let b = lower.get(column).copied().unwrap_or_default();
            let radius = a.norm().hypot(b.norm());
            if radius <= 0.0 || b.norm_sqr() <= 0.0 {
                continue;
            }
            let cosine = a.scale(1.0 / radius);
            let sine = b.scale(1.0 / radius);
            for (x, y) in upper.iter_mut().zip(lower.iter_mut()).take(active) {
                let (u, w) = (*x, *y);
                *x = cosine.conj().times(u).plus(sine.conj().times(w));
                *y = cosine.times(w).minus(sine.times(u));
            }
            rotations.push(Rotation {
                column,
                row,
                cosine,
                sine,
            });
        }
    }
    for rotation in rotations.iter() {
        for line in matrix.iter_mut().take(active) {
            let u = line.get(rotation.column).copied().unwrap_or_default();
            let w = line.get(rotation.row).copied().unwrap_or_default();
            if let Some(x) = line.get_mut(rotation.column) {
                *x = rotation.cosine.times(u).plus(rotation.sine.times(w));
            }
            if let Some(y) = line.get_mut(rotation.row) {
                *y = rotation
                    .cosine
                    .conj()
                    .times(w)
                    .minus(rotation.sine.conj().times(u));
            }
        }
    }
    shift_diagonal(matrix, active, shift);
}

/// Characteristic exponent of the log price in `f64`.
#[derive(Debug, Clone, Copy)]
pub(super) struct ModelKernel {
    /// Linear coefficient of the exponent: the drift of the log price net
    /// of the convexity of the diffusion and of the jumps.
    pub(super) location: f64,
    /// Variance of the diffusion, per year.
    pub(super) variance: f64,
    /// Jumps per year.
    pub(super) intensity: f64,
    /// Mean of the logarithm of a jump.
    pub(super) jump_mean: f64,
    /// Variance of the logarithm of a jump.
    pub(super) jump_variance: f64,
}

impl ModelKernel {
    pub(super) fn new(model: &LevyModel, drift: f64) -> Result<Self, PricingError> {
        let (sigma, intensity, jump_mean, jump_sd) = match model {
            LevyModel::BlackScholes { volatility } => {
                (decimal_to_f64(volatility.to_dec())?, 0.0, 0.0, 0.0)
            }
            LevyModel::Merton {
                volatility,
                jump_intensity,
                jump_mean,
                jump_volatility,
            } => (
                decimal_to_f64(volatility.to_dec())?,
                decimal_to_f64(jump_intensity.to_dec())?,
                decimal_to_f64(*jump_mean)?,
                decimal_to_f64(jump_volatility.to_dec())?,
            ),
        };
        let variance = sigma * sigma;
        let jump_variance = jump_sd * jump_sd;
        let mean_jump = exp_f64(jump_mean + 0.5 * jump_variance) - 1.0;
        Ok(Self {
            location: drift - 0.5 * variance - intensity * mean_jump,
            variance,
            intensity,
            jump_mean,
            jump_variance,
        })
    }

    /// `χ(u)` for complex `u`: `E[exp(iuX_s)] = exp(s·χ(u))`.
    fn exponent(&self, u: Complex) -> Complex {
        let square = u.times(u);
        let diffusion = Complex::new(-u.im * self.location, u.re * self.location)
            .minus(square.scale(0.5 * self.variance));
        if self.intensity > 0.0 {
            let jump = Complex::new(-u.im * self.jump_mean, u.re * self.jump_mean)
                .minus(square.scale(0.5 * self.jump_variance))
                .exponential();
            diffusion.plus(jump.minus(Complex::real(1.0)).scale(self.intensity))
        } else {
            diffusion
        }
    }

    /// `χ(−ip)` for real `p`: the growth rate of `E[S_s^p]`.
    fn cumulant(&self, p: f64) -> f64 {
        let diffusion = p * self.location + 0.5 * self.variance * p * p;
        if self.intensity > 0.0 {
            diffusion
                + self.intensity
                    * (exp_f64(p * self.jump_mean + 0.5 * self.jump_variance * p * p) - 1.0)
        } else {
            diffusion
        }
    }
}

/// What the inversion returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Quantity {
    Price,
    Delta,
}

/// The integrand at one frequency: its real part and its modulus, per
/// contract.
#[derive(Clone, Copy)]
struct Sample {
    real: Values,
    modulus: Values,
}

/// Gauss-Legendre rule on `[−1, 1]`.
struct Rule {
    nodes: [f64; NODES],
    weights: [f64; NODES],
}

impl Rule {
    fn new() -> Self {
        let mut nodes = [0.0; NODES];
        let mut weights = [0.0; NODES];
        let mut position = 1.0;
        for (node, weight) in nodes.iter_mut().zip(weights.iter_mut()) {
            let mut x = cos_f64(std::f64::consts::PI * (position - 0.25) / (NODES_F64 + 0.5));
            for _ in 0..50 {
                let (value, derivative) = legendre(x);
                let step = value / derivative;
                x -= step;
                if step.abs() <= f64::EPSILON {
                    break;
                }
            }
            let (_, derivative) = legendre(x);
            *node = x;
            *weight = 2.0 / ((1.0 - x * x) * derivative * derivative);
            position += 1.0;
        }
        Self { nodes, weights }
    }
}

/// The Legendre polynomial of degree `NODES` and its derivative at `x`,
/// with `|x| < 1`.
fn legendre(x: f64) -> (f64, f64) {
    let mut previous = 1.0;
    let mut current = x;
    let mut degree = 1.0;
    while degree < NODES_F64 {
        let next = ((2.0 * degree + 1.0) * x * current - degree * previous) / (degree + 1.0);
        previous = current;
        current = next;
        degree += 1.0;
    }
    (
        current,
        NODES_F64 * (x * current - previous) / (x * x - 1.0),
    )
}

/// Fourier inversion along `Im u = −damping` for every contract of a
/// network.
struct Inversion<'a> {
    network: &'a FundingNetwork,
    kernel: ModelKernel,
    log_moneyness: f64,
    damping: f64,
    /// Years between two funding payments; `None` for continuous funding.
    interval: Option<f64>,
    quantity: Quantity,
    workspace: Workspace,
}

/// Value of `−Δ·Re χ` beyond which the transform of a discrete schedule,
/// of the order of `e^{Δ·Re χ}`, is taken as zero: `e^{−100}` is far below
/// the tolerance of the inversion, and `e^{−Δ·χ}` stays well inside the
/// range of an `f64` for the linear solve.
const DISCRETE_CUTOFF: f64 = 100.0;

/// `(1 − e^{−Δ·χ})/Δ`: the exponent that takes the place of `χ` when
/// funding is paid every `Δ` years. Computed without the cancellation of
/// `1 − e^{−z}` for small `z`.
fn discrete_exponent(exponent: Complex, interval: f64) -> Complex {
    let growth = (-interval * exponent.re).exp_m1();
    let (half_sine, half_cosine) = (0.5 * interval * exponent.im).sin_cos();
    let sine = 2.0 * half_sine * half_cosine;
    let versine = 2.0 * half_sine * half_sine;
    // `e^{−z} = (1 + growth)·(cos b − i·sin b)` with `b = Δ·Im χ`.
    Complex::new(
        (versine - growth * (1.0 - versine)) / interval,
        (growth + 1.0) * sine / interval,
    )
}

impl Inversion<'_> {
    /// The integrand at frequency `v`, or `None` when the linear system is
    /// singular or the value is not finite.
    fn sample(&mut self, v: f64) -> Option<Sample> {
        let mut exponent = self.kernel.exponent(Complex::new(v, -self.damping));
        if !exponent.re.is_finite() || !exponent.im.is_finite() {
            return None;
        }
        if let Some(interval) = self.interval {
            if -interval * exponent.re > DISCRETE_CUTOFF {
                return Some(Sample {
                    real: [0.0; FUNDING_NETWORK_MAX_ORDER],
                    modulus: [0.0; FUNDING_NETWORK_MAX_ORDER],
                });
            }
            exponent = discrete_exponent(exponent, interval);
        }
        self.workspace
            .load(&self.network.values, exponent, &self.network.anchor);
        self.workspace.solve()?;
        // `iu = p + iv` on the contour.
        let below_one = Complex::new(self.damping - 1.0, v);
        let denominator = match self.quantity {
            Quantity::Price => Complex::new(self.damping, v).times(below_one),
            Quantity::Delta => below_one,
        };
        let (sine, cosine) = (v * self.log_moneyness).sin_cos();
        let factor = Complex::new(cosine, sine).over(denominator);
        let mut sample = Sample {
            real: [0.0; FUNDING_NETWORK_MAX_ORDER],
            modulus: [0.0; FUNDING_NETWORK_MAX_ORDER],
        };
        for ((real, modulus), solution) in sample
            .real
            .iter_mut()
            .zip(sample.modulus.iter_mut())
            .zip(self.workspace.rhs.iter())
        {
            let value = solution.times(factor);
            if !value.re.is_finite() || !value.im.is_finite() {
                return None;
            }
            *real = value.re;
            *modulus = value.norm();
        }
        Some(sample)
    }

    /// The rule applied to `[low, high]`.
    fn panel(&mut self, rule: &Rule, low: f64, high: f64) -> Option<Values> {
        let half_width = 0.5 * (high - low);
        let middle = 0.5 * (high + low);
        let mut total = [0.0; FUNDING_NETWORK_MAX_ORDER];
        for (node, weight) in rule.nodes.iter().zip(rule.weights.iter()) {
            let sample = self.sample(middle + half_width * node)?;
            for (target, value) in total.iter_mut().zip(sample.real.iter()) {
                *target += weight * half_width * value;
            }
        }
        Some(total)
    }

    /// Refines `whole`, the rule on `[low, high]`, by bisection until both
    /// halves agree with it to `tolerance`. `None` when `depth` bisections
    /// are not enough.
    fn refine(
        &mut self,
        rule: &Rule,
        low: f64,
        high: f64,
        whole: &Values,
        tolerance: f64,
        depth: u32,
    ) -> Option<Values> {
        let middle = 0.5 * (low + high);
        let left = self.panel(rule, low, middle)?;
        let right = self.panel(rule, middle, high)?;
        let mut refined = [0.0; FUNDING_NETWORK_MAX_ORDER];
        let mut error: f64 = 0.0;
        for (((target, a), b), coarse) in refined
            .iter_mut()
            .zip(left.iter())
            .zip(right.iter())
            .zip(whole.iter())
        {
            *target = a + b;
            error = error.max((a + b - coarse).abs());
        }
        if error <= tolerance {
            return Some(refined);
        }
        let deeper = depth.checked_sub(1)?;
        let left = self.refine(rule, low, middle, &left, 0.5 * tolerance, deeper)?;
        let right = self.refine(rule, middle, high, &right, 0.5 * tolerance, deeper)?;
        for ((target, a), b) in refined.iter_mut().zip(left.iter()).zip(right.iter()) {
            *target = a + b;
        }
        Some(refined)
    }

    /// The integral over `[0, ∞)`.
    fn integrate(&mut self) -> Result<Values, PricingError> {
        let failed = || {
            PricingError::method_error(
                METHOD,
                "the Fourier integrand could not be integrated to tolerance",
            )
        };
        let rule = Rule::new();
        // Panels start at the distance from the contour to the poles of the
        // payoff transform and double until each holds two cycles of
        // `exp(iv·ln(S/K))`.
        let frequency = self.log_moneyness.abs();
        let widest = if frequency > 0.0 {
            4.0 * std::f64::consts::PI / frequency
        } else {
            f64::INFINITY
        };
        let mut width = self
            .damping
            .abs()
            .min((self.damping - 1.0).abs())
            .min(widest);
        let mut low = 0.0;
        let mut total = [0.0; FUNDING_NETWORK_MAX_ORDER];
        let mut quiet = false;
        for _ in 0..MAX_PANELS {
            let high = low + width;
            let whole = self.panel(&rule, low, high).ok_or_else(failed)?;
            let part = self
                .refine(&rule, low, high, &whole, PANEL_TOLERANCE, MAX_DEPTH)
                .ok_or_else(failed)?;
            for (target, value) in total.iter_mut().zip(part.iter()) {
                *target += value;
            }
            // Past `high` the modulus decays and the phase turns at the rate
            // `frequency`, so the tail is at most the modulus times the
            // shorter of `high` and the area of one half cycle.
            let edge = self.sample(high).ok_or_else(failed)?;
            let reach = if frequency > 0.0 {
                high.min(2.0 / frequency)
            } else {
                high
            };
            let bound = edge.modulus.iter().fold(0.0_f64, |a, b| a.max(*b)) * reach;
            if bound <= TAIL_TOLERANCE {
                if quiet {
                    return Ok(total);
                }
                quiet = true;
            } else {
                quiet = false;
            }
            low = high;
            width = (2.0 * width).min(widest);
        }
        Err(PricingError::method_error(
            METHOD,
            "the Fourier inversion did not converge",
        ))
    }
}

/// Order `p` of the exponential moment along which the inversion runs:
/// above one for the call, below zero for the put, as far from the poles of
/// the payoff transform as the decay rate of the network allows.
fn damping(kernel: &ModelKernel, decay_rate: f64, side: OptionStyle) -> Result<f64, PricingError> {
    let (pole, direction) = match side {
        OptionStyle::Call => (1.0, 1.0),
        OptionStyle::Put => (0.0, -1.0),
    };
    let floor = kernel.cumulant(pole);
    let ceiling = floor + 0.5 * (decay_rate - floor);
    let mut offset = 0.75;
    for _ in 0..40 {
        let order = pole + direction * offset;
        if kernel.cumulant(order) <= ceiling {
            return Ok(order);
        }
        offset *= 0.5;
    }
    Err(PricingError::method_error(
        METHOD,
        "no damping keeps the exponential moment inside the decay rate of the network",
    ))
}

/// Prices or deltas of every contract, in `f64`.
fn network_values(
    network: &FundingNetwork,
    params: &FundingNetworkParams,
    schedule: FundingSchedule,
    quantity: Quantity,
) -> Result<Values, PricingError> {
    let spot = decimal_to_f64(params.underlying_price.to_dec())?;
    let strike = decimal_to_f64(params.strike_price.to_dec())?;
    let drift = decimal_to_f64(params.drift)?;
    let decay_rate = network.scheduled_decay(schedule)?;
    if drift >= decay_rate {
        return Err(PricingError::invalid_parameter(
            "drift",
            params.drift,
            "must be below the decay rate of the network",
        ));
    }
    let interval = match schedule {
        FundingSchedule::Continuous => None,
        FundingSchedule::Discrete { interval } => Some(decimal_to_f64(interval.to_dec())?),
    };
    let kernel = ModelKernel::new(&params.model, drift)?;
    let log_moneyness = ln_f64(spot / strike);
    // The side that is out of the money is inverted directly; the factor
    // `exp(p·ln(S/K))` is then at most one.
    let direct_side = if log_moneyness <= 0.0 {
        OptionStyle::Call
    } else {
        OptionStyle::Put
    };
    let order = damping(&kernel, decay_rate, direct_side)?;
    let integral = Inversion {
        network,
        kernel,
        log_moneyness,
        damping: order,
        interval,
        quantity,
        workspace: Workspace::new(network.order()),
    }
    .integrate()?;
    let scale = match quantity {
        Quantity::Price => strike * exp_f64(order * log_moneyness) / std::f64::consts::PI,
        Quantity::Delta => exp_f64((order - 1.0) * log_moneyness) / std::f64::consts::PI,
    };
    let mut values = [0.0; FUNDING_NETWORK_MAX_ORDER];
    if params.option_style == direct_side {
        for (target, value) in values.iter_mut().zip(integral.iter()) {
            *target = scale * value;
        }
        return Ok(values);
    }
    // Parity: `C_i − P_i = S·E[S_τ]/S − K`, with the forward factor
    // `Ξ_i(−i) = e_iᵀ·(A − μI)⁻¹·A·1`; a payment interval `Δ` turns `μ`
    // into `(1 − e^{−Δμ})/Δ`.
    let carry = match interval {
        None => drift,
        Some(interval) => -(-interval * drift).exp_m1() / interval,
    };
    let forward = network.solve_real(carry, &network.anchor)?;
    for ((target, value), factor) in values.iter_mut().zip(integral.iter()).zip(forward.iter()) {
        let direct = scale * value;
        *target = match (quantity, params.option_style) {
            (Quantity::Price, OptionStyle::Call) => direct + spot * factor - strike,
            (Quantity::Price, OptionStyle::Put) => direct - spot * factor + strike,
            (Quantity::Delta, OptionStyle::Call) => direct + factor,
            (Quantity::Delta, OptionStyle::Put) => direct - factor,
        };
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pricing::everlasting::{
        EverlastingParams, everlasting_delta, everlasting_funding, everlasting_price,
    };
    use optionstratlib_core::pos_or_panic;
    use rust_decimal::prelude::ToPrimitive;
    use rust_decimal_macros::dec;

    const SIGMA: f64 = 0.5;
    const RATE: f64 = 36.5;

    /// The signed network of order three whose top contract has a squared
    /// coefficient of variation of 0.2009, at a mean maturity of 30 days.
    const SIGNED: [[f64; 3]; 3] = [
        [27.461_221_363_433_275, 0.0, 0.0],
        [
            -21.681_869_488_719_47,
            27.461_221_363_433_275,
            8.229_101_161_600_752,
        ],
        [
            74.954_173_695_544_97,
            -98.344_616_472_620_29,
            27.461_221_363_433_28,
        ],
    ];

    fn decimal(value: f64) -> Decimal {
        Decimal::try_from(value).unwrap()
    }

    fn network(rows: &[&[f64]]) -> FundingNetwork {
        let rows: Vec<Vec<Decimal>> = rows
            .iter()
            .map(|row| row.iter().map(|value| decimal(*value)).collect())
            .collect();
        FundingNetwork::try_new(&rows).unwrap()
    }

    fn signed() -> FundingNetwork {
        network(&[&SIGNED[0], &SIGNED[1], &SIGNED[2]])
    }

    fn ladder(rate: f64, rungs: u32) -> FundingNetwork {
        FundingNetwork::ladder(pos_or_panic!(rate), NonZeroU32::new(rungs).unwrap()).unwrap()
    }

    fn black_scholes(sigma: f64) -> LevyModel {
        LevyModel::BlackScholes {
            volatility: pos_or_panic!(sigma),
        }
    }

    fn merton() -> LevyModel {
        LevyModel::Merton {
            volatility: pos_or_panic!(SIGMA),
            jump_intensity: pos_or_panic!(20.0),
            jump_mean: dec!(-0.05),
            jump_volatility: pos_or_panic!(0.10),
        }
    }

    fn params(
        spot: f64,
        strike: f64,
        mu: f64,
        model: LevyModel,
        style: OptionStyle,
    ) -> FundingNetworkParams {
        FundingNetworkParams::try_new(
            pos_or_panic!(spot),
            pos_or_panic!(strike),
            decimal(mu),
            model,
            style,
        )
        .unwrap()
    }

    fn floats(values: &[Decimal]) -> Vec<f64> {
        values.iter().map(|value| value.to_f64().unwrap()).collect()
    }

    fn closed_form(
        spot: f64,
        strike: f64,
        mu: f64,
        sigma: f64,
        rate: f64,
        rung: u32,
        style: OptionStyle,
    ) -> EverlastingParams {
        EverlastingParams::try_new(
            pos_or_panic!(spot),
            pos_or_panic!(strike),
            pos_or_panic!(sigma),
            decimal(mu),
            pos_or_panic!(rate),
            NonZeroU32::new(rung).unwrap(),
            style,
        )
        .unwrap()
    }

    fn relative(actual: f64, expected: f64) -> f64 {
        ((actual - expected) / expected).abs()
    }

    fn sorted(mut spectrum: Vec<Complex>) -> Vec<Complex> {
        spectrum.sort_by(|a, b| a.re.total_cmp(&b.re).then(a.im.total_cmp(&b.im)));
        spectrum
    }

    #[test]
    fn test_complex_operations_match_known_values() {
        let a = Complex::new(1.0, 2.0);
        let b = Complex::new(3.0, -4.0);
        assert_eq!(a.plus(b), Complex::new(4.0, -2.0));
        assert_eq!(a.minus(b), Complex::new(-2.0, 6.0));
        assert_eq!(a.times(b), Complex::new(11.0, 2.0));
        let quotient = a.over(b);
        assert!((quotient.re + 0.2).abs() < 1e-15);
        assert!((quotient.im - 0.4).abs() < 1e-15);
        // The other branch of the division: a divisor with the larger
        // imaginary part.
        let turned = a.over(Complex::new(1.0, 5.0));
        assert!((turned.times(Complex::new(1.0, 5.0)).minus(a)).norm() < 1e-15);
        let root = Complex::new(-3.0, -4.0).square_root();
        assert!((root.re - 1.0).abs() < 1e-15);
        assert!((root.im + 2.0).abs() < 1e-15);
        let turn = Complex::new(0.0, std::f64::consts::PI).exponential();
        assert!((turn.re + 1.0).abs() < 1e-15);
        assert!(turn.im.abs() < 1e-15);
        assert!((b.norm() - 5.0).abs() < 1e-15);
        assert!((b.norm_sqr() - 25.0).abs() < 1e-13);
        assert_eq!(b.conj(), Complex::new(3.0, 4.0));
    }

    #[test]
    fn test_rule_gauss_legendre_integrates_polynomials_exactly() {
        let rule = Rule::new();
        let weight: f64 = rule.weights.iter().sum();
        assert!((weight - 2.0).abs() < 1e-14);
        // Exact up to degree 31: even powers integrate to 2/(k + 1).
        for power in [2_i32, 8, 20, 30] {
            let integral: f64 = rule
                .nodes
                .iter()
                .zip(rule.weights.iter())
                .map(|(node, weight)| weight * node.powi(power))
                .sum();
            assert!((integral - 2.0 / f64::from(power + 1)).abs() < 1e-14);
        }
        let odd: f64 = rule
            .nodes
            .iter()
            .zip(rule.weights.iter())
            .map(|(node, weight)| weight * node.powi(7))
            .sum();
        assert!(odd.abs() < 1e-15);
    }

    #[test]
    fn test_eigenvalues_triangular_matrix_returns_diagonal_exactly() {
        let ladder = ladder(RATE, 16);
        let spectrum = eigenvalues(&ladder.values).unwrap();
        assert_eq!(spectrum.len(), 16);
        assert!(spectrum.iter().all(|e| e.re == RATE && e.im == 0.0));
    }

    #[test]
    fn test_eigenvalues_signed_network_match_reference() {
        let spectrum = sorted(eigenvalues(&signed().values).unwrap());
        let expected = [
            (27.461_221_363_433_275, -28.447_984_070_088_694),
            (27.461_221_363_433_275, 0.0),
            (27.461_221_363_433_275, 28.447_984_070_088_694),
        ];
        let mut expected: Vec<Complex> = expected
            .iter()
            .map(|(re, im)| Complex::new(*re, *im))
            .collect();
        expected = sorted(expected);
        // The three real parts coincide, so order by the imaginary part.
        let mut spectrum = spectrum;
        spectrum.sort_by(|a, b| a.im.total_cmp(&b.im));
        expected.sort_by(|a, b| a.im.total_cmp(&b.im));
        for (actual, reference) in spectrum.iter().zip(expected.iter()) {
            assert!(actual.minus(*reference).norm() < 1e-10, "{actual:?}");
        }
    }

    #[test]
    fn test_eigenvalues_companion_matrix_returns_the_roots() {
        // Companion matrix of (x − 1)(x − 2)(x − 3)(x − 4).
        let values = vec![
            vec![10.0, -35.0, 50.0, -24.0],
            vec![1.0, 0.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0, 0.0],
            vec![0.0, 0.0, 1.0, 0.0],
        ];
        let spectrum = sorted(eigenvalues(&values).unwrap());
        for (actual, root) in spectrum.iter().zip([1.0, 2.0, 3.0, 4.0]) {
            assert!((actual.re - root).abs() < 1e-10, "{actual:?}");
            assert!(actual.im.abs() < 1e-10, "{actual:?}");
        }
    }

    #[test]
    fn test_eigenvalues_rotation_block_returns_a_conjugate_pair() {
        let values = vec![vec![2.0, -5.0], vec![5.0, 2.0]];
        let mut spectrum = eigenvalues(&values).unwrap();
        spectrum.sort_by(|a, b| a.im.total_cmp(&b.im));
        assert!(spectrum[0].minus(Complex::new(2.0, -5.0)).norm() < 1e-12);
        assert!(spectrum[1].minus(Complex::new(2.0, 5.0)).norm() < 1e-12);
    }

    #[test]
    fn test_eigenvalues_dense_matrix_preserve_the_traces() {
        // A fixed dense matrix of order 12 without structure.
        let order = 12;
        let values: Vec<Vec<f64>> = (0..order)
            .map(|i| {
                (0..order)
                    .map(|j| {
                        let seed = (i * 31 + j * 17 + 7) as f64;
                        (seed * 0.618_033_988_749_895).fract() * 4.0 - 2.0
                            + if i == j { 3.0 } else { 0.0 }
                    })
                    .collect()
            })
            .collect();
        let spectrum = eigenvalues(&values).unwrap();
        assert_eq!(spectrum.len(), order);
        let trace: f64 = (0..order).map(|i| values[i][i]).sum();
        let square_trace: f64 = (0..order)
            .flat_map(|i| (0..order).map(move |j| (i, j)))
            .map(|(i, j)| values[i][j] * values[j][i])
            .sum();
        let sum = spectrum
            .iter()
            .fold(Complex::default(), |acc, e| acc.plus(*e));
        let sum_of_squares = spectrum
            .iter()
            .fold(Complex::default(), |acc, e| acc.plus(e.times(*e)));
        assert!((sum.re - trace).abs() < 1e-10);
        assert!(sum.im.abs() < 1e-10);
        assert!((sum_of_squares.re - square_trace).abs() < 1e-9);
        assert!(sum_of_squares.im.abs() < 1e-9);
    }

    #[test]
    fn test_funding_network_try_new_rejects_matrices_outside_the_domain() {
        assert!(matches!(
            FundingNetwork::try_new(&[]),
            Err(PricingError::InvalidParameter {
                parameter: "order",
                ..
            })
        ));
        let large = vec![vec![Decimal::ONE; 17]; 17];
        assert!(matches!(
            FundingNetwork::try_new(&large),
            Err(PricingError::InvalidParameter {
                parameter: "order",
                ..
            })
        ));
        let ragged = vec![vec![dec!(1), dec!(0)], vec![dec!(1)]];
        assert!(matches!(
            FundingNetwork::try_new(&ragged),
            Err(PricingError::InvalidParameter {
                parameter: "row_length",
                ..
            })
        ));
        // A negative eigenvalue, a zero one, and a pair on the left.
        for rows in [
            vec![vec![dec!(1), dec!(0)], vec![dec!(0), dec!(-1)]],
            vec![vec![dec!(1), dec!(1)], vec![dec!(1), dec!(1)]],
            vec![vec![dec!(-1), dec!(-5)], vec![dec!(5), dec!(-1)]],
        ] {
            assert!(matches!(
                FundingNetwork::try_new(&rows),
                Err(PricingError::InvalidParameter {
                    parameter: "funding_matrix",
                    ..
                })
            ));
        }
        assert!(matches!(
            FundingNetwork::ladder(Positive::ZERO, NonZeroU32::MIN),
            Err(PricingError::InvalidParameter {
                parameter: "funding_rate",
                ..
            })
        ));
        assert!(matches!(
            FundingNetwork::ladder(Positive::ONE, NonZeroU32::new(17).unwrap()),
            Err(PricingError::InvalidParameter {
                parameter: "rungs",
                ..
            })
        ));
    }

    #[test]
    fn test_funding_network_ladder_has_erlang_moments() {
        let ladder = ladder(RATE, 8);
        assert_eq!(ladder.order(), 8);
        assert!(ladder.is_non_negative());
        assert_eq!(ladder.decay_rate().unwrap(), decimal(RATE));
        assert_eq!(ladder.entry(0, 0), Some(decimal(RATE)));
        assert_eq!(ladder.entry(3, 2), Some(decimal(-RATE)));
        assert_eq!(ladder.entry(3, 1), Some(Decimal::ZERO));
        assert_eq!(ladder.entry(8, 0), None);
        for rung in 1..=8_u32 {
            let contract = (rung - 1) as usize;
            let j = f64::from(rung);
            let mean = ladder.mean_maturity(contract).unwrap().to_f64().unwrap();
            let variance = ladder
                .maturity_variance(contract)
                .unwrap()
                .to_f64()
                .unwrap();
            assert!(relative(mean, j / RATE) < 1e-13);
            assert!(relative(variance, j / (RATE * RATE)) < 1e-12);
        }
        assert!(matches!(
            ladder.mean_maturity(8),
            Err(PricingError::InvalidParameter {
                parameter: "contract",
                ..
            })
        ));
    }

    #[test]
    fn test_funding_network_signed_moments_match_reference() {
        let signed = signed();
        assert!(!signed.is_non_negative());
        let decay = signed.decay_rate().unwrap().to_f64().unwrap();
        assert!(relative(decay, 27.461_221_363_433_275) < 1e-10);
        let means = [
            0.036_414_986_309_806_92,
            0.040_536_452_724_908_55,
            0.082_191_780_821_917_8,
        ];
        let squared_cvs = [1.0, 0.591_498_658_233_176_3, 0.200_901_563_500_556_88];
        for (contract, (mean, squared_cv)) in means.iter().zip(squared_cvs.iter()).enumerate() {
            let actual = signed.mean_maturity(contract).unwrap().to_f64().unwrap();
            let variance = signed
                .maturity_variance(contract)
                .unwrap()
                .to_f64()
                .unwrap();
            assert!(relative(actual, *mean) < 1e-12);
            assert!(relative(variance / (actual * actual), *squared_cv) < 1e-10);
        }
    }

    #[test]
    fn test_funding_network_prices_ladder_match_closed_form() {
        let mut worst: f64 = 0.0;
        for mu in [0.0, 0.05, -0.2] {
            for rungs in [1_u32, 2, 4] {
                let ladder = ladder(RATE, rungs);
                for spot in [80.0, 95.0, 100.0, 105.0, 125.0] {
                    for style in [OptionStyle::Call, OptionStyle::Put] {
                        let inputs = params(spot, 100.0, mu, black_scholes(SIGMA), style);
                        let prices = floats(&funding_network_prices(&ladder, &inputs).unwrap());
                        for (index, price) in prices.iter().enumerate() {
                            let rung = index as u32 + 1;
                            let reference = everlasting_price(&closed_form(
                                spot, 100.0, mu, SIGMA, RATE, rung, style,
                            ))
                            .unwrap()
                            .to_f64()
                            .unwrap();
                            worst = worst.max(relative(*price, reference));
                        }
                    }
                }
            }
        }
        assert!(worst < 1e-9, "worst relative error {worst:e}");
    }

    #[test]
    fn test_funding_network_prices_first_rung_at_the_money_matches_closed_form() {
        let ladder = ladder(12.0, 1);
        let inputs = params(100.0, 100.0, 0.0, black_scholes(SIGMA), OptionStyle::Call);
        let price = floats(&funding_network_prices(&ladder, &inputs).unwrap())[0];
        let expected = 100.0 / (1.0 + 8.0 * 12.0 / (SIGMA * SIGMA)).sqrt();
        assert!(relative(price, expected) < 1e-11);
    }

    #[test]
    fn test_funding_network_prices_signed_black_scholes_match_reference() {
        // Integration of Black-Scholes values over each maturity density.
        let references = [
            (
                0.0,
                90.0,
                [
                    0.670_407_739_198_389_9,
                    0.762_189_961_995_568_4,
                    1.791_144_597_066_698_2,
                ],
            ),
            (
                0.0,
                100.0,
                [
                    3.371_460_843_968_284_5,
                    3.704_037_403_488_483,
                    5.555_905_005_567_598,
                ],
            ),
            (
                0.0,
                110.0,
                [
                    10.860_291_775_262_22,
                    10.982_975_088_101_886,
                    12.219_899_157_145_14,
                ],
            ),
            (
                0.05,
                90.0,
                [
                    0.704_730_598_593_989_9,
                    0.799_259_420_901_915_5,
                    1.891_386_848_412_142_3,
                ],
            ),
            (
                0.05,
                100.0,
                [
                    3.468_197_773_726_69,
                    3.811_445_469_269_365,
                    5.777_058_196_694_344,
                ],
            ),
            (
                0.05,
                110.0,
                [
                    11.023_671_450_755_75,
                    11.165_324_037_011_9,
                    12.567_951_267_653_646,
                ],
            ),
        ];
        let signed = signed();
        let mut worst: f64 = 0.0;
        for (mu, spot, expected) in references {
            let inputs = params(spot, 100.0, mu, black_scholes(SIGMA), OptionStyle::Call);
            let prices = floats(&funding_network_prices(&signed, &inputs).unwrap());
            for (price, reference) in prices.iter().zip(expected.iter()) {
                worst = worst.max(relative(*price, *reference));
            }
        }
        assert!(worst < 1e-9, "worst relative error {worst:e}");
    }

    #[test]
    fn test_funding_network_prices_merton_match_reference() {
        // Integration of the series prices of the jump diffusion over each
        // maturity density, with zero drift.
        let ladder_references = [
            (
                90.0,
                [
                    0.924_985_299_658_515_2,
                    2.065_373_705_540_603_4,
                    3.178_179_260_722_301_6,
                ],
            ),
            (
                100.0,
                [
                    3.879_124_817_209_081_8,
                    5.947_335_700_474_770_4,
                    7.498_393_208_778_743,
                ],
            ),
            (
                110.0,
                [
                    11.467_036_717_749_735,
                    12.882_708_627_636_779,
                    14.158_976_971_035_194,
                ],
            ),
        ];
        let signed_references = [
            (
                90.0,
                [
                    1.295_867_538_285_620_8,
                    1.482_744_273_653_302,
                    3.241_249_911_346_013,
                ],
            ),
            (
                100.0,
                [
                    4.511_352_909_481_110_5,
                    4.982_951_549_958_407,
                    7.608_938_458_378_519,
                ],
            ),
            (
                110.0,
                [
                    11.918_381_004_612_14,
                    12.180_610_494_329_471,
                    14.234_858_594_090_53,
                ],
            ),
        ];
        let mut worst: f64 = 0.0;
        for (net, references) in [
            (ladder(RATE, 3), ladder_references),
            (signed(), signed_references),
        ] {
            for (spot, expected) in references {
                let inputs = params(spot, 100.0, 0.0, merton(), OptionStyle::Call);
                let prices = floats(&funding_network_prices(&net, &inputs).unwrap());
                for (price, reference) in prices.iter().zip(expected.iter()) {
                    worst = worst.max(relative(*price, *reference));
                }
            }
        }
        assert!(worst < 1e-9, "worst relative error {worst:e}");
    }

    #[test]
    fn test_funding_network_prices_merton_without_jumps_equal_black_scholes() {
        let quiet = LevyModel::Merton {
            volatility: pos_or_panic!(SIGMA),
            jump_intensity: Positive::ZERO,
            jump_mean: dec!(-0.05),
            jump_volatility: pos_or_panic!(0.10),
        };
        let signed = signed();
        let with =
            funding_network_prices(&signed, &params(95.0, 100.0, 0.03, quiet, OptionStyle::Put))
                .unwrap();
        let without = funding_network_prices(
            &signed,
            &params(95.0, 100.0, 0.03, black_scholes(SIGMA), OptionStyle::Put),
        )
        .unwrap();
        assert_eq!(with, without);
    }

    #[test]
    fn test_funding_network_prices_parity_gives_the_forward_of_each_rung() {
        let (mu, spot, strike) = (0.08, 104.0, 100.0);
        let ladder = ladder(RATE, 4);
        let calls = floats(
            &funding_network_prices(
                &ladder,
                &params(spot, strike, mu, merton(), OptionStyle::Call),
            )
            .unwrap(),
        );
        let puts = floats(
            &funding_network_prices(
                &ladder,
                &params(spot, strike, mu, merton(), OptionStyle::Put),
            )
            .unwrap(),
        );
        for (index, (call, put)) in calls.iter().zip(puts.iter()).enumerate() {
            let forward = spot * (RATE / (RATE - mu)).powi(index as i32 + 1);
            assert!(((call - put) - (forward - strike)).abs() < 1e-10);
        }
    }

    #[test]
    fn test_funding_network_deltas_ladder_match_closed_form() {
        let ladder = ladder(RATE, 3);
        let mut worst: f64 = 0.0;
        for mu in [0.0, 0.05] {
            for spot in [90.0, 100.0, 108.0] {
                for style in [OptionStyle::Call, OptionStyle::Put] {
                    let inputs = params(spot, 100.0, mu, black_scholes(SIGMA), style);
                    let deltas = floats(&funding_network_deltas(&ladder, &inputs).unwrap());
                    for (index, delta) in deltas.iter().enumerate() {
                        let reference = everlasting_delta(&closed_form(
                            spot,
                            100.0,
                            mu,
                            SIGMA,
                            RATE,
                            index as u32 + 1,
                            style,
                        ))
                        .unwrap()
                        .to_f64()
                        .unwrap();
                        worst = worst.max((delta - reference).abs());
                    }
                }
            }
        }
        assert!(worst < 1e-8, "worst absolute error {worst:e}");
    }

    #[test]
    fn test_funding_network_deltas_signed_merton_match_finite_differences() {
        let signed = signed();
        let step = 1e-3;
        for spot in [93.0, 107.0] {
            for style in [OptionStyle::Call, OptionStyle::Put] {
                let price = |s: f64| {
                    floats(
                        &funding_network_prices(&signed, &params(s, 100.0, 0.02, merton(), style))
                            .unwrap(),
                    )
                };
                let (up, down) = (price(spot + step), price(spot - step));
                let deltas = floats(
                    &funding_network_deltas(&signed, &params(spot, 100.0, 0.02, merton(), style))
                        .unwrap(),
                );
                for ((delta, high), low) in deltas.iter().zip(up.iter()).zip(down.iter()) {
                    assert!((delta - (high - low) / (2.0 * step)).abs() < 1e-6);
                }
            }
        }
    }

    #[test]
    fn test_funding_network_funding_ladder_matches_closed_form() {
        let ladder = ladder(RATE, 3);
        for (spot, style) in [(96.0, OptionStyle::Call), (104.0, OptionStyle::Put)] {
            let inputs = params(spot, 100.0, 0.04, black_scholes(SIGMA), style);
            let funding = floats(&funding_network_funding(&ladder, &inputs).unwrap());
            for (index, value) in funding.iter().enumerate() {
                let reference = everlasting_funding(&closed_form(
                    spot,
                    100.0,
                    0.04,
                    SIGMA,
                    RATE,
                    index as u32 + 1,
                    style,
                ))
                .unwrap()
                .to_f64()
                .unwrap();
                assert!((value - reference).abs() < 1e-7, "{value} {reference}");
            }
        }
    }

    #[test]
    fn test_funding_network_funding_in_the_money_uses_the_payoff() {
        // In the money the first rung is funded against a positive payoff.
        let ladder = ladder(RATE, 2);
        let inputs = params(110.0, 100.0, 0.0, merton(), OptionStyle::Call);
        let prices = floats(&funding_network_prices(&ladder, &inputs).unwrap());
        let funding = floats(&funding_network_funding(&ladder, &inputs).unwrap());
        assert!((funding[0] - RATE * (prices[0] - 10.0)).abs() < 1e-9);
        assert!((funding[1] - RATE * (prices[1] - prices[0])).abs() < 1e-9);
    }

    #[test]
    fn test_funding_network_prices_far_from_the_money_stay_small_and_finite() {
        let signed = signed();
        let calls = floats(
            &funding_network_prices(
                &signed,
                &params(20.0, 100.0, 0.0, black_scholes(SIGMA), OptionStyle::Call),
            )
            .unwrap(),
        );
        let puts = floats(
            &funding_network_prices(
                &signed,
                &params(500.0, 100.0, 0.0, black_scholes(SIGMA), OptionStyle::Put),
            )
            .unwrap(),
        );
        for value in calls.iter().chain(puts.iter()) {
            assert!(value.is_finite());
            assert!(value.abs() < 1e-3, "{value}");
        }
    }

    #[test]
    fn test_funding_network_prices_high_volatility_shrinks_the_damping() {
        // 300% volatility against a slow network: the default damping would
        // leave the strip of finite moments.
        let ladder = ladder(4.0, 2);
        let kernel = ModelKernel::new(&black_scholes(3.0), 0.0).unwrap();
        let order = damping(&kernel, 4.0, OptionStyle::Call).unwrap();
        assert!(order > 1.0 && order < 1.75);
        let inputs = params(100.0, 100.0, 0.0, black_scholes(3.0), OptionStyle::Call);
        let prices = floats(&funding_network_prices(&ladder, &inputs).unwrap());
        for (index, price) in prices.iter().enumerate() {
            let reference = everlasting_price(&closed_form(
                100.0,
                100.0,
                0.0,
                3.0,
                4.0,
                index as u32 + 1,
                OptionStyle::Call,
            ))
            .unwrap()
            .to_f64()
            .unwrap();
            assert!(relative(*price, reference) < 1e-9);
        }
    }

    #[test]
    fn test_funding_network_prices_rejects_a_drift_at_the_decay_rate() {
        let ladder = ladder(2.0, 2);
        let inputs = params(100.0, 100.0, 2.0, black_scholes(SIGMA), OptionStyle::Call);
        assert!(matches!(
            funding_network_prices(&ladder, &inputs),
            Err(PricingError::InvalidParameter {
                parameter: "drift",
                ..
            })
        ));
    }

    #[test]
    fn test_funding_network_params_rejects_values_outside_the_domain() {
        let model = black_scholes(SIGMA);
        assert!(matches!(
            FundingNetworkParams::try_new(
                Positive::ZERO,
                Positive::HUNDRED,
                dec!(0),
                model,
                OptionStyle::Call
            ),
            Err(PricingError::InvalidParameter {
                parameter: "underlying_price",
                ..
            })
        ));
        assert!(matches!(
            FundingNetworkParams::try_new(
                Positive::HUNDRED,
                Positive::ZERO,
                dec!(0),
                model,
                OptionStyle::Call
            ),
            Err(PricingError::InvalidParameter {
                parameter: "strike_price",
                ..
            })
        ));
        assert!(matches!(
            FundingNetworkParams::try_new(
                Positive::HUNDRED,
                Positive::HUNDRED,
                dec!(0),
                LevyModel::BlackScholes {
                    volatility: Positive::ZERO
                },
                OptionStyle::Call
            ),
            Err(PricingError::InvalidParameter {
                parameter: "volatility",
                ..
            })
        ));
    }

    #[test]
    fn test_funding_network_params_accessors_return_the_inputs() {
        let inputs = params(101.0, 99.0, 0.01, merton(), OptionStyle::Put);
        assert_eq!(inputs.underlying_price(), pos_or_panic!(101.0));
        assert_eq!(inputs.strike_price(), pos_or_panic!(99.0));
        assert_eq!(inputs.drift(), decimal(0.01));
        assert_eq!(inputs.model(), merton());
        assert_eq!(inputs.model().volatility(), pos_or_panic!(SIGMA));
        assert_eq!(inputs.option_style(), OptionStyle::Put);
    }

    /// Undiscounted Black-Scholes call at maturity `years`, with drift `mu`.
    fn black_call(spot: f64, strike: f64, mu: f64, years: f64) -> f64 {
        use statrs::distribution::{ContinuousCDF, Normal};
        let normal = Normal::new(0.0, 1.0).unwrap();
        let root = SIGMA * years.sqrt();
        let d1 = ((spot / strike).ln() + (mu + 0.5 * SIGMA * SIGMA) * years) / root;
        spot * (mu * years).exp() * normal.cdf(d1) - strike * normal.cdf(d1 - root)
    }

    fn discrete(interval: f64) -> FundingSchedule {
        FundingSchedule::Discrete {
            interval: pos_or_panic!(interval),
        }
    }

    /// `Σ_k (I − Δ·A)^{k−1}·Δ·A·1·C(kΔ)` by direct iteration.
    fn power_sum(rows: &[&[f64]], interval: f64, spot: f64, strike: f64, mu: f64) -> Vec<f64> {
        let order = rows.len();
        let mut weights: Vec<f64> = rows
            .iter()
            .map(|row| interval * row.iter().sum::<f64>())
            .collect();
        let mut prices = vec![0.0; order];
        for period in 1..=4_000_u32 {
            let value = black_call(spot, strike, mu, f64::from(period) * interval);
            for (price, weight) in prices.iter_mut().zip(weights.iter()) {
                *price += weight * value;
            }
            weights = (0..order)
                .map(|i| {
                    weights[i] - interval * (0..order).map(|j| rows[i][j] * weights[j]).sum::<f64>()
                })
                .collect();
        }
        prices
    }

    #[test]
    fn test_funding_network_prices_with_daily_funding_matches_geometric_sum() {
        // One contract with a mean maturity of 30 days, funded every day.
        let rate = 365.0 / 30.0;
        let interval = 1.0 / 365.0;
        let mut worst = 0.0_f64;
        for (spot, mu, style) in [
            (100.0, 0.0, OptionStyle::Call),
            (92.0, 0.05, OptionStyle::Call),
            (109.0, 0.0, OptionStyle::Call),
            (100.0, 0.05, OptionStyle::Put),
        ] {
            let call = power_sum(&[&[rate]], interval, spot, 100.0, mu)[0];
            let expected = match style {
                OptionStyle::Call => call,
                // Parity on the forward `Σ_k p·(1 − p)^{k−1}·S·e^{μkΔ}`.
                OptionStyle::Put => {
                    let leave = rate * interval;
                    let growth = (mu * interval).exp();
                    call - spot * leave * growth / (1.0 - (1.0 - leave) * growth) + 100.0
                }
            };
            let prices = funding_network_prices_with(
                &ladder(rate, 1),
                &params(spot, 100.0, mu, black_scholes(SIGMA), style),
                discrete(interval),
            )
            .unwrap();
            worst = worst.max(relative(floats(&prices)[0], expected));
        }
        assert!(worst < 1e-8, "worst relative error {worst:e}");
    }

    #[test]
    fn test_funding_network_prices_with_discrete_ladder_matches_power_sum() {
        // Rung `j` leaves after `j` payments in its favour: negative
        // binomial weights, here from the powers of `I − Δ·A`.
        let rate = 3.0 * 365.0 / 30.0;
        let interval = 2.0 / 365.0;
        let rows: [&[f64]; 3] = [&[rate, 0.0, 0.0], &[-rate, rate, 0.0], &[0.0, -rate, rate]];
        let expected = power_sum(&rows, interval, 96.0, 100.0, 0.0);
        let prices = floats(
            &funding_network_prices_with(
                &ladder(rate, 3),
                &params(96.0, 100.0, 0.0, black_scholes(SIGMA), OptionStyle::Call),
                discrete(interval),
            )
            .unwrap(),
        );
        for (price, reference) in prices.iter().zip(expected.iter()) {
            assert!(
                relative(*price, *reference) < 1e-8,
                "{price} against {reference}"
            );
        }
    }

    #[test]
    fn test_funding_network_prices_with_signed_network_matches_power_sum() {
        let rows: [&[f64]; 3] = [&SIGNED[0], &SIGNED[1], &SIGNED[2]];
        let interval = 1.0 / 365.0;
        for spot in [90.0, 100.0, 110.0] {
            let expected = power_sum(&rows, interval, spot, 100.0, 0.0);
            let prices = floats(
                &funding_network_prices_with(
                    &signed(),
                    &params(spot, 100.0, 0.0, black_scholes(SIGMA), OptionStyle::Call),
                    discrete(interval),
                )
                .unwrap(),
            );
            for (price, reference) in prices.iter().zip(expected.iter()) {
                assert!(
                    relative(*price, *reference) < 1e-8,
                    "spot {spot}: {price} against {reference}"
                );
            }
        }
    }

    #[test]
    fn test_funding_network_prices_with_full_funding_is_a_european_option() {
        // `interval·rate = 1`: every payment closes the whole gap.
        for (spot, style) in [(100.0, OptionStyle::Call), (97.0, OptionStyle::Call)] {
            let prices = funding_network_prices_with(
                &ladder(12.0, 1),
                &params(spot, 100.0, 0.0, black_scholes(SIGMA), style),
                FundingSchedule::Discrete {
                    interval: Positive::ONE / pos_or_panic!(12.0),
                },
            )
            .unwrap();
            let expected = black_call(spot, 100.0, 0.0, 1.0 / 12.0);
            assert!(relative(floats(&prices)[0], expected) < 1e-8);
        }
    }

    #[test]
    fn test_funding_network_prices_with_shrinking_interval_tends_to_continuous_at_first_order() {
        let network = ladder(RATE, 2);
        let params = params(100.0, 100.0, 0.0, black_scholes(SIGMA), OptionStyle::Call);
        let continuous = floats(&funding_network_prices(&network, &params).unwrap());
        let gap = |hours: f64| -> Vec<f64> {
            floats(
                &funding_network_prices_with(&network, &params, discrete(hours / 8760.0)).unwrap(),
            )
            .iter()
            .zip(continuous.iter())
            .map(|(price, limit)| price - limit)
            .collect()
        };
        let (coarse, fine) = (gap(2.0), gap(1.0));
        for ((wide, narrow), limit) in coarse.iter().zip(fine.iter()).zip(continuous.iter()) {
            // Small against the price, and halved with the interval.
            assert!(wide.abs() < 0.02 * limit);
            assert!(wide.abs() > 1e-7 * limit);
            assert!(
                (wide / narrow - 2.0).abs() < 0.05,
                "ratio {}",
                wide / narrow
            );
        }
        // The same entry point with continuous funding is the plain pricer.
        assert_eq!(
            funding_network_prices_with(&network, &params, FundingSchedule::Continuous).unwrap(),
            funding_network_prices(&network, &params).unwrap()
        );
    }

    #[test]
    fn test_funding_network_deltas_with_discrete_funding_match_finite_differences() {
        let interval = 1.0 / 365.0;
        let step = 0.01;
        for (network, mu, style) in [
            (ladder(RATE, 3), 0.03, OptionStyle::Call),
            (signed(), 0.0, OptionStyle::Put),
        ] {
            let price = |spot: f64| {
                floats(
                    &funding_network_prices_with(
                        &network,
                        &params(spot, 100.0, mu, merton(), style),
                        discrete(interval),
                    )
                    .unwrap(),
                )
            };
            let deltas = floats(
                &funding_network_deltas_with(
                    &network,
                    &params(104.0, 100.0, mu, merton(), style),
                    discrete(interval),
                )
                .unwrap(),
            );
            let (up, down) = (price(104.0 + step), price(104.0 - step));
            for ((delta, high), low) in deltas.iter().zip(up.iter()).zip(down.iter()) {
                assert!((delta - (high - low) / (2.0 * step)).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn test_funding_network_decay_rate_with_matches_the_spectrum() {
        let network = ladder(RATE, 3);
        assert_eq!(
            network
                .decay_rate_with(FundingSchedule::Continuous)
                .unwrap(),
            network.decay_rate().unwrap()
        );
        // One eigenvalue `κ`: `−ln(1 − κΔ)/Δ`, above `κ`.
        let interval = 1.0 / 365.0;
        let rate = network
            .decay_rate_with(discrete(interval))
            .unwrap()
            .to_f64()
            .unwrap();
        assert!(relative(rate, -(1.0 - RATE * interval).ln() / interval) < 1e-12);
        assert!(rate > RATE);
        assert!(
            relative(
                network.stable_interval_bound().unwrap().to_f64().unwrap(),
                2.0 / RATE
            ) < 1e-12
        );
        // A complex pair lowers the bound below `2/Re λ`.
        let (real, imaginary) = (27.461_221_363_433_275, 28.447_984_070_088_694);
        let bound = signed().stable_interval_bound().unwrap().to_f64().unwrap();
        assert!(relative(bound, 2.0 * real / (real * real + imaginary * imaginary)) < 1e-9);
        assert!(bound < 2.0 / real);
    }

    #[test]
    fn test_funding_network_prices_with_rejects_schedules_outside_the_domain() {
        let params = params(100.0, 100.0, 0.0, black_scholes(SIGMA), OptionStyle::Call);
        let bound = signed().stable_interval_bound().unwrap().to_f64().unwrap();
        for interval in [bound * 1.000_001, 2.0 * bound] {
            assert!(matches!(
                funding_network_prices_with(&signed(), &params, discrete(interval)),
                Err(PricingError::InvalidParameter {
                    parameter: "interval",
                    ..
                })
            ));
        }
        // Just inside the bound the network is stable, if slow to clear.
        assert!(signed().decay_rate_with(discrete(0.9 * bound)).unwrap() > Decimal::ZERO);
        assert!(matches!(
            funding_network_prices_with(
                &signed(),
                &params,
                FundingSchedule::Discrete {
                    interval: Positive::ZERO
                }
            ),
            Err(PricingError::InvalidParameter {
                parameter: "interval",
                ..
            })
        ));
        // A single payment closes the gap: the rate is infinite and has no
        // `Decimal` form.
        assert!(matches!(
            ladder(12.0, 1).decay_rate_with(FundingSchedule::Discrete {
                interval: Positive::ONE / pos_or_panic!(12.0)
            }),
            Err(PricingError::NonFinite { .. })
        ));
    }
}
