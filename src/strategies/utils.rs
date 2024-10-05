/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/8/24
******************************************************************************/
use crate::model::types::PositiveF64;

pub enum FindOptimalSide {
    Upper,
    Lower,
    All,
    Range(PositiveF64, PositiveF64),
}

pub(crate) enum OptimizationCriteria {
    Ratio,
    Area,
}

pub(crate) fn calculate_price_range(
    start_price: PositiveF64,
    end_price: PositiveF64,
    step: PositiveF64,
) -> Vec<PositiveF64> {
    let mut range = Vec::new();
    let mut current_price = start_price;
    while current_price <= end_price {
        range.push(current_price);
        current_price += step;
    }
    range
}
