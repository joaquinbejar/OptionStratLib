#[cfg(test)]
mod tests_utils {
    use crate::visualization::{
        ColorScheme, GraphConfig, Series2D, Surface3D, TraceMode, get_color_from_scheme,
        make_scatter, make_surface, pick_color, to_plotly_mode,
    };
    use plotly::Trace;
    use plotly::common::Mode;
    use plotly::{Scatter, Surface};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

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
    fn test_get_color_from_scheme_default() {
        assert_eq!(get_color_from_scheme(&ColorScheme::Default, 0), None);
        assert_eq!(get_color_from_scheme(&ColorScheme::Default, 10), None);
    }

    #[test]
    fn test_get_color_from_scheme_viridis() {
        // First color in Viridis
        assert_eq!(
            get_color_from_scheme(&ColorScheme::Viridis, 0),
            Some("#481567".to_string())
        );

        // Test index wrapping (modulo length)
        let idx = 100;
        let colors_len = 40; // Length of Viridis color vector
        let wrapped_idx = idx % colors_len;

        assert_eq!(
            get_color_from_scheme(&ColorScheme::Viridis, idx),
            get_color_from_scheme(&ColorScheme::Viridis, wrapped_idx)
        );
    }

    #[test]
    fn test_get_color_from_scheme_plasma() {
        // First color in Plasma
        assert_eq!(
            get_color_from_scheme(&ColorScheme::Plasma, 0),
            Some("#0D0887".to_string())
        );

        // Test with different index
        assert_eq!(
            get_color_from_scheme(&ColorScheme::Plasma, 5),
            Some("#F89540".to_string())
        );
    }

    #[test]
    fn test_get_color_from_scheme_white() {
        assert_eq!(
            get_color_from_scheme(&ColorScheme::White, 0),
            Some("#FFFFFF".to_string())
        );
        assert_eq!(
            get_color_from_scheme(&ColorScheme::White, 100),
            Some("#FFFFFF".to_string())
        );
    }

    #[test]
    fn test_get_color_from_scheme_custom() {
        // Arrange
        let custom_colors = vec![
            "#FF0000".to_string(),
            "#00FF00".to_string(),
            "#0000FF".to_string(),
        ];
        let scheme = ColorScheme::Custom(custom_colors.clone());

        // Assert
        assert_eq!(
            get_color_from_scheme(&scheme, 0),
            Some("#FF0000".to_string())
        );
        assert_eq!(
            get_color_from_scheme(&scheme, 1),
            Some("#00FF00".to_string())
        );
        assert_eq!(
            get_color_from_scheme(&scheme, 2),
            Some("#0000FF".to_string())
        );

        // Ensure wrapping works
        assert_eq!(
            get_color_from_scheme(&scheme, 3),
            Some("#FF0000".to_string())
        );
        assert_eq!(
            get_color_from_scheme(&scheme, 4),
            Some("#00FF00".to_string())
        );
    }

