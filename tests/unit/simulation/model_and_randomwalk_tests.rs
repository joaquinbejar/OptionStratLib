use optionstratlib::simulation::steps::{Step, Xstep, Ystep};
use optionstratlib::simulation::{WalkParams, WalkType, WalkTypeAble};
use optionstratlib::simulation::randomwalk::RandomWalk;
use optionstratlib::simulation::simulator::Simulator;
use optionstratlib::visualization::Graph; // to exercise graph_data/graph_config
use optionstratlib::ExpirationDate;use positive::Positive;
use optionstratlib::utils::TimeFrame;
use positive::pos_or_panic;
use rust_decimal::Decimal;
use std::error::Error;
use std::fmt::Display;
use std::ops::AddAssign;

// A minimal walker that relies on default trait implementations
struct TestWalker;

impl<X, Y> WalkTypeAble<X, Y> for TestWalker
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
}

fn make_params(size: usize, start_price: Positive) -> WalkParams<Positive, Positive> {
    let init_x = Xstep::new(Positive::ONE, TimeFrame::Day, ExpirationDate::Days(pos_or_panic!(size as f64)));
    let init_y = Ystep::new(0, start_price);
    let init_step = Step { x: init_x, y: init_y };

    WalkParams {
        size,
        init_step,
        walk_type: WalkType::Brownian { dt: Positive::ONE, drift: Decimal::ZERO, volatility: pos_or_panic!(0.2) },
        walker: Box::new(TestWalker),
    }
}

fn simple_generator(params: &WalkParams<Positive, Positive>) -> Vec<Step<Positive, Positive>> {
    // Build a tiny deterministic series using Step::next
    let mut out = Vec::with_capacity(params.size);
    let mut current = params.init_step.clone();
    out.push(current.clone());
    for i in 1..params.size {
        // strictly increasing y to avoid errors
        let new_y = Positive::from(current.get_positive_value() + pos_or_panic!(i as f64));
        current = current.next(new_y).expect("step.next should succeed");
        out.push(current.clone());
    }
    out
}

#[test]
fn walktype_historical_display_is_covered() {
    let wt = WalkType::Historical {
        timeframe: TimeFrame::Day,
        prices: vec![Positive::ONE, Positive::TWO, pos_or_panic!(3.0)],
        symbol: Some("ABC".to_string()),
    };
    let s = format!("{}", wt);
    assert!(s.contains("Historical"));
    assert!(s.contains("ABC"));
    assert!(s.contains("prices"));
}

#[test]
fn randomwalk_profit_error_and_graph_paths() {
    let params = make_params(5, Positive::HUNDRED);
    let rw = RandomWalk::new("RW_Title".to_string(), &params, simple_generator);

    // Exercise Profit::calculate_profit_at error branch
    let err = rw.calculate_profit_at(&pos_or_panic!(101.0)).unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.to_lowercase().contains("not implemented"));

    // Exercise graph_data and graph_config code paths (no assertions on values needed for coverage)
    let data = rw.graph_data();
    let cfg = rw.graph_config();
    // basic sanity checks
    match data {
        optionstratlib::visualization::GraphData::Series(series) => {
            assert_eq!(series.name, "RW_Title");
            assert!(!series.x.is_empty());
            assert!(!series.y.is_empty());
        }
        _ => panic!("Expected 2D series"),
    }
    assert_eq!(cfg.title, "RW_Title");
    assert_eq!(cfg.width, 1600);
    assert_eq!(cfg.height, 900);
}

#[test]
fn simulator_last_values_and_profit_error() -> Result<(), Box<dyn Error>> {
    let params = make_params(4, pos_or_panic!(50.0));
    // build a simulator with 2 random walks
    let mut sim = Simulator::new("SIM".to_string(), 2, &params, simple_generator);

    // Accessors across random walks
    let _rws = sim.get_random_walks();
    let last_steps = sim.get_last_steps();
    assert_eq!(last_steps.len(), 2);

    let last_vals = sim.get_last_values();
    assert_eq!(last_vals.len(), 2);

    let last_pos = sim.get_last_positive_values();
    assert_eq!(last_pos.len(), 2);

    // Display implementation
    let disp = format!("{}", sim);
    assert!(disp.contains("Simulator Title: SIM"));

    // Profit::calculate_profit_at error branch
    let err = sim.calculate_profit_at(&pos_or_panic!(55.0)).unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.to_lowercase().contains("not implemented"));

    // Indexing
    let first = &sim[0];
    assert_eq!(first.get_title(), "SIM_0");
    let first_mut = &mut sim[0];
    first_mut.set_title("CHANGED".into());
    assert_eq!(sim[0].get_title(), "CHANGED");

    Ok(())
}
