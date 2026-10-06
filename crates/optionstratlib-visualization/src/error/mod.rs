//! Errors owned by the visualization layer.
//!
//! `GraphError` reports a chart that could not be built or written; it wraps
//! the math errors of the curves and surfaces it charts, each typed, and
//! those types stay with their own crate.

/// ### Graph Errors (`GraphError`)
mod graph;

pub use graph::GraphError;
