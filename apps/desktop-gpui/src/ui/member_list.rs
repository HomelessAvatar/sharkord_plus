//! Right-hand member list sidebar drawer component (240px fixed width, bg_subsurface).
//! Categorizes server members by presence status (Online, Offline) with avatar initials
//! and status indicator dots according to the Pure Neutral Dark design specification.

use gpui::prelude::*;
use gpui::*;
use serde::{Deserialize, Serialize};
use sharkord_core::types::User;

use crate::theme;

/// Categorized group of members partitioned by presence status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategorizedMemberGroup {
    pub title: String,
    pub members: Vec<User>,
}

/// Resolves the status dot indicator color based on user status string.
pub fn status_indicator_color(status: Option<&str>) -> Hsla {
    match status {
        Some("online") => theme::status_online(),
        Some("idle") => theme::status_idle(),
        Some("dnd") => theme::status_dnd(),
        Some("streaming") => theme::status_streaming(),
        _ => theme::status_offline(),
    }
}

/// Partitions a list of users into presence-based member groups:
/// ONLINE (`online`, `idle`, `dnd`, `streaming`) and OFFLINE (`offline` or None).
pub fn compose_member_groups(users: &[User]) -> Vec<CategorizedMemberGroup> {
    if users.is_empty() {
        return Vec::new();
    }

    let mut online_users = Vec::new();
    let mut offline_users = Vec::new();

    for user in users {
        let is_online = matches!(
            user.status.as_deref(),
            Some("online") | Some("idle") | Some("dnd") | Some("streaming")
        );

        if is_online {
            online_users.push(user.clone());
        } else {
            offline_users.push(user.clone());
        }
    }

    let mut groups = Vec::new();

    if !online_users.is_empty() {
        groups.push(CategorizedMemberGroup {
            title: format!("ONLINE — {}", online_users.len()),
            members: online_users,
        });
    }

    if !offline_users.is_empty() {
        groups.push(CategorizedMemberGroup {
            title: format!("OFFLINE — {}", offline_users.len()),
            members: offline_users,
        });
    }

    groups
}

/// Member list sidebar drawer view component.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MemberListSidebarView {
    pub groups: Vec<CategorizedMemberGroup>,
}

impl MemberListSidebarView {
    /// Creates a new `MemberListSidebarView` from a slice of users.
    pub fn new(users: &[User]) -> Self {
        let groups = compose_member_groups(users);
        Self { groups }
    }

    /// Returns the total count of members across all presence groups.
    pub fn total_members_count(&self) -> usize {
        self.groups.iter().map(|g| g.members.len()).sum()
    }

    /// Renders the member list sidebar element tree.
    pub fn render_element(&self) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w(px(240.0))
            .h_full()
            .bg(theme::bg_subsurface())
            .border_l_1()
            .border_color(theme::border_hairline())
            .overflow_hidden()
            .child(
                div()
                    .id("member_list_scroll")
                    .flex()
                    .flex_col()
                    .gap_4()
                    .p_3()
                    .flex_1()
                    .overflow_y_scroll()
                    .children(self.groups.iter().map(|group| {
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            // Group section header
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .py_1()
                                    .px_1()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(theme::text_muted())
                                    .child(group.title.clone()),
                            )
                            // Members in this section
                            .children(group.members.iter().map(|user| {
                                let status = user.status.as_deref();
                                let status_color = status_indicator_color(status);
                                let initial = user
                                    .name
                                    .chars()
                                    .next()
                                    .unwrap_or('?')
                                    .to_uppercase()
                                    .to_string();

                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_3()
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .child(
                                        // Avatar with status indicator dot
                                        div()
                                            .relative()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .w(px(32.0))
                                            .h(px(32.0))
                                            .rounded_full()
                                            .bg(theme::bg_surface_4())
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(theme::text_primary())
                                            .child(initial)
                                            .child(
                                                div()
                                                    .absolute()
                                                    .bottom_0()
                                                    .right_0()
                                                    .w(px(10.0))
                                                    .h(px(10.0))
                                                    .rounded_full()
                                                    .border_1()
                                                    .border_color(theme::bg_subsurface())
                                                    .bg(status_color),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(theme::text_primary())
                                            .child(user.name.clone()),
                                    )
                            }))
                    })),
            )
    }
}

impl Render for MemberListSidebarView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}
