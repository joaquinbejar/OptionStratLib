#[cfg(all(feature = "static_export", feature = "plotly"))]
mod plotly_render_tests {

    use optionstratlib::visualization::{Graph, GraphConfig, GraphData, OutputType, Series2D};

    use rust_decimal::Decimal;
    use std::path::PathBuf;
    use std::str::FromStr;
    use tempfile::tempdir;

    // Test implementation of the Graph trait
    struct TestGraph {
        data: GraphData,
        config: GraphConfig,
    }

    impl TestGraph {
        fn new(data: GraphData, config: GraphConfig) -> Self {
            Self { data, config }
        }
    }

    impl Graph for TestGraph {
        fn graph_data(&self) -> GraphData {
            self.data.clone()
        }

        fn graph_config(&self) -> GraphConfig {
            self.config.clone()
        }
    }

    // Helper function to create a sample Series2D
    fn create_sample_series() -> Series2D {
        Series2D {
            x: vec![
                Decimal::from_str("1.0").unwrap(),
                Decimal::from_str("2.0").unwrap(),
                Decimal::from_str("3.0").unwrap(),
            ],
            y: vec![
                Decimal::from_str("4.0").unwrap(),
                Decimal::from_str("5.0").unwrap(),
                Decimal::from_str("6.0").unwrap(),
            ],
            name: "Test Series".to_string(),
            mode: optionstratlib::visualization::TraceMode::Lines,
            line_color: Some("#FF0000".to_string()),
            line_width: Some(2.0),
        }
    }

