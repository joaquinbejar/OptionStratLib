//! Fixtures of the static arbitrage checks against references derived without
//! them: Example 3.1 and Theorems 4.1 and 4.2 of Gatheral and Jacquier
//! (arXiv:1204.0646), finite differences of `w`, Black-Scholes butterflies, and
//! closed forms for where slices cross and for `g` beside a zero vertex.

use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::{d_sqrt, decimal_to_f64};
use optionstratlib_pricing::error::{StaticArbitrageError, SviError};
use optionstratlib_pricing::volatility::{
    StaticArbitrageReport, StaticArbitrageViolation, SviParameters, SviSlice, TotalVarianceSlice,
    check_butterfly, check_calendar, check_static_arbitrage,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use statrs::distribution::{ContinuousCDF, Normal};

fn pos(value: Decimal) -> Positive {
    Positive::new_decimal(value).unwrap()
}

fn slice(a: Decimal, b: Decimal, rho: Decimal, m: Decimal, sigma: Decimal) -> SviSlice {
    SviSlice::try_new(
        SviParameters::try_new(a, b, rho, m, sigma).unwrap(),
        Positive::HUNDRED,
        Positive::ONE,
    )
    .unwrap()
}

fn with_forward_and_expiry(source: SviSlice, forward: Decimal, expiry: Decimal) -> SviSlice {
    SviSlice::try_new(source.parameters(), pos(forward), pos(expiry)).unwrap()
}

/// A flat slice of constant total variance, whose `g` is exactly one.
fn flat(w: Decimal, expiry: Decimal) -> SviSlice {
    with_forward_and_expiry(
        slice(w, dec!(0), dec!(0), dec!(0), dec!(0.1)),
        dec!(100),
        expiry,
    )
}

/// Example 3.1 of the paper, equation (3.8): `(a, b, m, rho, sigma)` at `t = 1`.
fn vogt() -> SviSlice {
    slice(
        dec!(-0.0410),
        dec!(0.1331),
        dec!(0.3060),
        dec!(0.3586),
        dec!(0.4153),
    )
}

/// The same `w` as [`vogt`], in `f64` and without the code under test.
fn vogt_w(k: f64) -> f64 {
    let (a, b, rho, m, sigma) = (-0.0410, 0.1331, 0.3060, 0.3586, 0.4153);
    let x = k - m;
    a + b * (rho * x + (x * x + sigma * sigma).sqrt())
}

/// `g` of equation (2.1) from central finite differences of `w`.
fn g_by_finite_differences(w: impl Fn(f64) -> f64, k: f64) -> f64 {
    let h = 1e-4;
    let (below, here, above) = (w(k - h), w(k), w(k + h));
    let slope = (above - below) / (2.0 * h);
    let curvature = (above - 2.0 * here + below) / (h * h);
    let first = 1.0 - k * slope / (2.0 * here);
    first * first - (slope * slope / 4.0) * (1.0 / here + 0.25) + curvature / 2.0
}

/// A grid from `first` to `last`, inclusive.
fn grid(first: Decimal, last: Decimal, step: Decimal) -> Vec<Decimal> {
    let mut points = Vec::new();
    let mut k = first;
    while k <= last {
        points.push(k);
        k += step;
    }
    points
}

fn butterfly_points(report: &StaticArbitrageReport) -> Vec<Decimal> {
    report
        .violations()
        .iter()
        .filter(|violation| matches!(violation, StaticArbitrageViolation::Butterfly { .. }))
        .map(StaticArbitrageViolation::log_moneyness)
        .collect()
}

fn calendar_points(report: &StaticArbitrageReport) -> Vec<Decimal> {
    report
        .violations()
        .iter()
        .filter(|violation| matches!(violation, StaticArbitrageViolation::Calendar { .. }))
        .map(StaticArbitrageViolation::log_moneyness)
        .collect()
}

fn g_of(violation: &StaticArbitrageViolation) -> Decimal {
    match violation {
        StaticArbitrageViolation::Butterfly { g, .. } => *g,
        other => panic!("expected a butterfly violation, got {other:?}"),
    }
}

#[test]
fn test_butterfly_vogt_negative_region_matches_finite_differences() {
    // In the paper's Figure 1, g is negative over part of the right wing and
    // reaches about -0.03 near k = 1: read the same from finite differences.
    let report = check_butterfly(&vogt(), &grid(dec!(-1.5), dec!(1.5), dec!(0.05))).unwrap();
    assert!(report.unevaluated().is_empty());
    for k_tenths in -30..=30 {
        let k = Decimal::from(k_tenths) * dec!(0.05);
        let expected = g_by_finite_differences(vogt_w, decimal_to_f64(k).unwrap());
        let found = report
            .violations()
            .iter()
            .find(|violation| violation.log_moneyness() == k);
        match found {
            Some(violation) => {
                assert!(expected < 0.0, "k = {k}, expected g = {expected}");
                let g = decimal_to_f64(g_of(violation)).unwrap();
                assert!((g - expected).abs() < 1e-6, "k = {k}: {g} vs {expected}");
            }
            None => assert!(expected > 0.0, "k = {k}, expected g = {expected}"),
        }
    }
    let deepest = report
        .violations()
        .iter()
        .map(|violation| violation.size())
        .max()
        .unwrap();
    assert!(
        deepest > pos(dec!(0.03)) && deepest < pos(dec!(0.035)),
        "{deepest}"
    );
}

#[test]
fn test_butterfly_vogt_g_matches_references_at_28_places() {
    // From finite differences of `w` with 80 significant digits and a step of
    // 1e-15, in a script that shares no code with this crate.
    let references = [
        (dec!(0.65), dec!(-0.0022972539615039763910760180)),
        (dec!(0.7), dec!(-0.0151259196280025064243815516)),
        (dec!(0.8), dec!(-0.0298184615476026588696208566)),
        (dec!(0.9), dec!(-0.0326851307090228998618288294)),
        (dec!(1.0), dec!(-0.0277416959046262877230504685)),
        (dec!(1.1), dec!(-0.0183863844951939256526788320)),
        (dec!(1.2), dec!(-0.0069151818382860806869150431)),
        (dec!(1.25), dec!(-0.0008461613158293710208153204)),
    ];
    let points: Vec<Decimal> = references.iter().map(|(k, _)| *k).collect();
    let report = check_butterfly(&vogt(), &points).unwrap();
    assert_eq!(report.violations().len(), references.len());
    for (violation, (k, expected)) in report.violations().iter().zip(references) {
        assert_eq!(violation.log_moneyness(), k);
        let difference = (g_of(violation) - expected).abs();
        assert!(
            difference < dec!(0.000000000000000000000001),
            "k = {k}: {difference}"
        );
        assert_eq!(violation.size(), pos(-g_of(violation)));
    }
}

#[test]
fn test_butterfly_vogt_negative_region_is_bracketed_at_both_ends() {
    // g crosses zero at k = 0.64240787... and at k = 1.25691299..., by bisection
    // on those finite differences.
    let points = [dec!(0.6424), dec!(0.6425), dec!(1.2569), dec!(1.2570)];
    let report = check_butterfly(&vogt(), &points).unwrap();
    assert_eq!(butterfly_points(&report), vec![dec!(0.6425), dec!(1.2569)]);
    assert!(report.unevaluated().is_empty());

    let report = check_butterfly(&vogt(), &grid(dec!(0.6), dec!(1.4), dec!(0.01))).unwrap();
    assert_eq!(
        butterfly_points(&report),
        grid(dec!(0.65), dec!(1.25), dec!(0.01))
    );
}

#[test]
fn test_butterfly_vogt_negative_g_is_a_butterfly_with_a_negative_price() {
    // A second opinion that never sees `g`: a long butterfly of Black-Scholes
    // calls at the implied volatility of the slice costs less than nothing
    // exactly where g is negative.
    let normal = Normal::new(0.0, 1.0).unwrap();
    let forward = 100.0_f64;
    let call = |strike: f64| {
        let k = (strike / forward).ln();
        let root = vogt_w(k).sqrt();
        let d_plus = -k / root + root / 2.0;
        forward * normal.cdf(d_plus) - strike * normal.cdf(d_plus - root)
    };
    let butterfly = |k: f64| {
        let strike = forward * k.exp();
        let wing = 2.0;
        call(strike - wing) - 2.0 * call(strike) + call(strike + wing)
    };
    for (k, reported) in [
        (dec!(0.2), false),
        (dec!(0.6), false),
        (dec!(0.9), true),
        (dec!(1.0), true),
        (dec!(1.4), false),
    ] {
        let report = check_butterfly(&vogt(), &[k]).unwrap();
        let price = butterfly(decimal_to_f64(k).unwrap());
        assert_eq!(report.violations().len(), usize::from(reported), "k = {k}");
        assert_eq!(price < 0.0, reported, "k = {k}, butterfly = {price}");
    }
}

#[test]
fn test_butterfly_violation_names_expiry_and_the_strike_of_its_forward() {
    let shifted = with_forward_and_expiry(vogt(), dec!(250), dec!(1));
    let report = check_butterfly(&shifted, &[dec!(0.9)]).unwrap();
    match report.violations() {
        [
            StaticArbitrageViolation::Butterfly {
                time_to_expiry,
                log_moneyness,
                strike,
                ..
            },
        ] => {
            assert_eq!(*time_to_expiry, Positive::ONE);
            assert_eq!(*log_moneyness, dec!(0.9));
            // 250 * e^0.9 = 614.9007777892374.
            let difference = (strike.to_dec() - dec!(614.9007777892374)).abs();
            assert!(difference < dec!(0.0000000001), "{strike}");
        }
        other => panic!("expected one violation, got {other:?}"),
    }
}

#[test]
fn test_butterfly_coarse_grid_misses_a_violation_a_finer_one_finds() {
    // This is the claim of the docs: passing says nothing between grid points.
    let coarse = [dec!(-1), dec!(0), dec!(0.5), dec!(1.5), dec!(2)];
    let report = check_butterfly(&vogt(), &coarse).unwrap();
    assert!(report.passed(), "{report:?}");
    assert!(report.violations().is_empty());

    let fine = grid(dec!(-1), dec!(2), dec!(0.1));
    assert!(!check_butterfly(&vogt(), &fine).unwrap().passed());
}

/// The crossing fixture: with `rho = 0` and `m = 0`, `w` is `a + b * S` with
/// `S = sqrt(k^2 + sigma^2)`, so the slices meet at `S = (a2 - a1) / (b1 - b2)`,
/// here `0.4`, at `k = +-sqrt(0.16 - 0.04) = +-0.34641016...`. Beyond that the
/// earlier expiry has the steeper wing and more total variance.
fn crossing() -> [SviSlice; 2] {
    let earlier = slice(dec!(0.01), dec!(0.2), dec!(0), dec!(0), dec!(0.2));
    let later = slice(dec!(0.05), dec!(0.1), dec!(0), dec!(0), dec!(0.2));
    [
        with_forward_and_expiry(earlier, dec!(100), dec!(0.5)),
        with_forward_and_expiry(later, dec!(110), dec!(1)),
    ]
}

#[test]
fn test_calendar_crossing_is_located_where_the_closed_form_says() {
    let points = [
        dec!(-0.3465),
        dec!(-0.3464),
        dec!(0),
        dec!(0.3464),
        dec!(0.3465),
        dec!(1),
    ];
    let report = check_calendar(&crossing(), &points).unwrap();
    assert_eq!(
        calendar_points(&report),
        vec![dec!(-0.3465), dec!(0.3465), dec!(1)]
    );
    assert!(report.unevaluated().is_empty());
}

#[test]
fn test_calendar_violation_names_both_expiries_both_strikes_and_its_size() {
    // At k = 0.375, S = 0.425 exactly: w1 = 0.095, w2 = 0.0925, a decrease of
    // 0.0025. The same k is a different strike on each forward.
    let report = check_calendar(&crossing(), &[dec!(0.375)]).unwrap();
    match report.violations() {
        [
            StaticArbitrageViolation::Calendar {
                earlier_expiry,
                later_expiry,
                log_moneyness,
                earlier_strike,
                later_strike,
                earlier_total_variance,
                later_total_variance,
                size,
                ..
            },
        ] => {
            assert_eq!(*earlier_expiry, pos(dec!(0.5)));
            assert_eq!(*later_expiry, Positive::ONE);
            assert_eq!(*log_moneyness, dec!(0.375));
            assert_eq!(*earlier_total_variance, pos(dec!(0.095)));
            assert_eq!(*later_total_variance, pos(dec!(0.0925)));
            assert_eq!(*size, pos(dec!(0.0025)));
            // 100 * e^0.375 and 110 * e^0.375.
            let tolerance = dec!(0.0000000001);
            assert!((earlier_strike.to_dec() - dec!(145.49914146182013)).abs() < tolerance);
            assert!((later_strike.to_dec() - dec!(160.04905560800213)).abs() < tolerance);
        }
        other => panic!("expected one violation, got {other:?}"),
    }
}

#[test]
fn test_calendar_asymmetric_crossing_matches_the_quadratic_roots() {
    // With a shared skew and shift, S = rho * x + sqrt(x^2 + sigma^2) = s has the
    // roots of (1 - rho^2) * x^2 + 2 * s * rho * x + sigma^2 - s^2 = 0.
    let (rho, m, sigma) = (-0.3_f64, 0.1_f64, 0.2_f64);
    let s = 0.4_f64;
    let room = 1.0 - rho * rho;
    let discriminant = (s * s * rho * rho + room * (s * s - sigma * sigma)).sqrt();
    let low = m + (-s * rho - discriminant) / room;
    let high = m + (-s * rho + discriminant) / room;

    let earlier = slice(dec!(0.01), dec!(0.2), dec!(-0.3), dec!(0.1), dec!(0.2));
    let later = slice(dec!(0.05), dec!(0.1), dec!(-0.3), dec!(0.1), dec!(0.2));
    let slices = [
        with_forward_and_expiry(earlier, dec!(100), dec!(0.5)),
        with_forward_and_expiry(later, dec!(100), dec!(1)),
    ];
    let beside = |k: f64| Decimal::try_from(k).unwrap();
    let step = 1e-6;
    let points = [
        beside(low - step),
        beside(low + step),
        beside(high - step),
        beside(high + step),
    ];
    let report = check_calendar(&slices, &points).unwrap();
    assert_eq!(calendar_points(&report), vec![points[0], points[3]]);
}

#[test]
fn test_calendar_compares_adjacent_expiries_in_the_right_direction() {
    // 0.01, 0.04, 0.03: only the last step decreases, and the first expiry is
    // below the last, so comparing non-adjacent expiries would miss it.
    let slices = [
        flat(dec!(0.01), dec!(0.25)),
        flat(dec!(0.04), dec!(0.5)),
        flat(dec!(0.03), dec!(1)),
    ];
    let report = check_calendar(&slices, &[dec!(0)]).unwrap();
    match report.violations() {
        [
            StaticArbitrageViolation::Calendar {
                earlier_expiry,
                later_expiry,
                size,
                ..
            },
        ] => {
            assert_eq!(*earlier_expiry, pos(dec!(0.5)));
            assert_eq!(*later_expiry, Positive::ONE);
            assert_eq!(*size, pos(dec!(0.01)));
        }
        other => panic!("expected one violation, got {other:?}"),
    }
}

#[test]
fn test_calendar_increasing_variance_passes_whatever_the_input_order() {
    let increasing = [
        flat(dec!(0.01), dec!(0.25)),
        flat(dec!(0.02), dec!(0.5)),
        flat(dec!(0.03), dec!(1)),
    ];
    assert!(check_calendar(&increasing, &[dec!(0)]).unwrap().passed());

    let shuffled = [increasing[2], increasing[0], increasing[1]];
    assert!(check_calendar(&shuffled, &[dec!(0)]).unwrap().passed());

    let violating = [
        flat(dec!(0.01), dec!(0.25)),
        flat(dec!(0.04), dec!(0.5)),
        flat(dec!(0.03), dec!(1)),
    ];
    let shuffled = [violating[1], violating[2], violating[0]];
    assert_eq!(
        check_calendar(&shuffled, &[dec!(0)]).unwrap(),
        check_calendar(&violating, &[dec!(0)]).unwrap()
    );
}

#[test]
fn test_calendar_equal_total_variance_is_not_a_violation() {
    let same = vogt();
    let slices = [
        with_forward_and_expiry(same, dec!(100), dec!(0.5)),
        with_forward_and_expiry(same, dec!(120), dec!(1)),
    ];
    let report = check_calendar(&slices, &grid(dec!(-2), dec!(2), dec!(0.1))).unwrap();
    assert!(report.passed(), "{report:?}");

    let report = check_calendar(
        &[flat(dec!(0.04), dec!(0.5)), flat(dec!(0.04), dec!(1))],
        &[dec!(-1), dec!(0), dec!(1)],
    )
    .unwrap();
    assert!(report.passed(), "{report:?}");
}

#[test]
fn test_calendar_rounding_band_of_svi_is_pinned_from_both_sides() {
    // At a = 1 and b = 0, each slice declares 1e-24 + 4e-28, so the band is
    // about 2e-24: a decrease of 1e-24 is rounding, and one of 3e-24 is reported.
    let later = dec!(1);
    let report = check_calendar(
        &[
            flat(later + dec!(0.000000000000000000000001), dec!(0.5)),
            flat(later, dec!(1)),
        ],
        &[dec!(0)],
    )
    .unwrap();
    assert!(report.passed(), "{report:?}");

    let report = check_calendar(
        &[
            flat(later + dec!(0.000000000000000000000003), dec!(0.5)),
            flat(later, dec!(1)),
        ],
        &[dec!(0)],
    )
    .unwrap();
    assert_eq!(report.violations().len(), 1, "{report:?}");
    assert_eq!(
        report.violations()[0].size(),
        pos(dec!(0.000000000000000000000003))
    );
}

#[test]
fn test_calendar_noise_at_the_last_place_is_not_a_violation() {
    // 1/3 at 28 places, and the same a unit below it.
    let third = Decimal::ONE / dec!(3);
    let below = third - Decimal::new(1, 28);
    let report = check_calendar(
        &[flat(third, dec!(0.5)), flat(below, dec!(1))],
        &grid(dec!(-1), dec!(1), dec!(0.25)),
    )
    .unwrap();
    assert!(report.passed(), "{report:?}");
}

#[test]
fn test_static_arbitrage_uncheckable_input_is_a_typed_error() {
    let slices = crossing();
    assert!(matches!(
        check_butterfly(&vogt(), &[]),
        Err(StaticArbitrageError::EmptyGrid)
    ));
    assert!(matches!(
        check_calendar(&slices, &[]),
        Err(StaticArbitrageError::EmptyGrid)
    ));
    assert!(matches!(
        check_static_arbitrage(&slices, &[]),
        Err(StaticArbitrageError::EmptyGrid)
    ));
    for few in [&slices[..1], &slices[..0]] {
        assert!(matches!(
            check_calendar(few, &[dec!(0)]),
            Err(StaticArbitrageError::TooFewSlices { required: 2, .. })
        ));
        assert!(matches!(
            check_static_arbitrage(few, &[dec!(0)]),
            Err(StaticArbitrageError::TooFewSlices { required: 2, .. })
        ));
    }
}

#[test]
fn test_static_arbitrage_duplicate_expiry_is_a_typed_error() {
    // Whether the variances differ or agree, and wherever the two slices sit.
    let different = [
        flat(dec!(0.04), dec!(1)),
        flat(dec!(0.01), dec!(0.5)),
        flat(dec!(0.05), dec!(1)),
    ];
    let equal = [flat(dec!(0.04), dec!(1)), flat(dec!(0.04), dec!(1.00))];
    assert!(matches!(
        check_calendar(&different, &[dec!(0)]),
        Err(StaticArbitrageError::DuplicateExpiry { time_to_expiry }) if time_to_expiry == Positive::ONE
    ));
    assert!(matches!(
        check_calendar(&equal, &[dec!(0)]),
        Err(StaticArbitrageError::DuplicateExpiry { .. })
    ));
    assert!(matches!(
        check_static_arbitrage(&different, &[dec!(0)]),
        Err(StaticArbitrageError::DuplicateExpiry { .. })
    ));
}

/// `g` beside the vertex of `a = -b * sigma`, `rho = 0`, `m = 0`, in closed form:
/// with `R = sqrt(k^2 + sigma^2)` and `R - sigma = k^2 / (R + sigma)`,
/// `4 * R^3 * g = (R - sigma) * (R * (R - sigma) - b * (R + 2 * sigma) - b^2 * (R + sigma) * R / 4)`.
fn zero_vertex_g(k: Decimal, b: Decimal, sigma: Decimal) -> Decimal {
    let root = d_sqrt(k * k + sigma * sigma, "test").unwrap();
    let above = k * k / (root + sigma);
    let bracket =
        root * above - b * (root + dec!(2) * sigma) - b * b * (root + sigma) * root / dec!(4);
    above * bracket / (dec!(4) * root * root * root)
}

#[test]
fn test_butterfly_at_a_zero_vertex_is_unevaluated_not_a_violation_or_a_pass() {
    // a + b * sigma = 0 exactly, so w(0) is exactly zero and w'(0) is too.
    let touching = slice(dec!(-0.01), dec!(0.1), dec!(0), dec!(0), dec!(0.1));
    assert_eq!(touching.total_variance(dec!(0)).unwrap(), Positive::ZERO);
    assert_eq!(touching.total_variance_slope(dec!(0)).unwrap(), dec!(0));

    let report = check_butterfly(&touching, &[dec!(0)]).unwrap();
    assert!(report.violations().is_empty(), "{report:?}");
    assert!(!report.passed());
    match report.unevaluated() {
        [point] => {
            assert_eq!(point.time_to_expiry, Positive::ONE);
            assert_eq!(point.log_moneyness, dec!(0));
            assert_eq!(point.strike, Positive::HUNDRED);
            assert_eq!(point.total_variance, Positive::ZERO);
            assert!(point.rounding > Positive::ZERO);
        }
        other => panic!("expected one unevaluated point, got {other:?}"),
    }
}

#[test]
fn test_butterfly_at_a_rounded_zero_vertex_is_unevaluated() {
    // With rho != 0 the vertex k* = m - rho * sigma / sqrt(1 - rho^2) and the
    // floor of a are both rounded, so w(k*) may be zero or a few units above it.
    let (b, rho, m, sigma) = (dec!(0.1), dec!(-0.5), dec!(0.05), dec!(0.2));
    let room = d_sqrt(Decimal::ONE - rho * rho, "test").unwrap();
    let a = -(b * sigma * room);
    let vertex = m - rho * sigma / room;
    let touching = slice(a, b, rho, m, sigma);
    let at_vertex = touching.total_variance(vertex).unwrap();
    assert!(
        at_vertex < pos(dec!(0.000000000000000000000000001)),
        "{at_vertex}"
    );

    let report = check_butterfly(&touching, &[vertex]).unwrap();
    assert!(report.violations().is_empty(), "{report:?}");
    assert_eq!(report.unevaluated().len(), 1, "{report:?}");
}

#[test]
fn test_butterfly_beside_a_zero_vertex_at_the_money_matches_the_closed_form() {
    // The true g is negative beside this vertex, so every point is a violation
    // of the size the closed form gives or, once w is too small to resolve, an
    // unevaluated point. None of them passes, and none has a wild value.
    let (b, sigma) = (dec!(0.1), dec!(0.1));
    let touching = slice(-(b * sigma), b, dec!(0), dec!(0), sigma);
    let evaluated = [
        dec!(0.1),
        dec!(0.01),
        dec!(0.001),
        dec!(0.0001),
        dec!(0.00001),
        dec!(0.000001),
    ];
    let unresolved = [
        dec!(0.0000001),
        dec!(0.00000001),
        dec!(0.0000000001),
        dec!(0.00000000000001),
    ];
    for k in evaluated.into_iter().flat_map(|k| [k, -k]) {
        let report = check_butterfly(&touching, &[k]).unwrap();
        assert!(report.unevaluated().is_empty(), "k = {k}: {report:?}");
        let [violation] = report.violations() else {
            panic!("k = {k}: expected a violation, got {report:?}");
        };
        let expected = zero_vertex_g(k, b, sigma);
        assert!(expected < dec!(0), "k = {k}: {expected}");
        let relative = ((g_of(violation) - expected) / expected).abs();
        assert!(
            relative < dec!(0.0001),
            "k = {k}: {} vs {expected}",
            g_of(violation)
        );
    }
    for k in unresolved.into_iter().flat_map(|k| [k, -k]) {
        let report = check_butterfly(&touching, &[k]).unwrap();
        assert!(report.violations().is_empty(), "k = {k}: {report:?}");
        assert_eq!(report.unevaluated().len(), 1, "k = {k}: {report:?}");
    }
}

#[test]
fn test_butterfly_zero_vertex_away_from_the_money_shows_in_the_wing() {
    // The docs say a zero vertex at k* != 0 has g tending to +inf beside it, and
    // shows up as a negative g only further out. At k* = 0.5, b = 0.1, sigma = 0.1.
    let touching = slice(dec!(-0.01), dec!(0.1), dec!(0), dec!(0.5), dec!(0.1));
    let w = |k: f64| {
        let x = k - 0.5;
        -0.01 + 0.1 * (x * x + 0.01).sqrt()
    };

    let at_vertex = check_butterfly(&touching, &[dec!(0.5)]).unwrap();
    assert!(at_vertex.violations().is_empty());
    assert_eq!(at_vertex.unevaluated().len(), 1);

    let beside = check_butterfly(&touching, &[dec!(0.49999), dec!(0.50001), dec!(0.51)]).unwrap();
    assert!(beside.passed(), "{beside:?}");

    let wing = check_butterfly(&touching, &[dec!(1.3)]).unwrap();
    let [violation] = wing.violations() else {
        panic!("expected a violation in the wing, got {wing:?}");
    };
    let expected = g_by_finite_differences(w, 1.3);
    assert!(expected < -0.02, "{expected}");
    let g = decimal_to_f64(g_of(violation)).unwrap();
    assert!((g - expected).abs() < 1e-6, "{g} vs {expected}");
}

#[test]
fn test_butterfly_zero_total_variance_everywhere_is_unevaluated_everywhere() {
    let nothing = slice(dec!(0), dec!(0), dec!(0), dec!(0), dec!(0.1));
    let points = [dec!(-1), dec!(0), dec!(1)];
    let report = check_butterfly(&nothing, &points).unwrap();
    assert!(report.violations().is_empty());
    assert_eq!(report.unevaluated().len(), points.len());
    assert!(!report.passed());
}

#[test]
fn test_butterfly_flat_slice_has_no_noise_to_report() {
    let report = check_butterfly(
        &flat(dec!(0.04), dec!(1)),
        &grid(dec!(-3), dec!(3), dec!(0.1)),
    );
    assert!(report.unwrap().passed());
}

/// An SSVI surface in raw parameters, by (3.3) with `Delta = mu = 0`, `omega =
/// theta`, `zeta = phi`, with the power law of Example 4.2 at `gamma = 1/2`:
/// `phi = eta / sqrt(theta)`.
fn ssvi(theta: Decimal, rho: Decimal, eta: Decimal, forward: Decimal, expiry: Decimal) -> SviSlice {
    let phi = eta / d_sqrt(theta, "test").unwrap();
    let room = d_sqrt(Decimal::ONE - rho * rho, "test").unwrap();
    with_forward_and_expiry(
        slice(
            theta / dec!(2) * (Decimal::ONE - rho * rho),
            theta * phi / dec!(2),
            rho,
            -rho / phi,
            room / phi,
        ),
        forward,
        expiry,
    )
}

#[test]
fn test_static_arbitrage_ssvi_surface_passes_with_an_empty_report() {
    // Example 4.2 meets Theorem 4.1 for any eta: the calendar condition holds.
    // Theorem 4.2 needs theta * phi * (1 + |rho|) < 4 and theta * phi^2 * (1 +
    // |rho|) <= 4: at eta = 1 and rho = -0.4, sqrt(0.32) * 1.4 < 4 and 1.4 <= 4.
    // With theta increasing in the expiry the theorems leave no violation to find.
    let (rho, eta) = (dec!(-0.4), dec!(1));
    let surface = [
        ssvi(dec!(0.02), rho, eta, dec!(100), dec!(0.1)),
        ssvi(dec!(0.04), rho, eta, dec!(101), dec!(0.25)),
        ssvi(dec!(0.08), rho, eta, dec!(103), dec!(0.5)),
        ssvi(dec!(0.16), rho, eta, dec!(106), dec!(1)),
        ssvi(dec!(0.32), rho, eta, dec!(112), dec!(2)),
    ];
    let points = grid(dec!(-3), dec!(3), dec!(0.025));
    let report = check_static_arbitrage(&surface, &points).unwrap();
    assert!(report.violations().is_empty(), "{report:?}");
    assert!(report.unevaluated().is_empty(), "{report:?}");
    assert!(report.passed());
}

#[test]
fn test_static_arbitrage_surface_reports_both_kinds_in_order() {
    let [earlier, _] = crossing();
    let surface = [
        with_forward_and_expiry(vogt(), dec!(100), dec!(0.25)),
        earlier,
    ];
    // Vogt has g < 0 at k = 0.9, with less total variance than the later slice.
    let report = check_static_arbitrage(&surface, &[dec!(0.9)]).unwrap();
    assert_eq!(butterfly_points(&report), vec![dec!(0.9)]);
    assert!(calendar_points(&report).is_empty());

    let [_, later] = crossing();
    let surface = [
        with_forward_and_expiry(vogt(), dec!(100), dec!(0.25)),
        with_forward_and_expiry(later, dec!(100), dec!(0.5)),
    ];
    let report = check_static_arbitrage(&surface, &[dec!(0.9), dec!(2)]).unwrap();
    // Vogt wins at k = 2: 0.2512 against 0.05 + 0.1 * sqrt(4.04) = 0.2510.
    assert_eq!(butterfly_points(&report), vec![dec!(0.9)]);
    assert_eq!(calendar_points(&report), vec![dec!(2)]);
    assert!(matches!(
        report.violations()[0],
        StaticArbitrageViolation::Butterfly { .. }
    ));
    assert!(matches!(
        report.violations()[1],
        StaticArbitrageViolation::Calendar { .. }
    ));
}

#[test]
fn test_static_arbitrage_surface_keeps_an_unevaluated_point() {
    let touching = with_forward_and_expiry(
        slice(dec!(-0.01), dec!(0.1), dec!(0), dec!(0), dec!(0.1)),
        dec!(100),
        dec!(0.5),
    );
    let report = check_static_arbitrage(&[touching, flat(dec!(1), dec!(1))], &[dec!(0)]).unwrap();
    assert!(report.violations().is_empty(), "{report:?}");
    assert_eq!(report.unevaluated().len(), 1);
    assert!(!report.passed());
}

#[test]
fn test_static_arbitrage_overflow_at_extreme_parameters_is_a_typed_error() {
    for k in [Decimal::MAX, Decimal::MIN, dec!(1000000000000000)] {
        assert!(matches!(
            check_butterfly(&vogt(), &[k]),
            Err(StaticArbitrageError::Svi(SviError::Decimal(_)))
        ));
        assert!(matches!(
            check_calendar(&crossing(), &[k]),
            Err(StaticArbitrageError::Svi(SviError::Decimal(_)))
        ));
    }
    // A violation with no strike to name is an error too, not a lost finding.
    let flat_wing = [flat(dec!(0.02), dec!(0.5)), flat(dec!(0.01), dec!(1))];
    assert!(matches!(
        check_calendar(&flat_wing, &[dec!(100)]),
        Err(StaticArbitrageError::StrikeOutOfRange { .. })
    ));
    assert!(matches!(
        check_calendar(&flat_wing, &[dec!(-100)]),
        Err(StaticArbitrageError::StrikeOutOfRange { .. })
    ));
}

/// A slice from outside the crate, which fails at one point.
#[derive(Debug, Clone, Copy)]
struct Quadratic {
    expiry: Positive,
    fails_at: Decimal,
}

impl TotalVarianceSlice for Quadratic {
    fn time_to_expiry(&self) -> Positive {
        self.expiry
    }
    fn forward(&self) -> Positive {
        Positive::HUNDRED
    }
    fn total_variance(&self, k: Decimal) -> Result<Positive, StaticArbitrageError> {
        if k == self.fails_at {
            return Err(StaticArbitrageError::SliceEvaluation {
                log_moneyness: k,
                reason: "no quote here".to_string(),
            });
        }
        Ok(pos(dec!(0.04) + k * k))
    }
    fn total_variance_slope(&self, k: Decimal) -> Result<Decimal, StaticArbitrageError> {
        Ok(dec!(2) * k)
    }
    fn total_variance_curvature(&self, _k: Decimal) -> Result<Decimal, StaticArbitrageError> {
        Ok(dec!(2))
    }
    fn total_variance_rounding(&self, _k: Decimal) -> Result<Positive, StaticArbitrageError> {
        Ok(Positive::ZERO)
    }
}

#[test]
fn test_total_variance_slice_can_be_implemented_outside_the_crate() {
    let quadratic = Quadratic {
        expiry: Positive::ONE,
        fails_at: dec!(100),
    };
    // g(0) = 1 + 2 / 2 = 2: no violation at the money, with w' = 0.
    assert!(check_butterfly(&quadratic, &[dec!(0)]).unwrap().passed());
    // A point the slice cannot evaluate fails the check rather than skip it.
    assert!(matches!(
        check_butterfly(&quadratic, &[dec!(0), dec!(100)]),
        Err(StaticArbitrageError::SliceEvaluation { .. })
    ));
}
