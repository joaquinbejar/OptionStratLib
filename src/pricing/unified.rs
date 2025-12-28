use crate::Options;
use crate::error::{PricingError, PricingResult};
use crate::pricing::black_scholes_model::black_scholes;
use crate::simulation::simulator::Simulator;
use positive::Positive;

/// Pricing engine selector for option pricing.
///
/// This enum allows selection between different pricing methods:
/// - `ClosedFormBS`: Uses the Black-Scholes closed-form formula
/// - `MonteCarlo`: Uses Monte Carlo simulation with a configured simulator
pub enum PricingEngine {
    /// Black-Scholes closed-form pricing for European options.
    ///
    /// This is the fastest pricing method with O(1) complexity.
    /// Best suited for European options with constant volatility assumptions.
    ClosedFormBS,

    /// Monte Carlo simulation-based pricing.
    ///
    /// Uses a configured `Simulator` to generate random price paths and
    /// estimate option prices. Supports various stochastic models through
    /// different `WalkType` configurations.
    MonteCarlo {
        /// The simulator configured with the desired stochastic model
        simulator: Simulator<Positive, Positive>,
    },
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
pub fn price_option(option: &Options, engine: &PricingEngine) -> PricingResult<Positive> {
    match engine {
        PricingEngine::ClosedFormBS => {
            let price_decimal = black_scholes(option)
                .map_err(|e| PricingError::method_error("Black-Scholes", &e.to_string()))?;

            // Convert Decimal to Positive using From trait
            Ok(Positive::new_decimal(price_decimal.abs())?)
        }
        PricingEngine::MonteCarlo { simulator } => simulator
            .get_mc_option_price(option)
            .map_err(|e| PricingError::simulation_error(&e.to_string())),
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
