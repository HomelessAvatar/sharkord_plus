//! Emoji picker popover component adhering to the Pure Neutral Dark specification.
//! Provides search filtering, quick-reactions top bar, categorized emojis,
//! keyboard navigation, and callback emission for message reactions.

use gpui::prelude::*;
use gpui::*;
use serde::{Deserialize, Serialize};

use crate::theme;

/// Individual emoji entry with character and searchable keywords.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmojiItem {
    pub emoji: String,
    pub name: String,
    pub keywords: Vec<String>,
    pub category: String,
}

impl EmojiItem {
    pub fn new(emoji: &str, name: &str, keywords: &[&str], category: &str) -> Self {
        Self {
            emoji: emoji.to_string(),
            name: name.to_string(),
            keywords: keywords.iter().map(|k| k.to_string()).collect(),
            category: category.to_string(),
        }
    }

    /// Returns true if the emoji name or any keyword matches the query.
    pub fn matches(&self, query: &str) -> bool {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return true;
        }
        if self.emoji.contains(&q) || self.name.to_lowercase().contains(&q) {
            return true;
        }
        self.keywords.iter().any(|k| k.to_lowercase().contains(&q))
    }
}

/// Category grouping for emoji picker tabs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmojiCategory {
    pub name: String,
    pub icon: String,
    pub emojis: Vec<EmojiItem>,
}