    // Helper function to create a sample GraphConfig
    fn create_sample_config() -> GraphConfig {
        GraphConfig {
            title: "Test Graph".to_string(),
            width: 800,
            height: 600,
            x_label: Some("X Axis".to_string()),
            y_label: Some("Y Axis".to_string()),
            z_label: Some("Z Axis".to_string()),
            line_style: optionstratlib::visualization::LineStyle::Solid,
            color_scheme: optionstratlib::visualization::ColorScheme::Viridis,
            legend: Some(vec!["Series 1".to_string()]),
            show_legend: true,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        // Test with different configuration options
        #[test]
        fn test_to_plot_with_minimal_config() {
            let series = create_sample_series();

            // Create a minimal configuration
            let minimal_config = GraphConfig {
                title: "Minimal Config".to_string(),
                width: 400,
                height: 300,
                x_label: None, // No x label
                y_label: None, // No y label
                z_label: None, // No z label
                line_style: optionstratlib::visualization::LineStyle::Solid,
                color_scheme: optionstratlib::visualization::ColorScheme::Viridis,
                legend: None, // No legend
                show_legend: false,
            };

            let graph = TestGraph::new(GraphData::Series(series), minimal_config);
            let plot = graph.to_plot();

            // Basic assertions to ensure the plot was created correctly
            assert_eq!(plot.data().len(), 1);
            let layout = plot.layout();
            assert!(layout.to_json().contains("\"width\":400"));
            assert!(layout.to_json().contains("\"height\":300"));
        }

        // Test with all axis labels specified
        #[test]
        fn test_to_plot_with_all_axis_labels() {
            let series = create_sample_series();

            // Create a configuration with all axis labels
            let config = GraphConfig {
                title: "All Axis Labels".to_string(),
                width: 600,
                height: 400,
                x_label: Some("X Axis Label".to_string()),
                y_label: Some("Y Axis Label".to_string()),
                z_label: Some("Z Axis Label".to_string()),
                line_style: optionstratlib::visualization::LineStyle::Solid,
                color_scheme: optionstratlib::visualization::ColorScheme::Viridis,
                legend: Some(vec!["Series 1".to_string()]),
                show_legend: true,
            };

            let graph = TestGraph::new(GraphData::Series(series), config);
            let plot = graph.to_plot();

            // Basic assertions to ensure the plot was created correctly with all axis labels
            assert_eq!(plot.data().len(), 1);
            let layout_json = plot.layout().to_json();
            assert!(layout_json.contains("\"title\":{\"text\":\"X Axis Label\""));
            assert!(layout_json.contains("\"title\":{\"text\":\"Y Axis Label\""));
            assert!(layout_json.contains("\"title\":{\"text\":\"Z Axis Label\""));
        }

        // Test error handling with invalid paths
        #[test]
        fn test_render_error_handling() {
            // Create a graph
            let series = create_sample_series();
            let config = create_sample_config();
            let graph = TestGraph::new(GraphData::Series(series), config);

            // Test with invalid paths for different output types
            let invalid_path = std::path::PathBuf::from("/nonexistent/directory/file.png");

            // Test render with PNG to invalid path
            let png_result = graph.render(OutputType::Png(&invalid_path));
            assert!(png_result.is_err());

            // Test render with SVG to invalid path
            let svg_result = graph.render(OutputType::Svg(&invalid_path));
            assert!(svg_result.is_err());

            // Test render with HTML to invalid path
            let html_result = graph.render(OutputType::Html(&invalid_path));
            assert!(html_result.is_err());
        }

        // Test the render method with PNG output
        #[test]
        fn test_render_png() {
            let temp_dir = tempdir().unwrap();
            let file_path = temp_dir.path().join("test_render.png");

            let series = create_sample_series();
            let config = create_sample_config();
            let graph = TestGraph::new(GraphData::Series(series), config);

            let result = graph.render(OutputType::Png(&file_path));
            if let Err(e) = &result {
                println!("PNG render error: {e}");
            }
            assert!(result.is_ok(), "Render to PNG should succeed");
        }

        // Test the render method with SVG output
        #[test]
        fn test_render_svg() {
            let temp_dir = tempdir().unwrap();
            let file_path = temp_dir.path().join("test_render.svg");

            let series = create_sample_series();
            let config = create_sample_config();
            let graph = TestGraph::new(GraphData::Series(series), config);

            let result = graph.render(OutputType::Svg(&file_path));
            if let Err(e) = &result {
                println!("SVG render error: {e}");
            }
            assert!(result.is_ok(), "Render to SVG should succeed");
        }

        // Test the render method with HTML output
        #[test]
        fn test_render_html() {
            let temp_dir = tempdir().unwrap();
            let file_path = temp_dir.path().join("test_render.html");

            let series = create_sample_series();
            let config = create_sample_config();
            let graph = TestGraph::new(GraphData::Series(series), config);

            let result = graph.render(OutputType::Html(&file_path));
            assert!(result.is_ok(), "Render to HTML should succeed");
        }

        // Test the render method with Browser output
        // This test doesn't actually open a browser, it just calls the show method
        #[test]
        fn test_render_browser() {
            let series = create_sample_series();
            let config = create_sample_config();
            let graph = TestGraph::new(GraphData::Series(series), config);

            let result = graph.render(OutputType::Browser);
            assert!(result.is_ok(), "Render to Browser should succeed");
        }

        // Test the write_png method directly
        #[test]
        fn test_write_png() {
            let temp_dir = tempdir().unwrap();
            let file_path = temp_dir.path().join("test_write.png");

            let series = create_sample_series();
            let config = create_sample_config();
            let graph = TestGraph::new(GraphData::Series(series), config);

            let result = graph.write_png(&file_path);
            if let Err(e) = &result {
                println!("PNG export error: {e}");
            }
            assert!(result.is_ok(), "Write to PNG should succeed");
        }

        // Test the write_svg method directly
        #[test]
        fn test_write_svg() {
            let temp_dir = tempdir().unwrap();
            let file_path = temp_dir.path().join("test_write.svg");

            let series = create_sample_series();
            let config = create_sample_config();
            let graph = TestGraph::new(GraphData::Series(series), config);

            let result = graph.write_svg(&file_path);
            if let Err(e) = &result {
                println!("SVG export error: {e}");
            }
            assert!(result.is_ok(), "Write to SVG should succeed");
        }

        // Test the write_html method directly
        #[test]
        fn test_write_html() {
            let temp_dir = tempdir().unwrap();
            let file_path = temp_dir.path().join("test_write.html");

            let series = create_sample_series();
            let config = create_sample_config();
            let graph = TestGraph::new(GraphData::Series(series), config);

            let result = graph.write_html(&file_path);
            assert!(result.is_ok(), "Write to HTML should succeed");
        }

        // Test the to_interactive_html method
        #[test]
        fn test_to_interactive_html() {
            let temp_dir = tempdir().unwrap();
            let file_path = temp_dir.path().join("test_interactive.html");

            let series = create_sample_series();
            let config = create_sample_config();
            let graph = TestGraph::new(GraphData::Series(series), config);

            let result = graph.to_interactive_html(&file_path);
            assert!(result.is_ok(), "Interactive HTML should succeed");
        }

        // Test error handling in file operations
        #[test]
        fn test_file_operation_error_handling() {
            // Create an invalid path that should cause an error
            let invalid_path = PathBuf::from("/nonexistent/directory/test.png");

            let series = create_sample_series();
            let config = create_sample_config();
            let graph = TestGraph::new(GraphData::Series(series), config);

            // Test PNG write error handling
            let png_result = graph.write_png(&invalid_path);
            assert!(png_result.is_err(), "Write to invalid PNG path should fail");

            // Test SVG write error handling
            let svg_result = graph.write_svg(&invalid_path);
            assert!(svg_result.is_err(), "Write to invalid SVG path should fail");

            // Test HTML write error handling
            let html_result = graph.write_html(&invalid_path);
            assert!(
                html_result.is_err(),
                "Write to invalid HTML path should fail"
            );

            // Test interactive HTML error handling
            let interactive_result = graph.to_interactive_html(&invalid_path);
            assert!(
                interactive_result.is_err(),
                "Interactive HTML to invalid path should fail"
            );
        }
    }
}
