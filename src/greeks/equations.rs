/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 11/8/24
******************************************************************************/
use crate::constants::{TRADING_DAYS, ZERO};
use crate::error::greeks::{CalculationErrorKind, GreeksError};
use crate::greeks::utils::{big_n, d1, n};
use crate::model::decimal::{d_add, d_div, d_exp, d_mul, d_sub};
use crate::model::types::{OptionStyle, OptionType};
use crate::{Options, Side};
use positive::Positive;
use pretty_simple_display::{DebugPretty, DisplaySimple};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::cell::OnceCell;
use utoipa::ToSchema;

/// Represents a complete set of option Greeks, which measure the sensitivity of an option's
/// price to various market factors.
///
/// Option Greeks are essential metrics in options trading and risk management, each quantifying
/// how the theoretical value of an option changes with respect to different parameters.
///
/// ## Fields
///
/// Each field represents a specific Greek measure:
///
/// * `delta`: Measures the rate of change in the option price relative to changes in the underlying asset price
/// * `gamma`: Measures the rate of change of delta in relation to changes in the underlying asset price
/// * `theta`: Measures the rate of change in the option price with respect to time decay (time sensitivity)
/// * `vega`: Measures the rate of change in the option price with respect to changes in implied volatility
/// * `rho`: Measures the rate of change in the option price with respect to the risk-free interest rate
/// * `rho_d`: Measures the rate of change in the option price with respect to the dividend yield
/// * `alpha`: Represents a measure of an option's excess return relative to what would be predicted by models
/// * `vanna`: Measures the rate of change of delta in relation to changes in implied volatility
/// * `vomma`: Measures the rate of change of vega in relation to changes in implied volatility
/// * `veta`: Measures the rate of change of vega in relation to changes in time
/// * `charm`: Measures the rate of change of delta in relation to changes in time
/// * `color`: Measures the rate of change of gamma in relation to changes in time
///
/// These metrics help traders understand and manage the various dimensions of risk in option positions.
#[derive(DebugPretty, DisplaySimple, Clone, PartialEq, Serialize, ToSchema)]
pub struct Greek {
    /// Measures sensitivity to changes in the underlying asset's price (first derivative)
    pub delta: Decimal,
    /// Measures the rate of change in delta (second derivative of the option price)
    pub gamma: Decimal,
    /// Measures the time decay of an option's value (sensitivity to the passage of time)
    pub theta: Decimal,
    /// Measures sensitivity to changes in implied volatility
    pub vega: Decimal,
    /// Measures sensitivity to changes in the risk-free interest rate
    pub rho: Decimal,
    /// Measures sensitivity to changes in the dividend yield
    pub rho_d: Decimal,
    /// Measures the option's theoretical value not explained by other Greeks
    pub alpha: Decimal,
    /// Measures the rate of change of delta in relation to changes in implied volatility
    pub vanna: Decimal,
    /// Measures the rate of change of vega in relation to changes in implied volatility
    pub vomma: Decimal,
    /// Measures the rate of change of vega in relation to changes in time
    pub veta: Decimal,
    /// Measures the rate of change of delta in relation to changes in time
    pub charm: Decimal,
    /// Measures the rate of change of gamma in relation to changes in time
    pub color: Decimal,
}

/// A struct representing a snapshot of the Greeks, financial measures used to assess risk and
/// sensitivity of derivative instruments such as options.
///
/// The Greeks provide insights into how various factors, such as price movement, time decay,
/// or volatility, affect the theoretical value of derivatives. This struct supports serialization
/// and deserialization for storage or communication purposes, and implements common traits like
/// `Debug`, `Clone`, and `PartialEq`.
///
/// # Wire compatibility
///
/// This type crosses REST boundaries (it is carried by [`crate::chains::OptionData`]
/// and by `TradeRecord`), so it deliberately does **not** set
/// `#[serde(deny_unknown_fields)]`. With that attribute in place, adding a
/// thirteenth greek would break deserialization of new payloads by consumers
/// built against an older version. Keep new fields additive.
///
/// # Meaning of the optional fields
///
/// `rho`, `rho_d` and `alpha` are `Option<Decimal>`. `None` means **not
/// computed or not meaningful for these inputs** — it is never a stand-in for
/// zero, and must not be defaulted to one. They serialize as an explicit
/// `null` rather than being skipped, so that "not meaningful" stays
/// distinguishable from "field absent" on the wire.
#[derive(DebugPretty, DisplaySimple, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct GreeksSnapshot {
    /// Measures sensitivity to changes in the underlying asset's price (first derivative)
    pub delta: Decimal,
    /// Measures the rate of change in delta (second derivative of the option price)
    pub gamma: Decimal,
    /// Measures the time decay of an option's value (sensitivity to the passage of time)
    pub theta: Decimal,
    /// Measures sensitivity to changes in implied volatility
    pub vega: Decimal,
    /// Measures sensitivity to changes in the risk-free interest rate
    pub rho: Option<Decimal>,
    /// Measures sensitivity to changes in the dividend yield
    pub rho_d: Option<Decimal>,
    /// Measures the option's theoretical value not explained by other Greeks
    pub alpha: Option<Decimal>,
    /// Measures the rate of change of delta in relation to changes in implied volatility
    pub vanna: Decimal,
    /// Measures the rate of change of vega in relation to changes in implied volatility
    pub vomma: Decimal,
    /// Measures the rate of change of vega in relation to changes in time
    pub veta: Decimal,
    /// Measures the rate of change of delta in relation to changes in time
    pub charm: Decimal,
    /// Measures the rate of change of gamma in relation to changes in time
    pub color: Decimal,
}

impl From<Greek> for GreeksSnapshot {
    /// Widens a [`Greek`] into a [`GreeksSnapshot`].
    ///
    /// The conversion is lossless: [`Greek`] holds the same twelve values with
    /// `rho`, `rho_d` and `alpha` as plain `Decimal`, so they are wrapped in
    /// `Some`. This deliberately performs no interpretation of sentinel values
    /// — see [`alpha`], which returns `Decimal::MAX` when theta is zero.
    /// Callers that publish a snapshot are responsible for mapping such
    /// sentinels to `None`.
    #[inline]
    fn from(greek: Greek) -> Self {
        Self {
            delta: greek.delta,
            gamma: greek.gamma,
            theta: greek.theta,
            vega: greek.vega,
            rho: Some(greek.rho),
            rho_d: Some(greek.rho_d),
            alpha: Some(greek.alpha),
            vanna: greek.vanna,
            vomma: greek.vomma,
            veta: greek.veta,
            charm: greek.charm,
            color: greek.color,
        }
    }
}

/// Trait that provides option Greeks calculation functionality for financial instruments.
///
/// The `Greeks` trait enables implementing types to calculate option sensitivity metrics
/// (Greeks) across multiple option positions. Any type that can provide access to a collection
/// of options can implement this trait to gain the ability to calculate aggregate Greek values.
///
/// This trait uses a composition approach where implementation only requires defining the
/// `get_options()` method, while default implementations for all Greek calculations are provided.
///
/// # Greek Calculations
///
/// The trait provides calculations for:
/// - Delta: Sensitivity to changes in the underlying asset's price
/// - Gamma: Rate of change of delta (acceleration of price movement)
/// - Theta: Time decay of option value
/// - Vega: Sensitivity to changes in volatility
/// - Rho: Sensitivity to changes in interest rates
/// - Rho_d: Sensitivity to changes in dividend yield
/// - Alpha: Ratio between gamma and theta
/// - Vanna: Rate of change of delta in relation to changes in implied volatility
/// - Vomma: Rate of change of vega in relation to changes in implied volatility
/// - Veta: Rate of change of vega in relation to changes in time
/// - Charm: Rate of change of delta in relation to changes in time
/// - Color: Rate of change of gamma in relation to changes in time
///
/// # Usage
///
/// Implementers only need to provide the `get_options()` method which returns a vector of
/// references to option contracts. The trait will handle aggregating the Greek values across
/// all options in the collection.
///
/// # Sign convention
///
/// Every method here returns the sensitivity of the **aggregate position**: each
/// leg is signed by its [`Side`] and scaled by its `quantity`, then summed. A
/// short leg contributes the negative of the equivalent long one, so a
/// short-premium strategy reports a positive theta when it collects decay.
///
/// The single exception is [`Greeks::alpha`], which sums per-leg `gamma / theta`
/// ratios. A short negates both terms, so each ratio is invariant under `Side`
/// by construction.
///
/// # Errors
///
/// Methods return `Result<T, GreeksError>` to handle various calculation errors that may
/// occur during Greek computations.
pub trait Greeks {
    /// Returns a vector of references to the option contracts for which Greeks will be calculated.
    ///
    /// This is the only method that must be implemented by types adopting this trait.
    /// All other methods have default implementations based on this method.
    ///
    /// # Errors
    ///
    /// Returns a `GreeksError` if there is an issue retrieving the options.
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError>;

    /// Calculates and returns all Greeks as a single `Greek` struct.
    ///
    /// This method provides a convenient way to obtain all Greek values at once.
    /// It calls each individual Greek calculation method and compiles the results.
    ///
    /// # Errors
    ///
    /// Returns a `GreeksError` if any individual Greek calculation fails, or
    /// [`crate::error::DecimalError::Overflow`] (wrapped by `GreeksError`) when
    /// one of the twelve running sums leaves the representable `Decimal` range.
    ///
    /// The `alpha` sum has one further failure of its own, and it is reachable
    /// on ordinary input. [`alpha`] returns `Decimal::MAX` for a leg whose theta
    /// has vanished, and that value cannot be added to a real one; a leg
    /// carrying it beside any other contribution is refused by [`add_alpha`],
    /// which names the leg. See [`Greeks::alpha`].
    fn greeks(&self) -> Result<Greek, GreeksError> {
        // Aggregate option by option rather than greek by greek, so the shared
        // Black-Scholes kernels are computed once per option instead of once
        // per greek. `Decimal` addition is exact, so the sums are identical to
        // accumulating each greek across every option in turn.
        let options = self.get_options()?;
        let mut delta = Decimal::ZERO;
        let mut gamma = Decimal::ZERO;
        let mut theta = Decimal::ZERO;
        let mut vega = Decimal::ZERO;
        let mut rho = Decimal::ZERO;
        let mut rho_d = Decimal::ZERO;
        let mut alpha = Decimal::ZERO;
        let mut vanna = Decimal::ZERO;
        let mut vomma = Decimal::ZERO;
        let mut veta = Decimal::ZERO;
        let mut charm = Decimal::ZERO;
        let mut color = Decimal::ZERO;
        for (index, option) in options.into_iter().enumerate() {
            let single = greeks_for(option)?;
            delta = d_add(delta, single.delta, "greeks::aggregate::delta")?;
            gamma = d_add(gamma, single.gamma, "greeks::aggregate::gamma")?;
            theta = d_add(theta, single.theta, "greeks::aggregate::theta")?;
            vega = d_add(vega, single.vega, "greeks::aggregate::vega")?;
            rho = d_add(rho, single.rho, "greeks::aggregate::rho")?;
            rho_d = d_add(rho_d, single.rho_d, "greeks::aggregate::rho_d")?;
            alpha = add_alpha(
                alpha,
                single.alpha,
                option,
                index,
                "greeks::aggregate::alpha",
            )?;
            vanna = d_add(vanna, single.vanna, "greeks::aggregate::vanna")?;
            vomma = d_add(vomma, single.vomma, "greeks::aggregate::vomma")?;
            veta = d_add(veta, single.veta, "greeks::aggregate::veta")?;
            charm = d_add(charm, single.charm, "greeks::aggregate::charm")?;
            color = d_add(color, single.color, "greeks::aggregate::color")?;
        }
        Ok(Greek {
            delta,
            gamma,
            theta,
            vega,
            rho,
            rho_d,
            alpha,
            vanna,
            vomma,
            veta,
            charm,
            color,
        })
    }

    /// Calculates the aggregate delta value for all options.
    ///
    /// Delta measures the rate of change in an option's price with respect to
    /// changes in the underlying asset's price.
    ///
    /// # Errors
    ///
    /// Returns a `GreeksError` if the options can't be retrieved or delta calculation
    /// fails, and [`crate::error::DecimalError::Overflow`] when the running sum
    /// leaves the representable `Decimal` range.
    fn delta(&self) -> Result<Decimal, GreeksError> {
        let options = self.get_options()?;
        let mut delta_value = Decimal::ZERO;
        for option in options {
            delta_value = d_add(delta_value, delta(option)?, "greeks::delta::aggregate")?;
        }
        Ok(delta_value)
    }

    /// Calculates the aggregate gamma value for all options.
    ///
    /// Gamma measures the rate of change of delta with respect to
    /// changes in the underlying asset's price.
    ///
    /// # Errors
    ///
    /// Returns a `GreeksError` if the options can't be retrieved or gamma calculation
    /// fails, and [`crate::error::DecimalError::Overflow`] when the running sum
    /// leaves the representable `Decimal` range.
    fn gamma(&self) -> Result<Decimal, GreeksError> {
        let options = self.get_options()?;
        let mut gamma_value = Decimal::ZERO;
        for option in options {
            gamma_value = d_add(gamma_value, gamma(option)?, "greeks::gamma::aggregate")?;
        }
        Ok(gamma_value)
    }

    /// Calculates the aggregate theta value for all options.
    ///
    /// Theta measures the rate of change of the option price with respect to time,
    /// also known as time decay.
    ///
    /// # Errors
    ///
    /// Returns a `GreeksError` if the options can't be retrieved or theta calculation
    /// fails, and [`crate::error::DecimalError::Overflow`] when the running sum
    /// leaves the representable `Decimal` range.
    fn theta(&self) -> Result<Decimal, GreeksError> {
        let options = self.get_options()?;
        let mut theta_value = Decimal::ZERO;
        for option in options {
            theta_value = d_add(theta_value, theta(option)?, "greeks::theta::aggregate")?;
        }
        Ok(theta_value)
    }

    /// Calculates the aggregate vega value for all options.
    ///
    /// Vega measures the sensitivity of the option price to changes in
    /// the volatility of the underlying asset.
    ///
    /// # Errors
    ///
    /// Returns a `GreeksError` if the options can't be retrieved or vega calculation
    /// fails, and [`crate::error::DecimalError::Overflow`] when the running sum
    /// leaves the representable `Decimal` range.
    fn vega(&self) -> Result<Decimal, GreeksError> {
        let options = self.get_options()?;
        let mut vega_value = Decimal::ZERO;
        for option in options {
            vega_value = d_add(vega_value, vega(option)?, "greeks::vega::aggregate")?;
        }
        Ok(vega_value)
    }

    /// Calculates the aggregate rho value for all options.
    ///
    /// Rho measures the sensitivity of the option price to changes in
    /// the risk-free interest rate.
    ///
    /// # Errors
    ///
    /// Returns a `GreeksError` if the options can't be retrieved or rho calculation
    /// fails, and [`crate::error::DecimalError::Overflow`] when the running sum
    /// leaves the representable `Decimal` range.
    fn rho(&self) -> Result<Decimal, GreeksError> {
        let options = self.get_options()?;
        let mut rho_value = Decimal::ZERO;
        for option in options {
            rho_value = d_add(rho_value, rho(option)?, "greeks::rho::aggregate")?;
        }
        Ok(rho_value)
    }

    /// Calculates the aggregate rho_d value for all options.
    ///
    /// Rho_d measures the sensitivity of the option price to changes in
    /// the dividend yield of the underlying asset.
    ///
    /// # Errors
    ///
    /// Returns a `GreeksError` if the options can't be retrieved or rho_d calculation
    /// fails, and [`crate::error::DecimalError::Overflow`] when the running sum
    /// leaves the representable `Decimal` range.
    fn rho_d(&self) -> Result<Decimal, GreeksError> {
        let options = self.get_options()?;
        let mut rho_d_value = Decimal::ZERO;
        for option in options {
            rho_d_value = d_add(rho_d_value, rho_d(option)?, "greeks::rho_d::aggregate")?;
        }
        Ok(rho_d_value)
    }

    /// Calculates the aggregate alpha value for all options.
    ///
    /// Alpha represents the ratio between gamma and theta, providing insight into
    /// the option's risk/reward efficiency with respect to time decay.
    ///
    /// # The sentinel in a sum
    ///
    /// [`alpha`] answers `Decimal::MAX` for a leg whose theta has vanished, and
    /// that value cannot be added to a real one. Two such legs leave the
    /// representable range. One beside an ordinary leg does not, which is
    /// worse: `Decimal::MAX` plus a value below one rescales and rounds
    /// straight back to `Decimal::MAX`, so the arithmetic succeeds while the
    /// ordinary leg's contribution is silently dropped.
    ///
    /// Both are refused, by an explicit guard rather than by the arithmetic;
    /// see [`add_alpha`]. A sentinel leg that is the only leg, or that sits
    /// beside legs whose own alpha is zero, still reports the sentinel, because
    /// nothing is lost in those sums.
    ///
    /// # Errors
    ///
    /// Returns a `GreeksError` if the options can't be retrieved or alpha calculation
    /// fails, and [`crate::error::DecimalError::Overflow`] when the running sum leaves
    /// the representable `Decimal` range. Returns
    /// [`GreeksError::CalculationError`] naming the leg when that leg's alpha is
    /// the sentinel and another leg contributes to the same sum.
    fn alpha(&self) -> Result<Decimal, GreeksError> {
        let options = self.get_options()?;
        let mut alpha_value = Decimal::ZERO;
        for (index, option) in options.into_iter().enumerate() {
            alpha_value = add_alpha(
                alpha_value,
                alpha(option)?,
                option,
                index,
                "greeks::alpha::aggregate",
            )?;
        }
        Ok(alpha_value)
    }

    /// Calculates the aggregate vanna value for all options.
    ///
    /// Vanna measures the sensitivity of the option delta to changes in
    /// the volatility of the underlying asset
    ///
    /// # Errors
    ///
    /// Returns a `GreeksError` if the options can't be retrieved or vanna calculation
    /// fails, and [`crate::error::DecimalError::Overflow`] when the running sum
    /// leaves the representable `Decimal` range.
    fn vanna(&self) -> Result<Decimal, GreeksError> {
        let options = self.get_options()?;
        let mut vanna_value = Decimal::ZERO;
        for option in options {
            vanna_value = d_add(vanna_value, vanna(option)?, "greeks::vanna::aggregate")?;
        }
        Ok(vanna_value)
    }

    /// Calculates the aggregate vomma value for all options.
    ///
    /// Vomma measures the sensitivity of the option vega to changes in
    /// the volatility of the underlying asset
    ///
    /// # Errors
    ///
    /// Returns a `GreeksError` if the options can't be retrieved or vomma calculation
    /// fails, and [`crate::error::DecimalError::Overflow`] when the running sum
    /// leaves the representable `Decimal` range.
    fn vomma(&self) -> Result<Decimal, GreeksError> {
        let options = self.get_options()?;
        let mut vomma_value = Decimal::ZERO;
        for option in options {
            vomma_value = d_add(vomma_value, vomma(option)?, "greeks::vomma::aggregate")?;
        }
        Ok(vomma_value)
    }

    /// Calculates the aggregate veta value for all options.
    ///
    /// Veta measures the sensitivity of the option vega in relation
    /// to changes in time
    ///
    /// # Errors
    ///
    /// Returns a `GreeksError` if the options can't be retrieved or veta calculation
    /// fails, and [`crate::error::DecimalError::Overflow`] when the running sum
    /// leaves the representable `Decimal` range.
    fn veta(&self) -> Result<Decimal, GreeksError> {
        let options = self.get_options()?;
        let mut veta_value = Decimal::ZERO;
        for option in options {
            veta_value = d_add(veta_value, veta(option)?, "greeks::veta::aggregate")?;
        }
        Ok(veta_value)
    }

    /// Calculates the aggregate charm value for all options.
    ///
    /// Charm measures the rate of change of the option delta with respect to time,
    /// also known as delta decay.
    ///
    /// # Errors
    ///
    /// Returns a `GreeksError` if the options can't be retrieved or charm calculation
    /// fails, and [`crate::error::DecimalError::Overflow`] when the running sum
    /// leaves the representable `Decimal` range.
    fn charm(&self) -> Result<Decimal, GreeksError> {
        let options = self.get_options()?;
        let mut charm_value = Decimal::ZERO;
        for option in options {
            charm_value = d_add(charm_value, charm(option)?, "greeks::charm::aggregate")?;
        }
        Ok(charm_value)
    }

    /// Calculates the aggregate color value for all options.
    ///
    /// Color measures the rate of change of the option gamma with respect to time,
    /// also known as gamma decay.
    ///
    /// # Errors
    ///
    /// Returns a `GreeksError` if the options can't be retrieved or color calculation
    /// fails, and [`crate::error::DecimalError::Overflow`] when the running sum
    /// leaves the representable `Decimal` range.
    fn color(&self) -> Result<Decimal, GreeksError> {
        let options = self.get_options()?;
        let mut color_value = Decimal::ZERO;
        for option in options {
            color_value = d_add(color_value, color(option)?, "greeks::color::aggregate")?;
        }
        Ok(color_value)
    }
}

/// The Black-Scholes intermediates that every greek shares, computed at most
/// once each.
///
/// Each of the twelve greek functions used to re-derive `d1`, `d2`, the normal
/// pdf and cdf, and the two discount factors from scratch, so one call to
/// [`Greeks::greeks`] evaluated the same handful of expensive `Decimal`
/// transcendentals dozens of times. Measured on an M-series Mac, those kernels
/// were about 93% of the aggregate's cost.
///
/// Every field is computed with the identical expression the individual greeks
/// used, so sharing them is value-preserving rather than an approximation.
/// `d2` in particular is derived exactly as [`crate::greeks::d2`] derives it,
/// `d1 - sigma * sqrt(T)`, which also avoids a second `d1`.
///
/// Everything past `d1` is computed lazily, so a caller that needs one greek
/// pays only for that greek's inputs while the aggregate path shares them all.
///
/// Only valid for a live European option: the caller must have established that
/// the option type is European, that the time to expiry is non-zero and that the
/// implied volatility is non-zero. Each greek keeps its own degenerate branch
/// for the cases this cannot represent.
#[derive(Debug, Clone)]
pub(crate) struct BlackScholesKernels {
    /// Time to expiry in years.
    t: Positive,
    /// `sqrt(T)`, shared by every greek that scales by time.
    sqrt_t: Positive,
    d1: Decimal,
    sigma: Positive,
    q: Decimal,
    r: Decimal,
    d2: OnceCell<Decimal>,
    n_d1: OnceCell<Decimal>,
    big_n_d1: OnceCell<Decimal>,
    big_n_neg_d1: OnceCell<Decimal>,
    big_n_d2: OnceCell<Decimal>,
    big_n_neg_d2: OnceCell<Decimal>,
    exp_minus_qt: OnceCell<Decimal>,
    exp_minus_rt: OnceCell<Decimal>,
}

impl BlackScholesKernels {
    /// Computes `d1` and `sqrt(T)` for `option` at a time to expiry of `t`.
    ///
    /// Everything else is derived on first use. `t` is passed in rather than
    /// re-derived because every caller has already obtained it to test its own
    /// degenerate branch.
    fn new(option: &Options, t: Positive) -> Result<Self, GreeksError> {
        let carry_rate = d_sub(
            option.risk_free_rate,
            option.dividend_yield.to_dec(),
            "greeks::kernels::carry_rate",
        )?;
        // `Positive::sqrt` panics on overflow; the checked counterpart
        // surfaces it as a `PositiveError` instead.
        let sqrt_t = t.checked_sqrt()?;
        let d1 = d1(
            option.underlying_price,
            option.strike_price,
            carry_rate,
            t,
            option.implied_volatility,
        )?;

        Ok(Self {
            t,
            sqrt_t,
            d1,
            sigma: option.implied_volatility,
            q: option.dividend_yield.to_dec(),
            r: option.risk_free_rate,
            d2: OnceCell::new(),
            n_d1: OnceCell::new(),
            big_n_d1: OnceCell::new(),
            big_n_neg_d1: OnceCell::new(),
            big_n_d2: OnceCell::new(),
            big_n_neg_d2: OnceCell::new(),
            exp_minus_qt: OnceCell::new(),
            exp_minus_rt: OnceCell::new(),
        })
    }

    /// Time to expiry in years.
    fn t(&self) -> Positive {
        self.t
    }

    /// `sqrt(T)`.
    fn sqrt_t(&self) -> Positive {
        self.sqrt_t
    }

    fn d1(&self) -> Decimal {
        self.d1
    }

    /// `d2 = d1 - sigma * sqrt(T)`, exactly as [`crate::greeks::d2`] derives it.
    ///
    /// Fallible because `sigma * sqrt(T)` overflows for extreme inputs, and
    /// the raw operator panics rather than reporting it.
    fn d2(&self) -> Result<Decimal, GreeksError> {
        cached(&self.d2, || {
            let vol_time = d_mul(
                self.sigma.to_dec(),
                self.sqrt_t.to_dec(),
                "greeks::kernels::d2::vol_time",
            )?;
            Ok(d_sub(self.d1, vol_time, "greeks::kernels::d2")?)
        })
    }

    /// Normal pdf at `d1`; the most expensive single kernel.
    fn n_d1(&self) -> Result<Decimal, GreeksError> {
        cached(&self.n_d1, || n(self.d1))
    }

