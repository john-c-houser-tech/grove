//! Builds the Grove QML module: the Rust types QML uses, and the QML itself.

use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("Grove")
            .version(1, 0)
            // Scaffolding for the port: a window-less stand-in for Main.qml,
            // which needs every singleton and takes its place once they exist.
            .qml_file("src/Port.qml")
            .depend("QtQuick"),
    )
    .files([
        "src/qml/backend.rs",
        "src/qml/garden.rs",
        "src/qml/light.rs",
        "src/qml/theme.rs",
        "src/qml/weather.rs",
    ])
    .qt_module("Quick")
    .build();
}
