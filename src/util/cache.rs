use std::path::PathBuf;

/// Returns the path to the HTTP cache directory.
///
/// # Panics
/// - If the cache directory cannot be found.
pub(crate) fn http_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .expect("Cache directory not found.")
        .join(env!("CARGO_PKG_NAME"))
        .join("http-cache")
}
