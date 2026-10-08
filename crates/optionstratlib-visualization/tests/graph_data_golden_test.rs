//! Regression pin for the chart data of every visualization-owned `Graph`
//! adapter (#543): `Options`, `Position`, `Curve`, `Vec<Curve>`, `Surface`,
//! `PlotBuilder`, `RandomWalk`, `Simulator` and each of the 22 concrete
//! strategies. For each one the `graph_data` and the `graph_config` are
//! serialised and compared with a golden file generated on the code that
//! preceded the extraction of this crate (#542, `624eeda2`), when the same
//! adapters still lived in the facade. Every field must match byte for byte:
//! moving an adapter must not change one point of a chart.
//!
//! Two entries were regenerated on purpose since: `strategy_bull_put_spread`
//! and `strategy_bear_put_spread` (#696), whose builders used to take each
//! other's legs. The golden pinned a bull put spread with a bear put spread's
//! payoff and the reverse; both now chart the textbook legs.
//! `strategy_long_butterfly_spread` and `strategy_short_butterfly_spread`
//! were regenerated for #706, which makes every butterfly the textbook 1/2/1
//! structure (the short one also took a body premium whose credit covers its
//! fees). The same issue removed `CallButterfly` and its
//! `strategy_call_butterfly` entry, and added `strategy_bull_call_ladder` for
//! the 1x1x1 ladder that type actually was. `strategy_covered_call`,
//! `strategy_collar` and `strategy_protective_put` were regenerated for #731,
//! which sizes their option legs in shares with per-share option fees: the
//! covered call is now capped and the collar capped and floored.
//!
//! The strategies that `StrategyRequest` can build are built that way, from
//! their positions, then charted through the concrete type: the
//! builder-to-visualization workflow left by the removal of the `Graph`
//! supertrait from the strategy contract (#658). `StrategyRequest::get_strategy`
//! returns a `Box<dyn Strategable>`, which carries no `Graph`, so a consumer
//! that renders dispatches on `strategy_type` itself, as [`chart_request`]
//! does. The single legs, `CoveredCall`, `Collar` and `ProtectivePut` have no
//! builder (`StrategyError::NotImplemented`) and are built with `new`, or
//! from a default value and one position where the type has no `new`.
//!
//! Every input is deterministic: expirations are relative (`Days`), the
//! walks replay historical prices, and the positions carry a fixed date.
//!
//! Regenerate with `OSL_WRITE_GOLDEN=1 cargo test -p optionstratlib-visualization --test graph_data_golden_test`
//! only when a change of chart data is intended and reviewed.

use chrono::{DateTime, TimeZone, Utc};
use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
use optionstratlib_core::model::{ExpirationDate, Options, Position, Positive};
use optionstratlib_core::utils::TimeFrame;
use optionstratlib_math::curves::{Curve, Point2D};
use optionstratlib_math::geometrics::GeometricObject;
use optionstratlib_math::surfaces::{Point3D, Surface};
use optionstratlib_simulation::simulation::simulator::Simulator;
use optionstratlib_simulation::simulation::steps::Step;
use optionstratlib_simulation::simulation::{
    WalkParams, WalkType, WalkTypeAble, generator_positive,
};
use optionstratlib_strategies::error::StrategyError;
use optionstratlib_strategies::strategies::base::{Positionable, StrategyType};
use optionstratlib_strategies::strategies::custom::CustomStrategy;
use optionstratlib_strategies::strategies::{
    BearCallSpread, BearPutSpread, BullCallLadder, BullCallSpread, BullPutSpread, Collar,
    CoveredCall, IronButterfly, IronCondor, LongButterflySpread, LongCall, LongPut, LongStraddle,
    LongStrangle, PoorMansCoveredCall, ProtectivePut, ShortButterflySpread, ShortCall, ShortPut,
    ShortStraddle, ShortStrangle, StrategyConstructor, StrategyRequest,
};
use optionstratlib_visualization::visualization::{ColorScheme, Graph, LineStyle, Plottable};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use std::error::Error;
use std::path::PathBuf;

