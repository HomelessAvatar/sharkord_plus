//! In-app auto-updater notification badge and modal dialog components in GPUI.
//! Adheres to the Pure Neutral Dark design system specifications.

use gpui::prelude::*;
use gpui::*;
use serde::{Deserialize, Serialize};
use sharkord_core::updater::{ReleaseInfo, UpdateStatus};

use crate::theme;

/// Small notification badge pill displaying available update version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdaterBadgeView {
    pub version: String,
}

impl UpdaterBadgeView {
    /// Creates a new `UpdaterBadgeView` with the specified version string.
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
        }
    }

    /// Creates an `UpdaterBadgeView` if the given `UpdateStatus` contains an available or ready update.
    pub fn from_status(status: &UpdateStatus) -> Option<Self> {
        match status {
            UpdateStatus::UpdateAvailable(info) => {
                let v = if info.version.is_empty() {
                    info.tag_name.clone()
                } else {
                    info.version.clone()
                };
                Some(Self::new(v))
            }
            UpdateStatus::ReadyToInstall { version, .. } => Some(Self::new(version.clone())),
            _ => None,
        }
    }

    /// Formats the badge text label.
    pub fn badge_label(&self) -> String {
        let v_str = if self.version.starts_with('v') || self.version.starts_with('V') {
            self.version.clone()
        } else {
            format!("v{}", self.version)
        };
        format!("✨ Update Available ({})", v_str)
    }

    /// Renders the small badge pill element.
    pub fn render_element(&self) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_1()
            .px_2()
            .py_0p5()
            .rounded_full()
            .bg(theme::bg_surface_4())
            .border_1()
            .border_color(theme::border_subtle())
            .text_xs()
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(theme::accent_hover())
            .cursor_pointer()
            .child(self.badge_label())
    }
}

impl Render for UpdaterBadgeView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}

/// Modal dialog view for software update notifications, release notes, and progress.
#[derive(Debug, Clone, PartialEq)]
pub struct UpdaterModalView {
    pub status: UpdateStatus,
}

impl UpdaterModalView {
    /// Creates a new `UpdaterModalView` initialized with the given `UpdateStatus`.
    pub fn new(status: &UpdateStatus) -> Self {
        Self {
            status: status.clone(),
        }
    }