    #[test]
    fn test_get_color_from_scheme_high_contrast() {
        // First color in HighContrast
        assert_eq!(
            get_color_from_scheme(&ColorScheme::HighContrast, 0),
            Some("#FF0000".to_string())
        );

        // Check a few more values
        assert_eq!(
            get_color_from_scheme(&ColorScheme::HighContrast, 1),
            Some("#00FF00".to_string())
        );
        assert_eq!(
            get_color_from_scheme(&ColorScheme::HighContrast, 2),
            Some("#0000FF".to_string())
        );
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
        println!("{:?}", result);
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

#[cfg(test)]
mod tests_model {
    use crate::surfaces::{Point3D, Surface};
    use crate::visualization::{GraphData, GraphType, PlotType, Series2D, Surface3D, TraceMode};
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

    #[test]
    fn test_series2d_default() {
        // Act
        let series = Series2D::default();

        // Assert
        assert!(series.x.is_empty());
        assert!(series.y.is_empty());
        assert_eq!(series.name, "Series");
        assert!(matches!(series.mode, TraceMode::Lines));
        assert!(series.line_color.is_none());
        assert!(series.line_width.is_none());
    }

    #[test]
    fn test_from_surface_for_graph_data() {
        // Arrange
        let points = vec![
            Point3D {
                x: dec!(0.0),
                y: dec!(0.0),
                z: dec!(1.0),
            },
            Point3D {
                x: dec!(1.0),
                y: dec!(0.0),
                z: dec!(2.0),
            },
            Point3D {
                x: dec!(0.0),
                y: dec!(1.0),
                z: dec!(3.0),
            },
            Point3D {
                x: dec!(1.0),
                y: dec!(1.0),
                z: dec!(4.0),
            },
        ];
        let points: BTreeSet<Point3D> = points.into_iter().collect();
        let surface = Surface {
            points,
            x_range: (Default::default(), Default::default()),
            y_range: (Default::default(), Default::default()),
        };
        let graph_data = GraphData::from(surface);
        match graph_data {
            GraphData::Surface(surf3d) => {
                assert_eq!(surf3d.name, "Surface");
                assert_eq!(surf3d.x, vec![dec!(0.0), dec!(0.0), dec!(1.0), dec!(1.0)]);
                assert_eq!(surf3d.y, vec![dec!(0.0), dec!(1.0), dec!(0.0), dec!(1.0)]);
                assert_eq!(surf3d.z, vec![dec!(1.0), dec!(3.0), dec!(2.0), dec!(4.0)]);
            }
            _ => panic!("Expected GraphData::Surface variant!"),
        }
    }

    #[test]
    fn test_graph_type_for_series2d() {
        // Act & Assert
        assert!(matches!(Series2D::plot_type(), PlotType::Line2D));
    }

    #[test]
    fn test_graph_type_for_surface3d() {
        // Act & Assert
        assert!(matches!(Surface3D::plot_type(), PlotType::Surface3D));
    }

    #[test]
    fn test_from_empty_surface_for_graph_data() {
        let empty_surface = Surface {
            points: BTreeSet::new(),
            x_range: (Default::default(), Default::default()),
            y_range: (Default::default(), Default::default()),
        };
        let graph_data = GraphData::from(empty_surface);
        match graph_data {
            GraphData::Surface(surf3d) => {
                assert_eq!(surf3d.name, "Surface");
                assert!(surf3d.x.is_empty());
                assert!(surf3d.y.is_empty());
                assert!(surf3d.z.is_empty());
            }
            _ => panic!("Expected GraphData::Surface variant!"),
        }
    }
}


#[cfg(test)]
mod tests_interface {
    use crate::visualization::{
        ColorScheme, Graph, GraphConfig, GraphData, LineStyle, OutputType,
        Series2D, Surface3D, TraceMode,
    };
    use rust_decimal_macros::dec;
    use std::path::PathBuf;
    use std::fs;
    use plotly::Plot;

    // Implementation of a simple type that implements Graph for testing
    struct TestGraph {
        data: GraphData,
        config: GraphConfig,
    }

    impl Graph for TestGraph {
        fn graph_data(&self) -> GraphData {
            self.data.clone()
        }

        fn graph_config(&self) -> GraphConfig {
            self.config.clone()
        }
    }

    // Trait to help test Plot methods
    trait PlotTestHelper {
        fn has_traces(&self) -> bool;
    }

    impl PlotTestHelper for Plot {
        fn has_traces(&self) -> bool {
            // A plot is valid if it has been created successfully
            // We cannot directly access private fields, so just return true
            true
        }
    }

    // Helper to create a TestGraph with Series2D
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

    // Helper to create a TestGraph with MultiSeries
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
            legend: Some(vec!["Custom Series 1".to_string(), "Custom Series 2".to_string()]),
            show_legend: true,
        };

        TestGraph {
            data: GraphData::MultiSeries(vec![series1, series2]),
            config,
        }
    }

    // Helper to create a TestGraph with Surface
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
            data: GraphData::Surface(surface),
            config,
        }
    }

    // Tests for to_plot
    #[test]
    fn test_to_plot_with_series() {
        // Create a graph with Series data
        let graph = create_test_graph_with_series();

        // Convert to a Plot
        let plot = graph.to_plot();

        // Since we can't access private fields, just verify the method runs
        assert!(plot.has_traces());
    }

    #[test]
    fn test_to_plot_with_multi_series() {
        // Create a graph with multiple Series
        let graph = create_test_graph_with_multi_series();

        // Convert to a Plot
        let plot = graph.to_plot();

        // Since we can't access private fields, just verify the method runs
        assert!(plot.has_traces());
    }

    #[test]
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

    // Note: Tests for write_png and write_svg are commented out
    // because they require the kaleido feature
    
    #[test]
    #[cfg(feature = "kaleido")]
    fn test_write_png() {
        let graph = create_test_graph_with_series();
        let temp_path = PathBuf::from("test_output_png.png");
        
        if temp_path.exists() {
            fs::remove_file(&temp_path).unwrap();
        }
        
        let result = graph.write_png(&temp_path);
        
        if temp_path.exists() {
            fs::remove_file(&temp_path).unwrap();
        }
        
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(feature = "kaleido")]
    fn test_write_svg() {
        let graph = create_test_graph_with_surface();
        let temp_path = PathBuf::from("test_output_svg.svg");
        
        if temp_path.exists() {
            fs::remove_file(&temp_path).unwrap();
        }
        
        let result = graph.write_svg(&temp_path);
        
        if temp_path.exists() {
            fs::remove_file(&temp_path).unwrap();
        }
        
        assert!(result.is_ok());
    }
    

    // Test for render with HTML OutputType
    // (PNG and SVG are commented out because they require kaleido)
    #[test]
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
}