/// Default built-in categories and emojis.
pub fn default_emoji_categories() -> Vec<EmojiCategory> {
    vec![
        EmojiCategory {
            name: "Smileys".to_string(),
            icon: "😀".to_string(),
            emojis: vec![
                EmojiItem::new("😀", "grinning", &["smile", "happy"], "Smileys"),
                EmojiItem::new("😃", "smiley", &["smile", "happy", "joy"], "Smileys"),
                EmojiItem::new("😄", "smile", &["happy", "laugh"], "Smileys"),
                EmojiItem::new("😁", "grin", &["teeth", "happy"], "Smileys"),
                EmojiItem::new("😆", "laughing", &["lol", "haha"], "Smileys"),
                EmojiItem::new("😅", "sweat_smile", &["relief", "sweat"], "Smileys"),
                EmojiItem::new("😂", "joy", &["tears", "laugh", "funny"], "Smileys"),
                EmojiItem::new("🤣", "rofl", &["rolling", "laughing"], "Smileys"),
                EmojiItem::new("😊", "blush", &["sweet", "shy"], "Smileys"),
                EmojiItem::new("😇", "innocent", &["angel", "halo"], "Smileys"),
                EmojiItem::new(
                    "🙂",
                    "slightly_smiling_face",
                    &["okay", "neutral"],
                    "Smileys",
                ),
                EmojiItem::new("😉", "wink", &["flirt", "joke"], "Smileys"),
                EmojiItem::new("😍", "heart_eyes", &["love", "adore"], "Smileys"),
                EmojiItem::new(
                    "🥰",
                    "smiling_face_with_3_hearts",
                    &["love", "affection"],
                    "Smileys",
                ),
                EmojiItem::new("😘", "kissing_heart", &["kiss", "love"], "Smileys"),
                EmojiItem::new("😋", "yum", &["delicious", "tasty"], "Smileys"),
                EmojiItem::new(
                    "😜",
                    "stuck_out_tongue_winking_eye",
                    &["playful", "cheeky"],
                    "Smileys",
                ),
                EmojiItem::new("🤔", "thinking", &["wonder", "ponder", "hmm"], "Smileys"),
                EmojiItem::new("🤐", "zipper_mouth", &["secret", "quiet"], "Smileys"),
                EmojiItem::new("😐", "neutral_face", &["meh", "indifferent"], "Smileys"),
                EmojiItem::new("😏", "smirk", &["sneaky", "cheeky"], "Smileys"),
                EmojiItem::new("😒", "unamused", &["bored", "displeased"], "Smileys"),
                EmojiItem::new("🙄", "roll_eyes", &["eyeroll", "whatever"], "Smileys"),
                EmojiItem::new("😬", "grimacing", &["nervous", "yikes"], "Smileys"),
                EmojiItem::new("🤥", "lying_face", &["pinocchio", "lie"], "Smileys"),
                EmojiItem::new("😴", "sleeping", &["tired", "sleep", "zzz"], "Smileys"),
                EmojiItem::new("🤯", "exploding_head", &["mind_blown", "shock"], "Smileys"),
                EmojiItem::new(
                    "🥳",
                    "partying_face",
                    &["celebrate", "party", "cheers"],
                    "Smileys",
                ),
                EmojiItem::new("😎", "sunglasses", &["cool", "awesome"], "Smileys"),
                EmojiItem::new("🤓", "nerd_face", &["geek", "glasses"], "Smileys"),
                EmojiItem::new("🧐", "monocle_face", &["curious", "inspect"], "Smileys"),
                EmojiItem::new("😢", "cry", &["sad", "tear"], "Smileys"),
                EmojiItem::new("😭", "sob", &["crying", "sad"], "Smileys"),
                EmojiItem::new("😱", "scream", &["omg", "scared", "fear"], "Smileys"),
                EmojiItem::new("😡", "rage", &["angry", "mad"], "Smileys"),
                EmojiItem::new("💀", "skull", &["dead", "skeleton"], "Smileys"),
                EmojiItem::new("🤡", "clown", &["fool", "circus"], "Smileys"),
                EmojiItem::new("💩", "poop", &["crap", "shit"], "Smileys"),
                EmojiItem::new("👻", "ghost", &["spooky", "boo"], "Smileys"),
            ],
        },
        EmojiCategory {
            name: "Gestures".to_string(),
            icon: "👍".to_string(),
            emojis: vec![
                EmojiItem::new(
                    "👍",
                    "thumbsup",
                    &["plus1", "agree", "yes", "good"],
                    "Gestures",
                ),
                EmojiItem::new(
                    "👎",
                    "thumbsdown",
                    &["minus1", "disagree", "no", "bad"],
                    "Gestures",
                ),
                EmojiItem::new("👌", "ok_hand", &["perfect", "fine"], "Gestures"),
                EmojiItem::new("🤌", "pinched_fingers", &["italian", "what"], "Gestures"),
                EmojiItem::new("✌", "v", &["peace", "victory"], "Gestures"),
                EmojiItem::new("🤞", "crossed_fingers", &["luck", "hope"], "Gestures"),
                EmojiItem::new("🤟", "love_you_gesture", &["ily", "love"], "Gestures"),
                EmojiItem::new("🤘", "metal", &["rock", "horns"], "Gestures"),
                EmojiItem::new("🤙", "call_me", &["shaka", "hang_loose"], "Gestures"),
                EmojiItem::new("👈", "point_left", &["left", "finger"], "Gestures"),
                EmojiItem::new("👉", "point_right", &["right", "finger"], "Gestures"),
                EmojiItem::new("👆", "point_up", &["up", "finger"], "Gestures"),
                EmojiItem::new("👇", "point_down", &["down", "finger"], "Gestures"),
                EmojiItem::new("👏", "clap", &["applause", "praise"], "Gestures"),
                EmojiItem::new("🙌", "raised_hands", &["celebration", "hooray"], "Gestures"),
                EmojiItem::new("👐", "open_hands", &["welcome", "hug"], "Gestures"),
                EmojiItem::new("🤲", "palms_up_together", &["pray", "hope"], "Gestures"),
                EmojiItem::new("🤝", "handshake", &["deal", "agreement"], "Gestures"),
                EmojiItem::new("🙏", "pray", &["please", "thank_you", "hope"], "Gestures"),
                EmojiItem::new("💪", "muscle", &["strong", "flex", "power"], "Gestures"),
                EmojiItem::new("👀", "eyes", &["look", "see", "watch"], "Gestures"),
            ],
        },
        EmojiCategory {
            name: "Symbols".to_string(),
            icon: "❤️".to_string(),
            emojis: vec![
                EmojiItem::new("❤️", "heart", &["love", "red_heart", "like"], "Symbols"),
                EmojiItem::new("🧡", "orange_heart", &["orange", "love"], "Symbols"),
                EmojiItem::new("💛", "yellow_heart", &["yellow", "love"], "Symbols"),
                EmojiItem::new("💚", "green_heart", &["green", "love"], "Symbols"),
                EmojiItem::new("💙", "blue_heart", &["blue", "love"], "Symbols"),
                EmojiItem::new("💜", "purple_heart", &["purple", "love"], "Symbols"),
                EmojiItem::new("🖤", "black_heart", &["black", "love"], "Symbols"),
                EmojiItem::new("🤍", "white_heart", &["white", "love"], "Symbols"),
                EmojiItem::new("💔", "broken_heart", &["sad", "heartbreak"], "Symbols"),
                EmojiItem::new("✨", "sparkles", &["magic", "stars", "shiny"], "Symbols"),
                EmojiItem::new("⭐", "star", &["favorite", "gold"], "Symbols"),
                EmojiItem::new("🌟", "glowing_star", &["shine", "bright"], "Symbols"),
                EmojiItem::new("⚡", "zap", &["lightning", "fast", "electric"], "Symbols"),
                EmojiItem::new("💥", "boom", &["collision", "explosion"], "Symbols"),
                EmojiItem::new("🔥", "fire", &["hot", "lit", "flame"], "Symbols"),
                EmojiItem::new("💯", "100", &["perfect", "score", "century"], "Symbols"),
                EmojiItem::new(
                    "✅",
                    "white_check_mark",
                    &["check", "done", "yes"],
                    "Symbols",
                ),
                EmojiItem::new("❌", "x", &["cross", "cancel", "no"], "Symbols"),
                EmojiItem::new("❗", "exclamation", &["important", "warn"], "Symbols"),
                EmojiItem::new("❓", "question", &["ask", "confused", "help"], "Symbols"),
            ],
        },
        EmojiCategory {
            name: "Objects".to_string(),
            icon: "🎉".to_string(),
            emojis: vec![
                EmojiItem::new("🎉", "tada", &["party", "celebration", "hooray"], "Objects"),
                EmojiItem::new(
                    "🚀",
                    "rocket",
                    &["launch", "fast", "space", "ship"],
                    "Objects",
                ),
                EmojiItem::new("🎁", "gift", &["present", "birthday"], "Objects"),
                EmojiItem::new("🏆", "trophy", &["winner", "first", "champion"], "Objects"),
                EmojiItem::new("💡", "bulb", &["idea", "light"], "Objects"),
                EmojiItem::new("💻", "laptop", &["computer", "tech", "work"], "Objects"),
                EmojiItem::new("📱", "phone", &["mobile", "cell"], "Objects"),
                EmojiItem::new("🎮", "video_game", &["gaming", "play"], "Objects"),
                EmojiItem::new("☕", "coffee", &["tea", "morning", "drink"], "Objects"),
                EmojiItem::new("🍕", "pizza", &["food", "slice"], "Objects"),
                EmojiItem::new("🍺", "beer", &["cheers", "drink", "pub"], "Objects"),
                EmojiItem::new("🔒", "lock", &["security", "private"], "Objects"),
                EmojiItem::new("🔑", "key", &["password", "access"], "Objects"),
                EmojiItem::new("🔔", "bell", &["notification", "alert"], "Objects"),
            ],
        },
    ]
}