/// Underlying symbol of every option.
const SYMBOL: &str = "TEST";

/// Days to expiry of every short-dated option.
const EXPIRY_DAYS: Decimal = dec!(30);

/// Days to expiry of the long leg of the poor man's covered call.
const LEAPS_DAYS: Decimal = dec!(365);

/// Implied volatility of every option, as a fraction per year.
const IMPLIED_VOLATILITY: Decimal = dec!(0.2);

/// Risk-free rate, as a fraction per year.
const RISK_FREE_RATE: Decimal = dec!(0.05);

/// Open and close fee of every leg, in quote currency per contract.
const FEE: Decimal = dec!(0.5);

#[derive(Clone)]
struct HistoricalWalker;

impl WalkTypeAble<Positive, Positive> for HistoricalWalker {}

fn pos(value: Decimal) -> Result<Positive, Box<dyn Error>> {
    Ok(Positive::new_decimal(value)?)
}

fn expiry(days: Decimal) -> Result<ExpirationDate, Box<dyn Error>> {
    Ok(ExpirationDate::Days(pos(days)?))
}

/// The fixed open date of every position.
fn opened() -> Result<DateTime<Utc>, Box<dyn Error>> {
    Utc.with_ymd_and_hms(2025, 1, 2, 15, 30, 0)
        .single()
        .ok_or_else(|| "invalid fixed date".into())
}

/// A European option on `SYMBOL` with the underlying at 100.
fn option(
    side: Side,
    style: OptionStyle,
    strike: Decimal,
    days: Decimal,
) -> Result<Options, Box<dyn Error>> {
    Ok(Options::new(
        OptionType::European,
        side,
        SYMBOL.to_string(),
        pos(strike)?,
        expiry(days)?,
        pos(IMPLIED_VOLATILITY)?,
        Positive::ONE,
        Positive::HUNDRED,
        RISK_FREE_RATE,
        style,
        Positive::ZERO,
        None,
    ))
}

/// One leg of a strategy: a position on [`option`] at `premium`.
fn leg(
    side: Side,
    style: OptionStyle,
    strike: Decimal,
    premium: Decimal,
) -> Result<Position, Box<dyn Error>> {
    leg_expiring(side, style, strike, premium, EXPIRY_DAYS)
}

/// One leg of `quantity` contracts: the body of a 1/2/1 butterfly.
fn leg_of(
    side: Side,
    style: OptionStyle,
    strike: Decimal,
    premium: Decimal,
    quantity: Positive,
) -> Result<Position, Box<dyn Error>> {
    let mut position = leg(side, style, strike, premium)?;
    position.option.quantity = quantity;
    Ok(position)
}

fn leg_expiring(
    side: Side,
    style: OptionStyle,
    strike: Decimal,
    premium: Decimal,
    days: Decimal,
) -> Result<Position, Box<dyn Error>> {
    Ok(Position::new(
        option(side, style, strike, days)?,
        pos(premium)?,
        opened()?,
        pos(FEE)?,
        pos(FEE)?,
        None,
        None,
    ))
}

/// One golden entry: the chart data and the chart configuration.
fn entry<G: Graph>(graph: &G) -> Result<Value, Box<dyn Error>> {
    Ok(json!({
        "data": serde_json::to_value(graph.graph_data())?,
        "config": serde_json::to_value(graph.graph_config())?,
    }))
}

