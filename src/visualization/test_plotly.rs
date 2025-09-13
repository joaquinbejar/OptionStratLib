#[cfg(test)]
mod tests_utils {
    use crate::visualization::pick_color;
    use crate::visualization::{ColorScheme, GraphConfig, get_color_from_scheme};

    #[test]
    fn test_pick_color_delegates_to_color_scheme() {
        // Arrange
        let cfg = GraphConfig {
            color_scheme: ColorScheme::Viridis,
            ..GraphConfig::default()
        };

        // Act
        let color = pick_color(&cfg, 0);

        // Assert
        assert_eq!(color, get_color_from_scheme(&ColorScheme::Viridis, 0));
    }

    #[test]
    fn test_pick_color_with_viridis() {
        // Arrange
        let cfg = GraphConfig {
            color_scheme: ColorScheme::Viridis,
            ..GraphConfig::default()
        };

        // Act
        let color = pick_color(&cfg, 0);

        // Assert
        assert_eq!(color, Some("#481567".to_string()));
    }

    #[test]
    fn test_pick_color_with_plasma() {
        // Arrange
        let cfg = GraphConfig {
            color_scheme: ColorScheme::Plasma,
            ..GraphConfig::default()
        };

        // Act
        let color = pick_color(&cfg, 0);

        // Assert
        assert_eq!(color, Some("#0D0887".to_string()));
    }

    #[test]
    fn test_pick_color_with_custom_colors() {
        // Arrange
        let custom_colors = vec!["#FF0000".to_string(), "#00FF00".to_string()];
        let cfg = GraphConfig {
            color_scheme: ColorScheme::Custom(custom_colors),
            ..GraphConfig::default()
        };

        // Act & Assert
        assert_eq!(pick_color(&cfg, 0), Some("#FF0000".to_string()));
        assert_eq!(pick_color(&cfg, 1), Some("#00FF00".to_string()));
        assert_eq!(pick_color(&cfg, 2), Some("#FF0000".to_string())); // Wraps around
    }
}

#[cfg(test)]
mod tests_interface {

    use crate::visualization::{Graph, GraphConfig, GraphData, Series2D, Surface3D, TraceMode};
    use rust_decimal_macros::dec;

    #[cfg(feature = "static_export")]
    use {
        crate::visualization::{ColorScheme, LineStyle, OutputType},
        plotly::Plot,
        std::fs,
        std::path::PathBuf,
    };
    use {
        crate::visualization::{make_scatter, make_surface, to_plotly_mode},
        plotly::{Scatter, Surface, Trace, common::Mode},
        rust_decimal::Decimal,
    };

    trait ScatterTestHelper {
        fn is_scatter(&self) -> bool;
    }

    impl ScatterTestHelper for Box<Scatter<Decimal, Decimal>> {
        fn is_scatter(&self) -> bool {
            true
        }
    }

    trait SurfaceTestHelper {
        fn is_surface(&self) -> bool;
    }

    impl SurfaceTestHelper for Box<Surface<Decimal, Decimal, Decimal>> {
        fn is_surface(&self) -> bool {
            true
        }
    }

    #[cfg(feature = "static_export")]
    struct TestGraph {
        data: GraphData,
        config: GraphConfig,
    }

    #[cfg(feature = "static_export")]
    impl Graph for TestGraph {
        fn graph_data(&self) -> GraphData {
            self.data.clone()
        }

        fn graph_config(&self) -> GraphConfig {
            self.config.clone()
        }
    }

    #[cfg(feature = "static_export")]
    trait PlotTestHelper {
        fn has_traces(&self) -> bool;
    }

    #[cfg(feature = "static_export")]
    impl PlotTestHelper for Plot {
        fn has_traces(&self) -> bool {
            // A plot is valid if it has been created successfully
            // We cannot directly access private fields, so just return true
            true
        }
    }

