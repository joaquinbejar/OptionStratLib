//! `theta_curve`, `charm_curve` and `color_curve` exist twice on
//! `OptionChain`: as `OptionChainProjections` (analytics) and as the
//! `ThetaCurve`, `CharmCurve` and `ColorCurve` metrics. Until #524 the
//! projection was an inherent method and won every call; now the prelude
//! resolves `chain.theta_curve()` to the metric. This pins that both give the
//! same points, so that switch changes no result.

use optionstratlib_analytics::analytics::OptionChainProjections;
use optionstratlib_analytics::metrics::{CharmCurve, ColorCurve, ThetaCurve};
use optionstratlib_market::chains::chain::OptionChain;
use optionstratlib_math::curves::Curve;

fn sample_chain() -> OptionChain {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/Chains/SP500-18-oct-2024-5781.88.json"
    );
    let mut chain = match OptionChain::load_from_json(path) {
        Ok(chain) => chain,
        Err(error) => panic!("sample chain must load: {error}"),
    };
    chain.set_expiration_date(optionstratlib_core::utils::time::get_x_days_formatted(30));
    chain
}

fn points(curve: Curve) -> Vec<(rust_decimal::Decimal, rust_decimal::Decimal)> {
    curve.points.iter().map(|p| (p.x, p.y)).collect()
}

fn assert_same(
    projection: Result<Curve, impl std::fmt::Display>,
    metric: Result<Curve, impl std::fmt::Display>,
) {
    match (projection, metric) {
        (Ok(a), Ok(b)) => assert_eq!(points(a), points(b)),
        (Err(a), Err(b)) => panic!("both failed: {a} / {b}"),
        (Ok(_), Err(e)) => panic!("metric failed where the projection did not: {e}"),
        (Err(e), Ok(_)) => panic!("projection failed where the metric did not: {e}"),
    }
}

#[test]
fn test_theta_curve_projection_matches_metric() {
    let chain = sample_chain();
    assert_same(
        OptionChainProjections::theta_curve(&chain),
        ThetaCurve::theta_curve(&chain),
    );
}

#[test]
fn test_charm_curve_projection_matches_metric() {
    let chain = sample_chain();
    assert_same(
        OptionChainProjections::charm_curve(&chain),
        CharmCurve::charm_curve(&chain),
    );
}

#[test]
fn test_color_curve_projection_matches_metric() {
    let chain = sample_chain();
    assert_same(
        OptionChainProjections::color_curve(&chain),
        ColorCurve::color_curve(&chain),
    );
}
