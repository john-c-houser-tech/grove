//! Everything the window runs on, made and wired together in one place: the
//! session, and the weather and light that follow it. It knows nothing of Qt;
//! the QML types in `qml/` show it to the window and pass on what changed.

use std::sync::Arc;

use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, Timelike};

use crate::garden::{self, Garden, Outcome, Plant};
use crate::language::{self, Catalog};
use crate::light::{Moment, Sky};
use crate::notifier;
use crate::options::Options;
use crate::player;
use crate::session::{Session, State, SEED_LIMIT};
use crate::settings;
use crate::sounds::{self, Period, Plan};
use crate::theme::Theme;
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
    /// The desktop's accent or corner radius changed.
    Theme,
    /// The language changed, so everything said is said again.
    Language,
    /// A plant grew to the end, which the desktop should be told.
    PlantGrown,
    /// Whether notifications work changed.
    NotifierProblem,
    /// A sound of that kind is coming in this session, and can be made.
    SoundComing(sounds::Kind),
    /// A sound of that kind is due now, with its picture.
    Sound(sounds::Kind),
    /// The session's sounds are over: one under way fades out, with its
    /// picture, and one still waiting is dropped.
    SoundsEnded,
    /// The sounds were muted or unmuted.
    Muted,
    /// Whether the sounds can be heard changed.
    SoundProblem,
    /// The garden's plants changed, in the way given.
    Garden(garden::Change),
    /// The garden's problem changed.
    GardenProblem,
}

/// The app's state and the rules that tie its parts together.
pub struct App {
    options: Options,
    /// The moment the sky is frozen at, when a time or date was given.
    fixed: Option<NaiveDateTime>,
    session: Session,
    weather: Weather,
    theme: Theme,
    sky: Sky,
    garden: Garden,
    /// The current language's strings, shared with whoever translates.
    catalog: Arc<Catalog>,
    /// The id of the last notification sent, which the next replaces, and
    /// why notifications cannot be sent, if they cannot.
    last_notification: u32,
    notifier_problem: Option<notifier::Problem>,
    /// The sounds still to come in this session.
    plan: Option<Plan>,
    muted: bool,
    sound_problem: Option<player::Problem>,
    random: fastrand::Rng,
}

impl App {
    /// The app, with its garden where the options say, or in the user's
    /// data directory.
    pub fn new(options: Options) -> Self {
        let path = options
            .garden_path
            .clone()
            .or_else(|| options.data_directory.as_deref().map(Garden::default_path));
        let garden = path.map_or_else(Garden::unkept, Garden::open);
        Self::with_garden(options, garden)
    }

