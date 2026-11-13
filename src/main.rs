//! grove — a focus timer that grows a plant while you work.

mod app;
mod color;
mod easing;
mod light;
mod options;
mod qml;
mod session;
mod weather;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};

fn main() {
    options::install(options::Options::from_environment());

    let mut application = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/Grove/src/Port.qml"));
    }
    if let Some(application) = application.as_mut() {
        application.exec();
    }
}
