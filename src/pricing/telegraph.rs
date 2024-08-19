/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/8/24
******************************************************************************/

use crate::constants::ZERO;
use crate::model::option::Options;
use rand::Rng;

#[derive(Clone)]
pub struct TelegraphProcess {
    lambda_up: f64,
    lambda_down: f64,
    current_state: i8,
}

impl TelegraphProcess {
    pub fn new(lambda_up: f64, lambda_down: f64) -> Self {
        let initial_state = if rand::random::<f64>() < 0.5 { 1 } else { -1 };
        TelegraphProcess {
            lambda_up,
            lambda_down,
            current_state: initial_state,
        }
    }

    pub fn next_state(&mut self, dt: f64) -> i8 {
        let lambda = if self.current_state == 1 {
            self.lambda_down
        } else {
            self.lambda_up
        };
        let probability = 1.0 - (-lambda * dt).exp();

        if rand::thread_rng().gen::<f64>() < probability {
            self.current_state *= -1;
        }

        self.current_state
    }

    pub fn get_current_state(&self) -> i8 {
        self.current_state
    }
}

pub fn telegraph(option: &Options, no_steps: usize) -> f64 {
    let mut price = option.underlying_price;
    let dt = option.time_to_expiration() / no_steps as f64;

    let lambda_up = ZERO; // TODO: fix this
    let lambda_down = ZERO; // TODO: fix this

    let telegraph_process = TelegraphProcess::new(lambda_up, lambda_down);

    if let tp = telegraph_process {
        let mut telegraph_process = tp.clone();

        for _ in 0..no_steps {
            let state = telegraph_process.next_state(dt);
            let drift = option.risk_free_rate - 0.5 * option.implied_volatility.powi(2);
            let volatility = option.implied_volatility * state as f64;

            price *= (drift * dt + volatility * (dt.sqrt() * rand::random::<f64>())).exp();
        }
    }

    let payoff = option.payoff_at_price(price);
    payoff * (-option.risk_free_rate * option.time_to_expiration()).exp()
}


// TODO: Unit Tests