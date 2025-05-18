//! Simpler macro to test that a strategy implements all required traits

/// Macro to test trait implementations for a specific strategy type.
///
/// This macro generates a module with a test function to ensure that the specified strategy type
/// implements all the necessary traits required by the system. It uses the `static_assertions` crate
/// to perform compile-time checks for these trait implementations.
///
/// # Parameters
///
/// * `$strategy_type:ty` - The type of the strategy to test.
/// * `$module_name:ident` - The name of the module to generate for the test code.
///
/// # Traits Tested
///
/// The macro asserts that the provided type `$strategy_type` implements the following traits:
///
/// - `Default`
/// - `StrategyConstructor`
/// - `BreakEvenable`
/// - `Positionable`
/// - `Strategable`
/// - `Strategies`
/// - `Validable`
/// - `Optimizable`
/// - `Profit`
/// - `Graph`
/// - `ProbabilityAnalysis`
/// - `Greeks`
/// - `DeltaNeutrality`
/// - `PnLCalculator`
/// - `Serialize`
/// - `Deserialize<'static>`
/// - `std::fmt::Display`
///
/// These traits are essential for strategies to function appropriately within the system.
///
/// This will generate a test module named `my_strategy_tests`, which contains a test function to
/// verify that `MyStrategyType` implements all the required traits.
///
/// # Note
///
/// This macro relies on the `static_assertions` crate to perform the compile-time checks. Ensure
/// that the crate is included in your `Cargo.toml`:
///
/// ```toml
/// [dev-dependencies]
/// static_assertions = "1.1"
/// ```
///
/// Additionally, ensure that `$crate::visualization::Graph` is correctly defined and accessible
/// in your project.
#[macro_export]
macro_rules! test_strategy_traits {
    ($strategy_type:ty, $module_name:ident) => {
        #[cfg(test)]
        mod $module_name {
            use super::*;
            use static_assertions::assert_impl_all;
            use std::fmt;
            use $crate::visualization::Graph;

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