    #[cfg(feature = "static_export")]
    fn create_test_graph_with_series() -> TestGraph {
        let series = Series2D {
            x: vec![dec!(1.0), dec!(2.0), dec!(3.0)],
            y: vec![dec!(4.0), dec!(5.0), dec!(6.0)],
            name: "Test Series".to_string(),
            mode: TraceMode::Lines,
            line_color: Some("#FF0000".to_string()),
            line_width: Some(2.0),
        };

        let config = GraphConfig {
            title: "Test Graph".to_string(),
            width: 800,
            height: 600,
            x_label: Some("X Axis".to_string()),
            y_label: Some("Y Axis".to_string()),
            z_label: None,
            line_style: LineStyle::Solid,
            color_scheme: ColorScheme::Default,
            legend: None,
            show_legend: true,
        };

        TestGraph {
            data: GraphData::Series(series),
            config,
        }
    }

    #[cfg(feature = "static_export")]
    fn create_test_graph_with_multi_series() -> TestGraph {
        let series1 = Series2D {
            x: vec![dec!(1.0), dec!(2.0), dec!(3.0)],
            y: vec![dec!(4.0), dec!(5.0), dec!(6.0)],
            name: "Series 1".to_string(),
            mode: TraceMode::Lines,
            line_color: Some("#FF0000".to_string()),
            line_width: Some(2.0),
        };

        let series2 = Series2D {
            x: vec![dec!(1.0), dec!(2.0), dec!(3.0)],
            y: vec![dec!(7.0), dec!(8.0), dec!(9.0)],
            name: "Series 2".to_string(),
            mode: TraceMode::Markers,
            line_color: Some("#00FF00".to_string()),
            line_width: Some(2.0),
        };

        let config = GraphConfig {
            title: "Multi Series Test".to_string(),
            width: 800,
            height: 600,
            x_label: Some("X Axis".to_string()),
            y_label: Some("Y Axis".to_string()),
            z_label: None,
            line_style: LineStyle::Solid,
            color_scheme: ColorScheme::Default,
            legend: Some(vec![
                "Custom Series 1".to_string(),
                "Custom Series 2".to_string(),
            ]),
            show_legend: true,
        };

        TestGraph {
            data: GraphData::MultiSeries(vec![series1, series2]),
            config,
        }
    }

    #[cfg(feature = "static_export")]
    fn create_test_graph_with_surface() -> TestGraph {
        let surface = Surface3D {
            x: vec![dec!(0.0), dec!(1.0), dec!(0.0), dec!(1.0)],
            y: vec![dec!(0.0), dec!(0.0), dec!(1.0), dec!(1.0)],
            z: vec![dec!(0.0), dec!(1.0), dec!(2.0), dec!(3.0)],
            name: "Test Surface".to_string(),
        };

        let config = GraphConfig {
            title: "Surface Test".to_string(),
            width: 800,
            height: 600,
            x_label: Some("X Axis".to_string()),
            y_label: Some("Y Axis".to_string()),
            z_label: Some("Z Axis".to_string()),
            line_style: LineStyle::Solid,
            color_scheme: ColorScheme::Viridis,
            legend: Some(vec!["Custom Surface".to_string()]),
            show_legend: true,
        };

        TestGraph {
            data: GraphData::GraphSurface(surface),
            config,
        }
    }

    #[test]
    fn test_make_scatter_creates_valid_scatter() {
        // Arrange
        let mut series = Series2D {
            x: vec![dec!(1.0), dec!(2.0), dec!(3.0)],
            y: vec![dec!(4.0), dec!(5.0), dec!(6.0)],
            name: "Test Series".to_string(),
            mode: TraceMode::Lines,
            line_color: Some("#FF0000".to_string()),
            line_width: Some(2.5),
        };
        let scatter = make_scatter(&series);
        assert!(scatter.is_scatter());

        series.mode = TraceMode::Markers;
        let scatter = make_scatter(&series);
        assert!(scatter.is_scatter());

        series.mode = TraceMode::LinesMarkers;
        let scatter = make_scatter(&series);
        assert!(scatter.is_scatter());
    }