    /// Returns the label for the primary action button based on the current update state.
    pub fn action_button_label(&self) -> &'static str {
        match &self.status {
            UpdateStatus::UpdateAvailable(_) => "Download & Install",
            UpdateStatus::Downloading { .. } => "Downloading...",
            UpdateStatus::ReadyToInstall { .. } => "Restart & Install",
            UpdateStatus::Checking => "Checking...",
            UpdateStatus::Error(_) => "Retry",
            UpdateStatus::UpToDate => "Up to Date",
            UpdateStatus::Idle => "Check for Updates",
        }
    }

    /// Returns the progress fraction (0.0 to 1.0) when downloading.
    pub fn progress_percent(&self) -> Option<f32> {
        match &self.status {
            UpdateStatus::Downloading { percent, .. } => Some(*percent),
            _ => None,
        }
    }

    /// Resolves target version string for display from the update status.
    fn target_version(&self) -> Option<String> {
        match &self.status {
            UpdateStatus::UpdateAvailable(info) => {
                if !info.version.is_empty() {
                    Some(info.version.clone())
                } else if !info.tag_name.is_empty() {
                    Some(info.tag_name.clone())
                } else {
                    None
                }
            }
            UpdateStatus::ReadyToInstall { version, .. } => Some(version.clone()),
            _ => None,
        }
    }

    /// Renders the centered 520px Pure Neutral Dark modal overlay dialog.
    pub fn render_element(&self) -> impl IntoElement {
        let current_pkg_version = env!("CARGO_PKG_VERSION");
        let target_ver = self.target_version();

        let header_title = match &self.status {
            UpdateStatus::ReadyToInstall { .. } => "Update Ready to Install",
            UpdateStatus::Downloading { .. } => "Downloading Update",
            UpdateStatus::Checking => "Checking for Updates",
            UpdateStatus::UpToDate => "VibeShark is Up to Date",
            UpdateStatus::Error(_) => "Update Error",
            _ => "Update Available",
        };

        let is_primary_enabled = matches!(
            &self.status,
            UpdateStatus::UpdateAvailable(_)
                | UpdateStatus::ReadyToInstall { .. }
                | UpdateStatus::Error(_)
                | UpdateStatus::Idle
        );

        // Version tags comparison pill
        let mut version_badge = div().flex().flex_row().items_center().gap_1p5().child(
            div()
                .px_2()
                .py_0p5()
                .rounded_md()
                .bg(theme::bg_subsurface())
                .text_xs()
                .text_color(theme::text_muted())
                .child(format!("v{}", current_pkg_version)),
        );

        if let Some(target) = &target_ver {
            let target_str = if target.starts_with('v') || target.starts_with('V') {
                target.clone()
            } else {
                format!("v{}", target)
            };

            version_badge = version_badge
                .child(div().text_xs().text_color(theme::text_muted()).child("→"))
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .bg(theme::bg_surface_4())
                        .text_xs()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme::status_online())
                        .child(target_str),
                );
        }

        // Build changelog / details content
        let changelog_content: AnyElement = match &self.status {
            UpdateStatus::UpdateAvailable(ReleaseInfo { name, body, .. }) => {
                let release_title = if !name.is_empty() {
                    name.as_str()
                } else {
                    "Release Notes"
                };

                let mut list = div().flex().flex_col().gap_1p5().child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme::text_primary())
                        .child(release_title.to_string()),
                );

                if body.trim().is_empty() {
                    list = list.child(
                        div()
                            .text_xs()
                            .text_color(theme::text_muted())
                            .child("No release notes provided with this update."),
                    );
                } else {
                    for line in body.lines() {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }

                        if trimmed.starts_with("### ") || trimmed.starts_with("## ") {
                            let heading = trimmed.trim_start_matches('#').trim();
                            list = list.child(
                                div()
                                    .pt_2()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(theme::text_primary())
                                    .child(heading.to_string()),
                            );
                        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                            let bullet = trimmed[2..].trim();
                            list = list.child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_start()
                                    .gap_1p5()
                                    .child(div().text_xs().text_color(theme::accent()).child("•"))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme::text_secondary())
                                            .child(bullet.to_string()),
                                    ),
                            );
                        } else {
                            list = list.child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_secondary())
                                    .child(trimmed.to_string()),
                            );
                        }
                    }
                }
                list.into_any_element()
            }
            UpdateStatus::ReadyToInstall { version, .. } => div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme::status_online())
                        .child(format!("Version {} Ready to Install", version)),
                )
                .child(div().text_xs().text_color(theme::text_secondary()).child(
                    "The update package has been downloaded and verified. \
                             Restart VibeShark now to apply the update seamlessly.",
                ))
                .into_any_element(),
            UpdateStatus::Downloading {
                percent,
                downloaded_bytes,
                total_bytes,
            } => {
                let mb_down = *downloaded_bytes as f64 / 1_048_576.0;
                let mb_total = *total_bytes as f64 / 1_048_576.0;
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme::text_primary())
                            .child("Downloading update package in the background..."),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::text_muted())
                            .child(format!(
                                "Progress: {:.1} MB of {:.1} MB ({:.0}%)",
                                mb_down,
                                mb_total,
                                percent * 100.0
                            )),
                    )
                    .into_any_element()
            }
            UpdateStatus::Error(err) => div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme::status_dnd())
                        .child("Update Check Failed"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme::text_secondary())
                        .child(err.clone()),
                )
                .into_any_element(),
            UpdateStatus::UpToDate => div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme::status_online())
                        .child("You're using the latest version of VibeShark."),
                )
                .into_any_element(),
            UpdateStatus::Checking => div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme::text_secondary())
                        .child("Checking GitHub Releases for new updates..."),
                )
                .into_any_element(),
            UpdateStatus::Idle => div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .text_color(theme::text_muted())
                        .child("No update check in progress."),
                )
                .into_any_element(),
        };

        // Absolute full-screen backdrop with centered 520px modal dialog
        let mut modal_box = div()
            .flex()
            .flex_col()
            .w(px(520.0))
            .max_h(px(480.0))
            .bg(theme::bg_elevated())
            .border_1()
            .border_color(theme::border_subtle())
            .rounded_xl()
            .shadow_lg()
            .overflow_hidden()
            // Header
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px_5()
                    .py_4()
                    .border_b_1()
                    .border_color(theme::border_hairline())
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(theme::text_primary())
                                    .child(header_title),
                            )
                            .child(version_badge),
                    )
                    .child(
                        div()
                            .px_2()
                            .py_0p5()
                            .rounded_sm()
                            .bg(theme::bg_surface())
                            .border_1()
                            .border_color(theme::border_hairline())
                            .text_xs()
                            .text_color(theme::text_muted())
                            .child("ESC"),
                    ),
            )
            // Scrollable changelog / description body
            .child(
                div()
                    .id("updater_changelog_scroll")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .max_h(px(220.0))
                    .p_4()
                    .mx_5()
                    .my_3()
                    .rounded_lg()
                    .bg(theme::bg_subsurface())
                    .border_1()
                    .border_color(theme::border_hairline())
                    .overflow_y_scroll()
                    .child(changelog_content),
            );

        // Progress bar (rendered when downloading)
        if let UpdateStatus::Downloading {
            percent,
            downloaded_bytes,
            total_bytes,
        } = &self.status
        {
            let mb_down = *downloaded_bytes as f64 / 1_048_576.0;
            let mb_total = *total_bytes as f64 / 1_048_576.0;

            modal_box = modal_box.child(
                div()
                    .flex()
                    .flex_col()
                    .px_5()
                    .pb_3()
                    .gap_1p5()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme::text_secondary())
                                    .child(format!(
                                        "Downloading update... {:.0}%",
                                        percent * 100.0
                                    )),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child(format!("{:.1} MB / {:.1} MB", mb_down, mb_total)),
                            ),
                    )
                    .child(
                        div()
                            .w_full()
                            .h(px(6.0))
                            .bg(theme::bg_surface())
                            .rounded_full()
                            .overflow_hidden()
                            .child(
                                div()
                                    .h_full()
                                    .w(gpui::DefiniteLength::Fraction(percent.clamp(0.0, 1.0)))
                                    .rounded_full()
                                    .bg(theme::accent()),
                            ),
                    ),
            );
        }

        // Action buttons footer
        let primary_btn_bg = if is_primary_enabled {
            theme::accent()
        } else {
            theme::bg_surface_4()
        };

        let primary_btn_text = if is_primary_enabled {
            theme::text_primary()
        } else {
            theme::text_muted()
        };

        let footer = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px_5()
            .py_3()
            .bg(theme::bg_surface())
            .border_t_1()
            .border_color(theme::border_hairline())
            .child(
                div()
                    .px_3()
                    .py_1p5()
                    .rounded_md()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(theme::text_muted())
                    .cursor_pointer()
                    .child("Dismiss"),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(theme::bg_surface_4())
                            .border_1()
                            .border_color(theme::border_hairline())
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(theme::text_secondary())
                            .cursor_pointer()
                            .child("Remind Me Later"),
                    )
                    .child(
                        div()
                            .px_4()
                            .py_1p5()
                            .rounded_md()
                            .bg(primary_btn_bg)
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(primary_btn_text)
                            .cursor_pointer()
                            .child(self.action_button_label()),
                    ),
            );

        modal_box = modal_box.child(footer);

        div()
            .absolute()
            .top_0()
            .left_0()
            .w_full()
            .h_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(hsla(0.0, 0.0, 0.0, 0.65))
            .child(modal_box)
    }
}

impl Render for UpdaterModalView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}
