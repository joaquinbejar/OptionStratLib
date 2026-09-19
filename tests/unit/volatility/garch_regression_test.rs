//! Deterministic pin of the property-test failure from #588.
//!
//! With `omega = 1`, `alpha = 1e-28`, `beta = 1` and four unit returns the
//! GARCH variance sequence reaches `4.0000000000000000000000000003` on the
//! third step, an input on which upstream `Decimal::sqrt` aborted with
//! `geo mean circuit breaker`. The unseeded property test
//! (`tests/property/volatility_panic_freedom_test.rs`) only hits it on some
//! runs; this test hits it on every run.

use optionstratlib::volatility::garch_volatility;
use rust_decimal_macros::dec;

#[test]
fn garch_volatility_survives_the_sqrt_circuit_breaker_input() {
    let returns = [dec!(1); 4];
    let result = garch_volatility(
        &returns,
        dec!(1),
        dec!(0.0000000000000000000000000001),
        dec!(1),
    );
    let vols = result.expect("garch_volatility must not fail on this input");
    assert_eq!(vols.len(), returns.len(), "one volatility per return");
    // The last variance is the circuit-breaker input; its root must square
    // back to it within one unit of the 28th decimal.
    let last = vols[3].to_dec();
    let residual = (last * last - dec!(4.0000000000000000000000000003)).abs();
    assert!(
        residual <= dec!(0.0000000000000000000000000001),
        "last volatility {last} squares to residual {residual}"
    );
}