/// State for the emoji picker popover.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmojiPickerState {
    pub is_open: bool,
    pub query: String,
    pub selected_category: usize,
    pub quick_reactions: Vec<String>,
    pub categories: Vec<EmojiCategory>,
    pub target_message_id: Option<u64>,
    pub selected_index: usize,
}

impl Default for EmojiPickerState {
    fn default() -> Self {
        Self::new()
    }
}

impl EmojiPickerState {
    /// Creates a new `EmojiPickerState` with default categories and quick reactions.
    pub fn new() -> Self {
        Self {
            is_open: false,
            query: String::new(),
            selected_category: 0,
            quick_reactions: vec![
                "👍".to_string(),
                "❤️".to_string(),
                "😂".to_string(),
                "🎉".to_string(),
                "🔥".to_string(),
                "🚀".to_string(),
            ],
            categories: default_emoji_categories(),
            target_message_id: None,
            selected_index: 0,
        }
    }

    /// Opens the emoji picker targeting a specific message.
    pub fn open_for_message(&mut self, message_id: u64) {
        self.is_open = true;
        self.target_message_id = Some(message_id);
        self.query.clear();
        self.selected_index = 0;
    }

    /// Closes the emoji picker and clears message target.
    pub fn close(&mut self) {
        self.is_open = false;
        self.target_message_id = None;
        self.query.clear();
    }

    /// Updates the search filter query.
    pub fn search(&mut self, query: impl Into<String>) {
        self.query = query.into();
        self.selected_index = 0;
    }

    /// Selects an active category tab index.
    pub fn select_category(&mut self, index: usize) {
        if index < self.categories.len() {
            self.selected_category = index;
            self.selected_index = 0;
        }
    }

    /// Returns the filtered emojis matching the current query.
    /// If query is empty, returns all emojis in the selected category.
    pub fn filtered_emojis(&self) -> Vec<EmojiItem> {
        let q = self.query.trim().to_lowercase();
        if q.is_empty() {
            if let Some(cat) = self.categories.get(self.selected_category) {
                cat.emojis.clone()
            } else {
                Vec::new()
            }
        } else {
            self.categories
                .iter()
                .flat_map(|cat| cat.emojis.iter().filter(|e| e.matches(&q)).cloned())
                .collect()
        }
    }

    /// Selects an emoji, closes the picker, and returns the selected emoji string.
    pub fn select_emoji(&mut self, emoji: &str) -> Option<String> {
        let res = emoji.to_string();
        self.close();
        Some(res)
    }

