import QtQuick
import Grove

// Scaffolding for the Rust port: proves QML can read and drive the Rust
// singletons. It starts a session, reports what it sees and quits when the
// session is done. Main.qml replaces it once every singleton is ported.
QtObject {
    id: port

    property int growthSteps: 0

    property Connections session: Connections {
        target: Backend
        function onStateChanged() {
            console.log("state", Backend.state, "progress", Backend.progress, "growth", Math.round(Weather.growth * 1000) / 1000,
                        "growth steps", port.growthSteps)
            if (Backend.state === "done")
                Qt.quit()
        }
    }
    property Connections growth: Connections {
        target: Weather
        function onGrowthChanged() { port.growthSteps++ }
        function onChanged() { console.log("weather", Weather.kind, Weather.season, "rain", Weather.rain) }
    }

    Component.onCompleted: {
        console.log("state", Backend.state, "seconds", Backend.seconds, "remaining", Backend.remaining)
        console.log("light", Light.skyTop, Light.sunUp, "ambient", Math.round(Light.ambient * 100) / 100, "night", Light.nightTint)
        console.log("weather", Weather.kind, Weather.season)
        Backend.start()
    }
}
