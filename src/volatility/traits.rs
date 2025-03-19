use crate::Positive;
use crate::chains::chain::OptionChain;
use crate::curves::Curve;
use std::error::Error;

/// A trait defining a volatility smile representation.
///
/// The `VolatilitySmile` trait is designed to encapsulate the concept of a
/// volatility smile, a key phenomenon in derivatives pricing and financial
/// modeling. A volatility smile occurs when implied volatility varies as a
/// function of strike price, often depicted as a curved graph resembling a
/// smile. This trait establishes the foundation for representing and
/// retrieving these smiles in the form of a mathematical curve.
///
/// # Overview
/// Implementors of this trait are required to provide the `smile` method, which
/// computes and returns a `Curve` object representing the volatility smile.
/// The `Curve` struct is a mathematical representation of the smile, where the
/// x-axis typically corresponds to strike prices (or some other independent variable),
/// and the y-axis corresponds to implied volatility.
///
/// # Usage
/// This trait serves as the basis for constructing and analyzing volatility smiles
/// in applications such as:
/// - Financial derivatives modeling
/// - Options pricing engines
/// - Quantitative analysis of market data
///
/// # Required Methods
/// - **`smile(&self) -> Curve`**
///   - Computes and returns the volatility smile as a `Curve`.
///   - The returned `Curve` can be used for graphical representation, numerical analysis,
///     or further mathematical operations, such as interpolation or transformations.
///
/// # Integration with Other Modules
/// The `VolatilitySmile` trait makes use of the `Curve` struct, defined in the
/// `crate::curves` module. The `Curve` provides the mathematical framework
/// necessary for representing and manipulating the smile data. High-quality
/// precision (via the use of `Decimal` and ordered points) ensures that the output
/// from the `smile` method is reliable and suitable for scientific or financial
/// applications.
///
/// # See Also
/// - [`crate::curves::Curve`]: The fundamental mathematical representation of the volatility smile.
/// - [`crate::curves::Point2D`]: The structure representing individual points in the `Curve`.
///
/// # Examples
/// To define a custom volatility model, users can implement this trait and provide
/// their specific logic for generating a `Curve` corresponding to the smile.
///
/// ```rust
/// use std::collections::BTreeSet;
/// use rust_decimal::Decimal;
/// use optionstratlib::curves::Curve;
/// use optionstratlib::error::greeks::CalculationErrorKind::DecimalError;
/// use optionstratlib::volatility::VolatilitySmile;
///
/// struct MySmile;
///
/// impl VolatilitySmile for MySmile {
///     fn smile(&self) -> Curve {
///         // Custom logic to build and return a Curve representing the smile
///         Curve { points: BTreeSet::new(), x_range: (Decimal::ZERO, Decimal::ZERO) }
///     }
/// }
/// ```
///
/// This enables integration of user-defined volatility models with the broader
/// ecosystem of mathematical and financial tools that utilize the `Curve` data type.
pub trait VolatilitySmile {
    /// Computes and returns a curve representing the volatility smile.
    ///
    /// # Returns
    /// - A [`Curve`] object that models the volatility smile. The x-axis typically
    ///   represents strike prices (or another independent variable), while the y-axis
    ///   represents implied volatility.
    ///   
    /// # Note
    /// - The `Curve` returned should ideally conform to the constraints and
    ///   ordering requirements specified in the `Curve` documentation.
    fn smile(&self) -> Curve;
}

/// Trait for providing at-the-money implied volatility.
///
/// This trait defines a method to retrieve the at-the-money (ATM) implied volatility.
/// Implementations should return a `Positive` value representing the ATM IV, or an error
/// if the value cannot be determined.
pub trait AtmIvProvider {
    /// Get the at-the-money implied volatility
    ///
    /// This method attempts to return the at-the-money implied volatility as an `Option<Positive>`.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Positive))` - If the ATM implied volatility is successfully retrieved.
    /// * `Ok(None)` - If the ATM implied volatility is not available or not applicable.
    /// * `Err(Box<dyn Error>)` - If an error occurs during the retrieval process.
    fn atm_iv(&self) -> Result<&Option<Positive>, Box<dyn Error>>;
}

impl AtmIvProvider for Positive {
    fn atm_iv(&self) -> Result<&Option<Positive>, Box<dyn Error>> {
        Ok(&None)
    }
}

impl AtmIvProvider for OptionChain {
    fn atm_iv(&self) -> Result<&Option<Positive>, Box<dyn Error>> {
        match self.atm_implied_volatility() {
            Ok(iv) => Ok(iv),
            Err(e) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("ATM IV not available: {}", e),
            ))),
        }
    }
}
