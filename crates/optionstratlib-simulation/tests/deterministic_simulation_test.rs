//! Fixed-seed regressions for the stochastic walks (#539).
//!
//! A walk is reproducible when [`WalkParams::seed`] is set: every built-in
//! kernel then draws from `deterministic_rng(seed)` instead of the thread
//! RNG. These tests pin what that promises, before the simulation and
//! backtest crates move further apart, so an extraction that reorders the
//! random stream or a reduction fails here instead of drifting silently.
//!
//! # Exact expectations
//!
//! Compared bit for bit, with `assert_eq!` on `Decimal`:
//!
//! * the price path of every stochastic [`WalkType`], and the volatility path
//!   of the stochastic-volatility ones (`Garch`, `Heston`, `Custom`,
//!   `Telegraph`);
//! * the per-walk seeds a seeded [`Simulator`] derives, through the terminal
//!   price of each walk;
//! * exit reasons, holding periods, P&L and [`PathStatistics`] of a
//!   `Decimal`-only evaluator over a seeded simulator;
//! * repeated runs, and [`walk_steps`] against [`walk_steps_par`];
//! * the Monte-Carlo price over a seeded simulator. Its per-path payoff
//!   passes through `f64` (`Options::payoff_at_price`: a `Decimal`
//!   difference converted to `f64`, one IEEE-754 multiply by the quantity,
//!   converted back), every step correctly rounded, so no transcendental or
//!   platform-dependent function is involved.
//!
//! These are exact because the only `f64` operations on the way are the
//! standard normal draw itself, converted once with `Decimal::from_f64`,
//! and the correctly rounded IEEE-754 `sqrt(dt)` of three kernels; every
//! later operation is `Decimal` arithmetic. The promise holds for a given
//! version of `rand` and `rand_distr`: a release that changes the `StdRng`
//! or `StandardNormal` stream changes these values, and the dependency bump
//! that brings it in is a reviewed change of these fixtures.
//!
//! An unseeded walk keeps drawing from the thread RNG and is reproducible
//! only in distribution; it is checked here only to differ run to run.

