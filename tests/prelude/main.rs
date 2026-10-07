//! The facade prelude names the items its defining crates own, on every
//! feature surface (#551).
//!
//! This target has no `required-features`, so it builds with the facade
//! defaults, with `--all-features`, with `--no-default-features` and with
//! each capability alone (`make lint` runs the facade tests once per entry
//! of `FACADE_FEATURE_SETS`). Each capability's checks are gated by the same
//! feature as its prelude items.
//!
//! `same_items!` imports each item explicitly from the prelude and from its
//! defining module (either import fails if that path stops exporting it),
//! then glob-imports both and names the item through the globs: two
//! different items under one name are ambiguous (`E0659`) and the target
//! fails to compile. It covers types, traits and macros alike.

use std::any::TypeId;

macro_rules! same_items {
    ($check:ident, [$($component:ident)::+], [$($name:ident),+ $(,)?]) => {
        #[allow(unused_imports)]
        mod $check {
            use $($component)::+ as component;
            use optionstratlib::prelude::*;
            use self::component::*;
            $(
                use optionstratlib::prelude::$name as _;
                use self::component::$name as _;
                use self::$name as _;
            )+
        }
    };
}

// Core: always present.
same_items!(
    core_model,
    [optionstratlib_core::model],
    [
        Options,
        Position,
        ExpirationDate,
        OptionStyle,
        OptionType,
        Side,
        Positive,
    ]
);
same_items!(core_utils, [optionstratlib_core::utils], [Len, TimeFrame]);
same_items!(
    core_macros,
    [optionstratlib_core],
    [pos_or_panic, spos, assert_pos_relative_eq]
);
same_items!(third_party_decimal, [rust_decimal], [Decimal]);
same_items!(third_party_dec, [rust_decimal_macros], [dec]);
same_items!(third_party_chrono, [chrono], [Utc]);
same_items!(
    third_party_tracing,
    [tracing],
    [debug, error, info, trace, warn]
);

#[cfg(feature = "math")]
same_items!(math_curves, [optionstratlib_math::curves], [Curve, Point2D]);
#[cfg(feature = "math")]
same_items!(
    math_surfaces,
    [optionstratlib_math::surfaces],
    [Surface, Point3D]
);
#[cfg(feature = "math")]
same_items!(
    math_geometrics,
    [optionstratlib_math::geometrics],
    [GeometricObject, ConstructionMethod, ConstructionParams,]
);

#[cfg(feature = "pricing")]
same_items!(pricing_greeks, [optionstratlib_pricing::greeks], [Greeks]);
#[cfg(feature = "pricing")]
same_items!(
    pricing_pricing,
    [optionstratlib_pricing::pricing],
    [OptionPricing, Profit]
);
#[cfg(feature = "pricing")]
same_items!(
    pricing_volatility,
    [optionstratlib_pricing::volatility],
    [VolatilitySmile]
);

#[cfg(feature = "market")]
same_items!(
    market_chains,
    [optionstratlib_market::chains],
    [
        OptionChain,
        OptionData,
        OptionChainBuildParams,
        FindOptimalSide,
    ]
);
#[cfg(feature = "market")]
same_items!(
    market_chain_utils,
    [optionstratlib_market::chains::utils],
    [OptionDataPriceParams]
);
#[cfg(feature = "market")]
same_items!(
    market_series,
    [optionstratlib_market::series],
    [OptionSeries, OptionSeriesBuildParams]
);

#[cfg(feature = "analytics")]
same_items!(
    analytics_pnl,
    [optionstratlib_analytics::pnl],
    [PnL, PnLCalculator]
);
#[cfg(feature = "analytics")]
same_items!(
    analytics_projections,
    [optionstratlib_analytics::analytics],
    [BasicCurves, BasicSurfaces,]
);

