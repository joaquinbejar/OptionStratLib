/// Represents mathematical or aggregation operations that can be applied
/// during a merge or combination process.
///
/// Variants:
///
/// * `Add` - Performs addition of the values.
/// * `Subtract` - Performs subtraction of the values.
/// * `Multiply` - Performs multiplication of the values.
/// * `Divide` - Performs division of the values.
/// * `Max` - Selects the maximum of the values.
/// * `Min` - Selects the minimum of the values.
pub enum MergeOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Max,
    Min,
}
