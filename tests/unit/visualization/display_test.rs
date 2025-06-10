#[cfg(test)]
mod display_tests {
    use optionstratlib::impl_json_debug_pretty;
    use optionstratlib::impl_json_display_pretty;
    use serde::{Deserialize, Serialize};

    // Create a test struct that will use our display macros
    #[derive(Serialize, Deserialize)]
    struct TestStruct {
        name: String,
        value: i32,
    }

    // Implement Display and Debug using our macros
    impl_json_display_pretty!(TestStruct);
    impl_json_debug_pretty!(TestStruct);

    #[test]
    fn test_json_display_pretty() {
        let test_struct = TestStruct {
            name: "test".to_string(),
            value: 42,
        };

        // Test Display implementation
        let display_output = format!("{}", test_struct);
        let expected = serde_json::to_string_pretty(&test_struct).unwrap();
        assert_eq!(display_output, expected);
    }

    #[test]
    fn test_json_debug_pretty() {
        let test_struct = TestStruct {
            name: "test".to_string(),
            value: 42,
        };

        // Test Debug implementation
        let debug_output = format!("{:?}", test_struct);
        let expected = serde_json::to_string_pretty(&test_struct).unwrap();
        assert_eq!(debug_output, expected);
    }

    #[test]
    fn test_json_display_error_handling() {
        // Create a struct that will fail to serialize
        #[derive(Serialize, Deserialize)]
        #[allow(dead_code)]
        struct BadStruct {
            #[serde(skip_serializing)]
            name: String,
            #[serde(serialize_with = "serialize_error")]
            value: i32,
        }

        // A serialization function that always fails
        fn serialize_error<S>(_: &i32, _: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Err(serde::ser::Error::custom("Test error"))
        }

        impl_json_display_pretty!(BadStruct);

        let bad_struct = BadStruct {
            name: "test".to_string(),
            value: 42,
        };

        // Test error handling in Display implementation
        let display_output = format!("{}", bad_struct);
        assert!(display_output.starts_with("Error serializing to JSON:"));
    }

    #[test]
    fn test_json_debug_error_handling() {
        // Create a struct that will fail to serialize
        #[derive(Serialize, Deserialize)]
        #[allow(dead_code)]
        struct BadDebugStruct {
            #[serde(skip_serializing)]
            name: String,
            #[serde(serialize_with = "serialize_debug_error")]
            value: i32,
        }

        // A serialization function that always fails
        fn serialize_debug_error<S>(_: &i32, _: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Err(serde::ser::Error::custom("Test debug error"))
        }

        impl_json_debug_pretty!(BadDebugStruct);

        let bad_struct = BadDebugStruct {
            name: "test".to_string(),
            value: 42,
        };

        // Test error handling in Debug implementation
        let debug_output = format!("{:?}", bad_struct);
        assert!(debug_output.starts_with("Error serializing to JSON:"));
    }
}
