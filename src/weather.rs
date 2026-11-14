//! The sky's mood, which is invented and never fetched. Each session rolls a
//! weather for its start and another for its end, weighted by the season, and
//! the plant grows through the change between them.

use fastrand::Rng;

use crate::easing::smoothstep;

/// A kind of weather.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Clear,
    Cloudy,
    Rain,
    Fog,
    Snow,
}

impl Kind {
    /// The name QML and the garden know the kind by.
    pub fn name(self) -> &'static str {
        match self {
            Kind::Clear => "clear",
            Kind::Cloudy => "cloudy",
            Kind::Rain => "rain",
            Kind::Fog => "fog",
            Kind::Snow => "snow",
        }
    }

    /// The kind of that name, if there is one.
    pub fn named(name: &str) -> Option<Self> {
        [Kind::Clear, Kind::Cloudy, Kind::Rain, Kind::Fog, Kind::Snow]
            .into_iter()
            .find(|kind| kind.name() == name)
    }

    /// The amounts of this weather once it has fully set in.
    pub fn conditions(self) -> Conditions {
        let (cloud, rain, fog, snow) = match self {
            Kind::Clear => (0.0, 0.0, 0.0, 0.0),
            Kind::Cloudy => (0.75, 0.0, 0.0, 0.0),
            Kind::Rain => (0.95, 0.8, 0.0, 0.0),
            Kind::Fog => (0.15, 0.0, 1.0, 0.0),
            Kind::Snow => (0.9, 0.0, 0.0, 0.85),
        };
        Conditions { cloud, rain, fog, snow }
    }

    /// Rolls a kind of weather by the season's weights.
    pub fn pick(random: &mut Rng, season: Season) -> Self {
        // Weights for clear, cloudy, rain, fog and snow. Summer is mostly
        // clear, autumn is the foggy season, and only winter brings snow.
        const KINDS: [Kind; 5] = [Kind::Clear, Kind::Cloudy, Kind::Rain, Kind::Fog, Kind::Snow];
        let weights: [f64; 5] = match season {
            Season::Spring => [3.0, 3.0, 3.0, 1.0, 0.0],
            Season::Summer => [6.0, 2.0, 0.7, 0.5, 0.0],
            Season::Autumn => [2.0, 3.0, 3.0, 2.5, 0.0],
            Season::Winter => [1.0, 3.0, 1.5, 2.0, 2.5],
        };
        let mut pick = random.f64() * weights.iter().sum::<f64>();
        for (kind, weight) in KINDS.into_iter().zip(weights) {
            if pick < weight {
                return kind;
            }
            pick -= weight;
        }
        Kind::Clear
    }
}

/// A season of the year.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    /// The name QML and the garden know the season by.
    pub fn name(self) -> &'static str {
        match self {
            Season::Spring => "spring",
            Season::Summer => "summer",
            Season::Autumn => "autumn",
            Season::Winter => "winter",
        }
    }

    /// The season of a month (1 to 12), by the calendar, in the northern
    /// hemisphere.
    pub fn of_month(month: u32) -> Self {
        match month {
            3..=5 => Season::Spring,
            6..=8 => Season::Summer,
            9..=11 => Season::Autumn,
            _ => Season::Winter,
        }
    }
}

/// How much of each weather there is, each from 0 to 1.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Conditions {
    pub cloud: f64,
    pub rain: f64,
    pub fog: f64,
    pub snow: f64,
}

impl Conditions {
    /// The dominant weather.
    pub fn kind(&self) -> Kind {
        if self.fog >= 0.45 {
            Kind::Fog
        } else if self.snow >= 0.4 {
            Kind::Snow
        } else if self.rain >= 0.4 {
            Kind::Rain
        } else if self.cloud >= 0.5 {
            Kind::Cloudy
        } else {
            Kind::Clear
        }
    }

    /// The largest difference in any one amount.
    fn distance(&self, other: &Self) -> f64 {
        (self.cloud - other.cloud)
            .abs()
            .max((self.rain - other.rain).abs())
            .max((self.fog - other.fog).abs())
            .max((self.snow - other.snow).abs())
    }
}

/// What a call changed that the window should hear about.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Changes {
    /// The sky (kind, season, cloud, rain, snow and fog) has changed: only
    /// once it has moved enough to see, about 1% of any amount, and when it
    /// settles. Growing does not redraw the sky, so growth is told apart.
    pub sky: bool,
    /// The plant's growth has changed, as often as the session's progress.
    pub growth: bool,
}

