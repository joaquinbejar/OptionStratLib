/// Simpler macro to test that a strategy implements all required traits
#[macro_export]
macro_rules! test_strategy_traits {
    ($strategy_type:ty, $module_name:ident) => {
        #[cfg(test)]
        mod $module_name {
            use super::*;
            use static_assertions::assert_impl_all;
            use std::fmt;

            #[test]
            fn test_traits() {
                assert_impl_all!($strategy_type:
                    Default,
                    StrategyConstructor,
                    BreakEvenable,
                    Positionable,
                    Strategable,
                    Strategies,
                    Validable,
                    Optimizable,
                    Profit,
                    Graph,
                    ProbabilityAnalysis,
                    Greeks,
                    DeltaNeutrality,
                    PnLCalculator,
                    Serialize,
                    Deserialize<'static>,
                    fmt::Display,
                );
            }
        }
    };
}