    /// The app, keeping its plants in `garden`.
    pub fn with_garden(options: Options, garden: Garden) -> Self {
        let now = Local::now().naive_local();
        let fixed = (options.time.is_some() || options.date.is_some()).then(|| {
            options
                .date
                .unwrap_or(now.date())
                .and_time(options.time.unwrap_or(now.time()))
        });
        let theme = Theme::read(options.theme_directory.clone());
        let catalog = Arc::new(Catalog::load(&language_code(&options), &options.catalogs));
        let mut app = Self {
            session: Session::new(options.seconds),
            weather: Weather::new(options.weather),
            sky: Sky::at(moment(fixed.unwrap_or(now)), theme.accent_color()),
            theme,
            fixed,
            garden,
            catalog,
            last_notification: 0,
            notifier_problem: None,
            plan: None,
            muted: options.settings_file.as_deref().is_some_and(settings::muted),
            sound_problem: None,
            random: fastrand::Rng::new(),
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

    pub fn options(&self) -> &Options {
        &self.options
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn garden(&self) -> &Garden {
        &self.garden
    }

    /// The current language's strings.
    pub fn catalog(&self) -> &Arc<Catalog> {
        &self.catalog
    }

    /// The name of the locale numbers and dates are formatted with.
    pub fn format_locale(&self) -> String {
        language::format_locale(self.catalog.code(), self.options.omarchy_path.as_deref())
    }

    /// Reads the language again, after its file changed. Catalogs do not
    /// change while Grove runs, so only another language is news.
    pub fn reload_language(&mut self) -> Vec<Event> {
        let code = language_code(&self.options);
        if code == self.catalog.code() {
            return Vec::new();
        }
        self.catalog = Arc::new(Catalog::load(&code, &self.options.catalogs));
        vec![Event::Language]
    }

    /// The id of the last notification sent, or 0, for the next to replace.
    pub fn last_notification(&self) -> u32 {
        self.last_notification
    }

    /// `None` when notifications work, otherwise why they cannot.
    pub fn notifier_problem(&self) -> Option<notifier::Problem> {
        self.notifier_problem
    }

    /// Takes the outcome of sending a notification: its id, or the problem.
    pub fn notified(&mut self, outcome: Result<u32, notifier::Problem>) -> Vec<Event> {
        let problem = outcome.err();
        if let Ok(id) = outcome {
            self.last_notification = id;
        }
        if problem == self.notifier_problem {
            return Vec::new();
        }
        self.notifier_problem = problem;
        vec![Event::NotifierProblem]
    }

    /// Whether the sounds are muted. Their pictures show either way.
    pub fn muted(&self) -> bool {
        self.muted
    }

    /// Mutes or unmutes the sounds, and remembers the choice.
    pub fn set_muted(&mut self, muted: bool) -> Vec<Event> {
        if muted == self.muted {
            return Vec::new();
        }
        self.muted = muted;
        if let Some(file) = &self.options.settings_file {
            settings::remember_muted(file, muted);
        }
        vec![Event::Muted]
    }

    /// How loud a sound is played, from 0 to 1.
    pub fn volume(&self, kind: sounds::Kind) -> f64 {
        kind.volume() * self.options.sound_volume
    }

    /// `None` when the sounds can be heard, otherwise why they cannot.
    pub fn sound_problem(&self) -> Option<player::Problem> {
        self.sound_problem
    }

    /// Takes what the player said of a sound: that it plays, or why not.
    pub fn sound_reported(&mut self, report: player::Report) -> Vec<Event> {
        let problem = match report {
            player::Report::Started(_) => None,
            player::Report::Failed(_, problem) | player::Report::Broke(problem) => Some(problem),
            player::Report::Muted(_) => return Vec::new(),
        };
        if problem == self.sound_problem {
            return Vec::new();
        }
        self.sound_problem = problem;
        vec![Event::SoundProblem]
    }

    /// The part of the day, as far as the sounds care.
    fn period(&self) -> Period {
        Period::of(self.sky.sun_up, self.sky.ambient)
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
        let mut events = Vec::new();
        // A sound due by now starts, if the sky still suits it: a plane
        // needs weather it can be seen and heard in.
        let now = self.weather.now();
        let flight_weather = now.rain < 0.25 && now.snow < 0.25 && now.fog < 0.35;
        let (elapsed, period) = (self.session.elapsed_seconds(), self.period());
        if let Some(kind) = self
            .plan
            .as_mut()
            .and_then(|plan| plan.due(elapsed, period, flight_weather))
        {
            events.push(Event::Sound(kind));
        }
        events.extend(self.session_changed(changes));
        events
    }

    /// Reads the clock again for the sky, unless the sky is frozen.
    pub fn refresh_light(&mut self) -> Vec<Event> {
        let when = self.fixed.unwrap_or_else(|| Local::now().naive_local());
        // Dusk warms toward the desktop accent.
        self.sky = Sky::at(moment(when), self.theme.accent_color());
        vec![Event::Light]
    }

    /// Reads the theme's accent again, after its files changed.
    pub fn reload_theme(&mut self) -> Vec<Event> {
        if !self.theme.reload() {
            return Vec::new();
        }
        let mut events = vec![Event::Theme];
        events.extend(self.refresh_light());
        events
    }

    /// Takes the rounding Hyprland reported.
    pub fn set_radius(&mut self, radius: i32) -> Vec<Event> {
        if self.theme.set_radius(radius) {
            vec![Event::Theme]
        } else {
            Vec::new()
        }
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
        if changes.state {
            // The distant sounds belong to a growing plant: any of the last
            // session end with it, and a new session plans its own.
            events.push(Event::SoundsEnded);
            self.plan = None;
            if self.session.state() == State::Growing {
                let seconds = f64::from(self.session.seconds());
                let period = self.period();
                self.plan = Plan::roll(&mut self.random, self.options.sound_test, period, seconds);
                if let Some(plan) = &self.plan {
                    events.push(Event::SoundComing(plan.kind()));
                }
            }
            self.keep_plant(&mut events);
        }
        events
    }

    /// Puts the plant of a session that just ended into the garden.
    fn keep_plant(&mut self, events: &mut Vec<Event>) {
        let outcome = match self.session.state() {
            State::Done => Outcome::Done,
            State::Wilted => Outcome::Wilted,
            State::Idle | State::Growing => return,
        };
        // A shortened test session (GROVE_SECONDS) is not saved unless a
        // garden is given, so trying the app out never adds make-believe
        // plants to the real garden.
        if self.session.seconds() != 25 * 60 && self.options.garden_path.is_none() {
            return;
        }
        // The same goes for telling the desktop.
        if outcome == Outcome::Done {
            events.push(Event::PlantGrown);
        }
        let plant = Plant {
            seed: self.session.seed(),
            outcome,
            season: self.weather.season().name().to_string(),
            weather: self.weather.kind().name().to_string(),
            progress: self.weather.growth(),
            when: Local::now().naive_local(),
            minutes: if outcome == Outcome::Done {
                self.session.minutes()
            } else {
                0
            },
        };
        events.extend(self.record(plant));
    }

    /// Puts a plant into the garden.
    pub fn record(&mut self, plant: Plant) -> Vec<Event> {
        let problem = self.garden.problem().cloned();
        let mut events = vec![Event::Garden(self.garden.record(plant))];
        if self.garden.problem() != problem.as_ref() {
            events.push(Event::GardenProblem);
        }
        events
    }

    /// Read afresh for each session, since Grove may stay open for days.
    fn today(&self) -> NaiveDate {
        self.options.date.unwrap_or_else(|| Local::now().date_naive())
    }
}

/// The language Omarchy is set to; English without Omarchy.
fn language_code(options: &Options) -> String {
    options
        .language_file
        .as_deref()
        .map_or_else(|| "en".to_string(), language::read_code)
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
            garden_path: None,
            data_directory: None,
            theme_directory: None,
            hyprland: None,
            language_file: None,
            catalogs: crate::language::Catalogs::BuiltIn,
            omarchy_path: None,
            notification_service: "org.grove.NoSuchService".into(),
            sound_test: None,
            sound_volume: 1.0,
            settings_file: None,
        }
    }

    /// An app whose options name no garden, so short sessions are not kept.
    /// Its garden is a file that is never there, never the user's.
    fn app(seconds: i32) -> App {
        App::with_garden(options(seconds), nowhere())
    }

    fn nowhere() -> Garden {
        Garden::open("/nonexistent/grove-tests/garden.json")
    }

    /// Options keeping a garden in `dir`.
    fn keeping(seconds: i32, dir: &tempfile::TempDir) -> Options {
        Options {
            garden_path: Some(dir.path().join("garden.json")),
            ..options(seconds)
        }
    }

    #[test]
    fn a_fixed_time_freezes_the_light() {
        let mut app = app(10);
        assert!(app.sky().sun_up);
        let before = *app.sky();
        assert_eq!(app.refresh_light(), vec![Event::Light]);
        assert_eq!(*app.sky(), before);
    }

    #[test]
    fn the_theme_colours_dusk_and_rounds_corners() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir(dir.path().join("theme")).unwrap();
        let colors = dir.path().join("theme/colors.toml");
        std::fs::write(&colors, "accent = \"#112233\"\n").unwrap();
        let dusk = Options {
            time: NaiveTime::from_hms_opt(20, 0, 0),
            theme_directory: Some(dir.path().to_path_buf()),
            ..options(10)
        };
        let mut app = App::with_garden(dusk, nowhere());
        assert_eq!(app.theme().accent(), "#112233");
        let horizon = app.sky().bottom;

        assert_eq!(app.reload_theme(), vec![]);
        std::fs::write(&colors, "accent = \"#ff0000\"\n").unwrap();
        assert_eq!(app.reload_theme(), vec![Event::Theme, Event::Light]);
        assert_ne!(app.sky().bottom, horizon);

        assert_eq!(app.set_radius(8), vec![Event::Theme]);
        assert_eq!(app.set_radius(8), vec![]);
        assert_eq!(app.theme().radius(), 8);
    }

