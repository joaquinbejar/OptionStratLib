use optionstratlib::chains::chain::OptionChain;
use optionstratlib::curves::BasicCurves;
use optionstratlib::curves::StatisticalCurve;
use optionstratlib::geometrics::MetricsExtractor;
use optionstratlib::model::BasicAxisTypes;
use optionstratlib::simulation::walk::{RandomWalkGraph, Walkable};
use optionstratlib::utils::time::{TimeFrame, get_tomorrow_formatted};
use optionstratlib::utils::{Len, setup_logger};
use optionstratlib::visualization::utils::{Graph, GraphBackend};
use optionstratlib::volatility::AtmIvProvider;
use optionstratlib::{OptionStyle, Side, pos, spos};
use rust_decimal_macros::dec;
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    let years = 3.0;
    let n_steps = 252 * years as usize;
    let mean = 0.0;
    let std_dev_change = pos!(0.001);
    let risk_free_rate = Some(dec!(0.0));
    let dividend_yield = spos!(0.02);
    let volatility_window = 20;

    let mut option_chain =
        OptionChain::load_from_json("examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    option_chain.update_expiration_date(get_tomorrow_formatted());
    let binding = option_chain.clone();
    let atm_iv = binding.atm_iv()?;

    let mut random_walk: RandomWalkGraph<OptionChain> = RandomWalkGraph::new(
        "Random Walk Chain".to_string(),
        risk_free_rate,
        dividend_yield,
        TimeFrame::Day,
        volatility_window,
        *atm_iv,
    );

    let curve = option_chain.curve(&BasicAxisTypes::Volatility, &OptionStyle::Call, &Side::Long)?;
    info!("{}", curve);

    let new_curve = curve.generate_statistical_curve(
        &curve.compute_basic_metrics()?,
        &curve.compute_shape_metrics()?,
        &curve.compute_range_metrics()?,
        &curve.compute_trend_metrics()?,
        curve.len(),
        None,
    )?;
    info!("{}", new_curve);

    info!("{:?}", curve.compute_basic_metrics()?);
    info!("{:?}", curve.compute_shape_metrics()?);
    info!("{:?}", curve.compute_range_metrics()?);

    info!("{}", option_chain);
    random_walk.generate_random_walk(
        n_steps,
        option_chain,
        mean,
        atm_iv.unwrap(),
        std_dev_change,
    )?;

    random_walk.graph(
        &random_walk.get_x_values(),
        GraphBackend::Bitmap {
            file_path: "Draws/Simulation/random_walk_option_chain.png",
            size: (1200, 800),
        },
        20,
    )?;

    Ok(())
}
