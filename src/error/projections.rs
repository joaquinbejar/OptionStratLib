/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/9/25
******************************************************************************/
//! Target crate (ADR-0001 D2): **analytics**. Owns `ProjectionError`.
//!
//! A projection turns option data into a curve or a surface, so it can fail
//! for a market reason (the data), a pricing reason (a Greek) or a math
//! reason (building the curve). Analytics is the layer that composes all
//! three, so it owns the error and keeps every cause typed rather than
//! flattening it into a message (#511).

use crate::error::{ChainError, CurveError, GreeksError, OptionsError, SurfaceError};
use thiserror::Error;

/// Failure of a curve or surface projection built from option data.
#[derive(Debug, Error)]
pub enum ProjectionError {
    /// A Greek needed by the projection could not be computed.
    #[error("greek `{greek}` failed: {source}")]
    Greek {
        /// The Greek that failed, so the caller need not parse the message.
        greek: &'static str,
        /// The pricing failure itself.
        #[source]
        source: Box<GreeksError>,
    },

    /// Building or sampling the curve failed.
    #[error(transparent)]
    Curve(Box<CurveError>),

    /// Building or sampling the surface failed.
    #[error(transparent)]
    Surface(Box<SurfaceError>),

    /// The option data the projection reads is unusable.
    #[error(transparent)]
    Chain(Box<ChainError>),

    /// An option could not be built or priced for the projection.
    #[error(transparent)]
    Options(Box<OptionsError>),

    /// The option data a point needs could not be retrieved.
    #[error("option data unavailable for the projection")]
    MissingOptionData,

    /// The projection produced no point at all, so there is nothing to
    /// build a curve or a surface from.
    #[error("no valid point was generated for the {kind}")]
    NoPoints {
        /// What was being built, `curve` or `surface`.
        kind: &'static str,
    },

    /// The axis asked for is not supported by this projection.
    #[error("unsupported axis `{axis}` for this projection")]
    UnsupportedAxis {
        /// The axis the caller asked for.
        axis: String,
    },
}

impl ProjectionError {
    /// Wraps a Greek failure, naming the Greek.
    #[cold]
    #[inline(never)]
    #[must_use]
    pub fn greek(greek: &'static str, source: GreeksError) -> Self {
        ProjectionError::Greek {
            greek,
            source: Box::new(source),
        }
    }
}

impl From<CurveError> for ProjectionError {
    #[inline]
    fn from(error: CurveError) -> Self {
        ProjectionError::Curve(Box::new(error))
    }
}

impl From<SurfaceError> for ProjectionError {
    #[inline]
    fn from(error: SurfaceError) -> Self {
        ProjectionError::Surface(Box::new(error))
    }
}

impl From<ChainError> for ProjectionError {
    #[inline]
    fn from(error: ChainError) -> Self {
        ProjectionError::Chain(Box::new(error))
    }
}

impl From<OptionsError> for ProjectionError {
    #[inline]
    fn from(error: OptionsError) -> Self {
        ProjectionError::Options(Box::new(error))
    }
}

impl From<GreeksError> for ProjectionError {
    #[inline]
    fn from(error: GreeksError) -> Self {
        ProjectionError::greek("unspecified", error)
    }
}
