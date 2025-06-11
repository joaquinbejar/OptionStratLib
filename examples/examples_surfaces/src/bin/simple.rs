use optionstratlib::error::GraphError;
use optionstratlib::visualization::{Surface3D, make_surface};
use plotly::{ImageFormat, Layout, Plot, common::Title};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::error::Error;
use std::f64::consts::PI;
use std::path::Path;
use tracing::info;

/// Simple test for Surface3D rendering
fn test_simple_surface() -> Result<(), GraphError> {
    info!("Creating simple 3D surface for testing...");

    // Create a simple surface (z = sin(x) * cos(y))
    let n_points = 20; // Grid resolution
    let mut x = Vec::with_capacity(n_points);
    let mut y = Vec::with_capacity(n_points);
    let mut z = Vec::with_capacity(n_points * n_points);

    // Generate x and y coordinates
    for i in 0..n_points {
        let x_val = -5.0 + (i as f64) * 10.0 / (n_points as f64 - 1.0);
        x.push(Decimal::from_f64(x_val).unwrap());
    }

    for i in 0..n_points {
        let y_val = -5.0 + (i as f64) * 10.0 / (n_points as f64 - 1.0);
        y.push(Decimal::from_f64(y_val).unwrap());
    }

    // Generate z values (flatten into a 1D array)
    for j in 0..n_points {
        for i in 0..n_points {
            let x_val = -5.0 + (i as f64) * 10.0 / (n_points as f64 - 1.0);
            let y_val = -5.0 + (j as f64) * 10.0 / (n_points as f64 - 1.0);

            // Simple sinusoidal function
            let z_val = (x_val * PI / 5.0).sin() * (y_val * PI / 5.0).cos();
            z.push(Decimal::from_f64(z_val).unwrap());
        }
    }

    // Create the Surface3D object
    let surface = Surface3D {
        x,
        y,
        z,
        name: "Test Surface".to_string(),
    };

    // Create a plot
    let mut plot = Plot::new();

    // Add the surface trace
    plot.add_trace(make_surface(&surface));

    // Configure layout
    let layout = Layout::new()
        .title(Title::from("Test 3D Surface"))
        .auto_size(true);

    plot.set_layout(layout);

    // Asegurarse de que el directorio existe
    std::fs::create_dir_all("Draws/Surfaces").unwrap_or_else(|e| {
        info!("Error al crear el directorio: {}", e);
    });

    // Save as HTML (should work)
    let html_path = Path::new("Draws/Surfaces/simple_surface.html");
    info!("Saving surface as HTML to: {}", html_path.display());
    plot.write_html(html_path);

    // Save as PNG (should work)
    let png_path = Path::new("Draws/Surfaces/simple_surface.png");
    info!("Saving surface as PNG to: {}", png_path.display());
    plot.write_image(png_path, ImageFormat::PNG, 1600, 900, 1.0);

    // We comment out the generation of static images due to issues with Kaleido
    // en macOS con arquitectura ARM (Apple Silicon)
    info!("Skipping static image generation due to Kaleido issues on macOS ARM");

    // To generate static images, consider using the web version of plotly
    // o exportar manualmente desde el archivo HTML generado

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    test_simple_surface()?;
    Ok(())
}
