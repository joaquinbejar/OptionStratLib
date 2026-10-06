//! Facade paths and direct-component paths name the same types (#520, #528,
//! #529, #530, #531, #536).
//!
//! Each function takes a type from its defining component crate and is
//! called with a value obtained through the `optionstratlib` facade (or the
//! other way round). A facade wrapper or copy would fail to compile here.

use optionstratlib::prelude::pos_or_panic;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn core_options(value: optionstratlib_core::model::Options) -> optionstratlib_core::model::Options {
    value
}

fn core_position(
    value: optionstratlib_core::model::Position,
) -> optionstratlib_core::model::Position {
    value
}

fn math_curve(value: optionstratlib_math::curves::Curve) -> optionstratlib_math::curves::Curve {
    value
}

fn math_surface(
    value: optionstratlib_math::surfaces::Surface,
) -> optionstratlib_math::surfaces::Surface {
    value
}

fn facade_option() -> optionstratlib::Options {
    optionstratlib::Options::new(
        optionstratlib::OptionType::European,
        optionstratlib::Side::Long,
        "XYZ".to_string(),
        pos_or_panic!(100.0),
        optionstratlib::ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        optionstratlib::prelude::Positive::ONE,
        pos_or_panic!(100.0),
        dec!(0.05),
        optionstratlib::OptionStyle::Call,
        optionstratlib::prelude::Positive::ZERO,
        None,
    )
}

#[test]
fn test_core_types_through_facade_root_model_and_prelude() {
    let through_root = facade_option();
    let through_core = core_options(through_root.clone());
    let back: optionstratlib::model::Options = through_core;
    let from_prelude: optionstratlib::prelude::Options = back;
    assert_eq!(from_prelude, through_root);

    let position = optionstratlib::model::Position::new(
        from_prelude,
        pos_or_panic!(2.5),
        chrono::Utc::now(),
        optionstratlib::prelude::Positive::ONE,
        optionstratlib::prelude::Positive::ONE,
        None,
        None,
    );
    let position: optionstratlib::prelude::Position = core_position(position);
    assert_eq!(position.option.strike_price, pos_or_panic!(100.0));
}

#[test]
fn test_core_errors_through_facade_error() {
    let from_core: optionstratlib_core::error::DecimalError =
        optionstratlib_core::error::DecimalError::invalid_value(-1.0, "probe");
    let through_facade: optionstratlib::error::DecimalError = from_core;
    let _: optionstratlib_core::error::PositionError =
        optionstratlib::error::PositionError::invalid_position_size(-1.0, "probe");
    assert!(through_facade.to_string().contains("probe"));
}

#[test]
fn test_math_types_through_facade_modules_and_prelude() {
    use optionstratlib::curves::{Curve, Point2D};
    use optionstratlib::geometrics::GeometricObject;
    use optionstratlib::surfaces::{Point3D, Surface};

    let curve = math_curve(Curve::from_vector(vec![
        Point2D::new(Decimal::ZERO, Decimal::ZERO),
        Point2D::new(Decimal::ONE, Decimal::ONE),
    ]));
    let from_prelude: optionstratlib::prelude::Curve = curve;
    assert_eq!(from_prelude.points.len(), 2);

    let surface = math_surface(Surface::from_vector(vec![Point3D::new(
        Decimal::ZERO,
        Decimal::ZERO,
        Decimal::ZERO,
    )]));
    let from_prelude: optionstratlib::prelude::Surface = surface;
    assert_eq!(from_prelude.points.len(), 1);

    let _: optionstratlib_math::geometrics::InterpolationType =
        optionstratlib::geometrics::InterpolationType::Linear;
    let _: optionstratlib_math::error::CurveError =
        optionstratlib::error::CurveError::ConstructionError("probe".to_string());
}

