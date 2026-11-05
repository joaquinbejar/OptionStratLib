//! Property-based tests for panic freedom and the promises of the static
//! arbitrage checks.
//!
//! The library is embedded in long-running services, where a panic kills the
//! worker thread, so every failure must come back as a typed
//! `StaticArbitrageError`, including for inputs that are extreme but structurally
//! valid. The inputs reach the edges of the `Decimal` range, the shape of the
//! input (empty grids, one slice, repeated expiries) and the vertex of a slice.
//!
//! A check that returns must also keep its promises: a finding has a positive
//! size and a positive strike, and a surface the theorems of the paper clear has
//! nothing to report, so rounding cannot create a violation on it.
//!
//! The trait says its values are stable, but nothing enforces that, so a slice that
//! gives a different value on every call is in here too: sorting more than 20 of
//! those by re-reading the expiry panics inside the standard library.

use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::d_sqrt;
use optionstratlib_pricing::error::StaticArbitrageError;
use optionstratlib_pricing::volatility::{
    StaticArbitrageReport, StaticArbitrageUnevaluatedPoint, StaticArbitrageViolation,
    SviParameters, SviSlice, TotalVarianceSlice, check_butterfly, check_calendar,
    check_static_arbitrage,
};
use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::cell::Cell;

/// The smallest representable positive `Decimal`.
const TINY: Decimal = Decimal::from_parts(1, 0, 0, false, 28);

fn positive(value: Decimal) -> Positive {
    Positive::new_decimal(value).unwrap_or(Positive::ZERO)
}

fn extreme_decimal() -> impl Strategy<Value = Decimal> {
    prop_oneof![
        Just(Decimal::ZERO),
        Just(TINY),
        Just(-TINY),
        Just(dec!(0.00000000000001)),
        Just(dec!(0.2)),
        Just(dec!(-0.5)),
        Just(dec!(0.9999999999999999999999999999)),
        Just(dec!(66)),
        Just(dec!(-66)),
        Just(dec!(1000000)),
        Just(dec!(-1000000)),
        Just(Decimal::MAX),
        Just(Decimal::MIN),
    ]
}

/// Values that are mostly ordinary, so that a check gets past the first overflow,
/// and at times extreme.
fn parameter() -> impl Strategy<Value = Decimal> {
    prop_oneof![
        3 => (-2000_i32..2000).prop_map(|cents| Decimal::from(cents) * dec!(0.001)),
        1 => extreme_decimal(),
    ]
}

fn extreme_positive() -> impl Strategy<Value = Positive> {
    prop_oneof![
        3 => (1_u32..4000).prop_map(|cents| positive(Decimal::from(cents) * dec!(0.1))),
        1 => extreme_decimal().prop_map(|value| positive(value.abs())),
    ]
}

/// A valid slice, by shifting `a` onto or above the floor where the parameters
/// fall below it, so the zero-minimum boundary is hit often.
fn any_slice() -> impl Strategy<Value = SviSlice> {
    (
        parameter(),
        parameter(),
        parameter(),
        parameter(),
        parameter(),
        extreme_positive(),
        extreme_positive(),
    )
        .prop_filter_map(
            "the parameters or the slice are invalid",
            |(a, b, rho, m, sigma, forward, expiry)| {
                let parameters = SviParameters::try_new(a, b, rho, m, sigma)
                    .ok()
                    .or_else(|| zero_minimum(b, rho, m, sigma))?;
                SviSlice::try_new(parameters, forward, expiry).ok()
            },
        )
}

/// The parameters with the `a` of a zero minimum, if there is one.
fn zero_minimum(b: Decimal, rho: Decimal, m: Decimal, sigma: Decimal) -> Option<SviParameters> {
    let room = Decimal::ONE.checked_sub(rho.checked_mul(rho)?)?;
    let root = d_sqrt(room, "test").ok()?;
    let floor = b.checked_mul(sigma)?.checked_mul(root)?;
    SviParameters::try_new(-floor, b, rho, m, sigma).ok()
}

fn any_grid() -> impl Strategy<Value = Vec<Decimal>> {
    prop::collection::vec(parameter(), 0..8)
}

