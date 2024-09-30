/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 30/9/24
******************************************************************************/
use plotters::prelude::RGBColor;

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
