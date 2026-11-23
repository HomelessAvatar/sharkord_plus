//! Markdown parser and syntax-highlighted element renderer for Sharkord GPUI client.
//! Supports bold, italic, strikethrough, inline code, spoilers, blockquotes,
//! headings, links, and fenced code blocks with language badges and copy actions.
//! Adheres strictly to the Pure Neutral Dark design specification (#0A0B0E base,
//! #111216 surface, #16171d subsurface, #1c1e24 elevated, #3B82F6 accent, #F3F4F6 text).

use gpui::prelude::*;
use gpui::*;
use serde::{Deserialize, Serialize};

use crate::theme;

/// Parsed Markdown AST token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkdownToken {
    /// Plain text string.
    Text(String),
    /// Bold text (**bold**).
    Bold(String),
    /// Italic text (*italic*).
    Italic(String),
    /// Strikethrough text (~~strike~~).
    Strikethrough(String),
    /// Monospaced inline code (`code`).
    InlineCode(String),
    /// Click-to-reveal spoiler (||spoiler||).
    Spoiler(String),
    /// Blockquote line (> quote).
    Blockquote(String),
    /// Section heading (# Heading).
    Heading(String),
    /// Hyperlink with title and target URL ([title](url)).
    Link { title: String, url: String },
    /// Multi-line fenced code block (```lang ... ```).
    CodeBlock {
        language: Option<String>,
        code: String,
    },
}

impl MarkdownToken {
    /// Returns the plain text representation of the token content.
    pub fn plain_text(&self) -> &str {
        match self {
            MarkdownToken::Text(s) => s,
            MarkdownToken::Bold(s) => s,
            MarkdownToken::Italic(s) => s,
            MarkdownToken::Strikethrough(s) => s,
            MarkdownToken::InlineCode(s) => s,
            MarkdownToken::Spoiler(s) => s,
            MarkdownToken::Blockquote(s) => s,
            MarkdownToken::Heading(s) => s,
            MarkdownToken::Link { title, .. } => title,
            MarkdownToken::CodeBlock { code, .. } => code,
        }
    }
}

/// State for click-to-reveal spoiler elements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpoilerState {
    pub content: String,
    pub is_revealed: bool,
}

impl SpoilerState {
    /// Creates a new concealed spoiler state.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_revealed: false,
        }
    }

    /// Reveals the hidden spoiler content.
    pub fn reveal(&mut self) {
        self.is_revealed = true;
    }

    /// Toggles the revealed state between hidden and visible.
    pub fn toggle(&mut self) {
        self.is_revealed = !self.is_revealed;
    }

    /// Hides the spoiler content again.
    pub fn hide(&mut self) {
        self.is_revealed = false;
    }
}

/// Parses inline markdown elements within a single line or segment.
fn parse_inline_tokens(input: &str) -> Vec<MarkdownToken> {
    let mut tokens = Vec::new();
    let mut remaining = input;

    while !remaining.is_empty() {
        // Look for the earliest occurrence of any inline marker:
        // **, ||, ~~, `, [, *
        let mut earliest: Option<(usize, &str)> = None;

        let find_marker = |sub: &str, m: &str| -> Option<usize> {
            sub.find(m)
        };

        let candidates = ["**", "||", "~~", "`", "[", "*"];
        for marker in candidates {
            if let Some(pos) = find_marker(remaining, marker) {
                // For single '*', ensure it is not part of '**'
                if marker == "*" {
                    let is_double = (pos > 0 && remaining.as_bytes().get(pos - 1) == Some(&b'*'))
                        || remaining.as_bytes().get(pos + 1) == Some(&b'*');
                    if is_double {
                        continue;
                    }
                }
                match earliest {
                    Some((min_pos, _)) if pos < min_pos => {
                        earliest = Some((pos, marker));
                    }
                    None => {
                        earliest = Some((pos, marker));
                    }
                    _ => {}
                }
            }
        }

        match earliest {
            Some((pos, marker)) => {
                // If there's text before the marker, emit it as Text
                if pos > 0 {
                    tokens.push(MarkdownToken::Text(remaining[..pos].to_string()));
                }

                let after_marker = &remaining[pos + marker.len()..];

                match marker {
                    "**" => {
                        if let Some(end_idx) = after_marker.find("**") {
                            let content = &after_marker[..end_idx];
                            tokens.push(MarkdownToken::Bold(content.to_string()));
                            remaining = &after_marker[end_idx + 2..];
                        } else {
                            tokens.push(MarkdownToken::Text(marker.to_string()));
                            remaining = after_marker;
                        }
                    }
                    "||" => {
                        if let Some(end_idx) = after_marker.find("||") {
                            let content = &after_marker[..end_idx];
                            tokens.push(MarkdownToken::Spoiler(content.to_string()));
                            remaining = &after_marker[end_idx + 2..];
                        } else {
                            tokens.push(MarkdownToken::Text(marker.to_string()));
                            remaining = after_marker;
                        }
                    }
                    "~~" => {
                        if let Some(end_idx) = after_marker.find("~~") {
                            let content = &after_marker[..end_idx];
                            tokens.push(MarkdownToken::Strikethrough(content.to_string()));
                            remaining = &after_marker[end_idx + 2..];
                        } else {
                            tokens.push(MarkdownToken::Text(marker.to_string()));
                            remaining = after_marker;
                        }
                    }
                    "`" => {
                        if let Some(end_idx) = after_marker.find('`') {
                            let content = &after_marker[..end_idx];
                            tokens.push(MarkdownToken::InlineCode(content.to_string()));
                            remaining = &after_marker[end_idx + 1..];
                        } else {
                            tokens.push(MarkdownToken::Text(marker.to_string()));
                            remaining = after_marker;
                        }
                    }
                    "*" => {
                        if let Some(end_idx) = after_marker.find('*') {
                            let content = &after_marker[..end_idx];
                            tokens.push(MarkdownToken::Italic(content.to_string()));
                            remaining = &after_marker[end_idx + 1..];
                        } else {
                            tokens.push(MarkdownToken::Text(marker.to_string()));
                            remaining = after_marker;
                        }
                    }
                    "[" => {
                        if let Some(close_bracket) = after_marker.find("](") {
                            let title = &after_marker[..close_bracket];
                            let url_part = &after_marker[close_bracket + 2..];
                            if let Some(close_paren) = url_part.find(')') {
                                let url = &url_part[..close_paren];
                                tokens.push(MarkdownToken::Link {
                                    title: title.to_string(),
                                    url: url.to_string(),
                                });
                                remaining = &url_part[close_paren + 1..];
                            } else {
                                tokens.push(MarkdownToken::Text(marker.to_string()));
                                remaining = after_marker;
                            }
                        } else {
                            tokens.push(MarkdownToken::Text(marker.to_string()));
                            remaining = after_marker;
                        }
                    }
                    _ => {
                        tokens.push(MarkdownToken::Text(marker.to_string()));
                        remaining = after_marker;
                    }
                }
            }
            None => {
                tokens.push(MarkdownToken::Text(remaining.to_string()));
                break;
            }
        }
    }

    tokens
}

