//! Seeded regression for the single-leg strategy simulations (#539).
//!
//! `single_leg_simulation_golden_test` pins the same orchestration on
//! replayed historical prices; this test drives it with stochastic walks.
//! The walks are seeded through `WalkParams::seed`, so the paths are fixed
//! and the simulation crate pins them bit for bit
//! (`optionstratlib-simulation/tests/deterministic_simulation_test.rs`).
//!
//! # Exact expectations
//!
//! * exit reason and holding period of every path;
//! * the profitable and loss counts, the win rate and the average holding
//!   period of the summary, which follow from them;
//! * the whole result of a second run in the same process, compared field
//!   by field after serialisation.
//!
//! # Tolerance-based expectations
//!
//! Compared within [`PNL_TOLERANCE`]:
//!
//! * the P&L of every path and the P&L statistics (mean, median, standard
//!   deviation, best, worst). Every mark is a Black-Scholes premium whose
//!   normal CDF is evaluated in `f64`, so the last digits follow the
//!   platform's floating-point library rather than the seeded stream.

use optionstratlib_backtest::backtesting::Simulate;
use optionstratlib_backtest::backtesting::results::SimulationStatsResult;
use optionstratlib_core::model::position::Position;
use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
use optionstratlib_core::model::{ExpirationDate, Options, Positive};
use optionstratlib_core::utils::TimeFrame;
use optionstratlib_simulation::simulation::simulator::Simulator;
use optionstratlib_simulation::simulation::steps::Step;
use optionstratlib_simulation::simulation::{
    ExitPolicy, WalkParams, WalkType, WalkTypeAble, generator_positive,
};
use optionstratlib_strategies::strategies::base::Positionable;
use optionstratlib_strategies::strategies::{LongCall, ShortPut};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::Value;

/// Seed of the simulated walks.
const SEED: u64 = 539;

/// Walks per simulation.
const WALKS: usize = 8;

/// Points per walk, the initial price included.
const STEPS: usize = 10;

/// Absolute tolerance on a P&L figure, in quote currency. The premia are
/// of order one to ten for one contract; an `f64` normal CDF is accurate to
/// about `1e-15` relative, so `1e-9` leaves several orders of magnitude for
/// platform differences while any change of path, exit or pricing still
/// fails.
const PNL_TOLERANCE: Decimal = dec!(0.000000001);

#[derive(Clone)]
struct Walker;

impl WalkTypeAble<Positive, Positive> for Walker {}

fn pos(value: Decimal) -> Positive {
    Positive::new_decimal(value).unwrap_or_else(|e| panic!("{value} is not positive: {e}"))
}

fn simulator() -> Simulator<Positive, Positive> {
    let params = WalkParams {
        size: STEPS,
        init_step: Step::new(
            Positive::ONE,
            TimeFrame::Day,
            ExpirationDate::Days(pos(dec!(30))),
            Positive::HUNDRED,
        ),
        walk_type: WalkType::GeometricBrownian {
            dt: pos(dec!(0.004)),
            drift: dec!(0.05),
            volatility: pos(dec!(0.4)),
        },
        walker: Box::new(Walker),
        seed: Some(SEED),
    };
    Simulator::new("regression".to_string(), WALKS, &params, generator_positive)
        .unwrap_or_else(|e| panic!("simulator failed: {e}"))
}

fn exit_policies() -> Vec<ExitPolicy> {
    vec![
        ExitPolicy::ProfitPercent(dec!(0.3)),
        ExitPolicy::LossPercent(dec!(0.3)),
        ExitPolicy::TimeSteps(4),
        ExitPolicy::Expiration,
    ]
}

fn long_call() -> LongCall {
    LongCall::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        ExpirationDate::Days(pos(dec!(30))),
        pos(dec!(0.4)),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        Positive::ZERO,
        pos(dec!(4.5)),
        pos(dec!(0.5)),
        pos(dec!(0.5)),
    )
    .unwrap_or_else(|e| panic!("{e}"))
}

