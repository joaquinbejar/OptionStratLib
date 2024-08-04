use crate::pricing::payoff::Payoff;

#[allow(dead_code)]
pub enum OptionType {
    European,
    American,
    Bermuda {
        exercise_dates: Vec<f64>,
    },
    Asian {
        averaging_type: AsianAveragingType,
    },
    Barrier {
        barrier_type: BarrierType,
        barrier_level: f64,
    },
    Binary {
        binary_type: BinaryType,
    },
    Lookback {
        lookback_type: LookbackType,
    },
    Compound {
        underlying_option: Box<OptionType>,
    },
    Chooser {
        choice_date: f64,
    },
    Cliquet {
        reset_dates: Vec<f64>,
    },
    Rainbow {
        num_assets: usize,
    },
    Spread {
        second_asset: f64,
    },
    Quanto {
        exchange_rate: f64,
    },
    Exchange {
        second_asset: f64,
    },
    Power {
        exponent: f64,
    },
}

#[allow(dead_code)]
pub enum AsianAveragingType {
    Arithmetic,
    Geometric,
}

#[allow(dead_code)]
pub enum BarrierType {
    UpAndIn,
    UpAndOut,
    DownAndIn,
    DownAndOut,
}

#[allow(dead_code)]
pub enum BinaryType {
    CashOrNothing,
    AssetOrNothing,
}

#[allow(dead_code)]
pub enum LookbackType {
    FloatingStrike,
    FixedStrike,
}

impl Payoff for OptionType {
    fn payoff(&self, spot: f64, strike: f64) -> f64 {
        match self {
            OptionType::European | OptionType::American => (spot - strike).max(0.0),
            // TODO: Implement payoff for other types of options
            _ => panic!("Payoff not implemented for this option type"),
        }
    }
}
