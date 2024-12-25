/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 25/12/24
 ******************************************************************************/
use num_traits::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use crate::error::decimal::DecimalError;
use crate::model::types::PositiveF64;

pub(crate) fn f64_to_decimal(value: f64) -> Result<Decimal, DecimalError> {
    let result = Decimal::from_f64(value);
    match result {
        Some(decimal) => Ok(decimal),
        None => Err(DecimalError::ConversionError {
            from_type: "f64".to_string(),
            to_type: "Decimal".to_string(),
            reason: "Failed to convert f64 to Decimal".to_string(),
        }),
    }
}

pub(crate) fn decimal_to_f64(value: Decimal) -> Result<f64, DecimalError> {
    value.to_f64().ok_or(DecimalError::ConversionError {
        from_type: "Decimal".to_string(),
        to_type: "f64".to_string(),
        reason: "Failed to convert Decimal to f64".to_string(),
    })
}


pub(crate) fn positive_f64_to_decimal(value: PositiveF64) -> Result<Decimal, DecimalError> {
    let result = Decimal::from_f64(value.value());
    match result {
        Some(decimal) => Ok(decimal),
        None => Err(DecimalError::ConversionError {
            from_type: "PositiveF64".to_string(),
            to_type: "Decimal".to_string(),
            reason: "Failed to convert f64 to Decimal".to_string(),
        }),
    }
}

pub(crate) fn decimal_exp(value: Decimal) -> Result<Decimal, DecimalError> {
    let value_f64 = value.to_f64().ok_or(DecimalError::ConversionError {
        from_type: "Decimal".to_string(),
        to_type: "f64".to_string(),
        reason: "Failed to convert Decimal to f64".to_string(),
    })?;
    let exp_f64 = value_f64.exp(); // Calcula la exponencial usando f64
    Decimal::from_f64(exp_f64).ok_or(DecimalError::ConversionError {
        from_type: "f64".to_string(),
        to_type: "Decimal".to_string(),
        reason: "Failed to convert f64 to Decimal".to_string(),
    })
}

pub(crate) fn decimal_sqrt(value: Decimal) -> Result<Decimal, DecimalError> {
    let value_f64 = value.to_f64().ok_or(DecimalError::ConversionError {
        from_type: "Decimal".to_string(),
        to_type: "f64".to_string(),
        reason: "Failed to convert Decimal to f64".to_string(),
    })?;
    let sqrt_f64 = value_f64.sqrt(); // Calcula la raíz cuadrada usando f64
    Decimal::from_f64(sqrt_f64).ok_or(DecimalError::ConversionError {
        from_type: "f64".to_string(),
        to_type: "Decimal".to_string(),
        reason: "Failed to convert f64 to Decimal".to_string(),
    })
}

pub(crate) fn decimal_pow_two(value: Decimal) -> Result<Decimal, DecimalError> {
    let value_f64 = value.to_f64().ok_or(DecimalError::ConversionError {
        from_type: "Decimal".to_string(),
        to_type: "f64".to_string(),
        reason: "Failed to convert Decimal to f64".to_string(),
    })?;
    let pow_f64 = value_f64.powi(2); // Calcula el cuadrado usando f64
    Decimal::from_f64(pow_f64).ok_or(DecimalError::ConversionError {
        from_type: "f64".to_string(),
        to_type: "Decimal".to_string(),
        reason: "Failed to convert f64 to Decimal".to_string(),
    })
}

pub(crate) fn decimal_ln(value: Decimal) -> Result<Decimal, DecimalError> {
    let value_f64 = value.to_f64().ok_or(DecimalError::ConversionError {
        from_type: "Decimal".to_string(),
        to_type: "f64".to_string(),
        reason: "Failed to convert Decimal to f64".to_string(),
    })?;
    let ln_f64 = value_f64.ln(); // Calcula el logaritmo natural usando f64
    Decimal::from_f64(ln_f64).ok_or(DecimalError::ConversionError {
        from_type: "f64".to_string(),
        to_type: "Decimal".to_string(),
        reason: "Failed to convert f64 to Decimal".to_string(),
    })
}

