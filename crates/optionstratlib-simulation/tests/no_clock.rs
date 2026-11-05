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
