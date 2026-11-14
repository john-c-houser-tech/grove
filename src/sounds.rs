//! Faint sounds from far away while a plant grows. At night a train
//! approaches a crossing, one to three times in a session; by day a small
//! plane flies over now and then. This file decides which and when; they
//! are made in `synth.rs` and played by `player.rs`.

use fastrand::Rng;

use crate::synth::{PLANE_SECONDS, TRAIN_SECONDS};

/// A distant sound, and the picture that goes with it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Train,
    Plane,
}

impl Kind {
    /// The kind of that name ("train" or "plane"), if there is one.
    pub fn named(name: &str) -> Option<Self> {
        match name {
            "train" => Some(Kind::Train),
            "plane" => Some(Kind::Plane),
            _ => None,
        }
    }

    /// How long the sound lasts, in seconds, and so its picture.
    pub fn seconds(self) -> f64 {
        match self {
            Kind::Train => TRAIN_SECONDS,
            Kind::Plane => PLANE_SECONDS,
        }
    }

    /// How long the sound lasts, in milliseconds.
    pub fn milliseconds(self) -> i32 {
        (self.seconds() * 1000.0) as i32
    }

    /// How loud the sound is played, from 0 to 1, before the user's own scale.
    pub fn volume(self) -> f64 {
        match self {
            Kind::Train => 0.45,
            Kind::Plane => 0.3,
        }
    }

    /// How many times the sound happens in one session: a train 1 to 3
    /// times, a plane 0 to 2.
    pub fn count(self, random: &mut Rng) -> usize {
        let r = random.f64();
        match self {
            Kind::Train if r < 0.45 => 1,
            Kind::Train if r < 0.8 => 2,
            Kind::Train => 3,
            Kind::Plane if r < 0.35 => 0,
            Kind::Plane if r < 0.8 => 1,
            Kind::Plane => 2,
        }
    }
}

/// The part of the day, as far as the sounds care.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Period {
    Day,
    Night,
    /// Dawn and dusk, when nothing happens.
    Between,
}

impl Period {
    /// The period under a sky: day in good light, night in the dark.
    pub fn of(sun_up: bool, ambient: f64) -> Self {
        if sun_up && ambient > 0.6 {
            Period::Day
        } else if !sun_up && ambient < 0.35 {
            Period::Night
        } else {
            Period::Between
        }
    }

    /// What is heard in this period: trains at night, planes by day.
    fn kind(self) -> Option<Kind> {
        match self {
            Period::Night => Some(Kind::Train),
            Period::Day => Some(Kind::Plane),
            Period::Between => None,
        }
    }
}

/// When sounds begin, in seconds from the start of a session: `count` of
/// them if they fit, always inside the session and well apart; fewer come
/// back if they do not fit.
pub fn times(random: &mut Rng, count: usize, session_seconds: f64, sound_seconds: f64) -> Vec<f64> {
    // Nothing right at the start or the end, and never two close together.
    let (margin, gap) = (20.0, sound_seconds + 90.0);
    let latest = session_seconds - sound_seconds - margin;
    for count in (1..=count).rev() {
        if latest < margin + (count - 1) as f64 * gap {
            continue;
        }
        for _attempt in 0..200 {
            let mut times: Vec<f64> = (0..count).map(|_| margin + random.f64() * (latest - margin)).collect();
            times.sort_by(f64::total_cmp);
            if times.windows(2).all(|pair| pair[1] - pair[0] >= gap) {
                return times;
            }
        }
    }
    Vec::new()
}

/// The sounds of one session: what, and when the ones still to come begin.
#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    kind: Kind,
    /// Seconds from the start of the session, soonest first.
    times: Vec<f64>,
    /// Whether the sound was asked for by name, so it happens whatever the
    /// hour and the weather.
    forced: bool,
}

impl Plan {
    /// The plan for a session of `session_seconds` starting in `period`.
    /// `forced` makes that sound happen three seconds in whatever the hour,
    /// for trying it out. `None` when nothing is to be heard.
    pub fn roll(random: &mut Rng, forced: Option<Kind>, period: Period, session_seconds: f64) -> Option<Self> {
        if let Some(kind) = forced {
            return Some(Self {
                kind,
                times: vec![3.0],
                forced: true,
            });
        }
        let kind = period.kind()?;
        let count = kind.count(random);
        let times = times(random, count, session_seconds, kind.seconds());
        (!times.is_empty()).then_some(Self {
            kind,
            times,
            forced: false,
        })
    }

    /// What this session's sounds are, so they can be made ahead of time.
    pub fn kind(&self) -> Kind {
        self.kind
    }