/// Parses an entire markdown string into structured `MarkdownToken` tokens.
/// Supports multi-line code blocks, headings, blockquotes, and inline styling.
pub fn parse_markdown_tokens(input: &str) -> Vec<MarkdownToken> {
    let mut tokens = Vec::new();
    let lines: Vec<&str> = input.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // 1. Check for fenced code block: starts with ```
        if trimmed.starts_with("```") {
            let lang_str = trimmed.trim_start_matches("```").trim();
            let language = if lang_str.is_empty() {
                None
            } else {
                Some(lang_str.to_string())
            };

            let mut code_lines = Vec::new();
            i += 1;
            while i < lines.len() {
                let code_line = lines[i];
                if code_line.trim().starts_with("```") {
                    break;
                }
                code_lines.push(code_line);
                i += 1;
            }

            tokens.push(MarkdownToken::CodeBlock {
                language,
                code: code_lines.join("\n"),
            });
            i += 1;
            continue;
        }

        // 2. Section Heading (# Heading)
        if trimmed.starts_with('#') {
            let heading_text = trimmed.trim_start_matches('#').trim().to_string();
            if !heading_text.is_empty() {
                tokens.push(MarkdownToken::Heading(heading_text));
                i += 1;
                continue;
            }
        }

        // 3. Blockquote (> Quote)
        if trimmed.starts_with('>') {
            let quote_text = trimmed.trim_start_matches('>').trim().to_string();
            tokens.push(MarkdownToken::Blockquote(quote_text));
            i += 1;
            continue;
        }

        // 4. Regular line: parse inline formatting
        let inline_tokens = parse_inline_tokens(line);
        tokens.extend(inline_tokens);

        i += 1;
    }

    tokens
}

/// Tokenizes source code lines into highlighted syntax tokens.
fn highlight_code_line(line: &str) -> Vec<(String, Hsla)> {
    let mut segments = Vec::new();
    let words = line.split_inclusive(|c: char| !c.is_alphanumeric() && c != '_');

    let rust_keywords = [
        "fn", "let", "mut", "pub", "struct", "impl", "enum", "const", "static", "if", "else",
        "match", "for", "while", "loop", "return", "break", "continue", "async", "await",
        "trait", "type", "where", "use", "mod", "as", "crate", "super", "self", "Self",
        "function", "class", "import", "export", "from", "default", "var", "val", "def",
    ];

    let rust_types = [
        "String", "str", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32",
        "i64", "i128", "isize", "f32", "f64", "bool", "char", "Option", "Result", "Vec",
        "HashMap", "Some", "None", "Ok", "Err",
    ];

    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        segments.push((line.to_string(), theme::text_muted()));
        return segments;
    }

    for word in words {
        let clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
        if rust_keywords.contains(&clean) {
            segments.push((word.to_string(), theme::accent()));
        } else if rust_types.contains(&clean) {
            segments.push((word.to_string(), theme::status_streaming()));
        } else if word.starts_with('"') || word.ends_with('"') {
            segments.push((word.to_string(), theme::status_online()));
        } else if clean.chars().all(|c| c.is_ascii_digit()) && !clean.is_empty() {
            segments.push((word.to_string(), theme::status_idle()));
        } else {
            segments.push((word.to_string(), theme::text_primary()));
        }
    }

    if segments.is_empty() {
        segments.push((line.to_string(), theme::text_primary()));
    }

    segments
}