/// What every report must say about itself, whatever the input.
fn assert_consistent(report: &StaticArbitrageReport) {
    assert_eq!(
        report.passed(),
        report.violations().is_empty() && report.unevaluated().is_empty()
    );
    for violation in report.violations() {
        assert!(violation.size() > Positive::ZERO, "{violation:?}");
        match violation {
            StaticArbitrageViolation::Butterfly {
                strike,
                g,
                size,
                time_to_expiry,
                ..
            } => {
                assert!(*g < Decimal::ZERO, "{violation:?}");
                assert_eq!(size.to_dec(), -*g, "{violation:?}");
                assert!(*strike > Positive::ZERO, "{violation:?}");
                assert!(*time_to_expiry > Positive::ZERO, "{violation:?}");
            }
            StaticArbitrageViolation::Calendar {
                earlier_expiry,
                later_expiry,
                earlier_strike,
                later_strike,
                earlier_total_variance,
                later_total_variance,
                ..
            } => {
                assert!(earlier_expiry < later_expiry, "{violation:?}");
                assert!(*earlier_strike > Positive::ZERO, "{violation:?}");
                assert!(*later_strike > Positive::ZERO, "{violation:?}");
                assert!(
                    earlier_total_variance > later_total_variance,
                    "{violation:?}"
                );
            }
            other => panic!("unknown kind: {other:?}"),
        }
    }
    for point in report.unevaluated() {
        match point {
            StaticArbitrageUnevaluatedPoint::Butterfly { strike, .. } => {
                assert!(*strike > Positive::ZERO, "{point:?}");
            }
            StaticArbitrageUnevaluatedPoint::Calendar {
                earlier_expiry,
                later_expiry,
                earlier_strike,
                later_strike,
                ..
            } => {
                assert!(earlier_expiry < later_expiry, "{point:?}");
                assert!(*earlier_strike > Positive::ZERO, "{point:?}");
                assert!(*later_strike > Positive::ZERO, "{point:?}");
            }
            other => panic!("unknown kind: {other:?}"),
        }
    }
}

/// A slice whose every value changes on every read, which the trait forbids and
/// nothing prevents. A linear congruential step keeps it deterministic.
struct Unstable {
    state: Cell<u64>,
}

impl Unstable {
    fn next(&self) -> Decimal {
        let state = self
            .state
            .get()
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state.set(state);
        Decimal::from((state >> 40) % 1_000)
    }
}

impl TotalVarianceSlice for Unstable {
    fn time_to_expiry(&self) -> Positive {
        positive(self.next() + Decimal::ONE)
    }
    fn forward(&self) -> Positive {
        positive(self.next() + Decimal::ONE)
    }
    fn total_variance(&self, _k: Decimal) -> Result<Positive, StaticArbitrageError> {
        Ok(positive(self.next()))
    }
    fn total_variance_slope(&self, _k: Decimal) -> Result<Decimal, StaticArbitrageError> {
        Ok(self.next() - dec!(500))
    }
    fn total_variance_curvature(&self, _k: Decimal) -> Result<Decimal, StaticArbitrageError> {
        Ok(self.next() - dec!(500))
    }
    fn total_variance_rounding(&self, _k: Decimal) -> Result<Positive, StaticArbitrageError> {
        Ok(positive(self.next() * dec!(0.0000000001)))
    }
}

/// A slice whose expiry falls by two on every read of any slice of the set. Read once
/// in turn, the slices differ. Read again, as a sort or a comparison would, two of
/// them can read equal.
struct Drifting<'a> {
    base: u32,
    clock: &'a Cell<u32>,
}

impl TotalVarianceSlice for Drifting<'_> {
    fn time_to_expiry(&self) -> Positive {
        self.clock.set(self.clock.get() + 1);
        positive(Decimal::from(self.base + 100_000 - 2 * self.clock.get()))
    }
    fn forward(&self) -> Positive {
        Positive::HUNDRED
    }
    fn total_variance(&self, _k: Decimal) -> Result<Positive, StaticArbitrageError> {
        Ok(Positive::ONE)
    }
    fn total_variance_slope(&self, _k: Decimal) -> Result<Decimal, StaticArbitrageError> {
        Ok(Decimal::ZERO)
    }
    fn total_variance_curvature(&self, _k: Decimal) -> Result<Decimal, StaticArbitrageError> {
        Ok(Decimal::ZERO)
    }
    fn total_variance_rounding(&self, _k: Decimal) -> Result<Positive, StaticArbitrageError> {
        Ok(Positive::ZERO)
    }
}

#[test]
fn test_drifting_expiry_reads_once_and_is_not_a_spurious_duplicate() {
    let clock = Cell::new(0);
    let slices: Vec<Drifting> = (0..64)
        .map(|base| Drifting {
            base,
            clock: &clock,
        })
        .collect();
    let report = check_calendar(&slices, &[dec!(0)]).unwrap();
    assert!(report.passed(), "{report:?}");
    assert_eq!(clock.get(), 64);
}

