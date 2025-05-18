/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/12/24
******************************************************************************/

//! # Chain Error Module
//!
//! This module provides error handling for operations related to option chains and their data.
//! It defines a comprehensive error hierarchy to handle various failure scenarios in option chain
//! operations, data validation, and file handling.
//!
//! ## Error Types
//!
//! * `ChainError` - The main error enum that encompasses all possible chain-related errors
//! * `OptionDataErrorKind` - Specific errors related to option data validation
//! * `ChainBuildErrorKind` - Errors that can occur during chain construction
//! * `FileErrorKind` - File operation related errors
//! * `StrategyErrorKind` - Strategy-specific validation errors
//!
//! ## Usage Example
//!
//! ```rust
//! use optionstratlib::error::chains::ChainError;
//!
//! fn validate_strike_price(strike: f64) -> Result<(), ChainError> {
//!     if strike <= 0.0 {
//!         return Err(ChainError::invalid_strike(
//!             strike,
//!             "Strike price must be positive"
//!         ));
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Error Creation Helpers
//!
//! The module provides several helper methods for creating common errors:
//!
//! * `invalid_strike` - Creates an error for invalid strike prices
//! * `invalid_volatility` - Creates an error for invalid volatility values
//! * `invalid_prices` - Creates an error for invalid bid/ask prices
//! * `invalid_legs` - Creates an error for invalid strategy legs
//! * `invalid_parameters` - Creates an error for invalid chain building parameters
//!
//! ## Conversions
//!
//! The module implements various conversion traits:
//!
//! * `From<io::Error>` - Converts IO errors to chain errors
//! * `From<String>` - Converts string messages to price calculation errors
//!
//! All error types implement `std::error::Error` and `std::fmt::Display` for proper error
//! handling and formatting.

use crate::error::{GreeksError, OptionsError};
use std::error::Error;
use std::fmt;
use std::io;

/// # ChainError
///
/// Represents the various error categories that can occur during option chain operations.
///
/// This enum encapsulates different types of errors that might occur when working with
/// option chains, including data validation issues, chain construction problems,
/// file operations errors, and strategy-related errors.
///
/// # Variants
///
/// * `OptionDataError` - Errors related to option contract data validation, such as
///   invalid strikes, prices, volatility values or delta values.
///
/// * `ChainBuildError` - Errors that occur during the option chain construction process,
///   such as invalid parameters, volatility adjustment issues, or strike generation problems.
///
/// * `FileError` - Errors related to file operations when reading from or writing to
///   external files, including I/O errors, format issues, or parsing problems.
///
/// * `StrategyError` - Errors related to option trading strategies, including issues
///   with leg validation or invalid combinations of options.
///
/// * `DynError` - A generic error variant for capturing dynamic error messages that
///   don't fit into the other specific categories.
///
/// # Usage
///
/// This error type is used throughout the option chain functionality to provide
/// detailed information about what went wrong during chain operations, allowing
/// for proper error handling and debugging.
#[derive(Debug)]
pub enum ChainError {
    /// Errors related to option data validation
    ///
    /// This variant captures issues with individual option contract data,
    /// such as invalid strikes, volatility values, or price information.
    OptionDataError(OptionDataErrorKind),

    /// Errors related to chain building
    ///
    /// This variant represents problems that occur during the construction
    /// of option chains, including parameter validation and strike generation.
    ChainBuildError(ChainBuildErrorKind),

    /// Errors related to file operations
    ///
    /// This variant handles issues with reading, writing, or parsing files
    /// containing option chain data.
    FileError(FileErrorKind),

    /// Errors related to strategies
    ///
    /// This variant captures problems with option trading strategies,
    /// such as invalid combinations or incorrect leg configurations.
    StrategyError(StrategyErrorKind),

    /// Dynamic error with custom message
    ///
    /// This variant provides flexibility for error conditions that don't
    /// fit into the other specific categories.
    DynError {
        /// A descriptive message explaining the error
        message: String,
    },
}