/// Builds the concrete strategy a request names and charts it.
///
/// `StrategyRequest::get_strategy` returns a `Box<dyn Strategable>`, which
/// carries no `Graph`; a consumer that renders names the concrete type, so
/// it dispatches on `strategy_type` and builds through `StrategyConstructor`.
fn chart_request(request: &StrategyRequest) -> Result<Value, Box<dyn Error>> {
    fn build<S: StrategyConstructor + Graph>(
        positions: &[Position],
    ) -> Result<Value, Box<dyn Error>> {
        entry(&S::get_strategy(positions)?)
    }
    let positions = request.positions.as_slice();
    match request.strategy_type {
        StrategyType::BullCallSpread => build::<BullCallSpread>(positions),
        StrategyType::BearCallSpread => build::<BearCallSpread>(positions),
        StrategyType::BullPutSpread => build::<BullPutSpread>(positions),
        StrategyType::BearPutSpread => build::<BearPutSpread>(positions),
        StrategyType::LongButterflySpread => build::<LongButterflySpread>(positions),
        StrategyType::ShortButterflySpread => build::<ShortButterflySpread>(positions),
        StrategyType::IronCondor => build::<IronCondor>(positions),
        StrategyType::IronButterfly => build::<IronButterfly>(positions),
        StrategyType::LongStraddle => build::<LongStraddle>(positions),
        StrategyType::ShortStraddle => build::<ShortStraddle>(positions),
        StrategyType::LongStrangle => build::<LongStrangle>(positions),
        StrategyType::ShortStrangle => build::<ShortStrangle>(positions),
        StrategyType::PoorMansCoveredCall => build::<PoorMansCoveredCall>(positions),
        StrategyType::BullCallLadder => build::<BullCallLadder>(positions),
        StrategyType::Custom => build::<CustomStrategy>(positions),
        _ => Err(Box::new(StrategyError::NotImplemented)),
    }
}

/// The positions of every strategy `StrategyRequest` can build.
fn requests() -> Result<Vec<(&'static str, StrategyRequest)>, Box<dyn Error>> {
    use OptionStyle::{Call, Put};
    use Side::{Long, Short};
    let cases = vec![
        (
            "bull_call_spread",
            StrategyType::BullCallSpread,
            vec![
                leg(Long, Call, dec!(95), dec!(7.5))?,
                leg(Short, Call, dec!(105), dec!(2.5))?,
            ],
        ),
        (
            "bear_call_spread",
            StrategyType::BearCallSpread,
            vec![
                leg(Short, Call, dec!(95), dec!(7.5))?,
                leg(Long, Call, dec!(105), dec!(2.5))?,
            ],
        ),
        // Textbook put verticals since #696: the bull put spread is long the
        // lower strike and short the higher, the bear put spread the reverse.
        (
            "bull_put_spread",
            StrategyType::BullPutSpread,
            vec![
                leg(Long, Put, dec!(95), dec!(2.5))?,
                leg(Short, Put, dec!(105), dec!(7.5))?,
            ],
        ),
        (
            "bear_put_spread",
            StrategyType::BearPutSpread,
            vec![
                leg(Short, Put, dec!(95), dec!(2.5))?,
                leg(Long, Put, dec!(105), dec!(7.5))?,
            ],
        ),
        // Textbook 1/2/1 butterflies since #706: the body carries twice the
        // wing quantity.
        (
            "long_butterfly_spread",
            StrategyType::LongButterflySpread,
            vec![
                leg(Long, Call, dec!(90), dec!(11.5))?,
                leg_of(Short, Call, dec!(100), dec!(4.5), Positive::TWO)?,
                leg(Long, Call, dec!(110), dec!(1.2))?,
            ],
        ),
        (
            "short_butterfly_spread",
            StrategyType::ShortButterflySpread,
            // A body premium of 2.5 leaves a 7.70 credit, above the 4.00 of
            // fees, so the chart has a profit zone in the wings.
            vec![
                leg(Short, Call, dec!(90), dec!(11.5))?,
                leg_of(Long, Call, dec!(100), dec!(2.5), Positive::TWO)?,
                leg(Short, Call, dec!(110), dec!(1.2))?,
            ],
        ),
        (
            "iron_condor",
            StrategyType::IronCondor,
            vec![
                leg(Long, Put, dec!(85), dec!(0.6))?,
                leg(Short, Put, dec!(95), dec!(2.1))?,
                leg(Short, Call, dec!(105), dec!(2.4))?,
                leg(Long, Call, dec!(115), dec!(0.7))?,
            ],
        ),
        (
            "iron_butterfly",
            StrategyType::IronButterfly,
            vec![
                leg(Long, Put, dec!(90), dec!(1.1))?,
                leg(Short, Put, dec!(100), dec!(4.1))?,
                leg(Short, Call, dec!(100), dec!(4.5))?,
                leg(Long, Call, dec!(110), dec!(1.2))?,
            ],
        ),
        (
            "long_straddle",
            StrategyType::LongStraddle,
            vec![
                leg(Long, Call, dec!(100), dec!(4.5))?,
                leg(Long, Put, dec!(100), dec!(4.1))?,
            ],
        ),
        (
            "short_straddle",
            StrategyType::ShortStraddle,
            vec![
                leg(Short, Call, dec!(100), dec!(4.5))?,
                leg(Short, Put, dec!(100), dec!(4.1))?,
            ],
        ),
        (
            "long_strangle",
            StrategyType::LongStrangle,
            vec![
                leg(Long, Put, dec!(95), dec!(2.1))?,
                leg(Long, Call, dec!(105), dec!(2.4))?,
            ],
        ),
        (
            "short_strangle",
            StrategyType::ShortStrangle,
            vec![
                leg(Short, Put, dec!(95), dec!(2.1))?,
                leg(Short, Call, dec!(105), dec!(2.4))?,
            ],
        ),
        (
            "poor_mans_covered_call",
            StrategyType::PoorMansCoveredCall,
            vec![
                leg_expiring(Long, Call, dec!(90), dec!(15.8), LEAPS_DAYS)?,
                leg(Short, Call, dec!(105), dec!(2.4))?,
            ],
        ),
        // The 1x1x1 call ladder that was `CallButterfly` before #706.
        (
            "bull_call_ladder",
            StrategyType::BullCallLadder,
            vec![
                leg(Long, Call, dec!(95), dec!(7.5))?,
                leg(Short, Call, dec!(100), dec!(4.5))?,
                leg(Short, Call, dec!(105), dec!(2.4))?,
            ],
        ),
        (
            "custom",
            StrategyType::Custom,
            vec![
                leg(Long, Call, dec!(100), dec!(4.5))?,
                leg(Short, Call, dec!(110), dec!(1.2))?,
                leg(Short, Put, dec!(90), dec!(1.1))?,
            ],
        ),
    ];
    Ok(cases
        .into_iter()
        .map(|(name, kind, positions)| (name, StrategyRequest::new(kind, positions)))
        .collect())
}

