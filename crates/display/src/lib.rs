//! Configurable colors for branch name, commit hash, and subject in the picker.

use std::collections::HashSet;
use std::path::PathBuf;

use parse::Branch;
use ratatui::style::{Color, Style};
pub use ratatui::text::Line;
use ratatui::text::Span;
use serde::Deserialize;

/// Column widths for aligned branch listing in the picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayLayout {
    pub name_width: usize,
    pub hash_width: usize,
}

impl DisplayLayout {
    /// Compute column widths from the longest name and hash in the list.
    pub fn from_branches(branches: &[Branch]) -> Self {
        let name_width = branches
            .iter()
            .map(|b| b.name.chars().count())
            .max()
            .unwrap_or(0);
        let hash_width = branches
            .iter()
            .map(|b| b.short_hash.chars().count())
            .max()
            .unwrap_or(0);
        Self {
            name_width,
            hash_width,
        }
    }

    /// Format `name`, `hash`, and `subject` on aligned columns (space-padded).
    pub fn format_line(&self, branch: &Branch) -> String {
        format!(
            "{:<name_w$} {:<hash_w$} {}",
            branch.name,
            branch.short_hash,
            branch.subject,
            name_w = self.name_width,
            hash_w = self.hash_width,
        )
    }

    /// Char-index boundaries for the three fields in [`Self::format_line`] output.
    pub fn field_ranges(self) -> FieldRanges {
        FieldRanges {
            name_end: self.name_width,
            hash_end: self.name_width + 1 + self.hash_width,
        }
    }

    /// Byte ranges in `display` for skim name vs subject matching.
    pub fn matching_ranges(self, display: &str) -> [(usize, usize); 2] {
        let name_end = char_index_to_byte(display, self.name_width);
        let subject_start = char_index_to_byte(display, self.name_width + 1 + self.hash_width + 1);
        [(0, name_end), (subject_start, display.len())]
    }
}

/// Byte offset at a character index (for skim match ranges).
pub fn char_index_to_byte(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

/// Colors for the three fields shown in the branch picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayColors {
    pub enabled: bool,
    pub name: Style,
    pub hash: Style,
    pub subject: Style,
}

impl Default for DisplayColors {
    fn default() -> Self {
        Self {
            enabled: true,
            name: Style::new().fg(Color::Cyan),
            hash: Style::new().fg(Color::Yellow),
            subject: Style::new().fg(Color::White),
        }
    }
}

impl DisplayColors {
    pub const fn disabled() -> Self {
        Self {
            enabled: false,
            name: Style::new(),
            hash: Style::new(),
            subject: Style::new(),
        }
    }
}

/// Overrides from CLI (None = leave config file value).
#[derive(Debug, Default, Clone)]
pub struct ColorOverrides {
    pub disabled: Option<bool>,
    pub name: Option<String>,
    pub hash: Option<String>,
    pub subject: Option<String>,
    /// `NAME:HASH:SUBJECT` shorthand
    pub triple: Option<String>,
}

/// Load colors from `~/.git-b/config.toml`, then apply CLI overrides.
pub fn load(overrides: &ColorOverrides) -> DisplayColors {
    let mut colors = load_config_file().unwrap_or_default();
    apply_overrides(&mut colors, overrides);
    colors
}

/// Path to the config file: `~/.git-b/config.toml`.
pub fn config_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".git-b").join("config.toml"))
}

#[derive(Debug, Default, Deserialize)]
struct ConfigFile {
    #[serde(default)]
    colors: ColorsSection,
}

#[derive(Debug, Deserialize)]
struct ColorsSection {
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(default = "default_name_color")]
    name: String,
    #[serde(default = "default_hash_color")]
    hash: String,
    #[serde(default = "default_subject_color")]
    subject: String,
}

impl Default for ColorsSection {
    fn default() -> Self {
        Self {
            enabled: true,
            name: default_name_color(),
            hash: default_hash_color(),
            subject: default_subject_color(),
        }
    }
}

fn default_true() -> bool {
    true
}
fn default_name_color() -> String {
    "cyan".into()
}
fn default_hash_color() -> String {
    "yellow".into()
}
fn default_subject_color() -> String {
    "white".into()
}

fn load_config_file() -> Option<DisplayColors> {
    let path = config_path()?;
    let contents = std::fs::read_to_string(&path).ok()?;
    parse_config(&contents).ok()
}

fn parse_config(contents: &str) -> Result<DisplayColors, String> {
    let file: ConfigFile = toml::from_str(contents).map_err(|e| e.to_string())?;
    colors_from_section(file.colors)
}

fn colors_from_section(section: ColorsSection) -> Result<DisplayColors, String> {
    if !section.enabled {
        return Ok(DisplayColors::disabled());
    }
    Ok(DisplayColors {
        enabled: true,
        name: style_for_field(parse_color(&section.name)?),
        hash: style_for_field(parse_color(&section.hash)?),
        subject: style_for_field(parse_color(&section.subject)?),
    })
}

fn apply_overrides(colors: &mut DisplayColors, o: &ColorOverrides) {
    if let Some(true) = o.disabled {
        colors.enabled = false;
        return;
    }
    if let Some(ref t) = o.triple {
        apply_triple(colors, t);
    }
    if let Some(ref v) = o.name {
        if let Ok(c) = parse_color(v) {
            colors.name = style_for_field(c);
        }
    }
    if let Some(ref v) = o.hash {
        if let Ok(c) = parse_color(v) {
            colors.hash = style_for_field(c);
        }
    }
    if let Some(ref v) = o.subject {
        if let Ok(c) = parse_color(v) {
            colors.subject = style_for_field(c);
        }
    }
}

