//! # Visualization
//!
//! This module provides tools for visualizing financial data and option strategies using charts and diagrams.
//!
//! ## Overview
//!
//! The visualization module offers a set of utilities for generating visual representations of financial data,
//! particularly focused on options pricing and trading strategies. It leverages the `plotters` library
//! for rendering high-quality charts and diagrams with cross-platform support.
//!
//! ## Module Structure
//!
//! - **binomial_tree**: Tools for visualizing binomial tree models used in options pricing.
//! - **model**: Data structures that represent visual elements like points, lines, and styling information.
//! - **utils**: Common utilities and traits for chart rendering, including the `Graph` trait and backend definitions.
//!
//! ## Key Features
//!
//! - Platform-agnostic rendering with support for both native applications and WebAssembly
//! - Consistent styling and theming for financial visualizations
//! - Specialized components for options strategy visualization
//! - Flexible backend system allowing output to bitmap images or HTML5 canvas
//!
//! ## Cross-Platform Support
//!
//! The visualization module is designed to work across different platforms:
//!
//! - **Native Applications**: Charts can be rendered to bitmap images (PNG, etc.)
//! - **WebAssembly**: When compiled to WebAssembly, charts can be rendered directly to HTML5 canvas elements
//!
//! The appropriate backend is selected automatically based on compilation targets.

/// This sub-module contains the implementation of the binomial tree model.
pub mod binomial_tree;

/// This sub-module contains the model structures and traits.  It's marked as `pub(crate)`
/// because it's primarily intended for internal use within the crate, although some
/// items might be exposed through re-exports in other modules.
pub(crate) mod model;

/// This sub-module contains various utility functions used throughout the crate.
pub mod utils;