    /// The sound to start now, `elapsed` seconds into the session, if one is
    /// due and the sky still suits it: night may have ended, or the weather
    /// turned. A sound that is due is used up either way.
    pub fn due(&mut self, elapsed: f64, period: Period, flight_weather: bool) -> Option<Kind> {
        if self.times.first().is_none_or(|time| *time > elapsed) {
            return None;
        }
        self.times.remove(0);
        let suits = match self.kind {
            Kind::Train => period == Period::Night,
            Kind::Plane => period == Period::Day && flight_weather,
        };
        (self.forced || suits).then_some(self.kind)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn trains_happen_one_to_three_times_and_planes_zero_to_two() {
        let mut random = Rng::with_seed(8);
        let (mut trains, mut planes) = (HashSet::new(), HashSet::new());
        for _ in 0..500 {
            let (train, plane) = (Kind::Train.count(&mut random), Kind::Plane.count(&mut random));
            assert!((1..=3).contains(&train));
            assert!((0..=2).contains(&plane));
            trains.insert(train);
            planes.insert(plane);
        }
        assert_eq!(trains.len(), 3);
        assert_eq!(planes.len(), 3);
    }

    #[test]
    fn events_are_spread_within_the_session() {
        let mut random = Rng::with_seed(4);
        for _ in 0..200 {
            let times = times(&mut random, 3, 1500.0, 44.0);
            assert_eq!(times.len(), 3);
            for (j, time) in times.iter().enumerate() {
                assert!(*time >= 20.0);
                assert!(time + 44.0 <= 1500.0 - 20.0);
                if j > 0 {
                    assert!(time - times[j - 1] >= 44.0 + 90.0);
                }
            }
        }
    }

    #[test]
    fn too_short_a_session_gets_no_sounds() {
        let mut random = Rng::with_seed(1);
        assert!(times(&mut random, 2, 30.0, 44.0).is_empty());
        // Room for one but not three: fewer come back rather than crowding.
        let some = times(&mut random, 3, 200.0, 44.0);
        assert!(!some.is_empty() && some.len() < 3);
    }

    #[test]
    fn trains_run_at_night_and_planes_fly_by_day() {
        assert_eq!(Period::of(true, 1.0), Period::Day);
        assert_eq!(Period::of(false, 0.22), Period::Night);
        assert_eq!(Period::of(true, 0.5), Period::Between);
        assert_eq!(Period::of(false, 0.5), Period::Between);
        let mut random = Rng::with_seed(3);
        for _ in 0..50 {
            assert_eq!(
                Plan::roll(&mut random, None, Period::Night, 1500.0).unwrap().kind(),
                Kind::Train
            );
            if let Some(plan) = Plan::roll(&mut random, None, Period::Day, 1500.0) {
                assert_eq!(plan.kind(), Kind::Plane);
            }
        }
    }

    #[test]
    fn no_sounds_at_dawn_or_dusk() {
        let mut random = Rng::with_seed(3);
        assert_eq!(Plan::roll(&mut random, None, Period::Between, 1500.0), None);
    }

    #[test]
    fn a_sound_asked_for_by_name_comes_three_seconds_in_whatever_the_hour() {
        let mut random = Rng::with_seed(3);
        let mut plan = Plan::roll(&mut random, Some(Kind::Plane), Period::Night, 60.0).unwrap();
        assert_eq!(plan.due(2.9, Period::Night, false), None);
        assert_eq!(plan.due(3.0, Period::Night, false), Some(Kind::Plane));
        // Only once.
        assert_eq!(plan.due(4.0, Period::Night, false), None);
    }

    #[test]
    fn a_sound_is_dropped_if_the_sky_no_longer_suits_it() {
        let train = |period| {
            let mut plan = Plan {
                kind: Kind::Train,
                times: vec![30.0, 400.0],
                forced: false,
            };
            (plan.due(31.0, period, true), plan.times.len())
        };
        assert_eq!(train(Period::Night), (Some(Kind::Train), 1));
        // Dawn came: this train is not heard, and it is not kept for later.
        assert_eq!(train(Period::Between), (None, 1));

        let plane = |period, weather| {
            Plan {
                kind: Kind::Plane,
                times: vec![30.0],
                forced: false,
            }
            .due(31.0, period, weather)
        };
        assert_eq!(plane(Period::Day, true), Some(Kind::Plane));
        assert_eq!(plane(Period::Day, false), None);
        assert_eq!(plane(Period::Night, true), None);
    }

    #[test]
    fn kinds_are_known_by_name() {
        assert_eq!(Kind::named("train"), Some(Kind::Train));
        assert_eq!(Kind::named("plane"), Some(Kind::Plane));
        assert_eq!(Kind::named("boat"), None);
        assert_eq!(Kind::Train.milliseconds(), 44_000);
        assert_eq!(Kind::Plane.milliseconds(), 30_000);
    }
}
