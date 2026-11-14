import QtQuick
import Grove

// Scaffolding for the Rust port: proves QML can read and drive the Rust
// singletons. It starts a session, reports what it sees and quits when the
// session is done. Main.qml replaces it once every singleton is ported.
QtObject {
    id: port

    property int growthSteps: 0
    property Timer leave: Timer { interval: 100; onTriggered: Qt.quit() }

    property Connections session: Connections {
        target: Backend
        function onStateChanged() {
            console.log("state", Backend.state, "progress", Backend.progress, "growth", Math.round(Weather.growth * 1000) / 1000,
                        "growth steps", port.growthSteps)
            // A moment for the garden to show the new plant before leaving.
            if (Backend.state === "done")
                port.leave.start()
        }
    }
    property Connections growth: Connections {
        target: Weather
        function onGrowthChanged() { port.growthSteps++ }
        function onChanged() { console.log("weather", Weather.kind, Weather.season, "rain", Weather.rain) }
    }

    // The garden as a view sees it: its rows, through a delegate.
    property Instantiator rows: Instantiator {
        model: Garden
        delegate: QtObject {
            required property int seed
            required property string outcome
            required property string weather
            required property real progress
            required property date when
            required property int minutes
            Component.onCompleted: console.log("plant", outcome, weather, progress, minutes, "min", when.getFullYear() > 2000)
        }
    }
    property Connections garden: Connections {
        target: Garden
        function onCountsChanged() { console.log("garden", Garden.grown, "grown", Garden.wilted, "wilted", Garden.focusedMinutes, "min") }
        function onProblemChanged() { console.log("garden problem:", Garden.problem) }
    }

    property Connections theme: Connections {
        target: Theme
        function onChanged() { console.log("theme", Theme.accent, Theme.accentForeground, "radius", Theme.radius) }
    }

    property Connections language: Connections {
        target: Translator
        function onChanged() { console.log("language", Translator.language, "rtl", Translator.rightToLeft, Qt.application.layoutDirection, port.said) }
    }
    // Retranslated by the engine when the language changes.
    property string said: qsTr("Garden") + " / " + qsTr("%n minutes", "", 25) + " / " + Number(1234.5).toLocaleString(Qt.locale(Translator.locale))

    Component.onCompleted: {
        console.log("language", Translator.language, "rtl", Translator.rightToLeft, port.said)
        console.log("theme", Theme.accent, Theme.accentForeground, "radius", Theme.radius)
        console.log("garden", Garden.grown, "grown", Garden.wilted, "wilted", "problem:", Garden.problem)
        console.log("state", Backend.state, "seconds", Backend.seconds, "remaining", Backend.remaining)
        console.log("light", Light.skyTop, Light.sunUp, "ambient", Math.round(Light.ambient * 100) / 100, "night", Light.nightTint)
        console.log("weather", Weather.kind, Weather.season)
        Backend.start()
    }
}