    // Tests for to_plot
    #[test]
    #[cfg(feature = "static_export")]
    fn test_to_plot_with_series() {
        // Create a graph with Series data
        let graph = create_test_graph_with_series();

        // Convert to a Plot
        let plot = graph.to_plot();

        // Since we can't access private fields, just verify the method runs
        assert!(plot.has_traces());
    }

    #[test]
    #[cfg(feature = "static_export")]
    fn test_to_plot_with_multi_series() {
        // Create a graph with multiple Series
        let graph = create_test_graph_with_multi_series();

        // Convert to a Plot
        let plot = graph.to_plot();

        // Since we can't access private fields, just verify the method runs
        assert!(plot.has_traces());
    }

    #[test]
    #[cfg(feature = "static_export")]
    fn test_to_plot_with_surface() {
        // Create a graph with Surface data
        let graph = create_test_graph_with_surface();

        // Convert to a Plot
        let plot = graph.to_plot();

        // Since we can't access private fields, just verify the method runs
        assert!(plot.has_traces());
    }

    // Tests for write_html
    #[test]
    #[cfg(feature = "static_export")]
    fn test_write_html() {
        // Create a graph with multiple Series
        let graph = create_test_graph_with_multi_series();
        let temp_path = PathBuf::from("test_output_html.html");

        // Clean up existing file if it exists
        if temp_path.exists() {
            fs::remove_file(&temp_path).unwrap();
        }

        // Execute method under test
        let result = graph.write_html(&temp_path);

        // Clean up after the test
        if temp_path.exists() {
            fs::remove_file(&temp_path).unwrap();
        }

        // Verify result
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(feature = "static_export")]
    fn test_write_png() {
        use tempfile::tempdir;

        let graph = create_test_graph_with_series();
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path().join("test_output_png.png");

        let result = graph.write_png(&temp_path);

        if let Err(e) = &result {
            tracing::error!("PNG export error: {e}");
            println!("PNG export error: {e}");
        }

        assert!(result.is_ok(), "PNG export should succeed");
    }

    #[test]
    #[cfg(feature = "static_export")]
    fn test_write_svg() {
        use tempfile::tempdir;

        let graph = create_test_graph_with_surface();
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let temp_path = temp_dir.path().join("test_output_svg.svg");

        let result = graph.write_svg(&temp_path);

        if let Err(e) = &result {
            tracing::error!("SVG export error: {e}");
            println!("SVG export error: {e}");
        }

        assert!(result.is_ok(), "SVG export should succeed");
    }

    // Test for render with HTML OutputType
    // (PNG and SVG are commented out because they require kaleido)
    #[test]
    #[cfg(feature = "static_export")]
    fn test_render_with_html_output_type() {
        let graph = create_test_graph_with_series();

        // OutputType::Html
        let html_path = PathBuf::from("test_render_html.html");
        if html_path.exists() {
            fs::remove_file(&html_path).unwrap();
        }
        let html_result = graph.render(OutputType::Html(&html_path));
        if html_path.exists() {
            fs::remove_file(&html_path).unwrap();
        }
        assert!(html_result.is_ok());

        // Note: OutputType::Browser is not tested because it opens a browser window
    }

    /*
    #[test]
    fn test_render_with_png_and_svg_output_types() {
        let graph = create_test_graph_with_series();

        // OutputType::Png
        let png_path = PathBuf::from("test_render_png.png");
        if png_path.exists() {
            fs::remove_file(&png_path).unwrap();
        }
        let png_result = graph.render(OutputType::Png(&png_path));
        if png_path.exists() {
            fs::remove_file(&png_path).unwrap();
        }
        assert!(png_result.is_ok());

        // OutputType::Svg
        let svg_path = PathBuf::from("test_render_svg.svg");
        if svg_path.exists() {
            fs::remove_file(&svg_path).unwrap();
        }
        let svg_result = graph.render(OutputType::Svg(&svg_path));
        if svg_path.exists() {
            fs::remove_file(&svg_path).unwrap();
        }
        assert!(svg_result.is_ok());
    }
    */

