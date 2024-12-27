/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 12/12/24
******************************************************************************/
use std::error::Error;
use rust_decimal::Decimal;
use crate::model::option::Options;

#[derive(Debug, Clone)]
pub struct OptionsInStrike {
    pub long_call: Options,
    pub short_call: Options,
    pub long_put: Options,
    pub short_put: Options,
}

impl OptionsInStrike {
    pub fn new(
        long_call: Options,
        short_call: Options,
        long_put: Options,
        short_put: Options,
    ) -> OptionsInStrike {
        OptionsInStrike {
            long_call,
            short_call,
            long_put,
            short_put,
        }
    }

    pub fn deltas(&self) -> Result<DeltasInStrike, Box<dyn  Error>> {
        Ok(DeltasInStrike {
            long_call: self.long_call.delta()?,
            short_call: self.short_call.delta()?,
            long_put: self.long_put.delta()?,
            short_put: self.short_put.delta()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DeltasInStrike {
    pub long_call: Decimal,
    pub short_call: Decimal,
    pub long_put: Decimal,
    pub short_put: Decimal,
}