    fn big_n_d1(&self) -> Result<Decimal, GreeksError> {
        cached(&self.big_n_d1, || Ok(big_n(self.d1)?))
    }

    fn big_n_neg_d1(&self) -> Result<Decimal, GreeksError> {
        cached(&self.big_n_neg_d1, || Ok(big_n(-self.d1)?))
    }

    fn big_n_d2(&self) -> Result<Decimal, GreeksError> {
        cached(&self.big_n_d2, || Ok(big_n(self.d2()?)?))
    }

    fn big_n_neg_d2(&self) -> Result<Decimal, GreeksError> {
        cached(&self.big_n_neg_d2, || Ok(big_n(-self.d2()?)?))
    }

    /// `exp(-qT)`, the dividend discount factor.
    ///
    /// Fallible because both the exponent and the exponential overflow for
    /// extreme inputs, where `Decimal`'s operators panic. An exponent so
    /// negative that the factor is below the representable scale flushes to
    /// zero, which is the discount factor's limit.
    fn exp_minus_qt(&self) -> Result<Decimal, GreeksError> {
        cached(&self.exp_minus_qt, || {
            let exponent = d_mul(
                -self.t.to_dec(),
                self.q,
                "greeks::kernels::exp_minus_qt::exponent",
            )?;
            Ok(d_exp(exponent, "greeks::kernels::exp_minus_qt")?)
        })
    }

    /// `exp(-rT)`, the risk-free discount factor. See [`Self::exp_minus_qt`].
    fn exp_minus_rt(&self) -> Result<Decimal, GreeksError> {
        cached(&self.exp_minus_rt, || {
            let exponent = d_mul(
                -self.r,
                self.t.to_dec(),
                "greeks::kernels::exp_minus_rt::exponent",
            )?;
            Ok(d_exp(exponent, "greeks::kernels::exp_minus_rt")?)
        })
    }
}

/// Memoises a fallible kernel in a [`OnceCell`].
///
/// [`OnceCell::get_or_init`] cannot carry a `Result`, and `get_or_try_init` is
/// still unstable, so this does the same job by hand. A failed computation is
/// not cached, which is harmless: the inputs are fixed, so a retry fails again.
fn cached<F>(cell: &OnceCell<Decimal>, compute: F) -> Result<Decimal, GreeksError>
where
    F: FnOnce() -> Result<Decimal, GreeksError>,
{
    if let Some(value) = cell.get() {
        return Ok(*value);
    }
    let value = compute()?;
    let _ = cell.set(value);
    Ok(value)
}

/// The position's signed size, `+quantity` when long and `-quantity` when short.
///
/// [`Options::quantity`] is [`Positive`] and can never carry direction, so
/// [`Side`] is the only carrier of it. Every greek is the sensitivity of the
/// *position*, so the sign belongs wherever the size is applied.
///
/// Negating a `Decimal` is exact, so this needs no checked multiplication.
#[inline]
#[must_use]
fn signed_quantity(option: &Options) -> Decimal {
    let quantity = option.quantity.to_dec();
    if option.is_long() {
        quantity
    } else {
        -quantity
    }
}

/// Builds the shared kernels when `option` is a live European contract.
///
/// Returns `None` for the degenerate cases — a non-European type, a zero time to
/// expiry, or a zero implied volatility — where the individual greeks disagree
/// on what to return and must each run their own branch.
fn kernels_for(option: &Options) -> Result<Option<BlackScholesKernels>, GreeksError> {
    if !matches!(option.option_type, OptionType::European) {
        return Ok(None);
    }
    let t = option.expiration_date.get_years()?;
    if t == Decimal::ZERO || option.implied_volatility == ZERO {
        return Ok(None);
    }
    Ok(Some(BlackScholesKernels::new(option, t)?))
}

/// Computes all twelve greeks for a single option, sharing one set of
/// Black-Scholes kernels across them.
///
/// This is the fast path behind [`Greeks::greeks`]. For a degenerate option —
/// non-European, at expiry, or at zero volatility — the twelve disagree on what
/// to return, so each individual function runs and owns its own branch.
fn greeks_for(option: &Options) -> Result<Greek, GreeksError> {
    let Some(kernels) = kernels_for(option)? else {
        let gamma = gamma(option)?;
        let theta = theta(option)?;
        return Ok(Greek {
            delta: delta(option)?,
            gamma,
            theta,
            vega: vega(option)?,
            rho: rho(option)?,
            rho_d: rho_d(option)?,
            alpha: alpha_from(gamma, theta)?,
            vanna: vanna(option)?,
            vomma: vomma(option)?,
            veta: veta(option)?,
            charm: charm(option)?,
            color: color(option)?,
        });
    };

    let gamma = gamma_with(option, &kernels)?;
    let theta = theta_with(option, &kernels)?;
    Ok(Greek {
        delta: delta_with(option, &kernels)?,
        gamma,
        theta,
        vega: vega_with(option, &kernels)?,
        rho: rho_with(option, &kernels)?,
        rho_d: rho_d_with(option, &kernels)?,
        // Reuses the gamma and theta above instead of recomputing both.
        alpha: alpha_from(gamma, theta)?,
        vanna: vanna_with(option, &kernels)?,
        vomma: vomma_with(option, &kernels)?,
        veta: veta_with(option, &kernels)?,
        charm: charm_with(option, &kernels)?,
        color: color_with(option, &kernels)?,
    })
}

/// Calculates the delta of an option.
///
/// The delta measures the sensitivity of an option's price to changes in the price of the
/// underlying asset. It is calculated differently for call and put options. For options
/// with zero implied volatility, the delta is determined based on whether the option is
/// in-the-money or out-of-the-money.
///
/// # Parameters
///
/// - `option: &Options`
///   A reference to an `Options` struct containing all the relevant parameters for the calculation:
///   - `underlying_price`: The current price of the underlying asset.
///   - `strike_price`: The strike price of the option.
///   - `risk_free_rate`: The annualized risk-free interest rate.
///   - `expiration_date`: The time to expiration of the option, in years.
///   - `implied_volatility`: The implied volatility of the option.
///   - `dividend_yield`: The dividend yield of the underlying asset.
///   - `quantity`: The quantity of the options.
///   - `option_style`: The style of the option (Call or Put).
///
/// # Returns
///
/// - `Ok(Decimal)`: The calculated delta value.
/// - `Err(GreeksError)`: Returns an error if any intermediate calculations fail.
///
/// # Calculation Details
///
/// - If `implied_volatility == 0`, the delta is determined based on whether the option is
///   in-the-money or out-of-the-money:
///   - Call Option:
///     - In-the-money: Delta = `sign`
///     - Out-of-the-money: Delta = 0
///   - Put Option:
///     - In-the-money: Delta = `-sign`
///     - Out-of-the-money: Delta = 0
/// - For options with non-zero implied volatility, the delta is calculated as:
///   - Call Option:
///     \[ \Delta_{\text{call}} = \text{sign} \cdot N(d1) \cdot e^{-qT} \]
///   - Put Option:
///     \[ \Delta_{\text{put}} = \text{sign} \cdot (N(d1) - 1) \cdot e^{-qT} \]
///     Where:
///     - \(N(d1)\): The cumulative distribution function (CDF) of the standard normal distribution evaluated at \(d1\).
///     - \(q\): The dividend yield.
///     - \(T\): Time to expiration.
///
/// - The delta is adjusted by multiplying it by the option quantity.
///
/// # Errors
///
/// - `GreeksError`: If the calculation of \(d1\) or the standard normal CDF (`big_n`) fails.
///
/// # Example
///
/// ```rust
/// use rust_decimal::Decimal;
/// use rust_decimal_macros::dec;
/// use tracing::{error, info};
/// use optionstratlib::constants::ZERO;
/// use optionstratlib::greeks::delta;
/// use optionstratlib::{ExpirationDate, Options};
/// use optionstratlib::model::types::{ OptionStyle, OptionType, Side};
/// use positive::{pos_or_panic, Positive};
/// let option = Options {
///     option_type: OptionType::European,
///     side: Side::Long,
///     underlying_price: Positive::HUNDRED,
///     strike_price: pos_or_panic!(95.0),
///     risk_free_rate: dec!(0.05),
///     expiration_date: ExpirationDate::Days(pos_or_panic!(30.0)),
///     implied_volatility: pos_or_panic!(0.2),
///     dividend_yield: Positive::ZERO,
///     quantity: Positive::ONE,
///     option_style: OptionStyle::Call,
///     underlying_symbol: "AAPL".to_string(),
///     exotic_params: None,
/// };
///
/// match delta(&option) {
///     Ok(result) => info!("Delta: {}", result),
///     Err(e) => error!("Error calculating delta: {:?}", e),
/// }
/// ```
///
/// # Sign convention
///
/// Returns the sensitivity of the **position**: signed by [`Side`] and scaled
/// by `quantity`. A short position reports the negative of the equivalent long.
pub fn delta(option: &Options) -> Result<Decimal, GreeksError> {
    if !matches!(option.option_type, OptionType::European) {
        // The numerical fallback prices through `price_option`, which takes the
        // absolute value, so it returns a per-contract long sensitivity. Sign
        // and scale it here to keep the documented convention.
        return Ok(d_mul(
            crate::greeks::numerical::numerical_delta(option)?,
            signed_quantity(option),
            "greeks::delta::numerical_position_weighted",
        )?);
    }
    let expiration_date = option.expiration_date.get_years()?;

    // For an option when the time to expiration is zero (i.e., at the moment of expiration),
    // the delta takes discrete values based solely on whether the option is In-The-Money (ITM) or
    // Out-of-The-Money (OTM):
    //
    // For a Call option:
    //
    // - **Delta = 1.0** if ITM (underlying price > strike price)
    // - **Delta = 0.0** if OTM (underlying price < strike price)
    //
    // For a Put option:
    //
    // - **Delta = -1.0** if ITM (underlying price < strike price)
    // - **Delta = 0.0** if OTM (underlying price > strike price)
    //
    // In both cases, when the underlying price is exactly equal to the strike price (At-The-Money,
    // ATM), technically, the delta would be **0.5 for Calls** and **-0.5 for Puts**, although this
    // scenario is less common in practice.
    //
    // This happens because at expiration, the option effectively becomes a direct position in the
    // underlying asset (**delta = 1 or -1**) if it is ITM, or has no value (**delta = 0**) if it is OTM.
    if expiration_date == Decimal::ZERO {
        // These arms already carry the side, so they scale by the bare quantity
        // rather than by `signed_quantity`, which would apply it twice.
        let per_contract = match (
            &option.option_style,
            &option.side,
            &option.strike_price,
            &option.underlying_price,
        ) {
            // Call Options
            (OptionStyle::Call, Side::Long, strike, price) if price > strike => Decimal::ONE,
            (OptionStyle::Call, Side::Long, _, _) => Decimal::ZERO,
            (OptionStyle::Call, Side::Short, strike, price) if price > strike => -Decimal::ONE,
            (OptionStyle::Call, Side::Short, _, _) => Decimal::ZERO,

            // Put Options
            (OptionStyle::Put, Side::Long, strike, price) if price < strike => -Decimal::ONE,
            (OptionStyle::Put, Side::Long, _, _) => Decimal::ZERO,
            (OptionStyle::Put, Side::Short, strike, price) if price < strike => Decimal::ONE,
            (OptionStyle::Put, Side::Short, _, _) => Decimal::ZERO,
        };
        return Ok(per_contract * option.quantity.to_dec());
    }

    let sign = if option.is_long() {
        Decimal::ONE
    } else {
        Decimal::NEGATIVE_ONE
    };
    if option.implied_volatility == ZERO {
        // `sign` is already applied here, so scale by the bare quantity.
        let per_contract = match option.option_style {
            OptionStyle::Call => {
                if option.underlying_price >= option.strike_price {
                    sign // Delta is 1 for Call in-the-money
                } else {
                    Decimal::ZERO // Delta is 0 for Call out-of-the-money
                }
            }
            OptionStyle::Put => {
                if option.underlying_price <= option.strike_price {
                    sign * Decimal::NEGATIVE_ONE // Delta is -1 for Put in-the-money
                } else {
                    Decimal::ZERO // Delta is 0 for Put out-of-the-money
                }
            }
        };
        return Ok(per_contract * option.quantity.to_dec());
    }

    let kernels = BlackScholesKernels::new(option, expiration_date)?;
    delta_with(option, &kernels)
}

/// Delta from precomputed kernels. See [`delta`] for the degenerate branches,
/// which carry the discrete values at expiry and at zero volatility.
fn delta_with(option: &Options, k: &BlackScholesKernels) -> Result<Decimal, GreeksError> {
    let sign = if option.is_long() {
        Decimal::ONE
    } else {
        Decimal::NEGATIVE_ONE
    };
    let div_date = k.exp_minus_qt()?;
    let n_d1 = k.big_n_d1()?;
    let delta = match option.option_style {
        OptionStyle::Call => d_mul(sign, n_d1, "greeks::delta::call_sign")?,
        OptionStyle::Put => d_mul(
            sign,
            d_sub(n_d1, Decimal::ONE, "greeks::delta::put_shift")?,
            "greeks::delta::put_sign",
        )?,
    };
    let delta = d_mul(delta, div_date, "greeks::delta::discounted")?;
    let delta: Decimal = delta.clamp(Decimal::NEGATIVE_ONE, Decimal::ONE);
    let quantity: Decimal = option.quantity.into();
    Ok(d_mul(delta, quantity, "greeks::delta::position_weighted")?)
}

/// Computes the gamma of an option.
///
/// Gamma measures the rate of change of the option's delta with respect to changes in the underlying
/// asset's price. It is a second-order derivative of the option price and provides insight into the
/// sensitivity of delta to movements in the underlying price.
///
/// # Parameters
///
/// - `option: &Options`
///   A reference to an `Options` struct containing the following relevant parameters:
///   - `underlying_price`: The current price of the underlying asset.
///   - `strike_price`: The strike price of the option.
///   - `risk_free_rate`: The risk-free interest rate.
///   - `expiration_date`: The time to expiration in years.
///   - `implied_volatility`: The implied volatility of the option.
///   - `dividend_yield`: The dividend yield of the underlying asset.
///   - `quantity`: The quantity of the options.
///
/// # Returns
///
/// - `Ok(Decimal)`: The calculated gamma value.
/// - `Err(GreeksError)`: Returns an error if the computation of `d1` or the probability density function `n(d1)` fails.
///
/// # Calculation
///
/// Gamma is calculated using the formula:
///
/// ```math
/// \Gamma = \frac{e^{-qT} \cdot N'(d1)}{S \cdot \sigma \cdot \sqrt{T}}
/// ```
///
/// Where:
/// - \(N'(d1)\): The standard normal probability density function (PDF) evaluated at \(d1\).
/// - \(S\): The price of the underlying asset.
/// - \(\sigma\): The implied volatility of the option.
/// - \(T\): The time to expiration in years.
/// - \(q\): The dividend yield of the underlying asset.
///
/// ### Steps:
/// 1. Compute \(d1\) using the `d1` function.
/// 2. Evaluate \(N'(d1)\) using the `n` function.
/// 3. Apply the gamma formula, accounting for the effect of the dividend yield \(e^{-qT}\).
/// 4. Multiply the result by the option's quantity.
///
/// # Edge Cases
///
/// - If the implied volatility (\(\sigma\)) is zero, gamma is returned as `0`.
///
/// # Example
///
/// ```rust
/// use rust_decimal_macros::dec;
/// use tracing::{error, info};
/// use optionstratlib::greeks::gamma;
/// use optionstratlib::{ExpirationDate, Options};
/// use optionstratlib::model::types::{ OptionStyle, OptionType, Side};
/// use positive::{pos_or_panic,Positive};
/// let option = Options {
///     option_type: OptionType::European,
///     side: Side::Long,
///     underlying_price: Positive::HUNDRED,
///     strike_price: pos_or_panic!(95.0),
///     risk_free_rate: dec!(0.05),
///     expiration_date: ExpirationDate::Days(pos_or_panic!(30.0)),
///     implied_volatility: pos_or_panic!(0.2),
///     dividend_yield: pos_or_panic!(0.01),
///     quantity: Positive::ONE,
///     option_style: OptionStyle::Call,
///     underlying_symbol: "".to_string(),
///     exotic_params: None,
/// };
///
/// match gamma(&option) {
///     Ok(result) => info!("Gamma: {}", result),
///     Err(e) => error!("Error calculating gamma: {:?}", e),
/// }
/// ```
///
/// # Notes
///
/// - This function assumes that the dividend yield \(q\) and the time to expiration \(T\) are
///   provided in consistent units.
/// - If the implied volatility or time to expiration is very small, the result may be close to 0,
///   as gamma becomes negligible in those cases.
///
/// # Errors
///
/// Returns [`GreeksError::ExpirationDate`] when the option's expiration
/// cannot be converted to a positive year fraction, and propagates any
/// [`GreeksError`] surfaced by `numerical_gamma` for non-European
/// options (typically [`GreeksError::Pricing`] when the perturbation
/// evaluation fails).
///
/// # Sign convention
///
/// Returns the sensitivity of the **position**: signed by [`Side`] and scaled
/// by `quantity`. A short position reports the negative of the equivalent long.
pub fn gamma(option: &Options) -> Result<Decimal, GreeksError> {
    if !matches!(option.option_type, OptionType::European) {
        // Same per-contract long value as the delta fallback; see there.
        return Ok(d_mul(
            crate::greeks::numerical::numerical_gamma(option)?,
            signed_quantity(option),
            "greeks::gamma::numerical_position_weighted",
        )?);
    }
    if option.implied_volatility == ZERO {
        return Ok(Decimal::ZERO);
    }
    let expiration_date: Positive = option.expiration_date.get_years()?;
    if expiration_date == Decimal::ZERO {
        // At expiration, gamma is 0 for all cases
        return Ok(Decimal::ZERO);
    }

    let kernels = BlackScholesKernels::new(option, expiration_date)?;
    gamma_with(option, &kernels)
}

/// Gamma from precomputed kernels. See [`gamma`] for the degenerate branches,
/// which the caller must have ruled out before building the kernels.
fn gamma_with(option: &Options, k: &BlackScholesKernels) -> Result<Decimal, GreeksError> {
    let underlying_price: Decimal = option.underlying_price.into();
    let implied_volatility: Positive = option.implied_volatility;

    let numerator = d_mul(
        k.exp_minus_qt()?,
        k.n_d1()?,
        "greeks::gamma::discounted_pdf",
    )?;
    let denominator = d_mul(
        underlying_price,
        implied_volatility.to_dec(),
        "greeks::gamma::price_vol",
    )?;
    let denominator = d_mul(
        denominator,
        k.sqrt_t().to_dec(),
        "greeks::gamma::price_vol_time",
    )?;
    let gamma: Decimal = d_div(numerator, denominator, "greeks::gamma")?;

    Ok(d_mul(
        gamma,
        signed_quantity(option),
        "greeks::gamma::position_weighted",
    )?)
}

/// Computes the Theta of an option.
///
/// Theta measures the sensitivity of the option's price to time decay, indicating the rate
/// at which the value of the option decreases as the expiration date approaches. This is
/// particularly important in options trading, as Theta reflects the "time decay" of the
/// option's extrinsic value.
///
/// # Parameters
///
/// - `option: &Options`
///   A reference to an `Options` struct containing the following relevant parameters:
///   - `underlying_price`: The current price of the underlying asset.
///   - `strike_price`: The strike price of the option.
///   - `risk_free_rate`: The risk-free interest rate.
///   - `expiration_date`: The time to expiration in years (provides `get_years` method).
///   - `implied_volatility`: The implied volatility of the option.
///   - `dividend_yield`: The dividend yield of the underlying asset.
///   - `option_style`: The style of the option (Call or Put).
///   - `quantity`: The quantity of the options.
///
/// # Returns
///
/// - `Ok(Decimal)`: The calculated Theta value for the option.
/// - `Err(GreeksError)`: Returns an error if any intermediate calculation fails (e.g., in `d1`, `d2`, or `n`).
///
/// # Formula
///
/// The Theta is calculated using the Black-Scholes model. The formula differs for call and put options:
///
/// **Call Options:**
///
/// ```math
/// \Theta_{\text{call}} =
/// -\frac{S \cdot \sigma \cdot e^{-qT} \cdot n(d1)}{2 \sqrt{T}}
/// - r \cdot K \cdot e^{-rT} \cdot N(d2)
/// + q \cdot S \cdot e^{-qT} \cdot N(d1)
/// ```
///
/// **Put Options:**
///
/// ```math
/// \Theta_{\text{put}} =
/// -\frac{S \cdot \sigma \cdot e^{-qT} \cdot n(d1)}{2 \sqrt{T}}
/// + r \cdot K \cdot e^{-rT} \cdot N(-d2)
/// - q \cdot S \cdot e^{-qT} \cdot N(-d1)
/// ```
///
/// Where:
/// - \( S \): Underlying price
/// - \( \sigma \): Implied volatility
/// - \( T \): Time to expiration (in years)
/// - \( r \): Risk-free rate
/// - \( q \): Dividend yield
/// - \( K \): Strike price
/// - \( N(d1) \): Cumulative distribution function (CDF) of the standard normal distribution at \( d1 \).
/// - \( n(d1) \): Probability density function (PDF) of the standard normal distribution at \( d1 \).
///
/// # Calculation Steps
/// 1. Compute \( d1 \) and \( d2 \) using the `d1` and `d2` functions.
/// 2. Calculate the common term:
///    ```math
///    \text{common\_term} = -\frac{S \cdot \sigma \cdot e^{-qT} \cdot n(d1)}{2 \sqrt{T}}
///    ```
/// 3. Apply the corresponding formula for Call or Put options, accounting for the effect of
///    dividends (\( e^{-qT} \)) and risk-free rate (\( e^{-rT} \)).
/// 4. Multiply the resulting Theta by the quantity of options.
///
/// # Example
///
/// ```rust
/// use rust_decimal_macros::dec;
/// use tracing::{error, info};
/// use optionstratlib::greeks::theta;
/// use optionstratlib::{ExpirationDate, Options};
/// use optionstratlib::model::types::{ OptionStyle, OptionType, Side};
/// use positive::{pos_or_panic,Positive};
/// let option = Options {
///     option_type: OptionType::European,
///     side: Side::Long,
///     underlying_price: Positive::HUNDRED,
///     strike_price: pos_or_panic!(95.0),
///     risk_free_rate: dec!(0.05),
///     expiration_date: ExpirationDate::Days(pos_or_panic!(30.0)),
///     implied_volatility: pos_or_panic!(0.2),
///     dividend_yield: pos_or_panic!(0.01),
///     quantity: Positive::ONE,
///     option_style: OptionStyle::Call,
///     underlying_symbol: "".to_string(),
///     exotic_params: None,
/// };
///
/// match theta(&option) {
///     Ok(result) => info!("Theta: {}", result),
///     Err(e) => error!("Error calculating Theta: {:?}", e),
/// }
/// ```
///
/// # Notes
///
/// - A positive Theta means the option gains value as time passes (rare and usually for short positions).
/// - A negative Theta is typical for long positions, as the option loses extrinsic value over time.
/// - If the implied volatility is zero, Theta may be close to zero for far-out-of-the-money options.
///
/// # Errors
///
/// Returns [`GreeksError::ExpirationDate`] when the option's expiration
/// cannot be converted to a positive year fraction, and propagates any
/// [`GreeksError`] surfaced by `numerical_theta` for non-European
/// options.
///
/// # Sign convention
///
/// Returns the sensitivity of the **position**: signed by [`Side`] and scaled
/// by `quantity`. A short position therefore reports a positive theta when it
/// collects decay.
pub fn theta(option: &Options) -> Result<Decimal, GreeksError> {
    let t = option.expiration_date.get_years()?;
    if t == Decimal::ZERO {
        return Ok(Decimal::ZERO);
    }

    let kernels = BlackScholesKernels::new(option, t)?;
    theta_with(option, &kernels)
}

