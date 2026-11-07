pragma ComponentBehavior: Bound
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
    // The sky colour overhead (see Clouds.qml): fog is near-white by day and,
    // as the light fails, a grey that takes the sky's cool cast, never brown.
    property color sky: "#3d6f9c"
    readonly property real level: 0.26 + 0.66 * ambient
    readonly property real cast: Math.max(0, Math.min(0.6, 0.8 * (1 - ambient) / 0.78))
    readonly property real skyMean: (sky.r + sky.g + sky.b) / 3
    readonly property color fogColor: Qt.rgba(Math.max(0, Math.min(1, level * 0.97 + cast * (sky.r - skyMean))),
                                              Math.max(0, Math.min(1, level * 0.99 + cast * (sky.g - skyMean))),
                                              Math.max(0, Math.min(1, level + 0.02 + cast * (sky.b - skyMean))), 1)

    // An image particle blends its colour as though it were already multiplied
    // by its alpha, so it is given that way; a plain colour with an alpha comes
    // out nearly opaque. `c` must be a color, not a colour name, which has no
    // channels to scale.
    function premultiplied(c, alpha) { return Qt.rgba(c.r * alpha, c.g * alpha, c.b * alpha, 1) }
    readonly property color dropColor: "#b4cde0"
    readonly property color flakeColor: root.ambient > 0.5 ? "#ffffff" : "#c9d6ee"

    ParticleSystem {
        id: rainSystem
        running: root.rain > 0.02
    }
    Emitter {
        system: rainSystem
        x: -80
        y: -30
        width: root.width + 160
        height: 1
        enabled: root.rain > 0.02
        emitRate: 420 * root.rain
        lifeSpan: 1000
        velocity: AngleDirection { angle: 98; angleVariation: 3; magnitude: 880; magnitudeVariation: 120 }
        // The streak is 14 px long inside a 16 px square image.
        size: 16
    }
    // Image particles are drawn together by the GPU; an item per drop would be
    // hundreds of items alive at once, each one laid out and drawn on its own.
    ImageParticle {
        system: rainSystem
        source: "images/rain.png"
        rotation: 8
        color: root.premultiplied(root.dropColor, 0.45)
        alpha: 0.45
    }

    // Snow drifts down and sways, a few large flakes among many small ones.
    ParticleSystem {
        id: snowSystem
        running: root.snow > 0.02
    }
    // Faint flakes, middling ones and bright ones in equal numbers, so some
    // seem farther off. An image particle cannot vary its opacity flake by flake
    // (see premultiplied()), so each opacity is a group of its own.
    Repeater {
        model: [{ group: "faint", alpha: 0.5 }, { group: "middling", alpha: 0.7 }, { group: "bright", alpha: 0.9 }]
        delegate: Item {
            id: flakes
            required property var modelData
            Emitter {
                system: snowSystem
                group: flakes.modelData.group
                x: -40
                y: -20
                width: root.width + 80
                height: 1
                enabled: root.snow > 0.02
                emitRate: 50 / 3 * root.snow
                lifeSpan: 8000
                velocity: AngleDirection { angle: 90; angleVariation: 8; magnitude: 100; magnitudeVariation: 30 }
                // Flakes 2 to 6 px across.
                size: 4
                sizeVariation: 2
            }
            ImageParticle {
                system: snowSystem
                groups: [flakes.modelData.group]
                source: "images/snow.png"
                // A round flake looks the same turned any way, but turning puts it
                // on Qt's quad path. The plainer path sizes particles by the
                // screen's whole-number scale rather than the window's, so at 125%
                // every flake came out 1.6 times too big.
                autoRotation: true
                color: root.premultiplied(root.flakeColor, flakes.modelData.alpha)
                alpha: flakes.modelData.alpha
            }
        }
    }
    Wander {
        system: snowSystem
        xVariance: 45
        pace: 90
    }

    // Veil: light overhead, heavy near the ground.
    Rectangle {
        anchors.fill: parent
        opacity: root.fog
        gradient: Gradient {
            GradientStop { position: 0.0; color: Qt.alpha(root.fogColor, 0.18) }
            GradientStop { position: 0.6; color: Qt.alpha(root.fogColor, 0.5) }
            GradientStop { position: 1.0; color: Qt.alpha(root.fogColor, 0.7) }
        }
    }

    // Wisps drifting along the ground, moved on by a timer for the same
    // reason as the clouds (see Clouds.qml). The drift is in seconds.
    property real drift: 0
    Timer {
        interval: 66
        repeat: true
        running: root.fog > 0.02
        onTriggered: root.drift += interval / 1000
    }
    Repeater {
        model: 5
        // Stratus bands from the cloud painter, so the fog has no hard edges.
        delegate: CloudShape {
            required property int index
            readonly property real a: Math.abs(Math.sin(index * 31.9 + 0.7))
            readonly property real w: root.width * (0.6 + 0.4 * a)
            // Seeds 1, 4, 7... are the long, low stratus kind.
            seed: 1 + 3 * (index + 2)
            tone: root.fogColor
            shade: root.fogColor
            width: w
            height: w * 0.24
            y: root.height * (0.5 + 0.34 * ((index * 0.37) % 1))
            // Crosses `4 + 2 * index` times in seven minutes.
            x: ((a + root.drift * (4 + index * 2) / 420) % 1) * (root.width + w) - w
            opacity: root.fog * 0.75
        }
    }
}
