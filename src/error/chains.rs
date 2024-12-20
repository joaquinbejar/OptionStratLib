/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/12/24
******************************************************************************/

use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ChainError {
    /// Errores relacionados con la validación de datos de opciones
    OptionDataError(OptionDataErrorKind),
    /// Errores relacionados con la construcción de la cadena
    ChainBuildError(ChainBuildErrorKind),
    /// Errores relacionados con operaciones de archivos
    FileError(FileErrorKind),
    /// Errores relacionados con las estrategias
    StrategyError(StrategyErrorKind),
}

/// Errores específicos para los datos de opciones
#[derive(Debug)]
pub enum OptionDataErrorKind {
    /// El strike price es inválido
    InvalidStrike { strike: f64, reason: String },
    /// La volatilidad implícita es inválida
    InvalidVolatility {
        volatility: Option<f64>,
        reason: String,
    },
    /// Los precios bid/ask son inválidos
    InvalidPrices {
        bid: Option<f64>,
        ask: Option<f64>,
        reason: String,
    },
    /// El delta es inválido
    InvalidDelta { delta: Option<f64>, reason: String },
    /// Error en el cálculo de precios
    PriceCalculationError(String),
}

/// Errores específicos para la construcción de cadenas
#[derive(Debug)]
pub enum ChainBuildErrorKind {
    /// Parámetros inválidos para la construcción
    InvalidParameters { parameter: String, reason: String },
    /// Error en el ajuste de volatilidad
    VolatilityAdjustmentError { skew_factor: f64, reason: String },
    /// Error en la generación de strikes
    StrikeGenerationError {
        reference_price: f64,
        interval: f64,
        reason: String,
    },
}

/// Errores relacionados con operaciones de archivos
#[derive(Debug)]
pub enum FileErrorKind {
    /// Error al leer/escribir archivo
    IOError(io::Error),
    /// Error en el formato del archivo
    InvalidFormat { format: String, reason: String },
    /// Error en el parsing de datos
    ParseError {
        line: usize,
        content: String,
        reason: String,
    },
}

/// Errores específicos para estrategias
#[derive(Debug, PartialEq)]
pub enum StrategyErrorKind {
    /// Error en la validación de legs
    InvalidLegs {
        expected: usize,
        found: usize,
        reason: String,
    },
    /// Error en la combinación de opciones
    InvalidCombination {
        strategy_type: String,
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
                skew_factor,
                reason,
            } => {
                write!(
                    f,
                    "Volatility adjustment error (skew factor: {}): {}",
                    skew_factor, reason
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

impl ChainError {
    pub fn invalid_strike(strike: f64, reason: &str) -> Self {
        ChainError::OptionDataError(OptionDataErrorKind::InvalidStrike {
            strike,
            reason: reason.to_string(),
        })
    }

    pub fn invalid_volatility(volatility: Option<f64>, reason: &str) -> Self {
        ChainError::OptionDataError(OptionDataErrorKind::InvalidVolatility {
            volatility,
            reason: reason.to_string(),
        })
    }

    pub fn invalid_prices(bid: Option<f64>, ask: Option<f64>, reason: &str) -> Self {
        ChainError::OptionDataError(OptionDataErrorKind::InvalidPrices {
            bid,
            ask,
            reason: reason.to_string(),
        })
    }

    pub fn invalid_legs(expected: usize, found: usize, reason: &str) -> Self {
        ChainError::StrategyError(StrategyErrorKind::InvalidLegs {
            expected,
            found,
            reason: reason.to_string(),
        })
    }

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