/// Represents specific error types related to option data validation and calculations.
///
/// This enum encapsulates various error conditions that can occur during option data
/// processing, including validation failures for strike prices, implied volatility,
/// pricing information, and greeks calculations. Each variant provides structured
/// information about the error context and reason.
///
/// # Variants
///
/// * `InvalidStrike` - Errors related to option strike price validation.
/// * `InvalidVolatility` - Errors related to implied volatility values.
/// * `InvalidPrices` - Errors related to bid/ask price validation.
/// * `InvalidDelta` - Errors related to delta calculation or validation.
/// * `PriceCalculationError` - Errors occurring during option price calculations.
/// * `OtherError` - General errors that don't fit in other categories.
///
/// # Usage
///
/// This error type is typically used in option data validation, pricing models,
/// and options trading calculations where precise error reporting is needed
/// for debugging and user feedback.
///
/// # Example
///
/// ```rust
/// use optionstratlib::error::chains::OptionDataErrorKind;
///
/// fn validate_strike_price(strike: f64) -> Result<(), OptionDataErrorKind> {
///     if strike <= 0.0 {
///         return Err(OptionDataErrorKind::InvalidStrike {
///             strike,
///             reason: "Strike price must be positive".to_string(),
///         });
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub enum OptionDataErrorKind {
    /// Invalid strike price
    ///
    /// This variant is used when an option's strike price is invalid,
    /// such as being negative, zero, or otherwise unsuitable for options calculations.
    InvalidStrike {
        /// The problematic strike price value
        strike: f64,
        /// Detailed explanation of why the strike price is invalid
        reason: String,
    },

    /// Invalid implied volatility
    ///
    /// This variant is used when the implied volatility value doesn't meet
    /// the required criteria for option pricing or calculations.
    InvalidVolatility {
        /// The problematic volatility value that caused the error (if available)
        volatility: Option<f64>,
        /// Detailed explanation of why the volatility is invalid
        reason: String,
    },

    /// Invalid bid/ask prices
    ///
    /// This variant is used when the bid and/or ask prices are problematic,
    /// such as negative values, bid higher than ask, or other inconsistencies.
    InvalidPrices {
        /// The problematic bid price value (if available)
        bid: Option<f64>,
        /// The problematic ask price value (if available)
        ask: Option<f64>,
        /// Detailed explanation of why the prices are invalid
        reason: String,
    },

    /// Invalid delta
    ///
    /// This variant is used when the delta value for an option is outside
    /// acceptable bounds or otherwise unsuitable for options calculations.
    InvalidDelta {
        /// The problematic delta value that caused the error (if available)
        delta: Option<f64>,
        /// Detailed explanation of why the delta is invalid
        reason: String,
    },

    /// Error in price calculation
    ///
    /// This variant captures various errors that can occur during
    /// option price calculation processes.
    PriceCalculationError(String),

    /// Other errors related to option data
    ///
    /// A general-purpose variant for errors that don't fit into other categories.
    OtherError(String),
}

/// Enum representing specific errors that can occur during option chain building processes.
///
/// This enum captures three main categories of errors that can arise when constructing or
/// manipulating option chains. It provides structured error information with detailed context
/// about what went wrong during the chain building process.
///
/// # Variants
///
/// * `InvalidParameters` - Represents errors related to invalid input parameters
///   provided for chain construction, such as invalid price ranges, dates, or
///   configuration settings.
///
/// * `VolatilityAdjustmentError` - Represents errors that occur during volatility
///   adjustment calculations, typically related to skew factors or model parameters
///   that cannot be properly applied.
///
/// * `StrikeGenerationError` - Represents errors that occur when generating strike
///   prices for an option chain, such as invalid intervals, reference prices outside
///   acceptable ranges, or other strike-specific issues.
///
/// # Usage
///
/// This error type is typically used in functions that build or modify option chains,
/// especially those dealing with strike price generation, volatility adjustments,
/// and parameter validation.
#[derive(Debug)]
pub enum ChainBuildErrorKind {
    /// Error indicating invalid parameters were provided for chain building
    ///
    /// This variant is used when one or more input parameters don't meet
    /// the required criteria for option chain construction.
    InvalidParameters {
        /// Name of the parameter that caused the error
        parameter: String,
        /// Detailed explanation of why the parameter is invalid
        reason: String,
    },