use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
use optionstratlib_core::model::{ExpirationDate, Options, Positive};
use optionstratlib_core::utils::{Len, TimeFrame};
use optionstratlib_simulation::error::SimulationError;
use optionstratlib_simulation::simulation::randomwalk::RandomWalk;
use optionstratlib_simulation::simulation::simulator::Simulator;
use optionstratlib_simulation::simulation::steps::{Step, Xstep};
use optionstratlib_simulation::simulation::{
    ExitPolicy, PathEvaluator, PathOutcome, PathStatistics, WalkParams, WalkPath, WalkType,
    WalkTypeAble, check_exit_policy, evaluate_paths, generator_positive, walk_steps,
    walk_steps_par,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Seed of every pinned fixture.
const SEED: u64 = 42;

/// Points per pinned path, the initial value included.
const SIZE: usize = 8;

/// Walks in the seeded simulator fixtures.
const SIMULATOR_WALKS: usize = 6;

#[derive(Clone)]
struct Walker;

impl WalkTypeAble<Positive, Positive> for Walker {}

fn pos(value: Decimal) -> Positive {
    Positive::new_decimal(value).unwrap_or_else(|e| panic!("{value} is not positive: {e}"))
}

fn walk_params(walk_type: WalkType, seed: Option<u64>) -> WalkParams<Positive, Positive> {
    WalkParams {
        size: SIZE,
        init_step: Step::new(
            Positive::ONE,
            TimeFrame::Day,
            ExpirationDate::Days(pos(dec!(30))),
            Positive::HUNDRED,
        ),
        walk_type,
        walker: Box::new(Walker),
        seed,
    }
}

fn dt() -> Positive {
    pos(dec!(0.004))
}

fn brownian() -> WalkType {
    WalkType::Brownian {
        dt: dt(),
        drift: dec!(0.05),
        volatility: pos(dec!(0.2)),
    }
}

fn geometric_brownian() -> WalkType {
    WalkType::GeometricBrownian {
        dt: dt(),
        drift: dec!(0.05),
        volatility: pos(dec!(0.2)),
    }
}

fn log_returns() -> WalkType {
    WalkType::LogReturns {
        dt: dt(),
        expected_return: dec!(0.05),
        volatility: pos(dec!(0.2)),
        autocorrelation: Some(dec!(0.1)),
    }
}

fn mean_reverting() -> WalkType {
    WalkType::MeanReverting {
        dt: dt(),
        volatility: pos(dec!(0.2)),
        speed: pos(dec!(2)),
        mean: pos(dec!(105)),
    }
}

fn jump_diffusion() -> WalkType {
    WalkType::JumpDiffusion {
        dt: dt(),
        drift: dec!(0.05),
        volatility: pos(dec!(0.2)),
        // `λ·dt = 0.4`, so the jump branch fires within the short pinned
        // path; a realistic one jump a year would almost never reach it.
        intensity: pos(dec!(100)),
        jump_mean: dec!(-1),
        jump_volatility: pos(dec!(2)),
    }
}

fn garch() -> WalkType {
    WalkType::Garch {
        dt: dt(),
        drift: dec!(0.05),
        volatility: pos(dec!(0.2)),
        alpha: pos(dec!(0.1)),
        beta: pos(dec!(0.8)),
    }
}

fn heston() -> WalkType {
    WalkType::Heston {
        dt: dt(),
        drift: dec!(0.05),
        volatility: pos(dec!(0.2)),
        kappa: pos(dec!(2)),
        theta: pos(dec!(0.04)),
        xi: pos(dec!(0.3)),
        rho: dec!(-0.7),
    }
}

fn custom() -> WalkType {
    WalkType::Custom {
        dt: dt(),
        drift: dec!(0.05),
        volatility: pos(dec!(0.2)),
        vov: pos(dec!(0.1)),
        vol_speed: Positive::ONE,
        vol_mean: pos(dec!(0.2)),
    }
}

fn telegraph() -> WalkType {
    WalkType::Telegraph {
        dt: dt(),
        drift: dec!(0.05),
        volatility: pos(dec!(0.2)),
        // `λ·dt = 1`, a switch probability of `1 - e^(-1) = 0.632` per
        // step, so the short pinned path switches in both directions. Since
        // #683 every positive rate switches at its own frequency, but a
        // realistic one (a few a year) would almost never switch within these
        // seven steps; the kernel's unit tests cover realistic rates.
        lambda_up: pos(dec!(250)),
        lambda_down: pos(dec!(250)),
        vol_multiplier_up: Some(pos(dec!(1.5))),
        vol_multiplier_down: Some(pos(dec!(0.5))),
    }
}

fn stochastic_walk_types() -> Vec<(&'static str, WalkType)> {
    vec![
        ("brownian", brownian()),
        ("geometric_brownian", geometric_brownian()),
        ("log_returns", log_returns()),
        ("mean_reverting", mean_reverting()),
        ("jump_diffusion", jump_diffusion()),
        ("garch", garch()),
        ("heston", heston()),
        ("custom", custom()),
        ("telegraph", telegraph()),
    ]
}

fn generate(walk_type: WalkType, seed: Option<u64>) -> WalkPath {
    let params = walk_params(walk_type, seed);
    Walker
        .generate_with_vol(&params)
        .unwrap_or_else(|e| panic!("walk failed: {e}"))
}

fn decimals(values: &[Positive]) -> Vec<Decimal> {
    values.iter().map(Positive::to_dec).collect()
}

/// The 128-bit representation (sign, scale and mantissa) of each value:
/// equal bits means bit-for-bit equal, not merely numerically equal.
fn bits(values: &[Decimal]) -> Vec<[u8; 16]> {
    values.iter().map(Decimal::serialize).collect()
}

fn assert_bit_identical(actual: &[Decimal], expected: &[Decimal], what: &str) {
    assert_eq!(actual, expected, "{what} drifted");
    assert_eq!(
        bits(actual),
        bits(expected),
        "{what} changed representation"
    );
}

fn assert_pinned(walk_type: WalkType, prices: &[Decimal], vols: Option<&[Decimal]>) {
    let path = generate(walk_type, Some(SEED));
    assert_bit_identical(&decimals(&path.prices), prices, "price path");
    match (path.vols.as_deref(), vols) {
        (Some(actual), Some(expected)) => {
            assert_bit_identical(&decimals(actual), expected, "volatility path");
        }
        (None, None) => {}
        (actual, expected) => panic!("volatility path {actual:?}, expected {expected:?}"),
    }
}

/// Closes a position opened at `entry` at `price` after `holding_period`
/// steps; `marks` holds the marks observed before the exit step.
fn close(
    entry: Decimal,
    marks: &mut Vec<Decimal>,
    price: Decimal,
    holding_period: usize,
    reason: ExitPolicy,
) -> Result<PathOutcome, SimulationError> {
    marks.push(price);
    let sum = marks
        .iter()
        .try_fold(Decimal::ZERO, |acc, m| acc.checked_add(*m))
        .ok_or_else(|| SimulationError::walk_error("mark sum overflowed"))?;
    let expired = reason == ExitPolicy::Expiration;
    Ok(PathOutcome {
        pnl: Some(price - entry),
        holding_period,
        hit_take_profit: matches!(reason, ExitPolicy::ProfitPercent(_)),
        hit_stop_loss: matches!(reason, ExitPolicy::LossPercent(_)),
        expired,
        max_premium: marks.iter().copied().max().unwrap_or(Decimal::ZERO),
        min_premium: marks.iter().copied().min().unwrap_or(Decimal::ZERO),
        avg_premium: sum / Decimal::from(marks.len()),
        expiration_premium: expired.then_some(price),
        exit_reason: reason,
    })
}

/// Long-the-underlying evaluator in `Decimal` only: the premium mark is
/// the underlying price, the P&L the move since the first step.
struct UnderlyingEvaluator;

impl PathEvaluator<Positive, Positive> for UnderlyingEvaluator {
    type Outcome = PathOutcome;
    type Error = SimulationError;

    fn evaluate_path(
        &self,
        walk: &RandomWalk<Positive, Positive>,
        exit: &ExitPolicy,
    ) -> Result<PathOutcome, SimulationError> {
        let steps = walk.get_steps();
        let entry = steps
            .first()
            .ok_or_else(|| SimulationError::walk_error("empty walk"))?
            .y
            .value()
            .to_dec();
        let mut marks: Vec<Decimal> = Vec::with_capacity(steps.len());
        let last = steps.len() - 1;
        for (i, step) in steps.iter().enumerate().skip(1) {
            let price = *step.y.value();
            if i == last {
                return close(entry, &mut marks, price.to_dec(), i, ExitPolicy::Expiration);
            }
            if let Some(reason) = check_exit_policy(
                exit,
                entry,
                price.to_dec(),
                i,
                step.x.days_left()?,
                price,
                true,
            ) {
                return close(entry, &mut marks, price.to_dec(), i, reason);
            }
            marks.push(price.to_dec());
        }
        close(entry, &mut marks, entry, 0, ExitPolicy::Expiration)
    }
}

fn exit_policies() -> Vec<ExitPolicy> {
    vec![
        ExitPolicy::ProfitPercent(dec!(0.02)),
        ExitPolicy::LossPercent(dec!(0.02)),
        ExitPolicy::TimeSteps(3),
        ExitPolicy::Or(vec![
            ExitPolicy::ProfitPercent(dec!(0.03)),
            ExitPolicy::LossPercent(dec!(0.01)),
        ]),
        ExitPolicy::Expiration,
    ]
}

/// Six walks of a volatile GBM, so the exit policies above all fire.
fn seeded_simulator() -> Simulator<Positive, Positive> {
    let params = walk_params(
        WalkType::GeometricBrownian {
            dt: dt(),
            drift: dec!(0.05),
            volatility: pos(dec!(0.5)),
        },
        Some(SEED),
    );
    Simulator::new(
        "seeded".to_string(),
        SIMULATOR_WALKS,
        &params,
        generator_positive,
    )
    .unwrap_or_else(|e| panic!("simulator failed: {e}"))
}

fn terminal_prices(sim: &Simulator<Positive, Positive>) -> Vec<Decimal> {
    decimals(&sim.get_last_positive_values())
}

#[test]
fn test_deterministic_brownian_seeded_path_matches_pinned() {
    assert_pinned(
        brownian(),
        &[
            dec!(100),
            dec!(100.08802014209121237058057256),
            dec!(100.25637504343791698043654649),
            dec!(100.58871078304273404381213382),
            dec!(100.30392520922187578627968135),
            dec!(99.46393985269607763351479046),
            dec!(99.19169039907966188683644823),
            dec!(99.43718589046482629800832658),
        ],
        None,
    );
}

#[test]
fn test_deterministic_geometric_brownian_seeded_path_matches_pinned() {
    assert_pinned(
        geometric_brownian(),
        &[
            dec!(100),
            dec!(100.10787828890252890383922254),
            dec!(100.29641348201915847536415810),
            dec!(100.65021556562893038434140288),
            dec!(100.38386007203343722218129335),
            dec!(99.56389357061166548445427736),
            dec!(99.31286206244896362677039762),
            dec!(99.57668440752406373512957141),
        ],
        None,
    );
}

#[test]
fn test_deterministic_log_returns_seeded_path_matches_pinned() {
    assert_pinned(
        log_returns(),
        &[
            dec!(100),
            dec!(100.10787828890252892241040573),
            dec!(100.30722803857506431233737127),
            dec!(100.68109546567887323839521717),
            dec!(100.45202241116493798049629410),
            dec!(99.60880739813317086770256249),
            dec!(99.27394304456446563983572676),
            dec!(99.50414875402758469652348811),
        ],
        None,
    );
}

#[test]
fn test_deterministic_mean_reverting_seeded_path_matches_pinned() {
    assert_pinned(
        mean_reverting(),
        &[
            dec!(100),
            dec!(100.13221114919577296963087215),
            dec!(100.34771610641624658892541652),
            dec!(100.73367690414997445908983504),
            dec!(100.46857263640487355621856041),
            dec!(99.62262943096154669372090792),
            dec!(99.37957646921661784658885737),
            dec!(99.68210012341730748113946004),
        ],
        None,
    );
}

#[test]
fn test_deterministic_jump_diffusion_seeded_path_matches_pinned() {
    // Re-baselined by #684: the jump trial now draws a genuine U(0,1), so
    // `P(jump) = λ·dt = 0.4` per step instead of `Φ(0.4)`.
    assert_pinned(
        jump_diffusion(),
        &[
            dec!(100),
            dec!(100.08802014209121235202940205),
            dec!(100.42064822804596520422076232),
            dec!(99.57712864670563978447619335),
            dec!(99.82158685372681364852693795),
            dec!(100.82133776726378230222513836),
            dec!(98.59969284016619166847340400),
            dec!(95.63425965993297671711051973),
        ],
        None,
    );
}

#[test]
fn test_deterministic_garch_seeded_path_matches_pinned() {
    assert_pinned(
        garch(),
        &[
            dec!(100),
            dec!(100.10336688881653756421115303),
            dec!(100.27596448549233680499900830),
            dec!(100.58608280702028345797291000),
            dec!(100.36578273056745709840065258),
            dec!(99.70099235586621448566599385),
            dec!(99.50531313351979873924938168),
            dec!(99.71528447302393936213092865),
        ],
        Some(&[
            dec!(0.2),
            dec!(0.1897366596101027599199336126),
            dec!(0.1811078943928072183653094155),
            dec!(0.1738973472868076414066665264),
            dec!(0.1679079030230773561055186269),
            dec!(0.1629571218312768785693045702),
            dec!(0.1588984116286625383194788556),
            dec!(0.155561668540681655548791907),
        ]),
    );
}

#[test]
fn test_deterministic_heston_seeded_path_matches_pinned() {
    assert_pinned(
        heston(),
        &[
            dec!(100),
            dec!(100.10797485303018063429261440),
            dec!(100.45910281820872985786868901),
            dec!(99.64429767193199361036681509),
            dec!(99.91439416786448225262199508),
            dec!(98.25346385711291601466096016),
            dec!(100.50930031058966009549883617),
            dec!(100.04233843138424897194720108),
        ],
        Some(&[
            dec!(0.2),
            dec!(0.2004391118506348460677873079),
            dec!(0.1971383085276330899405957948),
            dec!(0.2000909707167163267674786612),
            dec!(0.2086231865800161913652446986),
            dec!(0.2007740089299971712383637507),
            dec!(0.1885058693082086213581864264),
            dec!(0.187015857657258667286399211),
        ]),
    );
}

#[test]
fn test_deterministic_custom_seeded_path_matches_pinned() {
    assert_pinned(
        custom(),
        &[
            dec!(100),
            dec!(101.86773652401673464929152572),
            dec!(100.17619648186479043629716228),
            dec!(97.10191540376172634038473540),
            dec!(99.37892630504103372788713629),
            dec!(99.42483591561806043608290873),
            dec!(98.91758443621268875161836152),
            dec!(98.14060292202166221200987639),
        ],
        Some(&[
            dec!(0.2),
            dec!(0.2),
            dec!(0.2004391007104560617601470102),
            dec!(0.2012798752171895846318212547),
            dec!(0.2029405539152136695978965968),
            dec!(0.2015156260461093786112368066),
            dec!(0.1973146992634803887348182161),
            dec!(0.1959631931983443887342490228),
        ]),
    );
}

#[test]
fn test_deterministic_telegraph_seeded_path_matches_pinned() {
    // Re-baselined by #683: the regime switch now draws a genuine U(0,1),
    // so `P(switch) = 1 - e^(-λ·dt) = 1 - e^(-1)` per step. The first four
    // steps keep their values: the old and the new trial each consumed one
    // word of the stream there and took the same decisions.
    assert_pinned(
        telegraph(),
        &[
            dec!(100),
            dec!(100.51954860658225381004771531),
            dec!(100.11818070957388553378674917),
            dec!(100.50733678499377858361144734),
            dec!(98.05950298691601264031720460),
            dec!(99.21888939964810895011799167),
            dec!(98.05223532103304649453577772),
            dec!(97.57486799479786199841818131),
        ],
        Some(&[
            dec!(0.2),
            dec!(0.30),
            dec!(0.10),
            dec!(0.30),
            dec!(0.30),
            dec!(0.10),
            dec!(0.30),
            dec!(0.10),
        ]),
    );
}

/// `(exit reason, holding period, P&L)` of one walk.
type PinnedPath = (ExitPolicy, usize, Decimal);

/// Pinned outcomes of every walk, and the summary, for each policy of
/// [`exit_policies`], in the same order.
fn pinned_outcomes() -> Vec<(Vec<PinnedPath>, PathStatistics)> {
    vec![
        (
            vec![
                (
                    ExitPolicy::ProfitPercent(dec!(0.02)),
                    4,
                    dec!(3.13107420108338488440950979),
                ),
                (
                    ExitPolicy::Expiration,
                    7,
                    dec!(-9.50449297052891095084039500),
                ),
                (
                    ExitPolicy::ProfitPercent(dec!(0.02)),
                    1,
                    dec!(2.03790339885560979327158187),
                ),
                (
                    ExitPolicy::Expiration,
                    7,
                    dec!(1.18845730849111921068490390),
                ),
                (
                    ExitPolicy::Expiration,
                    7,
                    dec!(-13.55094218966630397629925114),
                ),
                (
                    ExitPolicy::Expiration,
                    7,
                    dec!(5.33867358493727103384128390),
                ),
            ],
            PathStatistics {
                total_paths: 6,
                profitable_count: 4,
                loss_count: 2,
                average_pnl: dec!(-1.89322111113797166748872778),
                median_pnl: dec!(1.613180353673364501978242885),
                std_dev_pnl: dec!(7.698418618611628117249908796),
                best_pnl: dec!(5.33867358493727103384128390),
                worst_pnl: dec!(-13.55094218966630397629925114),
                win_rate: dec!(66.666666666666666666666666670),
                average_holding_period: dec!(5.50),
            },
        ),
        (
            vec![
                (
                    ExitPolicy::LossPercent(dec!(0.02)),
                    5,
                    dec!(-2.59246614454599439043991906),
                ),
                (
                    ExitPolicy::LossPercent(dec!(0.02)),
                    2,
                    dec!(-7.34369550999532982339956817),
                ),
                (
                    ExitPolicy::Expiration,
                    7,
                    dec!(7.48628923580217360291365717),
                ),
                (
                    ExitPolicy::LossPercent(dec!(0.02)),
                    1,
                    dec!(-2.12944114934388107072304619),
                ),
                (
                    ExitPolicy::LossPercent(dec!(0.02)),
                    1,
                    dec!(-2.72399777335306077210996701),
                ),
                (
                    ExitPolicy::LossPercent(dec!(0.02)),
                    4,
                    dec!(-3.12153772724881238670084110),
                ),
            ],
            PathStatistics {
                total_paths: 6,
                profitable_count: 1,
                loss_count: 5,
                average_pnl: dec!(-1.7374748447808174734099473933),
                median_pnl: dec!(-2.658231958949527581274943035),
                std_dev_pnl: dec!(4.904712819710091469954636495),
                best_pnl: dec!(7.48628923580217360291365717),
                worst_pnl: dec!(-7.34369550999532982339956817),
                win_rate: dec!(16.666666666666666666666666670),
                average_holding_period: dec!(3.3333333333333333333333333333),
            },
        ),
        (
            vec![
                (
                    ExitPolicy::TimeSteps(3),
                    3,
                    dec!(1.93371828671763880277547853),
                ),
                (
                    ExitPolicy::TimeSteps(3),
                    3,
                    dec!(-3.47552927038764869490187364),
                ),
                (
                    ExitPolicy::TimeSteps(3),
                    3,
                    dec!(6.35894869176309911583868480),
                ),
                (
                    ExitPolicy::TimeSteps(3),
                    3,
                    dec!(-0.38136043321041113223742131),
                ),
                (
                    ExitPolicy::TimeSteps(3),
                    3,
                    dec!(-4.49851263865513125764874821),
                ),
                (
                    ExitPolicy::TimeSteps(3),
                    3,
                    dec!(0.11909558534201548462853665),
                ),
            ],
            PathStatistics {
                total_paths: 6,
                profitable_count: 3,
                loss_count: 3,
                average_pnl: dec!(0.0093933702615937197424428033),
                median_pnl: dec!(-0.13113242393419782380444233),
                std_dev_pnl: dec!(3.9153672796395549553726077206),
                best_pnl: dec!(6.35894869176309911583868480),
                worst_pnl: dec!(-4.49851263865513125764874821),
                win_rate: dec!(50.000),
                average_holding_period: dec!(3),
            },
        ),
        (
            vec![
                (
                    ExitPolicy::ProfitPercent(dec!(0.03)),
                    4,
                    dec!(3.13107420108338488440950979),
                ),
                (
                    ExitPolicy::LossPercent(dec!(0.01)),
                    2,
                    dec!(-7.34369550999532982339956817),
                ),
                (
                    ExitPolicy::ProfitPercent(dec!(0.03)),
                    2,
                    dec!(4.42215831814424192670993932),
                ),
                (
                    ExitPolicy::LossPercent(dec!(0.01)),
                    1,
                    dec!(-2.12944114934388107072304619),
                ),
                (
                    ExitPolicy::LossPercent(dec!(0.01)),
                    1,
                    dec!(-2.72399777335306077210996701),
                ),
                (
                    ExitPolicy::LossPercent(dec!(0.01)),
                    4,
                    dec!(-3.12153772724881238670084110),
                ),
            ],
            PathStatistics {
                total_paths: 6,
                profitable_count: 2,
                loss_count: 4,
                average_pnl: dec!(-1.2942399401189095403023288933),
                median_pnl: dec!(-2.4267194613484709214165066),
                std_dev_pnl: dec!(4.357460561192561078899785334),
                best_pnl: dec!(4.42215831814424192670993932),
                worst_pnl: dec!(-7.34369550999532982339956817),
                win_rate: dec!(33.333333333333333333333333330),
                average_holding_period: dec!(2.3333333333333333333333333333),
            },
        ),
        (
            vec![
                (
                    ExitPolicy::Expiration,
                    7,
                    dec!(-3.45291616149949416241319444),
                ),
                (
                    ExitPolicy::Expiration,
                    7,
                    dec!(-9.50449297052891095084039500),
                ),
                (
                    ExitPolicy::Expiration,
                    7,
                    dec!(7.48628923580217360291365717),
                ),
                (
                    ExitPolicy::Expiration,
                    7,
                    dec!(1.18845730849111921068490390),
                ),
                (
                    ExitPolicy::Expiration,
                    7,
                    dec!(-13.55094218966630397629925114),
                ),
                (
                    ExitPolicy::Expiration,
                    7,
                    dec!(5.33867358493727103384128390),
                ),
            ],
            PathStatistics {
                total_paths: 6,
                profitable_count: 3,
                loss_count: 3,
                average_pnl: dec!(-2.082488532077357540352165935),
                median_pnl: dec!(-1.13222942650418747586414527),
                std_dev_pnl: dec!(8.316536453333854824302524666),
                best_pnl: dec!(7.48628923580217360291365717),
                worst_pnl: dec!(-13.55094218966630397629925114),
                win_rate: dec!(50.000),
                average_holding_period: dec!(7),
            },
        ),
    ]
}

#[test]
fn test_deterministic_repeated_runs_are_identical_for_every_walk_type() {
    for (name, walk_type) in stochastic_walk_types() {
        let first = generate(walk_type.clone(), Some(SEED));
        for _ in 0..3 {
            let again = generate(walk_type.clone(), Some(SEED));
            assert_eq!(
                bits(&decimals(&again.prices)),
                bits(&decimals(&first.prices)),
                "{name}: price path differs between runs"
            );
            assert_eq!(
                again.vols.as_deref().map(|v| bits(&decimals(v))),
                first.vols.as_deref().map(|v| bits(&decimals(v))),
                "{name}: volatility path differs between runs"
            );
        }
    }
}

#[test]
fn test_deterministic_seed_selects_the_path_for_every_walk_type() {
    for (name, walk_type) in stochastic_walk_types() {
        let a = generate(walk_type.clone(), Some(SEED));
        let b = generate(walk_type, Some(SEED + 1));
        assert_ne!(a.prices, b.prices, "{name}: two seeds gave one path");
    }
}

#[test]
fn test_deterministic_unseeded_walk_draws_from_the_thread_rng() {
    // No seed keeps the 0.21 behaviour: a fresh path on every call.
    let a = generate(geometric_brownian(), None);
    let b = generate(geometric_brownian(), None);
    assert_eq!(a.prices.len(), SIZE);
    assert_ne!(a.prices, b.prices, "unseeded walks repeated a path");
}

#[test]
fn test_deterministic_historical_walk_ignores_the_seed() {
    let prices: Vec<Positive> = [100, 101, 99, 102, 98, 103, 97, 104]
        .into_iter()
        .map(|p| pos(Decimal::from(p)))
        .collect();
    let historical = WalkType::Historical {
        timeframe: TimeFrame::Day,
        prices: prices.clone(),
        symbol: None,
    };
    assert_eq!(generate(historical.clone(), Some(SEED)).prices, prices);
    assert_eq!(generate(historical, None).prices, prices);
}

/// A pure `next_y`: scales the price by one plus the step volatility.
fn scaled_by_vol(
    price: &Positive,
    vol: Option<Positive>,
    _x: &Xstep<Positive>,
) -> Result<Option<Positive>, SimulationError> {
    Ok(Some(price.checked_mul(
        &(vol.unwrap_or(Positive::ZERO) + Positive::ONE),
    )?))
}

#[test]
fn test_deterministic_walk_steps_par_equals_walk_steps_for_every_walk_type() {
    for (name, walk_type) in stochastic_walk_types() {
        let params = walk_params(walk_type.clone(), Some(SEED));
        let serial = walk_steps(&params, scaled_by_vol)
            .unwrap_or_else(|e: SimulationError| panic!("{name}: serial walk failed: {e}"));
        let parallel = walk_steps_par(&params, scaled_by_vol)
            .unwrap_or_else(|e: SimulationError| panic!("{name}: parallel walk failed: {e}"));
        assert_eq!(serial.len(), SIZE, "{name}");
        assert_eq!(parallel.len(), serial.len(), "{name}");
        for (s, p) in serial.iter().zip(&parallel) {
            assert_eq!(
                bits(&[s.y.value().to_dec()]),
                bits(&[p.y.value().to_dec()]),
                "{name}: y differs"
            );
            assert_eq!(s.y.index(), p.y.index(), "{name}: y index differs");
            assert_eq!(s.x.index(), p.x.index(), "{name}: x index differs");
            assert_eq!(
                s.x.days_left().ok(),
                p.x.days_left().ok(),
                "{name}: x days differ"
            );
        }

        // The driver consumes the same seeded path `generate_with_vol` pins.
        let path = generate(walk_type, Some(SEED));
        let identity = walk_steps(&params, |price, _vol, _x| {
            Ok::<_, SimulationError>(Some(*price))
        })
        .unwrap_or_else(|e| panic!("{name}: identity walk failed: {e}"));
        let driven: Vec<Positive> = identity.iter().map(|step| *step.y.value()).collect();
        assert_eq!(
            bits(&decimals(&driven)),
            bits(&decimals(&path.prices)),
            "{name}: driver and kernel disagree"
        );
    }
}

#[test]
fn test_deterministic_simulator_seeded_walks_match_pinned() {
    let sim = seeded_simulator();
    assert_eq!(sim.len(), SIMULATOR_WALKS);
    assert_bit_identical(
        &terminal_prices(&sim),
        &[
            dec!(96.54708383850050583758680556),
            dec!(90.49550702947108904915960500),
            dec!(107.48628923580217360291365717),
            dec!(101.18845730849111921068490390),
            dec!(86.44905781033369602370074886),
            dec!(105.33867358493727103384128390),
        ],
        "simulator terminal prices",
    );

    // Every walk of a rebuilt simulator repeats, step for step.
    let again = seeded_simulator();
    for (a, b) in sim.into_iter().zip(&again) {
        let a: Vec<Decimal> = a.get_steps().iter().map(|s| s.y.value().to_dec()).collect();
        let b: Vec<Decimal> = b.get_steps().iter().map(|s| s.y.value().to_dec()).collect();
        assert_eq!(bits(&a), bits(&b));
    }
}

#[test]
fn test_deterministic_simulator_walk_seeds_do_not_depend_on_walk_count() {
    let params = walk_params(geometric_brownian(), Some(SEED));
    let few = Simulator::new("few".to_string(), 2, &params, generator_positive)
        .unwrap_or_else(|e| panic!("simulator failed: {e}"));
    let many = Simulator::new("many".to_string(), 5, &params, generator_positive)
        .unwrap_or_else(|e| panic!("simulator failed: {e}"));
    let few = terminal_prices(&few);
    let many = terminal_prices(&many);
    assert_eq!(bits(&few), bits(&many[..2]));
    assert_ne!(many[0], many[1], "walks of one simulator repeated a path");
}

#[test]
fn test_deterministic_exit_outcomes_and_statistics_match_pinned() {
    let sim = seeded_simulator();
    for (policy, (expected, expected_stats)) in exit_policies().into_iter().zip(pinned_outcomes()) {
        let outcomes = evaluate_paths(&UnderlyingEvaluator, &sim, &policy)
            .unwrap_or_else(|e| panic!("{policy:?}: evaluation failed: {e}"));
        assert_eq!(outcomes.len(), expected.len(), "{policy:?}");
        for (outcome, (reason, holding_period, pnl)) in outcomes.iter().zip(&expected) {
            assert_eq!(&outcome.exit_reason, reason, "{policy:?}: exit reason");
            assert_eq!(
                outcome.holding_period, *holding_period,
                "{policy:?}: holding"
            );
            assert_bit_identical(&[outcome.pnl.unwrap_or(Decimal::MIN)], &[*pnl], "path P&L");
        }

        let stats = PathStatistics::from_outcomes(&outcomes)
            .unwrap_or_else(|e| panic!("{policy:?}: statistics failed: {e}"));
        assert_eq!(stats, expected_stats, "{policy:?}: statistics drifted");
        let decimal_fields = |s: &PathStatistics| {
            bits(&[
                s.average_pnl,
                s.median_pnl,
                s.std_dev_pnl,
                s.best_pnl,
                s.worst_pnl,
                s.win_rate,
                s.average_holding_period,
            ])
        };
        assert_eq!(
            decimal_fields(&stats),
            decimal_fields(&expected_stats),
            "{policy:?}: statistics changed representation"
        );
    }
}

#[test]
fn test_deterministic_seeded_mc_price_matches_pinned() {
    let option = Options::new(
        OptionType::European,
        Side::Long,
        "TEST".to_string(),
        Positive::HUNDRED,
        ExpirationDate::Days(pos(dec!(30))),
        pos(dec!(0.5)),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        OptionStyle::Call,
        Positive::ZERO,
        None,
    );
    let price = seeded_simulator()
        .get_mc_option_price(&option)
        .unwrap_or_else(|e| panic!("pricing failed: {e}"))
        .to_dec();
    // Re-baselined by #844: each path's payoff is the exact `Decimal`
    // intrinsic value instead of an `f64` difference taken back to
    // `Decimal`, which moves the mean by `8.0e-16` from
    // `2.3259914839804568426769173659`.
    let pinned = dec!(2.3259914839804576472702068362);
    assert_bit_identical(&[price], &[pinned], "Monte-Carlo price");
}
