//! The light in the garden, from the clock alone. There is no location
//! lookup: day length follows the calendar for a mid-northern latitude, which
//! is close enough for a toy and keeps the app offline.

use std::f64::consts::PI;

use crate::color::Color;
use crate::easing::{clamp01, smoothstep};

/// A moment as the sky needs it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Moment {
    /// The day of the year, 1 for the first of January.
    pub day_of_year: u32,
    /// The time of day in hours, from 0 up to 24.
    pub hour: f64,
}

/// The sky at one moment, worked out from the clock: where the sun or moon is
/// and how everything is lit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sky {
    /// The sky's colour overhead and at the horizon.
    pub top: Color,
    pub bottom: Color,
    /// Height of the sun above the horizon from -1 to 1, negative at night.
    pub elevation: f64,
    /// How far across the sky the sun or moon is, 0 at the left, 1 at the right.
    pub sun_x: f64,
    pub moon_x: f64,
    /// How high the moon is, 0 at the horizon and 1 overhead; -1 by day.
    pub moon_elevation: f64,
    /// Whether the sun is above the horizon; when it is not, the moon is up.
    pub sun_up: bool,
    /// 0 in darkness, 1 in full daylight; used to dim the plant and the ground.
    pub ambient: f64,
    /// How much the stars show, 0 by day to 1 in full dark.
    pub stars: f64,
    /// Peaks near sunrise and sunset, when the light turns golden.
    pub warmth: f64,
    /// The colour that golden light has, drawn toward the desktop accent.
    pub warm: Color,
}

/// Sunrise and sunset as hours of the day, for a day of the year.
pub fn sun_times(day_of_year: u32) -> (f64, f64) {
    // Day length swings about 4.3 to 15.7 hours across the year, longest at
    // the June solstice (day 172); solar noon sits near 12:30 on the clock.
    let season = (2.0 * PI * (f64::from(day_of_year) - 172.0) / 365.0).cos();
    let half = 6.07 + 1.78 * season;
    (12.5 - half, 12.5 + half)
}

/// Sky colours at one height of the sun.
struct Stop {
    elevation: f64,
    top: Color,
    bottom: Color,
}

/// The colours between the stops on either side of `elevation`.
fn along(stops: &[Stop], elevation: f64) -> (Color, Color) {
    let (first, last) = (&stops[0], &stops[stops.len() - 1]);
    if elevation <= first.elevation {
        return (first.top, first.bottom);
    }
    for pair in stops.windows(2) {
        let (below, above) = (&pair[0], &pair[1]);
        if elevation <= above.elevation {
            let t = (elevation - below.elevation) / (above.elevation - below.elevation);
            return (below.top.mix(above.top, t), below.bottom.mix(above.bottom, t));
        }
    }
    (last.top, last.bottom)
}

impl Sky {
    /// The sky at a moment, its dusk warmed toward `accent`.
    pub fn at(when: Moment, accent: Color) -> Self {
        let (sunrise, sunset) = sun_times(when.day_of_year);
        let hour = when.hour;

        let (sun_up, elevation, sun_x, moon_x, moon_elevation);
        if hour >= sunrise && hour <= sunset {
            let u = (hour - sunrise) / (sunset - sunrise);
            sun_up = true;
            elevation = (PI * u).sin();
            sun_x = u;
            moon_x = 0.5;
            moon_elevation = -1.0;
        } else {
            // The sun sinks only a little below the horizon, enough to take
            // the sky through twilight and into darkness and back.
            let since_sunset = if hour >= sunset {
                hour - sunset
            } else {
                hour + 24.0 - sunset
            };
            let v = since_sunset / (24.0 - (sunset - sunrise));
            sun_up = false;
            elevation = -0.4 * (PI * v).sin();
            sun_x = 0.5;
            moon_x = v;
            moon_elevation = (PI * v).sin();
        }

        // The horizon at golden hour is the warm colour, pulled toward the accent.
        let warm = Color::hex("#e8905a").mix(accent, 0.45);
        // Sky colours by the sun's height, from deep night to high noon.
        let stop = |elevation, top, bottom| Stop {
            elevation,
            top: Color::hex(top),
            bottom: Color::hex(bottom),
        };
        let stops = [
            stop(-0.40, "#060a14", "#0e1624"),
            stop(-0.15, "#0b1226", "#1c1f3a"),
            stop(-0.02, "#1d2550", "#6a4660"),
            Stop {
                elevation: 0.06,
                top: Color::hex("#33497a"),
                bottom: warm,
            },
            stop(0.25, "#4a78a8", "#9fc0cc"),
            stop(1.00, "#3d6f9c", "#8fb4c8"),
        ];
        let (top, bottom) = along(&stops, elevation);
        let d = (elevation - 0.04) / 0.12;
        Self {
            top,
            bottom,
            elevation,
            sun_x,
            moon_x,
            moon_elevation,
            sun_up,
            ambient: 0.22 + 0.78 * smoothstep(-0.12, 0.3, elevation),
            stars: clamp01((-elevation - 0.02) / 0.2),
            warmth: (-d * d).exp(),
            warm,
        }
    }

