use optionstratlib::curves::{Curve, Point2D};
use optionstratlib::surfaces::{Point3D, Surface};
use optionstratlib::visualization::{
    GraphData, GraphType, OutputType, PlotType, Series2D, Surface3D, TraceMode,
};
use rust_decimal_macros::dec;
use std::collections::BTreeSet;
use std::path::PathBuf;

#[test]
fn test_series2d_default() {
    let default_series = Series2D::default();
    
    assert!(default_series.x.is_empty());
    assert!(default_series.y.is_empty());
    assert_eq!(default_series.name, "Series");
    assert_eq!(default_series.mode, TraceMode::Lines);
    assert_eq!(default_series.line_color, None);
    assert_eq!(default_series.line_width, None);
}

#[test]
fn test_series2d_plot_type() {
    assert_eq!(Series2D::plot_type(), PlotType::Line2D);
}

#[test]
fn test_surface3d_plot_type() {
    assert_eq!(Surface3D::plot_type(), PlotType::Surface3D);
}

#[test]
fn test_graph_data_from_curve() {
    // Create a simple curve
    let mut points = BTreeSet::new();
    points.insert(Point2D::new(dec!(1.0), dec!(2.0)));
    points.insert(Point2D::new(dec!(2.0), dec!(4.0)));
    points.insert(Point2D::new(dec!(3.0), dec!(6.0)));
    
    let curve = Curve::new(points);
    
    // Convert to GraphData
    let graph_data = GraphData::from(curve.clone());
    
    // Verify the conversion
    match graph_data {
        GraphData::Series(series) => {
            assert_eq!(series.x, vec![dec!(1.0), dec!(2.0), dec!(3.0)]);
            assert_eq!(series.y, vec![dec!(2.0), dec!(4.0), dec!(6.0)]);
            assert_eq!(series.name, "Curve");
            assert_eq!(series.mode, TraceMode::Lines);
            assert_eq!(series.line_color, Some("#1f77b4".to_string()));
            assert_eq!(series.line_width, Some(2.0));
        }
        _ => panic!("Expected GraphData::Series"),
    }
}

#[test]
fn test_graph_data_from_vec_curves() {
    // Create two simple curves
    let mut points1 = BTreeSet::new();
    points1.insert(Point2D::new(dec!(1.0), dec!(2.0)));
    points1.insert(Point2D::new(dec!(2.0), dec!(4.0)));
    let curve1 = Curve::new(points1);
    
    let mut points2 = BTreeSet::new();
    points2.insert(Point2D::new(dec!(1.0), dec!(3.0)));
    points2.insert(Point2D::new(dec!(2.0), dec!(6.0)));
    let curve2 = Curve::new(points2);
    
    let curves = vec![curve1, curve2];
    
    // Convert to GraphData
    let graph_data = GraphData::from(curves);
    
    // Verify the conversion
    match graph_data {
        GraphData::MultiSeries(series_vec) => {
            assert_eq!(series_vec.len(), 2);
            
            // Check first series
            assert_eq!(series_vec[0].x, vec![dec!(1.0), dec!(2.0)]);
            assert_eq!(series_vec[0].y, vec![dec!(2.0), dec!(4.0)]);
            assert_eq!(series_vec[0].name, "Curve 1");
            assert_eq!(series_vec[0].mode, TraceMode::Lines);
            assert!(series_vec[0].line_color.is_some());
            assert_eq!(series_vec[0].line_width, Some(2.0));
            
            // Check second series
            assert_eq!(series_vec[1].x, vec![dec!(1.0), dec!(2.0)]);
            assert_eq!(series_vec[1].y, vec![dec!(3.0), dec!(6.0)]);
            assert_eq!(series_vec[1].name, "Curve 2");
            assert_eq!(series_vec[1].mode, TraceMode::Lines);
            assert!(series_vec[1].line_color.is_some());
            assert_eq!(series_vec[1].line_width, Some(2.0));
        }
        _ => panic!("Expected GraphData::MultiSeries"),
    }
}

#[test]
fn test_graph_data_from_surface() {
    // Create a simple surface
    let mut points = BTreeSet::new();
    points.insert(Point3D::new(dec!(1.0), dec!(2.0), dec!(3.0)));
    points.insert(Point3D::new(dec!(4.0), dec!(5.0), dec!(6.0)));
    points.insert(Point3D::new(dec!(7.0), dec!(8.0), dec!(9.0)));
    
    let surface = Surface::new(points);
    
    // Convert to GraphData
    let graph_data = GraphData::from(surface);
    
    // Verify the conversion
    match graph_data {
        GraphData::Surface(surface3d) => {
            assert_eq!(surface3d.x, vec![dec!(1.0), dec!(4.0), dec!(7.0)]);
            assert_eq!(surface3d.y, vec![dec!(2.0), dec!(5.0), dec!(8.0)]);
            assert_eq!(surface3d.z, vec![dec!(3.0), dec!(6.0), dec!(9.0)]);
            assert_eq!(surface3d.name, "Surface");
        }
        _ => panic!("Expected GraphData::Surface"),
    }
}

#[test]
fn test_output_type_variants() {
    // Test the Browser variant
    let browser_output = OutputType::Browser;
    assert!(matches!(browser_output, OutputType::Browser));
    
    // Test the Html variant
    let path = PathBuf::from("/tmp/test.html");
    let html_output = OutputType::Html(&path);
    assert!(matches!(html_output, OutputType::Html(_)));
    
    // The following tests are conditionally compiled only when the kaleido feature is enabled
    #[cfg(feature = "kaleido")]
    {
        let png_output = OutputType::Png(&path);
        assert!(matches!(png_output, OutputType::Png(_)));
        
        let svg_output = OutputType::Svg(&path);
        assert!(matches!(svg_output, OutputType::Svg(_)));
    }
}
