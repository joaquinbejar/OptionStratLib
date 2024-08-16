/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 16/8/24
******************************************************************************/
use optionstratlib::volatility::utils::{constant_volatility, ewma_volatility, garch_volatility, historical_volatility, interpolate_volatility_surface, simulate_heston_volatility};

fn main() {
    let returns = vec![0.01, 0.02, -0.01, 0.03, 0.00];

    // Calcula la volatilidad constante
    let const_vol = constant_volatility(&returns);
    println!("Constant Volatility: {:.6}", const_vol);

    // Calcula la volatilidad histórica con una ventana de tamaño 3
    let window_size = 3;
    let hist_vol = historical_volatility(&returns, window_size);
    println!(
        "Historical Volatility (window size {}): {:?}",
        window_size, hist_vol
    );

    // Calcula la volatilidad EWMA con un lambda de 0.94
    let lambda = 0.94;
    let ewma_vol = ewma_volatility(&returns, lambda);
    println!("EWMA Volatility (lambda {}): {:?}", lambda, ewma_vol);

    // Calcula la volatilidad GARCH(1,1)
    let omega = 0.000001;
    let alpha = 0.1;
    let beta = 0.85;
    let garch_vol = garch_volatility(&returns, omega, alpha, beta);
    println!("GARCH(1,1) Volatility: {:?}", garch_vol);

    // Simula la volatilidad usando el modelo de Heston
    let kappa = 0.5;
    let theta = 0.04;
    let xi = 0.1;
    let v0 = 0.04;
    let dt = 1.0 / 252.0;
    let steps = 5;
    let heston_vol = simulate_heston_volatility(kappa, theta, xi, v0, dt, steps);
    println!("Simulated Heston Volatility: {:?}", heston_vol);

    let volatility_surface = vec![
        (100.0, 0.5, 0.2),
        (100.0, 1.0, 0.25),
        (120.0, 0.5, 0.22),
        (120.0, 1.0, 0.28),
    ];

    let strike = 110.0;
    let time_to_expiry = 0.75;

    match interpolate_volatility_surface(strike, time_to_expiry, &volatility_surface) {
        Ok(vol) => println!("Interpolated Volatility: {:.4}", vol),
        Err(e) => println!("Error: {}", e),
    }
}