#[test]
fn test_core_modules_root_types_and_macros_through_facade() {
    let _: optionstratlib_core::utils::TimeFrame = optionstratlib::utils::TimeFrame::Day;
    let _: optionstratlib_core::model::ExpirationDate =
        optionstratlib::ExpirationDate::Days(pos_or_panic!(1.0));
    assert_eq!(
        optionstratlib::constants::TOLERANCE,
        optionstratlib_core::constants::TOLERANCE
    );
    let through_facade_macro = optionstratlib::f2du!(1.5);
    let through_core_macro = optionstratlib_core::f2du!(1.5);
    assert!(matches!(
        (through_facade_macro, through_core_macro),
        (Ok(a), Ok(b)) if a == b && a == dec!(1.5)
    ));
}

/// Compiles only when both arguments have the same type. Every function item
/// has its own type, so passing the facade path and the component path of a
/// function proves the facade re-exports that function rather than wrapping it.
fn same_item<T>(_: T, _: T) {}

/// How far the below/inside/above probabilities of one price range may sum
/// from one: the three come from separately rounded lognormal CDF values.
const PROBABILITY_SUM_TOLERANCE: Decimal = dec!(0.01);

fn pricing_engine(
    value: optionstratlib_pricing::pricing::GenericPricingEngine,
) -> optionstratlib_pricing::pricing::GenericPricingEngine {
    value
}

fn market_chain(
    value: optionstratlib_market::chains::OptionChain,
) -> optionstratlib_market::chains::OptionChain {
    value
}

fn market_series(
    value: optionstratlib_market::series::OptionSeries,
) -> optionstratlib_market::series::OptionSeries {
    value
}

#[test]
fn test_pricing_items_through_facade_modules_and_prelude() {
    same_item(
        optionstratlib::pricing::black_scholes,
        optionstratlib_pricing::pricing::black_scholes,
    );
    same_item(
        optionstratlib::prelude::black_scholes,
        optionstratlib_pricing::pricing::black_scholes,
    );
    same_item(
        optionstratlib::greeks::delta,
        optionstratlib_pricing::greeks::delta,
    );
    same_item(
        optionstratlib::volatility::implied_volatility,
        optionstratlib_pricing::volatility::implied_volatility,
    );

    let engine = pricing_engine(optionstratlib::pricing::GenericPricingEngine::ClosedFormBS);
    assert!(matches!(
        engine,
        optionstratlib::pricing::GenericPricingEngine::ClosedFormBS
    ));

    let option = facade_option();
    let through_facade = optionstratlib::pricing::black_scholes(&option);
    let through_component = optionstratlib_pricing::pricing::black_scholes(&option);
    assert!(matches!(
        (through_facade, through_component),
        (Ok(a), Ok(b)) if a == b && a > Decimal::ZERO
    ));

    let _: optionstratlib_pricing::error::PricingError =
        optionstratlib::error::PricingError::other("probe");
    let _: optionstratlib_pricing::error::GreeksError =
        optionstratlib::prelude::GreeksError::delta_error("probe");
}

#[test]
fn test_market_types_through_facade_modules_and_prelude() {
    let chain = market_chain(optionstratlib::chains::OptionChain::new(
        "XYZ",
        pos_or_panic!(100.0),
        "2030-01-17".to_string(),
        None,
        None,
    ));
    let from_prelude: optionstratlib::prelude::OptionChain = chain;
    assert_eq!(from_prelude.symbol, "XYZ");

    let series = market_series(optionstratlib::series::OptionSeries::new(
        "XYZ".to_string(),
        pos_or_panic!(100.0),
    ));
    let from_prelude: optionstratlib::prelude::OptionSeries = series;
    assert_eq!(from_prelude.symbol, "XYZ");

    let _: optionstratlib_market::error::ChainError =
        optionstratlib::error::ChainError::invalid_strike(-1.0, "probe");
}

fn analytics_pnl(value: optionstratlib_analytics::pnl::PnL) -> optionstratlib_analytics::pnl::PnL {
    value
}

