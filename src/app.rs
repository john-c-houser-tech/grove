//! Everything the window runs on, made and wired together in one place: the
//! session, and the weather and light that follow it. It knows nothing of Qt;
//! the QML types in `qml/` show it to the window and pass on what changed.

use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, Timelike};

use crate::color::Color;
use crate::light::{Moment, Sky, DEFAULT_ACCENT};
use crate::options::Options;
use crate::session::{Session, State, SEED_LIMIT};
use crate::weather::Weather;

/// Something that changed, for the window to hear about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// A session started, finished or was given up.
    State,
    /// The countdown moved on a second, or the plant grew enough to show.
    Progress,
    /// The sky's weather or the season changed enough to see.
    Weather,
    /// The plant's growth changed.
    Growth,
    /// The light moved on, or the accent changed.
    Light,
}

/// The app's state and the rules that tie its parts together.
pub struct App {
    options: Options,
    /// The moment the sky is frozen at, when a time or date was given.
    fixed: Option<NaiveDateTime>,
    session: Session,
    weather: Weather,
    accent: Color,
    sky: Sky,
}

impl App {
    pub fn new(options: Options) -> Self {
        let now = Local::now().naive_local();
        let fixed = (options.time.is_some() || options.date.is_some()).then(|| {
            options
                .date
                .unwrap_or(now.date())
                .and_time(options.time.unwrap_or(now.time()))
        });
        let accent = Color::hex(DEFAULT_ACCENT);
        let mut app = Self {
            session: Session::new(options.seconds),
            weather: Weather::new(options.weather),
            sky: Sky::at(moment(fixed.unwrap_or(now)), accent),
            accent,
            fixed,
            options,
        };
        // The idle window has weather too, until the first session rolls its own.
        app.weather.begin(fastrand::i32(1..SEED_LIMIT), app.today().month());
        app
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn weather(&self) -> &Weather {
        &self.weather
    }

    pub fn sky(&self) -> &Sky {
        &self.sky
    }

    /// Starts a session growing the plant of `seed`, unless one is growing.
    pub fn start(&mut self, seed: i32) -> Vec<Event> {
        let changes = self.session.start(seed);
        self.session_changed(changes)
    }

    /// Giving up early leaves a wilted plant behind.
    pub fn give_up(&mut self) -> Vec<Event> {
        let changes = self.session.give_up();
        self.session_changed(changes)
    }

    /// Advances the session clock by real time that has passed.
    pub fn advance(&mut self, milliseconds: i64) -> Vec<Event> {
        let changes = self.session.advance(milliseconds);
        self.session_changed(changes)
    }

    /// Reads the clock again for the sky, unless the sky is frozen.
    pub fn refresh_light(&mut self) -> Vec<Event> {
        self.sky = Sky::at(
            moment(self.fixed.unwrap_or_else(|| Local::now().naive_local())),
            self.accent,
        );
        vec![Event::Light]
    }

    /// Dusk warms toward the desktop accent.
    #[cfg_attr(not(test), expect(dead_code, reason = "called by the theme, which is ported next"))]
    pub fn set_accent(&mut self, accent: Color) -> Vec<Event> {
        if accent == self.accent {
            return Vec::new();
        }
        self.accent = accent;
        self.refresh_light()
    }

    /// What follows from a change to the session.
    fn session_changed(&mut self, changes: crate::session::Changes) -> Vec<Event> {
        let mut events = Vec::new();
        let mut weather = crate::weather::Changes::default();
        if changes.state {
            events.push(Event::State);
            // A new session rolls new weather; the plant's progress carries it along.
            if self.session.state() == State::Growing {
                weather = self.weather.begin(self.session.seed(), self.today().month());
            }
        }
        if changes.progress {
            events.push(Event::Progress);
            let moved = self.weather.set_progress(self.session.progress());
            weather.sky |= moved.sky;
            weather.growth |= moved.growth;
        }
        if weather.sky {
            events.push(Event::Weather);
        }
        if weather.growth {
            events.push(Event::Growth);
        }
        events
    }

    /// Read afresh for each session, since Grove may stay open for days.
    fn today(&self) -> NaiveDate {
        self.options.date.unwrap_or_else(|| Local::now().date_naive())
    }
}

/// A date and time as the sky takes it.
fn moment(when: NaiveDateTime) -> Moment {
    let time = when.time();
    Moment {
        day_of_year: when.ordinal(),
        hour: f64::from(time.hour()) + f64::from(time.minute()) / 60.0 + f64::from(time.second()) / 3600.0,
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveTime;

    use super::*;
    use crate::weather::{Kind, Season};

    fn options(seconds: i32) -> Options {
        Options {
            seconds,
            time: NaiveTime::from_hms_opt(12, 30, 0),
            date: NaiveDate::from_ymd_opt(2026, 6, 21),
            weather: Some(Kind::Rain),
        }
    }

    #[test]
    fn a_fixed_time_freezes_the_light() {
        let mut app = App::new(options(10));
        assert!(app.sky().sun_up);
        let before = *app.sky();
        assert_eq!(app.refresh_light(), vec![Event::Light]);
        assert_eq!(*app.sky(), before);
        // The accent only colours dusk, so noon looks the same under another.
        assert_eq!(app.set_accent(Color::hex("#112233")), vec![Event::Light]);
        assert_eq!(app.sky().top, before.top);
        assert_eq!(app.set_accent(Color::hex("#112233")), vec![]);
    }

    #[test]
    fn a_date_alone_fixes_the_season() {
        let app = App::new(Options {
            time: None,
            ..options(10)
        });
        assert_eq!(app.weather().season(), Season::Summer);
    }

    #[test]
    fn a_new_session_rolls_its_weather() {
        let mut app = App::new(options(10));
        let events = app.start(7);
        assert_eq!(
            events,
            vec![Event::State, Event::Progress, Event::Weather, Event::Growth]
        );
        assert_eq!(app.weather().kind(), Kind::Rain);
        assert_eq!(app.weather().growth(), 0.0);
    }

    #[test]
    fn the_plant_grows_with_the_session() {
        let mut app = App::new(options(10));
        app.start(7);
        let events = app.advance(5000);
        assert_eq!(events, vec![Event::Progress, Event::Growth]);
        // Rain nudges growth ahead of the clock.
        assert!(app.weather().growth() > app.session().progress());
        app.advance(5000);
        assert_eq!(app.session().state(), State::Done);
        assert_eq!(app.weather().growth(), 1.0);
    }

    #[test]
    fn giving_up_stops_the_growth() {
        let mut app = App::new(options(10));
        app.start(7);
        app.advance(4000);
        assert_eq!(app.give_up(), vec![Event::State]);
        let grown = app.weather().growth();
        assert_eq!(app.advance(4000), vec![]);
        assert_eq!(app.weather().growth(), grown);
    }
}
