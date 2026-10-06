//! Test-only support: the trait conformance check every concrete strategy
//! runs. It is compiled only for this crate's own unit tests and is not part
//! of the public API (#534).

/// Generates a test module asserting that a strategy type implements all the
/// traits the system requires of a strategy.
///
/// The checks are compile-time `static_assertions` (a dev-dependency of this
/// crate), wrapped in a `#[test]` so each strategy reports them as a test.
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
/// - `ProbabilityAnalysis`
/// - `Greeks`
/// - `DeltaNeutrality`
/// - `PnLCalculator`
/// - `Serialize`
/// - `Deserialize<'static>`
/// - `std::fmt::Display`
///
/// The traits are resolved in the invoking module (`use super::*`), so the
/// strategy file must have them in scope. Invoke it behind `#[cfg(test)]`:
///
/// ```ignore
/// #[cfg(test)]
/// crate::strategies::macros::test_strategy_traits!(LongCall, test_long_call_implementations);
/// ```
///
/// `Graph` is not asserted here: it is a visualization trait, and the visualization layer's
/// tests assert it for every concrete strategy.
macro_rules! test_strategy_traits {
    ($strategy_type:ty, $module_name:ident) => {
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

pub(crate) use test_strategy_traits;
