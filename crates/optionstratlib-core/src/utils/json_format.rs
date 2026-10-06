//! JSON-backed `Debug` and `Display` implementations.
//!
//! Many result and strategy types format themselves as their own JSON:
//! `Display` writes the compact form (`{"a":1}`) and `Debug` the indented
//! one. The two writers below are the only implementation of that
//! convention, and [`impl_json_display!`](crate::impl_json_display),
//! [`impl_json_debug!`](crate::impl_json_debug) and
//! [`impl_json_debug_pretty!`](crate::impl_json_debug_pretty) wire a type to
//! them. They replace the `pretty-simple-display` derive macros, so no crate
//! of the workspace depends on a formatting crate to print its types
//! (M6-05, #546).
//!
//! A value that fails to serialize does not abort formatting: the writers
//! print `Error serializing to JSON: <reason>` in place of the JSON, so
//! `to_string()` and `{:?}` never panic on such a value.
//!
//! ```rust
//! use optionstratlib_core::{impl_json_debug_pretty, impl_json_display};
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct Quote {
//!     bid: u32,
//!     ask: u32,
//! }
//! impl_json_display!(Quote);
//! impl_json_debug_pretty!(Quote);
//!
//! let quote = Quote { bid: 1, ask: 2 };
//! assert_eq!(quote.to_string(), r#"{"bid":1,"ask":2}"#);
//! assert_eq!(format!("{quote:?}"), "{\n  \"bid\": 1,\n  \"ask\": 2\n}");
//! ```

use serde::Serialize;
use std::fmt;

/// Writes `value` to `f` as compact JSON.
///
/// When `value` cannot be serialized, writes
/// `Error serializing to JSON: <reason>` instead.
///
/// # Errors
///
/// Returns [`fmt::Error`] only when the formatter itself fails to write.
#[inline]
pub fn write_json<T: Serialize + ?Sized>(value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match serde_json::to_string(value) {
        Ok(json) => f.write_str(&json),
        Err(e) => write!(f, "Error serializing to JSON: {e}"),
    }
}

/// Writes `value` to `f` as indented JSON (two spaces per level).
///
/// When `value` cannot be serialized, writes
/// `Error serializing to JSON: <reason>` instead.
///
/// # Errors
///
/// Returns [`fmt::Error`] only when the formatter itself fails to write.
#[inline]
pub fn write_json_pretty<T: Serialize + ?Sized>(
    value: &T,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match serde_json::to_string_pretty(value) {
        Ok(json) => f.write_str(&json),
        Err(e) => write!(f, "Error serializing to JSON: {e}"),
    }
}

/// Implements `Display` as compact JSON for one or more `Serialize` types.
///
/// Expands to an `impl std::fmt::Display` per type that delegates to
/// [`write_json`](crate::utils::json_format::write_json).
///
/// ```rust
/// use optionstratlib_core::impl_json_display;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Fill {
///     qty: u32,
/// }
/// impl_json_display!(Fill);
///
/// assert_eq!(Fill { qty: 3 }.to_string(), r#"{"qty":3}"#);
/// ```
#[macro_export]
macro_rules! impl_json_display {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl ::std::fmt::Display for $ty {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    $crate::utils::json_format::write_json(self, f)
                }
            }
        )+
    };
}

/// Implements `Debug` as compact JSON for one or more `Serialize` types.
///
/// Expands to an `impl std::fmt::Debug` per type that delegates to
/// [`write_json`](crate::utils::json_format::write_json).
///
/// ```rust
/// use optionstratlib_core::impl_json_debug;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Fill {
///     qty: u32,
/// }
/// impl_json_debug!(Fill);
///
/// assert_eq!(format!("{:?}", Fill { qty: 3 }), r#"{"qty":3}"#);
/// ```
#[macro_export]
macro_rules! impl_json_debug {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl ::std::fmt::Debug for $ty {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    $crate::utils::json_format::write_json(self, f)
                }
            }
        )+
    };
}

/// Implements `Debug` as indented JSON for one or more `Serialize` types.
///
/// Expands to an `impl std::fmt::Debug` per type that delegates to
/// [`write_json_pretty`](crate::utils::json_format::write_json_pretty).
///
/// ```rust
/// use optionstratlib_core::impl_json_debug_pretty;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Fill {
///     qty: u32,
/// }
/// impl_json_debug_pretty!(Fill);
///
/// assert_eq!(format!("{:?}", Fill { qty: 3 }), "{\n  \"qty\": 3\n}");
/// ```
#[macro_export]
macro_rules! impl_json_debug_pretty {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl ::std::fmt::Debug for $ty {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    $crate::utils::json_format::write_json_pretty(self, f)
                }
            }
        )+
    };
}

#[cfg(test)]
mod tests {
    use serde::ser::Error as _;
    use serde::{Serialize, Serializer};

    #[derive(Serialize)]
    struct Sample {
        name: &'static str,
        value: u32,
    }
    crate::impl_json_display!(Sample);
    crate::impl_json_debug_pretty!(Sample);

    #[derive(Serialize)]
    struct Compact {
        flag: bool,
    }
    crate::impl_json_debug!(Compact);

    /// A value whose serialization always fails.
    struct Unserializable;

    impl Serialize for Unserializable {
        fn serialize<S: Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
            Err(S::Error::custom("refused"))
        }
    }
    crate::impl_json_display!(Unserializable);
    crate::impl_json_debug!(Unserializable);

    #[test]
    fn test_json_display_sample_writes_compact_json() {
        let sample = Sample {
            name: "a",
            value: 1,
        };
        assert_eq!(sample.to_string(), r#"{"name":"a","value":1}"#);
    }

    #[test]
    fn test_json_debug_pretty_sample_writes_indented_json() {
        let sample = Sample {
            name: "a",
            value: 1,
        };
        assert_eq!(
            format!("{sample:?}"),
            "{\n  \"name\": \"a\",\n  \"value\": 1\n}"
        );
    }

    #[test]
    fn test_json_debug_compact_writes_compact_json() {
        assert_eq!(format!("{:?}", Compact { flag: true }), r#"{"flag":true}"#);
    }

    #[test]
    fn test_json_display_serialization_failure_writes_the_reason() {
        assert_eq!(
            Unserializable.to_string(),
            "Error serializing to JSON: refused"
        );
        assert_eq!(
            format!("{Unserializable:?}"),
            "Error serializing to JSON: refused"
        );
    }
}
