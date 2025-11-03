//! Macros for embedding assets

/// Embed assets and create Heisenberg config in one step
///
/// This macro generates a RustEmbed struct, registers it, and returns a configured
/// Heisenberg instance ready to use.
///
/// # Example
///
/// ```ignore
/// use heisenberg::embed_spa_assets;
///
/// #[tokio::main]
/// async fn main() {
///     let config = embed_spa_assets!("./web/build");
///     
///     let app = Router::new()
///         .route("/api/hello", get(handler))
///         .layer(HeisenbergLayer::new(config));
///     // ...
/// }
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

        $crate::Heisenberg::new().spa($folder).build()
    }};
}
