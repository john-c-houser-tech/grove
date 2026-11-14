//! How Grove is set up: from the environment for the real app, and by hand
//! for tests, which keep everything of the desktop's out of reach.

use std::path::PathBuf;

use chrono::{NaiveDate, NaiveTime};

use crate::follow::Hyprland;
use crate::language::Catalogs;
use crate::notifier;
use crate::sounds;
use crate::weather::Kind;

/// The settings everything is made from.
#[derive(Clone, Debug, PartialEq)]
pub struct Options {
    /// How long a session lasts, in seconds.
    pub seconds: i32,
    /// A fixed time of day and date for the sky, when given.
    pub time: Option<NaiveTime>,
    pub date: Option<NaiveDate>,
    /// A pinned weather.
    pub weather: Option<Kind>,
    /// Where the garden is kept; `None` for the user's data directory. A
    /// shortened session is only saved when a garden is given.
    pub garden_path: Option<PathBuf>,
    /// Where the desktop keeps applications' data, for the garden when no
    /// path is given; `None` keeps no garden at all.
    pub data_directory: Option<PathBuf>,
    /// Omarchy's current-theme directory; `None` to follow no theme.
    pub theme_directory: Option<PathBuf>,
    /// How to ask Hyprland for its rounding; `None` to leave corners sharp.
    pub hyprland: Option<Hyprland>,
    /// Where Omarchy keeps the language code; `None` for English.
    pub language_file: Option<PathBuf>,
    /// Where the catalogs are read from; tests give a directory of their own.
    pub catalogs: Catalogs,
    /// Where Omarchy is installed, for its registry of locales.
    pub omarchy_path: Option<PathBuf>,
    /// The desktop notification service; only tests name another.
    pub notification_service: String,
    /// A sound to hear a few seconds into every session, whatever the hour.
    pub sound_test: Option<sounds::Kind>,
    /// Scales the volume of the sounds, from 0 to 1, so the sound path can
    /// be exercised without a sound.
    pub sound_volume: f64,
    /// Where muting is remembered between runs; `None` to forget it.
    pub settings_file: Option<PathBuf>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            seconds: 25 * 60,
            time: None,
            date: None,
            weather: None,
            garden_path: None,
            data_directory: None,
            theme_directory: None,
            hyprland: None,
            language_file: None,
            catalogs: Catalogs::BuiltIn,
            omarchy_path: None,
            notification_service: notifier::SERVICE.to_string(),
            sound_test: None,
            sound_volume: 1.0,
            settings_file: None,
        }
    }
}

impl Options {
    /// Options from the environment: GROVE_SECONDS, GROVE_TIME (HH:MM),
    /// GROVE_DATE (YYYY-MM-DD), GROVE_WEATHER, GROVE_GARDEN, GROVE_SOUND_TEST
    /// and GROVE_SOUND_VOLUME, and where the
    /// desktop keeps its theme and Hyprland its sockets. Nothing else reads the
    /// environment.
    pub fn from_environment() -> Self {
        Self::from_variables(|name| std::env::var(name).ok())
    }

