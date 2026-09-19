//! `BasicAble` implementations for the simulation containers.
//!
//! `BasicAble` is a strategies trait; `Simulator` and `RandomWalk` are
//! simulation types. The implementations live here because a trait impl
//! must sit with the trait or with the type, and simulation must not
//! depend on strategies. Both only forward to the inherent `get_title`
//! accessors.
//!
//! These impls are scheduled for removal in the 0.22.0 breaking batch
//! (ADR-0001 D2, simulation row): once the crates are separate neither
//! side can host them, and the inherent accessors already cover the only
//! use they had.

use crate::simulation::randomwalk::RandomWalk; // facade-compat: simulation
use crate::simulation::simulator::Simulator; // facade-compat: simulation
use crate::strategies::base::BasicAble;
use positive::Positive;
use std::fmt::Display;
use std::ops::AddAssign;

impl<X, Y> BasicAble for Simulator<X, Y>
where
    X: AddAssign + Copy + Display + TryInto<Positive>,
    Y: Clone + Display + TryInto<Positive>,
{
    fn get_title(&self) -> String {
        Simulator::get_title(self).to_string()
    }
}

impl<X, Y> BasicAble for RandomWalk<X, Y>
where
    X: AddAssign + Copy + Display + TryInto<Positive>,
    Y: Clone + Display + TryInto<Positive>,
{
    fn get_title(&self) -> String {
        RandomWalk::get_title(self).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExpirationDate;
    use crate::simulation::steps::Step;
    use crate::simulation::{WalkParams, WalkType, WalkTypeAble, generator_positive};
    use crate::utils::TimeFrame;
    use positive::pos_or_panic;

    #[derive(Clone)]
    struct TestWalker;
    impl WalkTypeAble<Positive, Positive> for TestWalker {}

    fn params() -> WalkParams<Positive, Positive> {
        WalkParams {
            size: 2,
            init_step: Step::new(
                Positive::ONE,
                TimeFrame::Day,
                ExpirationDate::Days(pos_or_panic!(30.0)),
                Positive::HUNDRED,
            ),
            walker: Box::new(TestWalker),
            walk_type: WalkType::Historical {
                timeframe: TimeFrame::Day,
                prices: vec![Positive::HUNDRED, pos_or_panic!(101.0)],
                symbol: None,
            },
        }
    }

    #[test]
    fn test_basic_able_titles_match_inherent_accessors() {
        let sim = Simulator::new("Sim".to_string(), 1, &params(), generator_positive).unwrap();
        assert_eq!(BasicAble::get_title(&sim), "Sim");
        let walk = sim.first().unwrap();
        assert_eq!(BasicAble::get_title(walk), "Sim_0");
        assert_eq!(BasicAble::get_title(walk), RandomWalk::get_title(walk));
    }
}
