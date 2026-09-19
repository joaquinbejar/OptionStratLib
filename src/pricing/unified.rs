use crate::Options;
use crate::error::{PricingError, PricingResult};
use crate::pricing::black_76::black_76;
use crate::pricing::black_scholes_model::black_scholes;
use crate::pricing::garman_kohlhagen::garman_kohlhagen;
use crate::simulation::simulator::Simulator; // deferred edge: PricingEngine::MonteCarlo field, 0.22.0 batch (#508, ADR-0001 D3)
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
/// The same four arms as [`PricingEngine`]; the `MonteCarlo` arm holds any
/// [`MonteCarloPricer`] instead of a concrete simulator, so the dispatcher
/// itself has no knowledge of the simulation layer. Dispatch is static:
/// each `M` is monomorphised, there is no boxed trait object on the pricing
/// path. `M` defaults to [`NoMonteCarlo`]; a default type parameter does
/// not drive inference in expression position, so a closed-form-only
/// caller writes `GenericPricingEngine::<NoMonteCarlo>::ClosedFormBS` or
/// uses the [`ClosedFormEngine`] alias.
///
/// After the 0.22.0 bump this is the only engine (ADR-0001 D3); the facade
/// aliases `PricingEngine` to
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
/// The generic counterpart of [`price_option`]: the closed-form arms are
/// the same functions, and the Monte Carlo arm calls
/// [`MonteCarloPricer::price_monte_carlo`] on `M`.
///
/// # Errors
///
/// Same as [`price_option`]; with [`NoMonteCarlo`] the `MonteCarlo` arm
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

/// Pricing engine selector for option pricing.
///
/// The concrete engine: its `MonteCarlo` arm holds a `Simulator`. It is
/// the 0.21 shape kept for compatibility; [`GenericPricingEngine`] is the
/// same selector generic over its Monte Carlo pricer, and after the 0.22.0
/// bump this name becomes an alias of
/// `GenericPricingEngine<Simulator<Positive, Positive>>` (ADR-0001 D3).
///
/// This enum allows selection between different pricing methods:
/// - `ClosedFormBS`: Uses the Black-Scholes closed-form formula
/// - `ClosedFormBlack76`: Uses the Black-76 closed-form formula
/// - `MonteCarlo`: Uses Monte Carlo simulation with a configured simulator
/// - `ClosedFormGK`: Uses the Garman-Kohlhagen closed-form formula for FX options
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum PricingEngine {
    /// Black-Scholes closed-form pricing for European options.
    ///
    /// This is the fastest pricing method with O(1) complexity.
    /// Best suited for European options with constant volatility assumptions.
    ClosedFormBS,

    /// Black-76 closed-form pricing for options on futures and forwards.
    ///
    /// Fast O(1) pricing for European options on futures, forwards, swaptions, and caps/floors.
    /// The underlying price is the forward price F, not the spot price.
    ClosedFormBlack76,

    /// Monte Carlo simulation-based pricing.
    ///
    /// Uses a configured `Simulator` to generate random price paths and
    /// estimate option prices. Supports various stochastic models through
    /// different `WalkType` configurations.
    MonteCarlo {
        /// The simulator configured with the desired stochastic model
        simulator: Simulator<Positive, Positive>,
    },

    /// Garman–Kohlhagen closed-form pricing for European FX options.
    ///
    /// Fast O(1) pricing for European options on FX spot rates. Uses two
    /// interest rates (domestic via `risk_free_rate`, foreign via
    /// `dividend_yield`). Structurally identical to Black–Scholes–Merton
    /// with `q = r_f`.
    ClosedFormGK,
}

