//! The desktop's look: the accent colour from the current Omarchy theme, and
//! the corner radius Hyprland gives windows. This file reads them; `follow.rs`
//! watches for them changing.

use std::fs;
use std::path::{Path, PathBuf};

use crate::color::Color;

/// Omarchy's default yellow, for when no theme gives an accent.
pub const DEFAULT_ACCENT: &str = "#FFD60A";

/// The desktop's look as last read.
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    /// Omarchy's current-theme directory; `None` when there is none to follow.
    directory: Option<PathBuf>,
    accent: String,
    radius: i32,
}

impl Theme {
    /// The theme in `directory`, read now. Corners stay sharp, like Omarchy's
    /// default, until Hyprland says otherwise.
    pub fn read(directory: Option<PathBuf>) -> Self {
        let mut theme = Self {
            directory,
            accent: DEFAULT_ACCENT.to_string(),
            radius: 0,
        };
        theme.reload();
        theme
    }

    /// The theme's accent as #rrggbb.
    pub fn accent(&self) -> &str {
        &self.accent
    }

    /// The accent as a colour.
    pub fn accent_color(&self) -> Color {
        Color::from_hex(&self.accent).unwrap_or_else(|| Color::hex(DEFAULT_ACCENT))
    }

    /// Black or white, whichever reads on the accent (WCAG relative luminance).
    pub fn accent_foreground(&self) -> &'static str {
        let linear = |v: f64| {
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        let color = self.accent_color();
        let luminance = 0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b);
        if luminance > 0.179 {
            "black"
        } else {
            "white"
        }
    }

    /// The corner radius, in pixels, as Hyprland rounds windows.
    pub fn radius(&self) -> i32 {
        self.radius
    }

    /// Reads the accent again. Returns whether it changed.
    pub fn reload(&mut self) -> bool {
        let accent = self
            .directory
            .as_deref()
            .and_then(|directory| fs::read(colors_file(directory)).ok())
            // Only the accent's line need be text.
            .and_then(|colors| accent_in(&String::from_utf8_lossy(&colors)))
            .unwrap_or_else(|| DEFAULT_ACCENT.to_string());
        let changed = accent != self.accent;
        self.accent = accent;
        changed
    }

    /// Takes Hyprland's rounding. Returns whether it changed.
    pub fn set_radius(&mut self, radius: i32) -> bool {
        let changed = radius != self.radius;
        self.radius = radius;
        changed
    }
}

/// Where a theme directory keeps its colours.
fn colors_file(directory: &Path) -> PathBuf {
    directory.join("theme/colors.toml")
}

/// The accent in a theme's colors.toml: the first line of the form
/// `accent = "#rrggbb"`, in either kind of quote.
pub fn accent_in(colors: &str) -> Option<String> {
    let quote = |text: &str| text.starts_with(['"', '\'']);
    colors.lines().find_map(|line| {
        let value = line
            .trim_start()
            .strip_prefix("accent")?
            .trim_start()
            .strip_prefix('=')?
            .trim_start();
        let color = value.get(1..8)?;
        let hex = color.strip_prefix('#')?;
        (quote(value) && quote(value.get(8..)?) && hex.chars().all(|digit| digit.is_ascii_hexdigit()))
            .then(|| color.to_string())
    })
}

/// The rounding in what `hyprctl -j getoption decoration:rounding` printed.
pub fn rounding_in(output: &[u8]) -> Option<i32> {
    let value: serde_json::Value = serde_json::from_slice(output).ok()?;
    // A whole number, however it is written: 8 or 8.0.
    let rounding = value.get("int")?.as_f64()?;
    (rounding.fract() == 0.0 && (0.0..=f64::from(i32::MAX)).contains(&rounding)).then_some(rounding as i32)
}

