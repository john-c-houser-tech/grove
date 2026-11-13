//! How Grove is set up: from the environment for the real app, and by hand
//! for tests, which keep everything of the desktop's out of reach.

use std::sync::OnceLock;

use chrono::{NaiveDate, NaiveTime};

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
}

impl Default for Options {
    fn default() -> Self {
        Self {
            seconds: 25 * 60,
            time: None,
            date: None,
            weather: None,
        }
    }
}

impl Options {
    /// Options from the environment: GROVE_SECONDS, GROVE_TIME (HH:MM),
    /// GROVE_DATE (YYYY-MM-DD) and GROVE_WEATHER. Nothing else reads the
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
        options
    }
}

static INSTALLED: OnceLock<Options> = OnceLock::new();

/// Fixes the options the app behind the QML singletons is made from. The
/// engine makes the singletons itself, so they cannot be handed their options;
/// the first call wins.
pub fn install(options: Options) {
    let _ = INSTALLED.set(options);
}

/// The installed options, or the defaults if none were.
pub fn installed() -> Options {
    INSTALLED.get().cloned().unwrap_or_default()
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
        ]);
        assert_eq!(options.seconds, 30);
        assert_eq!(options.time, NaiveTime::from_hms_opt(17, 5, 0));
        assert_eq!(options.date, NaiveDate::from_ymd_opt(2026, 10, 20));
        assert_eq!(options.weather, Some(Kind::Fog));
    }

    #[test]
    fn nonsense_is_ignored() {
        let options = from(&[
            ("GROVE_SECONDS", "-4"),
            ("GROVE_TIME", "teatime"),
            ("GROVE_DATE", "20/10/2026"),
            ("GROVE_WEATHER", "hail"),
        ]);
        assert_eq!(options, Options::default());
    }
}
