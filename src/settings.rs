//! What Grove remembers between runs: whether the sounds are muted. It is
//! kept where Qt's settings kept it, in the same form, so a choice made
//! before the port still holds:
//!
//! ```ini
//! [General]
//! muted=true
//! ```

use std::fs;
use std::path::Path;

/// Whether the settings file says the sounds are muted. No file, or one
/// that does not say, means they are not.
pub fn muted(file: &Path) -> bool {
    let Ok(text) = fs::read_to_string(file) else {
        return false;
    };
    let mut general = true;
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            general = line == "[General]";
        } else if general {
            if let Some(value) = line.strip_prefix("muted=") {
                return value.trim() == "true";
            }
        }
    }
    false
}

/// Writes whether the sounds are muted, keeping anything else in the file.
/// A choice that cannot be written is only for this run; that is not worth
/// troubling the user over.
pub fn remember_muted(file: &Path, muted: bool) {
    let text = fs::read_to_string(file).unwrap_or_default();
    let entry = format!("muted={muted}");
    let mut lines: Vec<String> = Vec::new();
    let (mut general, mut written) = (true, false);
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            general = trimmed == "[General]";
        } else if general && trimmed.starts_with("muted=") {
            if !written {
                lines.push(entry.clone());
                written = true;
            }
            continue;
        }
        lines.push(line.to_string());
    }
    if !written {
        // Under [General], which is made if it is not there.
        let at = match lines.iter().position(|line| line.trim() == "[General]") {
            Some(heading) => heading + 1,
            None => {
                lines.insert(0, "[General]".to_string());
                1
            }
        };
        lines.insert(at, entry);
    }
    if let Some(folder) = file.parent() {
        let _ = fs::create_dir_all(folder);
    }
    let _ = fs::write(file, lines.join("\n") + "\n");
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn sounds_mute_is_remembered() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("grove/grove.conf");
        assert!(!muted(&file));
        remember_muted(&file, true);
        assert!(muted(&file));
        assert_eq!(fs::read_to_string(&file).unwrap(), "[General]\nmuted=true\n");
        remember_muted(&file, false);
        assert!(!muted(&file));
        assert_eq!(fs::read_to_string(&file).unwrap(), "[General]\nmuted=false\n");
    }

    #[test]
    fn what_qt_wrote_is_read_and_the_rest_is_kept() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("grove.conf");
        fs::write(&file, "[General]\nmuted=true\nother=1\n\n[Window]\nmuted=false\n").unwrap();
        assert!(muted(&file));
        remember_muted(&file, false);
        assert_eq!(
            fs::read_to_string(&file).unwrap(),
            "[General]\nmuted=false\nother=1\n\n[Window]\nmuted=false\n"
        );
        assert!(!muted(&file));
    }

    #[test]
    fn a_file_without_the_choice_gains_it() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("grove.conf");
        fs::write(&file, "[Window]\nwidth=3\n").unwrap();
        assert!(!muted(&file));
        remember_muted(&file, true);
        assert_eq!(
            fs::read_to_string(&file).unwrap(),
            "[General]\nmuted=true\n[Window]\nwidth=3\n"
        );
        assert!(muted(&file));
    }
}
