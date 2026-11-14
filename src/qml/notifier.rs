//! The desktop notification, as the Grove QML module's `Notifier` singleton.
//! The sending is in `../notifier.rs`; this file does it off the GUI thread
//! and says in the window when it cannot be done.

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
        #[qproperty(QString, problem, READ = problem, NOTIFY = problem_changed)]
        type Notifier = super::NotifierRust;

        /// Empty when notifications work, otherwise why they cannot.
        fn problem(self: &Notifier) -> QString;

        /// The problem changed, or the language did and it reads differently.
        #[qsignal]
        #[cxx_name = "problemChanged"]
        fn problem_changed(self: Pin<&mut Notifier>);
    }

    impl cxx_qt::Threading for Notifier {}
    impl cxx_qt::Initialize for Notifier {}
}

use core::pin::Pin;

use cxx_qt::Threading;
use cxx_qt_lib::QString;

use super::shared::{self, tr};
use crate::app::Event;
use crate::notifier::{send, Problem};

/// The `Notifier` QObject keeps nothing itself.
#[derive(Default)]
pub struct NotifierRust;

impl cxx_qt::Initialize for qobject::Notifier {
    fn initialize(self: Pin<&mut Self>) {
        shared::started("Notifier");
        let gui = self.qt_thread();
        shared::listen(move |event| {
            let _ = match event {
                Event::PlantGrown => gui.queue(|notifier| notifier.plant_grown()),
                Event::NotifierProblem | Event::Language => gui.queue(|notifier| notifier.problem_changed()),
                _ => Ok(()),
            };
        });
    }
}

impl qobject::Notifier {
    pub fn problem(&self) -> QString {
        let words = match shared::read(|app| app.notifier_problem()) {
            None => String::new(),
            Some(Problem::NoBus) => tr("No desktop session bus, so Grove cannot send notifications."),
            Some(Problem::NoService) => {
                tr("No notification service is running, so Grove cannot tell you when a plant is done.")
            }
        };
        QString::from(words.as_str())
    }

    /// Tells the desktop the plant is fully grown, from a thread of its own,
    /// so a slow or missing notification service never holds up the window.
    fn plant_grown(self: Pin<&mut Self>) {
        let (service, replaces) =
            shared::read(|app| (app.options().notification_service.clone(), app.last_notification()));
        let (summary, body) = (
            tr("Your plant is fully grown"),
            tr("Take a breath, then plant another."),
        );
        let gui = self.qt_thread();
        std::thread::spawn(move || {
            let outcome = send(&service, replaces, &summary, &body);
            let _ = gui.queue(move |_| shared::update(|app| app.notified(outcome)));
        });
    }
}
