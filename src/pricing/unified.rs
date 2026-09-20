use crate::Options;
use crate::error::{PricingError, PricingResult};
use crate::pricing::black_76::black_76;
use crate::pricing::black_scholes_model::black_scholes;
use crate::pricing::garman_kohlhagen::garman_kohlhagen;
use positive::Positive;

/// Contract a Monte Carlo pricer fulfils for the generic dispatcher.
///
/// This is the only thing pricing needs from a simulator: given an
/// option, produce a price. The simulation layer implements it for
/// `Simulator<Positive, Positive>` by delegating to its Monte Carlo
/// estimate; [`NoMonteCarlo`] implements it for engines that have no
/// simulator at all. `Send + Sync` so an engine can be shared across Rayon
/// workers when pricing a chain.
pub trait MonteCarloPricer: Send + Sync {
    /// Prices `option` by Monte Carlo simulation.
    ///
    /// # Errors
    ///
    /// Returns [`PricingError::SimulationError`] when the simulation
    /// cannot price the option (no paths, an invalid payoff, or no
    /// simulator configured). The shipped implementors (`Simulator` and
    /// `NoMonteCarlo`) report every failure through that variant; a
    /// third-party pricer may surface any other `PricingError` its payoff
    /// evaluation raises.
    fn price_monte_carlo(&self, option: &Options) -> PricingResult<Positive>;
}

impl<M> MonteCarloPricer for &M
where
    M: MonteCarloPricer + ?Sized,
{
    #[inline]
    fn price_monte_carlo(&self, option: &Options) -> PricingResult<Positive> {
        (**self).price_monte_carlo(option)
    }
}

/// Placeholder Monte Carlo pricer for engines without a simulator.
///
/// The default `M` of [`GenericPricingEngine`]: a closed-form-only engine
/// carries this zero-sized type, and its `MonteCarlo` arm reports
/// [`PricingError::SimulationError`] instead of pricing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NoMonteCarlo;

impl MonteCarloPricer for NoMonteCarlo {
    #[cold]
    #[inline(never)]
    fn price_monte_carlo(&self, _option: &Options) -> PricingResult<Positive> {
        Err(PricingError::simulation_error(
            "no Monte Carlo pricer is configured for this pricing engine",
        ))
    }
}

/// Pricing engine selector generic over its Monte Carlo pricer.
///
/// The same four arms as [`GenericPricingEngine`]; the `MonteCarlo` arm holds any
/// [`MonteCarloPricer`] instead of a concrete simulator, so the dispatcher
/// itself has no knowledge of the simulation layer. Dispatch is static:
/// each `M` is monomorphised, there is no boxed trait object on the pricing
/// path. `M` defaults to [`NoMonteCarlo`]; a default type parameter does
/// not drive inference in expression position, so a closed-form-only
/// caller writes `GenericPricingEngine::<NoMonteCarlo>::ClosedFormBS` or
/// uses the [`ClosedFormEngine`] alias.
///
/// After the 0.22.0 bump this is the only engine (ADR-0001 D3); the facade
/// aliases `GenericPricingEngine` to
/// `GenericPricingEngine<Simulator<Positive, Positive>>`.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum GenericPricingEngine<M: MonteCarloPricer = NoMonteCarlo> {
    /// Black-Scholes closed-form pricing for European options.
    ClosedFormBS,

    /// Black-76 closed-form pricing for options on futures and forwards.
    ClosedFormBlack76,

    /// Monte Carlo pricing through `M`.
    MonteCarlo {
        /// The configured Monte Carlo pricer (a `Simulator` in the facade).
        simulator: M,
    },

    /// Garman-Kohlhagen closed-form pricing for European FX options.
    ClosedFormGK,
}

/// A [`GenericPricingEngine`] with no Monte Carlo pricer: the three closed
/// forms only.
pub type ClosedFormEngine = GenericPricingEngine<NoMonteCarlo>;

/// Black-Scholes arm shared by both dispatchers.
#[inline]
fn price_closed_form_bs(option: &Options) -> PricingResult<Positive> {
    let price_decimal = black_scholes(option)?;
    Ok(Positive::new_decimal(price_decimal.abs())?)
}

/// Black-76 arm shared by both dispatchers.
#[inline]
fn price_closed_form_black76(option: &Options) -> PricingResult<Positive> {
    let price_decimal = black_76(option)?;
    Ok(Positive::new_decimal(price_decimal.abs())?)
}

/// Garman-Kohlhagen arm shared by both dispatchers.
#[inline]
fn price_closed_form_gk(option: &Options) -> PricingResult<Positive> {
    let price_decimal = garman_kohlhagen(option)?;
    Ok(Positive::new_decimal(price_decimal.abs())?)
}

/// Prices an option with a [`GenericPricingEngine`].
///
/// The one dispatcher: the closed-form arms are
/// the same functions, and the Monte Carlo arm calls
/// [`MonteCarloPricer::price_monte_carlo`] on `M`.
///
/// # Errors
///
/// With [`NoMonteCarlo`] the `MonteCarlo` arm
/// returns [`PricingError::SimulationError`].
#[must_use = "the price is the only product of this call"]
pub fn price_option_with<M>(
    option: &Options,
    engine: &GenericPricingEngine<M>,
) -> PricingResult<Positive>
where
    M: MonteCarloPricer,
{
    match engine {
        GenericPricingEngine::ClosedFormBS => price_closed_form_bs(option),
        GenericPricingEngine::ClosedFormBlack76 => price_closed_form_black76(option),
        GenericPricingEngine::MonteCarlo { simulator } => simulator.price_monte_carlo(option),
        GenericPricingEngine::ClosedFormGK => price_closed_form_gk(option),
    }
}

/// Prices an instrument with a [`GenericPricingEngine`].
///
/// The engine is generic over its Monte Carlo pricer, so pricing never names
/// a concrete simulation engine: pass [`NoMonteCarlo`] for the closed forms,
/// or any [`MonteCarloPricer`] (`simulation::Simulator` implements it).
pub trait Priceable {
    /// Prices `self` with `engine`.
    ///
    /// # Errors
    ///
    /// Propagates the `PricingError` of the selected arm; see
    /// [`price_option_with`] for the variant breakdown.
    fn price<M>(&self, engine: &GenericPricingEngine<M>) -> PricingResult<Positive>
    where
        M: MonteCarloPricer;
}

impl Priceable for Options {
    #[inline]
    fn price<M>(&self, engine: &GenericPricingEngine<M>) -> PricingResult<Positive>
    where
        M: MonteCarloPricer,
    {
        price_option_with(self, engine)
    }
}
