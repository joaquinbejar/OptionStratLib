/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 16/8/24
******************************************************************************/
use optionstratlib::utils::setup_logger;
use optionstratlib::volatility::{
    constant_volatility, ewma_volatility, garch_volatility, historical_volatility,
    simulate_heston_volatility,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::info;

fn main() {
    setup_logger();
    let returns = vec![dec!(0.01), dec!(0.02), dec!(0.01), dec!(0.03), dec!(0.00)];

    let const_vol = constant_volatility(&returns).unwrap();
    info!("Constant Volatility: {:.6}", const_vol);

    let window_size = 3;
    let hist_vol = historical_volatility(&returns, window_size).unwrap();
    info!(
        "Historical Volatility (window size {}): {:?}",
        window_size, hist_vol
    );

    let lambda = dec!(0.94);
    let ewma_vol = ewma_volatility(&returns, lambda).unwrap();
    info!("EWMA Volatility (lambda {}): {:?}", lambda, ewma_vol);

    let omega = dec!(0.000001);
    let alpha = dec!(0.1);
    let beta = dec!(0.85);
    let garch_vol = garch_volatility(&returns, omega, alpha, beta).unwrap();
    info!("GARCH(1,1) Volatility: {:?}", garch_vol);

    let kappa = dec!(0.5);
    let theta = dec!(0.04);
    let xi = dec!(0.1);
    let v0 = dec!(0.04);
    let dt = Decimal::ONE / dec!(252.0);
    let steps = 5;
    let heston_vol = simulate_heston_volatility(kappa, theta, xi, v0, dt, steps).unwrap();
    info!("Simulated Heston Volatility: {:?}", heston_vol);
}
