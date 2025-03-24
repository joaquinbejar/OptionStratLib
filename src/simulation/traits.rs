use crate::Positive;
use crate::model::decimal::decimal_normal_sample;
use crate::simulation::walk::{WalkParams, WalkType};
use crate::volatility::generate_ou_process;
use rust_decimal::{Decimal, MathematicalOps};
use std::error::Error;
use std::fmt::{Debug, Display};
use std::ops::AddAssign;

pub trait WalkTypeAble<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display ,
    Y: Copy + Into<Positive> + Display ,
{
    
    
    fn brownian(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, Box<dyn Error>> {
        match params.walk_type {
            WalkType::Brownian {
                dt,
                drift,
                volatility,
            } => {
                let mut values = Vec::new();
                let mut current_value: Positive = (*params.init_step.y.value()).into();
                for _ in 0..params.size {
                    let random_step = decimal_normal_sample() * volatility * dt;
                    current_value += drift * dt + random_step;
                    values.push(current_value);
                }
                Ok(values)
            }
            _ => Err("Invalid walk type for Brownian motion".into()),
        }
    }

    fn geometric_brownian(
        &self,
        params: &WalkParams<X, Y>,
    ) -> Result<Vec<Positive>, Box<dyn Error>> {
        match params.walk_type {
            WalkType::GeometricBrownian {
                dt,
                drift,
                volatility,
            } => {
                let mut values = Vec::new();
                let mut current_value: Positive = (*params.init_step.y.value()).into();
                for _ in 0..params.size {
                    let random_step = decimal_normal_sample() * volatility * dt;
                    current_value *= (drift * dt + random_step).exp();
                    values.push(current_value);
                }
                Ok(values)
            }
            _ => Err("Invalid walk type for Geometric Brownian motion".into()),
        }
    }

    fn log_returns(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, Box<dyn Error>> {
        match params.walk_type {
            WalkType::LogReturns {
                dt,
                expected_return,
                volatility,
                autocorrelation,
            } => {
                let mut values = Vec::new();
                let mut current_value: Positive = (*params.init_step.y.value()).into();
                let mut prev_log_return = Decimal::ZERO;
                for _ in 0..params.size {
                    let random_step = decimal_normal_sample() * volatility * dt;
                    let mut log_return = expected_return * dt + random_step;
                    if let Some(ac) = autocorrelation {
                        assert!(ac <= Decimal::ONE && ac >= -Decimal::ONE);
                        log_return += ac * prev_log_return;
                    }
                    current_value *= (log_return).exp();
                    values.push(current_value);
                    prev_log_return = log_return;
                }
                Ok(values)
            }
            _ => Err("Invalid walk type for Log Returns motion".into()),
        }
    }

    fn mean_reverting(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, Box<dyn Error>> {
        match params.walk_type {
            WalkType::MeanReverting {
                dt,
                volatility,
                speed,
                mean,
            } => Ok(generate_ou_process(
                (*params.init_step.y.value()).into(),
                mean,
                speed,
                volatility,
                dt,
                params.size,
            )),
            _ => Err("Invalid walk type for Mean Reverting motion".into()),
        }
    }

    fn jump_diffusion(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, Box<dyn Error>> {
        match params.walk_type {
            WalkType::JumpDiffusion {
                dt,
                drift,
                volatility,
                intensity,
                jump_mean,
                jump_volatility,
            } => {
                let mut values = Vec::new();
                let mut current_value: Positive = (*params.init_step.y.value()).into();
                for _ in 0..params.size {
                    let random_step = decimal_normal_sample() * volatility * dt;
                    current_value += drift * dt + random_step;
                    if decimal_normal_sample() < intensity.to_dec() {
                        let jump = jump_mean + jump_volatility * decimal_normal_sample();
                        current_value += jump;
                    }
                    values.push(current_value);
                }
                Ok(values)
            }
            _ => Err("Invalid walk type for Jump Diffusion motion".into()),
        }
    }

    fn garch(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, Box<dyn Error>> {
        match params.walk_type {
            WalkType::Garch { .. } => {
                // Implement GARCH process simulation here
                Ok(vec![])
            }
            _ => Err("Invalid walk type for GARCH motion".into()),
        }
    }

    fn heston(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, Box<dyn Error>> {
        match params.walk_type {
            WalkType::Heston { .. } => {
                // Implement Heston process simulation here
                Ok(vec![])
            }
            _ => Err("Invalid walk type for Heston motion".into()),
        }
    }

    fn custom(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, Box<dyn Error>> {
        match params.walk_type {
            WalkType::Custom {
                dt,
                drift,
                volatility,
                vov,
                vol_speed,
                vol_mean,
            } => {
                let volatilities =
                    generate_ou_process(volatility, vol_mean, vol_speed, vov, dt, params.size);
                let mut values = Vec::new();

                // Fix for the Positive conversion issue:
                // Clone the value first (since it's a reference) and then convert to Positive
                let mut current_value: Positive = (*params.init_step.y.value()).into();

                // Use iterator instead of index-based loop
                for &vol in volatilities.iter().take(params.size) {
                    let random_step = decimal_normal_sample() * vol * dt;
                    current_value += drift * dt + random_step;
                    values.push(current_value);
                }
                Ok(values)
            }
            _ => Err("Invalid walk type for Custom motion".into()),
        }
    }
}


impl<X, Y> Debug for Box<dyn WalkTypeAble<X, Y>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WalkTypeAble")
    }
}

impl<X, Y> Clone for Box<dyn WalkTypeAble<X, Y>> {
    fn clone(&self) -> Self {
        todo!()
    }
}