    /// Error in volatility adjustment calculations
    ///
    /// This variant is used when the system encounters problems adjusting
    /// volatility values, typically related to skew modeling.
    VolatilityAdjustmentError {
        /// The skew factor that caused the error
        smile_curve: f64,
        /// Detailed explanation of the volatility adjustment issue
        reason: String,
    },

    /// Error in strike price generation
    ///
    /// This variant is used when the system cannot properly generate
    /// strike prices for the option chain.
    StrikeGenerationError {
        /// The reference price used as a basis for strike generation
        reference_price: f64,
        /// The interval between strikes that was attempted
        interval: f64,
        /// Detailed explanation of the strike generation issue
        reason: String,
    },
}

/// Enum representing errors related to file operations in the system.
///
/// This enum captures three main categories of errors that can occur during file handling
/// operations: I/O errors from the standard library, format validation errors, and
/// data parsing errors. It provides structured error information that includes detailed
/// reasons for the error occurrence.
///
/// # Variants
///
/// * `IOError` - Represents low-level input/output errors from the Rust standard library,
///   such as permission issues, file not found, or network failures when reading files.
///
/// * `InvalidFormat` - Represents errors related to file format validation,
///   such as incorrect headers, unsupported format versions, or malformed data structures.
///
/// * `ParseError` - Represents errors occurring during the parsing of file contents,
///   providing context about the specific line and content that caused the error.
///
/// # Usage
///
/// This error type is typically used in file handling operations, data import/export
/// functionality, and file-based data processing pipelines where structured error
/// reporting is important for debugging and user feedback.
#[derive(Debug)]
pub enum FileErrorKind {
    /// Error when reading or writing a file
    ///
    /// This variant wraps a standard library I/O error and is used for low-level
    /// file system operations that fail.
    IOError(io::Error),

    /// Error indicating an invalid file format
    ///
    /// This variant is used when the file's overall structure or format doesn't
    /// match what's expected by the application.
    InvalidFormat {
        /// The format that was being processed (e.g., "CSV", "JSON", "XML")
        format: String,
        /// Detailed explanation of why the format is invalid
        reason: String,
    },

    /// Error occurring during parsing of file contents
    ///
    /// This variant provides detailed context about parsing errors, including
    /// the specific line where the error occurred and its content.
    ParseError {
        /// The line number where the parsing error occurred (0-based or 1-based, depending on implementation)
        line: usize,
        /// The content of the problematic line
        content: String,
        /// Detailed explanation of the parsing error
        reason: String,
    },
}

/// Enum representing specific error types that can occur in options trading strategies.
///
/// This enum captures errors that may arise during the validation and execution of
/// options trading strategies. It provides structured error information including
/// detailed reasons for various strategy-related failures.
///
/// # Variants
///
/// * `InvalidLegs` - Represents errors when the number of strategy legs doesn't match
///   the expected count for a particular strategy type.
///
/// * `InvalidCombination` - Represents errors when the combination of options in a
///   strategy doesn't meet the required criteria (strike prices, expiration dates,
///   option types, etc.).
///
/// # Usage
///
/// This error type is typically used in strategy validation, construction, and analysis
/// to provide specific feedback about why a particular options combination doesn't
/// constitute a valid strategy.
#[derive(Debug, PartialEq)]
pub enum StrategyErrorKind {
    /// Error indicating an invalid number of legs in a strategy
    ///
    /// This variant is used when the provided legs don't match the required
    /// count for a specific strategy type.
    InvalidLegs {
        /// The number of legs expected for the strategy
        expected: usize,
        /// The actual number of legs provided
        found: usize,
        /// Detailed explanation of why the legs configuration is invalid
        reason: String,
    },