/// The weather of one session.
#[derive(Clone, Debug)]
pub struct Weather {
    forced: Option<Kind>,
    season: Season,
    from: Kind,
    to: Kind,
    now: Conditions,
    progress: f64,
    boost: f64,
}

impl Weather {
    /// A kind pins the weather, for screenshots and tests; `None` lets it
    /// roll. Until `begin` rolls a session's weather, the sky is clear, or as
    /// pinned.
    pub fn new(forced: Option<Kind>) -> Self {
        let kind = forced.unwrap_or(Kind::Clear);
        Self {
            forced,
            season: Season::Summer,
            from: kind,
            to: kind,
            now: kind.conditions(),
            progress: 0.0,
            boost: 0.0,
        }
    }

    /// The dominant weather right now.
    pub fn kind(&self) -> Kind {
        self.now.kind()
    }

    /// The season of the session's date.
    pub fn season(&self) -> Season {
        self.season
    }

    /// How much of each weather there is now. The amounts move from the
    /// session's first weather to its second between 30% and 70% of the way
    /// through.
    pub fn now(&self) -> Conditions {
        self.now
    }

    /// Session progress as the plant shows it: rain nudges growth along, and
    /// a finished session is always a finished plant.
    pub fn growth(&self) -> f64 {
        self.progress + self.boost * (1.0 - self.progress)
    }

    /// Rolls the weather for a new session in `month` (1 to 12).
    pub fn begin(&mut self, seed: i32, month: u32) -> Changes {
        let mut random = Rng::with_seed(seed as u64);
        self.season = Season::of_month(month);
        if let Some(kind) = self.forced {
            self.from = kind;
            self.to = kind;
        } else {
            self.from = Kind::pick(&mut random, self.season);
            // About half of all sessions change weather part-way through.
            self.to = if random.f64() < 0.5 {
                self.from
            } else {
                Kind::pick(&mut random, self.season)
            };
        }
        self.progress = 0.0;
        self.boost = 0.0;
        self.now = self.at(0.0);
        Changes {
            sky: true,
            growth: true,
        }
    }

    /// Moves the weather through the session, 0 to 1, as the session's
    /// progress changes.
    pub fn set_progress(&mut self, progress: f64) -> Changes {
        let progress = progress.clamp(0.0, 1.0);
        let mut changes = Changes::default();
        // Rain makes the plant grow a little faster, never more than a sliver.
        let gained = progress - self.progress;
        if gained > 0.0 {
            self.boost = (self.boost + gained * self.at(self.progress).rain * 0.3).min(0.15);
        }
        let grown = self.growth();
        self.progress = progress;
        changes.growth = self.growth() != grown;

        // The hills, the field and the plant are all redrawn when the sky
        // changes, and it changes over minutes, so a change is only announced
        // once it has moved enough to see, and once more when it settles.
        let next = self.at(progress);
        let moved = next.distance(&self.now);
        let settled = progress <= 0.3 || progress >= 0.7;
        if moved >= 0.01 || (settled && moved > 0.0) {
            self.now = next;
            changes.sky = true;
        }
        changes
    }

