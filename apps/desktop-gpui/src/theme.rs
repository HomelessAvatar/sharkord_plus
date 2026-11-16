//! Pure Neutral Dark design system theme tokens for Sharkord Plus desktop.
//! Defined in accordance with DESIGN.md.

use gpui::{hsla, rgb, Hsla, Rgba};

// Hex string representations matching DESIGN.md
pub const BG_BASE_HEX: &str = "#0a0b0e";
pub const BG_SURFACE_HEX: &str = "#111216";
pub const BG_SUBSURFACE_HEX: &str = "#16171d";
pub const BG_ELEVATED_HEX: &str = "#1c1e24";
pub const BG_SURFACE_4_HEX: &str = "#242630";
pub const BORDER_HAIRLINE_HEX: &str = "rgba(255, 255, 255, 0.08)";
pub const BORDER_SUBTLE_HEX: &str = "rgba(255, 255, 255, 0.12)";
pub const TEXT_PRIMARY_HEX: &str = "#f3f4f6";
pub const TEXT_SECONDARY_HEX: &str = "#9ca3af";
pub const TEXT_MUTED_HEX: &str = "#6b7280";
pub const ACCENT_HEX: &str = "#3b82f6";
pub const ACCENT_HOVER_HEX: &str = "#60a5fa";
pub const STATUS_ONLINE_HEX: &str = "#22c55e";
pub const STATUS_IDLE_HEX: &str = "#eab308";
pub const STATUS_DND_HEX: &str = "#ef4444";
pub const STATUS_OFFLINE_HEX: &str = "#6b7280";
pub const STATUS_STREAMING_HEX: &str = "#8b5cf6";

// Pure Neutral Dark color integer values (RGB u32)
pub const BG_BASE: u32 = 0x0a0b0e;
pub const BG_SURFACE: u32 = 0x111216;
pub const BG_SUBSURFACE: u32 = 0x16171d;
pub const BG_ELEVATED: u32 = 0x1c1e24;
pub const BG_SURFACE_4: u32 = 0x242630;
pub const BORDER_HAIRLINE: (u32, f32) = (0xffffff, 0.08);
pub const BORDER_SUBTLE: (u32, f32) = (0xffffff, 0.12);
pub const TEXT_PRIMARY: u32 = 0xf3f4f6;
pub const TEXT_SECONDARY: u32 = 0x9ca3af;
pub const TEXT_MUTED: u32 = 0x6b7280;
pub const ACCENT: u32 = 0x3b82f6;
pub const ACCENT_HOVER: u32 = 0x60a5fa;
pub const STATUS_ONLINE: u32 = 0x22c55e;
pub const STATUS_IDLE: u32 = 0xeab308;
pub const STATUS_DND: u32 = 0xef4444;
pub const STATUS_OFFLINE: u32 = 0x6b7280;
pub const STATUS_STREAMING: u32 = 0x8b5cf6;

// Hsla conversion helpers (primary format for GPUI elements)
pub fn bg_base() -> Hsla {
    rgb(BG_BASE).into()
}

pub fn bg_surface() -> Hsla {
    rgb(BG_SURFACE).into()
}

pub fn bg_subsurface() -> Hsla {
    rgb(BG_SUBSURFACE).into()
}

pub fn bg_elevated() -> Hsla {
    rgb(BG_ELEVATED).into()
}

pub fn bg_surface_4() -> Hsla {
    rgb(BG_SURFACE_4).into()
}

pub fn border_hairline() -> Hsla {
    hsla(0.0, 0.0, 1.0, BORDER_HAIRLINE.1)
}

pub fn border_subtle() -> Hsla {
    hsla(0.0, 0.0, 1.0, BORDER_SUBTLE.1)
}

pub fn text_primary() -> Hsla {
    rgb(TEXT_PRIMARY).into()
}

pub fn text_secondary() -> Hsla {
    rgb(TEXT_SECONDARY).into()
}

pub fn text_muted() -> Hsla {
    rgb(TEXT_MUTED).into()
}

pub fn accent() -> Hsla {
    rgb(ACCENT).into()
}

pub fn accent_hover() -> Hsla {
    rgb(ACCENT_HOVER).into()
}

pub fn status_online() -> Hsla {
    rgb(STATUS_ONLINE).into()
}

pub fn status_idle() -> Hsla {
    rgb(STATUS_IDLE).into()
}

pub fn status_dnd() -> Hsla {
    rgb(STATUS_DND).into()
}

pub fn status_offline() -> Hsla {
    rgb(STATUS_OFFLINE).into()
}

pub fn status_streaming() -> Hsla {
    rgb(STATUS_STREAMING).into()
}

// Rgba conversion helpers
pub fn bg_base_rgba() -> Rgba {
    rgb(BG_BASE)
}

pub fn bg_surface_rgba() -> Rgba {
    rgb(BG_SURFACE)
}

pub fn bg_subsurface_rgba() -> Rgba {
    rgb(BG_SUBSURFACE)
}

pub fn bg_elevated_rgba() -> Rgba {
    rgb(BG_ELEVATED)
}

pub fn bg_surface_4_rgba() -> Rgba {
    rgb(BG_SURFACE_4)
}

pub fn border_hairline_rgba() -> Rgba {
    border_hairline().into()
}

pub fn border_subtle_rgba() -> Rgba {
    border_subtle().into()
}

pub fn text_primary_rgba() -> Rgba {
    rgb(TEXT_PRIMARY)
}

pub fn text_secondary_rgba() -> Rgba {
    rgb(TEXT_SECONDARY)
}

pub fn text_muted_rgba() -> Rgba {
    rgb(TEXT_MUTED)
}

pub fn accent_rgba() -> Rgba {
    rgb(ACCENT)
}

pub fn accent_hover_rgba() -> Rgba {
    rgb(ACCENT_HOVER)
}

pub fn status_online_rgba() -> Rgba {
    rgb(STATUS_ONLINE)
}

pub fn status_idle_rgba() -> Rgba {
    rgb(STATUS_IDLE)
}

pub fn status_dnd_rgba() -> Rgba {
    rgb(STATUS_DND)
}

pub fn status_offline_rgba() -> Rgba {
    rgb(STATUS_OFFLINE)
}

pub fn status_streaming_rgba() -> Rgba {
    rgb(STATUS_STREAMING)
}