/// Every chart, keyed by a stable name.
fn collect() -> Result<Value, Box<dyn Error>> {
    let mut charts = serde_json::Map::new();

    // Core model.
    let call = option(Side::Long, OptionStyle::Call, dec!(100), EXPIRY_DAYS)?;
    charts.insert("options_long_call".into(), entry(&call)?);
    let put = option(Side::Short, OptionStyle::Put, dec!(95), EXPIRY_DAYS)?;
    charts.insert("options_short_put".into(), entry(&put)?);
    let position = leg(Side::Long, OptionStyle::Put, dec!(105), dec!(7.5))?;
    charts.insert("position_long_put".into(), entry(&position)?);

    // Math containers and the plot builder.
    let curve = Curve::from_vector(vec![
        Point2D::new(dec!(0), dec!(0)),
        Point2D::new(dec!(1), dec!(1.5)),
        Point2D::new(dec!(2), dec!(1.25)),
        Point2D::new(dec!(3), dec!(4)),
    ]);
    charts.insert("curve".into(), entry(&curve)?);
    let shifted = Curve::from_vector(vec![
        Point2D::new(dec!(0), dec!(1)),
        Point2D::new(dec!(1), dec!(2.5)),
        Point2D::new(dec!(2), dec!(2.25)),
    ]);
    charts.insert("curves".into(), entry(&vec![curve.clone(), shifted])?);
    let surface = Surface::from_vector(vec![
        Point3D::new(dec!(0), dec!(0), dec!(0)),
        Point3D::new(dec!(1), dec!(0), dec!(0.5)),
        Point3D::new(dec!(0), dec!(1), dec!(1.5)),
        Point3D::new(dec!(1), dec!(1), dec!(2.25)),
    ]);
    charts.insert("surface".into(), entry(&surface)?);
    let builder = curve
        .plot()
        .title("Golden Curve")
        .x_label("Strike")
        .y_label("Value")
        .line_style(LineStyle::Dashed)
        .color_scheme(ColorScheme::HighContrast)
        .show_legend(true);
    charts.insert("plot_builder_curve".into(), entry(&builder)?);

    // Simulation containers, on replayed prices.
    let params = WalkParams {
        size: 5,
        init_step: Step::new(
            Positive::ONE,
            TimeFrame::Day,
            expiry(EXPIRY_DAYS)?,
            Positive::HUNDRED,
        ),
        walker: Box::new(HistoricalWalker),
        walk_type: WalkType::Historical {
            timeframe: TimeFrame::Day,
            prices: vec![
                Positive::HUNDRED,
                pos(dec!(101.5))?,
                pos(dec!(99.25))?,
                pos(dec!(102))?,
                pos(dec!(100.75))?,
            ],
            symbol: Some(SYMBOL.to_string()),
        },
        seed: None,
    };
    let simulator = Simulator::new("Golden".to_string(), 2, &params, generator_positive)?;
    let walk = simulator.first().ok_or("simulator holds no walk")?;
    charts.insert("random_walk".into(), entry(walk)?);
    charts.insert("simulator".into(), entry(&simulator)?);

    // Strategies built from positions.
    for (name, request) in requests()? {
        charts.insert(format!("strategy_{name}"), chart_request(&request)?);
    }

    // Strategies without a builder.
    let hundred = Positive::HUNDRED;
    let iv = pos(IMPLIED_VOLATILITY)?;
    let fee = pos(FEE)?;
    let days = expiry(EXPIRY_DAYS)?;
    let one = Positive::ONE;
    let zero = Positive::ZERO;
    let symbol = || SYMBOL.to_string();
    charts.insert(
        "strategy_long_call".into(),
        entry(&LongCall::new(
            symbol(),
            hundred,
            days,
            iv,
            one,
            hundred,
            RISK_FREE_RATE,
            zero,
            pos(dec!(4.5))?,
            fee,
            fee,
        )?)?,
    );
    let mut long_put = LongPut::default();
    long_put.add_position(&leg(Side::Long, OptionStyle::Put, dec!(100), dec!(4.1))?)?;
    charts.insert("strategy_long_put".into(), entry(&long_put)?);
    let mut short_call = ShortCall::default();
    short_call.add_position(&leg(Side::Short, OptionStyle::Call, dec!(100), dec!(4.5))?)?;
    charts.insert("strategy_short_call".into(), entry(&short_call)?);
    charts.insert(
        "strategy_short_put".into(),
        entry(&ShortPut::new(
            symbol(),
            hundred,
            days,
            iv,
            one,
            hundred,
            RISK_FREE_RATE,
            zero,
            pos(dec!(4.1))?,
            fee,
            fee,
        )?)?,
    );
    charts.insert(
        "strategy_covered_call".into(),
        entry(&CoveredCall::new(
            symbol(),
            hundred,
            pos(dec!(105))?,
            days,
            iv,
            RISK_FREE_RATE,
            zero,
            one,
            pos(dec!(2.4))?,
            fee,
            fee,
            fee,
            fee,
        )?)?,
    );
    charts.insert(
        "strategy_collar".into(),
        entry(&Collar::new(
            symbol(),
            hundred,
            pos(dec!(95))?,
            pos(dec!(105))?,
            days,
            iv,
            RISK_FREE_RATE,
            zero,
            one,
            pos(dec!(2.1))?,
            pos(dec!(2.4))?,
            fee,
            fee,
            fee,
            fee,
            fee,
            fee,
        )?)?,
    );
    charts.insert(
        "strategy_protective_put".into(),
        entry(&ProtectivePut::new(
            symbol(),
            hundred,
            pos(dec!(95))?,
            days,
            iv,
            RISK_FREE_RATE,
            zero,
            one,
            pos(dec!(2.1))?,
            fee,
            fee,
            fee,
            fee,
        )?)?,
    );

    Ok(Value::Object(charts))
}

fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/graph_data.json")
}

#[test]
fn test_graph_data_matches_pre_extraction_golden() -> Result<(), Box<dyn Error>> {
    let actual = collect()?;
    if std::env::var("OSL_WRITE_GOLDEN").is_ok() {
        std::fs::write(golden_path(), serde_json::to_string_pretty(&actual)?)?;
        return Ok(());
    }
    let expected = std::fs::read_to_string(golden_path())
        .map_err(|e| format!("golden file missing ({e}); generate it with OSL_WRITE_GOLDEN=1"))?;
    let expected: Value = serde_json::from_str(&expected)?;
    let (actual, expected) = (
        actual.as_object().ok_or("charts are not an object")?,
        expected.as_object().ok_or("golden file is not an object")?,
    );
    let mut actual_names: Vec<&String> = actual.keys().collect();
    let mut expected_names: Vec<&String> = expected.keys().collect();
    actual_names.sort();
    expected_names.sort();
    assert_eq!(
        actual_names, expected_names,
        "the set of pinned charts changed"
    );
    for (name, exp) in expected {
        assert_eq!(
            actual.get(name),
            Some(exp),
            "chart {name} diverged from the golden output"
        );
    }
    Ok(())
}

#[test]
fn test_chart_request_without_builder_is_not_implemented() -> Result<(), Box<dyn Error>> {
    let request = StrategyRequest::new(
        StrategyType::LongCall,
        vec![leg(Side::Long, OptionStyle::Call, dec!(100), dec!(4.5))?],
    );
    let error = chart_request(&request)
        .err()
        .ok_or("a long call has no builder")?;
    assert!(matches!(
        error.downcast_ref::<StrategyError>(),
        Some(StrategyError::NotImplemented)
    ));
    Ok(())
}

