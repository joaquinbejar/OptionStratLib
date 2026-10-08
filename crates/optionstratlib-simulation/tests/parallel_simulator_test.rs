//! The parallel [`Simulator::new`] against the serial build (#860).
//!
//! From [`PARALLEL_MIN_WALKS`] walks up, `Simulator::new` builds its walks on
//! the rayon pool. A seeded simulator must stay bit-identical to the serial
//! build it replaced: walk `i` seeded with the `i`-th `u64` drawn from
//! `deterministic_rng(seed)`, titled `{title}_{i}`, in index order. These
//! tests build that serial reference by hand, walk by walk through
//! [`RandomWalk::new`], and compare every step of every path exactly, on both
//! sides of the threshold and for several seeds. A failing build must report
//! the error of its lowest-indexed failing walk, as the serial loop did.

use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::utils::{TimeFrame, deterministic_rng};
use optionstratlib_simulation::error::SimulationError;
use optionstratlib_simulation::simulation::randomwalk::RandomWalk;
use optionstratlib_simulation::simulation::simulator::{PARALLEL_MIN_WALKS, Simulator};
use optionstratlib_simulation::simulation::steps::Step;
use optionstratlib_simulation::simulation::{
    WalkParams, WalkType, WalkTypeAble, generator_positive,
};
use rand::RngExt;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Points per path, the initial value included.
const STEPS: usize = 30;

/// Seeds every comparison runs for.
const SEEDS: [u64; 5] = [0, 1, 42, 685, u64::MAX];

#[derive(Clone)]
struct Walker;

impl WalkTypeAble<Positive, Positive> for Walker {}

fn pos(value: Decimal) -> Positive {
    Positive::new_decimal(value).unwrap_or_else(|e| panic!("{value} is not positive: {e}"))
}

