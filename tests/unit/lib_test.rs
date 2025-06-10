use optionstratlib::version;

#[test]
fn test_version() {
    // Test that the version function returns a non-empty string
    let version_str = version();
    assert!(!version_str.is_empty());

    // Test that the version matches the expected format (e.g., "0.5.1")
    assert!(version_str.chars().any(|c| c.is_ascii_digit()));
    assert!(version_str.contains('.'));

    // Verify it matches the cargo package version
    assert_eq!(version_str, env!("CARGO_PKG_VERSION"));
}