    /// Error indicating an invalid combination of options for a strategy
    ///
    /// This variant is used when the provided options don't form a valid
    /// strategy according to specific rules (e.g., strike price relationships,
    /// expiration alignment, option types).
    InvalidCombination {
        /// The type of strategy being validated (e.g., "Iron Condor", "Butterfly")
        strategy_type: String,
        /// Detailed explanation of why the combination is invalid
        reason: String,
    },
}

impl fmt::Display for ChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChainError::OptionDataError(err) => write!(f, "Option data error: {}", err),
            ChainError::ChainBuildError(err) => write!(f, "Chain build error: {}", err),
            ChainError::FileError(err) => write!(f, "File error: {}", err),
            ChainError::StrategyError(err) => write!(f, "Strategy error: {}", err),
            ChainError::DynError { message } => write!(f, "Error: {}", message),
        }
    }
}

impl fmt::Display for OptionDataErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptionDataErrorKind::InvalidStrike { strike, reason } => {
                write!(f, "Invalid strike price {}: {}", strike, reason)
            }
            OptionDataErrorKind::InvalidVolatility { volatility, reason } => {
                write!(
                    f,
                    "Invalid volatility {:?}: {}",
                    volatility.unwrap_or(0.0),
                    reason
                )
            }
            OptionDataErrorKind::InvalidPrices { bid, ask, reason } => {
                write!(
                    f,
                    "Invalid prices (bid: {:?}, ask: {:?}): {}",
                    bid, ask, reason
                )
            }
            OptionDataErrorKind::InvalidDelta { delta, reason } => {
                write!(f, "Invalid delta {:?}: {}", delta, reason)
            }
            OptionDataErrorKind::PriceCalculationError(msg) => {
                write!(f, "Price calculation error: {}", msg)
            }
            OptionDataErrorKind::OtherError(msg) => write!(f, "{}", msg),
        }
    }
}

impl fmt::Display for ChainBuildErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChainBuildErrorKind::InvalidParameters { parameter, reason } => {
                write!(f, "Invalid parameter '{}': {}", parameter, reason)
            }
            ChainBuildErrorKind::VolatilityAdjustmentError {
                smile_curve,
                reason,
            } => {
                write!(
                    f,
                    "Volatility adjustment error (skew factor: {}): {}",
                    smile_curve, reason
                )
            }
            ChainBuildErrorKind::StrikeGenerationError {
                reference_price,
                interval,
                reason,
            } => {
                write!(
                    f,
                    "Strike generation error (reference: {}, interval: {}): {}",
                    reference_price, interval, reason
                )
            }
        }
    }
}

impl fmt::Display for FileErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileErrorKind::IOError(err) => write!(f, "IO error: {}", err),
            FileErrorKind::InvalidFormat { format, reason } => {
                write!(f, "Invalid {} format: {}", format, reason)
            }
            FileErrorKind::ParseError {
                line,
                content,
                reason,
            } => {
                write!(
                    f,
                    "Parse error at line {}, content '{}': {}",
                    line, content, reason
                )
            }
        }
    }
}

impl fmt::Display for StrategyErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StrategyErrorKind::InvalidLegs {
                expected,
                found,
                reason,
            } => {
                write!(
                    f,
                    "Invalid number of legs (expected: {}, found: {}): {}",
                    expected, found, reason
                )
            }
            StrategyErrorKind::InvalidCombination {
                strategy_type,
                reason,
            } => {
                write!(
                    f,
                    "Invalid combination for strategy '{}': {}",
                    strategy_type, reason
                )
            }
        }
    }
}

impl Error for ChainError {}

impl From<io::Error> for ChainError {
    fn from(error: io::Error) -> Self {
        ChainError::FileError(FileErrorKind::IOError(error))
    }
}

impl From<OptionsError> for ChainError {
    fn from(error: OptionsError) -> Self {
        ChainError::OptionDataError(OptionDataErrorKind::PriceCalculationError(
            error.to_string(),
        ))
    }
}

