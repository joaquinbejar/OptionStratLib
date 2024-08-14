use rand::distributions::Distribution;
use statrs::distribution::Normal;

fn monte_carlo_option_pricing(
    s0: f64,           // Precio inicial del activo subyacente
    k: f64,            // Precio de ejercicio de la opción
    r: f64,            // Tasa de interés libre de riesgo
    sigma: f64,        // Volatilidad del activo subyacente
    t: f64,            // Tiempo hasta el vencimiento (en años)
    steps: usize,      // Número de pasos en el tiempo
    simulations: usize // Número de simulaciones de Monte Carlo
) -> f64 {
    let dt = t / steps as f64;
    let mut payoff_sum = 0.0;

    let normal = Normal::new(0.0, 1.0).unwrap();

    let mut rng = rand::thread_rng();

    for _ in 0..simulations {
        let mut st = s0;

        for _ in 0..steps {
            let z = normal.sample(&mut rng);
            st *= 1.0 + r * dt + sigma * z * dt.sqrt();
        }

        // Calcula el payoff para una opción de compra (Call)
        let payoff = f64::max(st - k, 0.0);
        payoff_sum += payoff;
    }

    // Valor promedio de los payoffs descontado al valor presente
    (payoff_sum / simulations as f64) * (-r * t).exp()
}

