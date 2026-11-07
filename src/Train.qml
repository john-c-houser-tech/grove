import QtQuick

// The railway across the valley, and the night train that runs on it.
//
// The track and the level crossing are always there, a thin line along the
// valley floor and a signal post with a crossbuck, lit or not. When a train
// comes it appears from behind the hill on the right, reaches the crossing as
// the loud horn sounds, and goes in behind the hill on the left, in step with
// the sound. Times here are seconds into the 44-second sound.
Item {
    id: root

    // Where the horizon sits, as a fraction of the height.
    property real horizon: 0.88
    property real progress: 0
    property int totalMilliseconds: 44000
    property bool running: false
    // The colour of the cars and the signal post, darker than the land.
    property color carColor: "#05070c"
    property color floorColor: "#33503f"
    property color ballastColor: "#26382d"
    property color railColor: "#6d7f78"
    property color roadColor: "#7a7258"

    // The crossing stands a little right of the middle, clear of the plant.
    readonly property real crossingX: 0.63
    readonly property real seconds: progress * totalMilliseconds / 1000
    readonly property real atCrossing: 26
    readonly property int carCount: 11
    // The train is 0.2 of the width long, and runs fast enough for its last
    // car to be behind the left hill before the sound ends.
    readonly property real trainLength: 0.2
    readonly property real speed: (crossingX + trainLength) / 16.5
    // Where the front of the train is, as a fraction of the width.
    readonly property real headX: crossingX - speed * (seconds - atCrossing)
    // Lights come up and go down gently at either end.
    readonly property real fade: Math.max(0, Math.min(1, seconds / 3, (totalMilliseconds / 1000 - seconds) / 3))

    NumberAnimation {
        id: run
        target: root
        property: "progress"
        from: 0
        to: 1
        easing.type: Easing.Linear
        onStopped: root.running = false
    }

    function go(milliseconds) {
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
    function nearness(x) {
        var d = (x - crossingX) / 0.3
        return Math.exp(-d * d)
    }
    function sizeAt(x) { return 0.65 + 0.5 * nearness(x) }
    function trackY(x) { return height * (horizon - 0.085) + height * 0.014 * nearness(x) }
    readonly property real carLength: height * 0.024
    readonly property real carHeight: height * 0.012

    // The valley floor, with the track along its far edge. The hills in front
    // cover its lower part.
    Canvas {
        id: bed
        anchors.fill: parent
        onWidthChanged: requestPaint()
        onHeightChanged: requestPaint()
        Connections {
            target: root
            function onFloorColorChanged() { bed.requestPaint() }
            function onBallastColorChanged() { bed.requestPaint() }
            function onRailColorChanged() { bed.requestPaint() }
            function onRoadColorChanged() { bed.requestPaint() }
        }
        onPaint: {
            var ctx = getContext("2d")
            ctx.reset()
            var W = width, H = height
            if (W <= 0 || H <= 0)
                return
            function line() {
                ctx.beginPath()
                for (var x = 0; x <= W; x += 4)
                    x === 0 ? ctx.moveTo(x, root.trackY(x / W)) : ctx.lineTo(x, root.trackY(x / W))
            }
            line()
            ctx.lineTo(W, H)
            ctx.lineTo(0, H)
            ctx.closePath()
            ctx.fillStyle = root.floorColor
            ctx.fill()
            // A dirt road comes up from the near meadow and over the rails at the
            // crossing, narrowing with distance.
            var cx = W * root.crossingX, cy = root.trackY(root.crossingX)
            ctx.beginPath()
            ctx.moveTo(cx - H * 0.006, cy - H * 0.004)
            ctx.lineTo(cx + H * 0.006, cy - H * 0.004)
            ctx.lineTo(cx + H * 0.05, H)
            ctx.lineTo(cx - H * 0.01, H)
            ctx.closePath()
            ctx.fillStyle = root.roadColor
            ctx.fill()
            // Ballast, then a rail catching what light there is on top of it.
            line()
            ctx.lineWidth = Math.max(2, H * 0.004)
            ctx.strokeStyle = root.ballastColor
            ctx.stroke()
            line()
            ctx.lineWidth = 1
            ctx.strokeStyle = root.railColor
            ctx.stroke()
        }
    }

    // The level crossing: a post with a crossbuck, and two red lamps on an arm
    // that flash in turn while the bells ring and are dark otherwise.
    Item {
        id: signal
        readonly property bool ringing: root.running && root.seconds >= 12.5 && root.seconds <= 34
        readonly property bool first: Math.floor((root.seconds - 12.5) / 0.62) % 2 === 0
        readonly property real unit: root.sizeAt(root.crossingX)
        readonly property real post: root.height * 0.044 * unit
        x: root.width * root.crossingX
        y: root.trackY(root.crossingX)

        Rectangle {
            x: -width / 2
            y: -signal.post
            width: Math.max(1, root.height * 0.0022 * signal.unit)
            height: signal.post
            color: root.carColor
        }
        // The crossbuck: two boards crossed at the top of the post.
        Repeater {
            model: 2
            delegate: Rectangle {
                x: -width / 2
                y: -signal.post * 0.94 - height / 2
                width: root.height * 0.02 * signal.unit
                height: Math.max(1, root.height * 0.0024 * signal.unit)
                rotation: index === 0 ? 35 : -35
                color: root.carColor
            }
        }
        // The arm the lamps hang from.
        Rectangle {
            x: -width / 2
            y: -signal.post * 0.62
            width: root.height * 0.017 * signal.unit
            height: Math.max(1, root.height * 0.0016 * signal.unit)
            color: root.carColor
        }
        Repeater {
            model: 2
            delegate: Item {
                readonly property bool lit: signal.ringing && (index === 0) === signal.first
                x: (index === 0 ? -1 : 1) * root.height * 0.007 * signal.unit
                y: -signal.post * 0.62 + root.height * 0.003 * signal.unit
                SoftGlow {
                    x: -width / 2
                    y: -height / 2
                    width: root.height * 0.05 * signal.unit
                    height: width
                    color: "#ff3b2a"
                    opacity: lit ? 0.95 * root.fade : 0
                }
                Rectangle {
                    x: -width / 2
                    y: -height / 2
                    width: Math.max(2, root.height * 0.004 * signal.unit)
                    height: width
                    radius: width / 2
                    color: lit ? "#ff6a55" : root.carColor
                }
            }
        }
    }

    // The cars, the front one first and on the left, since the train runs
    // from right to left.
    Repeater {
        model: root.carCount
        delegate: Item {
            readonly property real at: root.headX + index * root.trainLength / root.carCount
            readonly property real unit: root.sizeAt(at)
            visible: root.running && at > -0.03 && at < 1.03
            x: root.width * at - width / 2
            y: root.trackY(at) - height
            width: root.carLength * unit
            height: root.carHeight * unit * (index === 0 ? 1.25 : 1)
            opacity: root.fade

            Rectangle {
                anchors.fill: parent
                radius: Math.min(2, parent.height / 3)
                color: root.carColor
            }
            // A row of dim windows down the side, a few of them dark, which is
            // how a train reads from far off at night.
            Repeater {
                model: index === 0 ? 0 : 3
                delegate: Rectangle {
                    readonly property int window: model.index
                    x: parent.width * (0.14 + window * 0.3)
                    y: parent.height * 0.28
                    width: parent.width * 0.16
                    height: parent.height * 0.3
                    color: "#ffcf7a"
                    opacity: ((index * 7 + window * 3) % 5) === 0 ? 0.1 : 0.55
                }
            }
            // A lit window in the cab of the front unit, at its leading end.
            Rectangle {
                visible: index === 0
                x: parent.width * 0.22
                y: parent.height * 0.22
                width: parent.width * 0.16
                height: parent.height * 0.3
                color: "#ffd98a"
                opacity: 0.9
            }
        }
    }

    // The headlight and the glow it throws down the line ahead.
    Item {
        id: head
        readonly property real unit: root.sizeAt(root.headX)
        readonly property real front: root.width * root.headX - root.carLength * unit * 0.5
        readonly property real level: root.trackY(root.headX) - root.carHeight * unit * 0.6
        visible: root.running && root.headX > -0.02 && root.headX < 1.02
        opacity: root.fade

        SoftGlow {
            width: root.height * 0.2 * head.unit
            height: root.height * 0.05 * head.unit
            x: head.front - width * 0.9
            y: head.level - height / 2
            color: "#ffe8b0"
            strength: 0.5
        }
        SoftGlow {
            width: root.height * 0.045 * head.unit
            height: width
            x: head.front - width / 2
            y: head.level - height / 2
            color: "#fff6dc"
        }
    }
}