/// Theta from precomputed kernels. See [`theta`] for the degenerate branch.
fn theta_with(option: &Options, kernels: &BlackScholesKernels) -> Result<Decimal, GreeksError> {
    let s = option.underlying_price.to_dec();
    let k = option.strike_price.to_dec();
    let r = option.risk_free_rate;
    let q = option.dividend_yield.to_dec();
    let sigma = option.implied_volatility.to_dec();

    let exp_minus_rt = kernels.exp_minus_rt()?;
    let exp_minus_qt = kernels.exp_minus_qt()?;

    // Common term using n. The e^{-qT} factor discounts the S·n(d1) term to
    // present value (the underlying contributes S·e^{-qT} to the payoff);
    // omitting it made |theta| too large for dividend-paying underlyings.
    let decay = d_mul(exp_minus_qt, s, "greeks::theta::decay_spot")?;
    let decay = d_mul(decay, kernels.n_d1()?, "greeks::theta::decay_pdf")?;
    let decay = d_mul(decay, sigma, "greeks::theta::decay_vol")?;
    let decay_denominator = d_mul(
        Decimal::TWO,
        kernels.sqrt_t().to_dec(),
        "greeks::theta::decay_time",
    )?;
    let common_term = -d_div(decay, decay_denominator, "greeks::theta::decay")?;

    // Rate term: r · K · e^{-rT} · N(±d2); carry term: q · S · e^{-qT} · N(±d1).
    let rate_term = d_mul(r, k, "greeks::theta::rate_strike")?;
    let rate_term = d_mul(rate_term, exp_minus_rt, "greeks::theta::rate_discounted")?;
    let carry_term = d_mul(q, s, "greeks::theta::carry_spot")?;
    let carry_term = d_mul(carry_term, exp_minus_qt, "greeks::theta::carry_discounted")?;

    let theta = match option.option_style {
        OptionStyle::Call => {
            let rate = d_mul(rate_term, kernels.big_n_d2()?, "greeks::theta::call_rate")?;
            let carry = d_mul(carry_term, kernels.big_n_d1()?, "greeks::theta::call_carry")?;
            let theta = d_sub(common_term, rate, "greeks::theta::call_decay_rate")?;
            d_add(theta, carry, "greeks::theta::call")?
        }
        OptionStyle::Put => {
            let rate = d_mul(
                rate_term,
                kernels.big_n_neg_d2()?,
                "greeks::theta::put_rate",
            )?;
            let carry = d_mul(
                carry_term,
                kernels.big_n_neg_d1()?,
                "greeks::theta::put_carry",
            )?;
            let theta = d_add(common_term, rate, "greeks::theta::put_decay_rate")?;
            d_sub(theta, carry, "greeks::theta::put")?
        }
    };

    // Adjust for quantity and convert to daily value (banker's-rounded annualisation).
    let weighted = d_mul(
        theta,
        signed_quantity(option),
        "greeks::theta::position_weighted",
    )?;
    Ok(d_div(
        weighted,
        Decimal::from(365),
        "greeks::theta::per_day",
    )?)
}

/// Computes the vega of an option.
///
/// Vega measures the sensitivity of the option's price to changes in the implied volatility
/// of the underlying asset. It quantifies the expected change in the option's price for a
/// 1% change in the implied volatility. Vega is particularly important for understanding
/// how an option's value is affected by market conditions that alter volatility.
///
/// # Parameters
///
/// - `option: &Options`
///   A reference to an `Options` struct containing the necessary parameters:
///   - `underlying_price`: The current price of the underlying asset.
///   - `strike_price`: The strike price of the option.
///   - `risk_free_rate`: The annualized risk-free interest rate.
///   - `expiration_date`: The time to expiration in years (provides `get_years` method).
///   - `implied_volatility`: The implied volatility of the option.
///   - `dividend_yield`: The dividend yield of the underlying asset.
///   - `quantity`: The quantity of the options.
///   - `option_style`: The style of the option (e.g., European).
///
/// # Returns
///
/// - `Ok(Decimal)`: The computed vega value of the option.
/// - `Err(GreeksError)`: Returns an error if any intermediate calculation fails (e.g., in `d1` or `big_n`).
///
/// # Formula
///
/// Vega is computed using the Black-Scholes model formula:
///
/// ```math
/// \text{Vega} = S \cdot e^{-qT} \cdot n(d1) \cdot \sqrt{T}
/// ```
///
/// Where:
/// - \( S \): The price of the underlying asset.
/// - \( q \): The dividend yield of the underlying asset.
/// - \( T \): Time to expiration in years.
/// - \( n(d1) \): The probability density function (PDF) of the standard normal distribution at \( d1 \).
/// - \( d1 \): A parameter calculated using the Black-Scholes model.
///
/// # Calculation Steps
///
/// 1. Compute \( d1 \) using the `d1` function.
/// 2. Calculate the exponential factor \( e^{-qT} \), which accounts for the effect of dividends.
/// 3. Evaluate \( n(d1) \), the PDF of the standard normal distribution at \( d1 \).
/// 4. Multiply the underlying price, the exponential factor, \( n(d1) \), and the square root of time to expiration.
/// 5. Multiply the result by the quantity of options to adjust for position size.
///
/// # Example
///
/// ```rust
/// use rust_decimal_macros::dec;
/// use tracing::{error, info};
/// use optionstratlib::greeks::vega;
/// use optionstratlib::{ExpirationDate, Options};
/// use optionstratlib::model::types::{ OptionStyle, OptionType, Side};
/// use positive::{pos_or_panic,Positive};
///
/// let option = Options {
///     option_type: OptionType::European,
///     side: Side::Long,
///     underlying_price: Positive::HUNDRED,
///     strike_price: pos_or_panic!(95.0),
///     risk_free_rate: dec!(0.05),
///     expiration_date: ExpirationDate::Days(pos_or_panic!(30.0)),
///     implied_volatility: pos_or_panic!(0.2),
///     dividend_yield: pos_or_panic!(0.01),
///     quantity: Positive::ONE,
///     option_style: OptionStyle::Call,
///     underlying_symbol: "".to_string(),
///     exotic_params: None,
/// };
///
/// match vega(&option) {
///     Ok(result) => info!("Vega: {}", result),
///     Err(e) => error!("Error calculating Vega: {:?}", e),
/// }
/// ```
///
/// # Notes
///
/// - Vega is usually highest for at-the-money options and decreases as the option moves deeper
///   in-the-money or out-of-the-money.
/// - For shorter time to expiration, Vega is smaller as the sensitivity to volatility diminishes.
/// - A positive Vega indicates that an increase in implied volatility will increase the option's value.
///
/// # Errors
///
/// Returns [`GreeksError::ExpirationDate`] when the option's expiration
/// cannot be converted to a positive year fraction, and propagates any
/// [`GreeksError`] surfaced by `numerical_vega` for non-European
/// options.
///
/// # Sign convention
///
/// Returns the sensitivity of the **position**: signed by [`Side`] and scaled
/// by `quantity`. A short position reports the negative of the equivalent long.
pub fn vega(option: &Options) -> Result<Decimal, GreeksError> {
    let expiration_date: Positive = option.expiration_date.get_years()?;
    if expiration_date == Decimal::ZERO {
        // At expiration, volatility has no impact on option price
        return Ok(Decimal::ZERO);
    }
    let kernels = BlackScholesKernels::new(option, expiration_date)?;
    vega_with(option, &kernels)
}

/// Vega from precomputed kernels. See [`vega`] for the degenerate branches.
fn vega_with(option: &Options, k: &BlackScholesKernels) -> Result<Decimal, GreeksError> {
    let underlying_price: Decimal = option.underlying_price.to_dec();

    let vega = d_mul(
        underlying_price,
        k.exp_minus_qt()?,
        "greeks::vega::discounted_spot",
    )?;
    let vega = d_mul(vega, k.n_d1()?, "greeks::vega::pdf")?;
    let vega = d_mul(vega, k.sqrt_t().to_dec(), "greeks::vega::sqrt_time")?;
    // percentage of change in volatility
    let vega: Decimal = d_div(vega, Decimal::ONE_HUNDRED, "greeks::vega::per_percent")?;

    Ok(d_mul(
        vega,
        signed_quantity(option),
        "greeks::vega::position_weighted",
    )?)
}

/// Computes the rho of an options contract.
///
/// Rho measures the sensitivity of the option's price to changes in the risk-free interest rate.
/// It quantifies the expected change in the option's price for a 1% change in the risk-free rate.
/// This metric is useful for understanding how interest rate fluctuations affect the value of
/// options contracts.
///
/// # Parameters
///
/// - `option: &Options`
///   A reference to an `Options` struct containing the following fields:
///   - `underlying_price`: The current price of the underlying asset.
///   - `strike_price`: The strike price of the option.
///   - `risk_free_rate`: The annualized risk-free interest rate.
///   - `expiration_date`: The time to expiration in years (provides `get_years` method).
///   - `implied_volatility`: The implied volatility of the option.
///   - `option_style`: The style of the option (`Call` or `Put`).
///   - `quantity`: The quantity of the options.
///
/// # Returns
///
/// - `Ok(Decimal)`: The computed rho value for the options contract.
/// - `Err(GreeksError)`: Returns an error if any intermediate calculation fails (e.g., in `d2` or `big_n`).
///
/// # Formula
///
/// The rho is calculated differently for Call and Put options, as follows:
///
/// **Call Options:**
///
/// ```math
/// \rho_{\text{call}} = K \cdot T \cdot e^{-rT} \cdot N(d2)
/// ```
///
/// **Put Options:**
///
/// ```math
/// \rho_{\text{put}} = -K \cdot T \cdot e^{-rT} \cdot N(-d2)
/// ```
///
/// Where:
/// - \( K \): The strike price of the option.
/// - \( T \): The time to expiration (in years).
/// - \( r \): The risk-free interest rate.
/// - \( N(d2) \): The cumulative distribution function (CDF) of the standard normal distribution evaluated at \( d2 \).
/// - \( e^{-rT} \): The discount factor for the risk-free rate.
///
/// # Calculation Steps
///
/// 1. Compute \( d2 \) using the `d2` function.
/// 2. Calculate the discount factor \( e^{-rT} \).
/// 3. Evaluate \( N(d2) \) or \( N(-d2) \), depending on the option style.
/// 4. Multiply the strike price, time to expiration, discount factor, and \( N(d2) \) or \( N(-d2) \).
/// 5. Multiply the result by the option's quantity.
///
/// # Edge Cases
///
/// - If the discount factor (\( e^{-rT} \)) is zero, the rho is returned as zero.
/// - If \( N(d2) \) or \( N(-d2) \) is zero, the rho is returned as zero.
///
/// # Example
///
/// ```rust
/// use rust_decimal_macros::dec;
/// use tracing::{error, info};
/// use optionstratlib::greeks::rho;
/// use optionstratlib::{ExpirationDate, Options};
/// use optionstratlib::model::types::{ OptionStyle, OptionType, Side};
/// use positive::{pos_or_panic,Positive};
///
/// let option = Options {
///     option_type: OptionType::European,
///     side: Side::Long,
///     underlying_price: Positive::HUNDRED,
///     strike_price: pos_or_panic!(95.0),
///     risk_free_rate: dec!(0.05),
///     expiration_date: ExpirationDate::Days(pos_or_panic!(30.0)),
///     implied_volatility: pos_or_panic!(0.2),
///     dividend_yield: pos_or_panic!(0.01),
///     quantity: Positive::ONE,
///     option_style: OptionStyle::Call,
///     underlying_symbol: "".to_string(),
///     exotic_params: None,
/// };
///
/// match rho(&option) {
///     Ok(result) => info!("Rho: {}", result),
///     Err(e) => error!("Error calculating rho: {:?}", e),
/// }
/// ```
///
/// # Notes
///
/// - Rho is typically higher for options with longer time to expiration, as they are more
///   sensitive to changes in the risk-free rate.
/// - Call options have positive rho values, as an increase in interest rates increases their value.
/// - Put options have negative rho values, as an increase in interest rates decreases their value.
///
/// # Errors
///
/// Returns [`GreeksError::ExpirationDate`] when the option's expiration
/// cannot be converted to a positive year fraction, and propagates any
/// [`GreeksError`] surfaced by `numerical_rho` for non-European
/// options.
///
/// # Sign convention
///
/// Returns the sensitivity of the **position**: signed by [`Side`] and scaled
/// by `quantity`. A short position reports the negative of the equivalent long.
pub fn rho(option: &Options) -> Result<Decimal, GreeksError> {
    // Get time to expiration first and validate
    let t = option.expiration_date.get_years()?;
    if t == Decimal::ZERO {
        return Ok(Decimal::ZERO);
    }

    let kernels = BlackScholesKernels::new(option, t)?;
    rho_with(option, &kernels)
}

/// Rho from precomputed kernels. See [`rho`] for the degenerate branch.
fn rho_with(option: &Options, kernels: &BlackScholesKernels) -> Result<Decimal, GreeksError> {
    let t = kernels.t();
    let k = option.strike_price.to_dec();

    // Base rho without sign; the discount uses the risk-free rate, not the carry.
    let base_rho = d_mul(k, t.to_dec(), "greeks::rho::strike_time")?;
    let base_rho = d_mul(base_rho, kernels.exp_minus_rt()?, "greeks::rho::discounted")?;

    // Calculate final rho based on option type
    let rho = match option.option_style {
        OptionStyle::Call => d_mul(base_rho, kernels.big_n_d2()?, "greeks::rho::call")?,
        OptionStyle::Put => d_mul(-base_rho, kernels.big_n_neg_d2()?, "greeks::rho::put")?,
    };

    // Adjust for quantity and convert to basis points (banker's rounding).
    let weighted = d_mul(
        rho,
        signed_quantity(option),
        "greeks::rho::position_weighted",
    )?;
    Ok(d_div(
        weighted,
        Decimal::from(100),
        "greeks::rho::per_basis_point",
    )?)
}

/// Computes the sensitivity of the option price to changes in the dividend yield (Rho_d).
///
/// This function calculates how the price of an option changes with respect to variations
/// in the dividend yield of the underlying asset. This metric, often referred to as "dividend rho",
/// is essential for understanding the impact of dividends on the option's value.
///
/// # Parameters
///
/// - `option: &Options`
///   A reference to an `Options` struct containing the following relevant fields:
///   - `underlying_price`: The current price of the underlying asset.
///   - `strike_price`: The strike price of the option.
///   - `risk_free_rate`: The risk-free interest rate.
///   - `expiration_date`: The time to expiration in years (provides `get_years` method).
///   - `implied_volatility`: The implied volatility of the option.
///   - `dividend_yield`: The dividend yield of the underlying asset.
///   - `quantity`: The quantity of the options.
///   - `option_style`: The style of the option (`Call` or `Put`).
///
/// # Returns
///
/// - `Ok(Decimal)`: The calculated dividend sensitivity (`Rho_d`) value for the options contract.
/// - `Err(GreeksError)`: Returns an error if any intermediate calculation fails (e.g., in `d1` or `big_n`).
///
/// # Formula
///
/// The dividend sensitivity is calculated differently for Call and Put options:
///
/// **Call Options:**
///
/// ```math
/// \rho_d^{\text{call}} = -T \cdot S \cdot e^{-qT} \cdot N(d1)
/// ```
///
/// **Put Options:**
///
/// ```math
/// \rho_d^{\text{put}} = T \cdot S \cdot e^{-qT} \cdot N(-d1)
/// ```
///
/// Where:
/// - \( T \): Time to expiration (in years).
/// - \( S \): Price of the underlying asset.
/// - \( q \): Dividend yield.
/// - \( N(d1) \): The cumulative distribution function (CDF) of the standard normal distribution evaluated at \( d1 \).
/// - \( d1 \): A parameter calculated using the Black-Scholes model.
///
/// # Calculation Steps
///
/// 1. Compute \( d1 \) using the `d1` function.
/// 2. Evaluate the exponential factor \( e^{-qT} \), which accounts for the dividend yield.
/// 3. Calculate \( N(d1) \) or \( N(-d1) \), depending on the option style.
/// 4. Use the appropriate formula for Call or Put options.
/// 5. Multiply the result by the option's quantity to adjust for position size.
///
/// # Example
///
/// ```rust
/// use rust_decimal_macros::dec;
/// use tracing::{error, info};
/// use optionstratlib::greeks::rho_d;
/// use optionstratlib::{ExpirationDate, Options};
/// use optionstratlib::model::types::{ OptionStyle, OptionType, Side};
/// use positive::{pos_or_panic, Positive};
///
/// let option = Options {
///     option_type: OptionType::European,
///     side: Side::Long,
///     underlying_price: Positive::HUNDRED,
///     strike_price: pos_or_panic!(95.0),
///     risk_free_rate: dec!(0.05),
///     expiration_date: ExpirationDate::Days(pos_or_panic!(30.0)),
///     implied_volatility: pos_or_panic!(0.2),
///     dividend_yield: pos_or_panic!(0.01),
///     quantity: Positive::ONE,
///     option_style: OptionStyle::Call,
///     underlying_symbol: "".to_string(),
///     exotic_params: None,
/// };
///
/// match rho_d(&option) {
///     Ok(result) => info!("Dividend Rho (Rho_d): {}", result),
///     Err(e) => error!("Error calculating Rho_d: {:?}", e),
/// }
/// ```
///
/// # Notes
///
/// - **Call Options**: A higher dividend yield decreases the price of the call option,
///   leading to a negative dividend sensitivity.
/// - **Put Options**: A higher dividend yield increases the price of the put option,
///   leading to a positive dividend sensitivity.
/// - This calculation assumes that dividends are continuously compounded at the dividend yield rate.
/// - \( Rho_d \) is generally more significant for options with longer times to expiration.
///
/// # Errors
///
/// Returns [`GreeksError::ExpirationDate`] when the option's expiration
/// cannot be converted to a positive year fraction, and propagates any
/// [`GreeksError`] surfaced by intermediate Black–Scholes kernels
/// (typically [`GreeksError::Pricing`] on numerical failure).
///
/// # Sign convention
///
/// Returns the sensitivity of the **position**: signed by [`Side`] and scaled
/// by `quantity`. A short position reports the negative of the equivalent long.
pub fn rho_d(option: &Options) -> Result<Decimal, GreeksError> {
    let expiration_date: Positive = option.expiration_date.get_years()?;
    if expiration_date == Decimal::ZERO {
        // At expiration the dividend yield can no longer move the value.
        return Ok(Decimal::ZERO);
    }
    let kernels = BlackScholesKernels::new(option, expiration_date)?;
    rho_d_with(option, &kernels)
}

/// Dividend rho from precomputed kernels. See [`rho_d`] for the degenerate branch.
fn rho_d_with(option: &Options, k: &BlackScholesKernels) -> Result<Decimal, GreeksError> {
    let expiration_date = k.t();
    let underlying_price: Decimal = option.underlying_price.to_dec();

    let base = d_mul(
        expiration_date.to_dec(),
        underlying_price,
        "greeks::rho_d::time_spot",
    )?;
    let base = d_mul(base, k.exp_minus_qt()?, "greeks::rho_d::discounted")?;

    let rhod = match option.option_style {
        OptionStyle::Call => d_mul(-base, k.big_n_d1()?, "greeks::rho_d::call")?,
        OptionStyle::Put => d_mul(base, k.big_n_neg_d1()?, "greeks::rho_d::put")?,
    };

    let weighted = d_mul(
        rhod,
        signed_quantity(option),
        "greeks::rho_d::position_weighted",
    )?;
    Ok(d_div(
        weighted,
        Decimal::from(100),
        "greeks::rho_d::per_basis_point",
    )?)
}

/// Computes the alpha of an option, the ratio of gamma to theta.
///
/// Alpha expresses how much convexity a position buys per unit of time decay,
/// so it is the natural way to compare two positions that trade one against the
/// other.
///
/// # Sign convention
///
/// Unlike the other eleven, this one is **invariant** under [`Side`]. It is the
/// ratio `gamma / theta`, and a short position negates both, so the ratio is
/// unchanged. That is the correct result, not a missing case.
///
/// # Returns
///
/// `Decimal::ZERO` when gamma vanishes, and the `Decimal::MAX` sentinel when
/// theta vanishes but gamma does not. Callers that publish the value are
/// responsible for mapping that sentinel to something meaningful.
///
/// # Errors
///
/// Propagates any [`GreeksError`] surfaced by [`gamma`] or [`theta`].
pub fn alpha(option: &Options) -> Result<Decimal, GreeksError> {
    let gamma = gamma(option)?;
    let theta = theta(option)?;
    alpha_from(gamma, theta)
}

/// Alpha from a gamma and a theta that have already been computed.
///
/// Split out so the aggregate path does not recompute both from scratch, which
/// was roughly 11% of the cost of a full twelve-greek evaluation.
///
/// Returns `Decimal::MAX` as a sentinel when theta vanishes but gamma does not.
/// Callers that publish the value are responsible for mapping it to something
/// meaningful; see `OptionData::calculate_greeks`.
///
/// # Why the sentinel is still `Decimal::MAX`
///
/// The sentinel used to be a live hazard: [`Greeks::alpha`] and
/// [`Greeks::greeks`] summed it across a strategy's legs with the raw `+`
/// operator, so two legs whose theta vanished aborted the process on
/// `Decimal::MAX + Decimal::MAX`. Both aggregators now refuse to combine the
/// sentinel with another leg's contribution at all, and report which leg
/// carried it; see [`add_alpha`]. That covers the abort and the quieter
/// failure beside it, where `Decimal::MAX` plus a small value rescales back to
/// `Decimal::MAX` and drops the other leg without any arithmetic error.
///
/// Nothing prevents a caller from reaching the sentinel, so it is described
/// here rather than declared impossible: no constructor bounds the quantity
/// away from it. An ordinary at-the-money contract held at a quantity around
/// `1e-27` has a daily theta that rounds to zero at `Decimal`'s
/// twenty-eight-digit scale while its gamma still rounds to `1e-28`, which is
/// the state this branch answers.
///
/// Replacing the sentinel would change the documented return of the public
/// [`alpha`] and ripple into `OptionData::calculate_greeks`, which already maps
/// it to `None`, without buying any safety the guarded aggregation does not
/// already provide.
///
/// # Errors
///
/// Returns [`GreeksError`] when `gamma / theta` overflows the `Decimal`
/// range; the raw operator would panic instead.
fn alpha_from(gamma: Decimal, theta: Decimal) -> Result<Decimal, GreeksError> {
    match (gamma, theta) {
        (val, _) if val == Decimal::ZERO => Ok(Decimal::ZERO),
        (_, val) if val == Decimal::ZERO => Ok(Decimal::MAX),
        _ => Ok(d_div(gamma, theta, "greeks::alpha")?),
    }
}

/// Combines one leg's alpha into a running total, refusing the `Decimal::MAX`
/// sentinel wherever summing it would lose a contribution.
///
/// [`alpha_from`] answers `Decimal::MAX` for a leg whose theta has vanished.
/// That value cannot be added to a real one: `Decimal::MAX` plus anything of
/// magnitude below one rescales and rounds straight back to `Decimal::MAX`, so
/// `checked_add` answers `Some` with the accumulator standing still and the
/// other leg's alpha silently dropped. `checked_add` cannot detect that — the
/// arithmetic genuinely succeeded — so the guard is explicit rather than
/// arithmetic, and it runs before [`d_add`] is reached.
///
/// Refused, therefore:
///
/// - a sentinel leg meeting a total that already carries a contribution, and
/// - a contribution meeting a total that is already the sentinel,
///
/// which between them cover a sentinel beside an ordinary leg in either order,
/// and two sentinels (which would also have left the representable range).
///
/// Allowed, because nothing is lost: a sentinel that is the only leg, and a
/// sentinel beside legs whose own alpha is zero — the value `alpha` returns
/// for a vanished gamma. A one-leg aggregate therefore still reports the
/// documented sentinel, which is what `OptionData::greeks_snapshot` reads to
/// map the published `alpha` to `None`.
///
/// Only the two alpha accumulation paths use this. None of the other eleven
/// greeks has a sentinel, so a `Decimal::MAX` among those is a real value that
/// must sum, or overflow, normally.
///
/// # Errors
///
/// Returns [`GreeksError::CalculationError`] naming the offending leg when the
/// sentinel would meet a contribution, and
/// [`crate::error::DecimalError::Overflow`] when an ordinary sum leaves the
/// representable range.
#[inline]
fn add_alpha(
    total: Decimal,
    value: Decimal,
    option: &Options,
    index: usize,
    op: &'static str,
) -> Result<Decimal, GreeksError> {
    let sentinel_meets_a_contribution = (value == Decimal::MAX && total != Decimal::ZERO)
        || (total == Decimal::MAX && value != Decimal::ZERO);
    if sentinel_meets_a_contribution {
        return Err(alpha_sentinel_error(option, index));
    }
    Ok(d_add(total, value, op)?)
}

/// Builds the error [`add_alpha`] returns, naming the leg that carries the
/// sentinel.
///
/// Filed under [`CalculationErrorKind::ThetaError`] because a vanished theta is
/// the cause; there is no alpha-specific variant, and this matches how
/// `greeks::numerical` reports the same family of failures.
#[cold]
#[inline(never)]
fn alpha_sentinel_error(option: &Options, index: usize) -> GreeksError {
    GreeksError::CalculationError(CalculationErrorKind::ThetaError {
        reason: format!(
            "leg {index} ({symbol} {strike} {style:?} {side:?}) has a vanished theta, \
             so its alpha is the Decimal::MAX sentinel and cannot be summed into an \
             aggregate; read the leg's own alpha instead",
            symbol = option.underlying_symbol,
            strike = option.strike_price,
            style = option.option_style,
            side = option.side,
        ),
    })
}

