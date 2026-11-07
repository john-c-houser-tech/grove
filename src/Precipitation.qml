import QtQuick
import QtQuick.Particles

// Rain and fog, in front of the plant. Rain falls as streaks; fog is a veil
// that thickens toward the ground, with wisps drifting through it.
Item {
    id: root

    property real rain: 0
    property real fog: 0
    property real snow: 0
    property real ambient: 1
    // Fog is grey-blue in the dark and pale grey by day.
    readonly property color fogColor: Qt.rgba(0.2 + 0.58 * ambient, 0.23 + 0.58 * ambient, 0.28 + 0.56 * ambient, 1)

    ParticleSystem {
        id: system
        running: root.rain > 0.02
    }
    Emitter {
        system: system
        x: -80
        y: -30
        width: root.width + 160
        height: 1
        enabled: root.rain > 0.02
        emitRate: 420 * root.rain
        lifeSpan: 1000
        velocity: AngleDirection { angle: 98; angleVariation: 3; magnitude: 880; magnitudeVariation: 120 }
        size: 1
    }
    ItemParticle {
        system: system
        delegate: Rectangle {
            width: 1.5
            height: 14
            rotation: 8
            color: "#b4cde0"
            opacity: 0.45
        }
    }

    // Snow drifts down and sways, a few large flakes among many small ones.
    ParticleSystem {
        id: snowSystem
        running: root.snow > 0.02
    }
    Emitter {
        system: snowSystem
        x: -40
        y: -20
        width: root.width + 80
        height: 1
        enabled: root.snow > 0.02
        emitRate: 50 * root.snow
        lifeSpan: 8000
        velocity: AngleDirection { angle: 90; angleVariation: 8; magnitude: 100; magnitudeVariation: 30 }
        size: 1
    }
    Wander {
        system: snowSystem
        xVariance: 45
        pace: 90
    }
    ItemParticle {
        system: snowSystem
        delegate: Rectangle {
            width: 3 + Math.random() * 5
            height: width
            radius: width / 2
            color: root.ambient > 0.5 ? "#ffffff" : "#c9d6ee"
            opacity: 0.55 + Math.random() * 0.4
        }
    }

    // Veil: light overhead, heavy near the ground.
    Rectangle {
        anchors.fill: parent
        opacity: root.fog
        gradient: Gradient {
            GradientStop { position: 0.0; color: Qt.rgba(root.fogColor.r, root.fogColor.g, root.fogColor.b, 0.18) }
            GradientStop { position: 0.6; color: Qt.rgba(root.fogColor.r, root.fogColor.g, root.fogColor.b, 0.5) }
            GradientStop { position: 1.0; color: Qt.rgba(root.fogColor.r, root.fogColor.g, root.fogColor.b, 0.7) }
        }
    }

    // Wisps drifting along the ground.
    property real drift: 0
    NumberAnimation on drift {
        from: 0
        to: 1
        duration: 420000
        loops: Animation.Infinite
        running: root.fog > 0.02
    }
    Repeater {
        model: 5
        delegate: Rectangle {
            readonly property real a: Math.abs(Math.sin(index * 31.9 + 0.7))
            readonly property real w: root.width * (0.5 + 0.4 * a)
            width: w
            height: root.height * 0.07
            radius: height / 2
            y: root.height * (0.55 + 0.3 * ((index * 0.37) % 1))
            x: ((a + root.drift * (4 + index * 2)) % 1) * (root.width + w) - w
            color: root.fogColor
            opacity: root.fog * 0.24
        }
    }
}