#[test]
fn test_analytics_items_through_facade_modules_and_prelude() {
    let pnl = analytics_pnl(optionstratlib::pnl::PnL::new(
        Some(dec!(10.0)),
        None,
        optionstratlib::prelude::Positive::ONE,
        optionstratlib::prelude::Positive::ZERO,
        chrono::Utc::now(),
    ));
    let from_prelude: optionstratlib::prelude::PnL = pnl;
    assert_eq!(from_prelude.realized, Some(dec!(10.0)));

    same_item(
        optionstratlib::analytics::calculate_price_probability,
        optionstratlib_analytics::analytics::calculate_price_probability,
    );
    same_item(
        optionstratlib::risk::SPANMargin::new,
        optionstratlib_analytics::risk::SPANMargin::new,
    );

    let _: optionstratlib_analytics::error::TransactionError =
        optionstratlib::error::TransactionError::other("probe");
    let _: optionstratlib_analytics::error::ProbabilityError =
        optionstratlib::prelude::ProbabilityError::invalid_probability(1.5, "probe");
    let _: optionstratlib_analytics::error::probability::ProbabilityResult<()> =
        optionstratlib::error::probability::ProbabilityResult::<()>::Ok(());
}

fn analytics_volatility_adjustment(
    value: optionstratlib_analytics::analytics::VolatilityAdjustment,
) -> optionstratlib_analytics::analytics::VolatilityAdjustment {
    value
}

fn analytics_price_trend(
    value: optionstratlib_analytics::analytics::PriceTrend,
) -> optionstratlib_analytics::analytics::PriceTrend {
    value
}

fn analytics_delta_adjustment(
    value: optionstratlib_analytics::pnl::DeltaAdjustment,
) -> optionstratlib_analytics::pnl::DeltaAdjustment {
    value
}

fn analytics_delta_adjustment_same_size(
    value: optionstratlib_analytics::pnl::DeltaAdjustmentSameSize,
) -> optionstratlib_analytics::pnl::DeltaAdjustmentSameSize {
    value
}

/// The probability kernels and their inputs have one public path,
/// `optionstratlib::analytics`, and it names the analytics crate's items. The
/// `strategies::probabilities` aliases are gone (#530).
#[test]
fn test_probability_kernels_through_facade_analytics() {
    same_item(
        optionstratlib::analytics::calculate_price_probability,
        optionstratlib_analytics::analytics::calculate_price_probability,
    );
    same_item(
        optionstratlib::analytics::calculate_single_point_probability,
        optionstratlib_analytics::analytics::calculate_single_point_probability,
    );
    // Both paths name the same trait, so they resolve to the same method item.
    same_item(
        <optionstratlib::analytics::ProfitLossRange as optionstratlib::analytics::ProfitRangeProbability>::calculate_probability,
        <optionstratlib_analytics::analytics::ProfitLossRange as optionstratlib_analytics::analytics::ProfitRangeProbability>::calculate_probability,
    );

    let volatility =
        analytics_volatility_adjustment(optionstratlib::analytics::VolatilityAdjustment {
            base_volatility: pos_or_panic!(0.2),
            std_dev_adjustment: optionstratlib::prelude::Positive::ZERO,
        });
    let trend = analytics_price_trend(
        match optionstratlib::analytics::PriceTrend::new(dec!(0.0), dec!(0.95)) {
            Ok(trend) => trend,
            Err(e) => panic!("valid trend: {e}"),
        },
    );
    let probabilities = optionstratlib::analytics::calculate_price_probability(
        &pos_or_panic!(100.0),
        &pos_or_panic!(95.0),
        &pos_or_panic!(105.0),
        volatility,
        Some(trend),
        &optionstratlib::ExpirationDate::Days(pos_or_panic!(30.0)),
        None,
    );
    assert!(matches!(
        probabilities,
        Ok((below, inside, above))
            if ((below + inside + above).to_dec() - Decimal::ONE).abs() <= PROBABILITY_SUM_TOLERANCE
    ));
}

