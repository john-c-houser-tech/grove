//! The session, as the Grove QML module's `Backend` singleton. The rules are
//! in `../session.rs`; this file gives them a clock and tells QML what changed.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(QString, state, READ = state, NOTIFY = state_changed)]
        #[qproperty(f64, progress, READ = progress, NOTIFY = progress_changed)]
        #[qproperty(i32, remaining, READ = remaining, NOTIFY = progress_changed)]
        #[qproperty(i32, minutes, READ = minutes, CONSTANT)]
        #[qproperty(i32, seconds, READ = seconds, CONSTANT)]
        #[qproperty(i32, seed, READ = seed, NOTIFY = state_changed)]
        type Backend = super::BackendRust;

        /// "idle", "growing", "done" or "wilted".
        fn state(self: &Backend) -> QString;
        /// How far through the session, from 0 to 1.
        fn progress(self: &Backend) -> f64;
        /// Whole seconds left, rounded up.
        fn remaining(self: &Backend) -> i32;
        /// The session's full length.
        fn minutes(self: &Backend) -> i32;
        fn seconds(self: &Backend) -> i32;
        /// The plant's seed, new each session.
        fn seed(self: &Backend) -> i32;

        /// A session started, finished or was given up.
        #[qsignal]
        #[cxx_name = "stateChanged"]
        fn state_changed(self: Pin<&mut Backend>);
        /// The countdown moved on a second, or the plant grew enough to show.
        #[qsignal]
        #[cxx_name = "progressChanged"]
        fn progress_changed(self: Pin<&mut Backend>);

        /// Starts a session, unless one is already growing.
        #[qinvokable]
        fn start(self: Pin<&mut Backend>);
        /// Giving up early leaves a wilted plant behind.
        #[qinvokable]
        #[cxx_name = "giveUp"]
        fn give_up(self: Pin<&mut Backend>);
    }

    impl cxx_qt::Threading for Backend {}
    impl cxx_qt::Initialize for Backend {}
}

use core::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;

use super::shared;
use crate::app::Event;
use crate::session::{State, SEED_LIMIT};

/// How often the session clock is read while a plant grows.
const TICK: Duration = Duration::from_millis(50);

/// What the `Backend` QObject keeps on the Rust side.
pub struct BackendRust {
    /// When the session clock was last read.
    clock: Instant,
    /// Counts sessions, so the ticker of one that is over stops by itself.
    ticker: Arc<AtomicU64>,
}

impl Default for BackendRust {
    fn default() -> Self {
        Self {
            clock: Instant::now(),
            ticker: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl cxx_qt::Initialize for qobject::Backend {
    fn initialize(self: Pin<&mut Self>) {
        let gui = self.qt_thread();
        shared::listen(move |event| {
            // Queued rather than called, since the change may have come from
            // this object, which is then still borrowed. A Backend that is
            // gone has no one to tell.
            let _ = match event {
                Event::State => gui.queue(|backend| backend.state_changed()),
                Event::Progress => gui.queue(|backend| backend.progress_changed()),
                _ => Ok(()),
            };
        });
    }
}

impl qobject::Backend {
    pub fn state(&self) -> QString {
        QString::from(shared::read(|app| app.session().state().name()))
    }

    pub fn progress(&self) -> f64 {
        shared::read(|app| app.session().progress())
    }

    pub fn remaining(&self) -> i32 {
        shared::read(|app| app.session().remaining())
    }

    pub fn minutes(&self) -> i32 {
        shared::read(|app| app.session().minutes())
    }

    pub fn seconds(&self) -> i32 {
        shared::read(|app| app.session().seconds())
    }

    pub fn seed(&self) -> i32 {
        shared::read(|app| app.session().seed())
    }

    pub fn start(mut self: Pin<&mut Self>) {
        if shared::read(|app| app.session().state()) == State::Growing {
            return;
        }
        self.as_mut().rust_mut().clock = Instant::now();
        // A fresh seed per session, so no two plants are alike.
        shared::update(|app| app.start(fastrand::i32(1..SEED_LIMIT)));
        self.start_ticker();
    }

    pub fn give_up(self: Pin<&mut Self>) {
        shared::update(|app| app.give_up());
        self.retire_ticker();
    }

    /// Advances the session by the real time since the clock was last read.
    /// Real time rather than counted ticks, which drift under load.
    fn tick(mut self: Pin<&mut Self>) {
        let now = Instant::now();
        let mut rust = self.as_mut().rust_mut();
        let passed = now.duration_since(rust.clock).as_millis() as i64;
        // The clock moves on by what was counted, so the part of a
        // millisecond left over is counted next time, not lost: lost twenty
        // times a second, it would make a session end seconds late.
        rust.clock += Duration::from_millis(passed as u64);
        shared::update(|app| app.advance(passed));
        self.retire_ticker();
    }

    /// Stops the ticker once the session is no longer growing.
    fn retire_ticker(self: Pin<&mut Self>) {
        if shared::read(|app| app.session().state()) != State::Growing {
            self.rust().ticker.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// A thread that asks the GUI thread for a tick every `TICK` while this
    /// session grows. It holds nothing of the Backend but a way to queue work
    /// for it, and stops when the session is over or the Backend is gone.
    fn start_ticker(self: Pin<&mut Self>) {
        let counter = Arc::clone(&self.rust().ticker);
        let session = counter.fetch_add(1, Ordering::SeqCst) + 1;
        let gui = self.qt_thread();
        // A tick still waiting for the GUI thread is not joined by another:
        // each takes its time from the clock, so one is enough.
        let waiting = Arc::new(AtomicBool::new(false));
        std::thread::spawn(move || loop {
            std::thread::sleep(TICK);
            if counter.load(Ordering::SeqCst) != session {
                break;
            }
            if waiting.swap(true, Ordering::SeqCst) {
                continue;
            }
            let (counter, waiting) = (Arc::clone(&counter), Arc::clone(&waiting));
            let queued = gui.queue(move |backend| {
                waiting.store(false, Ordering::SeqCst);
                // A tick queued before the session ended is not for the next.
                if counter.load(Ordering::SeqCst) == session {
                    backend.tick();
                }
            });
            if queued.is_err() {
                break;
            }
        });
    }
}