/// Computes the vanna of an option.
///
/// Vanna measures the rate of change of delta in relation to changes in implied volatility.
/// It is a second-order derivative of the option value and can be useful to help the trader
/// to anticipate changes to the effectiveness of a delta-hedge as volatility changes.
///
/// # Parameters
///
/// - `option: &Options`
///   A reference to an `Options` struct containing the following relevant parameters:
///   - `underlying_price`: The current price of the underlying asset.
///   - `strike_price`: The strike price of the option.
///   - `risk_free_rate`: The risk-free interest rate.
///   - `expiration_date`: The time to expiration in years.
///   - `implied_volatility`: The implied volatility of the option.
///   - `dividend_yield`: The dividend yield of the underlying asset.
///   - `quantity`: The quantity of the options.
///
/// # Returns
///
/// - `Ok(Decimal)`: The calculated vanna value.
/// - `Err(GreeksError)`: Returns an error if the computation of `d2` or the probability density function `n(d1)` fails.
///
/// # Calculation
///
/// Vanna is calculated using the formula:
///
/// ```math
/// \text{Vanna} = -e^{-qT} \cdot N'(d1) \cdot \frac {d2}{\sigma}
/// ```
///
/// Where:
/// - \(N'(d1)\): The standard normal probability density function (PDF) evaluated at \(d1\).
/// - \(\sigma\): The implied volatility of the option.
/// - \(T\): The time to expiration in years.
/// - \(q\): The dividend yield of the underlying asset.
///
/// ### Steps:
/// 1. Compute \(d1\) using the `d1` function.
/// 2. Compute \(d2\) using the `d2` function.
/// 3. Evaluate \(N'(d1)\) using the `n` function.
/// 4. Compute the effect of the dividend yield \(-e^{-qT}\).
/// 5. Apply the vanna formula and divide by the implied_volatility (\(\sigma\)).
/// 6. Multiply the result by the option's quantity.
///
/// # Edge Cases
///
/// - If the implied volatility (\(\sigma\)) is zero, vanna is returned as `0`.
///
/// # Example
///
/// ```rust
/// use rust_decimal_macros::dec;
/// use tracing::{error, info};
/// use optionstratlib::greeks::vanna;
/// use optionstratlib::{ExpirationDate, Options};
/// use optionstratlib::model::types::{ OptionStyle, OptionType, Side};
/// use positive::{pos_or_panic,Positive};
/// let option = Options {
///     option_type: OptionType::European,
///     side: Side::Long,
///     underlying_price: Positive::HUNDRED,
///     strike_price: pos_or_panic!(95.0),
///     risk_free_rate: dec!(0.05),
///     expiration_date: ExpirationDate::Days(pos_or_panic!(30.0)),
///     implied_volatility: pos_or_panic!(0.2),
///     dividend_yield: pos_or_panic!(0.01),
///     quantity: Positive::ONE,
///     option_style: OptionStyle::Call,
///     underlying_symbol: "".to_string(),
///     exotic_params: None,
/// };
///
/// match vanna(&option) {
///     Ok(result) => info!("Vanna: {}", result),
///     Err(e) => error!("Error calculating vanna: {:?}", e),
/// }
/// ```
///
/// # Notes
///
/// - This function assumes that the dividend yield \(q\) and the time to expiration \(T\) are
///   provided in consistent units.
///
/// # Errors
///
/// Returns [`GreeksError::ExpirationDate`] when the option's expiration
/// cannot be converted to a positive year fraction, and propagates any
/// [`GreeksError`] surfaced by the underlying Black–Scholes
/// evaluation (typically [`GreeksError::Pricing`]).
///
/// # Sign convention
///
/// Returns the sensitivity of the **position**: signed by [`Side`] and scaled
/// by `quantity`. A short position reports the negative of the equivalent long.
pub fn vanna(option: &Options) -> Result<Decimal, GreeksError> {
    if option.implied_volatility == ZERO {
        return Ok(Decimal::ZERO);
    }

    let expiration_date: Positive = option.expiration_date.get_years()?;
    if expiration_date == Decimal::ZERO {
        // At expiration delta is a step function and no longer responds to
        // volatility, so vanna is zero.
        return Ok(Decimal::ZERO);
    }
    let kernels = BlackScholesKernels::new(option, expiration_date)?;
    vanna_with(option, &kernels)
}

/// Vanna from precomputed kernels. See [`vanna`] for the degenerate branches.
fn vanna_with(option: &Options, k: &BlackScholesKernels) -> Result<Decimal, GreeksError> {
    let implied_volatility: Positive = option.implied_volatility;
    // Discount factor e^{-qT} for the dividend-adjusted underlying term.
    // vanna = dDelta/dsigma = e^{-qT} * n(d1) * (dd1/dsigma), and the standard
    // result dd1/dsigma = -d2/sigma introduces an explicit minus sign, so the
    // closed form is -e^{-qT} * n(d1) * d2 / sigma (sign is opposite to d2).
    let standardised_d2 = d_div(
        k.d2()?,
        implied_volatility.to_dec(),
        "greeks::vanna::d2_over_sigma",
    )?;
    let vanna = d_mul(
        k.exp_minus_qt()?,
        k.n_d1()?,
        "greeks::vanna::discounted_pdf",
    )?;
    let vanna: Decimal = -d_mul(vanna, standardised_d2, "greeks::vanna")?;

    Ok(d_mul(
        vanna,
        signed_quantity(option),
        "greeks::vanna::position_weighted",
    )?)
}

/// Computes the vomma of an option.
///
/// Vomma (aka volga, vega convexity or DvegaDvol) measures the second order
/// sensitivity to volatility. Is the second derivative of the option value
/// with respect to the volatility. Stated in another way, vomma measures
/// the rate of change to vega as volatility changes.
///
/// # Parameters
///
/// - `option: &Options`
///   A reference to an `Options` struct containing the necessary parameters:
///   - `underlying_price`: The current price of the underlying asset.
///   - `strike_price`: The strike price of the option.
///   - `risk_free_rate`: The annualized risk-free interest rate.
///   - `expiration_date`: The time to expiration in years (provides `get_years` method).
///   - `implied_volatility`: The implied volatility of the option.
///   - `dividend_yield`: The dividend yield of the underlying asset.
///   - `quantity`: The quantity of the options.
///   - `option_style`: The style of the option (e.g., European).
///
/// # Returns
///
/// - `Ok(Decimal)`: The computed vomma value of the option.
/// - `Err(GreeksError)`: Returns an error if any intermediate calculation fails (e.g., in `d1` or `d2`).
///
/// # Formula
///
/// Vomma is computed using the Black-Scholes model formula:
///
/// ```math
/// \text{Vomma} = {Vega} \cdot \frac{d1 \cdot d2}{\sigma}
/// ```
///
/// Where:
/// - \( Vega \): The option's Vega value.
/// - \( d1 \): A parameter calculated using the Black-Scholes model.
/// - \( d2 \): A parameter calculated using the Black-Scholes model.
/// - \(\sigma\): The implied volatility of the option.
///
/// # Calculation Steps
///
/// 1. Compute \( Vega \) using the `vega` function.
/// 2. Compute \( d1 \) using the `d1` function.
/// 3. Compute \( d2 \) using the `d2` function.
/// 4. Compute Vomma and multiply the result by the quantity of options to adjust for position size.
///
/// # Example
///
/// ```rust
/// use rust_decimal_macros::dec;
/// use tracing::{error, info};
/// use optionstratlib::greeks::vomma;
/// use optionstratlib::{ExpirationDate, Options};
/// use optionstratlib::model::types::{ OptionStyle, OptionType, Side};
/// use positive::{pos_or_panic,Positive};
///
/// let option = Options {
///     option_type: OptionType::European,
///     side: Side::Long,
///     underlying_price: Positive::HUNDRED,
///     strike_price: pos_or_panic!(95.0),
///     risk_free_rate: dec!(0.05),
///     expiration_date: ExpirationDate::Days(pos_or_panic!(30.0)),
///     implied_volatility: pos_or_panic!(0.2),
///     dividend_yield: pos_or_panic!(0.01),
///     quantity: Positive::ONE,
///     option_style: OptionStyle::Call,
///     underlying_symbol: "".to_string(),
///     exotic_params: None,
/// };
///
/// match vomma(&option) {
///     Ok(result) => info!("Vomma: {}", result),
///     Err(e) => error!("Error calculating Vomma: {:?}", e),
/// }
/// ```
/// # Notes
///
/// Options far out-of-the money have the highest Vomma.
/// If you are long options you typically want to have as high positive Vomma
/// as possible. If short options, you typically want negative Vomma.
/// Positive Vomma tells you that you will earn more for every percentage point
/// increase in volatility, and if implied volatility is falling you will lose
/// less and less.
/// If you think the implied volatility will be volatile in the short term
/// you should typically try to find options with high Vomma.
///
/// # Errors
///
/// Returns [`GreeksError::ExpirationDate`] when the option's expiration
/// cannot be converted to a positive year fraction, and propagates any
/// [`GreeksError`] surfaced by the underlying Black–Scholes
/// evaluation.
///
/// # Sign convention
///
/// Returns the sensitivity of the **position**. The sign and the size are
/// inherited from [`vega`], which this is a derivative of, so neither is
/// applied a second time here.
pub fn vomma(option: &Options) -> Result<Decimal, GreeksError> {
    let expiration_date: Positive = option.expiration_date.get_years()?;
    if expiration_date == Decimal::ZERO {
        // At expiration, volatility has no impact on option price
        return Ok(Decimal::ZERO);
    }
    let kernels = BlackScholesKernels::new(option, expiration_date)?;
    vomma_with(option, &kernels)
}

/// Vomma from precomputed kernels. See [`vomma`] for the degenerate branch.
fn vomma_with(option: &Options, k: &BlackScholesKernels) -> Result<Decimal, GreeksError> {
    // `vega` already carries `option.quantity`. Vomma is a first derivative of
    // vega and is linear in position size, so the quantity must not be applied
    // a second time here.
    let vega = vega_with(option, k)?;
    let implied_volatility: Positive = option.implied_volatility;

    // `d1 · d2 / sigma` is the site that aborted a live request: a volatility
    // approaching zero pushes the quotient out of `Decimal`'s range, and both
    // the multiplication and the division panic there.
    let d1_d2 = d_mul(k.d1(), k.d2()?, "greeks::vomma::d1_d2")?;
    let scaled = d_div(
        d1_d2,
        implied_volatility.to_dec(),
        "greeks::vomma::d1_d2_over_sigma",
    )?;

    Ok(d_mul(vega, scaled, "greeks::vomma")?)
}

/// Computes the veta of an option.
///
/// Veta measures the rate of change to vega as time changes.
///
/// # Parameters
///
/// - `option: &Options`
///   A reference to an `Options` struct containing the necessary parameters:
///   - `underlying_price`: The current price of the underlying asset.
///   - `strike_price`: The strike price of the option.
///   - `risk_free_rate`: The annualized risk-free interest rate.
///   - `expiration_date`: The time to expiration in years (provides `get_years` method).
///   - `implied_volatility`: The implied volatility of the option.
///   - `dividend_yield`: The dividend yield of the underlying asset.
///   - `quantity`: The quantity of the options.
///   - `option_style`: The style of the option (e.g., European).
///
/// # Returns
///
/// - `Ok(Decimal)`: The computed veta value of the option.
/// - `Err(GreeksError)`: Returns an error if any intermediate calculation fails (e.g., in `d1` or `d2`).
///
/// # Formula
///
/// Veta is computed using the Black-Scholes model formula:
///
/// ```math
/// \text{Veta} = {-Vega} \left [q+\frac {(r-q)d1}{\sigma\sqrt T} -\frac {1 + d1 d2}{2T} \right]
/// ```
///
/// Where:
/// - \( Vega \): The option's Vega value.
/// - \( d1 \): A parameter calculated using the Black-Scholes model.
/// - \( d2 \): A parameter calculated using the Black-Scholes model.
/// - \(\sigma\): The implied volatility of the option.
///
/// # Calculation Steps
///
/// 1. Compute \( Vega \) using the `vega` function.
/// 2. Compute \( d1 \) using the `d1` function.
/// 3. Compute \( d2 \) using the `d2` function.
/// 4. Compute Veta and multiply the result by the quantity of options to adjust for position size.
///
/// # Example
///
/// ```rust
/// use rust_decimal_macros::dec;
/// use tracing::{error, info};
/// use optionstratlib::greeks::veta;
/// use optionstratlib::{ExpirationDate, Options};
/// use optionstratlib::model::types::{ OptionStyle, OptionType, Side};
/// use positive::{pos_or_panic,Positive};
///
/// let option = Options {
///     option_type: OptionType::European,
///     side: Side::Long,
///     underlying_price: Positive::HUNDRED,
///     strike_price: pos_or_panic!(95.0),
///     risk_free_rate: dec!(0.05),
///     expiration_date: ExpirationDate::Days(pos_or_panic!(30.0)),
///     implied_volatility: pos_or_panic!(0.2),
///     dividend_yield: pos_or_panic!(0.01),
///     quantity: Positive::ONE,
///     option_style: OptionStyle::Call,
///     underlying_symbol: "".to_string(),
///     exotic_params: None,
/// };
///
/// match veta(&option) {
///     Ok(result) => info!("Veta: {}", result),
///     Err(e) => error!("Error calculating Veta: {:?}", e),
/// }
/// ```
/// # Notes
///
/// - It is common practice to divide the mathematical result of veta by 100 times
///   the number of days per year to reduce the value to the percentage change in
///   vega per one day.
///
/// # Errors
///
/// Returns [`GreeksError::ExpirationDate`] when the option's expiration
/// cannot be converted to a positive year fraction, and propagates any
/// [`GreeksError`] surfaced by the underlying Black–Scholes
/// evaluation.
///
/// # Sign convention
///
/// Returns the sensitivity of the **position**. The sign and the size are
/// inherited from [`vega`], which this is a derivative of, so neither is
/// applied a second time here.
pub fn veta(option: &Options) -> Result<Decimal, GreeksError> {
    let expiration_date: Positive = option.expiration_date.get_years()?;
    if expiration_date == Decimal::ZERO {
        // At expiration, volatility has no impact on option price
        return Ok(Decimal::ZERO);
    }
    let kernels = BlackScholesKernels::new(option, expiration_date)?;
    veta_with(option, &kernels)
}

/// Veta from precomputed kernels. See [`veta`] for the degenerate branch.
fn veta_with(option: &Options, k: &BlackScholesKernels) -> Result<Decimal, GreeksError> {
    let expiration_date = k.t();
    let vega = vega_with(option, k)?;
    let implied_volatility: Positive = option.implied_volatility;
    let dividend_yield: Decimal = option.dividend_yield.into();
    let risk_free_rate: Decimal = option.risk_free_rate;
    let carry = d_sub(risk_free_rate, dividend_yield, "greeks::veta::carry")?;
    let add1_numerator = d_mul(carry, k.d1(), "greeks::veta::carry_d1")?;
    let add1_denominator = d_mul(
        implied_volatility.to_dec(),
        k.sqrt_t().to_dec(),
        "greeks::veta::vol_time",
    )?;
    let add1 = d_div(add1_numerator, add1_denominator, "greeks::veta::carry_term")?;

    let d1_d2 = d_mul(k.d1(), k.d2()?, "greeks::veta::d1_d2")?;
    let add2_numerator = d_add(Decimal::ONE, d1_d2, "greeks::veta::one_plus_d1_d2")?;
    let add2_denominator = d_mul(
        Decimal::TWO,
        expiration_date.to_dec(),
        "greeks::veta::two_tau",
    )?;
    let add2 = d_div(add2_numerator, add2_denominator, "greeks::veta::time_term")?;

    let bracket = d_add(dividend_yield, add1, "greeks::veta::bracket_carry")?;
    let bracket = d_sub(bracket, add2, "greeks::veta::bracket")?;
    let veta: Decimal = d_mul(-vega, bracket, "greeks::veta")?;
    // It is common practice to divide the mathematical result of veta by
    // 100 times the number of days per year to reduce the value to the
    // percentage change in vega per one day
    // `vega` already carries `option.quantity`. Veta is a first derivative of
    // vega and is linear in position size, so the quantity must not be applied
    // a second time here.
    let scale = d_mul(
        TRADING_DAYS.to_dec(),
        Decimal::ONE_HUNDRED,
        "greeks::veta::scale",
    )?;
    Ok(d_div(veta, scale, "greeks::veta::per_day_percent")?)
}

/// Computes the Charm of an option.
///
/// Charm, also known as DdeltaDtime or Delta decay, measures the sensitivity of
/// the option's delta to time decay. The mathematical result of the formula for
/// charm is expressed in delta per year. It is useful to divide this by the
/// number of days per year to arrive at the delta decay per day. This usage is
/// fairly accurate when the number of days remaining until option expiration is
/// large. When an option nears expiration, charm itsel may change quickly,
/// rendering full day estimate of delta decay inaccurate.
///
/// # Parameters
///
/// - `option: &Options`
///   A reference to an `Options` struct containing the following relevant parameters:
///   - `underlying_price`: The current price of the underlying asset.
///   - `strike_price`: The strike price of the option.
///   - `risk_free_rate`: The risk-free interest rate.
///   - `expiration_date`: The time to expiration in years (provides `get_years` method).
///   - `implied_volatility`: The implied volatility of the option.
///   - `dividend_yield`: The dividend yield of the underlying asset.
///   - `option_style`: The style of the option (Call or Put).
///   - `quantity`: The quantity of the options.
///
/// # Returns
///
/// - `Ok(Decimal)`: The calculated Charm value for the option.
/// - `Err(GreeksError)`: Returns an error if any intermediate calculation fails
///
/// # Formula
///
/// The Charm is calculated using the Black-Scholes model. The formula differs
/// for call and put options:
///
/// **Call Options:**
///
/// ```math
/// \text{Charm}_{\text{Call}} =
/// qe^{-q\tau}N(d_{\text{1}})
/// -e^{-q\tau}n(d_{\text{1}})
/// \frac{2(r-q)\tau-d_{\text{2}}\sigma\sqrt{\tau}}
/// {2\tau\sigma\sqrt{\tau}}
/// ```
///
/// **Put Options:**
///
/// ```math
/// \text{Charm}_{\text{Put}} =
/// -qe^{-q\tau}N(-d_{\text{1}})
/// -e^{-q\tau}n(d_{\text{1}})
/// \frac{2(r-q)\tau-d_{\text{2}}\sigma\sqrt{\tau}}
/// {2\tau\sigma\sqrt{\tau}}
/// ```
///
/// Where:
/// - \( S \): Underlying price
/// - \( \sigma \): Implied volatility
/// - \( \tau \): Time to expiration (in years)
/// - \( r \): Risk-free rate
/// - \( q \): Dividend yield
/// - \( K \): Strike price
/// - \( N(d1) \): Cumulative distribution function (CDF) of the standard normal
///   distribution at \( d1 \).
/// - \( n(d1) \): Probability density function (PDF) of the standard normal
///   distribution at \( d1 \).
///
/// # Calculation Steps
/// 1. Compute \( d1 \) and \( d2 \) using the `d1` and `d2` functions.
/// 2. Calculate the common term:
///    ```math
///    \text{common\_term} =
///    \frac{2(r-q)\tau-d_{\text{2}}\sigma\sqrt{\tau}}
///    {2\tau\sigma\sqrt{\tau}}
///    ```
/// 3. Apply the corresponding formula for Call or Put options, accounting for
///    the effect of dividends (\( e^{-q\tau} \)).
/// 4. Multiply the resulting Charm by the quantity of options.
///
/// # Example
///
/// ```rust
/// use rust_decimal_macros::dec;
/// use tracing::{error, info};
/// use optionstratlib::greeks::charm;
/// use optionstratlib::{ExpirationDate, Options};
/// use optionstratlib::model::types::{OptionStyle, OptionType, Side};
/// use positive::{pos_or_panic, Positive};
/// let option = Options {
///     option_type: OptionType::European,
///     side: Side::Long,
///     underlying_price: Positive::HUNDRED,
///     strike_price: pos_or_panic!(95.0),
///     risk_free_rate: dec!(0.05),
///     expiration_date: ExpirationDate::Days(pos_or_panic!(30.0)),
///     implied_volatility: pos_or_panic!(0.2),
///     dividend_yield: pos_or_panic!(0.01),
///     quantity: Positive::ONE,
///     option_style: OptionStyle::Call,
///     underlying_symbol: "".to_string(),
///     exotic_params: None,
/// };
///
/// match charm(&option) {
///     Ok(result) => info!("Charm: {}", result),
///     Err(e) => error!("Error calculating Charm: {:?}", e),
/// }
/// ```
///
/// # Notes
///
/// - With zero DTE Charm can be considered as zero.
/// - Charm effects are more pronounced near expiration.
///
/// # Errors
///
/// Returns [`GreeksError::ExpirationDate`] when the option's expiration
/// cannot be converted to a positive year fraction, and propagates any
/// [`GreeksError`] surfaced by intermediate Black–Scholes kernels.
///
/// # Sign convention
///
/// Returns the sensitivity of the **position**: signed by [`Side`] and scaled
/// by `quantity`. A short position reports the negative of the equivalent long.
pub fn charm(option: &Options) -> Result<Decimal, GreeksError> {
    let tau = option.expiration_date.get_years()?;
    // if DTE is zero we can assume Charm is also zero
    if tau == Decimal::ZERO {
        return Ok(Decimal::ZERO);
    }

    let kernels = BlackScholesKernels::new(option, tau)?;
    charm_with(option, &kernels)
}

/// Charm from precomputed kernels. See [`charm`] for the degenerate branch.
fn charm_with(option: &Options, k: &BlackScholesKernels) -> Result<Decimal, GreeksError> {
    let tau = k.t();
    let r = option.risk_free_rate;
    let q = option.dividend_yield.to_dec();
    let sigma = option.implied_volatility;
    let exp_minus_qt = k.exp_minus_qt()?;

    // common_term = (2(r-q)τ − d2·σ·√τ) / (2τ·σ·√τ)
    let carry = d_sub(r, q, "greeks::charm::carry")?;
    let carry_term = d_mul(Decimal::TWO, carry, "greeks::charm::two_carry")?;
    let carry_term = d_mul(carry_term, tau.to_dec(), "greeks::charm::carry_tau")?;
    let vol_time = d_mul(
        sigma.to_dec(),
        k.sqrt_t().to_dec(),
        "greeks::charm::vol_time",
    )?;
    let d2_term = d_mul(k.d2()?, vol_time, "greeks::charm::d2_vol_time")?;
    let numerator = d_sub(carry_term, d2_term, "greeks::charm::numerator")?;
    let denominator = d_mul(Decimal::TWO, tau.to_dec(), "greeks::charm::two_tau")?;
    let denominator = d_mul(denominator, vol_time, "greeks::charm::denominator")?;
    let common_term = d_div(numerator, denominator, "greeks::charm::common_term")?;

    let pdf_term = d_mul(exp_minus_qt, k.n_d1()?, "greeks::charm::discounted_pdf")?;
    let pdf_term = d_mul(pdf_term, common_term, "greeks::charm::pdf_common")?;

    let charm = match option.option_style {
        OptionStyle::Call => {
            let carry_leg = d_mul(q, exp_minus_qt, "greeks::charm::call_carry")?;
            let carry_leg = d_mul(carry_leg, k.big_n_d1()?, "greeks::charm::call_carry_cdf")?;
            d_sub(carry_leg, pdf_term, "greeks::charm::call")?
        }
        OptionStyle::Put => {
            let carry_leg = d_mul(-q, exp_minus_qt, "greeks::charm::put_carry")?;
            let carry_leg = d_mul(carry_leg, k.big_n_neg_d1()?, "greeks::charm::put_carry_cdf")?;
            d_sub(carry_leg, pdf_term, "greeks::charm::put")?
        }
    };
    // Adjust for quantity and convert to daily value.
    let weighted = d_mul(
        charm,
        signed_quantity(option),
        "greeks::charm::position_weighted",
    )?;
    Ok(d_div(
        weighted,
        Decimal::from(365),
        "greeks::charm::per_day",
    )?)
}

