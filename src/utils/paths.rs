//! Path manipulation utilities

use std::path::Path;

/// Normalize a path for cross-platform compatibility
pub fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_normalize_path_forward_slashes_unchanged() {
        let path = PathBuf::from("foo/bar/baz");
        assert_eq!(normalize_path(&path), "foo/bar/baz");
    }

    #[test]
    fn test_normalize_path_backslashes_converted() {
        // Simulate Windows-style path
        let path = PathBuf::from("foo\\bar\\baz");
        assert_eq!(normalize_path(&path), "foo/bar/baz");
    }

    #[test]
    fn test_normalize_path_mixed_slashes() {
        let path = PathBuf::from("foo/bar\\baz");
        assert_eq!(normalize_path(&path), "foo/bar/baz");
    }

    #[test]
    fn test_normalize_path_absolute() {
        let path = PathBuf::from("/usr/local/bin");
        assert_eq!(normalize_path(&path), "/usr/local/bin");
    }

    #[test]
    fn test_normalize_path_empty() {
        let path = PathBuf::from("");
        assert_eq!(normalize_path(&path), "");
    }

    #[test]
    fn test_normalize_path_single_component() {
        let path = PathBuf::from("file.txt");
        assert_eq!(normalize_path(&path), "file.txt");
    }

    #[test]
    fn test_normalize_path_with_dots() {
        let path = PathBuf::from("./foo/../bar");
        assert_eq!(normalize_path(&path), "./foo/../bar");
    }
}