    /// The conditions at a point in the session.
    fn at(&self, progress: f64) -> Conditions {
        // The change happens in the middle of the session, not at a fixed moment.
        let s = smoothstep(0.3, 0.7, progress);
        let (a, b) = (self.from.conditions(), self.to.conditions());
        let between = |from: f64, to: f64| from + (to - from) * s;
        Conditions {
            cloud: between(a.cloud, b.cloud),
            rain: between(a.rain, b.rain),
            fog: between(a.fog, b.fog),
            snow: between(a.snow, b.snow),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOVEMBER: u32 = 11;

    fn begun(forced: Option<Kind>, seed: i32, month: u32) -> Weather {
        let mut weather = Weather::new(forced);
        weather.begin(seed, month);
        weather
    }

    /// The share of 4000 rolls in `season` that come up `kind`.
    fn share(kind: Kind, season: Season, seed: u64) -> f64 {
        let mut random = Rng::with_seed(seed);
        (0..4000).filter(|_| Kind::pick(&mut random, season) == kind).count() as f64 / 4000.0
    }

    #[test]
    fn seasons_follow_the_calendar() {
        assert_eq!(Season::of_month(4), Season::Spring);
        assert_eq!(Season::of_month(7), Season::Summer);
        assert_eq!(Season::of_month(11), Season::Autumn);
        assert_eq!(Season::of_month(1), Season::Winter);
    }

    #[test]
    fn kinds_are_known_by_name() {
        assert_eq!(Kind::named("fog"), Some(Kind::Fog));
        assert_eq!(Kind::named("hail"), None);
        assert_eq!(Kind::named(""), None);
    }

    #[test]
    fn pinned_weather_never_changes() {
        let mut weather = begun(Some(Kind::Fog), 5, NOVEMBER);
        for progress in [0.0, 0.5, 1.0] {
            weather.set_progress(progress);
            assert_eq!(weather.kind(), Kind::Fog);
        }
        assert!(weather.now().fog > 0.8);
    }

    #[test]
    fn same_seed_gives_the_same_weather() {
        let (mut a, mut b) = (begun(None, 1234, NOVEMBER), begun(None, 1234, NOVEMBER));
        for progress in [0.0, 0.5, 1.0] {
            a.set_progress(progress);
            b.set_progress(progress);
            assert_eq!(a.kind(), b.kind());
            assert_eq!(a.now(), b.now());
        }
    }

    #[test]
    fn weather_can_change_during_a_session() {
        // Some seed in a hundred must start and end differently.
        let changed = (1..100).any(|seed| {
            let mut weather = begun(None, seed, NOVEMBER);
            let start = weather.kind();
            weather.set_progress(1.0);
            weather.kind() != start
        });
        assert!(changed);
    }

    #[test]
    fn autumn_is_foggier_than_summer() {
        assert!(share(Kind::Fog, Season::Autumn, 99) > share(Kind::Fog, Season::Summer, 99) * 2.0);
    }

    #[test]
    fn snow_only_falls_in_winter() {
        assert_eq!(share(Kind::Snow, Season::Spring, 5), 0.0);
        assert_eq!(share(Kind::Snow, Season::Summer, 5), 0.0);
        assert_eq!(share(Kind::Snow, Season::Autumn, 5), 0.0);
        assert!(share(Kind::Snow, Season::Winter, 5) > 0.15);
    }

    #[test]
    fn snow_is_snow_not_rain() {
        let mut weather = begun(Some(Kind::Snow), 1, 1);
        assert_eq!(weather.kind(), Kind::Snow);
        assert!(weather.now().snow > 0.8);
        assert_eq!(weather.now().rain, 0.0);
        // Snow does not hurry the plant along the way rain does.
        weather.set_progress(0.5);
        assert!((weather.growth() - 0.5).abs() < 0.01);
    }

    #[test]
    fn rain_speeds_growth_but_finishes_on_time() {
        let mut weather = begun(Some(Kind::Rain), 1, NOVEMBER);
        assert_eq!(weather.growth(), 0.0);
        weather.set_progress(0.5);
        assert!(weather.growth() > 0.5);
        assert!(weather.growth() < 0.62);
        weather.set_progress(1.0);
        assert_eq!(weather.growth(), 1.0);

        let mut dry = begun(Some(Kind::Clear), 1, NOVEMBER);
        dry.set_progress(0.5);
        assert!((dry.growth() - 0.5).abs() < 0.01);
    }

    #[test]
    fn growth_never_goes_backwards() {
        let mut weather = begun(Some(Kind::Rain), 1, NOVEMBER);
        let mut last = 0.0;
        for i in 0..=100 {
            weather.set_progress(f64::from(i) / 100.0);
            assert!(weather.growth() >= last);
            last = weather.growth();
        }
    }

    #[test]
    fn growing_does_not_redraw_steady_weather() {
        let mut weather = begun(Some(Kind::Rain), 1, NOVEMBER);
        let (mut sky, mut growth) = (0, 0);
        for i in 1..=100 {
            let changes = weather.set_progress(f64::from(i) / 100.0);
            sky += usize::from(changes.sky);
            growth += usize::from(changes.growth);
        }
        assert_eq!(sky, 0);
        assert_eq!(growth, 100);
    }

    #[test]
    fn changing_weather_is_announced_in_visible_steps_and_settles_exactly() {
        for seed in 1..100 {
            let mut weather = begun(None, seed, NOVEMBER);
            let mut jumped = begun(None, seed, NOVEMBER);
            jumped.set_progress(1.0);
            if jumped.kind() == weather.kind() {
                continue;
            }
            // A full session's worth of announced ticks.
            let sky = (1..=1500)
                .filter(|i| weather.set_progress(f64::from(*i) / 1500.0).sky)
                .count();
            assert!(sky > 5);
            assert!(sky <= 110);
            assert_eq!(weather.kind(), jumped.kind());
            assert_eq!(weather.now(), jumped.now());
            return;
        }
        panic!("no seed changed the weather");
    }
}