/// Implementation of factory methods for creating specific `ChainError` variants.
///
/// This implementation provides convenient factory methods for creating different types of errors
/// that may occur during option chain operations. These methods simplify error creation by
/// handling the construction of nested error types and providing a consistent interface.
impl ChainError {
    /// Creates a new error for invalid strike prices.
    ///
    /// This method constructs an `OptionDataError` with the `InvalidStrike` variant when
    /// a strike price fails validation checks.
    ///
    /// # Parameters
    ///
    /// * `strike` - The invalid strike price value that caused the error
    /// * `reason` - A description explaining why the strike price is invalid
    ///
    /// # Returns
    ///
    /// A `ChainError` containing the strike validation error details
    pub fn invalid_strike(strike: f64, reason: &str) -> Self {
        ChainError::OptionDataError(OptionDataErrorKind::InvalidStrike {
            strike,
            reason: reason.to_string(),
        })
    }

    /// Creates a new error for invalid volatility values.
    ///
    /// This method constructs an `OptionDataError` with the `InvalidVolatility` variant when
    /// a volatility value fails validation checks or is missing when required.
    ///
    /// # Parameters
    ///
    /// * `volatility` - The invalid or missing volatility value, wrapped in an Option
    /// * `reason` - A description explaining why the volatility is invalid
    ///
    /// # Returns
    ///
    /// A `ChainError` containing the volatility validation error details
    pub fn invalid_volatility(volatility: Option<f64>, reason: &str) -> Self {
        ChainError::OptionDataError(OptionDataErrorKind::InvalidVolatility {
            volatility,
            reason: reason.to_string(),
        })
    }

    /// Creates a new error for invalid price data.
    ///
    /// This method constructs an `OptionDataError` with the `InvalidPrices` variant when
    /// bid and/or ask prices fail validation checks or are missing when required.
    ///
    /// # Parameters
    ///
    /// * `bid` - The potentially invalid or missing bid price
    /// * `ask` - The potentially invalid or missing ask price
    /// * `reason` - A description explaining why the prices are invalid
    ///
    /// # Returns
    ///
    /// A `ChainError` containing the price validation error details
    pub fn invalid_prices(bid: Option<f64>, ask: Option<f64>, reason: &str) -> Self {
        ChainError::OptionDataError(OptionDataErrorKind::InvalidPrices {
            bid,
            ask,
            reason: reason.to_string(),
        })
    }

    /// Creates a new error for invalid strategy legs.
    ///
    /// This method constructs a `StrategyError` with the `InvalidLegs` variant when
    /// the number of legs in an options strategy doesn't match expectations.
    ///
    /// # Parameters
    ///
    /// * `expected` - The expected number of legs for the strategy
    /// * `found` - The actual number of legs provided
    /// * `reason` - A description explaining why the leg configuration is invalid
    ///
    /// # Returns
    ///
    /// A `ChainError` containing the strategy legs validation error details
    pub fn invalid_legs(expected: usize, found: usize, reason: &str) -> Self {
        ChainError::StrategyError(StrategyErrorKind::InvalidLegs {
            expected,
            found,
            reason: reason.to_string(),
        })
    }

    /// Creates a new error for invalid chain building parameters.
    ///
    /// This method constructs a `ChainBuildError` with the `InvalidParameters` variant when
    /// parameters used to build an option chain fail validation checks.
    ///
    /// # Parameters
    ///
    /// * `parameter` - The name of the invalid parameter
    /// * `reason` - A description explaining why the parameter is invalid
    ///
    /// # Returns
    ///
    /// A `ChainError` containing the parameter validation error details
    pub fn invalid_parameters(parameter: &str, reason: &str) -> Self {
        ChainError::ChainBuildError(ChainBuildErrorKind::InvalidParameters {
            parameter: parameter.to_string(),
            reason: reason.to_string(),
        })
    }
}

