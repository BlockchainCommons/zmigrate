use std::path::PathBuf;

/// Returns the path to the test fixtures directory.
/// This is a common utility used across all tests.
pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Helper to load a file path from components relative to the fixtures
/// directory.
///
/// # Arguments
/// * `path_elements` - Path components relative to the fixtures directory,
///   e.g., `["zcashd", "golden-v5.6.0", "node0_wallet.dat"]`
pub fn fixtures_path(path_elements: &[&str]) -> PathBuf {
    fixtures_dir().join(path_elements.iter().collect::<PathBuf>())
}