fn short_put() -> ShortPut {
    let option = Options::new(
        OptionType::European,
        Side::Short,
        "TEST".to_string(),
        Positive::HUNDRED,
        ExpirationDate::Days(pos(dec!(30))),
        pos(dec!(0.4)),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        OptionStyle::Put,
        Positive::ZERO,
        None,
    );
    let position = Position::new(
        option,
        pos(dec!(4.0)),
        chrono::Utc::now(),
        pos(dec!(0.5)),
        pos(dec!(0.5)),
        None,
        None,
    );
    let mut strategy = ShortPut::default();
    strategy
        .add_position(&position)
        .unwrap_or_else(|e| panic!("{e}"));
    strategy
}

fn run<S: Simulate<Positive, Positive>>(strategy: &S) -> Vec<SimulationStatsResult> {
    let sim = simulator();
    exit_policies()
        .into_iter()
        .map(|policy| {
            strategy
                .simulate(&sim, policy)
                .unwrap_or_else(|e| panic!("simulation failed: {e}"))
        })
        .collect()
}

/// The expected result of one simulation.
struct Pinned {
    /// `(exit reason, holding period, P&L)` of every path, in walk order.
    paths: Vec<(ExitPolicy, usize, Decimal)>,
    profitable_count: usize,
    loss_count: usize,
    average_pnl: Decimal,
    median_pnl: Decimal,
    std_dev_pnl: Decimal,
    best_pnl: Decimal,
    worst_pnl: Decimal,
    win_rate: Decimal,
    average_holding_period: Decimal,
}

fn assert_close(actual: Decimal, expected: Decimal, what: &str) {
    assert!(
        (actual - expected).abs() <= PNL_TOLERANCE,
        "{what}: {actual} is not within {PNL_TOLERANCE} of {expected}"
    );
}

fn assert_matches(runs: &[SimulationStatsResult], pinned: &[Pinned]) {
    assert_eq!(runs.len(), pinned.len());
    for ((run, expected), policy) in runs.iter().zip(pinned).zip(exit_policies()) {
        let label = format!("{policy:?}");
        assert_eq!(run.total_simulations, WALKS, "{label}");
        assert_eq!(run.results.len(), expected.paths.len(), "{label}");
        for (path, (reason, holding_period, pnl)) in run.results.iter().zip(&expected.paths) {
            assert_eq!(&path.exit_reason, reason, "{label}: exit reason");
            assert_eq!(
                path.holding_period, *holding_period,
                "{label}: holding period"
            );
            let total = path
                .pnl
                .total_pnl()
                .unwrap_or_else(|| panic!("{label}: path without P&L"));
            assert_close(total, *pnl, &format!("{label}: path P&L"));
        }
        assert_eq!(run.profitable_count, expected.profitable_count, "{label}");
        assert_eq!(run.loss_count, expected.loss_count, "{label}");
        assert_eq!(run.win_rate, expected.win_rate, "{label}: win rate");
        assert_eq!(
            run.average_holding_period, expected.average_holding_period,
            "{label}: average holding period"
        );
        assert_close(
            run.average_pnl,
            expected.average_pnl,
            &format!("{label}: mean"),
        );
        assert_close(
            run.median_pnl,
            expected.median_pnl,
            &format!("{label}: median"),
        );
        assert_close(
            run.std_dev_pnl,
            expected.std_dev_pnl,
            &format!("{label}: std dev"),
        );
        assert_close(run.best_pnl, expected.best_pnl, &format!("{label}: best"));
        assert_close(
            run.worst_pnl,
            expected.worst_pnl,
            &format!("{label}: worst"),
        );
    }
}

/// Serialised runs without the `date_time` stamps the P&L path takes from
/// the clock.
fn comparable(runs: &[SimulationStatsResult]) -> Value {
    fn strip(value: &mut Value) {
        match value {
            Value::Object(map) => {
                map.remove("date_time");
                map.values_mut().for_each(strip);
            }
            Value::Array(items) => items.iter_mut().for_each(strip),
            _ => {}
        }
    }
    let mut value = serde_json::to_value(runs).unwrap_or_else(|e| panic!("{e}"));
    strip(&mut value);
    value
}

