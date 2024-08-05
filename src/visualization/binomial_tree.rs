/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 4/8/24
******************************************************************************/

use plotters::prelude::*;

/// This function draws a binomial tree of asset prices and option prices using the plotters library and saves it to a specified file.
///
/// # Arguments
///
/// * `asset_tree` - A reference to a 2D vector containing the asset prices at each node of the binomial tree.
/// * `option_tree` - A reference to a 2D vector containing the option prices at each node of the binomial tree.
/// * `filename` - A string slice that represents the name of the file where the chart will be saved.
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
/// let asset_tree = vec![
///     vec![100.0],
///     vec![105.0, 95.0],
///     vec![110.25, 99.75, 90.25],
/// ];
///
/// let option_tree = vec![
///     vec![5.0],
///     vec![10.0, 0.0],
///     vec![15.0, 5.0, 0.0],
/// ];
///
/// let filename = "./Draws/Binomial Tree/binomial_tree.png";
/// draw_binomial_tree(&asset_tree, &option_tree, filename).unwrap();
/// ```
pub fn draw_binomial_tree(
    asset_tree: &[Vec<f64>],
    option_tree: &[Vec<f64>],
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
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
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0f32..steps as f32, min_price..max_price)?;
    chart.configure_mesh().draw()?;
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
            chart.draw_series(PointSeries::of_element(
                vec![(i as f32, value)],
                5,
                &BLACK,
                &|coord, size, style| {
                    EmptyElement::at(coord)
                        + Circle::new((0, 0), size, style.filled())
                        + Text::new(
                            format!("{:.2}", value),
                            (10, 0),
                            ("sans-serif", 10).into_font(),
                        )
                },
            ))?;
        }
    }

    // Draw the nodes of the option price tree
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

#[cfg(test)]
mod tests_draw_binomial_tree {
    use super::*;
    use std::fs;

    #[test]
    fn test_draw_binomial_tree_success() {
        let asset_tree = vec![vec![100.0], vec![110.0, 90.0], vec![121.0, 99.0, 81.0]];
        let option_tree = vec![vec![0.0], vec![5.0, 0.0], vec![10.0, 5.0, 0.0]];
        let filename = "./Draws/Binomial Tree/test_output.png";

        // Ensure the function runs without errors
        let result = draw_binomial_tree(&asset_tree, &option_tree, filename);
        assert!(result.is_ok());

        // Cleanup
        let _ = fs::remove_file(filename);
    }
}
