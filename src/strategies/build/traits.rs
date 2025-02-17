/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 16/2/25
******************************************************************************/
use crate::error::StrategyError;
use crate::greeks::Greeks;
use crate::model::Position;
use crate::strategies::Strategies;

pub trait StrategyConstructor: Strategies + Greeks  {
    fn get_strategy(_vec_options: &[Position]) -> Result<Self, StrategyError>
    where
        Self: Sized,
    {
        Err(StrategyError::NotImplemented)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{GreeksError, StrategyError};
    use crate::Options;
    use crate::strategies::base::{BreakEvenable, Positionable, StrategyBasic, Validable};
    use crate::strategies::Strategies;

    /// Mock para una estrategia específica
    #[derive(Debug, PartialEq)]
    struct TestStrategy;

    impl Validable for TestStrategy {}

    impl Positionable for TestStrategy {}

    impl BreakEvenable for TestStrategy {}

    impl StrategyBasic for TestStrategy {}

    impl Strategies for TestStrategy {}

    impl Greeks for TestStrategy {
        fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
            todo!()
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

    impl StrategyBasic for ValidStrategy {}

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
