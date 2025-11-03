//! Macros for embedding assets

/// Embed assets and start building Heisenberg config
///
/// This macro generates a RustEmbed struct, registers it, and returns a builder
/// that you can configure further before calling `.build()`.
///
/// # Simple Example (single SPA)
///
/// ```ignore
/// let config = heisenberg::embed_spa_assets!("./dist").build();
/// ```
///
/// # Multiple SPAs
///
/// ```ignore
/// heisenberg::embed_spa_assets!("./admin/dist");
/// heisenberg::embed_spa_assets!("./app/dist");
///
/// let config = Heisenberg::new()
///     .spa("./admin/dist").pattern("/admin/*")
///     .spa("./app/dist").pattern("/*")
///     .build();
/// ```
///
/// # Custom Route Pattern
///
/// ```ignore
/// let config = heisenberg::embed_spa_assets!("./dist")
///     .pattern("/app/*")
///     .build();
/// ```
#[macro_export]
macro_rules! embed_spa_assets {
    ($folder:expr) => {{
        const _: () = {
            #[derive($crate::rust_embed::RustEmbed)]
            #[folder = $folder]
            struct __HeisenbergEmbeddedAssets;

            #[$crate::ctor::ctor]
            fn __register_heisenberg_assets() {
                $crate::services::embed_registry::register_embedded_assets(
                    $folder,
                    |path: &str| __HeisenbergEmbeddedAssets::get(path).map(|f| f.data.to_vec()),
                );
            }
        };

        $crate::Heisenberg::new().spa($folder)
    }};
}
