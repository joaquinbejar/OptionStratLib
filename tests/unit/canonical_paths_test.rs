//! Every canonical 0.22 facade path names the type its defining component
//! crate owns (#550).
//!
//! One row per canonical path: the facade path on the left, the defining
//! crate's path on the right, compared by `TypeId`. A facade module is a
//! `pub use` of the component module, so one type per module stands for the
//! module; every flat error re-export and root re-export is listed on its
//! own. The table mirrors the canonical-path table of #550, and a facade
//! wrapper or copy of any type fails here.

use std::any::TypeId;

/// Asserts that two paths name the same `'static` type.
macro_rules! same {
    ($facade:ty, $component:ty) => {
        assert_eq!(
            TypeId::of::<$facade>(),
            TypeId::of::<$component>(),
            "{} is not {}",
            stringify!($facade),
            stringify!($component),
        );
    };
}

#[test]
fn test_canonical_root_paths_are_core_types() {
    same!(optionstratlib::Options, optionstratlib_core::model::Options);
    same!(
        optionstratlib::ExpirationDate,
        optionstratlib_core::model::ExpirationDate
    );
    same!(
        optionstratlib::OptionStyle,
        optionstratlib_core::model::OptionStyle
    );
    same!(
        optionstratlib::OptionType,
        optionstratlib_core::model::OptionType
    );
    same!(
        optionstratlib::RainbowType,
        optionstratlib_core::model::RainbowType
    );
    same!(optionstratlib::Side, optionstratlib_core::model::Side);
}

#[test]
fn test_canonical_module_paths_are_component_modules() {
    // core
    same!(
        optionstratlib::model::Position,
        optionstratlib_core::model::Position
    );
    same!(
        optionstratlib::model::types::Action,
        optionstratlib_core::model::types::Action
    );
    same!(
        optionstratlib::utils::TimeFrame,
        optionstratlib_core::utils::TimeFrame
    );
    assert_eq!(
        optionstratlib::constants::PI,
        optionstratlib_core::constants::PI
    );
    // math
    same!(
        optionstratlib::curves::Curve,
        optionstratlib_math::curves::Curve
    );
    same!(
        optionstratlib::surfaces::Surface,
        optionstratlib_math::surfaces::Surface
    );
    same!(
        optionstratlib::geometrics::ConstructionParams,
        optionstratlib_math::geometrics::ConstructionParams
    );
    // pricing
    same!(
        optionstratlib::pricing::BinomialPricingParams<'static>,
        optionstratlib_pricing::pricing::BinomialPricingParams<'static>
    );
    same!(
        optionstratlib::greeks::GreeksSnapshot,
        optionstratlib_pricing::greeks::GreeksSnapshot
    );
    same!(
        dyn optionstratlib::volatility::VolatilitySmile,
        dyn optionstratlib_pricing::volatility::VolatilitySmile
    );
    // simulation
    same!(
        optionstratlib::simulation::ExitPolicy,
        optionstratlib_simulation::simulation::ExitPolicy
    );
    // market
    same!(
        optionstratlib::chains::OptionChain,
        optionstratlib_market::chains::OptionChain
    );
    same!(
        optionstratlib::series::OptionSeries,
        optionstratlib_market::series::OptionSeries
    );
    // analytics
    same!(
        optionstratlib::analytics::PriceTrend,
        optionstratlib_analytics::analytics::PriceTrend
    );
    same!(optionstratlib::pnl::PnL, optionstratlib_analytics::pnl::PnL);
    same!(
        optionstratlib::risk::SPANMargin,
        optionstratlib_analytics::risk::SPANMargin
    );
    same!(
        dyn optionstratlib::metrics::SmileDynamicsCurve,
        dyn optionstratlib_analytics::metrics::SmileDynamicsCurve
    );
    // strategies
    same!(
        optionstratlib::strategies::BullCallSpread,
        optionstratlib_strategies::strategies::BullCallSpread
    );
    same!(
        optionstratlib::strategies::base::StrategyType,
        optionstratlib_strategies::strategies::base::StrategyType
    );
    // backtest
    same!(
        optionstratlib::backtesting::GeneralPerformanceMetrics,
        optionstratlib_backtest::backtesting::GeneralPerformanceMetrics
    );
    // visualization
    same!(
        optionstratlib::visualization::GraphData,
        optionstratlib_visualization::visualization::GraphData
    );
}

