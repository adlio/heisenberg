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
    use std::fs;
    use std::sync::Mutex;
    use tempfile::TempDir;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("my-app.v2"), "my_app_v2");
    }

    #[test]
    fn test_derive_name_from_path() {
        assert_eq!(derive_name_from_path("./my-app/dist"), "my_app_dist");
    }

    mod find_matching_spa_tests {
        use super::*;

        #[test]
        fn single_spa_table() {
            let config: toml::Value = toml::from_str("[spa]\noutput_dir = \"./dist\"").unwrap();
            assert_eq!(find_matching_spa(&config, None), Some("./dist".to_string()));
            // Single [spa] doesn't support name lookup
            assert_eq!(find_matching_spa(&config, Some("foo")), None);
        }

        #[test]
        fn spa_array_with_names() {
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
            // Nonexistent name
            assert_eq!(find_matching_spa(&config, Some("backend")), None);
            // No name arg with array format
            assert_eq!(find_matching_spa(&config, None), None);
        }

        #[test]
        fn returns_none_for_missing_or_incomplete_config() {
            let empty: toml::Value = toml::from_str("").unwrap();
            assert_eq!(find_matching_spa(&empty, None), None);

            let no_output: toml::Value = toml::from_str("[spa]\nname = \"app\"").unwrap();
            assert_eq!(find_matching_spa(&no_output, None), None);
        }
    }

    mod find_output_subdir_tests {
        use super::*;

        #[test]
        fn detects_framework_output_dirs() {
            let temp_dir = TempDir::new().unwrap();

            // Default when nothing exists
            assert_eq!(find_output_subdir(temp_dir.path()), "dist");

            // Create dirs and verify priority order
            fs::create_dir(temp_dir.path().join(".next")).unwrap();
            assert_eq!(find_output_subdir(temp_dir.path()), ".next");

            fs::create_dir(temp_dir.path().join("dist")).unwrap();
            assert_eq!(find_output_subdir(temp_dir.path()), "dist"); // dist > .next

            fs::create_dir(temp_dir.path().join("build")).unwrap();
            assert_eq!(find_output_subdir(temp_dir.path()), "build"); // build > dist
        }

        #[test]
        fn finds_sveltekit_directory() {
            let temp_dir = TempDir::new().unwrap();
            fs::create_dir_all(temp_dir.path().join(".svelte-kit/output")).unwrap();
            assert_eq!(find_output_subdir(temp_dir.path()), ".svelte-kit/output");
        }
    }

    mod find_output_dir_from_config_tests {
        use super::*;

        static ENV_MUTEX: Mutex<()> = Mutex::new(());

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
        fn reads_from_heisenberg_toml() {
            let _lock = acquire_lock();
            let temp_dir = TempDir::new().unwrap();
            fs::write(
                temp_dir.path().join("heisenberg.toml"),
                "[spa]\noutput_dir = \"./my-app/dist\"",
            )
            .unwrap();

            let _guard = EnvGuard::new("CARGO_MANIFEST_DIR", temp_dir.path().to_str().unwrap());
            assert_eq!(find_output_dir_from_config(None), "./my-app/dist");
        }

        #[test]
        #[should_panic(expected = "requires a matching entry in heisenberg.toml")]
        fn panics_when_config_missing() {
            let _lock = acquire_lock();
            let temp_dir = TempDir::new().unwrap();
            let _guard = EnvGuard::new("CARGO_MANIFEST_DIR", temp_dir.path().to_str().unwrap());
            find_output_dir_from_config(None);
        }

        #[test]
        #[should_panic(expected = "requires a matching entry in heisenberg.toml")]
        fn panics_when_spa_not_found() {
            let _lock = acquire_lock();
            let temp_dir = TempDir::new().unwrap();
            fs::write(
                temp_dir.path().join("heisenberg.toml"),
                "[[spa]]\nname = \"other\"\noutput_dir = \"./x\"",
            )
            .unwrap();

            let _guard = EnvGuard::new("CARGO_MANIFEST_DIR", temp_dir.path().to_str().unwrap());
            find_output_dir_from_config(Some("nonexistent"));
        }
    }
}
