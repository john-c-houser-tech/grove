import QtQuick

// A small plane crossing the sky far away, in step with the faint drone you
// hear: it is overhead at the middle of the flight, which is when the sound
// is loudest. Nothing is drawn between flights.
Item {
    id: root

    property real flightHeight: 0.88
    // How far through the flight it is, from 0 to 1.
    property real progress: 0
    property int direction: 1
    property real altitude: 0.5
    property color bodyColor: "#2b3644"
    property bool flying: false

    readonly property real span: Math.min(width, height) * 0.05
    visible: flying
    // 1 through a flight. When the session ends mid-flight it eases to 0 and
    // the plane goes with its sound.
    property real presence: 1
    opacity: presence

    NumberAnimation {
        id: flight
        target: root
        property: "progress"
        from: 0
        to: 1
        easing.type: Easing.Linear
        onStopped: root.flying = false
    }
    NumberAnimation {
        id: leaving
        target: root
        property: "presence"
        to: 0
        duration: 1500
        onFinished: flight.stop()
    }

    function fly(dir, milliseconds, high) {
        leaving.stop()
        presence = 1
        direction = dir
        altitude = high
        flight.duration = milliseconds
        // Restarting a flight under way stops it first, which clears `flying`,
        // so it is set after.
        flight.restart()
        flying = true
    }

    // Fades out a plane still in the sky.
    function leave() {
        if (flying)
            leaving.start()
    }

    Canvas {
        id: shape
        width: 34
        height: 14
        scale: root.span / 34
        transformOrigin: Item.TopLeft
        // From one edge of the window to the other, slowly climbing a little.
        x: root.direction > 0 ? -root.span + (root.width + 2 * root.span) * root.progress
                              : root.width + root.span - (root.width + 2 * root.span) * root.progress
        y: root.height * (0.12 + 0.22 * root.altitude) - root.height * 0.012 * root.progress
        // A plane seen from the side, drawn nose to the right and mirrored when it flies left.
        transform: Scale { origin.x: 17; xScale: root.direction }
        // Fade in and out at the edges of the sky, and fade behind thick cloud.
        opacity: Math.min(1, root.progress / 0.06, (1 - root.progress) / 0.06) * (1 - 0.6 * weather.cloud) * 0.9

        // A canvas that starts hidden needs asking to paint once it is shown.
        onVisibleChanged: if (visible) requestPaint()
        Component.onCompleted: requestPaint()
        Connections {
            target: root
            function onBodyColorChanged() { shape.requestPaint() }
        }
        onPaint: {
            var ctx = getContext("2d")
            ctx.reset()
            ctx.fillStyle = root.bodyColor
            // Fuselage, tapering to the tail.
            ctx.beginPath()
            ctx.moveTo(30, 8)
            ctx.quadraticCurveTo(30, 5.2, 24, 5.2)
            ctx.lineTo(8, 6.2)
            ctx.lineTo(2, 7.4)
            ctx.lineTo(8, 9.6)
            ctx.lineTo(24, 10.6)
            ctx.quadraticCurveTo(30, 10.8, 30, 8)
            ctx.fill()
            // High wing, seen almost edge-on.
            ctx.beginPath()
            ctx.moveTo(11, 5.4)
            ctx.lineTo(22, 5.4)
            ctx.lineTo(20.6, 3.2)
            ctx.lineTo(12.6, 3.2)
            ctx.closePath()
            ctx.fill()
            // Tail fin and tailplane.
            ctx.beginPath()
            ctx.moveTo(2, 7.4)
            ctx.lineTo(0.6, 2.2)
            ctx.lineTo(5.4, 6.6)
            ctx.closePath()
            ctx.fill()
            ctx.fillRect(1.5, 7, 6, 1)
            // The blur of the propeller.
            ctx.globalAlpha = 0.45
            ctx.beginPath()
            ctx.ellipse(30.6, 4.4, 1.4, 7.2)
            ctx.fill()
        }
    }
}