/// What to watch to hear of a change to the theme in `directory`: of the
/// directory, its theme and its colours, those that exist, and the nearest
/// ancestor that does with its parent. A theme switch replaces symlinks and
/// files, so the parents are watched too; watching the nearest existing
/// ancestor means installing Omarchy later works without restarting the app.
pub fn paths_to_watch(directory: &Path) -> Vec<PathBuf> {
    let mut ancestor = directory;
    while !ancestor.exists() {
        match ancestor.parent() {
            Some(parent) => ancestor = parent,
            None => break,
        }
    }
    let mut paths: Vec<PathBuf> = Vec::new();
    let candidates = [
        Some(ancestor.to_path_buf()),
        ancestor.parent().map(Path::to_path_buf),
        Some(directory.to_path_buf()),
        Some(directory.join("theme")),
        Some(colors_file(directory)),
    ];
    for path in candidates.into_iter().flatten() {
        if path.exists() && !paths.contains(&path) {
            paths.push(path);
        }
    }
    paths
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    fn with_colors(colors: &str) -> (TempDir, Theme) {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("theme")).unwrap();
        fs::write(colors_file(dir.path()), colors).unwrap();
        let theme = Theme::read(Some(dir.path().to_path_buf()));
        (dir, theme)
    }

    #[test]
    fn theme_falls_back() {
        let dir = TempDir::new().unwrap();
        let theme = Theme::read(Some(dir.path().to_path_buf()));
        assert_eq!(theme.accent(), "#FFD60A");
        assert_eq!(theme.accent_foreground(), "black");
        assert_eq!(Theme::read(None).accent(), "#FFD60A");
    }

    #[test]
    fn theme_reads_accent() {
        let (_dir, theme) = with_colors("accent = \"#112233\"\n");
        assert_eq!(theme.accent(), "#112233");
        assert_eq!(theme.accent_foreground(), "white");
    }

    #[test]
    fn the_accent_is_read_from_a_file_that_is_not_all_text() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("theme")).unwrap();
        fs::write(colors_file(dir.path()), b"name = \"\xff\xfe\"\naccent = \"#112233\"\n").unwrap();
        assert_eq!(Theme::read(Some(dir.path().to_path_buf())).accent(), "#112233");
    }

    #[test]
    fn the_accent_is_found_among_other_colours() {
        assert_eq!(
            accent_in("background = \"#000000\"\n  accent='#AbCdEf' # mine\n").as_deref(),
            Some("#AbCdEf")
        );
        assert_eq!(accent_in("accent_dim = \"#112233\"\n"), None);
        assert_eq!(accent_in("accent = \"#11223\"\n"), None);
        assert_eq!(accent_in("accent = \"#11223g\"\n"), None);
        assert_eq!(accent_in("accent = #112233\n"), None);
        assert_eq!(accent_in("# accent = \"#112233\"\n"), None);
        assert_eq!(accent_in("accent = \"#é12233\"\n"), None);
    }

    #[test]
    fn a_changed_accent_is_noticed_on_reload() {
        let (dir, mut theme) = with_colors("accent = \"#112233\"\n");
        assert!(!theme.reload());
        fs::write(colors_file(dir.path()), "accent = \"#445566\"\n").unwrap();
        assert!(theme.reload());
        assert_eq!(theme.accent(), "#445566");
        fs::remove_file(colors_file(dir.path())).unwrap();
        assert!(theme.reload());
        assert_eq!(theme.accent(), DEFAULT_ACCENT);
    }

    #[test]
    fn rounding_is_read_from_hyprctl() {
        assert_eq!(
            rounding_in(br#"{"option": "decoration:rounding", "int": 8, "set": true}"#),
            Some(8)
        );
        assert_eq!(rounding_in(br#"{"int": 0}"#), Some(0));
        assert_eq!(rounding_in(br#"{"int": 8.0}"#), Some(8));
        assert_eq!(rounding_in(br#"{"int": 8.5}"#), None);
        assert_eq!(rounding_in(br#"{"int": -1}"#), None);
        assert_eq!(rounding_in(br#"{"float": 8.0}"#), None);
        assert_eq!(rounding_in(b"no such option"), None);
    }

    #[test]
    fn the_radius_changes_only_when_it_differs() {
        let mut theme = Theme::read(None);
        assert!(!theme.set_radius(0));
        assert!(theme.set_radius(8));
        assert_eq!(theme.radius(), 8);
    }

    #[test]
    fn a_missing_theme_is_watched_for_from_what_exists() {
        let dir = TempDir::new().unwrap();
        let current = dir.path().join("state/omarchy/current");
        // Nothing of Omarchy's yet: the nearest folder that exists, and its parent.
        let paths = paths_to_watch(&current);
        assert_eq!(
            paths,
            [dir.path().to_path_buf(), dir.path().parent().unwrap().to_path_buf()]
        );

        fs::create_dir_all(current.join("theme")).unwrap();
        fs::write(colors_file(&current), "").unwrap();
        let paths = paths_to_watch(&current);
        assert_eq!(paths.len(), 4);
        assert!(paths.contains(&colors_file(&current)));
        assert!(paths.contains(&current.parent().unwrap().to_path_buf()));
    }
}