#[test]
fn test_canonical_error_paths_are_component_errors() {
    use optionstratlib::error as facade;

    // core
    same!(
        facade::DecimalError,
        optionstratlib_core::error::DecimalError
    );
    same!(
        facade::DecimalResult<()>,
        optionstratlib_core::error::DecimalResult<()>
    );
    same!(
        facade::OptionsError,
        optionstratlib_core::error::OptionsError
    );
    same!(
        facade::OptionsResult<()>,
        optionstratlib_core::error::OptionsResult<()>
    );
    same!(
        facade::PositionError,
        optionstratlib_core::error::PositionError
    );
    same!(facade::TradeError, optionstratlib_core::error::TradeError);
    same!(
        facade::OperationErrorKind,
        optionstratlib_core::error::OperationErrorKind
    );
    same!(
        facade::position::PositionValidationErrorKind,
        optionstratlib_core::error::position::PositionValidationErrorKind
    );
    // math
    same!(facade::CurveError, optionstratlib_math::error::CurveError);
    same!(
        facade::CurvesResult<()>,
        optionstratlib_math::error::CurvesResult<()>
    );
    same!(
        facade::InterpolationError,
        optionstratlib_math::error::InterpolationError
    );
    same!(
        facade::MetricsError,
        optionstratlib_math::error::MetricsError
    );
    same!(
        facade::SurfaceError,
        optionstratlib_math::error::SurfaceError
    );
    // pricing
    same!(
        facade::PricingError,
        optionstratlib_pricing::error::PricingError
    );
    same!(
        facade::PricingResult<()>,
        optionstratlib_pricing::error::PricingResult<()>
    );
    same!(
        facade::GreeksError,
        optionstratlib_pricing::error::GreeksError
    );
    same!(
        facade::VolatilityError,
        optionstratlib_pricing::error::VolatilityError
    );
    same!(
        facade::GreeksResult<()>,
        optionstratlib_pricing::error::GreeksResult<()>
    );
    same!(
        facade::greeks::GreeksResult<()>,
        optionstratlib_pricing::error::greeks::GreeksResult<()>
    );
    same!(
        facade::greeks::InputErrorKind,
        optionstratlib_pricing::error::greeks::InputErrorKind
    );
    // simulation
    same!(
        facade::SimulationError,
        optionstratlib_simulation::error::SimulationError
    );
    same!(
        facade::SimulationResult<()>,
        optionstratlib_simulation::error::SimulationResult<()>
    );
    // market
    same!(facade::ChainError, optionstratlib_market::error::ChainError);
    same!(facade::OhlcvError, optionstratlib_market::error::OhlcvError);
    same!(
        facade::chains::OptionDataErrorKind,
        optionstratlib_market::error::chains::OptionDataErrorKind
    );
    // analytics
    same!(
        facade::ProbabilityError,
        optionstratlib_analytics::error::ProbabilityError
    );
    same!(
        facade::ProbabilityResult<()>,
        optionstratlib_analytics::error::ProbabilityResult<()>
    );
    same!(
        facade::ProjectionError,
        optionstratlib_analytics::error::ProjectionError
    );
    same!(
        facade::TransactionError,
        optionstratlib_analytics::error::TransactionError
    );
    same!(
        facade::probability::ProbabilityCalculationErrorKind,
        optionstratlib_analytics::error::probability::ProbabilityCalculationErrorKind
    );
    // strategies
    same!(
        facade::StrategyError,
        optionstratlib_strategies::error::StrategyError
    );
    same!(
        facade::StrategyResult<()>,
        optionstratlib_strategies::error::StrategyResult<()>
    );
    same!(
        facade::strategies::BreakEvenErrorKind,
        optionstratlib_strategies::error::strategies::BreakEvenErrorKind
    );
    // backtest and visualization
    same!(
        facade::BacktestError,
        optionstratlib_backtest::error::BacktestError
    );
    same!(
        facade::GraphError,
        optionstratlib_visualization::error::GraphError
    );
    // The aggregate is facade-owned; it is not in the prelude (#551).
    same!(facade::Error, optionstratlib::error::Error);
}