#[cfg(feature = "strategies")]
same_items!(
    strategies_flat,
    [optionstratlib_strategies::strategies],
    [
        BasicAble,
        Strategable,
        Strategies,
        StrategyConstructor,
        Validable,
        DeltaNeutrality,
        BearCallSpread,
        BearPutSpread,
        BullCallLadder,
        BullCallSpread,
        BullPutSpread,
        Collar,
        CoveredCall,
        IronButterfly,
        IronCondor,
        LongButterflySpread,
        LongCall,
        LongPut,
        LongStraddle,
        LongStrangle,
        PoorMansCoveredCall,
        ProtectivePut,
        ShortButterflySpread,
        ShortCall,
        ShortPut,
        ShortStraddle,
        ShortStrangle,
    ]
);
#[cfg(feature = "strategies")]
same_items!(
    strategies_base,
    [optionstratlib_strategies::strategies::base],
    [BreakEvenable, Optimizable, Positionable,]
);
#[cfg(feature = "strategies")]
same_items!(
    strategies_custom,
    [optionstratlib_strategies::strategies::custom],
    [CustomStrategy]
);
#[cfg(feature = "strategies")]
same_items!(
    strategies_probabilities,
    [optionstratlib_strategies::strategies::probabilities],
    [ProbabilityAnalysis,]
);

#[cfg(feature = "simulation")]
same_items!(
    simulation_flat,
    [optionstratlib_simulation::simulation],
    [ExitPolicy, WalkParams, WalkType, WalkTypeAble,]
);
#[cfg(feature = "simulation")]
same_items!(
    simulation_randomwalk,
    [optionstratlib_simulation::simulation::randomwalk],
    [RandomWalk]
);
#[cfg(feature = "simulation")]
same_items!(
    simulation_simulator,
    [optionstratlib_simulation::simulation::simulator],
    [Simulator]
);
#[cfg(feature = "simulation")]
same_items!(
    simulation_steps,
    [optionstratlib_simulation::simulation::steps],
    [Step, Xstep, Ystep]
);

#[cfg(feature = "backtest")]
same_items!(
    backtest_flat,
    [optionstratlib_backtest::backtesting],
    [Simulate]
);

#[cfg(feature = "visualization")]
same_items!(
    visualization_flat,
    [optionstratlib_visualization::visualization],
    [Graph, Plottable]
);

/// The core prelude alone builds and prices nothing, but is enough to state
/// a contract: it is the whole prelude under `--no-default-features`.
#[test]
fn test_prelude_core_items_build_a_contract() {
    use optionstratlib::prelude::*;

    let option = Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".to_string(),
        pos_or_panic!(100.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        Positive::ONE,
        pos_or_panic!(100.0),
        dec!(0.05),
        OptionStyle::Call,
        Positive::ZERO,
        None,
    );
    assert_eq!(option.strike_price, pos_or_panic!(100.0));
    assert_eq!(
        TypeId::of::<Options>(),
        TypeId::of::<optionstratlib_core::model::Options>()
    );
    assert_eq!(
        TypeId::of::<Decimal>(),
        TypeId::of::<rust_decimal::Decimal>()
    );
}

/// With `pricing`, the prelude's `OptionPricing` puts the pricing methods on
/// `Options`, and `Greeks` and `Profit` their methods.
#[cfg(feature = "pricing")]
#[test]
fn test_prelude_pricing_traits_reach_option_methods() {
    use optionstratlib::prelude::*;

    let option = Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".to_string(),
        pos_or_panic!(40.0),
        ExpirationDate::Days(pos_or_panic!(182.5)),
        pos_or_panic!(0.2),
        Positive::ONE,
        pos_or_panic!(42.0),
        dec!(0.10),
        OptionStyle::Call,
        Positive::ZERO,
        None,
    );
    let price = match option.calculate_price_black_scholes() {
        Ok(price) => price,
        Err(e) => panic!("prices: {e}"),
    };
    // Hull's worked example: 4.76.
    assert!((price - dec!(4.7594)).abs() < dec!(0.001), "{price}");
    assert!(option.delta().is_ok());
    assert!(option.calculate_profit_at(&pos_or_panic!(50.0)).is_ok());
}
