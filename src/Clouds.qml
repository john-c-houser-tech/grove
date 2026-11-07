import QtQuick

// Soft clouds drifting across the sky. More of them appear as the cover
// grows, and rain darkens them.
Item {
    id: root

    property real cover: 0
    property real rain: 0
    property real ambient: 1

    // Seconds of drift, which every cloud follows at its own speed. A timer
    // moves them on fifteen times a second, plenty for clouds that take
    // minutes to cross; an animation would redraw the whole window at the
    // display's full rate to move each one a fraction of a pixel.
    property real drift: 0
    Timer {
        interval: 66
        repeat: true
        running: root.cover > 0.02
        onTriggered: root.drift += interval / 1000
    }

    // The sky colour overhead. In low light a cloud is lit mostly by the sky,
    // so it takes on the sky's cool cast; a plain grey beside a blue or violet
    // sky reads as brown, like smog.
    property color sky: "#3d6f9c"

    // White in full daylight, grey as the light fails, darker in rain.
    readonly property real shade: (0.24 + 0.73 * ambient) * (1 - 0.4 * rain)
    // How much of the sky's hue the cloud carries: none at midday, all of it at night.
    readonly property real cast: Math.max(0, Math.min(0.6, 0.8 * (1 - ambient) / 0.78))
    readonly property real skyMean: (sky.r + sky.g + sky.b) / 3
    function clamp01(v) { return Math.max(0, Math.min(1, v)) }
    function lit(level) {
        return Qt.rgba(clamp01(level * 0.98 + cast * (sky.r - skyMean)),
                       clamp01(level * 0.99 + cast * (sky.g - skyMean)),
                       clamp01(level + cast * (sky.b - skyMean)), 1)
    }
    readonly property color cloudColor: lit(shade)
    // The underside of a cloud is always a little darker, and bluer, than its top.
    readonly property color cloudShade: Qt.rgba(cloudColor.r * 0.66, cloudColor.g * 0.71, cloudColor.b * 0.8, 1)

    Repeater {
        model: 9
        delegate: CloudShape {
            id: cloud
            // Deterministic scatter, so the sky has the same clouds each time.
            readonly property real a: Math.abs(Math.sin(index * 91.7 + 1.3))
            readonly property real b: Math.abs(Math.sin(index * 47.1 + 2.9))
            readonly property real w: root.width * (0.3 + 0.24 * a)
            readonly property real speed: 6 + 10 * b
            seed: index + 1
            tone: root.cloudColor
            shade: root.cloudShade
            width: w
            height: w * 0.44
            y: root.height * (0.03 + 0.34 * b)
            // Crosses the sky `speed` times in ten minutes.
            x: ((a + root.drift * speed / 600) % 1) * (root.width + w) - w
            // Cloud number i appears once the cover passes i of 9.
            opacity: Math.max(0, Math.min(1, root.cover * 9 - index)) * (0.7 + 0.3 * root.cover)
            Behavior on opacity { NumberAnimation { duration: 800 } }
        }
    }
}