/// Prices an option using the specified pricing engine.
///
/// This is the unified entry point for option pricing that dispatches to
/// the appropriate pricing method based on the engine configuration.
///
/// # Arguments
///
/// * `option` - The option to price
/// * `engine` - The pricing engine to use
///
/// # Returns
///
/// Returns the option price as a `Positive` value, or a `PricingError` if pricing fails.
///
/// # Examples
///
/// ```rust
/// use optionstratlib::pricing::{PricingEngine, price_option};
/// use positive::{Positive, pos_or_panic};
/// use optionstratlib::{ExpirationDate, Options};
/// use optionstratlib::model::types::{OptionStyle, OptionType, Side};
/// use rust_decimal_macros::dec;
///
/// let option = Options {
///     option_type: OptionType::European,
///     side: Side::Long,
///     underlying_symbol: "AAPL".to_string(),
///     strike_price: Positive::HUNDRED,
///     expiration_date: ExpirationDate::Days(pos_or_panic!(30.0)),
///     implied_volatility: pos_or_panic!(0.2),
///     quantity: Positive::ONE,
///     underlying_price: pos_or_panic!(105.0),
///     risk_free_rate: dec!(0.05),
///     option_style: OptionStyle::Call,
///     dividend_yield: pos_or_panic!(0.01),
///     exotic_params: None,
/// };
/// let engine = PricingEngine::ClosedFormBS;
/// let price = price_option(&option, &engine)?;
/// Ok::<(), optionstratlib::error::PricingError>(())
/// ```
///
/// # Errors
///
/// Propagates the original `PricingError` returned by the selected engine
/// without wrapping, so callers can pattern-match on the structured
/// variants. From the closed-form engines (Black–Scholes, Black-76,
/// Garman–Kohlhagen) you may receive [`PricingError::ExpirationDate`],
/// [`PricingError::Greeks`] (for example zero-volatility or non-finite
/// intermediate values bubbled up from `d1`/`d2`), and (Black-76 and
/// Garman–Kohlhagen) [`PricingError::UnsupportedOptionType`] for
/// non-European inputs. From the binomial lattice you may receive
/// [`PricingError::BinomialNodeMissing`] or [`PricingError::SqrtFailure`].
/// The Monte Carlo engine surfaces failures as
/// [`PricingError::SimulationError`], and exotic engines surface their
/// own variants (barrier, binary, compound, chooser, cliquet, lookback,
/// telegraph).
pub fn price_option(option: &Options, engine: &PricingEngine) -> PricingResult<Positive> {
    price_option_with(option, &engine.as_generic())
}

impl PricingEngine {
    /// Views this engine as a [`GenericPricingEngine`] borrowing the
    /// simulator, so [`price_option`] is [`price_option_with`] and the two
    /// dispatchers cannot drift.
    #[must_use]
    #[inline]
    fn as_generic(&self) -> GenericPricingEngine<&Simulator<Positive, Positive>> {
        match self {
            PricingEngine::ClosedFormBS => GenericPricingEngine::ClosedFormBS,
            PricingEngine::ClosedFormBlack76 => GenericPricingEngine::ClosedFormBlack76,
            PricingEngine::MonteCarlo { simulator } => {
                GenericPricingEngine::MonteCarlo { simulator }
            }
            PricingEngine::ClosedFormGK => GenericPricingEngine::ClosedFormGK,
        }
    }
}

impl From<PricingEngine> for GenericPricingEngine<Simulator<Positive, Positive>> {
    /// Moves the concrete engine into its generic shape; the facade alias
    /// of ADR-0001 D3 makes the two the same type after the 0.22.0 bump.
    fn from(engine: PricingEngine) -> Self {
        match engine {
            PricingEngine::ClosedFormBS => GenericPricingEngine::ClosedFormBS,
            PricingEngine::ClosedFormBlack76 => GenericPricingEngine::ClosedFormBlack76,
            PricingEngine::MonteCarlo { simulator } => {
                GenericPricingEngine::MonteCarlo { simulator }
            }
            PricingEngine::ClosedFormGK => GenericPricingEngine::ClosedFormGK,
        }
    }
}

