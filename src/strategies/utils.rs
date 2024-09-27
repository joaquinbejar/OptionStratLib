/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/8/24
******************************************************************************/

pub enum FindOptimalSide {
    Upper,
    Lower,
    All,
}

pub(crate) enum OptimizationCriteria {
    Ratio,
    Area,
}

pub(crate) fn calculate_price_range(start_price: f64, end_price: f64, step: f64) -> Vec<f64> {
    let mut range = Vec::new();
    let mut current_price = start_price;
    while current_price <= end_price {
        range.push(current_price);
        current_price += step;
    }
    range
}
