/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 6/11/26
******************************************************************************/

//! Target crate (ADR-0001 D6): **simulation**. Owns `QuadraticRoughHestonError`.

use optionstratlib_core::error::DecimalError;
use rust_decimal::Decimal;
use thiserror::Error;

/// Reasons why a quadratic rough Heston kernel, simulation or price cannot be produced.
///
/// Raised by the constructors and functions around
/// [`QuadraticRoughHestonParams`](crate::simulation::QuadraticRoughHestonParams).
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum QuadraticRoughHestonError {
    /// The roughness exponent `alpha` is outside the open interval `(1/2, 1)`.
    #[error("alpha {alpha} is outside the open interval (1/2, 1)")]
    AlphaOutOfRange {
        /// The rejected exponent.
        alpha: Decimal,
    },

    /// The kernel was asked for no factors.
    #[error("the kernel needs at least one factor")]
    NoFactors,

    /// The geometric ratio `x_n` of the partition is not above one.
    #[error("the geometric ratio {ratio} is not above one")]
    RatioNotAboveOne {
        /// The rejected ratio.
        ratio: Decimal,
    },

    /// A kernel weight or discount has no positive value in a `Decimal`: it overflows,
    /// or it is smaller than the resolution of `1e-28`.
    #[error("the kernel {quantity} of factor {index} has no positive value in a decimal")]
    KernelOutOfRange {
        /// `weight` or `discount`.
        quantity: &'static str,
        /// The factor, from 1 to the number of factors.
        index: usize,
    },

    /// The time step is not positive: the horizon is not above zero, there are no steps,
    /// or the step rounds to zero in a `Decimal`.
    #[error("the step of a horizon of {horizon} years over {steps} steps is not positive")]
    NonPositiveStep {
        /// The horizon, in years.
        horizon: Decimal,
        /// The number of steps.
        steps: usize,
    },

    /// A model or run parameter is outside its domain.
    #[error("invalid parameter {parameter} = {value}: {reason}")]
    InvalidParameter {
        /// The rejected parameter.
        parameter: &'static str,
        /// The rejected value.
        value: Decimal,
        /// The condition the value breaks.
        reason: &'static str,
    },

    /// The number of initial factors is not the number of factors of the kernel.
    #[error("expected {expected} initial factors, got {actual}")]
    InitialFactorCount {
        /// The factors of the kernel.
        expected: usize,
        /// The initial factors supplied.
        actual: usize,
    },

    /// A value of the simulated state stopped being finite. The variance is not truncated,
    /// so this is how an exploding path is reported.
    #[error("the simulated {quantity} is not finite at step {step}")]
    NonFinite {
        /// `spot`, `z` or `variance`.
        quantity: &'static str,
        /// The step of the grid, from 1 to its number of steps.
        step: usize,
    },

    /// A buffer of the requested size cannot be allocated.
    #[error("cannot allocate {len} entries for the {buffer}")]
    BufferTooLarge {
        /// What the buffer holds.
        buffer: &'static str,
        /// The entries requested.
        len: usize,
    },

    /// A value does not fit a `Decimal`, or `Decimal` arithmetic failed.
    #[error(transparent)]
    Decimal(#[from] DecimalError),
}
