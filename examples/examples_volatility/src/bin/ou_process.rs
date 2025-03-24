use optionstratlib::volatility::generate_ou_process;
use optionstratlib::{Positive, pos};
use plotters::prelude::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use std::error::Error;
use tracing::info;
use optionstratlib::utils::setup_logger;


/// Creates multiple OU process simulations and plots them.
///
/// # Parameters
///
/// * `x0` - Initial value
/// * `mu` - Long-term mean
/// * `theta` - Mean reversion rate
/// * `sigma` - Volatility
/// * `dt` - Time step
/// * `steps` - Number of steps to simulate
/// * `num_simulations` - Number of paths to simulate
/// * `output_file` - File path to save the plot
///
/// # Returns
///
/// Result indicating success or failure
pub fn plot_multiple_ou_simulations(
    x0: Positive,
    mu: Positive,
    theta: Positive,
    sigma: Positive,
    dt: Positive,
    steps: usize,
    num_simulations: usize,
    output_file: &str,
) -> Result<(), Box<dyn Error>> {
    // Generate multiple OU processes
    let simulations: Vec<Vec<Positive>> = (0..num_simulations)
        .map(|_| generate_ou_process(x0, mu, theta, sigma, dt, steps))
        .collect();

    // Find y range for better visualization
    let y_max = simulations
        .iter()
        .flat_map(|sim| sim.iter().map(|&value| value.to_f64()))
        .fold(0.0, |max: f64, val| max.max(val))
        * 1.1; // Add 10% margin

    // Create the plotting area
    let root = BitMapBackend::new(output_file, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!(
                "Ornstein-Uhlenbeck Process Simulations (μ={}, θ={}, σ={})",
                mu.to_f64(),
                theta.to_f64(),
                sigma.to_f64()
            ),
            ("sans-serif", 20).into_font(),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(0f64..(steps as f64), 0f64..y_max)?;

    chart
        .configure_mesh()
        .x_labels(20)
        .y_labels(10)
        .x_desc("Time")
        .y_desc("Value")
        .axis_desc_style(("sans-serif", 15))
        .draw()?;

    // Plot each simulation with a different color
    let colors = [
        &BLUE,
        &RED,
        &GREEN,
        &CYAN,
        &MAGENTA,
        &YELLOW,
        &BLACK,
        &RGBColor(128, 0, 128),
        &RGBColor(128, 128, 0),
        &RGBColor(0, 128, 128),
    ];

    for (i, sim) in simulations.iter().enumerate() {
        let color = colors[i % colors.len()];

        let data: Vec<(f64, f64)> = sim
            .iter()
            .enumerate()
            .map(|(j, &value)| (j as f64, value.to_f64()))
            .collect();

        chart.draw_series(LineSeries::new(data, color))?;
    }

    // Draw the mean reversion level with a solid line
    let mean_value = mu.to_f64();

    // Just use a simple line for the mean level
    let mean_line = vec![(0.0, mean_value), (steps as f64, mean_value)];

    chart
        .draw_series(LineSeries::new(mean_line, &BLACK.mix(0.7)))?
        .label(format!("Mean Level (μ={})", mu.to_f64()))
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLACK.mix(0.7)));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .position(SeriesLabelPosition::UpperRight)
        .draw()?;

    root.present()?;

    info!(
        "Multiple simulations plot has been saved to {}",
        output_file
    );
    Ok(())
}

/// Example usage of the plotting functions
pub fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    // Example parameters
    let x0 = pos!(20.0);
    let mu = pos!(20.0);
    let theta = pos!(0.15);
    let sigma = pos!(2.0);
    let dt = pos!(0.1);
    let steps = 1000;

    // Plot multiple simulations
    plot_multiple_ou_simulations(
        x0,
        mu,
        theta,
        sigma,
        dt,
        steps,
        5, // Number of simulations
        "Draws/Simulation/ou_process_multiple.png",
    )?;

    Ok(())
}
