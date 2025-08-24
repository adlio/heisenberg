//! Path manipulation utilities

use std::path::Path;

/// Normalize a path for cross-platform compatibility
pub fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
