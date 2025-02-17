/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 16/2/25
******************************************************************************/
use crate::error::StrategyError;
use crate::model::Position;
use crate::strategies::Strategies;

pub trait StrategyConstructor: Strategies {
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
    use crate::error::StrategyError;
    use crate::strategies::base::{BreakEvenable, Positionable, Validable};
    use crate::strategies::Strategies;

    /// Mock para una estrategia específica
    #[derive(Debug, PartialEq)]
    struct TestStrategy;

    impl Validable for TestStrategy {}

    impl Positionable for TestStrategy {}

    impl BreakEvenable for TestStrategy {}

    impl Strategies for TestStrategy {}

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

    impl Strategies for ValidStrategy {}

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
