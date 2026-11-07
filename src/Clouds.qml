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
    readonly property color cloudColor: Qt.rgba(shade * 0.93, shade * 0.96, shade, 1)

    Repeater {
        model: 9
        delegate: Item {
            id: cloud
            // Deterministic scatter, so the sky has the same clouds each time.
            readonly property real a: Math.abs(Math.sin(index * 91.7 + 1.3))
            readonly property real b: Math.abs(Math.sin(index * 47.1 + 2.9))
            readonly property real w: root.width * (0.28 + 0.2 * a)
            readonly property real speed: 6 + 10 * b
            width: w
            height: w * 0.32
            y: root.height * (0.05 + 0.34 * b)
            x: ((a + root.drift * speed) % 1) * (root.width + w) - w
            // Cloud number i appears once the cover passes i of 9.
            opacity: Math.max(0, Math.min(1, root.cover * 9 - index)) * (0.5 + 0.4 * root.cover)
            Behavior on opacity { NumberAnimation { duration: 800 } }

            Repeater {
                model: [
                    { px: 0.0, py: 0.35, pw: 0.62, ph: 0.65 },
                    { px: 0.25, py: 0.0, pw: 0.5, ph: 0.8 },
                    { px: 0.5, py: 0.3, pw: 0.5, ph: 0.7 }
                ]
                delegate: Rectangle {
                    x: cloud.width * modelData.px
                    y: cloud.height * modelData.py
                    width: cloud.width * modelData.pw
                    height: cloud.height * modelData.ph
                    radius: height / 2
                    color: root.cloudColor
                }
            }
        }
    }
}
