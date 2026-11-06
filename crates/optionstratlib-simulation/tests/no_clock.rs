//! The simulation steps with and without the wall clock (#944).

use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_core::utils::TimeFrame;
use optionstratlib_simulation::simulation::steps::Xstep;

#[test]
fn test_days_step_needs_no_clock() {
    let step = Xstep::new(
        Positive::ONE,
        TimeFrame::Day,
        ExpirationDate::Days(pos_or_panic!(30.0)),
    );
    assert_eq!(step.days_left().unwrap(), pos_or_panic!(30.0));
    assert_eq!(
        step.next().unwrap().days_left().unwrap(),
        pos_or_panic!(29.0)
    );
}

/// The quadratic rough Heston kernel reads no time: it takes its horizon as a number, so a
/// run gives the same decimals with and without the clock.
#[test]
fn test_quadratic_rough_heston_needs_no_clock() {
    use optionstratlib_core::model::types::OptionStyle;
    use optionstratlib_simulation::simulation::{
        QuadraticRoughHestonGrid, QuadraticRoughHestonKernel, QuadraticRoughHestonParams,
        price_vanilla_quadratic_rough_heston, simulate_quadratic_rough_heston_path,
    };
    use rust_decimal_macros::dec;
    use std::num::NonZeroUsize;

    let kernel = QuadraticRoughHestonKernel::try_new(dec!(0.6), 3, dec!(4)).unwrap();
    let params = QuadraticRoughHestonParams::try_new(
        dec!(0.5),
        dec!(0.1),
        dec!(0.01),
        dec!(1.5),
        dec!(1),
        kernel,
        vec![dec!(-0.05); 3],
        Positive::HUNDRED,
    )
    .unwrap();
    let grid = QuadraticRoughHestonGrid::try_new(dec!(0.25), 10).unwrap();
    let path = simulate_quadratic_rough_heston_path(&params, &grid, 958).unwrap();
    assert_eq!(path.spot().len(), 11);
    let estimate = price_vanilla_quadratic_rough_heston(
        &params,
        &grid,
        OptionStyle::Call,
        Positive::HUNDRED,
        NonZeroUsize::new(100).unwrap(),
        958,
    )
    .unwrap();
    assert!(estimate.price() > dec!(0));
}

#[cfg(not(feature = "clock"))]
mod without_clock {
    use super::*;
    use chrono::{Duration, TimeZone, Utc};
    use optionstratlib_core::error::ClockError;
    use optionstratlib_core::utils::time::now;
    use optionstratlib_simulation::error::SimulationError;

    /// Fails if anything in this configuration turned the clock on.
    #[test]
    fn test_no_dependency_turns_the_clock_on() {
        assert_eq!(now(), Err(ClockError::Unavailable));
    }

    /// A parallel `Simulator::new` runs the generator on rayon workers, and its steps read
    /// their days there.
    #[test]
    fn test_simulator_carries_the_reference_to_its_workers() {
        use optionstratlib_simulation::simulation::simulator::Simulator;
        use optionstratlib_simulation::simulation::steps::Step;
        use optionstratlib_simulation::simulation::{WalkParams, WalkType, WalkTypeAble};

        #[derive(Clone)]
        struct Walker;
        impl WalkTypeAble<Positive, Positive> for Walker {}

        let base = Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap();
        ExpirationDate::set_reference_datetime(Some(base));
        let init_step = Step::new(
            Positive::ONE,
            TimeFrame::Day,
            ExpirationDate::DateTime(base + Duration::days(30)),
            Positive::HUNDRED,
        );
        let params = WalkParams {
            size: 2,
            init_step,
            walker: Box::new(Walker),
            walk_type: WalkType::Historical {
                timeframe: TimeFrame::Day,
                prices: vec![Positive::HUNDRED, Positive::HUNDRED],
                symbol: None,
            },
            seed: None,
        };
        let simulator = Simulator::new(
            "paths".to_string(),
            32,
            &params,
            |walk: &WalkParams<Positive, Positive>| {
                let next = walk.init_step.next(Positive::HUNDRED)?;
                Ok::<_, SimulationError>(vec![walk.init_step.clone(), next])
            },
        );
        ExpirationDate::set_reference_datetime(None);
        assert_eq!(simulator.unwrap().get_random_walks().len(), 32);
    }

    /// With no time to measure from, the step keeps its instant instead of
    /// reading as already expired, and asking for the days is a typed error.
    #[test]
    fn test_datetime_step_without_a_reference_reports_it_rather_than_expiring() {
        let expiry = Utc.with_ymd_and_hms(2030, 1, 31, 0, 0, 0).unwrap();
        let step = Xstep::new(
            Positive::ONE,
            TimeFrame::Day,
            ExpirationDate::DateTime(expiry),
        );
        assert!(matches!(
            step.days_left(),
            Err(SimulationError::ExpirationDate(_))
        ));
    }

    #[test]
    fn test_datetime_step_measures_from_the_reference_datetime() {
        let base = Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap();
        ExpirationDate::set_reference_datetime(Some(base));
        let step = Xstep::new(
            Positive::ONE,
            TimeFrame::Day,
            ExpirationDate::DateTime(base + Duration::days(30)),
        );
        let days = step.days_left();
        ExpirationDate::set_reference_datetime(None);
        assert_eq!(days.unwrap(), pos_or_panic!(30.0));
    }
}