/// Computes the Color of an option.
///
/// Color, also known as DgammaDtime or Gamma decay, measures the sensitivity of
/// the option's gamma to time decay. The mathematical result of the formula for
/// color is expressed in gamma per year. It is useful to divide this by the
/// number of days per year to arrive at the gamma decay per day. This usage is
/// fairly accurate when the number of days remaining until option expiration is
/// large. When an option nears expiration, color itsel may change quickly,
/// rendering full days estimate of gamma decay inaccurate.
///
/// # Parameters
///
/// - `option: &Options`
///   A reference to an `Options` struct containing the following relevant parameters:
///   - `underlying_price`: The current price of the underlying asset.
///   - `strike_price`: The strike price of the option.
///   - `risk_free_rate`: The risk-free interest rate.
///   - `expiration_date`: The time to expiration in years (provides `get_years` method).
///   - `implied_volatility`: The implied volatility of the option.
///   - `dividend_yield`: The dividend yield of the underlying asset.
///   - `option_style`: The style of the option (Call or Put).
///   - `quantity`: The quantity of the options.
///
/// # Returns
///
/// - `Ok(Decimal)`: The calculated Color value for the option.
/// - `Err(GreeksError)`: Returns an error if any intermediate calculation fails
///
/// # Formula
///
/// The Color is calculated using the Black-Scholes model. The formula is the
/// same for call and put options:
///
/// ```math
/// \text{Color} = -e^{-q\tau}
/// \frac{n(d_{\text{1}})}
/// {2S\tau\sigma\sqrt{\tau}}
/// \left[
/// 2q\tau+1+
/// \frac{2(r-q)\tau-d_{\text{2}}\sigma\sqrt{\tau}}
/// {\sigma\sqrt{\tau}}d_{\text{1}}
/// \right]
/// ```
///
/// Where:
/// - \( S \): Underlying price
/// - \( \sigma \): Implied volatility
/// - \( \tau \): Time to expiration (in years)
/// - \( r \): Risk-free rate
/// - \( q \): Dividend yield
/// - \( n(d1) \): Probability density function (PDF) of the standard normal
///   distribution at \( d1 \).
///
/// # Calculation Steps
/// 1. Compute \( d1 \) and \( d2 \) using the `d1` and `d2` functions.
/// 2. Compute \( n(d1) \) using the `n` function.
/// 3. Apply the corresponding Color formula, accounting for
///    the effect of dividends (\( e^{-q\tau} \)).
/// 4. Multiply the resulting Color value by the quantity of options.
///
/// # Example
///
/// ```rust
/// use rust_decimal_macros::dec;
/// use tracing::{error, info};
/// use optionstratlib::greeks::color;
/// use optionstratlib::{ExpirationDate, Options};
/// use optionstratlib::model::types::{ OptionStyle, OptionType, Side};
/// use positive::{pos_or_panic,Positive};
/// let option = Options {
///     option_type: OptionType::European,
///     side: Side::Long,
///     underlying_price: Positive::HUNDRED,
///     strike_price: pos_or_panic!(95.0),
///     risk_free_rate: dec!(0.05),
///     expiration_date: ExpirationDate::Days(pos_or_panic!(30.0)),
///     implied_volatility: pos_or_panic!(0.2),
///     dividend_yield: pos_or_panic!(0.01),
///     quantity: Positive::ONE,
///     option_style: OptionStyle::Call,
///     underlying_symbol: "".to_string(),
///     exotic_params: None,
/// };
///
/// match color(&option) {
///     Ok(result) => info!("Color: {}", result),
///     Err(e) => error!("Error calculating Color: {:?}", e),
/// }
/// ```
///
/// # Notes
/// - Color is generally negative for long options and positive for short options.
/// - Color will be more pronounced as expiration date approaches.
/// - When volatility increases Color sensitivity decrease.
/// - Deep ITM and OTM options have negligible Color.
///
/// # Errors
///
/// Returns [`GreeksError::ExpirationDate`] when the option's expiration
/// cannot be converted to a positive year fraction, and propagates any
/// [`GreeksError`] surfaced by intermediate Black–Scholes kernels.
///
/// # Sign convention
///
/// Returns the sensitivity of the **position**: signed by [`Side`] and scaled
/// by `quantity`. A short position reports the negative of the equivalent long.
pub fn color(option: &Options) -> Result<Decimal, GreeksError> {
    let tau = option.expiration_date.get_years()?;
    // if DTE is zero we can assume Color is also zero
    if tau == Decimal::ZERO {
        return Ok(Decimal::ZERO);
    }

    let kernels = BlackScholesKernels::new(option, tau)?;
    color_with(option, &kernels)
}

/// Color from precomputed kernels. See [`color`] for the degenerate branch.
fn color_with(option: &Options, k: &BlackScholesKernels) -> Result<Decimal, GreeksError> {
    let tau = k.t();
    let r = option.risk_free_rate;
    let s = option.underlying_price;
    let q = option.dividend_yield.to_dec();
    let sigma = option.implied_volatility;
    let exp_minus_qt = k.exp_minus_qt()?;

    // factor1 = n(d1) / (2·S·τ·σ·√τ)
    let scale = d_mul(Decimal::TWO, s.to_dec(), "greeks::color::two_spot")?;
    let scale = d_mul(scale, tau.to_dec(), "greeks::color::two_spot_tau")?;
    let vol_time = d_mul(
        sigma.to_dec(),
        k.sqrt_t().to_dec(),
        "greeks::color::vol_time",
    )?;
    let scale = d_mul(scale, vol_time, "greeks::color::factor1_denominator")?;
    let factor1 = d_div(k.n_d1()?, scale, "greeks::color::factor1")?;

    // factor2 = 2qτ + 1 + d1·(2(r-q)τ − d2·σ·√τ) / (σ·√τ)
    let carry = d_sub(r, q, "greeks::color::carry")?;
    let carry_term = d_mul(Decimal::TWO, carry, "greeks::color::two_carry")?;
    let carry_term = d_mul(carry_term, tau.to_dec(), "greeks::color::carry_tau")?;
    let d2_term = d_mul(k.d2()?, vol_time, "greeks::color::d2_vol_time")?;
    let numerator = d_sub(carry_term, d2_term, "greeks::color::factor2_numerator")?;
    let ratio = d_div(numerator, vol_time, "greeks::color::factor2_ratio")?;
    let ratio = d_mul(ratio, k.d1(), "greeks::color::factor2_ratio_d1")?;
    let dividend_term = d_mul(Decimal::TWO, q, "greeks::color::two_q")?;
    let dividend_term = d_mul(dividend_term, tau.to_dec(), "greeks::color::two_q_tau")?;
    let factor2 = d_add(dividend_term, Decimal::ONE, "greeks::color::factor2_base")?;
    let factor2 = d_add(factor2, ratio, "greeks::color::factor2")?;
    // Build the color numerator with checked multiplications so an
    // overflow on `-exp(-qt) * factor1 * factor2 * quantity` surfaces
    // a tagged `DecimalError::Overflow` instead of silently saturating
    // before the final checked `d_div(/, 365)`.
    let numerator = d_mul(
        -exp_minus_qt,
        factor1,
        "greeks::color::numerator_exp_factor1",
    )?;
    let numerator = d_mul(numerator, factor2, "greeks::color::numerator_factor2")?;
    let numerator = d_mul(
        numerator,
        signed_quantity(option),
        "greeks::color::numerator_quantity",
    )?;
    let color = d_div(numerator, Decimal::from(365), "greeks::color::per_day")?;
    Ok(color)
}

#[cfg(test)]
pub mod tests_delta_equations {
    use super::*;
    use crate::constants::ZERO;
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option;
    use crate::strategies::DELTA_THRESHOLD;
    use positive::constants::DAYS_IN_A_YEAR;

    use crate::{ExpirationDate, assert_decimal_eq};
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;
    use tracing::info;

    #[test]
    fn test_delta_no_volatility_itm() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(150.0),
            Positive::ONE,
            pos_or_panic!(150.0),
            Positive::ZERO,
        );
        let delta_value = delta(&option).unwrap();
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value.to_f64().unwrap(), 1.0, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_no_volatility_otm() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(110.0),
            Positive::ONE,
            pos_or_panic!(150.0),
            Positive::ZERO,
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, ZERO, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_no_volatility_itm_put() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos_or_panic!(150.0),
            Positive::ONE,
            pos_or_panic!(150.0),
            Positive::ZERO,
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, -1.0, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_no_volatility_otm_put() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos_or_panic!(160.0),
            Positive::ONE,
            pos_or_panic!(150.0),
            Positive::ZERO,
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, ZERO, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_no_volatility_itm_short() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(150.0),
            Positive::ONE,
            pos_or_panic!(150.0),
            Positive::ZERO,
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, -1.0, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_no_volatility_otm_short() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(110.0),
            Positive::ONE,
            pos_or_panic!(150.0),
            Positive::ZERO,
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, ZERO, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_no_volatility_itm_put_short() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            pos_or_panic!(150.0),
            Positive::ONE,
            pos_or_panic!(150.0),
            Positive::ZERO,
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, 1.0, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_no_volatility_otm_put_short() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            pos_or_panic!(160.0),
            Positive::ONE,
            pos_or_panic!(150.0),
            Positive::ZERO,
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, ZERO, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_deep_in_the_money_call() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(150.0),
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.20),
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Deep ITM Call Delta: {}", delta_value);
        assert_relative_eq!(delta_value, 0.9991784198733309, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_deep_out_of_the_money_call() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(50.0),
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.20),
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Deep OTM Call Delta: {}", delta_value);
        assert_relative_eq!(delta_value, 2.0418256951423236e-33, epsilon = 1e-4);
    }

    #[test]
    fn test_delta_at_the_money_put() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.20),
        );
        let delta_value = delta(&option).unwrap();
        info!("ATM Put Delta: {}", delta_value);
        assert_decimal_eq!(
            delta_value,
            dec!(-0.4653476616529870686572684641),
            DELTA_THRESHOLD
        );
    }

    #[test]
    fn test_delta_short_term_high_volatility() {
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.50),
        );
        option.expiration_date = ExpirationDate::Days(pos_or_panic!(7.0));
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Short-term High Vol Call Delta: {}", delta_value);
        assert_relative_eq!(delta_value, 0.518125955681732, epsilon = 1e-4);
    }

    #[test]
    fn test_delta_long_term_low_volatility() {
        let mut option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.10),
        );
        option.expiration_date = ExpirationDate::Days(DAYS_IN_A_YEAR);
        let delta_value = delta(&option).unwrap();
        info!("Long-term Low Vol Put Delta: {}", delta_value);
        assert_decimal_eq!(
            delta_value,
            dec!(-0.3231079315892283130741442305),
            DELTA_THRESHOLD
        );
    }

    #[test]
    fn test_delta_long_almost_zero_time_to_maturity() {
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(21637.0),
            Positive::ONE,
            pos_or_panic!(21825.0),
            pos_or_panic!(0.219),
        );
        option.expiration_date = ExpirationDate::Days(Positive::ONE);
        let delta_value = delta(&option).unwrap();
        info!("Long-term Low Vol Put Delta: {}", delta_value);
        assert_decimal_eq!(
            delta_value,
            dec!(-0.2298186207440564194124373536),
            DELTA_THRESHOLD
        );
    }
}

#[cfg(test)]
pub mod tests_gamma_equations {
    use super::*;
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option;
    use positive::constants::DAYS_IN_A_YEAR;

    use crate::ExpirationDate;
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use positive::pos_or_panic;
    use tracing::info;

    #[test]
    fn test_gamma_deep_in_the_money_call() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(150.0),
            Positive::ONE,
            pos_or_panic!(120.0),
            pos_or_panic!(0.2),
        );
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("Deep ITM Call Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.000016992916331106763, epsilon = 1e-8);
    }

    #[test]
    fn test_gamma_deep_out_of_the_money_call() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(50.0),
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.20),
        );
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("Deep OTM Call Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.0, epsilon = 1e-34);
    }

    #[test]
    fn test_gamma_at_the_money_put() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.20),
        );
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("ATM Put Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.06926321174822156, epsilon = 1e-8);
    }

    #[test]
    fn test_gamma_short_term_high_volatility() {
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.50),
        );
        option.expiration_date = ExpirationDate::Days(pos_or_panic!(7.0));
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("Short-term High Vol Call Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.05754408301594555, epsilon = 1e-8);
    }

    #[test]
    fn test_gamma_long_term_low_volatility() {
        let mut option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.10),
        );
        option.expiration_date = ExpirationDate::Days(DAYS_IN_A_YEAR);
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("Long-term Low Vol Put Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.03569396592472471, epsilon = 1e-8);
    }

    #[test]
    fn test_gamma_zero_volatility() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            Positive::ZERO,
        );
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("Zero Volatility Call Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.0, epsilon = 1e-8);
    }

    #[test]
    fn test_gamma_extreme_high_volatility() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(5.0),
        );
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("Extreme High Volatility Put Gamma: {}", gamma_value);
        // Short position, so gamma carries the negative sign; see #428.
        assert_relative_eq!(gamma_value, -0.002147363766511278, epsilon = 1e-8);
    }
}

#[cfg(test)]
mod tests_gamma_equations_values {
    use super::*;
    use crate::model::types::{OptionStyle, Side};

    use crate::{ExpirationDate, OptionType};
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use positive::pos_or_panic;
    use tracing::info;

    #[test]
    fn test_50_vol_10() {
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "XYZ".parse().unwrap(),
            pos_or_panic!(50.0),
            ExpirationDate::Days(pos_or_panic!(365.0)),
            pos_or_panic!(0.10),
            Positive::ONE,
            pos_or_panic!(50.0),
            Decimal::ZERO,
            OptionStyle::Call,
            Positive::ZERO,
            None,
        );
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.0796887828189609, epsilon = 1e-8);
    }

    #[test]
    fn test_50_vol_5() {
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "XYZ".parse().unwrap(),
            pos_or_panic!(50.0),
            ExpirationDate::Days(pos_or_panic!(365.0)),
            pos_or_panic!(0.05),
            Positive::ONE,
            pos_or_panic!(50.0),
            Decimal::ZERO,
            OptionStyle::Call,
            Positive::ZERO,
            None,
        );
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.15952705216736393, epsilon = 1e-8);
    }

    #[test]
    fn test_50_vol_20() {
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "XYZ".parse().unwrap(),
            pos_or_panic!(50.0),
            ExpirationDate::Days(pos_or_panic!(365.0)),
            pos_or_panic!(0.2),
            Positive::ONE,
            pos_or_panic!(50.0),
            Decimal::ZERO,
            OptionStyle::Call,
            Positive::ZERO,
            None,
        );
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.03969525474873078, epsilon = 1e-8);
    }
}

#[cfg(test)]
pub mod tests_vega_equation {
    use super::*;
    use crate::ExpirationDate;
    use crate::model::types::{OptionType, Side};
    use num_traits::ToPrimitive;
    use positive::constants::DAYS_IN_A_YEAR;
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    fn create_test_option(
        underlying_price: Positive,
        strike_price: Positive,
        implied_volatility: Positive,
        dividend_yield: Positive,
        expiration_in_days: Positive,
    ) -> Options {
        Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            strike_price,
            ExpirationDate::Days(expiration_in_days),
            implied_volatility,
            Positive::ONE, // Quantity
            underlying_price,
            dec!(0.05), // Risk-free rate
            OptionStyle::Call,
            dividend_yield,
            None, // No exotic params for this test
        )
    }

    #[test]
    fn test_vega_atm() {
        let option = create_test_option(
            Positive::HUNDRED,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
            Positive::ZERO,
            DAYS_IN_A_YEAR,
        );
        let vega = vega(&option).unwrap().to_f64().unwrap();
        let expected_vega = 0.3752403469;
        assert!(
            (vega - expected_vega).abs() < 1e-5,
            "Vega ATM test failed: expected {expected_vega}, got {vega}"
        );
    }

    #[test]
    fn test_vega_otm() {
        let option = create_test_option(
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            pos_or_panic!(0.2),
            Positive::ZERO,
            DAYS_IN_A_YEAR,
        );
        let vega = vega(&option).unwrap().to_f64().unwrap();
        let expected_vega = 0.35347991;
        assert!(
            (vega - expected_vega).abs() < 1e-5,
            "Vega OTM test failed: expected {expected_vega}, got {vega}"
        );
    }

    #[test]
    fn test_vega_short_expiration() {
        let option = create_test_option(
            Positive::HUNDRED,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
            Positive::ZERO,
            Positive::ONE,
        );
        let vega = vega(&option).unwrap().to_f64().unwrap();
        let expected_vega = 0.020878089;
        assert!(
            (vega - expected_vega).abs() < 1e-5,
            "Vega short expiration test failed: expected {expected_vega}, got {vega}"
        );
    }

    #[test]
    fn test_vega_with_dividends() {
        let option = create_test_option(
            Positive::HUNDRED,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
            pos_or_panic!(0.03),
            Positive::ONE,
        );
        let vega = vega(&option).unwrap().to_f64().unwrap();
        let expected_vega = 0.0208763735;
        assert!(
            (vega - expected_vega).abs() < 1e-5,
            "Vega with dividends test failed: expected {expected_vega}, got {vega}"
        );
    }

    #[test]
    fn test_vega_itm() {
        let option = create_test_option(
            pos_or_panic!(110.0),
            Positive::HUNDRED,
            pos_or_panic!(0.2),
            Positive::ZERO,
            Positive::ONE,
        );
        let vega = vega(&option).unwrap().to_f64().unwrap();
        let expected_vega = 0.0;
        assert!(
            (vega - expected_vega).abs() < 1e-5,
            "Vega ITM test failed: expected {expected_vega}, got {vega}"
        );
    }
}

#[cfg(test)]
pub mod tests_rho_equations {
    use super::*;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use crate::{ExpirationDate, assert_decimal_eq};
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use positive::constants::DAYS_IN_A_YEAR;
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    fn create_test_option(style: OptionStyle) -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "TEST".to_string(),
            strike_price: Positive::HUNDRED,
            expiration_date: ExpirationDate::Days(DAYS_IN_A_YEAR),
            implied_volatility: pos_or_panic!(0.2),
            quantity: Positive::ONE,
            underlying_price: Positive::HUNDRED,
            risk_free_rate: dec!(0.05),
            option_style: style,
            dividend_yield: Positive::ZERO,
            exotic_params: None,
        }
    }

    #[test]
    fn test_rho_call_option() {
        let option = create_test_option(OptionStyle::Call);
        let result = rho(&option).unwrap().to_f64().unwrap();
        assert_relative_eq!(result, 0.532324815464, epsilon = 1e-8);
    }

    #[test]
    fn test_rho_put_option() {
        let option = create_test_option(OptionStyle::Put);
        let result = rho(&option).unwrap().to_f64().unwrap();
        assert_relative_eq!(result, -0.41890460905, epsilon = 1e-8);
    }

    #[test]
    fn test_rho_zero_time_to_expiry() {
        let mut option = create_test_option(OptionStyle::Call);
        option.expiration_date = ExpirationDate::Days(Positive::ZERO);
        let result = rho(&option).is_ok();
        assert!(result);
        assert_decimal_eq!(rho(&option).unwrap(), Decimal::ZERO, dec!(1e-8));
    }

    #[test]
    fn test_rho_zero_risk_free_rate() {
        let mut option = create_test_option(OptionStyle::Call);
        option.risk_free_rate = dec!(0.0);
        let result = rho(&option).unwrap().to_f64().unwrap();
        assert_relative_eq!(result, 0.460172162, epsilon = 1e-8);
    }

    #[test]
    fn test_rho_deep_out_of_money_call() {
        let mut option = create_test_option(OptionStyle::Call);
        option.strike_price = pos_or_panic!(1000.0);
        let result = rho(&option).unwrap().to_f64().unwrap();
        assert_relative_eq!(result, 0.0, epsilon = 1e-8);
    }

    #[test]
    fn test_rho_deep_out_of_money_put() {
        let mut option = create_test_option(OptionStyle::Put);
        option.strike_price = Positive::ONE;
        let result = rho(&option).unwrap().to_f64().unwrap();
        assert_relative_eq!(result, 0.0, epsilon = 1e-8);
    }

    #[test]
    fn test_rho_high_volatility() {
        let mut option = create_test_option(OptionStyle::Call);
        option.implied_volatility = Positive::ONE;
        let result = rho(&option).unwrap().to_f64().unwrap();
        assert_relative_eq!(result, 0.3104386883, epsilon = 0.0001);
    }
}

#[cfg(test)]
pub mod tests_theta_long_equations {
    use super::*;
    use crate::ExpirationDate;
    use crate::model::types::Side;
    use crate::model::utils::create_sample_option;
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use positive::constants::DAYS_IN_A_YEAR;
    use positive::pos_or_panic;

    #[test]
    fn test_theta_call_option() {
        // Create a sample call option
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(150.0), // underlying price
            Positive::ONE,        // quantity
            pos_or_panic!(155.0), // strike price
            pos_or_panic!(0.20),  // implied volatility
        );

        // Expected theta value for a call option (precomputed or from known source)
        let expected_theta = -0.05569703183000544;

        // Compute the theta value using the function
        let calculated_theta = theta(&option).unwrap().to_f64().unwrap();

        // Assert the calculated theta is close to the expected value
        assert_relative_eq!(calculated_theta, expected_theta, epsilon = 1e-8);
    }

    #[test]
    fn test_theta_put_option() {
        // Create a sample put option
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos_or_panic!(150.0), // underlying price
            Positive::ONE,        // quantity
            pos_or_panic!(145.0), // strike price
            pos_or_panic!(0.25),  // implied volatility
        );

        // Expected theta value for a put option (precomputed or from known source)
        let expected_theta = -0.05620624081929407;

        // Compute the theta value using the function
        let calculated_theta = theta(&option).unwrap().to_f64().unwrap();

        // Assert the calculated theta is close to the expected value
        assert_relative_eq!(calculated_theta, expected_theta, epsilon = 1e-8);
    }

    #[test]
    fn test_theta_call_option_near_expiry() {
        // Create a sample call option near expiry
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(150.0), // underlying price
            Positive::ONE,        // quantity
            pos_or_panic!(150.0), // strike price
            pos_or_panic!(0.15),  // implied volatility
        );
        option.expiration_date = ExpirationDate::Days(Positive::ONE); // Option close to expiry

        // Expected theta value for a near-expiry call option (precomputed)
        let expected_theta = -0.24314466256999295;

        // Compute the theta value using the function
        let calculated_theta = theta(&option).unwrap().to_f64().unwrap();

        // Assert the calculated theta is close to the expected value
        assert_relative_eq!(calculated_theta, expected_theta, epsilon = 1e-8);
    }

    #[test]
    fn test_theta_put_option_far_from_expiry() {
        // Create a sample put option far from expiry
        let mut option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos_or_panic!(140.0), // underlying price
            Positive::ONE,        // quantity
            pos_or_panic!(130.0), // strike price
            pos_or_panic!(0.30),  // implied volatility
        );
        option.expiration_date = ExpirationDate::Days(DAYS_IN_A_YEAR); // Option far from expiry

        // Expected theta value for a far-expiry put option (precomputed)
        let expected_theta = -0.013947672323606776;

        // Compute the theta value using the function
        let calculated_theta = theta(&option).unwrap().to_f64().unwrap();

        // Assert the calculated theta is close to the expected value
        assert_relative_eq!(calculated_theta, expected_theta, epsilon = 1e-8);
    }
}

#[cfg(test)]
pub mod tests_theta_short_equations {
    use super::*;
    use crate::ExpirationDate;
    use crate::model::types::Side;
    use crate::model::utils::create_sample_option;
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use positive::constants::DAYS_IN_A_YEAR;
    use positive::pos_or_panic;

    #[test]
    fn test_theta_short_call_option() {
        // Create a sample short call option
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(150.0), // underlying price
            Positive::ONE,        // quantity
            pos_or_panic!(155.0), // strike price
            pos_or_panic!(0.20),  // implied volatility
        );

        // A short call collects decay, so its theta is positive; see #428.
        let expected_theta = 0.05569703183000544;

        // Compute the theta value using the function
        let calculated_theta = theta(&option).unwrap().to_f64().unwrap();

        // Assert the calculated theta is close to the expected value
        assert_relative_eq!(calculated_theta, expected_theta, epsilon = 1e-8);
    }

    #[test]
    fn test_theta_short_put_option() {
        // Create a sample short put option
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            pos_or_panic!(150.0), // underlying price
            Positive::ONE,        // quantity
            pos_or_panic!(145.0), // strike price
            pos_or_panic!(0.25),  // implied volatility
        );

        // A short put collects decay, so its theta is positive; see #428.
        let expected_theta = 0.05620624081929407;

        // Compute the theta value using the function
        let calculated_theta = theta(&option).unwrap().to_f64().unwrap();

        // Assert the calculated theta is close to the expected value
        assert_relative_eq!(calculated_theta, expected_theta, epsilon = 1e-8);
    }

    #[test]
    fn test_theta_short_call_option_near_expiry() {
        // Create a sample short call option near expiry
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(150.0), // underlying price
            Positive::ONE,        // quantity
            pos_or_panic!(150.0), // strike price
            pos_or_panic!(0.15),  // implied volatility
        );
        option.expiration_date = ExpirationDate::Days(Positive::ONE); // Option close to expiry

        // Expected theta value for a short near-expiry call option (precomputed)
        let expected_theta = 0.24314466256999295;

        // Compute the theta value using the function
        let calculated_theta = theta(&option).unwrap().to_f64().unwrap();

        // Assert the calculated theta is close to the expected value
        assert_relative_eq!(calculated_theta, expected_theta, epsilon = 1e-8);
    }

    #[test]
    fn test_theta_short_put_option_far_from_expiry() {
        // Create a sample short put option far from expiry
        let mut option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            pos_or_panic!(140.0), // underlying price
            Positive::ONE,        // quantity
            pos_or_panic!(130.0), // strike price
            pos_or_panic!(0.30),  // implied volatility
        );
        option.expiration_date = ExpirationDate::Days(DAYS_IN_A_YEAR); // Option far from expiry

        // Expected theta value for a far-expiry short put option (precomputed)
        let expected_theta = 0.013947672323606776;

        // Compute the theta value using the function
        let calculated_theta = theta(&option).unwrap().to_f64().unwrap();

        // Assert the calculated theta is close to the expected value
        assert_relative_eq!(calculated_theta, expected_theta, epsilon = 1e-8);
    }
}

#[cfg(test)]
mod tests_greeks_trait {
    use super::*;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use crate::{ExpirationDate, assert_decimal_eq};
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    // A simple struct for testing the Greeks trait
    struct TestOptionCollection {
        options: Vec<Options>,
    }

