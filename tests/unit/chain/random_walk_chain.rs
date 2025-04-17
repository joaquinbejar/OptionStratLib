/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/3/25
******************************************************************************/
use optionstratlib::chains::OptionChain;
use optionstratlib::simulation::randomwalk::RandomWalk;
use optionstratlib::simulation::steps::{Step, Xstep, Ystep};
use optionstratlib::simulation::{WalkParams, WalkType, WalkTypeAble};
use optionstratlib::utils::time::{convert_time_frame, get_x_days_formatted};
use optionstratlib::utils::{Len, TimeFrame, setup_logger};
use optionstratlib::{ExpirationDate, Positive, pos};
use rust_decimal_macros::dec;
use std::error::Error;
use tracing::info;

struct MockWalker {}
impl MockWalker {
    fn new() -> Self {
        MockWalker {}
    }
}

impl WalkTypeAble<Positive, OptionChain> for MockWalker {}

fn create_chain_from_step(
    previous_y_step: &Ystep<OptionChain>,
    new_price: &Positive,
    volatility: Option<Positive>,
) -> Result<OptionChain, Box<dyn std::error::Error>> {
    let chain = previous_y_step.value();
    let mut chain_params = chain.to_build_params()?;
    chain_params.set_underlying_price(new_price);
    if let Some(volatility) = volatility {
        chain_params.set_implied_volatility(Some(volatility));
    }

    let new_chain = OptionChain::build_chain(&chain_params);
    assert!(new_chain.len() > 0,);
    Ok(new_chain)
}

fn generator(walk_params: &WalkParams<Positive, OptionChain>) -> Vec<Step<Positive, OptionChain>> {
    info!("{}", walk_params);
    let mut y_steps = walk_params.walker.geometric_brownian(walk_params).unwrap();
    let _ = y_steps.remove(0);
    let mut steps: Vec<Step<Positive, OptionChain>> = vec![walk_params.init_step.clone()];
    let mut previous_x_step = walk_params.init_step.x;
    let mut previous_y_step = walk_params.ystep();

    for y_step in y_steps.iter() {
        previous_x_step = match previous_x_step.next() {
            Ok(x_step) => x_step,
            Err(_) => break,
        };
        // convert y_step to OptionChain
        let y_step_chain: OptionChain =
            create_chain_from_step(&previous_y_step, y_step, Some(pos!(0.20))).unwrap();
        previous_y_step = previous_y_step.next(y_step_chain).clone();
        let step = Step {
            x: previous_x_step,
            y: previous_y_step.clone(),
        };
        info!("{}", step);
        assert!(*step.x.index() >= 1);
        assert_eq!(*step.x.time_unit(), TimeFrame::Minute);
        assert_eq!(*step.x.step_size_in_time(), Positive::ONE);
        assert!(*step.y.index() >= 1);
        steps.push(step)
    }

    assert!(steps.len() <= walk_params.size);
    steps
}
#[test]
fn test_random_walk_chain() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let n_steps = 20;

    let mut initial_chain =
        OptionChain::load_from_json("examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    initial_chain.update_expiration_date(get_x_days_formatted(2));

    assert_eq!(initial_chain.underlying_price, pos!(5781.88));
    assert_eq!(initial_chain.symbol, "SP500");

    let std_dev = pos!(20.0);
    let walker = Box::new(MockWalker::new());
    let days = pos!(30.0);

    let walk_params = WalkParams {
        size: n_steps,
        init_step: Step {
            x: Xstep::new(Positive::ONE, TimeFrame::Minute, ExpirationDate::Days(days)),
            y: Ystep::new(0, initial_chain),
        },
        walk_type: WalkType::GeometricBrownian {
            dt: convert_time_frame(pos!(1.0) / days, &TimeFrame::Minute, &TimeFrame::Day), // TODO
            drift: dec!(0.0),
            volatility: std_dev,
        },
        walker,
    };

    let random_walk = RandomWalk::new("Random Walk".to_string(), &walk_params, generator);
    info!("Random Walk: {}", random_walk);
    assert_eq!(random_walk.len(), n_steps);

    info!("Last Chain: {}", random_walk.last().unwrap().y.value());

    Ok(())
}
