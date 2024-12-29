/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 30/9/24
******************************************************************************/
use plotters::prelude::{RGBColor, ShapeStyle};

/// Represents how label offset should be interpreted
#[derive(Clone, Debug, PartialEq)]
pub enum LabelOffsetType {
    /// Absolute offset in pixel coordinates
    Absolute(f64, f64),
    /// Relative offset as percentage of the chart dimensions
    Relative(f64, f64),
    /// Automatic offset based on point location and chart boundaries
    Auto,
}

impl LabelOffsetType {
    pub fn get_offset(&self) -> (f64, f64) {
        match self {
            LabelOffsetType::Absolute(x, y) => (*x, *y),
            LabelOffsetType::Relative(x, y) => (*x, *y),
            LabelOffsetType::Auto => (2.0, 2.0),
        }
    }
}

/// Represents a point on a chart, including its coordinates, label, and styling options.
///
/// This struct is generic over the type `T` for coordinates, allowing flexibility
/// in the type of data used to represent the position of the point.
///
/// # Fields
///
/// * `coordinates`: The location of the chart point, of generic type `T`.
/// * `label`: A descriptive label for the chart point.
/// * `label_offset`: Specifies how the label should be offset relative to the point,
///   using the `LabelOffsetType` enum.
/// * `point_color`: The color used to draw the point.
/// * `label_color`: The color of the label text.
/// * `point_size`: The size in pixels of the visual representation of the point.
/// * `font_size`: The size in pixels of the font used for the label.
#[derive(Clone)]
pub struct ChartPoint<T> {
    pub coordinates: T,
    pub label: String,
    pub label_offset: LabelOffsetType,
    pub(crate) point_color: RGBColor,
    pub(crate) label_color: RGBColor,
    pub(crate) point_size: u32,
    pub(crate) font_size: u32,
}

impl<T> ChartPoint<T> {
    /// Creates a new ChartPoint with default styling
    pub fn new(coordinates: T, label: String) -> Self {
        Self {
            coordinates,
            label,
            label_offset: LabelOffsetType::Auto,
            point_color: RGBColor(0, 0, 0),
            label_color: RGBColor(0, 0, 0),
            point_size: 5,
            font_size: 12,
        }
    }

    /// Calculates the actual pixel offset for the label based on the chart dimensions
    pub fn calculate_offset(&self, chart_width: f64, chart_height: f64) -> (f64, f64) {
        match self.label_offset {
            LabelOffsetType::Absolute(x, y) => (x, y),
            LabelOffsetType::Relative(x_percent, y_percent) => {
                let x_offset = chart_width * (x_percent / 100.0);
                let y_offset = chart_height * (y_percent / 100.0);
                (x_offset, y_offset)
            }
            LabelOffsetType::Auto => {
                // Default margin as percentage of chart size
                let margin_percent = 2.0;
                let x_offset = chart_width * (margin_percent / 100.0);
                let y_offset = chart_height * (margin_percent / 100.0);
                (x_offset, y_offset)
            }
        }
    }

    pub fn with_absolute_offset(mut self, x: f64, y: f64) -> Self {
        self.label_offset = LabelOffsetType::Absolute(x, y);
        self
    }

    pub fn with_relative_offset(mut self, x_percent: f64, y_percent: f64) -> Self {
        self.label_offset = LabelOffsetType::Relative(x_percent, y_percent);
        self
    }

    pub fn with_auto_offset(mut self) -> Self {
        self.label_offset = LabelOffsetType::Auto;
        self
    }

    pub fn with_point_color(mut self, color: RGBColor) -> Self {
        self.point_color = color;
        self
    }

    pub fn with_label_color(mut self, color: RGBColor) -> Self {
        self.label_color = color;
        self
    }

    pub fn with_point_size(mut self, size: u32) -> Self {
        self.point_size = size;
        self
    }

    pub fn with_font_size(mut self, size: u32) -> Self {
        self.font_size = size;
        self
    }
}

/// `ChartVerticalLine` is a generic structure designed to represent a vertical line in a chart
/// with specified coordinates, style, and labeling attributes.
///
/// # Type Parameters
///
/// - `X`: The type representing the x-coordinate. This should be a type implementing
///   required traits, such as `PartialOrd`, to facilitate comparison of coordinate values.
/// - `Y`: The type representing the y-coordinate bounds (range). This should also implement
///   necessary traits for value manipulation and comparison.
///
/// # Fields
///
/// - `x_coordinate`: The x-coordinate where the vertical line is placed on the chart. It is of
///   type `X`.
/// - `y_range`: A tuple representing the range (min and max) on the y-axis for the vertical line.
///   This is of type `(Y, Y)`.
/// - `label`: A `String` that serves as the label for the vertical line, providing context or
///   description within the chart.
/// - `label_offset`: A tuple of `f64` values indicating the offset for the label positioning
///   relative to the line.
/// - `line_color`: A private field representing the color of the line. It is of type `RGBColor`.
/// - `label_color`: A private field indicating the color of the label. It is also of type `RGBColor`.
/// - `line_style`: A private field defining the styling attributes of the line, encapsulated
///   by `ShapeStyle`.
/// - `font_size`: A private field specifying the font size of the label. It is of type `u32`.
///
#[allow(dead_code)]
#[derive(Clone)]
pub struct ChartVerticalLine<X, Y> {
    pub x_coordinate: X,
    pub y_range: (Y, Y),
    pub label: String,
    pub label_offset: (f64, f64),
    pub(crate) line_color: RGBColor,
    pub(crate) label_color: RGBColor,
    pub(crate) line_style: ShapeStyle,
    pub(crate) font_size: u32,
}

