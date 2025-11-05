use tokio::signal;

/// Wait for Ctrl+C signal
///
/// Use with axum::serve for graceful shutdown that cleans up dev servers
///
/// # Examples
///
/// ```ignore
/// axum::serve(listener, app)
///     .with_graceful_shutdown(heisenberg::shutdown_signal())
///     .await?;
/// ```
pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