/// Trait for types that can be priced using a pricing engine.
///
/// This trait provides a unified interface for pricing financial instruments.
pub trait Priceable {
    /// Prices the instrument using the specified pricing engine.
    ///
    /// # Arguments
    ///
    /// * `engine` - The pricing engine to use
    ///
    /// # Returns
    ///
    /// Returns the price as a `Positive` value, or a `PricingError` if pricing fails.
    ///
    /// # Errors
    ///
    /// Propagates any `PricingError` returned by the selected
    /// engine; see [`price_option`] for the full variant breakdown.
    fn price(&self, engine: &PricingEngine) -> PricingResult<Positive>;
}

/// Implementation of `Priceable` for `Options`.
///
/// This allows options to be priced using the unified pricing API.
impl Priceable for Options {
    fn price(&self, engine: &PricingEngine) -> PricingResult<Positive> {
        price_option(self, engine)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExpirationDate;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    fn option(style: OptionStyle) -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "TEST".to_string(),
            strike_price: Positive::HUNDRED,
            expiration_date: ExpirationDate::Days(pos_or_panic!(30.0)),
            implied_volatility: pos_or_panic!(0.2),
            quantity: Positive::ONE,
            underlying_price: pos_or_panic!(105.0),
            risk_free_rate: dec!(0.05),
            option_style: style,
            dividend_yield: pos_or_panic!(0.01),
            exotic_params: None,
        }
    }

    #[test]
    fn test_generic_closed_form_arms_price_exactly_like_pricing_engine() {
        for style in [OptionStyle::Call, OptionStyle::Put] {
            let option = option(style);
            let pairs = [
                (PricingEngine::ClosedFormBS, ClosedFormEngine::ClosedFormBS),
                (
                    PricingEngine::ClosedFormBlack76,
                    ClosedFormEngine::ClosedFormBlack76,
                ),
                (PricingEngine::ClosedFormGK, ClosedFormEngine::ClosedFormGK),
            ];
            for (concrete, generic) in pairs {
                let expected = price_option(&option, &concrete).unwrap();
                assert_eq!(price_option_with(&option, &generic).unwrap(), expected);
                assert_eq!(
                    price_option_with(&option, &GenericPricingEngine::from(concrete)).unwrap(),
                    expected
                );
            }
        }
    }

    #[test]
    fn test_closed_form_engine_is_black_scholes_for_bs_arm() {
        let option = option(OptionStyle::Call);
        let expected = Positive::new_decimal(black_scholes(&option).unwrap().abs()).unwrap();
        assert_eq!(
            price_option_with(&option, &GenericPricingEngine::<NoMonteCarlo>::ClosedFormBS)
                .unwrap(),
            expected
        );
    }

    #[test]
    fn test_no_monte_carlo_reports_simulation_error() {
        let option = option(OptionStyle::Call);
        let engine = ClosedFormEngine::MonteCarlo {
            simulator: NoMonteCarlo,
        };
        assert!(matches!(
            price_option_with(&option, &engine),
            Err(PricingError::SimulationError { .. })
        ));
        assert!(matches!(
            NoMonteCarlo.price_monte_carlo(&option),
            Err(PricingError::SimulationError { .. })
        ));
        let defaulted: NoMonteCarlo = Default::default();
        assert_eq!(NoMonteCarlo, defaulted);
    }

    #[test]
    fn test_borrowed_pricer_delegates() {
        let option = option(OptionStyle::Put);
        let pricer = NoMonteCarlo;
        let engine = GenericPricingEngine::MonteCarlo { simulator: &pricer };
        assert!(price_option_with(&option, &engine).is_err());
        let engine = GenericPricingEngine::<&NoMonteCarlo>::ClosedFormBS;
        assert_eq!(
            price_option_with(&option, &engine).unwrap(),
            price_option(&option, &PricingEngine::ClosedFormBS).unwrap()
        );
    }

    #[test]
    fn test_generic_engine_is_send_sync_and_clone() {
        fn assert_send_sync<T: Send + Sync + Clone>(_: &T) {}
        assert_send_sync(&ClosedFormEngine::ClosedFormBS);
    }
}
