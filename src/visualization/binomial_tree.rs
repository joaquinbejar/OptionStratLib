/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 4/8/24
******************************************************************************/

use plotters::prelude::*;
#[cfg(target_arch = "wasm32")]
use plotters_canvas::CanvasBackend;

use super::utils::GraphBackend;

/// This function draws a binomial tree of asset prices and option prices using the plotters library and saves it to a specified file.
///
/// # Arguments
///
/// * `asset_tree` - A reference to a 2D vector containing the asset prices at each node of the binomial tree.
/// * `option_tree` - A reference to a 2D vector containing the option prices at each node of the binomial tree.
/// * `backend` - A GraphBackend object that represents the backend where the chart will be saved.
///
/// # Returns
///
/// This function returns a `Result` which is `Ok` if the drawing and saving process goes fine, otherwise it will return an error inside a `Box`.
///
/// # Errors
///
/// This function may return an error if the drawing or saving process fails.
///
/// # Examples
///
/// ```
/// use optionstratlib::visualization::binomial_tree::draw_binomial_tree;
/// use optionstratlib::visualization::utils::GraphBackend;
/// let asset_tree = vec![
///     vec![100.0],
///     vec![105.0, 95.0],
///     vec![110.25, 99.75, 90.25],
///     vec![115.25, 105.0, 95.0, 85.0],
/// ];
///
/// let option_tree = vec![
///     vec![5.0],
///     vec![10.0, 0.0],
///     vec![15.0, 5.0, 0.0],
///     vec![20.0, 10.0, 0.0, 0.0],
/// ];
///
/// let backend = GraphBackend::Bitmap { file_path: "./Draws/Binomial Tree/binomial_tree.png", size: (1200, 800) };
/// draw_binomial_tree(&asset_tree, &option_tree, backend).unwrap();
/// ```
pub fn draw_binomial_tree(
    asset_tree: &[Vec<f64>],
    option_tree: &[Vec<f64>],
    backend: GraphBackend,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = match backend {
        #[cfg(not(target_arch = "wasm32"))]
        GraphBackend::Bitmap { file_path, size } => {
            let root = BitMapBackend::new(file_path, size).into_drawing_area();
            root.fill(&WHITE)?;
            root
        }
        #[cfg(target_arch = "wasm32")]
        GraphBackend::Canvas { canvas } => {
            let root = CanvasBackend::with_canvas_object(canvas)
                .unwrap()
                .into_drawing_area();
            root.fill(&WHITE)?;
            root
        }
    };
    let steps = asset_tree.len() - 1;
    let max_price = asset_tree
        .iter()
        .flat_map(|row| row.iter().cloned())
        .fold(f64::NEG_INFINITY, f64::max);
    let min_price = asset_tree
        .iter()
        .flat_map(|row| row.iter().cloned())
        .fold(f64::INFINITY, f64::min);

    let mut chart = ChartBuilder::on(&root)
        .caption("Binomial Tree", ("sans-serif", 30))
        .margin(10)
        .top_x_label_area_size(40)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .right_y_label_area_size(60)
        .build_cartesian_2d(
            -0.1f32..(steps as f32 + 0.1),
            min_price - 5.0..max_price + 5.0,
        )?;

    chart
        .configure_mesh()
        .x_labels(steps + 1)
        .y_labels(20)
        .draw()?;

    // Draw the asset price tree lines
    for i in 0..steps {
        for j in 0..=i {
            if j < i + 1 {
                chart.draw_series(LineSeries::new(
                    vec![
                        (i as f32, asset_tree[i][j]),
                        ((i + 1) as f32, asset_tree[i + 1][j]),
                    ],
                    &BLACK,
                ))?;
                chart.draw_series(LineSeries::new(
                    vec![
                        (i as f32, asset_tree[i][j]),
                        ((i + 1) as f32, asset_tree[i + 1][j + 1]),
                    ],
                    &BLACK,
                ))?;
            }
        }
    }

    // Draw the nodes of the asset price tree
    for (i, step_vec) in asset_tree.iter().enumerate().take(steps + 1) {
        for (_j, &value) in step_vec.iter().enumerate().take(i + 1) {
            let x_offset = if i == steps { -30 } else { 0 };
            chart.draw_series(PointSeries::of_element(
                vec![(i as f32, value)],
                5,
                &BLACK,
                &|coord, size, style| {
                    EmptyElement::at(coord)
                        + Circle::new((0, 0), size, style.filled())
                        + Text::new(
                            format!("{:.2}", value),
                            (x_offset, -23),
                            ("sans-serif", 18).into_font(),
                        )
                },
            ))?;
        }
    }

    // Draw the nodes of the option price tree
    for i in 0..=steps {
        for j in 0..=i {
            let x_offset = if i == steps { -30 } else { 0 };
            chart.draw_series(PointSeries::of_element(
                vec![(i as f32, asset_tree[i][j])],
                5,
                &RED,
                &|coord, size, style| {
                    EmptyElement::at(coord)
                        + Circle::new((0, 0), size, style.filled())
                        + Text::new(
                            format!("{:.2}", option_tree[i][j]),
                            (x_offset, 23),
                            ("sans-serif", 18).into_font(),
                        )
                },
            ))?;
        }
    }
    root.present()?;
    Ok(())
}

#[cfg(test)]
mod tests_draw_binomial_tree {
    
    #[cfg(not(target_arch = "wasm32"))]
    use {crate::visualization::utils::GraphBackend,
     crate::visualization::binomial_tree::draw_binomial_tree};

    // Common test data setup
    fn setup_test_data() -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
        let asset_tree = vec![vec![100.0], vec![110.0, 90.0], vec![121.0, 99.0, 81.0]];
        let option_tree = vec![vec![0.0], vec![5.0, 0.0], vec![10.0, 5.0, 0.0]];
        (asset_tree, option_tree)
    }

    // Native-only test
    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_draw_binomial_tree_bitmap() -> Result<(), Box<dyn std::error::Error>> {
        let (asset_tree, option_tree) = setup_test_data();
        let backend = GraphBackend::Bitmap {
            file_path: "./Draws/Binomial Tree/test_output.png",
            size: (1200, 800),
        };

        let result = draw_binomial_tree(&asset_tree, &option_tree, backend);
        assert!(result.is_ok());

        std::fs::remove_file("./Draws/Binomial Tree/test_output.png")?;
        Ok(())
    }
}