    /// Moves the keyboard selection index to the next item.
    pub fn move_selection_next(&mut self) {
        let len = self.filtered_emojis().len();
        if len > 0 {
            self.selected_index = (self.selected_index + 1) % len;
        }
    }

    /// Moves the keyboard selection index to the previous item.
    pub fn move_selection_prev(&mut self) {
        let len = self.filtered_emojis().len();
        if len > 0 {
            if self.selected_index == 0 {
                self.selected_index = len - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    /// Returns the currently highlighted emoji string from keyboard navigation.
    pub fn selected_emoji(&self) -> Option<String> {
        let filtered = self.filtered_emojis();
        filtered.get(self.selected_index).map(|e| e.emoji.clone())
    }
}

/// Standalone visual element view for the emoji picker popover.
#[derive(Debug, Clone, PartialEq)]
pub struct EmojiPickerView {
    pub state: EmojiPickerState,
}

impl EmojiPickerView {
    /// Creates a new `EmojiPickerView` wrapping the given state.
    pub fn new(state: EmojiPickerState) -> Self {
        Self { state }
    }

    /// Renders the complete Pure Neutral Dark emoji picker popover element tree.
    pub fn render_element(&self) -> impl IntoElement {
        let filtered = self.state.filtered_emojis();
        let query_str = &self.state.query;

        div()
            .id("emoji_picker_popover")
            .flex()
            .flex_col()
            .w(px(320.0))
            .max_h(px(400.0))
            .rounded_xl()
            .bg(theme::bg_elevated())
            .border_1()
            .border_color(theme::border_subtle())
            .shadow_xl()
            .overflow_hidden()
            // 1. Search Bar Header
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_2()
                    .bg(theme::bg_surface())
                    .border_b_1()
                    .border_color(theme::border_hairline())
                    .child(div().text_xs().text_color(theme::text_muted()).child("🔍"))
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .text_color(if query_str.is_empty() {
                                theme::text_muted()
                            } else {
                                theme::text_primary()
                            })
                            .child(if query_str.is_empty() {
                                "Search emoji...".to_string()
                            } else {
                                query_str.clone()
                            }),
                    ),
            )
            // 2. Quick Reactions Strip
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .py_1p5()
                    .bg(theme::bg_subsurface())
                    .border_b_1()
                    .border_color(theme::border_hairline())
                    .children(self.state.quick_reactions.iter().map(|emoji| {
                        div()
                            .cursor_pointer()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(32.0))
                            .h(px(32.0))
                            .rounded_md()
                            .hover(|s| s.bg(theme::bg_surface_4()))
                            .text_base()
                            .child(emoji.clone())
                    })),
            )
            // 3. Category Tabs
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .px_2()
                    .py_1()
                    .bg(theme::bg_base())
                    .border_b_1()
                    .border_color(theme::border_hairline())
                    .children(self.state.categories.iter().enumerate().map(|(idx, cat)| {
                        let is_active = idx == self.state.selected_category && query_str.is_empty();
                        div()
                            .cursor_pointer()
                            .flex()
                            .items_center()
                            .justify_center()
                            .px_2()
                            .py_1()
                            .rounded_sm()
                            .bg(if is_active {
                                theme::bg_surface_4()
                            } else {
                                hsla(0.0, 0.0, 0.0, 0.0)
                            })
                            .border_b_2()
                            .border_color(if is_active {
                                theme::accent()
                            } else {
                                hsla(0.0, 0.0, 0.0, 0.0)
                            })
                            .text_xs()
                            .child(cat.icon.clone())
                    })),
            )
            // 4. Scrollable Emoji Grid
            .child(
                div()
                    .id("emoji_grid_scroll")
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap_1()
                    .p_2()
                    .max_h(px(240.0))
                    .overflow_y_scroll()
                    .children(if filtered.is_empty() {
                        vec![div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w_full()
                            .py_8()
                            .text_xs()
                            .text_color(theme::text_muted())
                            .child("No emojis found")
                            .into_any_element()]
                    } else {
                        filtered
                            .iter()
                            .enumerate()
                            .map(|(idx, item)| {
                                let is_selected = idx == self.state.selected_index;
                                div()
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(32.0))
                                    .h(px(32.0))
                                    .rounded_md()
                                    .bg(if is_selected {
                                        theme::bg_surface_4()
                                    } else {
                                        hsla(0.0, 0.0, 0.0, 0.0)
                                    })
                                    .hover(|s| s.bg(theme::bg_surface_4()))
                                    .text_lg()
                                    .child(item.emoji.clone())
                                    .into_any_element()
                            })
                            .collect()
                    }),
            )
    }
}
