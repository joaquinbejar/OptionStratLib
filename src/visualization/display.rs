#[macro_export]
/// Implements the `Display` trait for the specified types, formatting the output as pretty-printed JSON.
///
/// This macro generates an implementation of `std::fmt::Display` that serializes the object to a
/// pretty-printed JSON string. If serialization fails, it displays an error message.
///
macro_rules! impl_json_display_pretty {
    ($($t:ty),+) => {
        $(
            impl std::fmt::Display for $t {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match serde_json::to_string_pretty(self) {
                        Ok(pretty_json) => write!(f, "{}", pretty_json),
                        Err(e) => write!(f, "Error serializing to JSON: {}", e),
                    }
                }
            }
        )+
    }
}

#[macro_export]
/// Implements the `Debug` trait for the specified types, formatting the output as pretty-printed JSON.
///
/// This macro generates an implementation of `std::fmt::Debug` that serializes the object to a
/// pretty-printed JSON string. If serialization fails, it displays an error message.
///
macro_rules! impl_json_debug_pretty {
    ($($t:ty),+) => {
        $(
            impl std::fmt::Debug for $t {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match serde_json::to_string_pretty(self) {
                        Ok(pretty_json) => write!(f, "{}", pretty_json),
                        Err(e) => write!(f, "Error serializing to JSON: {}", e),
                    }
                }
            }
        )+
    }
}

#[macro_export]
/// Implements the `Debug` trait for the specified types, formatting the output as compact JSON.
///
/// This macro generates an implementation of `std::fmt::Debug` that serializes the object to a
/// compact JSON string. If serialization fails, it displays an error message.
///
macro_rules! impl_json_debug {
    ($($t:ty),+) => {
        $(
            impl std::fmt::Debug for $t {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match serde_json::to_string(self) {
                        Ok(pretty_json) => write!(f, "{}", pretty_json),
                        Err(e) => write!(f, "Error serializing to JSON: {}", e),
                    }
                }
            }
        )+
    }
}

#[macro_export]
/// Implements the `Display` trait for the specified types, formatting the output as compact JSON.
///
/// This macro generates an implementation of `std::fmt::Display` that serializes the object to a
/// compact JSON string. If serialization fails, it displays an error message.
///
macro_rules! impl_json_display {
    ($($t:ty),+) => {
        $(
            impl std::fmt::Display for $t {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match serde_json::to_string(self) {
                        Ok(pretty_json) => write!(f, "{}", pretty_json),
                        Err(e) => write!(f, "Error serializing to JSON: {}", e),
                    }
                }
            }
        )+
    }
}
