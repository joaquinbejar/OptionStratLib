use utoipa::ToSchema;

/// Represents mathematical or aggregation operations that can be applied
/// during a merge or combination process of geometric objects or curves.
///
/// This enum defines the possible operations when merging or combining multiple
/// data sets, such as curves or points. It is commonly used with the `Arithmetic` trait
/// to specify how values should be combined during geometric operations.
///
/// # Operations
///
/// * `Add` - Performs addition of values (a + b)
/// * `Subtract` - Performs subtraction of values (a - b)
/// * `Multiply` - Performs multiplication of values (a * b)
/// * `Divide` - Performs division of values (a / b), with appropriate error handling for division by zero
/// * `Max` - Selects the maximum value between operands (max(a, b))
/// * `Min` - Selects the minimum value between operands (min(a, b))
///
/// The operation specified will determine how values are combined when merging
/// geometric objects that implement the `Arithmetic` trait.
#[derive(Clone, Copy, Debug, PartialEq, ToSchema)]
pub enum MergeOperation {
    /// Adds values together (a + b)
    Add,
    /// Subtracts the second value from the first (a - b)
    Subtract,
    /// Multiplies values together (a * b)
    Multiply,
    /// Divides the first value by the second (a / b)
    Divide,
    /// Selects the maximum between values (max(a, b))
    Max,
    /// Selects the minimum between values (min(a, b))
    Min,
}
