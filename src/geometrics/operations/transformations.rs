use crate::curves::Point2D;
use rust_decimal::Decimal;

#[allow(dead_code)]
pub enum CurveTransformation {
    /// Translates the curve horizontally and vertically
    Translate { dx: Decimal, dy: Decimal },

    /// Scales the curve by factors in x and y directions
    Scale { sx: Decimal, sy: Decimal },

    /// Rotates the curve around a given point
    Rotate {
        angle: Decimal,
        pivot: Option<Point2D>,
    },

    /// Reflects the curve across an axis
    Reflect {
        /// Horizontal (x-axis) or Vertical (y-axis) reflection
        axis: ReflectionAxis,
    },

    /// Warps the curve using a non-linear transformation
    Warp {
        /// A function that defines how to transform each point
        transformation_fn: Box<dyn Fn(Point2D) -> Point2D>,
    },
}

/// Defines the axis of reflection
#[allow(dead_code)]
pub enum ReflectionAxis {
    XAxis,
    YAxis,
    Line(Point2D, Point2D), // Reflect across an arbitrary line
}

#[allow(dead_code)]
pub enum TopologicalTransformation {
    /// Extracts a subset of the curve between two x-coordinates
    Slice { start: Decimal, end: Decimal },

    /// Applies a moving window function to smooth the curve
    Smooth { window_size: usize },

    /// Applies a derivative transformation
    Differentiate,

    /// Applies an integral transformation
    Integrate,

    /// Normalizes the curve (e.g., min-max scaling)
    Normalize,

    /// Removes noise or outliers from the curve
    Denoise { method: DenoiseMethod },
}

#[allow(dead_code)]
pub enum DenoiseMethod {
    MovingAverage,
    LowPassFilter,
    MedianFilter,
}

#[allow(dead_code)]
pub enum SpectralTransformation {
    /// Performs Fourier Transform
    FourierTransform,

    /// Performs Wavelet Transform
    WaveletTransform,

    /// Applies frequency filtering
    FrequencyFilter {
        low_cutoff: Option<Decimal>,
        high_cutoff: Option<Decimal>,
    },
}

#[allow(dead_code)]
pub enum StatisticalTransformation {
    /// Calculates moving statistics
    MovingStatistics {
        window_size: usize,
        statistic: MovingStatisticType,
    },

    /// Applies a statistical transformation
    Transform { transformation: StatTransformType },
}

#[allow(dead_code)]
pub enum MovingStatisticType {
    Mean,
    Median,
    StandardDeviation,
    Variance,
}

#[allow(dead_code)]
pub enum StatTransformType {
    Log,
    Exponential,
    Power { exponent: Decimal },
    ZScore,
}

#[allow(dead_code)]
pub enum DomainSpecificTransformation {
    /// Financial curve transformations
    Financial { method: FinancialTransformMethod },

    /// Signal processing transformations
    SignalProcessing { method: SignalProcessingMethod },
}

#[allow(dead_code)]
pub enum FinancialTransformMethod {
    Returns,
    LogReturns,
    CumulativeReturns,
}

#[allow(dead_code)]
pub enum SignalProcessingMethod {
    Envelope,
    Rectification,
    Windowing { window_type: WindowType },
}

#[allow(dead_code)]
pub enum WindowType {
    Hamming,
    Hanning,
    Blackman,
}