    impl Greeks for TestOptionCollection {
        fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
            Ok(self.options.iter().collect())
        }
    }

    // Helper function to create a test option
    fn create_test_option_at(
        side: Side,
        style: OptionStyle,
        quantity: Positive,
        expiration_date: ExpirationDate,
    ) -> Options {
        Options::new(
            OptionType::European,
            side,
            "TEST".to_string(),
            Positive::HUNDRED,
            expiration_date,
            pos_or_panic!(0.2),
            quantity,
            Positive::HUNDRED,
            dec!(0.05),
            style,
            pos_or_panic!(0.01),
            None,
        )
    }

    #[test]
    fn test_rho_d_and_vanna_are_zero_at_expiry() {
        // Both used to reach `d1` unguarded and fail with InvalidTime, while
        // their ten siblings returned zero. That asymmetry made the whole
        // twelve-greek set unobtainable at expiry.
        let option = create_test_option_at(
            Side::Long,
            OptionStyle::Call,
            Positive::ONE,
            ExpirationDate::Days(Positive::ZERO),
        );
        match rho_d(&option) {
            Ok(value) => assert_eq!(value, Decimal::ZERO),
            Err(e) => panic!("rho_d should be zero at expiry, got {e}"),
        }
        match vanna(&option) {
            Ok(value) => assert_eq!(value, Decimal::ZERO),
            Err(e) => panic!("vanna should be zero at expiry, got {e}"),
        }
    }

    #[test]
    fn test_greeks_succeeds_at_expiry() {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            let option = create_test_option_at(
                Side::Long,
                style,
                Positive::ONE,
                ExpirationDate::Days(Positive::ZERO),
            );
            let greek = match option.greeks() {
                Ok(greek) => greek,
                Err(e) => panic!("greeks() should succeed at expiry for {style:?}: {e}"),
            };
            assert_eq!(greek.gamma, Decimal::ZERO);
            assert_eq!(greek.theta, Decimal::ZERO);
            assert_eq!(greek.vega, Decimal::ZERO);
            assert_eq!(greek.rho_d, Decimal::ZERO);
            assert_eq!(greek.vanna, Decimal::ZERO);
        }
    }

    // Helper function to create a test option
    fn create_test_option(side: Side, style: OptionStyle, quantity: Positive) -> Options {
        Options::new(
            OptionType::European,
            side,
            "TEST".to_string(),
            Positive::HUNDRED, // strike_price
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2), // implied_volatility
            quantity,
            Positive::HUNDRED, // underlying_price
            dec!(0.05),        // risk_free_rate
            style,
            pos_or_panic!(0.01), // dividend_yield
            None,                // exotic_params
        )
    }

    #[test]
    fn test_greeks_single_option() {
        let option = create_test_option(Side::Long, OptionStyle::Call, Positive::ONE);
        let collection = TestOptionCollection {
            options: vec![option],
        };

        let greeks = collection.greeks().unwrap();

        // Test each greek value
        assert_decimal_eq!(
            greeks.delta,
            dec!(0.5338307582207135564475476937),
            dec!(0.000001)
        );
        assert_decimal_eq!(
            greeks.gamma,
            dec!(0.0692632117482215620683508231),
            dec!(0.000001)
        );
        assert_decimal_eq!(
            greeks.theta,
            dec!(-0.0434671314177636287945041349),
            dec!(0.000001)
        );
        assert_decimal_eq!(
            greeks.vega,
            dec!(0.1138573343806381728362131205),
            dec!(0.000001)
        );
        assert_decimal_eq!(
            greeks.rho,
            dec!(0.041863419880440417503050762),
            dec!(0.000001)
        );
        assert_decimal_eq!(
            greeks.rho_d,
            dec!(-0.0438765006756750824436552223),
            dec!(0.000001)
        );
        assert_decimal_eq!(
            greeks.vanna,
            dec!(-0.0569286671903190864181065602),
            dec!(0.000001)
        );
        assert_decimal_eq!(
            greeks.vomma,
            dec!(0.0014037205608571828124031742),
            dec!(0.000001)
        );
        assert_decimal_eq!(
            greeks.veta,
            dec!(0.000027236903336955576237203),
            dec!(0.000001)
        );
    }

    #[test]
    fn test_greeks_multiple_options() {
        let option1 = create_test_option(Side::Long, OptionStyle::Call, Positive::ONE);
        let option2 = create_test_option(Side::Short, OptionStyle::Put, Positive::ONE);
        let collection = TestOptionCollection {
            options: vec![option1, option2],
        };

        let greeks = collection.greeks().unwrap();

        // A long call and a short put at the same strike and expiry is a
        // synthetic long forward. The greeks that carry no `option_style` branch
        // are identical for the two legs, so signing them cancels those exactly;
        // the ones that do branch survive.
        for (name, value) in [
            ("gamma", greeks.gamma),
            ("vega", greeks.vega),
            ("vanna", greeks.vanna),
            ("vomma", greeks.vomma),
            ("veta", greeks.veta),
            ("color", greeks.color),
        ] {
            assert_eq!(
                value,
                Decimal::ZERO,
                "{name} has no style branch, so a synthetic forward cancels it"
            );
        }
        for (name, value) in [
            ("delta", greeks.delta),
            ("theta", greeks.theta),
            ("rho", greeks.rho),
            ("rho_d", greeks.rho_d),
            ("charm", greeks.charm),
        ] {
            assert!(
                value.abs() > Decimal::ZERO,
                "{name} branches on style, so it must survive the synthetic forward"
            );
        }
    }

    #[test]
    fn test_greeks_simple_validation() {
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "AAPL".to_string(),
            pos_or_panic!(155.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.20),
            Positive::ONE,
            pos_or_panic!(150.0),
            dec!(0.05),
            OptionStyle::Call,
            pos_or_panic!(0.00),
            None,
        );

        let greeks = option.greeks().unwrap();

        assert_decimal_eq!(greeks.delta, dec!(0.3186329), dec!(0.000001));
        assert_decimal_eq!(greeks.gamma, dec!(0.0415044), dec!(0.000001));
        assert_decimal_eq!(greeks.theta, dec!(-0.0574808), dec!(0.000001));
        assert_decimal_eq!(greeks.vega, dec!(0.15350973), dec!(0.000001));
        assert_decimal_eq!(greeks.rho, dec!(0.03786580), dec!(0.000001));
        assert_decimal_eq!(greeks.rho_d, dec!(-0.03928351), dec!(0.000001));
        assert_decimal_eq!(
            greeks.vanna,
            dec!(0.9439386484253192473553911946),
            dec!(0.000001)
        );
        assert_decimal_eq!(greeks.vomma, dec!(0.19140525), dec!(0.000001));
        assert_decimal_eq!(greeks.veta, dec!(0.00004880), dec!(0.000001));
    }

    #[test]
    fn test_greeks_zero_quantity() {
        let option = create_test_option(Side::Long, OptionStyle::Call, Positive::ZERO);
        let collection = TestOptionCollection {
            options: vec![option],
        };

        let greeks = collection.greeks().unwrap();

        // All greeks should be zero for zero quantity
        assert_eq!(greeks.delta, dec!(0.0));
        assert_eq!(greeks.gamma, dec!(0.0));
        assert_eq!(greeks.theta, dec!(0.0));
        assert_eq!(greeks.vega, dec!(0.0));
        assert_eq!(greeks.rho, dec!(0.0));
        assert_eq!(greeks.rho_d, dec!(0.0));
        assert_eq!(greeks.vanna, dec!(0.0));
        assert_eq!(greeks.vomma, dec!(0.0));
        assert_eq!(greeks.veta, dec!(0.0));
    }

    #[test]
    fn test_dividend_high_q_carry_regression() {
        // Regression test for the dividend (carry) bug family.
        //
        // Before the fix, the analytic Greeks passed the bare risk-free rate
        // (b = r) into d1/d2, silently dropping the dividend yield. Here the
        // carry is strongly negative (b = r - q = 0.05 - 0.08 = -0.03, over a
        // full year), so every value below is materially different from the
        // buggy, dividend-blind output and pins the corrected Black-Scholes
        // (Merton 1973) formulas.
        //
        // Reference computation (independent Python Black-Scholes Merton
        // implementation, cross-checked against finite differences of the
        // closed-form delta): b = -0.03, d1 = 0.05, d2 = -0.25.
        //
        // Tolerance is 1e-12, not tighter: `big_n` evaluates the normal CDF in
        // f64 via statrs and widens the result with `Decimal::from_f64`, so
        // genuine precision here is ~1e-16. The digits past the 16th are
        // Decimal rounding noise, and asserting them would turn this into a
        // bit-pattern lock that a statrs patch release could break without
        // changing any of the mathematics. 1e-12 still separates these values
        // from the pre-fix dividend-blind ones, which differ in the second
        // significant digit (delta 0.5762569743 vs 0.4799640108).
        // Sanity checks implied by the values:
        //   * rho_d == -delta exactly, since S*T = 100 (both rho_d and delta
        //     are scaled by 1/100) and rho_d = -S*T*e^{-qT}*N(d1).
        //   * vanna > 0, because vanna = -e^{-qT}*n(d1)*d2/sigma and d2 = -0.25
        //     (sign is opposite to d2).
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "HIQ".to_string(),
            pos_or_panic!(100.0),
            ExpirationDate::Days(pos_or_panic!(365.0)),
            pos_or_panic!(0.3),
            Positive::ONE,
            pos_or_panic!(100.0),
            dec!(0.05),
            OptionStyle::Call,
            pos_or_panic!(0.08),
            None,
        );
        let g = option.greeks().unwrap();
        assert_decimal_eq!(g.delta, dec!(0.4799640107901480920925461492), dec!(1e-12));
        assert_decimal_eq!(g.gamma, dec!(0.0122603363406382730603468554), dec!(1e-12));
        assert_decimal_eq!(g.theta, dec!(-0.0098247973187618777368017226), dec!(1e-12));
        assert_decimal_eq!(g.vega, dec!(0.3678100902191481918104056619), dec!(1e-12));
        assert_decimal_eq!(g.rho, dec!(0.381722350876409446703382601), dec!(1e-12));
        assert_decimal_eq!(g.rho_d, dec!(-0.4799640107901480920925461492), dec!(1e-12));
        assert_decimal_eq!(g.vanna, dec!(0.3065084085159568265086713849), dec!(1e-12));
        assert_decimal_eq!(g.vomma, dec!(-0.0153254204257978413254335693), dec!(1e-12));
        assert_decimal_eq!(g.veta, dec!(0.0000061119236221931867190717), dec!(1e-12));
        assert_decimal_eq!(g.charm, dec!(0.0000800051194732414864990234), dec!(1e-12));
        assert_decimal_eq!(g.color, dec!(-0.000019524165747934236209114), dec!(1e-12));
    }

    #[test]
    fn test_greeks_opposing_positions() {
        let option1 = Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            pos_or_panic!(50.0), // strike_price
            ExpirationDate::Days(pos_or_panic!(365.0)),
            pos_or_panic!(0.2), // implied_volatility
            Positive::ONE,
            pos_or_panic!(50.0), // underlying_price
            dec!(0.05),          // risk_free_rate
            OptionStyle::Call,
            pos_or_panic!(0.01), // dividend_yield
            None,                // exotic_params
        );
        let option2 = Options::new(
            OptionType::European,
            Side::Short,
            "TEST".to_string(),
            pos_or_panic!(50.0), // strike_price
            ExpirationDate::Days(pos_or_panic!(365.0)),
            pos_or_panic!(0.2), // implied_volatility
            Positive::ONE,
            pos_or_panic!(50.0), // underlying_price
            dec!(0.05),          // risk_free_rate
            OptionStyle::Call,
            pos_or_panic!(0.01), // dividend_yield
            None,                // exotic_params
        );
        let collection = TestOptionCollection {
            options: vec![option1, option2],
        };

        let greeks = collection.greeks().unwrap();

        // A long and a short of the identical contract is a closed position, so
        // every greek cancels exactly. Before #428 only delta did, and the other
        // eleven summed as though both legs were long.
        assert_eq!(greeks.delta, Decimal::ZERO);
        assert_eq!(greeks.gamma, Decimal::ZERO);
        assert_eq!(greeks.theta, Decimal::ZERO);
        assert_eq!(greeks.vega, Decimal::ZERO);
        assert_eq!(greeks.rho, Decimal::ZERO);
        assert_eq!(greeks.rho_d, Decimal::ZERO);
        assert_eq!(greeks.vanna, Decimal::ZERO);
        assert_eq!(greeks.vomma, Decimal::ZERO);
        assert_eq!(greeks.veta, Decimal::ZERO);
        assert_eq!(greeks.charm, Decimal::ZERO);
        assert_eq!(greeks.color, Decimal::ZERO);
    }

    #[test]
    fn test_individual_greek_methods() {
        let option1 = create_test_option(Side::Long, OptionStyle::Call, Positive::ONE);
        let option2 = create_test_option(Side::Short, OptionStyle::Put, Positive::ONE);
        let collection = TestOptionCollection {
            options: vec![option1, option2],
        };

        // Test each individual greek method
        let delta = collection.delta().unwrap();
        let gamma = collection.gamma().unwrap();
        let theta = collection.theta().unwrap();
        let vega = collection.vega().unwrap();
        let rho = collection.rho().unwrap();
        let rho_d = collection.rho_d().unwrap();
        let vanna = collection.vanna().unwrap();
        let vomma = collection.vomma().unwrap();
        let veta = collection.veta().unwrap();

        // Same synthetic long forward as `test_greeks_multiple_options`: the
        // style-independent greeks cancel once the short leg is signed, and the
        // style-dependent ones survive.
        assert_eq!(gamma, Decimal::ZERO, "gamma should cancel");
        assert_eq!(vega, Decimal::ZERO, "vega should cancel");
        assert_eq!(vanna, Decimal::ZERO, "vanna should cancel");
        assert_eq!(vomma, Decimal::ZERO, "vomma should cancel");
        assert_eq!(veta, Decimal::ZERO, "veta should cancel");
        assert!(delta.abs() > Decimal::ZERO, "Delta calculation failed");
        assert!(theta.abs() > Decimal::ZERO, "Theta calculation failed");
        assert!(rho.abs() > Decimal::ZERO, "Rho calculation failed");
        assert!(rho_d.abs() > Decimal::ZERO, "Rho_d calculation failed");
    }

    #[test]
    fn test_empty_option_collection() {
        let collection = TestOptionCollection { options: vec![] };

        // All greeks should be zero for empty collection
        let greeks = collection.greeks().unwrap();
        assert_eq!(greeks.delta, dec!(0.0));
        assert_eq!(greeks.gamma, dec!(0.0));
        assert_eq!(greeks.theta, dec!(0.0));
        assert_eq!(greeks.vega, dec!(0.0));
        assert_eq!(greeks.rho, dec!(0.0));
        assert_eq!(greeks.rho_d, dec!(0.0));
        assert_eq!(greeks.vanna, dec!(0.0));
        assert_eq!(greeks.vomma, dec!(0.0));
        assert_eq!(greeks.veta, dec!(0.0));
    }

    #[test]
    fn test_greeks_with_different_expirations() {
        let mut option1 = create_test_option(Side::Long, OptionStyle::Call, Positive::ONE);
        let mut option2 = create_test_option(Side::Long, OptionStyle::Call, Positive::ONE);

        // Set different expiration dates
        option1.expiration_date = ExpirationDate::Days(pos_or_panic!(30.0));
        option2.expiration_date = ExpirationDate::Days(pos_or_panic!(60.0));

        let collection = TestOptionCollection {
            options: vec![option1, option2],
        };

        let greeks = collection.greeks().unwrap();

        // Verify values are calculated correctly for different expirations
        assert!(greeks.delta.abs() > dec!(0.0));
        assert!(greeks.gamma.abs() > dec!(0.0));
        assert!(greeks.theta.abs() > dec!(0.0));
        assert!(greeks.vega.abs() > dec!(0.0));
        assert!(greeks.rho.abs() > dec!(0.0));
        assert!(greeks.rho_d.abs() > dec!(0.0));
        assert!(greeks.vanna.abs() > dec!(0.0));
        assert!(greeks.vomma.abs() > dec!(0.0));
        assert!(greeks.veta.abs() > dec!(0.0));
    }
}

#[cfg(test)]
pub mod tests_vanna_equation {
    use super::*;
    use crate::ExpirationDate;
    use crate::model::types::{OptionType, Side};
    use num_traits::ToPrimitive;
    use positive::constants::DAYS_IN_A_YEAR;
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    fn create_test_option(
        underlying_price: Positive,
        strike_price: Positive,
        implied_volatility: Positive,
        dividend_yield: Positive,
        expiration_in_days: Positive,
    ) -> Options {
        Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            strike_price,
            ExpirationDate::Days(expiration_in_days),
            implied_volatility,
            Positive::ONE, // Quantity
            underlying_price,
            dec!(0.05), // Risk-free rate
            OptionStyle::Call,
            dividend_yield,
            None, // No exotic params for this test
        )
    }

    #[test]
    fn test_vanna_atm() {
        let option = create_test_option(
            Positive::HUNDRED,  // underlying_price
            Positive::HUNDRED,  // strike_price
            pos_or_panic!(0.2), // implied_volatility
            Positive::ZERO,     // dividend_yield
            DAYS_IN_A_YEAR,     // expiration_in_days
        );
        let vanna = vanna(&option).unwrap().to_f64().unwrap();
        let expected_vanna = -0.2814302601877034;
        assert!(
            (vanna - expected_vanna).abs() < 1e-5,
            "Vega ATM test failed: expected {expected_vanna}, got {vanna}"
        );
    }

    #[test]
    fn test_vanna_otm() {
        let option = create_test_option(
            pos_or_panic!(90.0), // underlying_price
            Positive::HUNDRED,   // strike_price
            pos_or_panic!(0.2),  // implied_volatility
            Positive::ZERO,      // dividend_yield
            DAYS_IN_A_YEAR,      // expiration_in_days
        );
        let vanna = vanna(&option).unwrap().to_f64().unwrap();
        let expected_vanna = 0.7399563431070563;
        assert!(
            (vanna - expected_vanna).abs() < 1e-5,
            "Vanna OTM test failed: expected {expected_vanna}, got {vanna}"
        );
    }

    #[test]
    fn test_vanna_short_expiration() {
        let option = create_test_option(
            Positive::HUNDRED,  // underlying_price
            Positive::HUNDRED,  // strike_price
            pos_or_panic!(0.2), // implied_volatility
            Positive::ZERO,     // dividend_yield
            Positive::ONE,      // expiration_in_days
        );
        let vanna = vanna(&option).unwrap().to_f64().unwrap();
        let expected_vanna = -0.015658567140361693;
        assert!(
            (vanna - expected_vanna).abs() < 1e-5,
            "Vanna short expiration test failed: expected {expected_vanna}, got {vanna}"
        );
    }

    #[test]
    fn test_vanna_with_dividends() {
        let option = create_test_option(
            Positive::HUNDRED,   // underlying_price
            Positive::HUNDRED,   // strike_price
            pos_or_panic!(0.2),  // implied_volatility
            pos_or_panic!(0.03), // dividend_yield
            Positive::ONE,       // expiration_in_days
        );
        let vanna = vanna(&option).unwrap().to_f64().unwrap();
        // With dividends (q = 0.03) and a one-day expiry, the carry is
        // r - q = 0.02 = sigma^2/2, so d1 = sigma*sqrt(T) and d2 = d1 - sigma*sqrt(T)
        // collapses to exactly 0; vanna = -e^{-qT} * n(d1) * d2 / sigma is therefore ~0.
        let expected_vanna = 0.0;
        assert!(
            (vanna - expected_vanna).abs() < 1e-5,
            "Vanna with dividends test failed: expected {expected_vanna}, got {vanna}"
        );
    }

    #[test]
    fn test_vanna_itm() {
        let option = create_test_option(
            pos_or_panic!(110.0), // underlying_price
            Positive::HUNDRED,    // strike_price
            pos_or_panic!(0.2),   // implied_volatility
            Positive::ZERO,       // dividend_yield
            Positive::ONE,        // expiration_in_days
        );
        let vanna = vanna(&option).unwrap().to_f64().unwrap();
        let expected_vanna = 0.0;
        assert!(
            (vanna - expected_vanna).abs() < 1e-5,
            "Vanna ITM test failed: expected {expected_vanna}, got {vanna}"
        );
    }
}

#[cfg(test)]
pub mod tests_vomma_equation {
    use super::*;
    use crate::ExpirationDate;
    use crate::model::types::{OptionType, Side};
    use num_traits::ToPrimitive;
    use positive::constants::DAYS_IN_A_YEAR;
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    fn create_test_option(
        underlying_price: Positive,
        strike_price: Positive,
        implied_volatility: Positive,
        dividend_yield: Positive,
        expiration_in_days: Positive,
    ) -> Options {
        Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            strike_price,
            ExpirationDate::Days(expiration_in_days),
            implied_volatility,
            Positive::ONE, // Quantity
            underlying_price,
            dec!(0.05), // Risk-free rate
            OptionStyle::Call,
            dividend_yield,
            None, // No exotic params for this test
        )
    }

    #[test]
    fn test_vomma_atm() {
        let option = create_test_option(
            Positive::HUNDRED,  // underlying_price
            Positive::HUNDRED,  // strike_price
            pos_or_panic!(0.2), // implied_volatility
            Positive::ZERO,     // dividend_yield
            DAYS_IN_A_YEAR,     // expiration_in_days
        );
        let vomma = vomma(&option).unwrap().to_f64().unwrap();
        let expected_vomma = 0.09850059;
        assert!(
            (vomma - expected_vomma).abs() < 1e-5,
            "Vomma ATM test failed: expected {expected_vomma}, got {vomma}"
        );
    }

    #[test]
    fn test_vomma_otm() {
        let option = create_test_option(
            pos_or_panic!(90.0), // underlying_price
            Positive::HUNDRED,   // strike_price
            pos_or_panic!(0.2),  // implied_volatility
            Positive::ZERO,      // dividend_yield
            DAYS_IN_A_YEAR,      // expiration_in_days
        );
        let vomma = vomma(&option).unwrap().to_f64().unwrap();
        let expected_vomma = 0.11774357;
        assert!(
            (vomma - expected_vomma).abs() < 1e-5,
            "Vomma OTM test failed: expected {expected_vomma}, got {vomma}"
        );
    }

    #[test]
    fn test_vomma_short_expiration() {
        let option = create_test_option(
            Positive::HUNDRED,  // underlying_price
            Positive::HUNDRED,  // strike_price
            pos_or_panic!(0.2), // implied_volatility
            Positive::ZERO,     // dividend_yield
            Positive::ONE,      // expiration_in_days
        );
        let vomma = vomma(&option).unwrap().to_f64().unwrap();
        let expected_vomma = 0.0000150150;
        assert!(
            (vomma - expected_vomma).abs() < 1e-5,
            "Vomma short expiration test failed: expected {expected_vomma}, got {vomma}"
        );
    }

    #[test]
    fn test_vomma_with_dividends() {
        let option = create_test_option(
            Positive::HUNDRED,   // underlying_price
            Positive::HUNDRED,   // strike_price
            pos_or_panic!(0.2),  // implied_volatility
            pos_or_panic!(0.03), // dividend_yield
            Positive::ONE,       // expiration_in_days
        );
        let vomma = vomma(&option).unwrap().to_f64().unwrap();
        // Same degeneracy as vanna above: short expiry plus dividends drives
        // vomma to ~0.
        let expected_vomma = 0.0;
        assert!(
            (vomma - expected_vomma).abs() < 1e-5,
            "Vomma with dividends test failed: expected {expected_vomma}, got {vomma}"
        );
    }

    #[test]
    fn test_vomma_itm() {
        let option = create_test_option(
            pos_or_panic!(110.0), // underlying_price
            Positive::HUNDRED,    // strike_price
            pos_or_panic!(0.2),   // implied_volatility
            Positive::ZERO,       // dividend_yield
            Positive::ONE,        // expiration_in_days
        );
        let vomma = vomma(&option).unwrap().to_f64().unwrap();
        let expected_vomma = 0.0;
        assert!(
            (vomma - expected_vomma).abs() < 1e-5,
            "Vomma ITM test failed: expected {expected_vomma}, got {vomma}"
        );
    }
}

