//! Macros for embedding assets

/// Embed SPA assets and return a handle
///
/// # Examples
///
/// ```ignore
/// // Direct path
/// let app = heisenberg::embed_spa!("./dist");
///
/// // Multiple SPAs
/// let admin = heisenberg::embed_spa!("./admin/dist", admin);
/// let app = heisenberg::embed_spa!("./app/dist", app);
/// ```
#[macro_export]
macro_rules! embed_spa {
    ($spa_dir:expr) => {
        $crate::embed_spa!($spa_dir, __default)
    };
    ($spa_dir:expr, $id:ident) => {{
        $crate::paste::paste! {
            #[derive($crate::rust_embed::RustEmbed)]
            #[folder = $spa_dir]
            struct [<__HeisenbergEmbeddedAssets_ $id>];

            #[$crate::ctor::ctor]
            fn [<__register_heisenberg_assets_ $id>]() {
                $crate::services::embed_registry::register_embedded_assets(
                    $spa_dir,
                    |path: &str| [<__HeisenbergEmbeddedAssets_ $id>]::get(path).map(|f| f.data.to_vec()),
                );
            }
        }

        $crate::EmbeddedSpa::new($spa_dir, "")
    }};
}
