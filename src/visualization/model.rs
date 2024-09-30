/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 30/9/24
******************************************************************************/
use plotters::prelude::{RGBColor, ShapeStyle};

#[derive(Clone)]
pub struct ChartPoint<T> {
    pub coordinates: T,
    pub label: String,
    pub label_offset: (f64, f64),
    pub(crate) point_color: RGBColor,
    pub(crate) label_color: RGBColor,
    pub(crate) point_size: u32,
    pub(crate) font_size: u32,
}

#[derive(Clone)]
pub struct ChartVerticalLine<X, Y> {
    pub x_coordinate: X,
    pub y_range: (Y, Y), // (y_start, y_end)
    pub label: String,
    pub label_offset: (f64, f64),
    pub(crate) line_color: RGBColor,
    pub(crate) label_color: RGBColor,
    pub(crate) line_style: ShapeStyle,
    pub(crate) font_size: u32,
}