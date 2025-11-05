//! Macros for embedding assets

/// Embed SPA assets and return a handle
///
/// This macro embeds assets from a SPA directory and returns an `EmbeddedSpa` handle
/// that you pass to `Heisenberg::spa()` for configuration.
///
/// # Examples
///
/// ```ignore
/// // Single SPA
/// let app = heisenberg::embed_spa!("./web");
/// let config = Heisenberg::new()
///     .spa("/*", app)
///     .build();
///
/// // Multiple SPAs with unique identifiers
/// let admin = heisenberg::embed_spa!("./web/admin", admin);
/// let user = heisenberg::embed_spa!("./web/user", user);
/// let config = Heisenberg::new()
///     .spa("/admin/*", admin)
///     .spa("/*", user)
///     .build();
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
