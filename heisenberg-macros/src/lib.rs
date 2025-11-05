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