#[test]
fn test_chart_request_with_invalid_positions_is_an_error() -> Result<(), Box<dyn Error>> {
    // An iron condor needs four legs.
    let request = StrategyRequest::new(
        StrategyType::IronCondor,
        vec![leg(Side::Long, OptionStyle::Put, dec!(85), dec!(0.6))?],
    );
    let error = chart_request(&request)
        .err()
        .ok_or("one leg is not an iron condor")?;
    assert!(error.downcast_ref::<StrategyError>().is_some());
    Ok(())
}

#[test]
fn test_single_leg_added_to_default_charts_like_new() -> Result<(), Box<dyn Error>> {
    // The four single-leg golden entries were regenerated in #780: two are
    // built by `new`, which reported no break-even, and two as `Default` +
    // `add_position`, which left it stale. The same leg now charts the same
    // way through either path. `LongPut::new` and `ShortCall::new` are
    // private, so their two strategies are compared in their own unit
    // tests instead.
    let hundred = Positive::HUNDRED;
    let iv = pos(IMPLIED_VOLATILITY)?;
    let fee = pos(FEE)?;
    let days = expiry(EXPIRY_DAYS)?;
    let one = Positive::ONE;
    let zero = Positive::ZERO;
    let symbol = || SYMBOL.to_string();
    for premium in [dec!(4.1), dec!(4.5)] {
        let premium_pos = pos(premium)?;
        let call = leg(Side::Long, OptionStyle::Call, dec!(100), premium)?;
        let mut added = LongCall::default();
        added.add_position(&call)?;
        let built = LongCall::new(
            symbol(),
            hundred,
            days,
            iv,
            one,
            hundred,
            RISK_FREE_RATE,
            zero,
            premium_pos,
            fee,
            fee,
        )?;
        assert_eq!(entry(&added)?, entry(&built)?, "LongCall at {premium}");

        let put = leg(Side::Short, OptionStyle::Put, dec!(100), premium)?;
        let mut added = ShortPut::default();
        added.add_position(&put)?;
        let built = ShortPut::new(
            symbol(),
            hundred,
            days,
            iv,
            one,
            hundred,
            RISK_FREE_RATE,
            zero,
            premium_pos,
            fee,
            fee,
        )?;
        assert_eq!(entry(&added)?, entry(&built)?, "ShortPut at {premium}");
    }
    Ok(())
}
