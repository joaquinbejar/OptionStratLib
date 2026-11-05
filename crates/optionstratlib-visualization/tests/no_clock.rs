//! The visualization crate without the wall clock (#944).

#[cfg(not(feature = "clock"))]
mod without_clock {
    use optionstratlib_core::error::ClockError;
    use optionstratlib_core::utils::time::now;

    /// Fails if anything in this configuration turned the clock on, the
    /// Plotly and native export dependencies included.
    #[test]
    fn test_no_dependency_turns_the_clock_on() {
        assert_eq!(now(), Err(ClockError::Unavailable));
    }
}
