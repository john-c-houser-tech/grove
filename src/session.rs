//! One focus session at a time. The plant on screen is a pure function of
//! `progress` and `seed`, so everything the window shows comes from here.
//! Nothing in this file knows about Qt; `backend.rs` shows it to QML.

/// Seeds are drawn from 1 up to this, 2^30.
pub const SEED_LIMIT: i32 = 0x4000_0000;

/// Where a session stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Idle,
    Growing,
    Done,
    Wilted,
}

impl State {
    /// The name QML knows the state by.
    pub fn name(self) -> &'static str {
        match self {
            State::Idle => "idle",
            State::Growing => "growing",
            State::Done => "done",
            State::Wilted => "wilted",
        }
    }
}

/// What a call changed that the window should hear about.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Changes {
    /// A session started, finished or was given up.
    pub state: bool,
    /// The countdown moved on a second, or the plant grew enough to show
    /// (0.2% of a session); not on every tick, which no one could see.
    pub progress: bool,
}

/// A session and its clock.
#[derive(Clone, Debug)]
pub struct Session {
    seconds: i32,
    state: State,
    elapsed_ms: i64,
    progress: f64,
    // What the window was last told, so ticks too small to see go unannounced.
    shown_remaining: i32,
    shown_progress: f64,
    seed: i32,
}

impl Session {
    /// `seconds` is the session length; tests and GROVE_SECONDS shorten it.
    pub fn new(seconds: i32) -> Self {
        Self {
            seconds: seconds.max(1),
            state: State::Idle,
            elapsed_ms: 0,
            progress: 0.0,
            shown_remaining: 0,
            shown_progress: 0.0,
            seed: 1,
        }
    }

    pub fn state(&self) -> State {
        self.state
    }

    /// How far through the session, from 0 to 1.
    pub fn progress(&self) -> f64 {
        self.progress
    }

    /// Whole seconds left, rounded up, so the countdown never shows 0 early.
    pub fn remaining(&self) -> i32 {
        let left = (self.total_ms() - self.elapsed_ms).max(0);
        ((left + 999) / 1000) as i32
    }

    /// The session's full length.
    pub fn minutes(&self) -> i32 {
        self.seconds / 60
    }

    pub fn seconds(&self) -> i32 {
        self.seconds
    }

    /// The plant's seed, new each session, so no two plants are alike.
    pub fn seed(&self) -> i32 {
        self.seed
    }

    /// Starts a session growing the plant of `seed`, unless one is already
    /// growing.
    pub fn start(&mut self, seed: i32) -> Changes {
        if self.state == State::Growing {
            return Changes::default();
        }
        self.elapsed_ms = 0;
        self.progress = 0.0;
        self.seed = seed;
        self.shown_remaining = self.remaining();
        self.shown_progress = 0.0;
        self.state = State::Growing;
        Changes {
            state: true,
            progress: true,
        }
    }

    /// Giving up early leaves a wilted plant behind.
    pub fn give_up(&mut self) -> Changes {
        if self.state != State::Growing {
            return Changes::default();
        }
        self.state = State::Wilted;
        Changes {
            state: true,
            progress: false,
        }
    }

    /// Advances the session clock by real time that has passed.
    pub fn advance(&mut self, milliseconds: i64) -> Changes {
        if self.state != State::Growing {
            return Changes::default();
        }
        let mut changes = Changes::default();
        self.elapsed_ms += milliseconds.max(0);
        let total = self.total_ms();
        self.progress = (self.elapsed_ms as f64 / total as f64).min(1.0);
        // A tick grows the plant by a thirty-thousandth of a full session, far
        // too little to see, and every listener redraws for it. So the window
        // hears only when the countdown moves on a second or the plant has
        // grown enough to show, which in a short test session is still many
        // times a second.
        let left = self.remaining();
        if left != self.shown_remaining || self.progress - self.shown_progress >= 0.002 || self.elapsed_ms >= total {
            self.shown_remaining = left;
            self.shown_progress = self.progress;
            changes.progress = true;
        }
        if self.elapsed_ms >= total {
            self.state = State::Done;
            changes.state = true;
        }
        changes
    }

    fn total_ms(&self) -> i64 {
        i64::from(self.seconds) * 1000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn growing(seconds: i32) -> Session {
        let mut session = Session::new(seconds);
        session.start(7);
        session
    }

    /// How many of `ticks` ticks of `ms` announce progress.
    fn announced(session: &mut Session, ticks: usize, ms: i64) -> usize {
        (0..ticks).filter(|_| session.advance(ms).progress).count()
    }

    #[test]
    fn starts_idle() {
        let session = Session::new(10);
        assert_eq!(session.state(), State::Idle);
        assert_eq!(session.progress(), 0.0);
    }

    #[test]
    fn grows_with_time() {
        let mut session = growing(10);
        assert_eq!(session.state(), State::Growing);
        assert_eq!(session.seed(), 7);
        session.advance(2500);
        assert_eq!(session.progress(), 0.25);
        assert_eq!(session.remaining(), 8);
    }

    #[test]
    fn finishes_when_the_time_is_up() {
        let mut session = growing(10);
        let changes = session.advance(10_000);
        assert!(changes.state);
        assert_eq!(session.state(), State::Done);
        assert_eq!(session.progress(), 1.0);
        assert_eq!(session.remaining(), 0);
    }

    #[test]
    fn giving_up_wilts_the_plant() {
        let mut session = growing(10);
        session.advance(4000);
        assert!(session.give_up().state);
        assert_eq!(session.state(), State::Wilted);
        // The plant keeps the size it reached, and time stops.
        session.advance(4000);
        assert_eq!(session.progress(), 0.4);
    }

    #[test]
    fn give_up_does_nothing_when_idle() {
        let mut session = Session::new(10);
        assert_eq!(session.give_up(), Changes::default());
        assert_eq!(session.state(), State::Idle);
    }

    #[test]
    fn starting_again_resets() {
        let mut session = growing(10);
        session.advance(5000);
        session.give_up();
        session.start(8);
        assert_eq!(session.state(), State::Growing);
        assert_eq!(session.progress(), 0.0);
        assert_eq!(session.seed(), 8);
    }

    #[test]
    fn starting_while_growing_keeps_the_session() {
        let mut session = growing(10);
        session.advance(5000);
        assert_eq!(session.start(8), Changes::default());
        assert_eq!(session.progress(), 0.5);
        assert_eq!(session.seed(), 7);
    }

    #[test]
    fn full_sessions_announce_progress_once_a_second() {
        let mut session = growing(25 * 60);
        // A second of 50 ms ticks: only the countdown moving on is worth a redraw.
        assert_eq!(announced(&mut session, 20, 50), 1);
        assert_eq!(session.remaining(), 25 * 60 - 1);
    }

    #[test]
    fn short_sessions_still_grow_smoothly() {
        let mut session = growing(10);
        assert_eq!(announced(&mut session, 20, 50), 20);
    }

    #[test]
    fn the_end_of_a_session_is_always_announced() {
        let mut session = growing(25 * 60);
        session.advance(25 * 60 * 1000 - 10);
        assert_eq!(announced(&mut session, 1, 10), 1);
        assert_eq!(session.progress(), 1.0);
    }
}
