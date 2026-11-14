//! The desktop's look, as the Grove QML module's `Theme` singleton, followed
//! live. The rules are in `../theme.rs` and the watching in `../follow.rs`.

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
        #[qproperty(QString, accent, READ = accent, NOTIFY = changed)]
        #[qproperty(QString, accent_foreground, cxx_name = "accentForeground", READ = accent_foreground, NOTIFY = changed)]
        #[qproperty(f64, radius, READ = radius, NOTIFY = changed)]
        type Theme = super::ThemeRust;

        /// The theme's accent as #rrggbb.
        fn accent(self: &Theme) -> QString;
        /// The black or white that reads on the accent.
        #[cxx_name = "accentForeground"]
        fn accent_foreground(self: &Theme) -> QString;
        /// The corner radius, in pixels, as Hyprland rounds windows.
        fn radius(self: &Theme) -> f64;

        /// The accent or the radius changed.
        #[qsignal]
        fn changed(self: Pin<&mut Theme>);
    }

    impl cxx_qt::Threading for Theme {}
    impl cxx_qt::Initialize for Theme {}
}

use core::pin::Pin;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;

use super::shared;
use crate::app::Event;
use crate::follow::{follow, Following};

/// The `Theme` QObject keeps the threads that follow the desktop for it; they
/// stop when it goes.
#[derive(Default)]
pub struct ThemeRust {
    following: Option<Following>,
}

impl cxx_qt::Initialize for qobject::Theme {
    fn initialize(mut self: Pin<&mut Self>) {
        shared::started("Theme");
        let gui = self.qt_thread();
        shared::listen(move |event| {
            if event == Event::Theme {
                let _ = gui.queue(|theme| theme.changed());
            }
        });
        let (directory, hyprland) =
            shared::read(|app| (app.options().theme_directory.clone(), app.options().hyprland.clone()));
        // The threads only carry the news here; the app is the GUI thread's.
        let (files, corners) = (self.qt_thread(), self.qt_thread());
        let following = follow(
            directory,
            hyprland,
            move || files.queue(|_| shared::update(|app| app.reload_theme())).is_ok(),
            move |radius| {
                corners
                    .queue(move |_| shared::update(|app| app.set_radius(radius)))
                    .is_ok()
            },
        );
        self.as_mut().rust_mut().following = Some(following);
    }
}

impl qobject::Theme {
    pub fn accent(&self) -> QString {
        QString::from(shared::read(|app| app.theme().accent().to_string()).as_str())
    }

    pub fn accent_foreground(&self) -> QString {
        QString::from(shared::read(|app| app.theme().accent_foreground()))
    }

    pub fn radius(&self) -> f64 {
        f64::from(shared::read(|app| app.theme().radius()))
    }
}