/// Renders a syntax-highlighted code block with language badge and one-click copy button.
pub fn render_code_block(language: Option<&str>, code: &str) -> AnyElement {
    let lang_display = language.unwrap_or("text").to_uppercase();
    let lines: Vec<&str> = code.lines().collect();

    div()
        .flex()
        .flex_col()
        .w_full()
        .my_2()
        .rounded_lg()
        .bg(theme::bg_surface())
        .border_1()
        .border_color(theme::border_subtle())
        .overflow_hidden()
        // Header Bar: Language Badge & Copy Button
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .h(px(32.0))
                .px_3()
                .bg(theme::bg_subsurface())
                .border_b_1()
                .border_color(theme::border_hairline())
                // Language Badge
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_sm()
                        .bg(theme::bg_surface_4())
                        .text_xs()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme::text_secondary())
                        .child(lang_display),
                )
                // One-click Copy Button
                .child(
                    div()
                        .cursor_pointer()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1()
                        .px_2()
                        .py_0p5()
                        .rounded_sm()
                        .bg(theme::bg_elevated())
                        .border_1()
                        .border_color(theme::border_hairline())
                        .hover(|s| s.bg(theme::bg_surface_4()))
                        .text_xs()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(theme::text_primary())
                        .child("Copy"),
                ),
        )
        // Code Block Body with Monospace Font & Syntax Coloring
        .child(
            div()
                .flex()
                .flex_col()
                .p_3()
                .gap_0p5()
                .overflow_hidden()
                .bg(theme::bg_surface())
                .children(lines.into_iter().enumerate().map(|(idx, line)| {
                    let segments = highlight_code_line(line);
                    div()
                        .flex()
                        .flex_row()
                        .items_baseline()
                        .gap_3()
                        // Line number
                        .child(
                            div()
                                .w(px(24.0))
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child(format!("{}", idx + 1)),
                        )
                        // Code line segments
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .flex_wrap()
                                .items_baseline()
                                .text_xs()
                                .children(segments.into_iter().map(|(text, color)| {
                                    div().text_color(color).child(text)
                                })),
                        )
                })),
        )
        .into_any_element()
}

/// Renders a single markdown token or group of tokens into a styled GPUI `AnyElement`.
pub fn render_markdown_element(text: &str) -> AnyElement {
    let tokens = parse_markdown_tokens(text);

    div()
        .flex()
        .flex_col()
        .gap_1p5()
        .w_full()
        .children(tokens.into_iter().map(|token| match token {
            MarkdownToken::Heading(h) => div()
                .text_base()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(theme::text_primary())
                .child(h)
                .into_any_element(),
            MarkdownToken::Blockquote(quote) => div()
                .flex()
                .flex_row()
                .border_l_2()
                .border_color(theme::accent())
                .pl_3()
                .py_1()
                .bg(theme::bg_surface())
                .rounded_r_md()
                .text_sm()
                .italic()
                .text_color(theme::text_secondary())
                .child(quote)
                .into_any_element(),
            MarkdownToken::CodeBlock { language, code } => {
                render_code_block(language.as_deref(), &code)
            }
            MarkdownToken::Bold(b) => div()
                .text_sm()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(theme::text_primary())
                .child(b)
                .into_any_element(),
            MarkdownToken::Italic(i) => div()
                .text_sm()
                .italic()
                .text_color(theme::text_primary())
                .child(i)
                .into_any_element(),
            MarkdownToken::Strikethrough(s) => div()
                .text_sm()
                .line_through()
                .text_color(theme::text_muted())
                .child(s)
                .into_any_element(),
            MarkdownToken::InlineCode(c) => div()
                .px_1p5()
                .py_0p5()
                .rounded_sm()
                .bg(theme::bg_surface_4())
                .border_1()
                .border_color(theme::border_hairline())
                .text_xs()
                .text_color(theme::accent())
                .child(c)
                .into_any_element(),
            MarkdownToken::Spoiler(s) => div()
                .cursor_pointer()
                .px_2()
                .py_0p5()
                .rounded_md()
                .bg(theme::bg_elevated())
                .border_1()
                .border_color(theme::border_subtle())
                .text_xs()
                .text_color(theme::text_muted())
                .hover(|st| st.text_color(theme::text_primary()))
                .child(format!("|| {} ||", s))
                .into_any_element(),
            MarkdownToken::Link { title, url } => div()
                .cursor_pointer()
                .text_sm()
                .text_color(theme::accent())
                .underline()
                .hover(|st| st.text_color(theme::accent_hover()))
                .child(format!("{title} ({url})"))
                .into_any_element(),
            MarkdownToken::Text(t) => div()
                .text_sm()
                .text_color(theme::text_primary())
                .child(t)
                .into_any_element(),
        }))
        .into_any_element()
}
