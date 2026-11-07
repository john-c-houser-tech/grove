import QtQuick

// Soft clouds drifting across the sky. More of them appear as the cover
// grows, and rain darkens them.
Item {
    id: root

    property real cover: 0
    property real rain: 0
    property real ambient: 1

    // One slow loop that every cloud drifts along at its own speed.
    property real drift: 0
    NumberAnimation on drift {
        from: 0
        to: 1
        duration: 600000
        loops: Animation.Infinite
        running: root.cover > 0.02
    }

    readonly property real shade: (0.2 + 0.72 * ambient) * (1 - 0.4 * rain)
    readonly property color cloudColor: Qt.rgba(shade * 0.95, shade * 0.97, shade, 1)
    // The underside of a cloud is always a little darker than its top.
    readonly property color cloudShade: Qt.rgba(shade * 0.62, shade * 0.68, shade * 0.78, 1)

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
            x: ((a + root.drift * speed) % 1) * (root.width + w) - w
            // Cloud number i appears once the cover passes i of 9.
            opacity: Math.max(0, Math.min(1, root.cover * 9 - index)) * (0.7 + 0.3 * root.cover)
            Behavior on opacity { NumberAnimation { duration: 800 } }
        }
    }
}
