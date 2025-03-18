/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 18/3/25
******************************************************************************/
use crate::Positive;
use crate::chains::chain::OptionChain;
use crate::simulation::types::Walktypable;
use crate::simulation::{RandomWalkGraph, Walkable};
use rust_decimal::Decimal;
use std::error::Error;

impl Walkable<Positive, OptionChain> for OptionChain {
    fn get_y_values(&self) -> &Vec<OptionChain> {
        todo!()
    }

    fn get_y_values_ref(&mut self) -> &mut Vec<OptionChain> {
        todo!()
    }

    fn get_random_walk(&self) -> Result<RandomWalkGraph<OptionChain>, Box<dyn Error>> {
        todo!()
    }
}

impl Walktypable for OptionChain {
    fn walk_next(&self, _exp: f64) -> Result<Self, Box<dyn Error>> {
        todo!()
    }

    fn walk_max(&self) -> Result<Self, Box<dyn Error>> {
        todo!()
    }

    fn walk_dec(&self) -> Result<Decimal, Box<dyn Error>> {
        todo!()
    }

    fn walk_positive(&self) -> Result<Positive, Box<dyn Error>> {
        todo!()
    }
}