    /// The golden colour, with an alpha that rises with warmth, for tinting
    /// what the low sun lights.
    pub fn warm_tint(&self) -> Color {
        self.warm.with_alpha(self.warmth * 0.2)
    }

    /// A translucent blue that dims whatever it is applied to at night.
    pub fn night_tint(&self) -> Color {
        Color::rgba(0.02, 0.04, 0.12, (1.0 - self.ambient) * 0.62)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::DEFAULT_ACCENT;

    // Days of 2026: 21 June, 6 November and 21 December.
    const MIDSUMMER: u32 = 172;
    const NOVEMBER_6: u32 = 310;
    const MIDWINTER: u32 = 355;

    fn sky(day_of_year: u32, hour: f64) -> Sky {
        Sky::at(Moment { day_of_year, hour }, Color::hex(DEFAULT_ACCENT))
    }

    #[test]
    fn sunrise_is_earlier_in_summer() {
        let (summer_rise, summer_set) = sun_times(MIDSUMMER);
        let (winter_rise, winter_set) = sun_times(MIDWINTER);
        assert!(summer_rise < winter_rise);
        assert!(summer_set > winter_set);
        assert!(summer_set - summer_rise > 15.0);
        assert!(winter_set - winter_rise < 9.0);
    }

    #[test]
    fn noon_is_bright_and_midnight_is_dark() {
        let noon = sky(MIDSUMMER, 12.5);
        let midnight = sky(MIDSUMMER, 0.5);
        assert!(noon.sun_up);
        assert!(!midnight.sun_up);
        assert_eq!(noon.ambient, 1.0);
        assert!(midnight.ambient < 0.3);
        assert_eq!(noon.stars, 0.0);
        assert_eq!(midnight.stars, 1.0);
        assert!(noon.top.lightness() > midnight.top.lightness());
    }

    #[test]
    fn light_turns_golden_at_sunset() {
        let (_, set) = sun_times(NOVEMBER_6);
        assert!(sky(NOVEMBER_6, set - 0.3).warmth > 0.5);
        assert!(sky(NOVEMBER_6, 12.5).warmth < 0.05);
        assert!(sky(NOVEMBER_6, 1.0).warmth < 0.05);
    }

    #[test]
    fn moon_shares_the_night_sky() {
        let midnight = sky(NOVEMBER_6, 0.0);
        assert!(midnight.moon_elevation > 0.5);
        assert!(midnight.moon_x > 0.0 && midnight.moon_x < 1.0);
    }

    #[test]
    fn the_accent_only_colours_dusk() {
        let noon = Moment {
            day_of_year: MIDSUMMER,
            hour: 12.5,
        };
        assert_eq!(
            Sky::at(noon, Color::hex("#112233")).top,
            Sky::at(noon, Color::hex(DEFAULT_ACCENT)).top
        );
    }

    #[test]
    fn the_tints_follow_the_light() {
        let midnight = sky(MIDSUMMER, 0.5);
        let noon = sky(MIDSUMMER, 12.5);
        assert!(midnight.night_tint().a > 0.4);
        assert_eq!(noon.night_tint().a, 0.0);
        assert!(noon.warm_tint().a < 0.01);
    }
}
