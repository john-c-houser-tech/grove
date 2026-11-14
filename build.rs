//! Builds the Grove QML module: the Rust types QML uses, and the QML itself.

use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("Grove")
            .version(1, 0)
            .qml_files([
                "src/CloudShape.qml",
                "src/Clouds.qml",
                "src/GardenView.qml",
                "src/GrowingPlant.qml",
                "src/Main.qml",
                "src/Plane.qml",
                "src/Precipitation.qml",
                "src/SkyBackdrop.qml",
                "src/SoftGlow.qml",
                "src/Train.qml",
            ])
            .depend("QtQuick"),
    )
    // Every file with a bridge in it: one left out compiles, but QML never
    // hears of its types.
    .files([
        "src/qml/backend.rs",
        "src/qml/garden.rs",
        "src/qml/layers.rs",
        "src/qml/light.rs",
        "src/qml/notifier.rs",
        "src/qml/paint.rs",
        "src/qml/script.rs",
        "src/qml/sounds.rs",
        "src/qml/testing.rs",
        "src/qml/theme.rs",
        "src/qml/translator.rs",
        "src/qml/weather.rs",
    ])
    // The pictures the rain and snow are made of, beside the QML that names
    // them.
    .qrc_resources(["src/images/rain.png", "src/images/snow.png"])
    .qt_module("Quick")
    // Only for a header: the scene tests send keys the way Qt's own tests do.
    .qt_module("Test")
    .build();
}
