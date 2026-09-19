//! Regression pin for the single-leg strategy simulations
//! (`LongCall`, `LongPut`, `ShortCall`, `ShortPut`): deterministic historical
//! walks through every exit policy, serialised and compared with a golden
//! file generated on the code that preceded roadmap M1-08 (#505), which
//! moved the orchestration into `backtesting`. `PnL.date_time` is stamped
//! with `Utc::now()` by the P&L path, so it is normalised before the
//! comparison; every other field must match byte for byte.
//!
//! Regenerate with `OSL_WRITE_GOLDEN=1 cargo test --test tests single_leg_simulation_golden`
//! only when a numerical change is intended and reviewed.

use optionstratlib::ExpirationDate;
use optionstratlib::backtesting::results::SimulationStatsResult;
use optionstratlib::model::Options;
use optionstratlib::model::position::Position;
use optionstratlib::model::types::{OptionStyle, OptionType, Side};
use optionstratlib::simulation::steps::Step;
use optionstratlib::simulation::{
    ExitPolicy, Simulate, WalkParams, WalkType, WalkTypeAble, generator_positive,
    simulator::Simulator,
};
use optionstratlib::strategies::base::Positionable;
use optionstratlib::strategies::{LongCall, LongPut, ShortCall, ShortPut};
use optionstratlib::utils::TimeFrame;
use positive::{Positive, pos_or_panic};
use rust_decimal_macros::dec;
use serde_json::Value;
use std::path::PathBuf;

#[derive(Clone)]
struct HistoricalWalker;
impl WalkTypeAble<Positive, Positive> for HistoricalWalker {}

fn walk_params(prices: &[f64]) -> WalkParams<Positive, Positive> {
    let init_step = Step::new(
        Positive::ONE,
        TimeFrame::Day,
        ExpirationDate::Days(pos_or_panic!(30.0)),
        Positive::HUNDRED,
    );
    WalkParams {
        size: prices.len(),
        init_step,
        walker: Box::new(HistoricalWalker),
        walk_type: WalkType::Historical {
            timeframe: TimeFrame::Day,
            prices: prices.iter().map(|p| pos_or_panic!(*p)).collect(),
            symbol: Some("TEST".to_string()),
        },
    }
}

fn paths() -> Vec<Vec<f64>> {
    vec![
        vec![100.0, 105.0, 110.0, 115.0, 120.0],
        vec![100.0, 95.0, 90.0, 85.0, 80.0],
        vec![100.0, 101.0, 99.5, 100.5, 100.0, 100.2],
        vec![100.0, 108.0, 97.0, 112.0, 94.0, 103.0, 100.0],
        vec![100.0, 100.0, 100.0],
    ]
}

fn exit_policies() -> Vec<ExitPolicy> {
    vec![
        ExitPolicy::Expiration,
        ExitPolicy::ProfitPercent(dec!(0.25)),
        ExitPolicy::LossPercent(dec!(0.25)),
        ExitPolicy::FixedPrice(pos_or_panic!(110.0)),
        ExitPolicy::MinPrice(pos_or_panic!(90.0)),
        ExitPolicy::MaxPrice(pos_or_panic!(112.0)),
        ExitPolicy::TimeSteps(2),
    ]
}

fn strip_timestamps(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.remove("date_time");
            for v in map.values_mut() {
                strip_timestamps(v);
            }
        }
        Value::Array(items) => items.iter_mut().for_each(strip_timestamps),
        _ => {}
    }
}

fn run<S: Simulate<Positive, Positive>>(
    name: &str,
    strategy: &S,
    out: &mut serde_json::Map<String, Value>,
) {
    for (pi, prices) in paths().iter().enumerate() {
        for (ei, policy) in exit_policies().into_iter().enumerate() {
            let params = walk_params(prices);
            let simulator = Simulator::new("golden".to_string(), 2, &params, generator_positive)
                .unwrap_or_else(|e| panic!("simulator setup failed: {e}"));
            let key = format!("{name}/path{pi}/exit{ei}");
            let entry = match strategy.simulate(&simulator, policy) {
                Ok(stats) => {
                    let stats: SimulationStatsResult = stats;
                    let mut v = serde_json::to_value(&stats)
                        .unwrap_or_else(|e| panic!("serialisation failed: {e}"));
                    strip_timestamps(&mut v);
                    v
                }
                Err(e) => Value::String(format!("error: {e}")),
            };
            out.insert(key, entry);
        }
    }
}

fn collect() -> Value {
    let mut out = serde_json::Map::new();
    let long_call = LongCall::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.20),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        Positive::ZERO,
        pos_or_panic!(5.0),
        pos_or_panic!(0.5),
        pos_or_panic!(0.5),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let leg = |side: Side, style: OptionStyle| {
        let option = Options::new(
            OptionType::European,
            side,
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.20),
            Positive::ONE,
            Positive::HUNDRED,
            dec!(0.05),
            style,
            Positive::ZERO,
            None,
        );
        Position::new(
            option,
            pos_or_panic!(5.0),
            chrono::Utc::now(),
            pos_or_panic!(0.5),
            pos_or_panic!(0.5),
            None,
            None,
        )
    };
    let mut long_put = LongPut::default();
    long_put
        .add_position(&leg(Side::Long, OptionStyle::Put))
        .unwrap_or_else(|e| panic!("{e}"));
    let mut short_call = ShortCall::default();
    short_call
        .add_position(&leg(Side::Short, OptionStyle::Call))
        .unwrap_or_else(|e| panic!("{e}"));
    let mut short_put = ShortPut::default();
    short_put
        .add_position(&leg(Side::Short, OptionStyle::Put))
        .unwrap_or_else(|e| panic!("{e}"));
    run("long_call", &long_call, &mut out);
    run("long_put", &long_put, &mut out);
    run("short_call", &short_call, &mut out);
    run("short_put", &short_put, &mut out);
    Value::Object(out)
}

fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/unit/backtesting/golden/single_leg_simulation.json")
}

#[test]
fn single_leg_simulation_golden() {
    let actual = collect();
    let pretty = serde_json::to_string_pretty(&actual).unwrap_or_else(|e| panic!("{e}"));
    if std::env::var("OSL_WRITE_GOLDEN").is_ok() {
        std::fs::write(golden_path(), &pretty).unwrap_or_else(|e| panic!("{e}"));
        return;
    }
    let expected = std::fs::read_to_string(golden_path()).unwrap_or_else(|e| {
        panic!("golden file missing ({e}); generate it with OSL_WRITE_GOLDEN=1")
    });
    let expected: Value = serde_json::from_str(&expected).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        actual.as_object().map(|m| m.len()),
        expected.as_object().map(|m| m.len())
    );
    for (key, exp) in expected.as_object().into_iter().flatten() {
        let got = actual.get(key).unwrap_or(&Value::Null);
        assert_eq!(got, exp, "run {key} diverged from the golden output");
    }
}
