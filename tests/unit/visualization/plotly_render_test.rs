#[cfg(feature = "kaleido")]
mod plotly_render_tests {
    use optionstratlib::error::GraphError;
    use optionstratlib::visualization::{Graph, GraphConfig, GraphData, OutputType, Series2D};
    use mockall::*;
    use rust_decimal::Decimal;
    use std::path::{Path, PathBuf};
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

        // Test the render method with PNG output
        #[test]
        fn test_render_png() {
            let temp_dir = tempdir().unwrap();
            let file_path = temp_dir.path().join("test_render.png");
            
            let series = create_sample_series();
            let config = create_sample_config();
            let graph = TestGraph::new(GraphData::Series(series), config);
            
            let result = graph.render(OutputType::Png(&file_path));
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
            assert!(html_result.is_err(), "Write to invalid HTML path should fail");
            
            // Test interactive HTML error handling
            let interactive_result = graph.to_interactive_html(&invalid_path);
            assert!(interactive_result.is_err(), "Interactive HTML to invalid path should fail");
        }
    }
}
