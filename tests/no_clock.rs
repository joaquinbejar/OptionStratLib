//! The facade without the wall clock (#944).

#[cfg(not(feature = "clock"))]
mod without_clock {
    use optionstratlib::error::ClockError;
    use optionstratlib::utils::time::now;

    /// Fails if anything in this configuration turned the clock on: a
    /// capability, an optional dependency or a dev-dependency. Features unify,
    /// so a run with the clock on would prove nothing.
    #[test]
    fn test_no_dependency_turns_the_clock_on() {
        assert_eq!(now(), Err(ClockError::Unavailable));
    }
}