#[test]
fn test_simulation_regression_long_call_matches_pinned() {
    assert_matches(&run(&long_call()), &pinned_long_call());
}

#[test]
fn test_simulation_regression_short_put_matches_pinned() {
    assert_matches(&run(&short_put()), &pinned_short_put());
}

#[test]
fn test_simulation_regression_repeated_runs_are_identical() {
    assert_eq!(
        comparable(&run(&long_call())),
        comparable(&run(&long_call()))
    );
    assert_eq!(
        comparable(&run(&short_put())),
        comparable(&run(&short_put()))
    );
}

/// Pinned runs of the long call, one per policy of [`exit_policies`].
fn pinned_long_call() -> Vec<Pinned> {
    vec![
        Pinned {
            paths: vec![
                (ExitPolicy::Expiration, 9, dec!(-5.5)),
                (
                    ExitPolicy::ProfitPercent(dec!(0.3)),
                    1,
                    dec!(3.207582080201830932281215528),
                ),
                (
                    ExitPolicy::ProfitPercent(dec!(0.3)),
                    2,
                    dec!(1.822337570985544491744100059),
                ),
                (ExitPolicy::Expiration, 9, dec!(-5.5)),
                (
                    ExitPolicy::ProfitPercent(dec!(0.3)),
                    1,
                    dec!(1.704620703477682257744094981),
                ),
                (
                    ExitPolicy::ProfitPercent(dec!(0.3)),
                    3,
                    dec!(1.672362054598565730825308895),
                ),
                (ExitPolicy::Expiration, 9, dec!(-5.5)),
                (
                    ExitPolicy::ProfitPercent(dec!(0.3)),
                    3,
                    dec!(2.399645690765703997617429602),
                ),
            ],
            profitable_count: 5,
            loss_count: 3,
            average_pnl: dec!(-0.7116814874963340737234813669),
            median_pnl: dec!(1.688491379038123994284701938),
            std_dev_pnl: dec!(3.995862122823232656635165660),
            best_pnl: dec!(3.207582080201830932281215528),
            worst_pnl: dec!(-5.5),
            win_rate: dec!(62.5000),
            average_holding_period: dec!(4.625),
        },
        Pinned {
            paths: vec![
                (
                    ExitPolicy::LossPercent(dec!(0.3)),
                    5,
                    dec!(-1.873116644925064794992547813),
                ),
                (
                    ExitPolicy::LossPercent(dec!(0.3)),
                    8,
                    dec!(-1.935466038495070270253816082),
                ),
                (ExitPolicy::Expiration, 9, dec!(-4.95686432820606)),
                (
                    ExitPolicy::LossPercent(dec!(0.3)),
                    1,
                    dec!(-1.704863938451404892287660820),
                ),
                (
                    ExitPolicy::LossPercent(dec!(0.3)),
                    3,
                    dec!(-1.649922177454245957916141368),
                ),
                (
                    ExitPolicy::LossPercent(dec!(0.3)),
                    7,
                    dec!(-1.472824950650201181930285963),
                ),
                (
                    ExitPolicy::LossPercent(dec!(0.3)),
                    6,
                    dec!(-1.690892606608094798956075283),
                ),
                (ExitPolicy::Expiration, 9, dec!(6.41328278407697)),
            ],
            profitable_count: 1,
            loss_count: 7,
            average_pnl: dec!(-1.1088334875891464870420659161),
            median_pnl: dec!(-1.6978782725297498456218680515),
            std_dev_pnl: dec!(3.2464560137716129843293442998),
            best_pnl: dec!(6.41328278407697),
            worst_pnl: dec!(-4.95686432820606),
            win_rate: dec!(12.5000),
            average_holding_period: dec!(6),
        },
        Pinned {
            paths: vec![
                (
                    ExitPolicy::TimeSteps(4),
                    4,
                    dec!(-0.298864335412128693339829591),
                ),
                (
                    ExitPolicy::TimeSteps(4),
                    4,
                    dec!(-0.582262748804528938650865281),
                ),
                (
                    ExitPolicy::TimeSteps(4),
                    4,
                    dec!(2.226906919199552958482801188),
                ),
                (
                    ExitPolicy::TimeSteps(4),
                    4,
                    dec!(-1.003395957968561270266149085),
                ),
                (
                    ExitPolicy::TimeSteps(4),
                    4,
                    dec!(-0.149167280048989778872177085),
                ),
                (
                    ExitPolicy::TimeSteps(4),
                    4,
                    dec!(2.610417252482343645917507100),
                ),
                (
                    ExitPolicy::TimeSteps(4),
                    4,
                    dec!(0.135564678012620818722870688),
                ),
                (
                    ExitPolicy::TimeSteps(4),
                    4,
                    dec!(1.132084857054452990944997302),
                ),
            ],
            profitable_count: 4,
            loss_count: 4,
            average_pnl: dec!(0.5089104230643452166173944045),
            median_pnl: dec!(-0.0068013010181844800746531985),
            std_dev_pnl: dec!(1.3341060217151479746370722691),
            best_pnl: dec!(2.610417252482343645917507100),
            worst_pnl: dec!(-1.003395957968561270266149085),
            win_rate: dec!(50.000),
            average_holding_period: dec!(4),
        },
        Pinned {
            paths: vec![
                (ExitPolicy::Expiration, 9, dec!(-5.5)),
                (ExitPolicy::Expiration, 9, dec!(-5.5)),
                (ExitPolicy::Expiration, 9, dec!(-4.95686432820606)),
                (ExitPolicy::Expiration, 9, dec!(-5.5)),
                (ExitPolicy::Expiration, 9, dec!(2.26699861897471)),
                (ExitPolicy::Expiration, 9, dec!(-5.5)),
                (ExitPolicy::Expiration, 9, dec!(-5.5)),
                (ExitPolicy::Expiration, 9, dec!(6.41328278407697)),
            ],
            profitable_count: 2,
            loss_count: 6,
            average_pnl: dec!(-2.9720728656442975),
            median_pnl: dec!(-5.50),
            std_dev_pnl: dec!(4.651025168491967713593143179),
            best_pnl: dec!(6.41328278407697),
            worst_pnl: dec!(-5.5),
            win_rate: dec!(25.000),
            average_holding_period: dec!(9),
        },
    ]
}

