/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 16/2/25
******************************************************************************/
use crate::error::StrategyError;
use crate::strategies::Strategies;
use optionstratlib_core::model::Position;
use optionstratlib_core::model::leg::SpotPosition;
use optionstratlib_pricing::greeks::Greeks;

/// Defines a common interface for constructing financial option strategies from
/// collections of option positions.
///
/// This trait extends both the `Strategies` and `Greeks` traits, ensuring that
/// implementers can both operate as option strategies and calculate Greek values
/// for risk analysis. It provides a default implementation of the strategy
/// construction method that returns a "not implemented" error, which concrete
/// implementations should override.
///
/// # Type Requirements
///
/// Implementers must also implement:
/// - `Strategies`: Provides strategy-specific operations and calculations
/// - `Greeks`: Provides access to option sensitivity calculations (delta, gamma, etc.)
///
pub trait StrategyConstructor: Strategies + Greeks {
    /// Attempts to construct a strategy from a vector of option positions.
    ///
    /// This method analyzes the provided option positions and attempts to
    /// recognize and construct a specific options strategy. The default
    /// implementation returns a `NotImplemented` error, so concrete types
    /// must provide their own implementation.
    ///
    /// # Parameters
    ///
    /// * `_vec_options` - A slice of `Position` objects representing the
    ///   option positions to analyze
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - The successfully constructed strategy
    /// * `Err(StrategyError)` - If the positions don't match the expected
    ///   pattern for this strategy type
    ///
    /// # Errors
    ///
    /// Returns `StrategyError::OperationError` when the provided
    /// positions do not match the leg count or side/style pattern
    /// required by the implementing strategy type, and
    /// `StrategyError::InvalidStrategy` when they match it but the
    /// assembled strategy fails its own `Validable::validate` (#696).
    /// The default implementation returns `StrategyError::NotImplemented`.
    fn get_strategy(_vec_positions: &[Position]) -> Result<Self, StrategyError>
    where
        Self: Sized,
    {
        Err(StrategyError::NotImplemented)
    }

    /// Constructs the strategy from its option positions and, for a strategy
    /// that holds the underlying, its share leg (#831).
    ///
    /// The default implementation is [`StrategyConstructor::get_strategy`]
    /// for a strategy without a share leg; `CoveredCall`, `ProtectivePut` and
    /// `Collar` override it and require `spot_leg`.
    ///
    /// # Errors
    ///
    /// Returns `StrategyError::OperationError` when `spot_leg` is given to a
    /// strategy that has no share leg, and whatever
    /// [`StrategyConstructor::get_strategy`] returns otherwise.
    fn get_strategy_with_spot(
        spot_leg: Option<&SpotPosition>,
        vec_positions: &[Position],
    ) -> Result<Self, StrategyError>
    where
        Self: Sized,
    {
        match spot_leg {
            None => Self::get_strategy(vec_positions),
            Some(_) => Err(StrategyError::invalid_parameters(
                "get_strategy_with_spot",
                "this strategy has no share leg",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::StrategyError;
    use crate::strategies::Strategies;
    use crate::strategies::base::{BasicAble, BreakEvenable, Positionable, Validable};
    use optionstratlib_core::model::Options;
    use optionstratlib_pricing::error::GreeksError;

    /// Mock for a specific strategy
    #[derive(Debug, PartialEq)]
    struct TestStrategy;

    impl Validable for TestStrategy {}

    impl Positionable for TestStrategy {}

    impl BreakEvenable for TestStrategy {}

    impl BasicAble for TestStrategy {}

    impl Strategies for TestStrategy {}

    impl Greeks for TestStrategy {
        fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
            unimplemented!()
        }
    }

    /// Implementamos `StrategyConstructor` sin sobrescribir `get_strategy` (debe devolver `NotImplemented`)
    impl StrategyConstructor for TestStrategy {}

    #[test]
    fn test_get_strategy_not_implemented() {
        let options = vec![];
        let result = TestStrategy::get_strategy(&options);

        assert!(matches!(result, Err(StrategyError::NotImplemented)));
    }

    #[derive(Debug, PartialEq)]
    struct ValidStrategy;

    impl Validable for ValidStrategy {}

    impl Positionable for ValidStrategy {}

    impl BreakEvenable for ValidStrategy {}

    impl BasicAble for ValidStrategy {}

    impl Strategies for ValidStrategy {}

    impl Greeks for ValidStrategy {
        fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
            todo!()
        }
    }

    impl StrategyConstructor for ValidStrategy {
        fn get_strategy(_vec_options: &[Position]) -> Result<Self, StrategyError>
        where
            Self: Sized,
        {
            Ok(ValidStrategy)
        }
    }

    #[test]
    fn test_get_strategy_success() {
        let options = vec![];
        let result = ValidStrategy::get_strategy(&options);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ValidStrategy);
    }
}
