//! The sky's mood, as the Grove QML module's `Weather` singleton. The rules
//! are in `../weather.rs`, and the app moves it along with the session.

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
        #[qproperty(QString, kind, READ = kind, NOTIFY = changed)]
        #[qproperty(QString, season, READ = season, NOTIFY = changed)]
        #[qproperty(f64, cloud, READ = cloud, NOTIFY = changed)]
        #[qproperty(f64, rain, READ = rain, NOTIFY = changed)]
        #[qproperty(f64, snow, READ = snow, NOTIFY = changed)]
        #[qproperty(f64, fog, READ = fog, NOTIFY = changed)]
        #[qproperty(f64, growth, READ = growth, NOTIFY = growth_changed)]
        type Weather = super::WeatherRust;

        /// The dominant weather right now: "clear", "cloudy", "rain", "snow"
        /// or "fog".
        fn kind(self: &Weather) -> QString;
        /// "spring", "summer", "autumn" or "winter", from the session's date.
        fn season(self: &Weather) -> QString;
        // How much of each there is now, 0 to 1.
        fn cloud(self: &Weather) -> f64;
        fn rain(self: &Weather) -> f64;
        fn snow(self: &Weather) -> f64;
        fn fog(self: &Weather) -> f64;
        /// Session progress as the plant shows it, 0 to 1.
        fn growth(self: &Weather) -> f64;

        /// The sky has changed enough to see, or settled.
        #[qsignal]
        fn changed(self: Pin<&mut Weather>);
        /// The plant's growth has changed.
        #[qsignal]
        #[cxx_name = "growthChanged"]
        fn growth_changed(self: Pin<&mut Weather>);
    }

    impl cxx_qt::Threading for Weather {}
    impl cxx_qt::Initialize for Weather {}
}

use core::pin::Pin;

use cxx_qt::Threading;
use cxx_qt_lib::QString;

use super::shared;
use crate::app::Event;

/// The `Weather` QObject keeps nothing itself: the weather is the app's.
#[derive(Default)]
pub struct WeatherRust;

impl cxx_qt::Initialize for qobject::Weather {
    fn initialize(self: Pin<&mut Self>) {
        let gui = self.qt_thread();
        shared::listen(move |event| {
            let _ = match event {
                Event::Weather => gui.queue(|weather| weather.changed()),
                Event::Growth => gui.queue(|weather| weather.growth_changed()),
                _ => Ok(()),
            };
        });
    }
}

impl qobject::Weather {
    pub fn kind(&self) -> QString {
        QString::from(shared::read(|app| app.weather().kind().name()))
    }

    pub fn season(&self) -> QString {
        QString::from(shared::read(|app| app.weather().season().name()))
    }

    pub fn cloud(&self) -> f64 {
        shared::read(|app| app.weather().now().cloud)
    }

    pub fn rain(&self) -> f64 {
        shared::read(|app| app.weather().now().rain)
    }

    pub fn snow(&self) -> f64 {
        shared::read(|app| app.weather().now().snow)
    }

    pub fn fog(&self) -> f64 {
        shared::read(|app| app.weather().now().fog)
    }

    pub fn growth(&self) -> f64 {
        shared::read(|app| app.weather().growth())
    }
}
