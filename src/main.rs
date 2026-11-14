//! grove — a focus timer that grows a plant while you work.

mod app;
mod color;
mod easing;
mod follow;
mod garden;
mod language;
mod light;
mod notifier;
mod options;
mod player;
mod qml;
mod session;
mod settings;
mod sounds;
mod synth;
mod theme;
mod weather;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};

fn main() {
    options::install(options::Options::from_environment());

    let mut application = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/Grove/src/qml/Port.qml"));
    }
    if let Some(application) = application.as_mut() {
        application.exec();
    }
}
