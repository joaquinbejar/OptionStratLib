#[cfg(test)]
mod tests_utils {
    use crate::visualization::{ColorScheme, get_color_from_scheme};

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
            GraphData::GraphSurface(surf3d) => {
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
            GraphData::GraphSurface(surf3d) => {
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
    use crate::visualization::{Graph, GraphConfig, GraphData, Series2D, TraceMode};
    use rust_decimal_macros::dec;

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
