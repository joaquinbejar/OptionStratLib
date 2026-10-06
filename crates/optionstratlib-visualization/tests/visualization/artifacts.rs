//! Assertions on the image files the static export writes, shared by the
//! tests that need a WebDriver (`plotly_render_test`, `plotly_test`).

use std::path::Path;

/// The eight-byte signature every PNG file starts with.
const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

/// Asserts that `path` holds a real PNG: it exists, is not empty and starts
/// with the PNG signature.
pub fn assert_png_artifact(path: &Path) {
    let bytes = std::fs::read(path).expect("the PNG file should exist");
    assert!(!bytes.is_empty(), "the PNG file should not be empty");
    assert!(
        bytes.starts_with(PNG_SIGNATURE),
        "the file should start with the PNG signature"
    );
}

/// Asserts that `path` holds a real SVG: it exists, is not empty and contains
/// an `<svg` element.
pub fn assert_svg_artifact(path: &Path) {
    let bytes = std::fs::read(path).expect("the SVG file should exist");
    assert!(!bytes.is_empty(), "the SVG file should not be empty");
    assert!(
        String::from_utf8_lossy(&bytes).contains("<svg"),
        "the file should contain an <svg element"
    );
}