impl From<String> for ChainError {
    fn from(msg: String) -> Self {
        ChainError::OptionDataError(OptionDataErrorKind::PriceCalculationError(msg))
    }
}

impl From<GreeksError> for ChainError {
    fn from(err: GreeksError) -> Self {
        ChainError::OptionDataError(OptionDataErrorKind::OtherError(err.to_string()))
    }
}

impl From<Box<dyn Error>> for ChainError {
    fn from(error: Box<dyn Error>) -> Self {
        ChainError::DynError {
            message: error.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_data_errors() {
        let error = ChainError::invalid_strike(-10.0, "Strike cannot be negative");
        assert!(matches!(
            error,
            ChainError::OptionDataError(OptionDataErrorKind::InvalidStrike { .. })
        ));

        let error = ChainError::invalid_volatility(Some(-0.5), "Volatility must be positive");
        assert!(matches!(
            error,
            ChainError::OptionDataError(OptionDataErrorKind::InvalidVolatility { .. })
        ));
    }

    #[test]
    fn test_error_messages() {
        let error = ChainError::invalid_strike(0.0, "Strike must be positive");
        assert!(error.to_string().contains("Strike must be positive"));
    }

    #[test]
    fn test_chain_build_errors() {
        let error = ChainError::ChainBuildError(ChainBuildErrorKind::InvalidParameters {
            parameter: "chain_size".to_string(),
            reason: "Must be greater than 0".to_string(),
        });
        assert!(error.to_string().contains("chain_size"));
        assert!(error.to_string().contains("Must be greater than 0"));
    }

    #[test]
    fn test_strategy_errors() {
        let error = ChainError::invalid_legs(4, 3, "Iron Condor requires exactly 4 legs");
        assert!(error.to_string().contains("4"));
        assert!(error.to_string().contains("3"));
        assert!(error.to_string().contains("Iron Condor"));
    }

    #[test]
    fn test_file_errors() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let error = ChainError::from(io_error);
        assert!(matches!(
            error,
            ChainError::FileError(FileErrorKind::IOError(..))
        ));
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;

    #[test]
    fn test_chain_build_error_display() {
        let error = ChainBuildErrorKind::InvalidParameters {
            parameter: "size".to_string(),
            reason: "must be positive".to_string(),
        };
        assert!(error.to_string().contains("size"));
        assert!(error.to_string().contains("must be positive"));

        let error = ChainBuildErrorKind::VolatilityAdjustmentError {
            smile_curve: 0.5,
            reason: "invalid adjustment".to_string(),
        };
        assert!(error.to_string().contains("0.5"));
        assert!(error.to_string().contains("invalid adjustment"));
    }

    #[test]
    fn test_file_error_display() {
        let error = FileErrorKind::InvalidFormat {
            format: "CSV".to_string(),
            reason: "invalid header".to_string(),
        };
        assert!(error.to_string().contains("CSV"));
        assert!(error.to_string().contains("invalid header"));

        let error = FileErrorKind::ParseError {
            line: 42,
            content: "bad data".to_string(),
            reason: "invalid number".to_string(),
        };
        assert!(error.to_string().contains("42"));
        assert!(error.to_string().contains("bad data"));
    }

    #[test]
    fn test_option_data_error_display() {
        let error = OptionDataErrorKind::InvalidDelta {
            delta: Some(1.5),
            reason: "delta cannot exceed 1".to_string(),
        };
        assert!(error.to_string().contains("1.5"));
        assert!(error.to_string().contains("delta cannot exceed 1"));
    }

    #[test]
    fn test_strategy_error_equality() {
        let error1 = StrategyErrorKind::InvalidLegs {
            expected: 4,
            found: 3,
            reason: "Iron Condor needs 4 legs".to_string(),
        };
        let error2 = StrategyErrorKind::InvalidLegs {
            expected: 4,
            found: 3,
            reason: "Iron Condor needs 4 legs".to_string(),
        };
        assert_eq!(error1, error2);
    }

    #[test]
    fn test_error_conversions() {
        // Test de String a ChainError
        let string_error: ChainError = "test error".to_string().into();
        assert!(matches!(
            string_error,
            ChainError::OptionDataError(OptionDataErrorKind::PriceCalculationError(_))
        ));

        // Test de io::Error a ChainError
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let chain_error = ChainError::from(io_error);
        assert!(matches!(
            chain_error,
            ChainError::FileError(FileErrorKind::IOError(_))
        ));

        let std_error: Box<dyn Error> = Box::new(std::io::Error::other("dynamic error"));
        let chain_error = ChainError::from(std_error);
        assert!(matches!(chain_error, ChainError::DynError { .. }));
    }

    #[test]
    fn test_helper_methods() {
        let error = ChainError::invalid_strike(-10.0, "Strike must be positive");
        assert!(matches!(
            error,
            ChainError::OptionDataError(OptionDataErrorKind::InvalidStrike { .. })
        ));

        let error = ChainError::invalid_volatility(None, "Volatility missing");
        assert!(matches!(
            error,
            ChainError::OptionDataError(OptionDataErrorKind::InvalidVolatility { .. })
        ));
    }

    #[test]
    fn test_chain_error_file_error() {
        let error = ChainError::FileError(FileErrorKind::IOError(io::Error::new(
            io::ErrorKind::NotFound,
            "File not found",
        )));
        assert_eq!(format!("{}", error), "File error: IO error: File not found");
    }

    #[test]
    fn test_chain_error_dyn_error() {
        let error = ChainError::DynError {
            message: "Dynamic error occurred".to_string(),
        };
        assert_eq!(format!("{}", error), "Error: Dynamic error occurred");
    }

    #[test]
    fn test_option_data_error_invalid_volatility() {
        let error = OptionDataErrorKind::InvalidVolatility {
            volatility: Some(0.25),
            reason: "Out of bounds".to_string(),
        };
        assert_eq!(
            format!("{}", error),
            "Invalid volatility 0.25: Out of bounds"
        );
    }

    #[test]
    fn test_option_data_error_invalid_prices() {
        let error = OptionDataErrorKind::InvalidPrices {
            bid: Some(1.0),
            ask: Some(2.0),
            reason: "Bid-ask spread too wide".to_string(),
        };
        assert_eq!(
            format!("{}", error),
            "Invalid prices (bid: Some(1.0), ask: Some(2.0)): Bid-ask spread too wide"
        );
    }

    #[test]
    fn test_option_data_error_price_calculation_error() {
        let error = OptionDataErrorKind::PriceCalculationError("Division by zero".to_string());
        assert_eq!(
            format!("{}", error),
            "Price calculation error: Division by zero"
        );
    }

    #[test]
    fn test_chain_build_error_strike_generation_error() {
        let error = ChainBuildErrorKind::StrikeGenerationError {
            reference_price: 100.0,
            interval: 5.0,
            reason: "Invalid strike intervals".to_string(),
        };
        assert_eq!(
            format!("{}", error),
            "Strike generation error (reference: 100, interval: 5): Invalid strike intervals"
        );
    }

    #[test]
    fn test_file_error_io_error() {
        let error = FileErrorKind::IOError(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Permission denied",
        ));
        assert_eq!(format!("{}", error), "IO error: Permission denied");
    }

    #[test]
    fn test_strategy_error_invalid_combination() {
        let error = StrategyErrorKind::InvalidCombination {
            strategy_type: "Straddle".to_string(),
            reason: "Conflicting legs".to_string(),
        };
        assert_eq!(
            format!("{}", error),
            "Invalid combination for strategy 'Straddle': Conflicting legs"
        );
    }

    #[test]
    fn test_chain_error_invalid_prices_constructor() {
        let error = ChainError::invalid_prices(Some(1.0), Some(2.0), "Spread too wide");
        assert_eq!(
            format!("{}", error),
            "Option data error: Invalid prices (bid: Some(1.0), ask: Some(2.0)): Spread too wide"
        );
    }
}
