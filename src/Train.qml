import QtQuick

// A train on the plain at the foot of the hills, in step with the sound: it
// has been heard for a while, comes into view, reaches the crossing as the
// loud horn sounds, and runs off the far side. Only the headlight, a lit cab
// window and the crossing lamps show, as they would at night. Times here are
// seconds into the 44-second sound.
Item {
    id: root

    // Where the horizon sits, as a fraction of the height.
    property real horizon: 0.88
    property real progress: 0
    property int direction: 1
    // How far along from the entry edge the crossing lies, in widths.
    property real crossing: 0.55
    property int totalMilliseconds: 44000
    property bool running: false
    // The colour of the cars, a little darker than the land behind them.
    property color carColor: "#05070c"

    readonly property real seconds: progress * totalMilliseconds / 1000
    readonly property real atCrossing: 26
    readonly property real carCount: 11
    // The train is 0.2 of the width long, and runs fast enough to be clear of
    // the far edge by the end.
    readonly property real trainLength: 0.2
    readonly property real speed: (1 + trainLength - crossing) / 16.5
    // Where the front of the train is, in widths from the entry edge.
    readonly property real headAt: crossing + speed * (seconds - atCrossing)
    // Lights come up and go down gently at either end.
    readonly property real fade: Math.max(0, Math.min(1, seconds / 3, (totalMilliseconds / 1000 - seconds) / 3))

    visible: running

    NumberAnimation {
        id: run
        target: root
        property: "progress"
        from: 0
        to: 1
        easing.type: Easing.Linear
        onStopped: root.running = false
    }

    function go(dir, milliseconds, crossingAt) {
        direction = dir
        crossing = crossingAt
        totalMilliseconds = milliseconds
        running = true
        run.duration = milliseconds
        run.restart()
    }

    // Freezes the train at a point in its run, for looking at it.
    function show(at) {
        run.stop()
        progress = at
        running = true
    }

    // The track bends toward the viewer at the crossing and away on either
    // side, so the train swells a little as it arrives and shrinks as it goes.
    function nearness(u) {
        var d = (u - crossing) / 0.3
        return Math.exp(-d * d)
    }
    function sizeAt(u) { return 0.65 + 0.5 * nearness(u) }
    function screenX(u) { return (direction > 0 ? u : 1 - u) * width }
    function trackY(u) { return height * (horizon - 0.085) + height * 0.014 * nearness(u) }
    readonly property real carLength: height * 0.024
    readonly property real carHeight: height * 0.012

    // The crossing: a post with two red lamps that flash in turn with the bells.
    Item {
        id: signal
        readonly property bool ringing: root.seconds >= 12.5 && root.seconds <= 34
        readonly property bool first: Math.floor((root.seconds - 12.5) / 0.62) % 2 === 0
        readonly property real unit: root.sizeAt(root.crossing)
        x: root.screenX(root.crossing)
        y: root.trackY(root.crossing)
        opacity: root.fade

        Rectangle {
            x: -width / 2
            y: -height
            width: Math.max(1, root.height * 0.002 * signal.unit)
            height: root.height * 0.034 * signal.unit
            color: root.carColor
        }
        Repeater {
            model: 2
            delegate: Item {
                readonly property bool lit: signal.ringing && (index === 0) === signal.first
                x: (index === 0 ? -1 : 1) * root.height * 0.007 * signal.unit
                y: -root.height * 0.03 * signal.unit
                SoftGlow {
                    x: -width / 2
                    y: -height / 2
                    width: root.height * 0.05 * signal.unit
                    height: width
                    color: "#ff3b2a"
                    opacity: lit ? 0.95 : 0
                }
                Rectangle {
                    x: -width / 2
                    y: -height / 2
                    width: Math.max(1.5, root.height * 0.0035 * signal.unit)
                    height: width
                    radius: width / 2
                    color: lit ? "#ff6a55" : "#2a1210"
                }
            }
        }
    }

    // The cars, front first. The front one carries the headlight and a lit cab.
    Repeater {
        model: root.carCount
        delegate: Item {
            readonly property real u: root.headAt - index * root.trainLength / root.carCount
            readonly property real unit: root.sizeAt(u)
            visible: u > -0.03 && u < 1.03
            x: root.screenX(u) - width / 2
            y: root.trackY(u) - height
            width: root.carLength * unit
            height: root.carHeight * unit * (index === 0 ? 1.25 : 1)
            opacity: root.fade

            Rectangle {
                anchors.fill: parent
                radius: Math.min(2, parent.height / 3)
                color: root.carColor
            }
            // A row of dim windows down the side, a few of them dark, which is how
            // a train reads from far off at night.
            Repeater {
                model: index === 0 ? 0 : 3
                delegate: Rectangle {
                    x: parent.width * (0.14 + modelIndex * 0.3)
                    y: parent.height * 0.28
                    width: parent.width * 0.16
                    height: parent.height * 0.3
                    color: "#ffcf7a"
                    opacity: ((index * 7 + modelIndex * 3) % 5) === 0 ? 0.1 : 0.55
                    readonly property int modelIndex: model.index
                }
            }
            // A lit window in the cab of the front unit.
            Rectangle {
                visible: index === 0
                x: root.direction > 0 ? parent.width * 0.62 : parent.width * 0.22
                y: parent.height * 0.22
                width: parent.width * 0.16
                height: parent.height * 0.3
                color: "#ffd98a"
                opacity: 0.9
            }
        }
    }

    // The headlight and the glow it throws down the line, on the front unit.
    Item {
        id: head
        readonly property real unit: root.sizeAt(root.headAt)
        readonly property real front: root.screenX(root.headAt) + root.direction * root.carLength * unit * 0.5
        visible: root.headAt > -0.02 && root.headAt < 1.02
        opacity: root.fade * Math.min(1, root.headAt / 0.04, (1.02 - root.headAt) / 0.04)

        SoftGlow {
            width: root.height * 0.2 * head.unit
            height: root.height * 0.05 * head.unit
            x: head.front + (root.direction > 0 ? -width * 0.1 : -width * 0.9)
            y: root.trackY(root.headAt) - root.carHeight * head.unit * 0.6 - height / 2
            color: "#ffe8b0"
            strength: 0.5
        }
        SoftGlow {
            width: root.height * 0.045 * head.unit
            height: width
            x: head.front - width / 2
            y: root.trackY(root.headAt) - root.carHeight * head.unit * 0.6 - height / 2
            color: "#fff6dc"
        }
    }
}