/// An SSVI slice at the power law `phi = eta / sqrt(theta)`, in raw parameters.
fn ssvi(theta: Decimal, rho: Decimal, eta: Decimal, expiry: Decimal) -> SviSlice {
    let phi = eta / d_sqrt(theta, "test").unwrap();
    let room = d_sqrt(Decimal::ONE - rho * rho, "test").unwrap();
    let parameters = SviParameters::try_new(
        theta / dec!(2) * (Decimal::ONE - rho * rho),
        theta * phi / dec!(2),
        rho,
        -rho / phi,
        room / phi,
    )
    .unwrap();
    SviSlice::try_new(parameters, Positive::HUNDRED, positive(expiry)).unwrap()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// More slices than the standard sort handles without a total order check, so
    /// re-reading a changing expiry would panic in it on nearly every run.
    #[test]
    fn test_unstable_slices_never_panic(
        seeds in prop::collection::vec(any::<u64>(), 21..64),
        grid in any_grid(),
    ) {
        let slices: Vec<Unstable> = seeds
            .iter()
            .map(|&seed| Unstable { state: Cell::new(seed) })
            .collect();
        if let Ok(report) = check_calendar(&slices, &grid) {
            prop_assert!(!grid.is_empty());
            prop_assert!(report.violations().len() <= (slices.len() - 1) * grid.len());
        }
        if check_static_arbitrage(&slices, &grid).is_ok() {
            prop_assert!(!grid.is_empty());
        }
    }

    #[test]
    fn test_check_butterfly_extreme_inputs_never_panic(slice in any_slice(), grid in any_grid()) {
        match check_butterfly(&slice, &grid) {
            Ok(report) => {
                prop_assert!(!grid.is_empty());
                assert_consistent(&report);
            }
            Err(StaticArbitrageError::EmptyGrid) => prop_assert!(grid.is_empty()),
            Err(_) => prop_assert!(!grid.is_empty()),
        }
    }

    #[test]
    fn test_check_calendar_extreme_inputs_never_panic(
        slices in prop::collection::vec(any_slice(), 0..4),
        grid in any_grid(),
    ) {
        match check_calendar(&slices, &grid) {
            Ok(report) => {
                prop_assert!(!grid.is_empty());
                prop_assert!(slices.len() >= 2);
                assert_consistent(&report);
            }
            Err(StaticArbitrageError::EmptyGrid) => prop_assert!(grid.is_empty()),
            Err(StaticArbitrageError::TooFewSlices { got, required }) => {
                prop_assert_eq!(got, slices.len());
                prop_assert!(got < required);
            }
            Err(_) => prop_assert!(slices.len() >= 2),
        }
    }

    #[test]
    fn test_check_static_arbitrage_extreme_inputs_never_panic(
        slices in prop::collection::vec(any_slice(), 0..4),
        grid in any_grid(),
    ) {
        if let Ok(report) = check_static_arbitrage(&slices, &grid) {
            prop_assert!(!grid.is_empty());
            prop_assert!(slices.len() >= 2);
            assert_consistent(&report);
        }
    }

    /// Theorems 4.1 and 4.2 leave nothing to find on these surfaces at any `k`,
    /// so whatever a check reports on them is the arithmetic's doing.
    #[test]
    fn test_ssvi_within_the_theorems_reports_nothing(
        theta_thousandths in 1_u32..1000,
        ratio_hundredths in 101_u32..400,
        rho_hundredths in -90_i32..=90,
        eta_hundredths in 1_u32..=100,
        k_thousandths in prop::collection::vec(-5000_i32..5000, 1..40),
    ) {
        let theta = Decimal::from(theta_thousandths) * dec!(0.001);
        let later_theta = theta * Decimal::from(ratio_hundredths) * dec!(0.01);
        let rho = Decimal::from(rho_hundredths) * dec!(0.01);
        let eta = Decimal::from(eta_hundredths) * dec!(0.01);
        // theta is below 4, so with eta <= 1 and |rho| <= 0.9 Theorem 4.2 holds at
        // both slices: eta * sqrt(theta) * (1 + |rho|) < 3.8 and eta^2 * (1 + |rho|)
        // <= 1.9. Theorem 4.1 holds by Example 4.2, with theta increasing.
        let surface = [
            ssvi(theta, rho, eta, dec!(0.5)),
            ssvi(later_theta, rho, eta, dec!(1)),
        ];
        let grid: Vec<Decimal> = k_thousandths
            .iter()
            .map(|&k| Decimal::from(k) * dec!(0.001))
            .collect();
        let report = check_static_arbitrage(&surface, &grid).unwrap();
        prop_assert!(report.passed(), "{:?}", report);
    }

    /// Calendar on the same `w` is a difference of zero at every point.
    #[test]
    fn test_calendar_same_total_variance_reports_nothing(
        a_thousandths in 0_u32..1000,
        b_thousandths in 0_u32..1000,
        rho_hundredths in -99_i32..99,
        m_thousandths in -1000_i32..1000,
        sigma_thousandths in 1_u32..1000,
        k_thousandths in prop::collection::vec(-5000_i32..5000, 1..40),
    ) {
        let parameters = SviParameters::try_new(
            Decimal::from(a_thousandths) * dec!(0.001),
            Decimal::from(b_thousandths) * dec!(0.001),
            Decimal::from(rho_hundredths) * dec!(0.01),
            Decimal::from(m_thousandths) * dec!(0.001),
            Decimal::from(sigma_thousandths) * dec!(0.001),
        )
        .unwrap();
        let earlier = SviSlice::try_new(parameters, Positive::HUNDRED, positive(dec!(0.5))).unwrap();
        let later = SviSlice::try_new(parameters, positive(dec!(120)), Positive::ONE).unwrap();
        let grid: Vec<Decimal> = k_thousandths
            .iter()
            .map(|&k| Decimal::from(k) * dec!(0.001))
            .collect();
        let report = check_calendar(&[earlier, later], &grid).unwrap();
        prop_assert!(report.passed(), "{:?}", report);
    }
}