/// `DeltaAdjustment` and `DeltaAdjustmentSameSize` are reachable only through
/// `optionstratlib::pnl`, and `DELTA_THRESHOLD` only through
/// `optionstratlib::greeks` (#530).
#[test]
fn test_delta_adjustment_types_through_facade_pnl_and_greeks() {
    let adjustment =
        analytics_delta_adjustment(optionstratlib::pnl::DeltaAdjustment::NoAdjustmentNeeded);
    let same_size =
        analytics_delta_adjustment_same_size(optionstratlib::pnl::DeltaAdjustmentSameSize {
            first: Box::new(optionstratlib::pnl::DeltaAdjustment::NoAdjustmentNeeded),
            second: Box::new(adjustment),
        });
    assert_eq!(
        *same_size.first,
        optionstratlib::pnl::DeltaAdjustment::NoAdjustmentNeeded
    );

    let through_facade: &Decimal = &optionstratlib::greeks::DELTA_THRESHOLD;
    assert_eq!(
        *through_facade,
        optionstratlib_pricing::greeks::DELTA_THRESHOLD
    );
    assert_eq!(*through_facade, optionstratlib::prelude::DELTA_THRESHOLD);
}

/// Strategy adjustment P&L stays on the upper layer: the strategy's
/// `DeltaNeutrality` adapter returns the analytics-owned
/// `pnl::DeltaAdjustment`, and the strategy's `PnLCalculator` prices it.
#[test]
fn test_strategy_delta_adjustments_are_analytics_owned_and_priced()
-> Result<(), Box<dyn std::error::Error>> {
    use optionstratlib::pnl::PnLCalculator;
    use optionstratlib::strategies::{DeltaNeutrality, ShortStrangle};

    let strategy = ShortStrangle::new(
        "CL".to_string(),
        pos_or_panic!(7138.5),
        pos_or_panic!(7450.0),
        pos_or_panic!(7250.0),
        optionstratlib::ExpirationDate::Days(pos_or_panic!(45.0)),
        pos_or_panic!(0.19),
        pos_or_panic!(0.21),
        dec!(0.05),
        optionstratlib::prelude::Positive::ZERO,
        optionstratlib::prelude::Positive::ONE,
        pos_or_panic!(84.2),
        pos_or_panic!(353.2),
        pos_or_panic!(7.01),
        pos_or_panic!(7.01),
        pos_or_panic!(7.01),
        pos_or_panic!(7.01),
    )?;

    let adjustments: Vec<optionstratlib_analytics::pnl::DeltaAdjustment> =
        strategy.delta_adjustments()?;
    let priced = adjustments
        .iter()
        .filter(|adjustment| {
            !matches!(
                adjustment,
                optionstratlib::pnl::DeltaAdjustment::NoAdjustmentNeeded
            )
        })
        .map(|adjustment| strategy.adjustments_pnl(adjustment))
        .collect::<Result<Vec<optionstratlib_analytics::pnl::PnL>, _>>()?;
    assert!(!priced.is_empty());
    Ok(())
}

fn strategies_bull_call_spread(
    value: optionstratlib_strategies::strategies::BullCallSpread,
) -> optionstratlib_strategies::strategies::BullCallSpread {
    value
}

fn strategies_request(
    value: optionstratlib_strategies::strategies::StrategyRequest,
) -> optionstratlib_strategies::strategies::StrategyRequest {
    value
}

fn strategies_type(
    value: optionstratlib_strategies::strategies::base::StrategyType,
) -> optionstratlib_strategies::strategies::base::StrategyType {
    value
}

/// The strategies, their traits and `StrategyError` reached through the
/// facade (`strategies`, `error`, `prelude`) are the
/// `optionstratlib-strategies` items, and `FindOptimalSide` has one public
/// path, the market one: the strategies re-export is gone (#531).
#[test]
fn test_strategies_items_through_facade_modules_and_prelude() {
    let spread = strategies_bull_call_spread(optionstratlib::strategies::BullCallSpread::default());
    let from_prelude: optionstratlib::prelude::BullCallSpread = spread;
    assert_eq!(
        optionstratlib::prelude::BasicAble::get_title(&from_prelude),
        optionstratlib_strategies::strategies::BasicAble::get_title(
            &optionstratlib_strategies::strategies::BullCallSpread::default()
        )
    );

    let request = strategies_request(optionstratlib::strategies::StrategyRequest::new(
        optionstratlib::strategies::base::StrategyType::BullCallSpread,
        Vec::new(),
    ));
    assert_eq!(
        strategies_type(request.strategy_type),
        optionstratlib::prelude::StrategyType::BullCallSpread
    );

    // Both paths name the same trait, so they resolve to the same method item.
    same_item(
        <optionstratlib::strategies::IronCondor as optionstratlib::strategies::Strategies>::get_max_profit,
        <optionstratlib_strategies::strategies::IronCondor as optionstratlib_strategies::strategies::Strategies>::get_max_profit,
    );
    same_item(
        <optionstratlib::strategies::LongCall as optionstratlib::prelude::DeltaNeutrality>::delta_neutrality,
        <optionstratlib_strategies::strategies::LongCall as optionstratlib_strategies::strategies::DeltaNeutrality>::delta_neutrality,
    );

    let _: optionstratlib_strategies::error::StrategyError =
        optionstratlib::prelude::StrategyError::operation_not_supported("probe", "probe");
    let _: optionstratlib_strategies::error::strategies::StrategyResult<()> =
        optionstratlib::error::strategies::StrategyResult::<()>::Ok(());

    let side: optionstratlib_market::chains::utils::FindOptimalSide =
        optionstratlib::prelude::FindOptimalSide::All;
    assert!(matches!(
        side,
        optionstratlib::chains::utils::FindOptimalSide::All
    ));
}

