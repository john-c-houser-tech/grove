//! The light in the garden, as the Grove QML module's `Light` singleton. The
//! rules are in `../light.rs`; this file reads the clock twice a minute.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qcolor.h");
        type QColor = cxx_qt_lib::QColor;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(QColor, sky_top, cxx_name = "skyTop", READ = sky_top, NOTIFY = changed)]
        #[qproperty(QColor, sky_bottom, cxx_name = "skyBottom", READ = sky_bottom, NOTIFY = changed)]
        #[qproperty(f64, elevation, READ = elevation, NOTIFY = changed)]
        #[qproperty(f64, sun_x, cxx_name = "sunX", READ = sun_x, NOTIFY = changed)]
        #[qproperty(f64, moon_x, cxx_name = "moonX", READ = moon_x, NOTIFY = changed)]
        #[qproperty(f64, moon_elevation, cxx_name = "moonElevation", READ = moon_elevation, NOTIFY = changed)]
        #[qproperty(bool, sun_up, cxx_name = "sunUp", READ = sun_up, NOTIFY = changed)]
        #[qproperty(f64, ambient, READ = ambient, NOTIFY = changed)]
        #[qproperty(f64, stars, READ = stars, NOTIFY = changed)]
        #[qproperty(f64, warmth, READ = warmth, NOTIFY = changed)]
        #[qproperty(QColor, warm_tint, cxx_name = "warmTint", READ = warm_tint, NOTIFY = changed)]
        #[qproperty(QColor, night_tint, cxx_name = "nightTint", READ = night_tint, NOTIFY = changed)]
        type Light = super::LightRust;

        // The sky now, field by field; see `Sky` for what each one means.
        #[cxx_name = "skyTop"]
        fn sky_top(self: &Light) -> QColor;
        #[cxx_name = "skyBottom"]
        fn sky_bottom(self: &Light) -> QColor;
        fn elevation(self: &Light) -> f64;
        #[cxx_name = "sunX"]
        fn sun_x(self: &Light) -> f64;
        #[cxx_name = "moonX"]
        fn moon_x(self: &Light) -> f64;
        #[cxx_name = "moonElevation"]
        fn moon_elevation(self: &Light) -> f64;
        #[cxx_name = "sunUp"]
        fn sun_up(self: &Light) -> bool;
        fn ambient(self: &Light) -> f64;
        fn stars(self: &Light) -> f64;
        fn warmth(self: &Light) -> f64;
        /// The golden colour, with an alpha that rises with warmth.
        #[cxx_name = "warmTint"]
        fn warm_tint(self: &Light) -> QColor;
        /// A translucent blue that dims whatever it is applied to at night.
        #[cxx_name = "nightTint"]
        fn night_tint(self: &Light) -> QColor;

        /// The sky has moved on, as it does twice a minute, or the accent
        /// changed.
        #[qsignal]
        fn changed(self: Pin<&mut Light>);
    }

    impl cxx_qt::Threading for Light {}
    impl cxx_qt::Initialize for Light {}
}

use core::pin::Pin;
use std::time::Duration;

use cxx_qt::Threading;
use cxx_qt_lib::QColor;

use super::color::qcolor;
use super::shared;
use crate::app::Event;

/// The sky moves slowly; twice a minute is smooth enough to never notice.
const REFRESH: Duration = Duration::from_secs(30);

/// The `Light` QObject keeps nothing itself: the sky is the app's.
#[derive(Default)]
pub struct LightRust;

impl cxx_qt::Initialize for qobject::Light {
    fn initialize(self: Pin<&mut Self>) {
        let gui = self.qt_thread();
        shared::listen(move |event| {
            if event == Event::Light {
                let _ = gui.queue(|light| light.changed());
            }
        });
        // A thread that asks the GUI thread to read the clock again, for as
        // long as there is a Light to ask. A frozen sky comes out the same.
        let gui = self.qt_thread();
        std::thread::spawn(move || loop {
            std::thread::sleep(REFRESH);
            if gui.queue(|_| shared::update(|app| app.refresh_light())).is_err() {
                break;
            }
        });
    }
}

impl qobject::Light {
    pub fn sky_top(&self) -> QColor {
        qcolor(shared::read(|app| app.sky().top))
    }

    pub fn sky_bottom(&self) -> QColor {
        qcolor(shared::read(|app| app.sky().bottom))
    }

    pub fn elevation(&self) -> f64 {
        shared::read(|app| app.sky().elevation)
    }

    pub fn sun_x(&self) -> f64 {
        shared::read(|app| app.sky().sun_x)
    }

    pub fn moon_x(&self) -> f64 {
        shared::read(|app| app.sky().moon_x)
    }

    pub fn moon_elevation(&self) -> f64 {
        shared::read(|app| app.sky().moon_elevation)
    }

    pub fn sun_up(&self) -> bool {
        shared::read(|app| app.sky().sun_up)
    }

    pub fn ambient(&self) -> f64 {
        shared::read(|app| app.sky().ambient)
    }

    pub fn stars(&self) -> f64 {
        shared::read(|app| app.sky().stars)
    }

    pub fn warmth(&self) -> f64 {
        shared::read(|app| app.sky().warmth)
    }

    pub fn warm_tint(&self) -> QColor {
        qcolor(shared::read(|app| app.sky().warm_tint()))
    }

    pub fn night_tint(&self) -> QColor {
        qcolor(shared::read(|app| app.sky().night_tint()))
    }
}