    /// Options from named variables, as the environment gives them.
    pub fn from_variables(variable: impl Fn(&str) -> Option<String>) -> Self {
        let mut options = Self::default();
        // GROVE_SECONDS shortens a session so the growth can be watched quickly.
        if let Some(seconds) = variable("GROVE_SECONDS").and_then(|v| v.parse::<i32>().ok()) {
            if seconds > 0 {
                options.seconds = seconds;
            }
        }
        // GROVE_TIME and GROVE_DATE freeze the sky, and GROVE_WEATHER pins the
        // weather, to see any of them.
        options.time = variable("GROVE_TIME").and_then(|v| NaiveTime::parse_from_str(&v, "%H:%M").ok());
        options.date = variable("GROVE_DATE").and_then(|v| NaiveDate::parse_from_str(&v, "%Y-%m-%d").ok());
        options.weather = variable("GROVE_WEATHER").and_then(|v| Kind::named(&v));
        options.garden_path = variable("GROVE_GARDEN").filter(|v| !v.is_empty()).map(PathBuf::from);
        options.sound_test = variable("GROVE_SOUND_TEST").and_then(|v| sounds::Kind::named(&v));
        if let Some(volume) = variable("GROVE_SOUND_VOLUME").and_then(|v| v.parse::<f64>().ok()) {
            options.sound_volume = volume.clamp(0.0, 1.0);
        }
        // The desktop's own: Omarchy's theme, and Hyprland when it is running.
        let set = |name: &str| variable(name).filter(|v| !v.is_empty());
        options.theme_directory = set("HOME").map(|home| PathBuf::from(home).join(".local/state/omarchy/current"));
        options.data_directory = set("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| set("HOME").map(|home| PathBuf::from(home).join(".local/share")));
        options.language_file = set("HOME").map(|home| PathBuf::from(home).join(".config/omarchy/language"));
        options.omarchy_path = set("OMARCHY_PATH").map(PathBuf::from);
        // Where QSettings kept it, so a mute chosen before the port still holds.
        options.settings_file = set("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| set("HOME").map(|home| PathBuf::from(home).join(".config")))
            .map(|config| config.join("grove/grove.conf"));
        options.hyprland = set("HYPRLAND_INSTANCE_SIGNATURE").map(|signature| Hyprland {
            hyprctl: PathBuf::from("hyprctl"),
            events: set("XDG_RUNTIME_DIR").map(|runtime| {
                PathBuf::from(runtime)
                    .join("hypr")
                    .join(signature)
                    .join(".socket2.sock")
            }),
        });
        options
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn from(pairs: &[(&str, &str)]) -> Options {
        Options::from_variables(|name| {
            pairs
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| value.to_string())
        })
    }

    #[test]
    fn nothing_set_gives_a_full_session_and_a_live_sky() {
        assert_eq!(from(&[]), Options::default());
        assert_eq!(Options::default().seconds, 25 * 60);
    }

    #[test]
    fn the_variables_are_read() {
        let options = from(&[
            ("GROVE_SECONDS", "30"),
            ("GROVE_TIME", "17:05"),
            ("GROVE_DATE", "2026-10-20"),
            ("GROVE_WEATHER", "fog"),
            ("GROVE_GARDEN", "/tmp/garden.json"),
            ("GROVE_SOUND_TEST", "train"),
            ("GROVE_SOUND_VOLUME", "7"),
        ]);
        assert_eq!(options.sound_test, Some(sounds::Kind::Train));
        assert_eq!(options.sound_volume, 1.0);
        assert_eq!(options.garden_path, Some(PathBuf::from("/tmp/garden.json")));
        assert_eq!(options.seconds, 30);
        assert_eq!(options.time, NaiveTime::from_hms_opt(17, 5, 0));
        assert_eq!(options.date, NaiveDate::from_ymd_opt(2026, 10, 20));
        assert_eq!(options.weather, Some(Kind::Fog));
    }

    #[test]
    fn the_desktop_is_found_from_the_environment() {
        let options = from(&[
            ("HOME", "/home/someone"),
            ("HYPRLAND_INSTANCE_SIGNATURE", "abc"),
            ("XDG_RUNTIME_DIR", "/run/user/1000"),
            ("OMARCHY_PATH", "/opt/omarchy"),
        ]);
        assert_eq!(
            options.language_file,
            Some(PathBuf::from("/home/someone/.config/omarchy/language"))
        );
        assert_eq!(options.omarchy_path, Some(PathBuf::from("/opt/omarchy")));
        assert_eq!(
            options.data_directory,
            Some(PathBuf::from("/home/someone/.local/share"))
        );
        assert_eq!(
            options.settings_file,
            Some(PathBuf::from("/home/someone/.config/grove/grove.conf"))
        );
        let elsewhere = from(&[("HOME", "/home/someone"), ("XDG_CONFIG_HOME", "/cfg")]);
        assert_eq!(
            from(&[("XDG_DATA_HOME", "/data")]).data_directory,
            Some(PathBuf::from("/data"))
        );
        assert_eq!(elsewhere.settings_file, Some(PathBuf::from("/cfg/grove/grove.conf")));
        assert_eq!(
            options.theme_directory,
            Some(PathBuf::from("/home/someone/.local/state/omarchy/current"))
        );
        let hyprland = options.hyprland.unwrap();
        assert_eq!(hyprland.hyprctl, PathBuf::from("hyprctl"));
        assert_eq!(
            hyprland.events,
            Some(PathBuf::from("/run/user/1000/hypr/abc/.socket2.sock"))
        );
        // Without Hyprland no one is asked for the rounding.
        assert_eq!(from(&[("HOME", "/home/someone")]).hyprland, None);
    }

    #[test]
    fn nonsense_is_ignored() {
        let options = from(&[
            ("GROVE_SECONDS", "-4"),
            ("GROVE_TIME", "teatime"),
            ("GROVE_DATE", "20/10/2026"),
            ("GROVE_WEATHER", "hail"),
            ("GROVE_GARDEN", ""),
            ("GROVE_SOUND_TEST", "boat"),
            ("GROVE_SOUND_VOLUME", "loud"),
        ]);
        assert_eq!(options, Options::default());
    }
}
