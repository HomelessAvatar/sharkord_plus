//! Screen sharing capture engine powered by `xcap`.
//! Enumerates active monitors and visible application windows with safe headless fallback.

use serde::{Deserialize, Serialize};

/// Type of capture source: entire display monitor or individual window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureSourceType {
    Monitor,
    Window,
}

impl CaptureSourceType {
    /// Returns true if the capture source is a physical monitor display.
    pub fn is_monitor(&self) -> bool {
        matches!(self, CaptureSourceType::Monitor)
    }

    /// Returns true if the capture source is an application window.
    pub fn is_window(&self) -> bool {
        matches!(self, CaptureSourceType::Window)
    }
}

/// Metadata describing an available capture target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureSource {
    pub id: String,
    pub name: String,
    pub source_type: CaptureSourceType,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

impl CaptureSource {
    /// Creates a new `CaptureSource`.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        source_type: CaptureSourceType,
        width: u32,
        height: u32,
        is_primary: bool,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            source_type,
            width,
            height,
            is_primary,
        }
    }

    /// Returns true if this source is a physical monitor display.
    pub fn is_monitor(&self) -> bool {
        self.source_type.is_monitor()
    }

    /// Returns true if this source is an application window.
    pub fn is_window(&self) -> bool {
        self.source_type.is_window()
    }
}

/// Screen and application window capture enumeration engine.
pub struct ScreenCaptureEngine;

impl ScreenCaptureEngine {
    /// Returns fallback capture sources for headless environments or when no displays are found.
    pub fn fallback_sources() -> Vec<CaptureSource> {
        vec![
            CaptureSource {
                id: "monitor:primary".to_string(),
                name: "Primary Display (1920x1080)".to_string(),
                source_type: CaptureSourceType::Monitor,
                width: 1920,
                height: 1080,
                is_primary: true,
            },
            CaptureSource {
                id: "window:terminal".to_string(),
                name: "Sharkord Terminal".to_string(),
                source_type: CaptureSourceType::Window,
                width: 1280,
                height: 720,
                is_primary: false,
            },
        ]
    }

    /// Enumerates all active display monitors and application windows.
    /// Falls back to safe mock sources if enumeration produces no items.
    pub fn enumerate_sources() -> Vec<CaptureSource> {
        let mut sources = Vec::new();

        // 1. Enumerate Monitors
        if let Ok(monitors) = xcap::Monitor::all() {
            for monitor in monitors {
                let name = monitor.name().unwrap_or_else(|_| "Display".to_string());
                let width = monitor.width().unwrap_or(1920);
                let height = monitor.height().unwrap_or(1080);
                let is_primary = monitor.is_primary().unwrap_or(false);
                let id = format!("monitor:{}", monitor.id().map(|i| i.to_string()).unwrap_or_else(|_| name.clone()));

                sources.push(CaptureSource {
                    id,
                    name,
                    source_type: CaptureSourceType::Monitor,
                    width,
                    height,
                    is_primary,
                });
            }
        }

        // 2. Enumerate Windows
        if let Ok(windows) = xcap::Window::all() {
            for window in windows {
                let title = window.title().unwrap_or_default();
                let app_name = window.app_name().unwrap_or_default();
                let name = if !title.trim().is_empty() {
                    title.trim().to_string()
                } else if !app_name.trim().is_empty() {
                    app_name.trim().to_string()
                } else {
                    continue;
                };

                let width = window.width().unwrap_or(1280);
                let height = window.height().unwrap_or(720);
                let id = format!("window:{}", window.id().map(|i| i.to_string()).unwrap_or_else(|_| name.clone()));

                sources.push(CaptureSource {
                    id,
                    name,
                    source_type: CaptureSourceType::Window,
                    width,
                    height,
                    is_primary: false,
                });
            }
        }

        if sources.is_empty() {
            Self::fallback_sources()
        } else {
            sources
        }
    }
}