fn walk_params(walk_type: WalkType, seed: Option<u64>) -> WalkParams<Positive, Positive> {
    WalkParams {
        size: STEPS,
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

fn geometric_brownian() -> WalkType {
    WalkType::GeometricBrownian {
        dt: pos(dec!(0.004)),
        drift: dec!(0.05),
        volatility: pos(dec!(0.2)),
    }
}

fn heston() -> WalkType {
    WalkType::Heston {
        dt: pos(dec!(0.004)),
        drift: dec!(0.05),
        volatility: pos(dec!(0.2)),
        kappa: pos(dec!(2.0)),
        theta: pos(dec!(0.04)),
        xi: pos(dec!(0.3)),
        rho: dec!(-0.7),
    }
}

/// Walk counts on both sides of the threshold.
fn sizes() -> Vec<usize> {
    vec![
        1,
        PARALLEL_MIN_WALKS - 1,
        PARALLEL_MIN_WALKS,
        PARALLEL_MIN_WALKS + 1,
        64,
        200,
    ]
}

/// The seeds `Simulator::new` hands its walks, in walk order.
fn walk_seeds(seed: u64, size: usize) -> Vec<u64> {
    let mut rng = deterministic_rng(seed);
    (0..size).map(|_| rng.random::<u64>()).collect()
}

/// The serial build: one walk at a time, each with its own seed.
fn serial_reference(
    title: &str,
    size: usize,
    params: &WalkParams<Positive, Positive>,
) -> Vec<RandomWalk<Positive, Positive>> {
    let seed = params.seed.expect("the reference is built for seeded runs");
    walk_seeds(seed, size)
        .into_iter()
        .enumerate()
        .map(|(i, walk_seed)| {
            let mut walk_params = params.clone();
            walk_params.seed = Some(walk_seed);
            RandomWalk::new(format!("{title}_{i}"), &walk_params, generator_positive)
                .unwrap_or_else(|e| panic!("serial walk {i} failed: {e}"))
        })
        .collect()
}

fn assert_same_walks(
    built: &Simulator<Positive, Positive>,
    reference: &[RandomWalk<Positive, Positive>],
    context: &str,
) {
    let walks = built.get_random_walks();
    assert_eq!(walks.len(), reference.len(), "{context}: walk count");
    for (i, (walk, expected)) in walks.iter().zip(reference).enumerate() {
        assert_eq!(
            walk.get_title(),
            expected.get_title(),
            "{context}: title of walk {i}"
        );
        let steps = walk.get_steps();
        let expected_steps = expected.get_steps();
        assert_eq!(
            steps.len(),
            expected_steps.len(),
            "{context}: walk {i} length"
        );
        for (j, (step, expected_step)) in steps.iter().zip(&expected_steps).enumerate() {
            assert_eq!(
                step.get_value(),
                expected_step.get_value(),
                "{context}: walk {i} step {j} value"
            );
            assert_eq!(
                step.get_index(),
                expected_step.get_index(),
                "{context}: walk {i} step {j} index"
            );
            assert_eq!(
                format!("{step:?}"),
                format!("{expected_step:?}"),
                "{context}: walk {i} step {j}"
            );
        }
    }
}

// The serial-side tests build `PARALLEL_MIN_WALKS - 1` walks.
const _: () = assert!(PARALLEL_MIN_WALKS > 1);

#[test]
fn test_seeded_simulator_matches_serial_build_geometric_brownian() {
    for seed in SEEDS {
        let params = walk_params(geometric_brownian(), Some(seed));
        for size in sizes() {
            let built = Simulator::new("gbm".to_string(), size, &params, generator_positive)
                .unwrap_or_else(|e| panic!("seed {seed}, {size} walks: {e}"));
            let reference = serial_reference("gbm", size, &params);
            assert_same_walks(
                &built,
                &reference,
                &format!("gbm seed {seed}, {size} walks"),
            );
        }
    }
}

#[test]
fn test_seeded_simulator_matches_serial_build_heston() {
    for seed in SEEDS {
        let params = walk_params(heston(), Some(seed));
        for size in [PARALLEL_MIN_WALKS, 64] {
            let built = Simulator::new("heston".to_string(), size, &params, generator_positive)
                .unwrap_or_else(|e| panic!("seed {seed}, {size} walks: {e}"));
            let reference = serial_reference("heston", size, &params);
            assert_same_walks(
                &built,
                &reference,
                &format!("heston seed {seed}, {size} walks"),
            );
        }
    }
}

#[test]
fn test_seeded_parallel_simulator_is_repeatable() {
    let params = walk_params(geometric_brownian(), Some(42));
    let first = Simulator::new("again".to_string(), 200, &params, generator_positive)
        .unwrap_or_else(|e| panic!("first run: {e}"));
    for run in 0..5 {
        let next = Simulator::new("again".to_string(), 200, &params, generator_positive)
            .unwrap_or_else(|e| panic!("run {run}: {e}"));
        let reference: Vec<RandomWalk<Positive, Positive>> =
            first.get_random_walks().into_iter().cloned().collect();
        assert_same_walks(&next, &reference, &format!("run {run}"));
    }
}

#[test]
fn test_unseeded_parallel_simulator_keeps_index_order() {
    let params = walk_params(geometric_brownian(), None);
    let built = Simulator::new("free".to_string(), 64, &params, generator_positive)
        .unwrap_or_else(|e| panic!("unseeded build: {e}"));
    let titles: Vec<String> = built
        .get_random_walks()
        .iter()
        .map(|walk| walk.get_title().to_string())
        .collect();
    let expected: Vec<String> = (0..64).map(|i| format!("free_{i}")).collect();
    assert_eq!(titles, expected);
    for walk in built.get_random_walks() {
        assert_eq!(walk.get_steps().len(), STEPS);
    }
}

type Params = WalkParams<Positive, Positive>;

type Path = Vec<Step<Positive, Positive>>;

/// A generator that fails on the walks whose seed is in `failing`, and
/// reports that seed, so the test can tell which walk's error came back.
fn failing_on(failing: Vec<u64>) -> impl Fn(&Params) -> Result<Path, u64> + Clone + Send + Sync {
    move |params: &Params| {
        let seed = params.seed.unwrap_or_default();
        if failing.contains(&seed) {
            return Err(seed);
        }
        generator_positive(params).map_err(|e: SimulationError| panic!("generator failed: {e}"))
    }
}

#[test]
fn test_parallel_simulator_reports_lowest_failing_walk() {
    let size = 200;
    let seeds = walk_seeds(42, size);
    // Listed out of order, so the result cannot follow the list.
    let failing = vec![seeds[150], seeds[17], seeds[40]];
    let params = walk_params(geometric_brownian(), Some(42));
    let result = Simulator::new("fail".to_string(), size, &params, failing_on(failing));
    assert_eq!(result.err(), Some(seeds[17]));
}

#[test]
fn test_serial_simulator_reports_lowest_failing_walk() {
    let size = PARALLEL_MIN_WALKS - 1;
    let seeds = walk_seeds(42, size);
    let failing = vec![seeds[size - 1], seeds[0]];
    let params = walk_params(geometric_brownian(), Some(42));
    let result = Simulator::new("fail".to_string(), size, &params, failing_on(failing));
    assert_eq!(result.err(), Some(seeds[0]));
}

/// More walks than one parallel round of 1024 holds, so the seed stream and
/// the error order are checked across round boundaries.
const ACROSS_ROUNDS: usize = 2 * 1024 + 5;

#[test]
fn test_seeded_simulator_matches_serial_build_across_rounds() {
    for seed in [7, 685] {
        let params = walk_params(geometric_brownian(), Some(seed));
        let built = Simulator::new(
            "rounds".to_string(),
            ACROSS_ROUNDS,
            &params,
            generator_positive,
        )
        .unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        let reference = serial_reference("rounds", ACROSS_ROUNDS, &params);
        assert_same_walks(&built, &reference, &format!("rounds seed {seed}"));
    }
}

#[test]
fn test_parallel_simulator_reports_lowest_failing_walk_across_rounds() {
    let seeds = walk_seeds(42, ACROSS_ROUNDS);
    let failing = vec![seeds[2050], seeds[1500], seeds[1900]];
    let params = walk_params(geometric_brownian(), Some(42));
    let result = Simulator::new(
        "fail".to_string(),
        ACROSS_ROUNDS,
        &params,
        failing_on(failing),
    );
    assert_eq!(result.err(), Some(seeds[1500]));
}
