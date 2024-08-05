/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 4/8/24
 ******************************************************************************/

use plotters::prelude::*;

pub fn draw_binomial_tree(
    asset_tree: &Vec<Vec<f64>>,
    option_tree: &Vec<Vec<f64>>,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let steps = asset_tree.len() - 1;
    let max_price = asset_tree.iter().flat_map(|row| row.iter().cloned()).fold(f64::NEG_INFINITY, f64::max);
    let min_price = asset_tree.iter().flat_map(|row| row.iter().cloned()).fold(f64::INFINITY, f64::min);

    let mut chart = ChartBuilder::on(&root)
        .caption("Árbol Binomial", ("sans-serif", 30))
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0f32..steps as f32, min_price..max_price)?;

    chart.configure_mesh().draw()?;

    // Dibujar las líneas del árbol de precios del activo
    for i in 0..steps {
        for j in 0..=i {
            if j < i + 1 {
                chart.draw_series(LineSeries::new(
                    vec![(i as f32, asset_tree[i][j]), ((i + 1) as f32, asset_tree[i + 1][j])],
                    &BLACK,
                ))?;
                chart.draw_series(LineSeries::new(
                    vec![(i as f32, asset_tree[i][j]), ((i + 1) as f32, asset_tree[i + 1][j + 1])],
                    &BLACK,
                ))?;
            }
        }
    }

    // Dibujar los nodos del árbol de precios del activo
    for i in 0..=steps {
        for j in 0..=i {
            chart.draw_series(PointSeries::of_element(
                vec![(i as f32, asset_tree[i][j])],
                5,
                &BLACK,
                &|coord, size, style| {
                    EmptyElement::at(coord)
                        + Circle::new((0, 0), size, style.filled())
                        + Text::new(
                        format!("{:.2}", asset_tree[i][j]),
                        (10, 0),
                        ("sans-serif", 10).into_font(),
                    )
                },
            ))?;
        }
    }

    // Dibujar los nodos del árbol de precios de la opción
    for i in 0..=steps {
        for j in 0..=i {
            chart.draw_series(PointSeries::of_element(
                vec![(i as f32, asset_tree[i][j])],
                5,
                &RED,
                &|coord, size, style| {
                    EmptyElement::at(coord)
                        + Circle::new((0, 0), size, style.filled())
                        + Text::new(
                        format!("{:.2}", option_tree[i][j]),
                        (10, 15),
                        ("sans-serif", 10).into_font(),
                    )
                },
            ))?;
        }
    }

    root.present()?;
    Ok(())
}