#[cfg(test)]
mod tests_chart_point {
    use super::*;
    #[cfg(feature = "wasm")]
    use wasm_bindgen_test::*;

    #[cfg(feature = "wasm")]
    wasm_bindgen_test_configure!(run_in_browser);

    #[cfg_attr(not(feature = "wasm"), test)]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn test_relative_offset_scaling() {
        // Create a point with 5% relative offset
        let point =
            ChartPoint::new((100.0, 100.0), "Test".to_string()).with_relative_offset(5.0, 5.0);

        // Test with different chart sizes
        let (small_x, small_y) = point.calculate_offset(100.0, 100.0);
        let (large_x, large_y) = point.calculate_offset(1000.0, 1000.0);

        // The ratio should be maintained
        assert_eq!(small_x, 5.0); // 5% of 100
        assert_eq!(large_x, 50.0); // 5% of 1000
        assert_eq!(small_y, 5.0);
        assert_eq!(large_y, 50.0);
    }

    #[cfg_attr(not(feature = "wasm"), test)]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn test_relative_offset_scaling_bis() {
        let point =
            ChartPoint::new((100.0, 100.0), "Test".to_string()).with_relative_offset(5.0, 5.0);

        let (small_x, small_y) = point.calculate_offset(100.0, 100.0);
        let (large_x, large_y) = point.calculate_offset(1000.0, 1000.0);

        assert_eq!(small_x, 5.0); // 5% of 100
        assert_eq!(large_x, 50.0); // 5% of 1000
        assert_eq!(small_y, 5.0);
        assert_eq!(large_y, 50.0);
    }

    #[cfg_attr(not(feature = "wasm"), test)]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn test_auto_offset_scaling() {
        let point = ChartPoint::new((100.0, 100.0), "Test".to_string()).with_auto_offset();

        let (small_x, small_y) = point.calculate_offset(100.0, 100.0);
        let (large_x, large_y) = point.calculate_offset(1000.0, 1000.0);

        // Auto offset should be 2% of chart size
        assert_eq!(small_x, 2.0);
        assert_eq!(large_x, 20.0);
        assert_eq!(small_y, 2.0);
        assert_eq!(large_y, 20.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "wasm")]
    use wasm_bindgen_test::*;

    #[cfg(feature = "wasm")]
    wasm_bindgen_test_configure!(run_in_browser);

    // Helper function to compare f64 values with a small epsilon
    fn assert_float_eq(a: f64, b: f64) {
        assert!((a - b).abs() < f64::EPSILON);
    }

    mod chart_point_creation {
        use super::*;

        #[cfg_attr(not(feature = "wasm"), test)]
        #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
        fn test_new_chart_point_defaults() {
            let point = ChartPoint::new((100.0, 100.0), "Test".to_string());

            assert_eq!(point.coordinates, (100.0, 100.0));
            assert_eq!(point.label, "Test");
            assert_eq!(point.label_offset, LabelOffsetType::Auto);
            assert_eq!(point.point_color, RGBColor(0, 0, 0));
            assert_eq!(point.label_color, RGBColor(0, 0, 0));
            assert_eq!(point.point_size, 5);
            assert_eq!(point.font_size, 12);
        }

        #[cfg_attr(not(feature = "wasm"), test)]
        #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
        fn test_empty_label() {
            let point = ChartPoint::new((0.0, 0.0), String::new());
            assert_eq!(point.label, "");
        }
    }

    mod offset_calculations {
        use super::*;

        #[cfg_attr(not(feature = "wasm"), test)]
        #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
        fn test_absolute_offset() {
            let point =
                ChartPoint::new((0.0, 0.0), "Test".to_string()).with_absolute_offset(10.0, 20.0);

            let (x, y) = point.calculate_offset(100.0, 100.0);
            assert_float_eq(x, 10.0);
            assert_float_eq(y, 20.0);

            // Should be the same regardless of chart size
            let (x2, y2) = point.calculate_offset(1000.0, 1000.0);
            assert_float_eq(x2, 10.0);
            assert_float_eq(y2, 20.0);
        }

        #[cfg_attr(not(feature = "wasm"), test)]
        #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
        fn test_relative_offset() {
            let point =
                ChartPoint::new((0.0, 0.0), "Test".to_string()).with_relative_offset(10.0, 20.0);

            let (x, y) = point.calculate_offset(100.0, 100.0);
            assert_float_eq(x, 10.0);
            assert_float_eq(y, 20.0);

            let (x2, y2) = point.calculate_offset(200.0, 200.0);
            assert_float_eq(x2, 20.0);
            assert_float_eq(y2, 40.0);
        }

        #[cfg_attr(not(feature = "wasm"), test)]
        #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
        fn test_auto_offset() {
            let point = ChartPoint::new((0.0, 0.0), "Test".to_string()).with_auto_offset();

            let (x, y) = point.calculate_offset(100.0, 100.0);
            assert_float_eq(x, 2.0); // 2% of width
            assert_float_eq(y, 2.0); // 2% of height

            let (x2, y2) = point.calculate_offset(200.0, 200.0);
            assert_float_eq(x2, 4.0);
            assert_float_eq(y2, 4.0);
        }

        #[cfg_attr(not(feature = "wasm"), test)]
        #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
        fn test_zero_dimensions() {
            let point =
                ChartPoint::new((0.0, 0.0), "Test".to_string()).with_relative_offset(10.0, 10.0);

            let (x, y) = point.calculate_offset(0.0, 0.0);
            assert_float_eq(x, 0.0);
            assert_float_eq(y, 0.0);
        }
    }

    mod builder_methods {
        use super::*;

        #[cfg_attr(not(feature = "wasm"), test)]
        #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
        fn test_with_point_color() {
            let point = ChartPoint::new((0.0, 0.0), "Test".to_string())
                .with_point_color(RGBColor(255, 0, 0));
            assert_eq!(point.point_color, RGBColor(255, 0, 0));
        }

        #[cfg_attr(not(feature = "wasm"), test)]
        #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
        fn test_with_label_color() {
            let point = ChartPoint::new((0.0, 0.0), "Test".to_string())
                .with_label_color(RGBColor(0, 255, 0));
            assert_eq!(point.label_color, RGBColor(0, 255, 0));
        }

        #[cfg_attr(not(feature = "wasm"), test)]
        #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
        fn test_with_point_size() {
            let point = ChartPoint::new((0.0, 0.0), "Test".to_string()).with_point_size(10);
            assert_eq!(point.point_size, 10);
        }

        #[cfg_attr(not(feature = "wasm"), test)]
        #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
        fn test_with_font_size() {
            let point = ChartPoint::new((0.0, 0.0), "Test".to_string()).with_font_size(16);
            assert_eq!(point.font_size, 16);
        }

        #[cfg_attr(not(feature = "wasm"), test)]
        #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
        fn test_builder_chain() {
            let point = ChartPoint::new((0.0, 0.0), "Test".to_string())
                .with_point_color(RGBColor(255, 0, 0))
                .with_label_color(RGBColor(0, 255, 0))
                .with_point_size(10)
                .with_font_size(16)
                .with_absolute_offset(5.0, 5.0);

            assert_eq!(point.point_color, RGBColor(255, 0, 0));
            assert_eq!(point.label_color, RGBColor(0, 255, 0));
            assert_eq!(point.point_size, 10);
            assert_eq!(point.font_size, 16);
            assert_eq!(point.label_offset, LabelOffsetType::Absolute(5.0, 5.0));
        }
    }

    mod label_offset_type {
        use super::*;

        #[cfg_attr(not(feature = "wasm"), test)]
        #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
        fn test_label_offset_type_equality() {
            assert_eq!(
                LabelOffsetType::Absolute(1.0, 2.0),
                LabelOffsetType::Absolute(1.0, 2.0)
            );
            assert_eq!(
                LabelOffsetType::Relative(1.0, 2.0),
                LabelOffsetType::Relative(1.0, 2.0)
            );
            assert_eq!(LabelOffsetType::Auto, LabelOffsetType::Auto);

            assert_ne!(
                LabelOffsetType::Absolute(1.0, 2.0),
                LabelOffsetType::Absolute(2.0, 1.0)
            );
            assert_ne!(LabelOffsetType::Relative(1.0, 2.0), LabelOffsetType::Auto);
        }

        #[cfg_attr(not(feature = "wasm"), test)]
        #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
        fn test_label_offset_type_clone() {
            let offset = LabelOffsetType::Absolute(1.0, 2.0);
            let cloned = offset.clone();
            assert_eq!(offset, cloned);
        }
    }

    mod different_coordinate_types {
        use super::*;

        #[cfg_attr(not(feature = "wasm"), test)]
        #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
        fn test_integer_coordinates() {
            let point = ChartPoint::new((1, 2), "Test".to_string());
            assert_eq!(point.coordinates, (1, 2));
        }

        #[cfg_attr(not(feature = "wasm"), test)]
        #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
        fn test_float_coordinates() {
            let point = ChartPoint::new((1.5f32, 2.5f32), "Test".to_string());
            assert_eq!(point.coordinates, (1.5f32, 2.5f32));
        }

        #[cfg_attr(not(feature = "wasm"), test)]
        #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
        fn test_tuple_coordinates() {
            let point = ChartPoint::new(((1, 2), (3, 4)), "Test".to_string());
            assert_eq!(point.coordinates, ((1, 2), (3, 4)));
        }
    }
}

