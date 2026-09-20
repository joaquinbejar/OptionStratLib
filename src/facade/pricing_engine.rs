//! The pricing dispatcher over the closed forms and the concrete simulator.
//!
//! `PricingEngine` stores a [`crate::simulation::simulator::Simulator`], so
//! pricing would have to name simulation to own it. The component-level form
//! is [`crate::pricing::GenericPricingEngine`] with the
//! [`crate::pricing::MonteCarloPricer`] contract (#508); this enum is the
//! 0.21 compatibility surface over it, and `price_option` delegates to
//! [`crate::pricing::price_option_with`] through a borrowing view, so the
//! two dispatchers cannot drift.

use crate::Options;
use crate::error::PricingResult;
use crate::pricing::{GenericPricingEngine, price_option_with};
use crate::simulation::simulator::Simulator;
use positive::Positive;

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
/// Garman–Kohlhagen) you may receive [`crate::error::PricingError::ExpirationDate`],
/// [`crate::error::PricingError::Greeks`] (for example zero-volatility or non-finite
/// intermediate values bubbled up from `d1`/`d2`), and (Black-76 and
/// Garman–Kohlhagen) [`crate::error::PricingError::UnsupportedOptionType`] for
/// non-European inputs. From the binomial lattice you may receive
/// [`crate::error::PricingError::BinomialNodeMissing`] or [`crate::error::PricingError::SqrtFailure`].
/// The Monte Carlo engine surfaces failures as
/// [`crate::error::PricingError::SimulationError`], and exotic engines surface their
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
    use crate::error::PricingError;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use crate::pricing::black_scholes_model::black_scholes;
    use crate::pricing::{ClosedFormEngine, MonteCarloPricer, NoMonteCarlo};
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
