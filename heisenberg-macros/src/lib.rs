use proc_macro::TokenStream;
use quote::quote;
use std::path::{Path, PathBuf};
use syn::Ident;

#[proc_macro]
pub fn embed_spa(input: TokenStream) -> TokenStream {
    let input_str = input.to_string().trim().to_string();

    // Two modes:
    // 1. "name" (no slash) - always looks up in heisenberg.toml
    // 2. "./path" (has slash) - always infers config from scanning the path
    let (identifier, output_dir): (String, String) = if input_str.is_empty() {
        // No argument - use default SPA from config
        ("__default".to_string(), find_output_dir_from_config(None))
    } else {
        let arg = input_str.trim_matches('"');
        if arg.contains('/') {
            // Path mode: infer everything from scanning the directory
            let identifier = derive_name_from_path(arg);
            let output_dir = infer_output_dir_from_path(arg);
            (identifier, output_dir)
        } else {
            // Name mode: look up in heisenberg.toml
            let identifier = sanitize_name(arg);
            let output_dir = find_output_dir_from_config(Some(arg));
            (identifier, output_dir)
        }
    };

    let name_ident = syn::parse_str::<Ident>(&identifier).unwrap();

    let expanded = quote! {
        {
            ::heisenberg::paste::paste! {
                #[derive(::heisenberg::rust_embed::RustEmbed)]
                #[folder = #output_dir]
                struct [<__HeisenbergEmbeddedAssets_ #name_ident>];

                #[::heisenberg::ctor::ctor]
                fn [<__register_heisenberg_assets_ #name_ident>]() {
                    use ::heisenberg::rust_embed::RustEmbed;
                    ::heisenberg::services::embed_registry::register_embedded_assets(
                        #output_dir,
                        |path: &str| [<__HeisenbergEmbeddedAssets_ #name_ident>]::get(path).map(|f| f.data.to_vec()),
                    );
                }
            }

            ::heisenberg::EmbeddedSpa::new(#output_dir, "")
        }
    };

    TokenStream::from(expanded)
}

/// Name mode: look up output_dir from heisenberg.toml
fn find_output_dir_from_config(spa_name: Option<&str>) -> String {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let config_path = PathBuf::from(&manifest_dir).join("heisenberg.toml");

    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(config) = toml::from_str::<toml::Value>(&content) {
                if let Some(output) = find_matching_spa(&config, spa_name) {
                    return output;
                }
            }
        }
    }

    // Fallback if config not found or SPA not in config
    panic!(
        "embed_spa!(\"{}\") requires a matching entry in heisenberg.toml",
        spa_name.unwrap_or("default")
    );
}

/// Path mode: infer output_dir by scanning the directory
fn infer_output_dir_from_path(path: &str) -> String {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let full_path = PathBuf::from(&manifest_dir).join(path);

    // If path points to a working directory (has package.json), find output subdir
    if full_path.join("package.json").exists() {
        let output_subdir = find_output_subdir(&full_path);
        return format!("{}/{}", path, output_subdir);
    }

    // Otherwise assume path is already the output directory
    path.to_string()
}

fn find_output_subdir(working_dir: &Path) -> String {
    // Check common output directories in order of preference
    for output_name in &["build", "dist", ".next", ".svelte-kit/output"] {
        if working_dir.join(output_name).exists() {
            return output_name.to_string();
        }
    }
    // Default to dist if nothing exists yet
    "dist".to_string()
}

fn find_matching_spa(config: &toml::Value, spa_name: Option<&str>) -> Option<String> {
    match spa_name {
        None => {
            // No name provided - look for single [spa]
            if let Some(spa) = config.get("spa").and_then(|v| v.as_table()) {
                return spa
                    .get("output_dir")
                    .and_then(|v| v.as_str())
                    .map(String::from);
            }
        }
        Some(name) => {
            // Name provided - look for matching [[spa]] with name field
            if let Some(spas) = config.get("spa").and_then(|v| v.as_array()) {
                for spa in spas {
                    if let Some(spa_table) = spa.as_table() {
                        if spa_table.get("name").and_then(|v| v.as_str()) == Some(name) {
                            return spa_table
                                .get("output_dir")
                                .and_then(|v| v.as_str())
                                .map(String::from);
                        }
                    }
                }
            }
        }
    }
    None
}

fn sanitize_name(name: &str) -> String {
    name.replace(['-', '.'], "_")
}

fn derive_name_from_path(path: &str) -> String {
    // Sanitize entire path to create a unique identifier
    // "./admin-webapp" -> "admin_webapp"
    // "./frontend/admin" -> "frontend_admin"
    path.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    mod sanitize_name_tests {
        use super::*;

        #[test]
        fn replaces_hyphens_with_underscores() {
            assert_eq!(sanitize_name("my-app"), "my_app");
            assert_eq!(sanitize_name("my-cool-app"), "my_cool_app");
        }

        #[test]
        fn replaces_dots_with_underscores() {
            assert_eq!(sanitize_name("my.app"), "my_app");
            assert_eq!(sanitize_name("my.cool.app"), "my_cool_app");
        }

        #[test]
        fn handles_mixed_special_chars() {
            assert_eq!(sanitize_name("my-app.v2"), "my_app_v2");
            assert_eq!(sanitize_name("app-name.prod"), "app_name_prod");
        }

        #[test]
        fn preserves_underscores() {
            assert_eq!(sanitize_name("my_app"), "my_app");
            assert_eq!(sanitize_name("my_cool_app"), "my_cool_app");
        }

        #[test]
        fn handles_alphanumeric_only() {
            assert_eq!(sanitize_name("myapp"), "myapp");
            assert_eq!(sanitize_name("myapp123"), "myapp123");
        }
    }

    mod derive_name_from_path_tests {
        use super::*;

        #[test]
        fn converts_simple_relative_path() {
            assert_eq!(derive_name_from_path("./admin-webapp"), "admin_webapp");
        }

        #[test]
        fn converts_nested_path() {
            assert_eq!(derive_name_from_path("./frontend/admin"), "frontend_admin");
        }

        #[test]
        fn converts_deeply_nested_path() {
            assert_eq!(
                derive_name_from_path("./apps/frontend/admin"),
                "apps_frontend_admin"
            );
        }

        #[test]
        fn handles_path_with_dots() {
            assert_eq!(derive_name_from_path("./my.app"), "my_app");
        }

        #[test]
        fn handles_path_with_mixed_separators() {
            assert_eq!(derive_name_from_path("./my-app/dist"), "my_app_dist");
        }

        #[test]
        fn trims_leading_and_trailing_underscores() {
            // Leading "./" becomes "__" which gets trimmed
            assert_eq!(derive_name_from_path("./app"), "app");
            // Trailing "/" becomes "_" which gets trimmed
            assert_eq!(derive_name_from_path("./app/"), "app");
        }
    }

    mod find_matching_spa_tests {
        use super::*;

        #[test]
        fn finds_single_spa_table_without_name() {
            let config: toml::Value = toml::from_str(
                r#"
                [spa]
                output_dir = "./dist"
                "#,
            )
            .unwrap();

            assert_eq!(find_matching_spa(&config, None), Some("./dist".to_string()));
        }

        #[test]
        fn returns_none_for_single_spa_when_name_provided() {
            let config: toml::Value = toml::from_str(
                r#"
                [spa]
                output_dir = "./dist"
                "#,
            )
            .unwrap();

            // Single [spa] doesn't support name lookup
            assert_eq!(find_matching_spa(&config, Some("frontend")), None);
        }

        #[test]
        fn finds_named_spa_in_array() {
            let config: toml::Value = toml::from_str(
                r#"
                [[spa]]
                name = "frontend"
                output_dir = "./frontend/dist"

                [[spa]]
                name = "admin"
                output_dir = "./admin/dist"
                "#,
            )
            .unwrap();

            assert_eq!(
                find_matching_spa(&config, Some("frontend")),
                Some("./frontend/dist".to_string())
            );
            assert_eq!(
                find_matching_spa(&config, Some("admin")),
                Some("./admin/dist".to_string())
            );
        }

        #[test]
        fn returns_none_for_nonexistent_spa_name() {
            let config: toml::Value = toml::from_str(
                r#"
                [[spa]]
                name = "frontend"
                output_dir = "./frontend/dist"
                "#,
            )
            .unwrap();

            assert_eq!(find_matching_spa(&config, Some("backend")), None);
        }

        #[test]
        fn returns_none_for_empty_config() {
            let config: toml::Value = toml::from_str("").unwrap();

            assert_eq!(find_matching_spa(&config, None), None);
            assert_eq!(find_matching_spa(&config, Some("app")), None);
        }

        #[test]
        fn returns_none_when_output_dir_missing() {
            let config: toml::Value = toml::from_str(
                r#"
                [spa]
                name = "app"
                "#,
            )
            .unwrap();

            assert_eq!(find_matching_spa(&config, None), None);
        }

        #[test]
        fn returns_none_for_array_spa_without_name_arg() {
            let config: toml::Value = toml::from_str(
                r#"
                [[spa]]
                name = "frontend"
                output_dir = "./frontend/dist"
                "#,
            )
            .unwrap();

            // No name provided but config uses array format
            assert_eq!(find_matching_spa(&config, None), None);
        }
    }

    mod find_output_subdir_tests {
        use super::*;
        use std::fs;
        use tempfile::TempDir;

        #[test]
        fn finds_build_directory() {
            let temp_dir = TempDir::new().unwrap();
            fs::create_dir(temp_dir.path().join("build")).unwrap();

            assert_eq!(find_output_subdir(temp_dir.path()), "build");
        }

        #[test]
        fn finds_dist_directory() {
            let temp_dir = TempDir::new().unwrap();
            fs::create_dir(temp_dir.path().join("dist")).unwrap();

            assert_eq!(find_output_subdir(temp_dir.path()), "dist");
        }

        #[test]
        fn finds_next_directory() {
            let temp_dir = TempDir::new().unwrap();
            fs::create_dir(temp_dir.path().join(".next")).unwrap();

            assert_eq!(find_output_subdir(temp_dir.path()), ".next");
        }

        #[test]
        fn finds_sveltekit_directory() {
            let temp_dir = TempDir::new().unwrap();
            fs::create_dir_all(temp_dir.path().join(".svelte-kit/output")).unwrap();

            assert_eq!(find_output_subdir(temp_dir.path()), ".svelte-kit/output");
        }

        #[test]
        fn prefers_build_over_dist() {
            let temp_dir = TempDir::new().unwrap();
            fs::create_dir(temp_dir.path().join("build")).unwrap();
            fs::create_dir(temp_dir.path().join("dist")).unwrap();

            assert_eq!(find_output_subdir(temp_dir.path()), "build");
        }

        #[test]
        fn prefers_dist_over_next() {
            let temp_dir = TempDir::new().unwrap();
            fs::create_dir(temp_dir.path().join("dist")).unwrap();
            fs::create_dir(temp_dir.path().join(".next")).unwrap();

            assert_eq!(find_output_subdir(temp_dir.path()), "dist");
        }

        #[test]
        fn defaults_to_dist_when_no_output_dir_exists() {
            let temp_dir = TempDir::new().unwrap();

            assert_eq!(find_output_subdir(temp_dir.path()), "dist");
        }
    }

    mod find_output_dir_from_config_tests {
        use super::*;
        use std::fs;
        use std::sync::Mutex;
        use tempfile::TempDir;

        // Mutex to ensure tests that modify CARGO_MANIFEST_DIR don't run concurrently
        static ENV_MUTEX: Mutex<()> = Mutex::new(());

        // Helper to acquire lock, recovering from poison if a previous test panicked
        fn acquire_lock() -> std::sync::MutexGuard<'static, ()> {
            ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner())
        }

        struct EnvGuard {
            key: &'static str,
            original: Option<String>,
        }

        impl EnvGuard {
            fn new(key: &'static str, value: &str) -> Self {
                let original = std::env::var(key).ok();
                std::env::set_var(key, value);
                Self { key, original }
            }
        }

        impl Drop for EnvGuard {
            fn drop(&mut self) {
                match &self.original {
                    Some(val) => std::env::set_var(self.key, val),
                    None => std::env::remove_var(self.key),
                }
            }
        }

        #[test]
        fn finds_output_dir_from_single_spa_config() {
            let _lock = acquire_lock();
            let temp_dir = TempDir::new().unwrap();

            let config_content = r#"
[spa]
output_dir = "./my-app/dist"
"#;
            fs::write(temp_dir.path().join("heisenberg.toml"), config_content).unwrap();

            let _guard = EnvGuard::new("CARGO_MANIFEST_DIR", temp_dir.path().to_str().unwrap());

            let result = find_output_dir_from_config(None);
            assert_eq!(result, "./my-app/dist");
        }

        #[test]
        fn finds_output_dir_from_named_spa_in_array() {
            let _lock = acquire_lock();
            let temp_dir = TempDir::new().unwrap();

            let config_content = r#"
[[spa]]
name = "frontend"
output_dir = "./frontend/dist"

[[spa]]
name = "admin"
output_dir = "./admin/build"
"#;
            fs::write(temp_dir.path().join("heisenberg.toml"), config_content).unwrap();

            let _guard = EnvGuard::new("CARGO_MANIFEST_DIR", temp_dir.path().to_str().unwrap());

            assert_eq!(
                find_output_dir_from_config(Some("frontend")),
                "./frontend/dist"
            );
            assert_eq!(find_output_dir_from_config(Some("admin")), "./admin/build");
        }

        #[test]
        #[should_panic(expected = "requires a matching entry in heisenberg.toml")]
        fn panics_when_config_file_missing() {
            let _lock = acquire_lock();
            let temp_dir = TempDir::new().unwrap();
            // No heisenberg.toml created

            let _guard = EnvGuard::new("CARGO_MANIFEST_DIR", temp_dir.path().to_str().unwrap());

            find_output_dir_from_config(None);
        }

        #[test]
        #[should_panic(expected = "requires a matching entry in heisenberg.toml")]
        fn panics_when_spa_name_not_found() {
            let _lock = acquire_lock();
            let temp_dir = TempDir::new().unwrap();

            let config_content = r#"
[[spa]]
name = "frontend"
output_dir = "./frontend/dist"
"#;
            fs::write(temp_dir.path().join("heisenberg.toml"), config_content).unwrap();

            let _guard = EnvGuard::new("CARGO_MANIFEST_DIR", temp_dir.path().to_str().unwrap());

            find_output_dir_from_config(Some("nonexistent"));
        }

        #[test]
        #[should_panic(expected = "requires a matching entry in heisenberg.toml")]
        fn panics_when_config_malformed() {
            let _lock = acquire_lock();
            let temp_dir = TempDir::new().unwrap();

            // Invalid TOML
            fs::write(
                temp_dir.path().join("heisenberg.toml"),
                "not valid toml [[[",
            )
            .unwrap();

            let _guard = EnvGuard::new("CARGO_MANIFEST_DIR", temp_dir.path().to_str().unwrap());

            find_output_dir_from_config(None);
        }
    }
}
