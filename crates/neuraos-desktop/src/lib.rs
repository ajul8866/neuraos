//! neuraos-desktop -- Native desktop integration for NeuraOS
//!
//! Provides system tray, notifications, and native window management
//! for desktop environments (Linux/Wayland, macOS, Windows).

use std::sync::Arc;
use tokio::sync::broadcast;

pub mod notifications;
pub mod tray;
pub mod window;

pub use notifications::{Notification, NotificationLevel, NotificationManager};
pub use tray::{TrayEvent, TrayManager};
pub use window::{WindowConfig, WindowManager};

/// Desktop integration configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DesktopConfig {
    pub app_name: String,
    pub enable_tray: bool,
    pub enable_notifications: bool,
    pub notification_level: notifications::NotificationLevel,
    pub auto_start: bool,
}

impl Default for DesktopConfig {
    fn default() -> Self {
        Self {
            app_name: "NeuraOS".to_string(),
            enable_tray: true,
            enable_notifications: true,
            notification_level: notifications::NotificationLevel::Info,
            auto_start: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DesktopEvent {
    OpenRequested,
    QuitRequested,
    NotificationActivated { id: u32, action: String },
    SystemSleep,
    SystemWake,
}

pub struct DesktopIntegration {
    config: DesktopConfig,
    notification_manager: Arc<NotificationManager>,
    tray_manager: Arc<TrayManager>,
    event_tx: broadcast::Sender<DesktopEvent>,
}

impl DesktopIntegration {
    pub fn new(config: DesktopConfig) -> Self {
        let (event_tx, _) = broadcast::channel(128);
        Self {
            config,
            notification_manager: Arc::new(NotificationManager::new()),
            tray_manager: Arc::new(TrayManager::new()),
            event_tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DesktopEvent> {
        self.event_tx.subscribe()
    }

    pub async fn notify(&self, notification: Notification) -> anyhow::Result<u32> {
        self.notification_manager.send(notification).await
    }

    pub async fn set_tray_status(&self, status: &str) -> anyhow::Result<()> {
        self.tray_manager.set_tooltip(status).await
    }

    pub fn config(&self) -> &DesktopConfig { &self.config }
}

pub mod notifications {
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum NotificationLevel { Debug, Info, Warning, Error, Critical }

    #[derive(Debug, Clone)]
    pub struct Notification {
        pub title: String,
        pub body: String,
        pub level: NotificationLevel,
        pub actions: Vec<String>,
        pub timeout_ms: Option<u32>,
    }

    impl Notification {
        pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
            Self { title: title.into(), body: body.into(), level: NotificationLevel::Info,
                   actions: vec![], timeout_ms: Some(5000) }
        }
    }

    pub struct NotificationManager {
        counter: std::sync::atomic::AtomicU32,
    }

    impl NotificationManager {
        pub fn new() -> Self { Self { counter: std::sync::atomic::AtomicU32::new(1) } }
        pub async fn send(&self, _n: Notification) -> anyhow::Result<u32> {
            Ok(self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
        }
    }
}

pub mod tray {
    #[derive(Debug, Clone)]
    pub enum TrayEvent { LeftClick, RightClick, DoubleClick, MenuItemClicked(String) }

    pub struct TrayManager { tooltip: tokio::sync::RwLock<String> }
    impl TrayManager {
        pub fn new() -> Self { Self { tooltip: tokio::sync::RwLock::new("NeuraOS".to_string()) } }
        pub async fn set_tooltip(&self, t: &str) -> anyhow::Result<()> {
            *self.tooltip.write().await = t.to_string(); Ok(())
        }
    }
}

pub mod window {
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct WindowConfig { pub title: String, pub width: u32, pub height: u32,
                              pub resizable: bool, pub always_on_top: bool }
    impl Default for WindowConfig {
        fn default() -> Self {
            Self { title: "NeuraOS".to_string(), width: 1200, height: 800,
                   resizable: true, always_on_top: false }
        }
    }
    pub struct WindowManager { config: WindowConfig }
    impl WindowManager {
        pub fn new(config: WindowConfig) -> Self { Self { config } }
        pub fn config(&self) -> &WindowConfig { &self.config }
    }
}