#[cfg(test)]
pub mod tests_veta_equation {
    use super::*;
    use crate::ExpirationDate;
    use crate::model::types::{OptionType, Side};
    use num_traits::ToPrimitive;
    use positive::constants::DAYS_IN_A_YEAR;
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    fn create_test_option(
        underlying_price: Positive,
        strike_price: Positive,
        implied_volatility: Positive,
        dividend_yield: Positive,
        expiration_in_days: Positive,
    ) -> Options {
        Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            strike_price,
            ExpirationDate::Days(expiration_in_days),
            implied_volatility,
            Positive::ONE, // Quantity
            underlying_price,
            dec!(0.05), // Risk-free rate
            OptionStyle::Call,
            dividend_yield,
            None, // No exotic params for this test
        )
    }

    #[test]
    fn test_veta_atm() {
        let option = create_test_option(
            Positive::HUNDRED,  // underlying_price
            Positive::HUNDRED,  // strike_price
            pos_or_panic!(0.2), // implied_volatility
            Positive::ZERO,     // dividend_yield
            DAYS_IN_A_YEAR,     // expiration_in_days
        );
        let veta = veta(&option).unwrap().to_f64().unwrap();
        let expected_veta = 0.0000065332;
        assert!(
            (veta - expected_veta).abs() < 1e-5,
            "Veta ATM test failed: expected {expected_veta}, got {veta}"
        );
    }

    #[test]
    fn test_veta_otm() {
        let option = create_test_option(
            pos_or_panic!(90.0), // underlying_price
            Positive::HUNDRED,   // strike_price
            pos_or_panic!(0.2),  // implied_volatility
            Positive::ZERO,      // dividend_yield
            DAYS_IN_A_YEAR,      // expiration_in_days
        );
        let veta = veta(&option).unwrap().to_f64().unwrap();
        let expected_veta = 0.0000081007;
        assert!(
            (veta - expected_veta).abs() < 1e-5,
            "Veta OTM test failed: expected {expected_veta}, got {veta}"
        );
    }

    #[test]
    fn test_veta_short_expiration() {
        let option = create_test_option(
            Positive::HUNDRED,  // underlying_price
            Positive::HUNDRED,  // strike_price
            pos_or_panic!(0.2), // implied_volatility
            Positive::ZERO,     // dividend_yield
            Positive::ONE,      // expiration_in_days
        );
        let veta = veta(&option).unwrap().to_f64().unwrap();
        let expected_veta = 0.0001511497;
        assert!(
            (veta - expected_veta).abs() < 1e-5,
            "Veta short expiration test failed: expected {expected_veta}, got {veta}"
        );
    }

    #[test]
    fn test_veta_with_dividends() {
        let option = create_test_option(
            Positive::HUNDRED,   // underlying_price
            Positive::HUNDRED,   // strike_price
            pos_or_panic!(0.2),  // implied_volatility
            pos_or_panic!(0.03), // dividend_yield
            Positive::ONE,       // expiration_in_days
        );
        let veta = veta(&option).unwrap().to_f64().unwrap();
        let expected_veta = 0.0001511559;
        assert!(
            (veta - expected_veta).abs() < 1e-5,
            "Veta with dividends test failed: expected {expected_veta}, got {veta}"
        );
    }

    #[test]
    fn test_veta_itm() {
        let option = create_test_option(
            pos_or_panic!(110.0), // underlying_price
            Positive::HUNDRED,    // strike_price
            pos_or_panic!(0.2),   // implied_volatility
            Positive::ZERO,       // dividend_yield
            Positive::ONE,        // expiration_in_days
        );
        let veta = veta(&option).unwrap().to_f64().unwrap();
        let expected_veta = 0.0;
        assert!(
            (veta - expected_veta).abs() < 1e-5,
            "Veta ITM test failed: expected {expected_veta}, got {veta}"
        );
    }
}

#[cfg(test)]
pub mod tests_charm_equations {
    use super::*;
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option_with_days;

    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use positive::pos_or_panic;
    use tracing::info;

    #[test]
    fn test_charm_call_itm() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,   // underlying price
            Positive::ONE,       // quantity
            pos_or_panic!(95.0), // strike price
            pos_or_panic!(0.3),  // volatility
            pos_or_panic!(30.0), // expiration days
        );
        let charm_value = charm(&option).unwrap();
        info!("Charm Call ITM Value: {}", charm_value);
        assert_relative_eq!(
            charm_value.to_f64().unwrap(),
            0.00274096463168,
            epsilon = 1e-8
        );
    }

    #[test]
    fn test_charm_put_itm() {
        let option = create_sample_option_with_days(
            OptionStyle::Put,
            Side::Long,
            pos_or_panic!(95.0), // underlying price
            Positive::ONE,       // quantity
            Positive::HUNDRED,   // strike price
            pos_or_panic!(0.3),  // volatility
            pos_or_panic!(30.0), // expiration days
        );
        let charm_value = charm(&option).unwrap();
        info!("Charm Put ITM Value: {}", charm_value);
        assert_relative_eq!(
            charm_value.to_f64().unwrap(),
            -0.0039614286773,
            epsilon = 1e-8
        );
    }

    #[test]
    fn test_charm_call_atm() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(95.0), // underlying price
            Positive::ONE,       // quantity
            pos_or_panic!(95.0), // strike price
            pos_or_panic!(0.3),  // volatility
            pos_or_panic!(30.0), // expiration days
        );
        let charm_value = charm(&option).unwrap();
        info!("Charm Call ATM Value: {}", charm_value);
        assert_relative_eq!(
            charm_value.to_f64().unwrap(),
            -0.000523300995754,
            epsilon = 1e-8
        );
    }

    #[test]
    fn test_charm_put_atm() {
        let option = create_sample_option_with_days(
            OptionStyle::Put,
            Side::Long,
            pos_or_panic!(95.0), // underlying price
            Positive::ONE,       // quantity
            pos_or_panic!(95.0), // strike price
            pos_or_panic!(0.3),  // volatility
            pos_or_panic!(30.0), // expiration days
        );
        let charm_value = charm(&option).unwrap();
        info!("Charm Put ATM Value: {}", charm_value);
        assert_relative_eq!(
            charm_value.to_f64().unwrap(),
            -0.000550675746984,
            epsilon = 1e-8
        );
    }

    #[test]
    fn test_charm_call_otm() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(90.0), // underlying price
            Positive::ONE,       // quantity
            pos_or_panic!(95.0), // strike price
            pos_or_panic!(0.3),  // volatility
            pos_or_panic!(30.0), // expiration days
        );
        let charm_value = charm(&option).unwrap();
        info!("Charm Call OTM Value: {}", charm_value);
        assert_relative_eq!(
            charm_value.to_f64().unwrap(),
            -0.00405183908388,
            epsilon = 1e-8
        );
    }

    #[test]
    fn test_charm_put_otm() {
        let option = create_sample_option_with_days(
            OptionStyle::Put,
            Side::Long,
            pos_or_panic!(95.0), // underlying price
            Positive::ONE,       // quantity
            pos_or_panic!(90.0), // strike price
            pos_or_panic!(0.3),  // volatility
            pos_or_panic!(30.0), // expiration days
        );
        let charm_value = charm(&option).unwrap();
        info!("Charm Put OTM Value: {}", charm_value);
        assert_relative_eq!(
            charm_value.to_f64().unwrap(),
            0.00282022266253,
            epsilon = 1e-8
        );
    }
}

/// Tests for second-order volatility Greeks (Vanna, Vomma, Veta) edge cases.
///
/// These tests cover:
/// - High and low volatility environments
/// - Near expiration scenarios
/// - Extreme changes in underlying price (deep ITM/OTM)
#[cfg(test)]
pub mod tests_volatility_greeks_edge_cases {
    use super::*;
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option_with_days;
    use positive::pos_or_panic;

    use tracing::info;

    // ==================== VANNA EDGE CASES ====================

    #[test]
    fn test_vanna_high_volatility() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.8), // High volatility (80%)
            pos_or_panic!(30.0),
        );
        let vanna_value = vanna(&option).unwrap();
        info!("Vanna High Volatility: {}", vanna_value);
        // Vanna should be smaller in absolute terms with high volatility
        assert!(vanna_value.abs() < Decimal::ONE);
    }

    #[test]
    fn test_vanna_low_volatility() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.05), // Low volatility (5%)
            pos_or_panic!(30.0),
        );
        let vanna_value = vanna(&option).unwrap();
        info!("Vanna Low Volatility: {}", vanna_value);
        // Vanna calculation should still work with low volatility
        assert!(vanna_value.abs() < Decimal::MAX);
    }

    #[test]
    fn test_vanna_near_expiration() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
            Positive::ONE, // 1 day to expiration
        );
        let vanna_value = vanna(&option).unwrap();
        info!("Vanna Near Expiration: {}", vanna_value);
        // Near expiration, vanna should still be calculable
        assert!(vanna_value.abs() < Decimal::MAX);
    }

    #[test]
    fn test_vanna_deep_itm() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(150.0), // Deep ITM (underlying >> strike)
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
            pos_or_panic!(30.0),
        );
        let vanna_value = vanna(&option).unwrap();
        info!("Vanna Deep ITM: {}", vanna_value);
        // Deep ITM options have small vanna
        assert!(vanna_value.abs() < Decimal::ONE);
    }

    #[test]
    fn test_vanna_deep_otm() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(50.0), // Deep OTM (underlying << strike)
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
            pos_or_panic!(30.0),
        );
        let vanna_value = vanna(&option).unwrap();
        info!("Vanna Deep OTM: {}", vanna_value);
        // Deep OTM options have small vanna
        assert!(vanna_value.abs() < Decimal::ONE);
    }

    #[test]
    fn test_vanna_zero_volatility() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            Positive::ZERO, // Zero volatility
            pos_or_panic!(30.0),
        );
        let vanna_value = vanna(&option).unwrap();
        info!("Vanna Zero Volatility: {}", vanna_value);
        // With zero volatility, vanna should be zero
        assert_eq!(vanna_value, Decimal::ZERO);
    }

    // ==================== VOMMA EDGE CASES ====================

    #[test]
    fn test_vomma_high_volatility() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.8), // High volatility (80%)
            pos_or_panic!(30.0),
        );
        let vomma_value = vomma(&option).unwrap();
        info!("Vomma High Volatility: {}", vomma_value);
        // Vomma should be calculable with high volatility
        assert!(vomma_value.abs() < Decimal::MAX);
    }

    #[test]
    fn test_vomma_low_volatility() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.05), // Low volatility (5%)
            pos_or_panic!(30.0),
        );
        let vomma_value = vomma(&option).unwrap();
        info!("Vomma Low Volatility: {}", vomma_value);
        // Vomma should be calculable with low volatility
        assert!(vomma_value.abs() < Decimal::MAX);
    }

    #[test]
    fn test_vomma_near_expiration() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
            Positive::ONE, // 1 day to expiration
        );
        let vomma_value = vomma(&option).unwrap();
        info!("Vomma Near Expiration: {}", vomma_value);
        // Near expiration, vomma should still be calculable
        assert!(vomma_value.abs() < Decimal::MAX);
    }

    #[test]
    fn test_vomma_at_expiration() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
            Positive::ZERO, // At expiration
        );
        let vomma_value = vomma(&option).unwrap();
        info!("Vomma At Expiration: {}", vomma_value);
        // At expiration, vomma should be zero
        assert_eq!(vomma_value, Decimal::ZERO);
    }

    #[test]
    fn test_vomma_deep_otm() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(50.0), // Deep OTM
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
            pos_or_panic!(30.0),
        );
        let vomma_value = vomma(&option).unwrap();
        info!("Vomma Deep OTM: {}", vomma_value);
        // Deep OTM options have highest vomma
        assert!(vomma_value.abs() < Decimal::MAX);
    }

    // ==================== VETA EDGE CASES ====================

    #[test]
    fn test_veta_high_volatility() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.8), // High volatility (80%)
            pos_or_panic!(30.0),
        );
        let veta_value = veta(&option).unwrap();
        info!("Veta High Volatility: {}", veta_value);
        // Veta should be calculable with high volatility
        assert!(veta_value.abs() < Decimal::MAX);
    }

    #[test]
    fn test_veta_low_volatility() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.05), // Low volatility (5%)
            pos_or_panic!(30.0),
        );
        let veta_value = veta(&option).unwrap();
        info!("Veta Low Volatility: {}", veta_value);
        // Veta should be calculable with low volatility
        assert!(veta_value.abs() < Decimal::MAX);
    }

    #[test]
    fn test_veta_near_expiration() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
            Positive::ONE, // 1 day to expiration
        );
        let veta_value = veta(&option).unwrap();
        info!("Veta Near Expiration: {}", veta_value);
        // Near expiration, veta should still be calculable
        assert!(veta_value.abs() < Decimal::MAX);
    }

    #[test]
    fn test_veta_at_expiration() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
            Positive::ZERO, // At expiration
        );
        let veta_value = veta(&option).unwrap();
        info!("Veta At Expiration: {}", veta_value);
        // At expiration, veta should be zero
        assert_eq!(veta_value, Decimal::ZERO);
    }

    #[test]
    fn test_veta_deep_itm() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(150.0), // Deep ITM
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
            pos_or_panic!(30.0),
        );
        let veta_value = veta(&option).unwrap();
        info!("Veta Deep ITM: {}", veta_value);
        // Deep ITM options have small veta
        assert!(veta_value.abs() < Decimal::MAX);
    }

    #[test]
    fn test_veta_deep_otm() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(50.0), // Deep OTM
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
            pos_or_panic!(30.0),
        );
        let veta_value = veta(&option).unwrap();
        info!("Veta Deep OTM: {}", veta_value);
        // Deep OTM options have small veta
        assert!(veta_value.abs() < Decimal::MAX);
    }

    #[test]
    fn test_veta_long_dated_option() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
            pos_or_panic!(365.0), // 1 year to expiration
        );
        let veta_value = veta(&option).unwrap();
        info!("Veta Long Dated: {}", veta_value);
        // Long dated options should have calculable veta
        assert!(veta_value.abs() < Decimal::MAX);
    }

    // ==================== CHARM EDGE CASES ====================

    #[test]
    fn test_charm_high_volatility() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED, // underlying_price
            Positive::ONE,
            Positive::HUNDRED,   // strike_price
            pos_or_panic!(0.8),  // High volatility (80%)
            pos_or_panic!(30.0), // expiration_days
        );
        let charm_value = charm(&option).unwrap();
        info!("Charm High Volatility: {}", charm_value);
        // Charm when is far from expiration is negligible
        assert!(charm_value.abs() < Decimal::ONE);
    }

    #[test]
    fn test_charm_low_volatility() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED, // underlying_price
            Positive::ONE,
            Positive::HUNDRED,   // strike_price
            pos_or_panic!(0.05), // Low volatility (5%)
            pos_or_panic!(30.0), // expiration_days
        );
        let charm_value = charm(&option).unwrap();
        info!("Charm Low Volatility: {}", charm_value);
        // Charm when is far from expiration is negligible
        // low volatility increases the value
        assert!(charm_value.abs() < Decimal::ONE);
    }

    #[test]
    fn test_charm_near_expiration() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED, // underlying_price
            Positive::ONE,
            Positive::HUNDRED, // strike_price
            pos_or_panic!(0.2),
            Positive::ONE, // 1 day to expiration
        );
        let charm_value = charm(&option).unwrap();
        info!("Charm Near Expiration: {}", charm_value);
        // Near expiration Charm is increasing
        assert!(charm_value.abs() < Decimal::ONE);
    }

    #[test]
    fn test_charm_at_expiration() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED, // underlying_price
            Positive::ONE,
            Positive::HUNDRED, // strike_price
            pos_or_panic!(0.2),
            Positive::ZERO, // At expiration
        );
        let charm_value = charm(&option).unwrap();
        info!("Charm At Expiration: {}", charm_value);
        // At expiration, Charm should be zero
        assert_eq!(charm_value, Decimal::ZERO);
    }

    #[test]
    fn test_charm_deep_itm() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(150.0), // Deep ITM
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
            pos_or_panic!(30.0), // expiration_days
        );
        let charm_value = charm(&option).unwrap();
        info!("Charm Deep ITM: {}", charm_value);
        // Deep ITM options far from expiration have small Charm
        assert!(charm_value.abs() < Decimal::ONE);
    }

    #[test]
    fn test_charm_deep_otm() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(50.0), // Deep OTM
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
            pos_or_panic!(30.0), // expiration_days
        );
        let charm_value = charm(&option).unwrap();
        info!("Charm Deep OTM: {}", charm_value);
        // For deep OTM options Charm should be zero
        assert_eq!(charm_value.abs(), Decimal::ZERO);
    }

    #[test]
    fn test_charm_long_dated_option() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
            pos_or_panic!(365.0), // 1 year to expiration
        );
        let charm_value = charm(&option).unwrap();
        info!("Charm Long Dated: {}", charm_value);
        // Long dated options should have calculable Charm
        assert!(charm_value.abs() < Decimal::ONE);
    }

    // ==================== COLOR EDGE CASES ====================

    #[test]
    fn test_color_high_volatility() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED, // underlying_price
            Positive::ONE,
            Positive::HUNDRED,   // strike_price
            pos_or_panic!(0.8),  // High volatility (80%)
            pos_or_panic!(30.0), // expiration_days
        );
        let color_value = color(&option).unwrap();
        info!("Color High Volatility: {}", color_value);
        // Color when is far from expiration is negligible
        assert!(color_value.abs() < Decimal::ONE);
    }

    #[test]
    fn test_color_low_volatility() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED, // underlying_price
            Positive::ONE,
            Positive::HUNDRED,   // strike_price
            pos_or_panic!(0.05), // Low volatility (5%)
            pos_or_panic!(30.0), // expiration_days
        );
        let color_value = color(&option).unwrap();
        info!("Color Low Volatility: {}", color_value);
        // Color when is far from expiration is negligible
        // low volatility increases the value
        assert!(color_value.abs() < Decimal::ONE);
    }

    #[test]
    fn test_color_near_expiration() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED, // underlying_price
            Positive::ONE,
            Positive::HUNDRED, // strike_price
            pos_or_panic!(0.2),
            Positive::ONE, // 1 day to expiration
        );
        let color_value = color(&option).unwrap();
        info!("Color Near Expiration: {}", color_value);
        // Near expiration Color is increasing
        assert!(color_value.abs() < Decimal::ONE);
    }

    #[test]
    fn test_color_at_expiration() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED, // underlying_price
            Positive::ONE,
            Positive::HUNDRED, // strike_price
            pos_or_panic!(0.2),
            Positive::ZERO, // At expiration
        );
        let color_value = color(&option).unwrap();
        info!("Color At Expiration: {}", color_value);
        // At expiration, Color should be zero
        assert_eq!(color_value, Decimal::ZERO);
    }

    #[test]
    fn test_color_deep_itm() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(150.0), // Deep ITM
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
            pos_or_panic!(30.0), // expiration_days
        );
        let color_value = color(&option).unwrap();
        info!("Color Deep ITM: {}", color_value);
        // Deep ITM options far from expiration have small Color
        assert!(color_value.abs() < Decimal::ONE);
    }

    #[test]
    fn test_color_deep_otm() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(50.0), // Deep OTM
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
            pos_or_panic!(30.0), // expiration_days
        );
        let color_value = color(&option).unwrap();
        info!("Color Deep OTM: {}", color_value);
        // For deep OTM options Color should be zero
        assert_eq!(color_value.abs(), Decimal::ZERO);
    }

    #[test]
    fn test_color_long_dated_option() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
            pos_or_panic!(365.0), // 1 year to expiration
        );
        let color_value = color(&option).unwrap();
        info!("Color Long Dated: {}", color_value);
        // Long dated options should have calculable Color
        assert!(color_value.abs() < Decimal::ONE);
    }

    // ==================== COMBINED SCENARIOS ====================

    #[test]
    fn test_volatility_greeks_extreme_scenario() {
        // High volatility + near expiration + ATM
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            Positive::ONE, // 100% volatility
            Positive::TWO, // 2 days to expiration
        );

        let vanna_value = vanna(&option).unwrap();
        let vomma_value = vomma(&option).unwrap();
        let veta_value = veta(&option).unwrap();
        let charm_value = charm(&option).unwrap();
        let color_value = color(&option).unwrap();

        info!("Extreme Scenario - Vanna: {}", vanna_value);
        info!("Extreme Scenario - Vomma: {}", vomma_value);
        info!("Extreme Scenario - Veta: {}", veta_value);
        info!("Extreme Scenario - Charm: {}", charm_value);
        info!("Extreme Scenario - Color: {}", color_value);

        // All should be finite values
        assert!(vanna_value.abs() < Decimal::MAX);
        assert!(vomma_value.abs() < Decimal::MAX);
        assert!(veta_value.abs() < Decimal::MAX);
        assert!(charm_value.abs() < Decimal::MAX);
        assert!(color_value.abs() < Decimal::MAX);
    }

    #[test]
    fn test_volatility_greeks_put_option() {
        let option = create_sample_option_with_days(
            OptionStyle::Put,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
            pos_or_panic!(30.0),
        );

        let vanna_value = vanna(&option).unwrap();
        let vomma_value = vomma(&option).unwrap();
        let veta_value = veta(&option).unwrap();
        let charm_value = charm(&option).unwrap();
        let color_value = color(&option).unwrap();

        info!("Put Option - Vanna: {}", vanna_value);
        info!("Put Option - Vomma: {}", vomma_value);
        info!("Put Option - Veta: {}", veta_value);
        info!("Put Option - Charm: {}", charm_value);
        info!("Put Option - Color: {}", color_value);

        // All should be finite values
        assert!(vanna_value.abs() < Decimal::MAX);
        assert!(vomma_value.abs() < Decimal::MAX);
        assert!(veta_value.abs() < Decimal::MAX);
        assert!(charm_value.abs() < Decimal::MAX);
        assert!(color_value.abs() < Decimal::MAX);
    }

    #[test]
    fn test_vanna_atm_vs_otm_comparison() {
        let atm_option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED, // ATM
            pos_or_panic!(0.2),
            pos_or_panic!(30.0),
        );

        let otm_option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            pos_or_panic!(110.0), // OTM
            pos_or_panic!(0.2),
            pos_or_panic!(30.0),
        );

        let vanna_atm = vanna(&atm_option).unwrap();
        let vanna_otm = vanna(&otm_option).unwrap();

        info!("Vanna ATM: {}", vanna_atm);
        info!("Vanna OTM: {}", vanna_otm);

        // Both should be calculable
        assert!(vanna_atm.abs() < Decimal::MAX);
        assert!(vanna_otm.abs() < Decimal::MAX);
    }

    #[test]
    fn test_vomma_smile_effect() {
        // Test vomma at different strikes to verify smile effect
        let strikes = vec![
            pos_or_panic!(90.0),
            pos_or_panic!(95.0),
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(110.0),
        ];

        for strike in strikes {
            let option = create_sample_option_with_days(
                OptionStyle::Call,
                Side::Long,
                Positive::HUNDRED,
                Positive::ONE,
                strike,
                pos_or_panic!(0.2),
                pos_or_panic!(30.0),
            );
            let vomma_value = vomma(&option).unwrap();
            info!("Vomma at strike {}: {}", strike, vomma_value);
            assert!(vomma_value.abs() < Decimal::MAX);
        }
    }
}

#[cfg(test)]
pub mod tests_color_equations {
    use super::*;
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option_with_days;

    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use positive::pos_or_panic;
    use tracing::info;

    #[test]
    fn test_color_itm() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,   // underlying price
            Positive::ONE,       // quantity
            pos_or_panic!(95.0), // strike price
            pos_or_panic!(0.3),  // volatility
            pos_or_panic!(30.0), // expiration days
        );
        let color_value = color(&option).unwrap();
        info!("Color ITM Value: {}", color_value);
        assert_relative_eq!(
            color_value.to_f64().unwrap(),
            -0.000400671355466,
            epsilon = 1e-8
        );
    }

    #[test]
    fn test_color_atm() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(95.0), // underlying price
            Positive::ONE,       // quantity
            pos_or_panic!(95.0), // strike price
            pos_or_panic!(0.3),  // volatility
            pos_or_panic!(30.0), // expiration days
        );
        let color_value = color(&option).unwrap();
        info!("Color ATM Value: {}", color_value);
        assert_relative_eq!(
            color_value.to_f64().unwrap(),
            -0.000817099264221,
            epsilon = 1e-8
        );
    }

    #[test]
    fn test_color_atm_near_expiration() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(95.0), // underlying price
            Positive::ONE,       // quantity
            pos_or_panic!(95.0), // strike price
            pos_or_panic!(0.3),  // volatility
            pos_or_panic!(0.5),  // expiration days
        );
        let color_value = color(&option).unwrap();
        info!("Color ATM Near Expiration Value: {}", color_value);
        assert_relative_eq!(
            color_value.to_f64().unwrap(),
            -0.378230424889,
            epsilon = 1e-8
        );
    }

    #[test]
    fn test_color_atm_right_before_expiration() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(95.0),  // underlying price
            Positive::ONE,        // quantity
            pos_or_panic!(95.0),  // strike price
            pos_or_panic!(0.3),   // volatility
            pos_or_panic!(0.001), // expiration days
        );
        let color_value = color(&option).unwrap();
        info!("Color ATM Right Before Expiration Value: {}", color_value);
        assert_relative_eq!(
            color_value.to_f64().unwrap(),
            -4228.4548921660125,
            epsilon = 1e-8
        );
    }

    #[test]
    fn test_color_otm() {
        let option = create_sample_option_with_days(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(90.0), // underlying price
            Positive::ONE,       // quantity
            pos_or_panic!(95.0), // strike price
            pos_or_panic!(0.3),  // volatility
            pos_or_panic!(30.0), // expiration days
        );
        let color_value = color(&option).unwrap();
        info!("Color OTM Value: {}", color_value);
        assert_relative_eq!(
            color_value.to_f64().unwrap(),
            -0.000452958052918,
            epsilon = 1e-8
        );
    }
}

#[cfg(test)]
mod tests_shared_kernel_equivalence {
    use super::*;
    use crate::greeks::utils::d2 as fresh_d2_fn;
    use crate::model::types::OptionType;
    use crate::{ExpirationDate, Options};
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    /// Every branch `kernels_for` distinguishes, so the fast path and the
    /// per-greek fallbacks are both exercised.
    fn branches() -> Vec<(&'static str, OptionType, Positive, Positive)> {
        vec![
            // name, type, time to expiry in days, implied volatility
            (
                "live european",
                OptionType::European,
                pos_or_panic!(30.0),
                pos_or_panic!(0.2),
            ),
            (
                "at expiry",
                OptionType::European,
                Positive::ZERO,
                pos_or_panic!(0.2),
            ),
            (
                "zero volatility",
                OptionType::European,
                pos_or_panic!(30.0),
                Positive::ZERO,
            ),
            (
                "non european",
                OptionType::American,
                pos_or_panic!(30.0),
                pos_or_panic!(0.2),
            ),
        ]
    }

