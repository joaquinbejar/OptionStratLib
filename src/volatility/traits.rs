use crate::Positive;
use crate::chains::chain::OptionChain;
use crate::curves::Curve;
use crate::error::VolatilityError;

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
    fn atm_iv(&self) -> Result<&Positive, VolatilityError>;
}

impl AtmIvProvider for Positive {
    fn atm_iv(&self) -> Result<&Positive, VolatilityError> {
        Ok(self)
    }
}

impl AtmIvProvider for OptionChain {
    fn atm_iv(&self) -> Result<&Positive, VolatilityError> {
        match self.get_atm_implied_volatility() {
            Ok(iv) => Ok(iv),
            Err(e) => Err(format!("ATM IV not available: {e}").into()),
        }
    }
}

#[cfg(test)]
mod tests_volatility_traits {
    use super::*;
    use crate::curves::{Curve, Point2D};
    use crate::metrics::VolatilitySkew;
    use crate::pos;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

    struct TestSmile;

    impl VolatilitySmile for TestSmile {
        fn smile(&self) -> Curve {
            create_sample_curve()
        }
    }

    struct TestIvProvider {
        iv: Positive,
    }

    impl AtmIvProvider for TestIvProvider {
        fn atm_iv(&self) -> Result<&Positive, VolatilityError> {
            Ok(&self.iv)
        }
    }

    fn create_sample_curve() -> Curve {
        let mut points = BTreeSet::new();
        points.insert(Point2D::new(dec!(90.0), dec!(0.25)));
        points.insert(Point2D::new(dec!(95.0), dec!(0.22)));
        points.insert(Point2D::new(dec!(100.0), dec!(0.20)));
        points.insert(Point2D::new(dec!(105.0), dec!(0.22)));
        points.insert(Point2D::new(dec!(110.0), dec!(0.25)));

        Curve {
            points,
            x_range: (dec!(90.0), dec!(110.0)),
        }
    }

    #[test]
    fn test_volatility_smile_implementation() {
        let smile = TestSmile;
        let curve = smile.smile();

        // Verify the curve has expected properties
        assert_eq!(curve.points.len(), 5);
        assert_eq!(curve.x_range, (dec!(90.0), dec!(110.0)));

        // Check specific points
        let points: Vec<&Point2D> = curve.points.iter().collect();
        assert_eq!(points[0].x, dec!(90.0));
        assert_eq!(points[0].y, dec!(0.25));
        assert_eq!(points[2].x, dec!(100.0));
        assert_eq!(points[2].y, dec!(0.20));
        assert_eq!(points[4].x, dec!(110.0));
        assert_eq!(points[4].y, dec!(0.25));
    }

    #[test]
    fn test_volatility_smile() {
        let smile = TestSmile;
        let curve = smile.smile();

        assert_eq!(curve.points.len(), 5);
        assert_eq!(curve.x_range, (dec!(90.0), dec!(110.0)));

        let points: Vec<&Point2D> = curve.points.iter().collect();
        assert_eq!(points[0].x, dec!(90.0));
        assert_eq!(points[0].y, dec!(0.25));
        assert_eq!(points[2].x, dec!(100.0));
        assert_eq!(points[2].y, dec!(0.20));
        assert_eq!(points[4].x, dec!(110.0));
        assert_eq!(points[4].y, dec!(0.25));
    }

    #[test]
    fn test_volatility_smile_with_empty_curve() {
        struct EmptySmile;

        impl VolatilitySmile for EmptySmile {
            fn smile(&self) -> Curve {
                Curve {
                    points: BTreeSet::new(),
                    x_range: (Decimal::ZERO, Decimal::ZERO),
                }
            }
        }

        let smile = EmptySmile;
        let curve = smile.smile();

        assert!(curve.points.is_empty());
        assert_eq!(curve.x_range, (Decimal::ZERO, Decimal::ZERO));
    }

    #[test]
    fn test_atm_iv_provider_for_positive() {
        let value = pos!(0.2);

        // Test AtmIvProvider implementation for Positive
        let result = value.atm_iv();

        assert!(result.is_ok());
    }

    #[test]
    fn test_atm_iv_provider_some() {
        let provider = TestIvProvider { iv: pos!(0.25) };

        let result = provider.atm_iv();
        assert!(result.is_ok());
    }