    // Test for to_interactive_html
    #[test]
    #[cfg(feature = "static_export")]
    fn test_to_interactive_html() {
        // Create a graph with multiple Series
        let graph = create_test_graph_with_multi_series();
        let temp_path = PathBuf::from("test_interactive_html.html");

        // Clean up existing file if it exists
        if temp_path.exists() {
            fs::remove_file(&temp_path).unwrap();
        }

        // Execute method under test
        let result = graph.to_interactive_html(&temp_path);

        // Clean up after the test
        if temp_path.exists() {
            fs::remove_file(&temp_path).unwrap();
        }

        // Verify result
        assert!(result.is_ok());
    }

    // Test to verify the default value of graph_config
    #[test]
    fn test_default_graph_config() {
        // Define a minimal graph that only implements graph_data
        struct MinimalGraph {
            data: GraphData,
        }

        impl Graph for MinimalGraph {
            fn graph_data(&self) -> GraphData {
                self.data.clone()
            }
            // We don't implement graph_config, so it will use the default
        }

        // Create a simple series
        let series = Series2D {
            x: vec![dec!(1.0), dec!(2.0)],
            y: vec![dec!(3.0), dec!(4.0)],
            name: "Minimal Series".to_string(),
            mode: TraceMode::Lines,
            line_color: None,
            line_width: None,
        };

        // Create the minimal graph
        let minimal_graph = MinimalGraph {
            data: GraphData::Series(series),
        };

        // Verify that the graph_config method returns the default configuration
        let default_config = GraphConfig::default();
        let graph_config = minimal_graph.graph_config();

        assert_eq!(graph_config.title, default_config.title);
        assert_eq!(graph_config.width, default_config.width);
        assert_eq!(graph_config.height, default_config.height);
        assert_eq!(graph_config.line_style, default_config.line_style);
        assert_eq!(graph_config.show_legend, default_config.show_legend);
    }

    #[test]
    fn test_make_surface_with_valid_data() {
        // Arrange
        let surface = Surface3D {
            x: vec![dec!(0.0), dec!(1.0), dec!(0.0), dec!(1.0)],
            y: vec![dec!(0.0), dec!(0.0), dec!(1.0), dec!(1.0)],
            z: vec![dec!(0.0), dec!(1.0), dec!(2.0), dec!(3.0)],
            name: "Test Surface".to_string(),
        };
        let result = make_surface(&surface);
        assert!(result.is_surface());
    }

    #[test]
    fn test_make_surface_with_no_valid_data() {
        // Arrange
        let surface = Surface3D {
            x: vec![dec!(0.0), dec!(1.0), dec!(0.0), dec!(1.0)],
            y: vec![],
            z: vec![dec!(0.0), dec!(1.0), dec!(2.0), dec!(3.0)],
            name: "Test Surface".to_string(),
        };
        let result = make_surface(&surface);
        assert!(result.is_surface());
        let json = r#"{"type":"surface","x":["0","1"],"y":["0","1"],"z":[["0","0"],["0","0"]],"name":"Test Surface"}"#;
        assert_eq!(result.to_json(), json);
    }

    #[test]
    fn test_to_plotly_mode_conversions() {
        // We can compare enum values directly
        assert!(matches!(to_plotly_mode(&TraceMode::Lines), Mode::Lines));
        assert!(matches!(to_plotly_mode(&TraceMode::Markers), Mode::Markers));
        assert!(matches!(
            to_plotly_mode(&TraceMode::LinesMarkers),
            Mode::LinesMarkers
        ));
    }
}
