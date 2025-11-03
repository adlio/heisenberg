//! Macros for embedding assets

/// Embed assets for a SPA route
///
/// This macro generates a RustEmbed struct and registers it with Heisenberg.
/// Use this in your main.rs before creating the Heisenberg config.
///
/// # Example
///
/// ```ignore
/// use heisenberg::embed_spa_assets;
///
/// embed_spa_assets!("./web/build");
///
/// #[tokio::main]
/// async fn main() {
///     let config = heisenberg::Heisenberg::new()
///         .spa("./web/build")
///         .build();
///     // ...
/// }
/// ```
#[macro_export]
macro_rules! embed_spa_assets {
    ($folder:expr) => {
        #[derive(::rust_embed::RustEmbed)]
        #[folder = $folder]
        struct __HeisenbergEmbeddedAssets;

        const _: () = {
            #[::ctor::ctor]
            fn __register_heisenberg_assets() {
                $crate::services::embed_registry::register_embedded_assets(
                    $folder,
                    |path: &str| __HeisenbergEmbeddedAssets::get(path).map(|f| f.data.to_vec()),
                );
            }
        };
    };
}
