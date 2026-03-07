//! # neuraos-desktop
//! Desktop integration for NeuraOS -- system tray, native notifications, local UI.

#![warn(missing_docs)]

pub mod notifications;
pub mod tray;
pub mod window;

pub mod notifications {
    //! Desktop notification support.
    use anyhow::Result;

    /// Send a desktop notification.
    pub fn notify(title: &str, body: &str) -> Result<()> {
        tracing::info!("Desktop notification: [{title}] {body}");
        // Platform-specific implementation goes here
        Ok(())
    }
}

pub mod tray {
    //! System tray icon and menu.
}

pub mod window {
    //! Native window management.
}
