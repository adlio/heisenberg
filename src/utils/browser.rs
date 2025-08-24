//! Cross-platform browser opening functionality

use crate::error::HeisenbergError;
use std::process::Command;

/// Open a URL in the default browser
pub async fn open_browser(url: &str) -> Result<(), HeisenbergError> {
    let result = tokio::task::spawn_blocking({
        let url = url.to_string();
        move || open_browser_sync(&url)
    })
    .await
    .map_err(|e| HeisenbergError::config(
        format!("Failed to spawn browser task: {}", e),
        "• This is an internal error with async task spawning\n• Try disabling browser opening with .open_browser(false)\n• Report this issue if it persists"
    ))?;

    result
}

/// Synchronous browser opening implementation
fn open_browser_sync(url: &str) -> Result<(), HeisenbergError> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn().map_err(|e| {
            HeisenbergError::config(
                format!("Failed to open browser on macOS: {}", e),
                "• Ensure the 'open' command is available\n• Check if a default browser is set\n• Try opening the URL manually to test\n• Disable browser opening with .open_browser(false)"
            )
        })?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/c", "start", "", url])
            .spawn()
            .map_err(|e| {
                HeisenbergError::config(
                    format!("Failed to open browser on Windows: {}", e),
                    "• Ensure the 'start' command is available\n• Check if a default browser is set\n• Try opening the URL manually to test\n• Disable browser opening with .open_browser(false)"
                )
            })?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(url).spawn().map_err(|e| {
            HeisenbergError::config(
                format!("Failed to open browser on Linux: {}", e),
                "• Ensure 'xdg-open' is installed (usually part of xdg-utils)\n• Check if a default browser is set\n• Try opening the URL manually to test\n• Disable browser opening with .open_browser(false)"
            )
        })?;
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        return Err(HeisenbergError::config(
            "Browser opening not supported on this platform",
            "• This platform is not supported for automatic browser opening\n• Disable browser opening with .open_browser(false)\n• Open the URL manually after starting the server"
        ));
    }

    Ok(())
}
