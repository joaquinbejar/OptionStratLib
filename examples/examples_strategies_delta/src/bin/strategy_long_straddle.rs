use optionstratlib::f2p;
use optionstratlib::model::types::ExpirationDate;
use optionstratlib::strategies::base::Strategies;
use optionstratlib::strategies::delta_neutral::DeltaNeutrality;
use optionstratlib::strategies::straddle::LongStraddle;
use optionstratlib::strategies::strangle::LongStrangle;
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::visualization::utils::Graph;
use optionstratlib::Positive;
use std::error::Error;
use tracing::info;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let underlying_price = f2p!(7140.0);
    let strategy = LongStraddle::new(
        "CL".to_string(),
        underlying_price, // underlying_price
        f2p!(7202.0),     // call_strike 7450
        ExpirationDate::Days(45.0),
        0.3745,    // implied_volatility
        0.00,      // risk_free_rate
        0.0,       // dividend_yield
        f2p!(1.0), // quantity
        84.2,      // premium_short_call
        353.2,     // premium_short_put
        7.01,      // open_fee_short_call
        7.01,      // close_fee_short_call
        7.01,      // open_fee_short_put
        7.01,      // close_fee_short_put
    );

    info!("Title: {}", strategy.title());
    info!("Break Even Points: {:?}", strategy.break_even_points);
    info!(
        "Net Premium Received: ${:.2}",
        strategy.net_premium_received()?
    );
    info!(
        "Max Profit: ${:.2}",
        strategy.max_profit().unwrap_or(Positive::ZERO)
    );
    info!(
        "Max Loss: ${}",
        strategy.max_loss().unwrap_or(Positive::ZERO)
    );
    info!("Total Fees: ${:.2}", strategy.fees()?);

    info!("Profit Area: {:.2}%", strategy.profit_area()?);
    info!("Profit Ratio: {:.2}%", strategy.profit_ratio()?);

    info!("Delta:  {:#?}", strategy.calculate_net_delta());
    info!("Delta Neutral:  {}", strategy.is_delta_neutral());
    info!(
        "Delta Suggestions:  {:#?}",
        strategy.suggest_delta_adjustments()
    );

    Ok(())
}