fn simulation_exit_policy(
    value: optionstratlib_simulation::simulation::ExitPolicy,
) -> optionstratlib_simulation::simulation::ExitPolicy {
    value
}

fn simulation_ystep(
    value: optionstratlib_simulation::simulation::steps::Ystep<Decimal>,
) -> optionstratlib_simulation::simulation::steps::Ystep<Decimal> {
    value
}

/// The walks, steps, exit policies and `SimulationError` reached through the
/// facade (`simulation`, `error`, `prelude`) are the
/// `optionstratlib-simulation` items (#536).
#[test]
fn test_simulation_items_through_facade_modules_and_prelude() {
    let policy = simulation_exit_policy(optionstratlib::simulation::ExitPolicy::ProfitPercent(
        dec!(0.5),
    ));
    let from_prelude: optionstratlib::prelude::ExitPolicy = policy;
    assert_eq!(
        from_prelude,
        optionstratlib_simulation::simulation::ExitPolicy::ProfitPercent(dec!(0.5))
    );

    let step = simulation_ystep(optionstratlib::prelude::Ystep::new(1, dec!(10.5)));
    assert_eq!(*step.value(), dec!(10.5));

    // Both paths name the same function item.
    same_item(
        optionstratlib::simulation::generate_ou_process
            as fn(_, _, _, _, _, _) -> Result<_, optionstratlib::error::SimulationError>,
        optionstratlib_simulation::simulation::generate_ou_process
            as fn(_, _, _, _, _, _) -> Result<_, optionstratlib_simulation::error::SimulationError>,
    );

    let _: optionstratlib_simulation::error::SimulationError =
        optionstratlib::error::SimulationError::invalid_parameters("probe");
    let _: optionstratlib_simulation::error::simulation::SimulationResult<()> =
        optionstratlib::error::simulation::SimulationResult::<()>::Ok(());
    let _: optionstratlib_simulation::error::SimulationResult<()> =
        optionstratlib::error::SimulationResult::<()>::Ok(());
}

/// The synthetic generators are `optionstratlib-market` items, reached
/// through the facade's `chains` / `series` modules and the prelude (#537).
#[cfg(feature = "synthetic")]
#[test]
fn test_synthetic_generators_through_facade_modules_and_prelude() {
    same_item(
        optionstratlib::chains::generator_optionchain,
        optionstratlib_market::chains::generator_optionchain,
    );
    same_item(
        optionstratlib::prelude::generator_optionchain,
        optionstratlib_market::chains::generator_optionchain,
    );
    same_item(
        optionstratlib::series::generator_optionseries,
        optionstratlib_market::series::generator_optionseries,
    );
    same_item(
        optionstratlib::prelude::generator_optionseries,
        optionstratlib_market::series::generator_optionseries,
    );
    let error: optionstratlib_market::error::ChainError = optionstratlib::error::ChainError::from(
        optionstratlib::error::SimulationError::walk_error("probe"),
    );
    assert!(matches!(
        error,
        optionstratlib::error::ChainError::Generator(_)
    ));
}