fn apply_triple(colors: &mut DisplayColors, spec: &str) {
    colors.enabled = true;
    let mut parts = spec.split(':');
    if let Some(p) = parts.next() {
        if let Ok(c) = parse_color(p.trim()) {
            colors.name = style_for_field(c);
        }
    }
    if let Some(p) = parts.next() {
        if let Ok(c) = parse_color(p.trim()) {
            colors.hash = style_for_field(c);
        }
    }
    if let Some(p) = parts.next() {
        if let Ok(c) = parse_color(p.trim()) {
            colors.subject = style_for_field(c);
        }
    }
}

/// Parse a color name (`cyan`, `bright-green`, `reset`, etc.).
pub fn parse_color(s: &str) -> Result<Color, String> {
    let s = s.trim();
    if s.is_empty() || s.eq_ignore_ascii_case("default") || s.eq_ignore_ascii_case("none") {
        return Ok(Color::Reset);
    }

    let (bright, name) = if let Some(rest) = s.strip_prefix("bright-") {
        (true, rest)
    } else {
        (false, s)
    };

    let base = match name.to_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "gray" | "grey" => Color::Gray,
        "reset" => Color::Reset,
        _ => return Err(format!("unknown color: {s}")),
    };

    Ok(if bright { brighten(base) } else { base })
}

fn brighten(color: Color) -> Color {
    match color {
        Color::Red => Color::LightRed,
        Color::Green => Color::LightGreen,
        Color::Yellow => Color::LightYellow,
        Color::Blue => Color::LightBlue,
        Color::Magenta => Color::LightMagenta,
        Color::Cyan => Color::LightCyan,
        Color::Black | Color::White | Color::Gray => Color::Gray,
        other => other,
    }
}

fn style_for_field(color: Color) -> Style {
    if color == Color::Reset {
        Style::default()
    } else {
        Style::new().fg(color)
    }
}

/// Char-index boundaries for `name hash subject` layout.
#[derive(Debug, Clone, Copy)]
pub struct FieldRanges {
    pub name_end: usize,
    pub hash_end: usize,
}

fn style_at(index: usize, ranges: FieldRanges, colors: DisplayColors) -> Style {
    if index < ranges.name_end {
        colors.name
    } else if index < ranges.hash_end {
        colors.hash
    } else {
        colors.subject
    }
}

/// Build a styled line with per-field colors and optional query-match highlighting (char indices).
pub fn colored_line(
    text: &str,
    ranges: FieldRanges,
    colors: DisplayColors,
    base_style: Style,
    matched_style: Style,
    highlight: &HashSet<usize>,
) -> Line<'static> {
    if !colors.enabled {
        return Line::from(text.to_string());
    }

    let mut line = Line::default();
    let mut current = String::new();
    let mut current_style = Style::default();

    let push_span = |line: &mut Line, content: &mut String, style: Style| {
        if !content.is_empty() {
            line.push_span(Span::styled(std::mem::take(content), style));
        }
    };

    for (ci, ch) in text.chars().enumerate() {
        let mut field_style = style_at(ci, ranges, colors).patch(base_style);
        if highlight.contains(&ci) {
            field_style = field_style.patch(matched_style);
        }

        if field_style == current_style && !current.is_empty() {
            current.push(ch);
        } else {
            push_span(&mut line, &mut current, current_style);
            current_style = field_style;
            current.push(ch);
        }
    }
    push_span(&mut line, &mut current, current_style);

    if line.spans.is_empty() {
        line.push_span(Span::raw(text.to_string()));
    }

    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_named_colors() {
        assert_eq!(parse_color("cyan").unwrap(), Color::Cyan);
        assert_eq!(parse_color("bright-red").unwrap(), Color::LightRed);
    }

    #[test]
    fn parse_config_toml() {
        let colors = parse_config(
            r#"
[colors]
enabled = true
name = "green"
hash = "blue"
subject = "magenta"
"#,
        )
        .unwrap();
        assert!(colors.enabled);
        assert_eq!(colors.name.fg, Some(Color::Green));
        assert_eq!(colors.hash.fg, Some(Color::Blue));
        assert_eq!(colors.subject.fg, Some(Color::Magenta));
    }

    #[test]
    fn parse_config_disabled() {
        let colors = parse_config("[colors]\nenabled = false\n").unwrap();
        assert!(!colors.enabled);
    }

    #[test]
    fn aligned_columns_pad_shorter_names() {
        let branches = [
            Branch {
                name: "main".into(),
                short_hash: "abc1234".into(),
                subject: "init".into(),
            },
            Branch {
                name: "feature/long-name".into(),
                short_hash: "abc1234".into(),
                subject: "wip".into(),
            },
        ];
        let layout = DisplayLayout::from_branches(&branches);
        assert_eq!(layout.name_width, "feature/long-name".chars().count());

        let short = layout.format_line(&branches[0]);
        let long = layout.format_line(&branches[1]);
        let hash_col = layout.name_width + 1;
        assert_eq!(
            short.chars().skip(hash_col).take(7).collect::<String>(),
            "abc1234"
        );
        assert_eq!(
            long.chars().skip(hash_col).take(7).collect::<String>(),
            "abc1234"
        );

        let subject_col = layout.name_width + 1 + layout.hash_width + 1;
        assert_eq!(short.chars().skip(subject_col).collect::<String>(), "init");
        assert_eq!(long.chars().skip(subject_col).collect::<String>(), "wip");
    }

    #[test]
    fn cli_no_color_overrides_file() {
        let mut colors = parse_config("[colors]\nname = \"red\"\n").unwrap();
        apply_overrides(
            &mut colors,
            &ColorOverrides {
                disabled: Some(true),
                ..Default::default()
            },
        );
        assert!(!colors.enabled);
    }
}