    #[test]
    fn the_language_follows_its_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let file = dir.path().join("language");
        std::fs::write(&file, "de\n").unwrap();
        let mut app = App::with_garden(
            Options {
                language_file: Some(file.clone()),
                ..options(10)
            },
            nowhere(),
        );
        assert_eq!(app.catalog().code(), "de");
        assert_eq!(app.catalog().translate("Garden", None), Some("Garten"));
        assert_eq!(app.format_locale(), "de");

        assert_eq!(app.reload_language(), vec![]);
        std::fs::write(&file, "fr\n").unwrap();
        assert_eq!(app.reload_language(), vec![Event::Language]);
        assert_eq!(app.catalog().translate("Garden", None), Some("Jardin"));
        std::fs::remove_file(&file).unwrap();
        assert_eq!(app.reload_language(), vec![Event::Language]);
        assert_eq!(app.catalog().code(), "en");
    }

    #[test]
    fn a_notification_problem_is_told_when_it_comes_and_goes() {
        let mut app = app(10);
        assert_eq!(
            app.notified(Err(notifier::Problem::NoService)),
            vec![Event::NotifierProblem]
        );
        assert_eq!(app.notified(Err(notifier::Problem::NoService)), vec![]);
        assert_eq!(app.notifier_problem(), Some(notifier::Problem::NoService));
        assert_eq!(app.notified(Ok(41)), vec![Event::NotifierProblem]);
        assert_eq!(app.notifier_problem(), None);
        // The next notification replaces this one.
        assert_eq!(app.last_notification(), 41);
        assert_eq!(app.notified(Ok(42)), vec![]);
        assert_eq!(app.last_notification(), 42);
    }

    #[test]
    fn a_plane_flies_over_a_day_session_and_a_train_passes_at_night() {
        // The options fix noon in June, in rain, which grounds planes unless
        // one is asked for by name.
        let mut asked = App::with_garden(
            Options {
                sound_test: Some(sounds::Kind::Plane),
                ..options(60)
            },
            nowhere(),
        );
        assert!(asked.start(7).contains(&Event::SoundComing(sounds::Kind::Plane)));
        assert!(!asked.advance(2900).contains(&Event::Sound(sounds::Kind::Plane)));
        assert!(asked.advance(200).contains(&Event::Sound(sounds::Kind::Plane)));
        assert!(!asked.advance(5000).contains(&Event::Sound(sounds::Kind::Plane)));

        let night = Options {
            time: NaiveTime::from_hms_opt(1, 0, 0),
            ..options(25 * 60)
        };
        let mut app = App::with_garden(night, nowhere());
        // Every night session has at least one train.
        assert!(app.start(7).contains(&Event::SoundComing(sounds::Kind::Train)));
        let mut trains = 0;
        for _ in 0..(25 * 60 - 1) {
            trains += app
                .advance(1000)
                .iter()
                .filter(|event| **event == Event::Sound(sounds::Kind::Train))
                .count();
        }
        assert!((1..=3).contains(&trains), "{trains} trains");
    }

    #[test]
    fn a_session_that_ends_takes_its_sounds_with_it() {
        let mut app = App::with_garden(
            Options {
                sound_test: Some(sounds::Kind::Train),
                ..options(60)
            },
            nowhere(),
        );
        app.start(7);
        assert!(app.give_up().contains(&Event::SoundsEnded));
        // The train due three seconds in never comes.
        assert_eq!(app.advance(5000), vec![]);
        // A new session ends whatever was left of the old before planning its own.
        let events = app.start(8);
        let ended = events.iter().position(|event| *event == Event::SoundsEnded).unwrap();
        let coming = events
            .iter()
            .position(|event| *event == Event::SoundComing(sounds::Kind::Train))
            .unwrap();
        assert!(ended < coming);
    }

    #[test]
    fn muting_is_remembered_between_runs() {
        let dir = tempfile::TempDir::new().unwrap();
        let remembering = || Options {
            settings_file: Some(dir.path().join("grove.conf")),
            ..options(10)
        };
        let mut first = App::with_garden(remembering(), nowhere());
        assert!(!first.muted());
        assert_eq!(first.set_muted(true), vec![Event::Muted]);
        assert_eq!(first.set_muted(true), vec![]);
        assert!(App::with_garden(remembering(), nowhere()).muted());
        // Without a settings file the choice is only for this run.
        let mut forgetful = app(10);
        forgetful.set_muted(true);
        assert!(forgetful.muted());
    }

    #[test]
    fn a_sound_problem_is_told_when_it_comes_and_goes() {
        use player::{Problem, Report};
        let mut app = app(10);
        let plane = sounds::Kind::Plane;
        assert_eq!(
            app.sound_reported(Report::Failed(plane, Problem::NoOutput)),
            vec![Event::SoundProblem]
        );
        assert_eq!(app.sound_reported(Report::Muted(plane)), vec![]);
        assert_eq!(app.sound_problem(), Some(Problem::NoOutput));
        assert_eq!(app.sound_reported(Report::Started(plane)), vec![Event::SoundProblem]);
        assert_eq!(app.sound_problem(), None);
        assert_eq!(
            app.sound_reported(Report::Broke(Problem::Unusable)),
            vec![Event::SoundProblem]
        );
    }

    #[test]
    fn the_volume_can_be_scaled_down_to_nothing() {
        let quiet = App::with_garden(
            Options {
                sound_volume: 0.5,
                ..options(10)
            },
            nowhere(),
        );
        assert_eq!(quiet.volume(sounds::Kind::Train), 0.225);
        assert_eq!(app(10).volume(sounds::Kind::Plane), 0.3);
    }

    #[test]
    fn a_date_alone_fixes_the_season() {
        let app = App::with_garden(
            Options {
                time: None,
                ..options(10)
            },
            nowhere(),
        );
        assert_eq!(app.weather().season(), Season::Summer);
    }

    #[test]
    fn a_new_session_rolls_its_weather() {
        let mut app = app(10);
        let events = app.start(7);
        assert_eq!(
            events,
            vec![
                Event::State,
                Event::Progress,
                Event::Weather,
                Event::Growth,
                Event::SoundsEnded
            ]
        );
        assert_eq!(app.weather().kind(), Kind::Rain);
        assert_eq!(app.weather().growth(), 0.0);
    }

    #[test]
    fn the_plant_grows_with_the_session() {
        let mut app = app(10);
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
    fn a_finished_plant_goes_into_the_garden() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut app = App::new(keeping(120, &dir));
        app.start(7);
        let events = app.advance(120_000);
        assert!(events.contains(&Event::Garden(garden::Change::Reset)));
        assert!(events.contains(&Event::PlantGrown));
        assert!(!events.contains(&Event::GardenProblem));
        let plant = &app.garden().plants()[0];
        assert_eq!((plant.seed, plant.outcome, plant.minutes), (7, Outcome::Done, 2));
        assert_eq!(
            (plant.season.as_str(), plant.weather.as_str(), plant.progress),
            ("summer", "rain", 1.0)
        );
        assert_eq!(App::new(keeping(120, &dir)).garden().plants().len(), 1);
    }

    #[test]
    fn a_wilted_plant_is_kept_as_far_as_it_grew() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut app = App::new(keeping(120, &dir));
        app.start(7);
        app.advance(30_000);
        // A wilted plant is nothing to tell the desktop about.
        assert!(!app.give_up().contains(&Event::PlantGrown));
        let plant = &app.garden().plants()[0];
        assert_eq!((plant.outcome, plant.minutes), (Outcome::Wilted, 0));
        assert!(plant.progress > 0.25 && plant.progress < 0.4);
    }

    #[test]
    fn a_shortened_session_is_not_kept_without_a_garden_of_its_own() {
        // The path is never touched: the session is too short to be saved.
        let mut app = app(10);
        app.start(7);
        let events = app.advance(10_000);
        assert!(!events
            .iter()
            .any(|event| matches!(event, Event::Garden(_) | Event::PlantGrown)));
    }

    #[test]
    fn a_spoiled_garden_is_reported_once() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut app = App::new(keeping(120, &dir));
        std::fs::write(dir.path().join("garden.json"), "this is not json").unwrap();
        app.start(7);
        let events = app.advance(120_000);
        assert!(events.contains(&Event::Garden(garden::Change::Inserted)));
        assert!(events.contains(&Event::GardenProblem));
        app.start(8);
        assert!(!app.advance(120_000).contains(&Event::GardenProblem));
    }

    #[test]
    fn giving_up_stops_the_growth() {
        let mut app = app(10);
        app.start(7);
        app.advance(4000);
        assert_eq!(app.give_up(), vec![Event::State, Event::SoundsEnded]);
        let grown = app.weather().growth();
        assert_eq!(app.advance(4000), vec![]);
        assert_eq!(app.weather().growth(), grown);
    }
}
