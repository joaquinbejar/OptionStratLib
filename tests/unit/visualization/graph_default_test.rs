// Test the Graph trait implementation
// This module tests the Graph trait regardless of which feature is enabled
#[cfg(test)]
mod graph_tests {
    use optionstratlib::visualization::{Graph, GraphConfig, GraphData};
    use rust_decimal_macros::dec;
    use optionstratlib::visualization::Series2D;
    use optionstratlib::visualization::TraceMode;

    // A minimal implementation of Graph that only implements graph_data()
    // This will force the default implementation of graph_config() to be used
    struct MinimalGraph {
        data: GraphData,
    }

    impl MinimalGraph {
        fn new(data: GraphData) -> Self {
            Self { data }
        }
    }

    impl Graph for MinimalGraph {
        fn graph_data(&self) -> GraphData {
            self.data.clone()
        }
        
        // Deliberately NOT implementing graph_config() to test the default implementation
    }

    #[test]
    fn test_default_graph_config_implementation() {
        // Create a simple GraphData for testing
        let series = Series2D {
            x: vec![dec!(1.0), dec!(2.0)],
            y: vec![dec!(3.0), dec!(4.0)],
            name: "Test Series".to_string(),
            mode: TraceMode::Lines,
            line_color: None,
            line_width: None,
        };
        let data = GraphData::Series(series);
        
        // Create a graph that uses the default implementation
        let graph = MinimalGraph::new(data);
        
        // Test that the default implementation returns the default config
        let config = graph.graph_config();
        assert_eq!(config, GraphConfig::default());
    }
}