#[cfg(test)]
mod tests_label_offset {
    use super::*;
    #[cfg(feature = "wasm")]
    use wasm_bindgen_test::*;

    #[cfg(feature = "wasm")]
    wasm_bindgen_test_configure!(run_in_browser);

    #[cfg_attr(not(feature = "wasm"), test)]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn test_absolute_offset_get() {
        let offset = LabelOffsetType::Absolute(10.0, 20.0);
        let (x, y) = offset.get_offset();
        assert_eq!(x, 10.0);
        assert_eq!(y, 20.0);
    }

    #[cfg_attr(not(feature = "wasm"), test)]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn test_relative_offset_get() {
        let offset = LabelOffsetType::Relative(5.0, 15.0);
        let (x, y) = offset.get_offset();
        assert_eq!(x, 5.0);
        assert_eq!(y, 15.0);
    }

    #[cfg_attr(not(feature = "wasm"), test)]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn test_auto_offset_get() {
        let offset = LabelOffsetType::Auto;
        let (x, y) = offset.get_offset();
        assert_eq!(x, 2.0);
        assert_eq!(y, 2.0);
    }

    #[cfg_attr(not(feature = "wasm"), test)]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn test_negative_values() {
        let offset = LabelOffsetType::Absolute(-10.0, -20.0);
        let (x, y) = offset.get_offset();
        assert_eq!(x, -10.0);
        assert_eq!(y, -20.0);
    }

    #[cfg_attr(not(feature = "wasm"), test)]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn test_zero_values() {
        let offset = LabelOffsetType::Absolute(0.0, 0.0);
        let (x, y) = offset.get_offset();
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
    }

    #[cfg_attr(not(feature = "wasm"), test)]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn test_floating_point_precision() {
        let offset = LabelOffsetType::Relative(0.1, 0.2);
        let (x, y) = offset.get_offset();
        assert!((x - 0.1).abs() < f64::EPSILON);
        assert!((y - 0.2).abs() < f64::EPSILON);
    }

    #[cfg_attr(not(feature = "wasm"), test)]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn test_large_values() {
        let offset = LabelOffsetType::Absolute(f64::MAX / 2.0, f64::MAX / 2.0);
        let (x, y) = offset.get_offset();
        assert_eq!(x, f64::MAX / 2.0);
        assert_eq!(y, f64::MAX / 2.0);
    }

    #[cfg_attr(not(feature = "wasm"), test)]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn test_different_xy_values() {
        let test_cases = vec![
            LabelOffsetType::Absolute(1.0, 2.0),
            LabelOffsetType::Relative(3.0, 4.0),
            LabelOffsetType::Auto,
        ];

        for offset in test_cases {
            let (x, y) = offset.get_offset();
            match offset {
                LabelOffsetType::Absolute(ex, ey) => {
                    assert_eq!(x, ex);
                    assert_eq!(y, ey);
                }
                LabelOffsetType::Relative(ex, ey) => {
                    assert_eq!(x, ex);
                    assert_eq!(y, ey);
                }
                LabelOffsetType::Auto => {
                    assert_eq!(x, 2.0);
                    assert_eq!(y, 2.0);
                }
            }
        }
    }

    #[cfg_attr(not(feature = "wasm"), test)]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn test_clone_and_get_offset() {
        let original = LabelOffsetType::Absolute(5.0, 10.0);
        let cloned = original.clone();

        let (orig_x, orig_y) = original.get_offset();
        let (clone_x, clone_y) = cloned.get_offset();

        assert_eq!(orig_x, clone_x);
        assert_eq!(orig_y, clone_y);
    }

    #[cfg_attr(not(feature = "wasm"), test)]
    #[cfg_attr(feature = "wasm", wasm_bindgen_test)]
    fn test_multiple_calls_consistency() {
        let offset = LabelOffsetType::Relative(7.0, 14.0);

        let (x1, y1) = offset.get_offset();
        let (x2, y2) = offset.get_offset();

        assert_eq!(x1, x2);
        assert_eq!(y1, y2);
    }
}
