/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 12/7/26
******************************************************************************/
//! Deterministic walkers for generator tests.
//!
//! `RampWalker` replaces the RNG-driven default walk methods with a fixed
//! arithmetic ramp so multi-step generator tests can assert exact prices,
//! indices and dates without seeds or wall-clock dependence.

use crate::error::SimulationError;
use crate::simulation::{WalkParams, WalkTypeAble};
use positive::Positive;
use std::convert::TryInto;
use std::fmt::Display;
use std::ops::AddAssign;

/// Test walker producing the deterministic ramp
/// `start, start + delta, start + 2·delta, …` (length `params.size`) for
/// every synthetic walk type. `Historical` keeps the default trait behavior
/// (slice of the provided prices) so its data-validation semantics stay
/// testable.
#[derive(Clone)]
pub(crate) struct RampWalker {
    /// Price increment per step.
    pub delta: Positive,
}

impl RampWalker {
    fn ramp<X, Y>(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, SimulationError>
    where
        X: Copy + TryInto<Positive> + AddAssign + Display,
        Y: TryInto<Positive> + Display + Clone,
    {
        let start = params.ystep_as_positive()?;
        Ok((0..params.size)
            .map(|i| start + self.delta * i as f64)
            .collect())
    }
}

impl<X, Y> WalkTypeAble<X, Y> for RampWalker
where
    X: Copy + TryInto<Positive> + AddAssign + Display,
    Y: TryInto<Positive> + Display + Clone,
{
    fn brownian(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, SimulationError> {
        self.ramp(params)
    }

    fn geometric_brownian(
        &self,
        params: &WalkParams<X, Y>,
    ) -> Result<Vec<Positive>, SimulationError> {
        self.ramp(params)
    }

    fn log_returns(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, SimulationError> {
        self.ramp(params)
    }

    fn mean_reverting(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, SimulationError> {
        self.ramp(params)
    }

    fn jump_diffusion(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, SimulationError> {
        self.ramp(params)
    }

    fn garch(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, SimulationError> {
        self.ramp(params)
    }

    fn heston(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, SimulationError> {
        self.ramp(params)
    }

    fn custom(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, SimulationError> {
        self.ramp(params)
    }

    fn telegraph(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, SimulationError> {
        self.ramp(params)
    }
}

/// Test walker whose every walk method returns an empty path, for pinning
/// the empty-output contract of the generators.
#[derive(Clone)]
pub(crate) struct EmptyWalker;

impl<X, Y> WalkTypeAble<X, Y> for EmptyWalker
where
    X: Copy + TryInto<Positive> + AddAssign + Display,
    Y: TryInto<Positive> + Display + Clone,
{
    fn brownian(&self, _params: &WalkParams<X, Y>) -> Result<Vec<Positive>, SimulationError> {
        Ok(Vec::new())
    }

    fn geometric_brownian(
        &self,
        _params: &WalkParams<X, Y>,
    ) -> Result<Vec<Positive>, SimulationError> {
        Ok(Vec::new())
    }
}