/// Pinned runs of the short put, one per policy of [`exit_policies`].
fn pinned_short_put() -> Vec<Pinned> {
    vec![
        Pinned {
            paths: vec![
                (ExitPolicy::Expiration, 9, dec!(-0.912739912121253)),
                (ExitPolicy::Expiration, 9, dec!(1.541515494349878)),
                (
                    ExitPolicy::ProfitPercent(dec!(0.3)),
                    3,
                    dec!(1.635533698094377516757398728),
                ),
                (ExitPolicy::Expiration, 9, dec!(-4.28126306205059)),
                (
                    ExitPolicy::ProfitPercent(dec!(0.3)),
                    6,
                    dec!(1.682067642661889267345688736),
                ),
                (
                    ExitPolicy::ProfitPercent(dec!(0.3)),
                    5,
                    dec!(1.403863450529006403679550832),
                ),
                (ExitPolicy::Expiration, 9, dec!(-0.504973265378803)),
                (
                    ExitPolicy::ProfitPercent(dec!(0.3)),
                    6,
                    dec!(1.650917212619922063794621052),
                ),
            ],
            profitable_count: 5,
            loss_count: 3,
            average_pnl: dec!(0.2768651573380534064471574185),
            median_pnl: dec!(1.472689472439442201839775416),
            std_dev_pnl: dec!(2.1173104000693130872899867501),
            best_pnl: dec!(1.682067642661889267345688736),
            worst_pnl: dec!(-4.28126306205059),
            win_rate: dec!(62.5000),
            average_holding_period: dec!(7),
        },
        Pinned {
            paths: vec![
                (
                    ExitPolicy::LossPercent(dec!(0.3)),
                    1,
                    dec!(-1.782921816137214845099264432),
                ),
                (
                    ExitPolicy::LossPercent(dec!(0.3)),
                    8,
                    dec!(-1.732805850441040126304509125),
                ),
                (ExitPolicy::Expiration, 9, dec!(3.0)),
                (
                    ExitPolicy::LossPercent(dec!(0.3)),
                    1,
                    dec!(-2.718017733184435806098167439),
                ),
                (
                    ExitPolicy::LossPercent(dec!(0.3)),
                    3,
                    dec!(-2.289274345312455327755746286),
                ),
                (
                    ExitPolicy::LossPercent(dec!(0.3)),
                    7,
                    dec!(-1.346981995512277619738658914),
                ),
                (
                    ExitPolicy::LossPercent(dec!(0.3)),
                    3,
                    dec!(-1.572380791738175402064074891),
                ),
                (ExitPolicy::Expiration, 9, dec!(3.0)),
            ],
            profitable_count: 2,
            loss_count: 6,
            average_pnl: dec!(-0.6802978165406998908825526359),
            median_pnl: dec!(-1.652593321089607764184292008),
            std_dev_pnl: dec!(2.3112805554105594551938297316),
            best_pnl: dec!(3.0),
            worst_pnl: dec!(-2.718017733184435806098167439),
            win_rate: dec!(25.000),
            average_holding_period: dec!(5.125),
        },
        Pinned {
            paths: vec![
                (
                    ExitPolicy::TimeSteps(4),
                    4,
                    dec!(-0.676245974691880823325380579),
                ),
                (
                    ExitPolicy::TimeSteps(4),
                    4,
                    dec!(-0.930095379520480737893167279),
                ),
                (
                    ExitPolicy::TimeSteps(4),
                    4,
                    dec!(0.928549612361976648392093717),
                ),
                (
                    ExitPolicy::TimeSteps(4),
                    4,
                    dec!(-1.348447395593028199279580327),
                ),
                (
                    ExitPolicy::TimeSteps(4),
                    4,
                    dec!(-0.550073046278885001116665595),
                ),
                (
                    ExitPolicy::TimeSteps(4),
                    4,
                    dec!(1.100276110856968061788147636),
                ),
                (
                    ExitPolicy::TimeSteps(4),
                    4,
                    dec!(-0.323759858168298794672034806),
                ),
                (
                    ExitPolicy::TimeSteps(4),
                    4,
                    dec!(0.350174176428433977381054926),
                ),
            ],
            profitable_count: 3,
            loss_count: 5,
            average_pnl: dec!(-0.1812027193256493585906915384),
            median_pnl: dec!(-0.4369164522235918978943502005),
            std_dev_pnl: dec!(0.8848612212921940704821047304),
            best_pnl: dec!(1.100276110856968061788147636),
            worst_pnl: dec!(-1.348447395593028199279580327),
            win_rate: dec!(37.5000),
            average_holding_period: dec!(4),
        },
        Pinned {
            paths: vec![
                (ExitPolicy::Expiration, 9, dec!(-0.912739912121253)),
                (ExitPolicy::Expiration, 9, dec!(1.541515494349878)),
                (ExitPolicy::Expiration, 9, dec!(3.0)),
                (ExitPolicy::Expiration, 9, dec!(-4.28126306205059)),
                (ExitPolicy::Expiration, 9, dec!(3.0)),
                (ExitPolicy::Expiration, 9, dec!(1.460091540272506)),
                (ExitPolicy::Expiration, 9, dec!(-0.504973265378803)),
                (ExitPolicy::Expiration, 9, dec!(3.0)),
            ],
            profitable_count: 5,
            loss_count: 3,
            average_pnl: dec!(0.78782884938396725),
            median_pnl: dec!(1.5008035173111920),
            std_dev_pnl: dec!(2.5623663312963552077830803072),
            best_pnl: dec!(3.0),
            worst_pnl: dec!(-4.28126306205059),
            win_rate: dec!(62.5000),
            average_holding_period: dec!(9),
        },
    ]
}