    fn option_for(
        option_type: OptionType,
        days: Positive,
        implied_volatility: Positive,
        style: OptionStyle,
        side: Side,
        quantity: Positive,
    ) -> Options {
        Options::new(
            option_type,
            side,
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(days),
            implied_volatility,
            quantity,
            pos_or_panic!(105.0),
            dec!(0.05),
            style,
            pos_or_panic!(0.02),
            None,
        )
    }

    /// The aggregate shares one set of kernels across the twelve greeks and
    /// reuses gamma and theta for alpha, while the public functions each build
    /// their own. The two must agree exactly, in every branch, for both styles
    /// and both sides — including on failure, so the fast path can neither
    /// succeed where an individual greek errors nor error where they all
    /// succeed.
    ///
    /// The zero-volatility branch is the one that fails today: seven of the
    /// twelve have no guard for it, so `greeks()` errors there while `delta`
    /// and `gamma` still return their discrete values.
    #[test]
    fn test_greeks_matches_the_individual_functions_exactly() {
        for (name, option_type, days, iv) in branches() {
            for style in [OptionStyle::Call, OptionStyle::Put] {
                for side in [Side::Long, Side::Short] {
                    let option = option_for(
                        option_type.clone(),
                        days,
                        iv,
                        style,
                        side,
                        pos_or_panic!(3.0),
                    );
                    let label = format!("{name} {style:?} {side:?}");

                    let individual = [
                        ("delta", delta(&option)),
                        ("gamma", gamma(&option)),
                        ("theta", theta(&option)),
                        ("vega", vega(&option)),
                        ("rho", rho(&option)),
                        ("rho_d", rho_d(&option)),
                        ("alpha", alpha(&option)),
                        ("vanna", vanna(&option)),
                        ("vomma", vomma(&option)),
                        ("veta", veta(&option)),
                        ("charm", charm(&option)),
                        ("color", color(&option)),
                    ];
                    let all_ok = individual.iter().all(|(_, r)| r.is_ok());

                    match (option.greeks(), all_ok) {
                        (Ok(aggregate), true) => {
                            let values = [
                                aggregate.delta,
                                aggregate.gamma,
                                aggregate.theta,
                                aggregate.vega,
                                aggregate.rho,
                                aggregate.rho_d,
                                aggregate.alpha,
                                aggregate.vanna,
                                aggregate.vomma,
                                aggregate.veta,
                                aggregate.charm,
                                aggregate.color,
                            ];
                            for ((greek, single), aggregated) in
                                individual.iter().zip(values.iter())
                            {
                                assert_eq!(
                                    aggregated,
                                    &expect(single, &label),
                                    "{greek} disagrees for {label}"
                                );
                            }
                        }
                        (Err(_), false) => {
                            // Both paths reject these inputs, which is the
                            // contract for the zero-volatility branch.
                        }
                        (Ok(_), false) => {
                            panic!("greeks() succeeded for {label} where an individual greek fails")
                        }
                        (Err(e), true) => panic!(
                            "greeks() failed for {label} where every individual greek succeeds: {e}"
                        ),
                    }
                }
            }
        }
    }

    fn expect(result: &Result<Decimal, GreeksError>, label: &str) -> Decimal {
        match result {
            Ok(value) => *value,
            Err(e) => panic!("individual greek failed for {label}: {e}"),
        }
    }

    /// The aggregate now sums option by option rather than greek by greek.
    /// `Decimal` addition is exact, so a multi-leg position must match the sum
    /// of the per-option aggregates term by term.
    #[test]
    fn test_greeks_aggregation_is_order_independent() {
        struct Legs(Vec<Options>);
        impl Greeks for Legs {
            fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
                Ok(self.0.iter().collect())
            }
        }

        let legs = Legs(vec![
            option_for(
                OptionType::European,
                pos_or_panic!(30.0),
                pos_or_panic!(0.2),
                OptionStyle::Call,
                Side::Long,
                pos_or_panic!(2.0),
            ),
            option_for(
                OptionType::European,
                pos_or_panic!(45.0),
                pos_or_panic!(0.35),
                OptionStyle::Put,
                Side::Short,
                pos_or_panic!(3.0),
            ),
            // A degenerate leg alongside live ones, so the mixed path is covered.
            option_for(
                OptionType::European,
                Positive::ZERO,
                pos_or_panic!(0.2),
                OptionStyle::Call,
                Side::Short,
                Positive::ONE,
            ),
        ]);

        let Ok(total) = legs.greeks() else {
            panic!("aggregate greeks should succeed");
        };

        let mut delta_sum = Decimal::ZERO;
        let mut charm_sum = Decimal::ZERO;
        let mut alpha_sum = Decimal::ZERO;
        for option in &legs.0 {
            delta_sum += expect(&delta(option), "leg");
            charm_sum += expect(&charm(option), "leg");
            alpha_sum += expect(&alpha(option), "leg");
        }
        assert_eq!(total.delta, delta_sum);
        assert_eq!(total.charm, charm_sum);
        assert_eq!(total.alpha, alpha_sum);
    }

    /// Each cached kernel must equal the value the pre-refactor code derived
    /// from scratch, which is what makes the sharing value-preserving rather
    /// than an approximation.
    #[test]
    fn test_cached_kernels_match_freshly_derived_values() {
        let option = option_for(
            OptionType::European,
            pos_or_panic!(30.0),
            pos_or_panic!(0.2),
            OptionStyle::Call,
            Side::Long,
            Positive::ONE,
        );
        let Ok(t) = option.expiration_date.get_years() else {
            panic!("expiration should resolve");
        };
        let Ok(kernels) = BlackScholesKernels::new(&option, t) else {
            panic!("kernels should build for a live european option");
        };
        let carry = option.risk_free_rate - option.dividend_yield.to_dec();

        let Ok(fresh_d1) = d1(
            option.underlying_price,
            option.strike_price,
            carry,
            t,
            option.implied_volatility,
        ) else {
            panic!("d1 should compute");
        };
        let Ok(fresh_d2) = fresh_d2_fn(
            option.underlying_price,
            option.strike_price,
            carry,
            t,
            option.implied_volatility,
        ) else {
            panic!("d2 should compute");
        };

        assert_eq!(kernels.d1(), fresh_d1, "d1");
        assert_eq!(kernels.d2().ok(), Some(fresh_d2), "d2 derived from d1");
        assert_eq!(kernels.sqrt_t(), t.sqrt(), "sqrt(T)");
        assert_eq!(kernels.n_d1().ok(), n(fresh_d1).ok(), "n(d1)");
        assert_eq!(kernels.big_n_d1().ok(), big_n(fresh_d1).ok(), "big_n(d1)");
        assert_eq!(
            kernels.big_n_neg_d1().ok(),
            big_n(-fresh_d1).ok(),
            "big_n(-d1)"
        );
        assert_eq!(kernels.big_n_d2().ok(), big_n(fresh_d2).ok(), "big_n(d2)");
        assert_eq!(
            kernels.big_n_neg_d2().ok(),
            big_n(-fresh_d2).ok(),
            "big_n(-d2)"
        );
        assert_eq!(
            kernels.exp_minus_qt().ok(),
            d_exp(-t.to_dec() * option.dividend_yield, "test::exp_minus_qt").ok(),
            "exp(-qT)"
        );
        assert_eq!(
            kernels.exp_minus_rt().ok(),
            d_exp(-option.risk_free_rate * t, "test::exp_minus_rt").ok(),
            "exp(-rT)"
        );
    }
}

#[cfg(test)]
mod tests_side_sign_convention {
    use super::*;
    use crate::model::types::OptionType;
    use crate::{ExpirationDate, Options};
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    fn option(style: OptionStyle, side: Side, quantity: Positive) -> Options {
        Options::new(
            OptionType::European,
            side,
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            quantity,
            pos_or_panic!(105.0),
            dec!(0.05),
            style,
            pos_or_panic!(0.02),
            None,
        )
    }

    fn ok(result: Result<Decimal, GreeksError>, what: &str) -> Decimal {
        match result {
            Ok(value) => value,
            Err(e) => panic!("{what} should compute: {e}"),
        }
    }

    fn binary_option(side: Side, quantity: Positive) -> Options {
        Options::new(
            OptionType::Binary {
                binary_type: crate::model::types::BinaryType::CashOrNothing,
            },
            side,
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            quantity,
            pos_or_panic!(105.0),
            dec!(0.05),
            OptionStyle::Call,
            pos_or_panic!(0.02),
            None,
        )
    }

    /// `delta` and `gamma` fall back to the numerical engine for anything that
    /// is not European. That path prices through `price_option`, which takes the
    /// absolute value, so it produces a per-contract long sensitivity and used
    /// to return it unchanged: a short exotic reported the same exposure as its
    /// long equivalent, and any quantity above one was under-scaled.
    #[test]
    fn test_non_european_fallback_is_signed_and_scaled() {
        let one_long = binary_option(Side::Long, Positive::ONE);
        let Ok(delta_one) = delta(&one_long) else {
            panic!("a binary option should price through the numerical fallback");
        };
        let Ok(gamma_one) = gamma(&one_long) else {
            panic!("a binary option should price through the numerical fallback");
        };
        assert!(
            delta_one != Decimal::ZERO,
            "the fixture must produce a non-zero delta for this test to mean anything"
        );

        // Scaled by quantity.
        let three_long = binary_option(Side::Long, pos_or_panic!(3.0));
        assert_eq!(ok(delta(&three_long), "delta"), delta_one * dec!(3));
        assert_eq!(ok(gamma(&three_long), "gamma"), gamma_one * dec!(3));

        // Signed by side.
        let three_short = binary_option(Side::Short, pos_or_panic!(3.0));
        assert_eq!(ok(delta(&three_short), "delta"), -delta_one * dec!(3));
        assert_eq!(ok(gamma(&three_short), "gamma"), -gamma_one * dec!(3));
    }

    #[test]
    fn test_signed_quantity_carries_the_side() {
        for quantity in [Positive::ONE, Positive::TWO, pos_or_panic!(7.5)] {
            let long = option(OptionStyle::Call, Side::Long, quantity);
            let short = option(OptionStyle::Call, Side::Short, quantity);
            assert_eq!(signed_quantity(&long), quantity.to_dec());
            assert_eq!(signed_quantity(&short), -quantity.to_dec());
        }
    }

    /// Flipping the side must negate eleven of the twelve. `alpha` is the
    /// exception by construction, being the ratio `gamma / theta`.
    #[test]
    fn test_flipping_the_side_negates_every_greek_except_alpha() {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            let long = option(style, Side::Long, pos_or_panic!(3.0));
            let short = option(style, Side::Short, pos_or_panic!(3.0));

            for (name, l, s) in [
                ("delta", delta(&long), delta(&short)),
                ("gamma", gamma(&long), gamma(&short)),
                ("theta", theta(&long), theta(&short)),
                ("vega", vega(&long), vega(&short)),
                ("rho", rho(&long), rho(&short)),
                ("rho_d", rho_d(&long), rho_d(&short)),
                ("vanna", vanna(&long), vanna(&short)),
                ("vomma", vomma(&long), vomma(&short)),
                ("veta", veta(&long), veta(&short)),
                ("charm", charm(&long), charm(&short)),
                ("color", color(&long), color(&short)),
            ] {
                let long_value = ok(l, name);
                assert_eq!(
                    ok(s, name),
                    -long_value,
                    "{name} should negate when the side flips, for {style:?}"
                );
            }

            // The ratio is invariant: a short negates numerator and denominator.
            assert_eq!(
                ok(alpha(&short), "alpha"),
                ok(alpha(&long), "alpha"),
                "alpha is a ratio and must not change with the side"
            );
        }
    }

    /// A long and a short of the same contract are a closed position, so every
    /// greek must net to zero.
    #[test]
    fn test_offsetting_legs_net_to_zero() {
        struct Legs(Vec<Options>);
        impl Greeks for Legs {
            fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
                Ok(self.0.iter().collect())
            }
        }

        let legs = Legs(vec![
            option(OptionStyle::Call, Side::Long, pos_or_panic!(4.0)),
            option(OptionStyle::Call, Side::Short, pos_or_panic!(4.0)),
        ]);
        let Ok(g) = legs.greeks() else {
            panic!("aggregate should compute");
        };

        for (name, value) in [
            ("delta", g.delta),
            ("gamma", g.gamma),
            ("theta", g.theta),
            ("vega", g.vega),
            ("rho", g.rho),
            ("rho_d", g.rho_d),
            ("vanna", g.vanna),
            ("vomma", g.vomma),
            ("veta", g.veta),
            ("charm", g.charm),
            ("color", g.color),
        ] {
            assert_eq!(value, Decimal::ZERO, "{name} should net to zero");
        }
    }

    /// A short premium position collects decay, so its theta is positive.
    #[test]
    fn test_short_premium_theta_is_positive() {
        let short_call = option(OptionStyle::Call, Side::Short, Positive::ONE);
        assert!(
            ok(theta(&short_call), "theta").is_sign_positive(),
            "a short call collects decay, so theta must be positive"
        );
        let long_call = option(OptionStyle::Call, Side::Long, Positive::ONE);
        assert!(
            ok(theta(&long_call), "theta").is_sign_negative(),
            "a long call pays decay, so theta must be negative"
        );
    }
}

#[cfg(test)]
mod tests_checked_aggregation {
    use super::*;
    use crate::model::types::OptionType;
    use crate::{ExpirationDate, Options};
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    /// A quantity small enough that the daily theta of an ordinary at-the-money
    /// contract rounds to zero at `Decimal`'s twenty-eight-digit scale while its
    /// gamma still rounds to a non-zero value. That is exactly the state
    /// [`alpha_from`] answers with the `Decimal::MAX` sentinel.
    const SENTINEL_QUANTITY: Decimal = Decimal::from_parts(1, 0, 0, false, 27);

    /// The smallest thing that implements [`Greeks`]: a bare list of legs.
    struct Legs(Vec<Options>);

    impl Greeks for Legs {
        fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
            Ok(self.0.iter().collect())
        }
    }

    fn positive(value: Decimal) -> Positive {
        match Positive::new_decimal(value) {
            Ok(v) => v,
            Err(e) => panic!("fixture quantity should be positive: {e}"),
        }
    }

    fn leg(
        option_type: OptionType,
        strike: Positive,
        days: Positive,
        implied_volatility: Positive,
        style: OptionStyle,
        side: Side,
        quantity: Positive,
    ) -> Options {
        Options::new(
            option_type,
            side,
            "TEST".to_string(),
            strike,
            ExpirationDate::Days(days),
            implied_volatility,
            quantity,
            Positive::HUNDRED,
            dec!(0.05),
            style,
            Positive::ZERO,
            None,
        )
    }

    /// An at-the-money call at [`SENTINEL_QUANTITY`], whose alpha is the
    /// `Decimal::MAX` sentinel.
    fn sentinel_leg() -> Options {
        leg(
            OptionType::European,
            Positive::HUNDRED,
            pos_or_panic!(30.0),
            pos_or_panic!(0.2),
            OptionStyle::Call,
            Side::Long,
            positive(SENTINEL_QUANTITY),
        )
    }

    fn ok(result: Result<Decimal, GreeksError>, what: &str) -> Decimal {
        match result {
            Ok(value) => value,
            Err(e) => panic!("{what} should compute: {e}"),
        }
    }

    /// Asserts that `result` is the sentinel refusal, and that its message
    /// names the offending leg.
    fn expect_sentinel_refusal(result: Result<Decimal, GreeksError>, what: &str) {
        match result {
            Ok(value) => panic!("{what} should be refused, got {value}"),
            Err(GreeksError::CalculationError(CalculationErrorKind::ThetaError { reason })) => {
                assert!(
                    reason.contains("leg ") && reason.contains("sentinel"),
                    "the refusal should name the leg and the sentinel, got {reason}"
                );
            }
            Err(e) => panic!("{what} should report the sentinel refusal, got {e}"),
        }
    }

    /// The fixture has to actually reach the sentinel, otherwise the two-leg
    /// tests below prove nothing.
    #[test]
    fn test_alpha_sub_contract_quantity_returns_the_sentinel() {
        let option = sentinel_leg();
        assert_eq!(
            ok(theta(&option), "theta"),
            Decimal::ZERO,
            "the fixture's daily theta must round to zero"
        );
        assert_ne!(
            ok(gamma(&option), "gamma"),
            Decimal::ZERO,
            "the fixture's gamma must stay non-zero, or alpha takes the zero branch"
        );
        assert_eq!(
            ok(alpha(&option), "alpha"),
            Decimal::MAX,
            "a vanished theta against a live gamma is the sentinel"
        );
    }

    /// The defect this module exists for: two legs at the sentinel used to sum
    /// `Decimal::MAX + Decimal::MAX` with the raw operator and abort the
    /// process. The aggregate refuses them instead.
    #[test]
    fn test_alpha_two_sentinel_legs_are_refused() {
        let legs = Legs(vec![sentinel_leg(), sentinel_leg()]);
        expect_sentinel_refusal(legs.alpha(), "the aggregate alpha of two sentinel legs");
    }

    /// Same for the twelve-field aggregate, which is the call the strategies
    /// panic-freedom proptest used to skip.
    #[test]
    fn test_greeks_two_sentinel_legs_are_refused() {
        let legs = Legs(vec![sentinel_leg(), sentinel_leg()]);
        match legs.greeks() {
            Ok(g) => panic!("greeks() should refuse two sentinel legs, got {g:?}"),
            Err(GreeksError::CalculationError(CalculationErrorKind::ThetaError { reason })) => {
                assert!(
                    reason.contains("sentinel"),
                    "the refusal should mention the sentinel, got {reason}"
                );
            }
            Err(e) => panic!("greeks() should report the sentinel refusal, got {e}"),
        }
    }

    /// A sentinel leg beside an ordinary one is refused, in either order.
    ///
    /// This is the case `checked_add` cannot detect and so the one an explicit
    /// guard has to catch: `Decimal::MAX` plus a value below one rescales and
    /// rounds straight back to `Decimal::MAX`, so the addition genuinely
    /// succeeds and returns `Some`, with the accumulator standing still and the
    /// ordinary leg's alpha dropped. Reporting an aggregate that is wrong in a
    /// way the caller cannot see is worse than reporting nothing.
    #[test]
    fn test_alpha_sentinel_beside_an_ordinary_leg_is_refused() {
        // An at-the-money call at 80% volatility, whose alpha is about -0.11.
        let ordinary = leg(
            OptionType::European,
            Positive::HUNDRED,
            pos_or_panic!(30.0),
            pos_or_panic!(0.8),
            OptionStyle::Call,
            Side::Long,
            Positive::ONE,
        );
        let ordinary_alpha = ok(alpha(&ordinary), "alpha");
        assert!(
            ordinary_alpha != Decimal::ZERO && ordinary_alpha.abs() < Decimal::ONE,
            "the fixture must contribute a small non-zero alpha, got {ordinary_alpha}"
        );
        // The unguarded arithmetic really does succeed and stand still, which
        // is what makes the guard necessary.
        assert_eq!(
            Decimal::MAX.checked_add(ordinary_alpha),
            Some(Decimal::MAX),
            "checked_add cannot detect this, so the guard must"
        );

        expect_sentinel_refusal(
            Legs(vec![sentinel_leg(), ordinary.clone()]).alpha(),
            "the aggregate alpha with the sentinel leg first",
        );
        expect_sentinel_refusal(
            Legs(vec![ordinary, sentinel_leg()]).alpha(),
            "the aggregate alpha with the sentinel leg last",
        );
    }

    /// A single sentinel leg is not a sum: nothing is dropped, so it keeps
    /// reporting the documented sentinel rather than an error.
    ///
    /// `OptionData::greeks_snapshot` depends on this. It calls `greeks()` on one
    /// option through `impl Greeks for Options` and maps an `alpha` of
    /// `Decimal::MAX` to `None` on the wire; refusing a lone sentinel would
    /// drop the other eleven greeks from that snapshot too.
    #[test]
    fn test_alpha_one_sentinel_leg_still_returns_the_sentinel() {
        let legs = Legs(vec![sentinel_leg()]);
        assert_eq!(ok(legs.alpha(), "alpha"), Decimal::MAX);
        let Ok(aggregate) = legs.greeks() else {
            panic!("a lone sentinel leg should still aggregate");
        };
        assert_eq!(aggregate.alpha, Decimal::MAX);
    }

    /// A sentinel beside a leg whose own alpha is zero is not refused either:
    /// `alpha` returns zero for a vanished gamma, and adding zero drops
    /// nothing. Refusing it would be a false positive.
    #[test]
    fn test_alpha_sentinel_beside_a_zero_alpha_leg_is_allowed() {
        // At expiry gamma vanishes, which is the branch `alpha_from` answers
        // with zero.
        let expired = leg(
            OptionType::European,
            Positive::HUNDRED,
            Positive::ZERO,
            pos_or_panic!(0.2),
            OptionStyle::Call,
            Side::Long,
            Positive::ONE,
        );
        assert_eq!(
            ok(alpha(&expired), "alpha"),
            Decimal::ZERO,
            "the fixture must contribute exactly zero"
        );
        assert_eq!(
            ok(Legs(vec![sentinel_leg(), expired.clone()]).alpha(), "alpha"),
            Decimal::MAX
        );
        assert_eq!(
            ok(Legs(vec![expired, sentinel_leg()]).alpha(), "alpha"),
            Decimal::MAX
        );
    }

    /// Every leg combination whose sums stay inside the `Decimal` range must
    /// aggregate to exactly what the raw `+` operator produced.
    ///
    /// `Decimal::checked_add` and `Add::add` both dispatch to the same
    /// `add_impl`; they differ only in whether a non-`Ok` result panics or
    /// yields `None`. This walks a matrix of legs and asserts the equality
    /// term by term, for the twelve-field aggregate and for each of the twelve
    /// per-greek aggregators.
    #[test]
    fn test_checked_aggregation_matches_the_unchecked_sum() {
        let mut options = Vec::new();
        for style in [OptionStyle::Call, OptionStyle::Put] {
            for side in [Side::Long, Side::Short] {
                for strike in [pos_or_panic!(80.0), Positive::HUNDRED, pos_or_panic!(120.0)] {
                    options.push(leg(
                        OptionType::European,
                        strike,
                        pos_or_panic!(30.0),
                        pos_or_panic!(0.2),
                        style,
                        side,
                        pos_or_panic!(3.0),
                    ));
                    options.push(leg(
                        OptionType::European,
                        strike,
                        pos_or_panic!(3650.0),
                        pos_or_panic!(0.8),
                        style,
                        side,
                        pos_or_panic!(0.25),
                    ));
                }
            }
        }
        let legs = Legs(options);

        // The reference: the pre-change accumulation, greek by greek, with the
        // raw operator. None of these legs approaches the range limit.
        let mut reference = Greek {
            delta: Decimal::ZERO,
            gamma: Decimal::ZERO,
            theta: Decimal::ZERO,
            vega: Decimal::ZERO,
            rho: Decimal::ZERO,
            rho_d: Decimal::ZERO,
            alpha: Decimal::ZERO,
            vanna: Decimal::ZERO,
            vomma: Decimal::ZERO,
            veta: Decimal::ZERO,
            charm: Decimal::ZERO,
            color: Decimal::ZERO,
        };
        for option in &legs.0 {
            reference.delta += ok(delta(option), "delta");
            reference.gamma += ok(gamma(option), "gamma");
            reference.theta += ok(theta(option), "theta");
            reference.vega += ok(vega(option), "vega");
            reference.rho += ok(rho(option), "rho");
            reference.rho_d += ok(rho_d(option), "rho_d");
            reference.alpha += ok(alpha(option), "alpha");
            reference.vanna += ok(vanna(option), "vanna");
            reference.vomma += ok(vomma(option), "vomma");
            reference.veta += ok(veta(option), "veta");
            reference.charm += ok(charm(option), "charm");
            reference.color += ok(color(option), "color");
        }

        let Ok(aggregate) = legs.greeks() else {
            panic!("the aggregate should compute for ordinary legs");
        };
        assert_eq!(
            aggregate, reference,
            "greeks() must match the unchecked sum"
        );

        for (name, aggregated, expected) in [
            ("delta", legs.delta(), reference.delta),
            ("gamma", legs.gamma(), reference.gamma),
            ("theta", legs.theta(), reference.theta),
            ("vega", legs.vega(), reference.vega),
            ("rho", legs.rho(), reference.rho),
            ("rho_d", legs.rho_d(), reference.rho_d),
            ("alpha", legs.alpha(), reference.alpha),
            ("vanna", legs.vanna(), reference.vanna),
            ("vomma", legs.vomma(), reference.vomma),
            ("veta", legs.veta(), reference.veta),
            ("charm", legs.charm(), reference.charm),
            ("color", legs.color(), reference.color),
        ] {
            assert_eq!(
                ok(aggregated, name),
                expected,
                "{name} must match the unchecked sum"
            );
        }
    }

    /// An empty leg set aggregates to zero rather than erroring, on both paths.
    #[test]
    fn test_checked_aggregation_of_no_legs_is_zero() {
        let legs = Legs(Vec::new());
        assert_eq!(ok(legs.alpha(), "alpha"), Decimal::ZERO);
        let Ok(aggregate) = legs.greeks() else {
            panic!("an empty leg set should aggregate to zero");
        };
        assert_eq!(aggregate.delta, Decimal::ZERO);
        assert_eq!(aggregate.alpha, Decimal::ZERO);
    }
}