    #[test]
    fn test_atm_iv_provider_error() {
        struct ErrorIvProvider;

        impl AtmIvProvider for ErrorIvProvider {
            fn atm_iv(&self) -> Result<&Positive, VolatilityError> {
                Err("ATM IV not available: test error".into())
            }
        }

        let provider = ErrorIvProvider;
        let result = provider.atm_iv();

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("ATM IV not available"));
    }

    #[test]
    fn test_combined_traits_usage() {
        // Create an implementation that provides both traits
        struct CombinedProvider {
            iv_value: Positive,
        }

        impl VolatilitySmile for CombinedProvider {
            fn smile(&self) -> Curve {
                create_sample_curve()
            }
        }

        impl AtmIvProvider for CombinedProvider {
            fn atm_iv(&self) -> Result<&Positive, VolatilityError> {
                Ok(&self.iv_value)
            }
        }

        // Test with Some value
        let provider_with_iv = CombinedProvider {
            iv_value: pos!(0.2),
        };

        // As VolatilitySmile
        let curve = provider_with_iv.smile();
        assert_eq!(curve.points.len(), 5);

        // As AtmIvProvider
        let iv_result = provider_with_iv.atm_iv();
        assert!(iv_result.is_ok());
    }

    #[test]
    fn test_atm_iv_provider_for_option_chain_success() {
        use crate::chains::chain::OptionChain;

        let mut chain = OptionChain::new(
            "TEST",
            pos!(100.0),
            "2025-12-31".to_string(),
            Some(dec!(0.05)),
            None,
        );

        // Add options with implied volatility
        chain.add_option(
            pos!(95.0),
            Some(pos!(6.0)),
            Some(pos!(6.5)),
            Some(pos!(1.0)),
            Some(pos!(1.5)),
            pos!(0.25),
            Some(dec!(0.7)),
            Some(dec!(-0.3)),
            Some(dec!(0.02)),
            None,
            None,
            None,
        );

        chain.add_option(
            pos!(100.0),
            Some(pos!(3.0)),
            Some(pos!(3.5)),
            Some(pos!(3.0)),
            Some(pos!(3.5)),
            pos!(0.20),
            Some(dec!(0.5)),
            Some(dec!(-0.5)),
            Some(dec!(0.025)),
            None,
            None,
            None,
        );

        chain.add_option(
            pos!(105.0),
            Some(pos!(1.0)),
            Some(pos!(1.5)),
            Some(pos!(6.0)),
            Some(pos!(6.5)),
            pos!(0.22),
            Some(dec!(0.3)),
            Some(dec!(-0.7)),
            Some(dec!(0.02)),
            None,
            None,
            None,
        );

        let result = chain.atm_iv();
        assert!(result.is_ok());
        let iv = result.unwrap();
        assert!(*iv > Positive::ZERO);
    }

    #[test]
    fn test_atm_iv_provider_for_option_chain_empty() {
        use crate::chains::chain::OptionChain;

        let chain = OptionChain::new("TEST", pos!(100.0), "2025-12-31".to_string(), None, None);

        let result = chain.atm_iv();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("ATM IV not available"));
    }

    #[test]
    fn test_volatility_smile_symmetry() {
        struct SymmetricSmile;

        impl VolatilitySmile for SymmetricSmile {
            fn smile(&self) -> Curve {
                let mut points = BTreeSet::new();
                // Symmetric smile around ATM
                points.insert(Point2D::new(dec!(80.0), dec!(0.30)));
                points.insert(Point2D::new(dec!(90.0), dec!(0.22)));
                points.insert(Point2D::new(dec!(100.0), dec!(0.18)));
                points.insert(Point2D::new(dec!(110.0), dec!(0.22)));
                points.insert(Point2D::new(dec!(120.0), dec!(0.30)));

                Curve {
                    points,
                    x_range: (dec!(80.0), dec!(120.0)),
                }
            }
        }

        let smile = SymmetricSmile;
        let curve = smile.smile();

        let points: Vec<&Point2D> = curve.points.iter().collect();

        // Verify symmetry: IV at 80 == IV at 120, IV at 90 == IV at 110
        assert_eq!(points[0].y, points[4].y); // 80 and 120
        assert_eq!(points[1].y, points[3].y); // 90 and 110
        // ATM has lowest IV
        assert!(points[2].y < points[1].y);
    }

    #[test]
    fn test_atm_iv_provider_positive_value() {
        let value = pos!(0.35);
        let result = value.atm_iv();

        assert!(result.is_ok());
        let iv = result.unwrap();
        assert_eq!(*iv, pos!(0.35));
    }

    #[test]
    fn test_atm_iv_provider_small_value() {
        let value = pos!(0.01);
        let result = value.atm_iv();

        assert!(result.is_ok());
        let iv = result.unwrap();
        assert_eq!(*iv, pos!(0.01));
    }

    #[test]
    fn test_volatility_smile_single_point() {
        struct SinglePointSmile;

        impl VolatilitySmile for SinglePointSmile {
            fn smile(&self) -> Curve {
                let mut points = BTreeSet::new();
                points.insert(Point2D::new(dec!(100.0), dec!(0.20)));

                Curve {
                    points,
                    x_range: (dec!(100.0), dec!(100.0)),
                }
            }
        }

        let smile = SinglePointSmile;
        let curve = smile.smile();

        assert_eq!(curve.points.len(), 1);
        let point = curve.points.iter().next().unwrap();
        assert_eq!(point.x, dec!(100.0));
        assert_eq!(point.y, dec!(0.20));
    }

    #[test]
    fn test_combined_smile_and_skew_traits() {
        struct CombinedVolatility;

        impl VolatilitySmile for CombinedVolatility {
            fn smile(&self) -> Curve {
                create_sample_curve()
            }
        }

        impl VolatilitySkew for CombinedVolatility {
            fn volatility_skew(&self) -> Curve {
                let mut points = BTreeSet::new();
                points.insert(Point2D::new(dec!(-10.0), dec!(0.25)));
                points.insert(Point2D::new(dec!(0.0), dec!(0.20)));
                points.insert(Point2D::new(dec!(10.0), dec!(0.25)));

                Curve {
                    points,
                    x_range: (dec!(-10.0), dec!(10.0)),
                }
            }
        }

        let vol = CombinedVolatility;

        // Test smile
        let smile_curve = vol.smile();
        assert_eq!(smile_curve.points.len(), 5);

        // Test skew
        let skew_curve = vol.volatility_skew();
        assert_eq!(skew_curve.points.len(), 3);
    }
}
