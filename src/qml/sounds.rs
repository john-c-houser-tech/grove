//! The distant sounds, as the Grove QML module's `Sounds` singleton. What
//! happens when is decided in `../sounds.rs` and by the app; the playing is
//! `../player.rs`'s, on its own thread. Both sounds are also seen: this file
//! tells the window when the plane crosses the sky and the train the plain.

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
        #[qproperty(bool, muted, READ = muted, WRITE = set_muted, NOTIFY = muted_changed)]
        #[qproperty(QString, problem, READ = problem, NOTIFY = problem_changed)]
        type Sounds = super::SoundsRust;

        /// Muting silences the sounds but keeps the train and plane in the
        /// picture; the choice is remembered.
        fn muted(self: &Sounds) -> bool;
        #[cxx_name = "setMuted"]
        fn set_muted(self: Pin<&mut Sounds>, muted: bool);
        /// Empty when sound works, otherwise why it cannot.
        fn problem(self: &Sounds) -> QString;

        #[qsignal]
        #[cxx_name = "mutedChanged"]
        fn muted_changed(self: Pin<&mut Sounds>);
        /// The problem changed, or the language did and it reads differently.
        #[qsignal]
        #[cxx_name = "problemChanged"]
        fn problem_changed(self: Pin<&mut Sounds>);
        /// A plane crosses the sky: direction is 1 for left to right and -1
        /// for the other way, and altitude from 0 to 1 is how high it flies.
        #[qsignal]
        #[cxx_name = "planeFlyover"]
        fn plane_flyover(self: Pin<&mut Sounds>, direction: i32, milliseconds: i32, altitude: f64);
        /// A train runs across the valley, timed to the sound that lasts
        /// this long.
        #[qsignal]
        #[cxx_name = "trainPassing"]
        fn train_passing(self: Pin<&mut Sounds>, milliseconds: i32);
        /// The session has ended, so a train or plane still passing fades
        /// away with its sound.
        #[qsignal]
        #[cxx_name = "fadingOut"]
        fn fading_out(self: Pin<&mut Sounds>);
    }

    impl cxx_qt::Threading for Sounds {}
    impl cxx_qt::Initialize for Sounds {}
}

use core::pin::Pin;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;

use super::shared::{self, tr};
use crate::app::Event;
use crate::player::{Player, Problem, Report};
use crate::session::State;
use crate::sounds::Kind;

/// What the `Sounds` QObject keeps: the player, whose thread ends with it.
#[derive(Default)]
pub struct SoundsRust {
    player: Option<Player>,
}

impl cxx_qt::Initialize for qobject::Sounds {
    fn initialize(mut self: Pin<&mut Self>) {
        shared::started("Sounds");
        // The player reports from its own thread; the news is carried here.
        let gui = self.qt_thread();
        let player = Player::new(move |report| {
            let _ = gui.queue(move |sounds| sounds.reported(report));
        });
        player.set_muted(shared::read(|app| app.muted()));
        self.as_mut().rust_mut().player = Some(player);

        let gui = self.qt_thread();
        shared::listen(move |event| {
            let _ = match event {
                Event::SoundComing(kind) => gui.queue(move |sounds| sounds.with_player(|player| player.prepare(kind))),
                Event::Sound(kind) => gui.queue(move |sounds| sounds.sound(kind)),
                Event::SoundsEnded => gui.queue(|sounds| sounds.ended()),
                Event::Muted => gui.queue(|sounds| sounds.muted_changed()),
                Event::SoundProblem | Event::Language => gui.queue(|sounds| sounds.problem_changed()),
                _ => Ok(()),
            };
        });
    }
}

impl qobject::Sounds {
    pub fn muted(&self) -> bool {
        shared::read(|app| app.muted())
    }

    pub fn set_muted(self: Pin<&mut Self>, muted: bool) {
        self.as_ref().with_player(|player| player.set_muted(muted));
        shared::update(|app| app.set_muted(muted));
    }

    pub fn problem(&self) -> QString {
        let words = match shared::read(|app| app.sound_problem()) {
            None => String::new(),
            Some(Problem::NoOutput) => tr("No audio output found, so the distant sounds are silent."),
            Some(Problem::Unusable) => tr("The audio output could not be used, so the distant sounds are silent."),
        };
        QString::from(words.as_str())
    }

    fn with_player(&self, ask: impl FnOnce(&Player)) {
        if let Some(player) = &self.rust().player {
            ask(player);
        }
    }

    /// A sound is due. Muted, only its picture goes; otherwise the picture
    /// starts when the player says the sound has.
    fn sound(self: Pin<&mut Self>, kind: Kind) {
        if self.muted() {
            self.show(kind);
        } else {
            let volume = shared::read(|app| app.volume(kind));
            self.as_ref().with_player(|player| player.play(kind, volume));
        }
    }

    /// The session's sounds are over.
    fn ended(self: Pin<&mut Self>) {
        self.as_ref().with_player(Player::end_session);
        self.fading_out();
    }

    /// Takes the player's word on a sound: whether or not it can be heard,
    /// what goes with it is shown, starting together with it. A sound whose
    /// session ended while its output was being opened is too late for
    /// both: it is faded out, and nothing crosses the idle window.
    fn reported(self: Pin<&mut Self>, report: Report) {
        shared::update(|app| app.sound_reported(report));
        let growing = shared::read(|app| app.session().state()) == State::Growing;
        match report {
            Report::Started(kind) | Report::Failed(kind, _) | Report::Muted(kind) if growing => self.show(kind),
            Report::Started(_) => self.as_ref().with_player(Player::end_session),
            Report::Failed(..) | Report::Muted(_) | Report::Broke(_) => {}
        }
    }

    /// Shows what goes with a sound.
    fn show(self: Pin<&mut Self>, kind: Kind) {
        match kind {
            Kind::Plane => {
                let direction = if fastrand::bool() { 1 } else { -1 };
                self.plane_flyover(direction, kind.milliseconds(), fastrand::f64());
            }
            Kind::Train => self.train_passing(kind.milliseconds()),
        }
    }
}
