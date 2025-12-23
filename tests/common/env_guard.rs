//! RAII guard for environment variables in tests.

#![allow(dead_code)]

/// RAII guard for environment variables.
///
/// Sets an environment variable on creation and restores the previous value
/// (or removes it) when dropped. This is panic-safe - cleanup happens even
/// if the test panics.
///
/// # Example
/// ```ignore
/// let _guard = EnvGuard::set("HEISENBERG_MODE", "proxy");
/// // test code...
/// // guard drops automatically, restoring previous value
/// ```
pub struct EnvGuard {
    key: &'static str,
    previous_value: Option<String>,
}

impl EnvGuard {
    /// Set an environment variable, returning a guard that restores the previous value on drop.
    pub fn set(key: &'static str, value: &str) -> Self {
        let previous_value = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self {
            key,
            previous_value,
        }
    }

    /// Remove an environment variable, returning a guard that restores the previous value on drop.
    pub fn remove(key: &'static str) -> Self {
        let previous_value = std::env::var(key).ok();
        std::env::remove_var(key);
        Self {
            key,
            previous_value,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.previous_value {
            Some(val) => std::env::set_var(self.key, val),
            None => std::env::remove_var(self.key),
        }
    }
}
