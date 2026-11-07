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
    // Beyond the crossing: the bare earth of the cut slope above the road,
    // the tipped earth of the fill slope below it, and the hillside itself.
    property color cutColor: "#4a4232"
    property color fillColor: "#3d3a2c"
    property color hillColor: "#44605a"
    property color hazeColor: "#9fb7c4"

    // The crossing stands a little right of the middle, clear of the plant.
    readonly property real crossingX: 0.68
    readonly property real seconds: progress * totalMilliseconds / 1000
    readonly property real atCrossing: 26
    // Everything on the railway is drawn to one scale, in metres, so the train
    // and the signal agree: a metre is this many pixels at the crossing.
    readonly property real metre: height * 0.007
    // A diesel at the front and three coaches, with a coupling's gap between.
    readonly property var consist: [
        { kind: "diesel", length: 17, tall: 4.75 },
        { kind: "coach", length: 25, tall: 4.4 },
        { kind: "coach", length: 25, tall: 4.4 },
        { kind: "coach", length: 25, tall: 4.4 }
    ]
    readonly property real coupling: 0.6
    readonly property real lengthMetres: 17 + 3 * 25 + 3 * coupling
    // The train's length as a fraction of the width, and a speed that gets
    // its last coach behind the left hill before the sound ends.
    readonly property real trainLength: lengthMetres * metre / Math.max(1, width)
    readonly property real speed: (crossingX + trainLength) / 16.5
    // Where the front of the train is, as a fraction of the width. It runs
    // right to left.
    readonly property real headX: crossingX - speed * (seconds - atCrossing)
    // Lights come up and go down gently at either end.
    readonly property real fade: Math.max(0, Math.min(1, seconds / 3, (totalMilliseconds / 1000 - seconds) / 3))
    // The diesel's ditch lights flash while its horn sounds for the crossing.
    readonly property bool horning: (seconds >= 3 && seconds <= 12) || (seconds >= 16 && seconds <= 26.5)

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

    // The track is straight and level across the valley, and everything on it
    // is at one distance and so one scale: a train is rigid, and every wheel
    // sits on the rail.
    function sizeAt(x) { return 1 }
    function trackY(x) { return height * (horizon - 0.085) }

    // Where each vehicle's front end is, in pixels, laid end to end from the
    // front of the train.
    readonly property var layout: {
        var out = [], x = width * headX
        for (var i = 0; i < consist.length; i++) {
            var unit = sizeAt(x / Math.max(1, width))
            out.push({ x: x, unit: unit })
            x += (consist[i].length + coupling) * metre * unit
        }
        return out
    }

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
            function onCutColorChanged() { bed.requestPaint() }
            function onFillColorChanged() { bed.requestPaint() }
            function onHillColorChanged() { bed.requestPaint() }
            function onHazeColorChanged() { bed.requestPaint() }
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
            // A dirt road. On this side it comes a little toward us, then curves
            // away to the right along the valley floor and goes behind the hill
            // there. Beyond the rails it climbs the hillside to the left, where
            // the hill is higher, and goes into a cut in the slope. The signal
            // stands on its right-hand verge.
            var post = W * root.crossingX, cy = root.trackY(root.crossingX)
            var gap = H * 0.006, crossing = H * 0.05
            var centre = post - gap - crossing / 2

            function curve(p0, p1, p2, p3, t) {
                var u = 1 - t
                return {
                    x: u * u * u * p0.x + 3 * u * u * t * p1.x + 3 * u * t * t * p2.x + t * t * t * p3.x,
                    y: u * u * u * p0.y + 3 * u * u * t * p1.y + 3 * u * t * t * p2.y + t * t * t * p3.y
                }
            }
            // A road along a curve, seen from low down: as wide as the crossing
            // where it runs straight toward or away from us, thin where it turns
            // to run across the view, and narrower with distance (farther).
            function road(p0, p1, p2, p3, farther) {
                var upper = [], lower = []
                for (var i = 0; i <= 40; i++) {
                    var t = i / 40
                    var at = curve(p0, p1, p2, p3, t)
                    var next = curve(p0, p1, p2, p3, Math.min(1, t + 0.01)), prev = curve(p0, p1, p2, p3, Math.max(0, t - 0.01))
                    var dx = next.x - prev.x, dy = next.y - prev.y, len = Math.sqrt(dx * dx + dy * dy) || 1
                    var toward = Math.abs(dy) / len
                    var shrink = 1 - farther * t
                    var half = H * (0.006 + 0.019 * toward) * shrink
                    var nx = -dy / len, ny = dx / len
                    upper.push({ x: at.x + nx * half, y: at.y + ny * half * (1 - toward) - (1 - toward) * half * 0.2 })
                    lower.push({ x: at.x - nx * half, y: at.y - ny * half * (1 - toward) + (1 - toward) * half * 0.2 })
                }
                ctx.beginPath()
                ctx.moveTo(upper[0].x, upper[0].y)
                for (var a = 1; a < upper.length; a++)
                    ctx.lineTo(upper[a].x, upper[a].y)
                for (var b = lower.length - 1; b >= 0; b--)
                    ctx.lineTo(lower[b].x, lower[b].y)
                ctx.closePath()
                ctx.fill()
                return { upper: upper, lower: lower }
            }

            ctx.fillStyle = root.roadColor
            // Near side.
            road({ x: centre, y: cy }, { x: centre, y: cy + H * 0.03 },
                 { x: centre + W * 0.1, y: cy + H * 0.034 }, { x: W * 1.08, y: cy + H * 0.028 }, 0)
            // Far side: the road is cut into the hillside as it climbs, like an
            // inslope road. It leaves the rails at full width, bends left and
            // climbs across the face of the shoulder of hill, with a cut slope of
            // bare earth above it and a fill slope below, then curves around
            // the shoulder and out of sight.
            var far0 = { x: centre, y: cy }, far1 = { x: centre - W * 0.006, y: cy - H * 0.02 }
            var far2 = { x: centre - W * 0.06, y: cy - H * 0.03 }, far3 = { x: centre - W * 0.13, y: cy - H * 0.056 }
            // Measure the road's edges first, without drawing.
            var edges = (function () {
                var saved = ctx.fillStyle
                ctx.fillStyle = "transparent"
                var e = road(far0, far1, far2, far3, 0.7)
                ctx.fillStyle = saved
                return e
            })()
            var n = edges.upper.length
            function along(i) { return i / (n - 1) }
            // The fill slope: earth tipped down the hill below the road, deepest
            // where the road is farthest out on the slope.
            var fillFrom = Math.floor(n * 0.2)
            ctx.fillStyle = root.fillColor
            ctx.beginPath()
            ctx.moveTo(edges.lower[fillFrom].x, edges.lower[fillFrom].y)
            for (var i = fillFrom + 1; i < n; i++)
                ctx.lineTo(edges.lower[i].x, edges.lower[i].y)
            for (var k = n - 1; k >= fillFrom; k--) {
                var tf = (along(k) - along(fillFrom)) / (1 - along(fillFrom))
                var drop = H * 0.012 * Math.sin(Math.PI * Math.min(1, tf * 1.1)) * (1 - 0.5 * along(k))
                ctx.lineTo(edges.lower[k].x + W * 0.004 * tf, edges.lower[k].y + drop)
            }
            ctx.closePath()
            ctx.fill()
            // The road itself.
            ctx.fillStyle = root.roadColor
            road(far0, far1, far2, far3, 0.7)
            // The cut slope: the bank left where the road was dug into the hill,
            // tallest partway up and gone where the road leaves the flat.
            var cutFrom = Math.floor(n * 0.25)
            var cutTop = []
            for (var c = cutFrom; c < n; c++) {
                var tc = (along(c) - along(cutFrom)) / (1 - along(cutFrom))
                var tall = H * (0.004 + 0.02 * Math.sin(Math.PI * 0.5 * Math.min(1, tc * 1.6))) * (1 - 0.35 * tc)
                cutTop.push({ x: edges.upper[c].x - W * 0.003 * tc, y: edges.upper[c].y - tall })
            }
            ctx.fillStyle = root.cutColor
            ctx.beginPath()
            ctx.moveTo(edges.upper[cutFrom].x, edges.upper[cutFrom].y)
            for (var u2 = cutFrom + 1; u2 < n; u2++)
                ctx.lineTo(edges.upper[u2].x, edges.upper[u2].y)
            for (var v2 = cutTop.length - 1; v2 >= 0; v2--)
                ctx.lineTo(cutTop[v2].x, cutTop[v2].y)
            ctx.closePath()
            ctx.fill()
            // Rain has scored the cut with short runnels down its face.
            ctx.strokeStyle = Qt.rgba(root.cutColor.r * 0.82, root.cutColor.g * 0.82, root.cutColor.b * 0.82, 0.45)
            ctx.lineWidth = 1
            for (var r2 = 3; r2 < cutTop.length - 2; r2 += 5) {
                var topPt = cutTop[r2], footPt = edges.upper[cutFrom + r2]
                ctx.beginPath()
                ctx.moveTo(topPt.x, topPt.y + (footPt.y - topPt.y) * 0.15)
                ctx.lineTo(topPt.x + (footPt.x - topPt.x) * 0.7 + 1, topPt.y + (footPt.y - topPt.y) * 0.75)
                ctx.stroke()
            }
            // The hillside above the cut lies farther up the slope and turns a
            // little away from us, so it is a shade darker than the slope below
            // the road. It runs up to the skyline of the ridge.
            function hash(i, seed) {
                var v = Math.sin(i * 127.1 + seed * 311.7) * 43758.5453
                return v - Math.floor(v)
            }
            function noise(v, seed) {
                var i = Math.floor(v), f = v - i
                var u = f * f * (3 - 2 * f)
                return hash(i, seed) * (1 - u) + hash(i + 1, seed) * u
            }
            // The same ridge line as the hills draw (Hills.qml, mid()).
            function ridgeY(px) {
                var u = px / W
                var shoulder = (u - 0.6) / 0.12
                var rise = H * (0.035 + 0.15 * noise(u * 2.6 + 5, 4)) + H * 0.025 * noise(u * 8.2, 5)
                           + H * 0.07 * Math.exp(-shoulder * shoulder)
                return H * root.horizon - rise
            }
            var bend = curve(far0, far1, far2, far3, 0.86)
            var span = H * 0.034
            // The edge of the shoulder: a rounded hill seen from here shows a
            // convex edge, bulging toward us, from the skyline down across the
            // road's last stretch. The road wraps round the outside of it and
            // passes out of sight behind.
            var foldTop = { x: bend.x - W * 0.004, y: ridgeY(bend.x - W * 0.004) + 1 }
            var fold3 = { x: bend.x - W * 0.008, y: bend.y + span * 0.6 }
            var fold1 = { x: bend.x + W * 0.03, y: foldTop.y + (fold3.y - foldTop.y) * 0.3 }
            var fold2 = { x: bend.x + W * 0.03, y: foldTop.y + (fold3.y - foldTop.y) * 0.78 }
            function traceFold() {
                ctx.bezierCurveTo(fold1.x, fold1.y, fold2.x, fold2.y, fold3.x, fold3.y)
            }

            // The slope above the cut, darker, fading out toward the crossing so
            // it has no edge there; its left side is hidden under the flank.
            var leftX = bend.x - W * 0.02, rightX = cutTop[0].x
            var shade = ctx.createLinearGradient(rightX - W * 0.06, 0, rightX + W * 0.02, 0)
            shade.addColorStop(0, Qt.rgba(0.02, 0.03, 0.03, 0.15))
            shade.addColorStop(1, Qt.rgba(0.02, 0.03, 0.03, 0))
            ctx.fillStyle = shade
            ctx.beginPath()
            ctx.moveTo(cutTop[0].x, cutTop[0].y)
            for (var d2 = 1; d2 < cutTop.length; d2++)
                ctx.lineTo(cutTop[d2].x, cutTop[d2].y)
            ctx.lineTo(leftX, cutTop[cutTop.length - 1].y)
            for (var sx = leftX; sx <= rightX + W * 0.02; sx += 3)
                ctx.lineTo(sx, ridgeY(sx) + 1)
            ctx.closePath()
            ctx.fill()

            // Where the road curves around the shoulder, the near flank of the
            // hill comes across it, and it passes out of sight. The flank runs
            // up to the skyline, coloured exactly as the hillside around it:
            // fading toward the sky colour at its foot, as the ridges are painted.
            var y0 = H * root.horizon
            var flank = ctx.createLinearGradient(0, y0 - H * 0.2, 0, y0 + 2)
            flank.addColorStop(0, root.hillColor)
            flank.addColorStop(1, Qt.rgba(root.hillColor.r + (root.hazeColor.r - root.hillColor.r) * 0.4,
                                          root.hillColor.g + (root.hazeColor.g - root.hillColor.g) * 0.4,
                                          root.hillColor.b + (root.hazeColor.b - root.hillColor.b) * 0.4, 1))
            ctx.fillStyle = flank
            var farLeft = bend.x - W * 0.08
            ctx.beginPath()
            ctx.moveTo(foldTop.x, foldTop.y)
            traceFold()
            ctx.lineTo(farLeft, fold3.y)
            for (var fx = farLeft; fx <= foldTop.x; fx += 3)
                ctx.lineTo(fx, ridgeY(fx) + 1)
            ctx.closePath()
            ctx.fill()
            // A soft shadow along the fold.
            ctx.strokeStyle = Qt.darker(root.hillColor, 1.12)
            ctx.lineWidth = Math.max(1, H * 0.002)
            ctx.beginPath()
            ctx.moveTo(foldTop.x, foldTop.y)
            traceFold()
            ctx.stroke()
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

    // The level crossing, to the same scale as the train: a post with a
    // crossbuck, and two red lamps on an arm that flash in turn while the
    // bells ring and are dark otherwise.
    Item {
        id: signal
        readonly property bool ringing: root.running && root.seconds >= 12.5 && root.seconds <= 34
        readonly property bool first: Math.floor((root.seconds - 12.5) / 0.62) % 2 === 0
        readonly property real m: root.metre * root.sizeAt(root.crossingX)
        x: root.width * root.crossingX
        y: root.trackY(root.crossingX)

        Rectangle {
            x: -width / 2
            y: -4.6 * signal.m
            width: Math.max(1, 0.2 * signal.m)
            height: 4.6 * signal.m
            color: root.carColor
        }
        // The crossbuck: two boards crossed near the top of the post.
        Repeater {
            model: 2
            delegate: Rectangle {
                x: -width / 2
                y: -4.15 * signal.m - height / 2
                width: 2.7 * signal.m
                height: Math.max(1, 0.24 * signal.m)
                rotation: index === 0 ? 35 : -35
                color: root.carColor
            }
        }
        // The arm the lamps hang from.
        Rectangle {
            x: -width / 2
            y: -3.0 * signal.m
            width: 1.7 * signal.m
            height: Math.max(1, 0.16 * signal.m)
            color: root.carColor
        }
        Repeater {
            model: 2
            delegate: Item {
                readonly property bool lit: signal.ringing && (index === 0) === signal.first
                x: (index === 0 ? -0.62 : 0.62) * signal.m
                y: -2.7 * signal.m
                SoftGlow {
                    x: -width / 2
                    y: -height / 2
                    width: 3.4 * signal.m
                    height: width
                    color: "#ff3b2a"
                    opacity: lit ? 0.95 * root.fade : 0
                }
                // The lamp's black backing, and the lens on it.
                Rectangle {
                    x: -width / 2
                    y: -height / 2
                    width: Math.max(2, 0.6 * signal.m)
                    height: width
                    radius: width / 2
                    color: root.carColor
                }
                Rectangle {
                    x: -width / 2
                    y: -height / 2
                    width: Math.max(1.5, 0.34 * signal.m)
                    height: width
                    radius: width / 2
                    color: lit ? "#ff6a55" : Qt.lighter(root.carColor, 1.6)
                }
            }
        }
    }

    // The train. Each vehicle is drawn once at full size, in metres, and only
    // moved and scaled as it runs.
    Repeater {
        model: root.consist.length
        delegate: Canvas {
            id: vehicle
            readonly property var spec: root.consist[index]
            readonly property var slot: root.layout[index]
            readonly property int number: index
            width: spec.length * root.metre
            height: spec.tall * root.metre
            transformOrigin: Item.BottomLeft
            scale: slot.unit
            x: slot.x
            y: root.trackY((slot.x + width * slot.unit / 2) / Math.max(1, root.width)) - height
            visible: root.running && slot.x < root.width + 10 && slot.x + width * slot.unit > -10
            opacity: root.fade

            onWidthChanged: requestPaint()
            onHeightChanged: requestPaint()
            Connections {
                target: root
                function onCarColorChanged() { vehicle.requestPaint() }
            }

            onPaint: {
                var ctx = getContext("2d")
                ctx.reset()
                var m = root.metre
                if (m <= 0)
                    return
                var h = spec.tall
                ctx.scale(m, m)
                // Drawing in metres, with heights measured up from the rail.
                function box(x0, y0, x1, y1) { ctx.fillRect(x0, h - y1, x1 - x0, y1 - y0) }
                function wheel(cx) {
                    ctx.beginPath()
                    ctx.arc(cx, h - 0.5, 0.48, 0, Math.PI * 2)
                    ctx.fill()
                }
                var body = root.carColor
                var under = Qt.darker(root.carColor, 1.4)
                var warm = "#ffcf7a"

                if (spec.kind === "diesel") {
                    // Trucks with two axles each, and the fuel tank between them.
                    ctx.fillStyle = under
                    box(2.4, 0.5, 5.6, 1.2)
                    box(11.4, 0.5, 14.6, 1.2)
                    wheel(3.2); wheel(4.8); wheel(12.2); wheel(13.8)
                    ctx.beginPath()
                    ctx.moveTo(6.2, h - 1.2); ctx.lineTo(10.8, h - 1.2)
                    ctx.lineTo(10.4, h - 0.55); ctx.lineTo(6.6, h - 0.55)
                    ctx.closePath()
                    ctx.fill()
                    // The body: a short sloping nose at the front, on the left,
                    // the windshield above it, then the cab and a long hood.
                    ctx.beginPath()
                    ctx.moveTo(0.3, h - 1.55)
                    ctx.lineTo(0.3, h - 2.5)
                    ctx.lineTo(1.1, h - 3.05)
                    ctx.lineTo(1.35, h - 3.1)
                    ctx.lineTo(2.1, h - 4.3)
                    ctx.lineTo(2.5, h - 4.4)
                    ctx.lineTo(16.6, h - 4.4)
                    ctx.lineTo(16.9, h - 4.15)
                    ctx.lineTo(16.9, h - 1.55)
                    ctx.closePath()
                    ctx.fill()
                    // On the roof: the exhaust stack, and the raised housing for
                    // the radiator fans at the rear.
                    box(8.2, 4.4, 8.9, 4.7)
                    box(12.0, 4.4, 16.2, 4.6)
                    // The lit cab window, and the windshield catching some of it.
                    ctx.fillStyle = warm
                    ctx.globalAlpha = 0.85
                    box(2.5, 3.1, 3.6, 3.9)
                    ctx.globalAlpha = 0.25
                    ctx.beginPath()
                    ctx.moveTo(1.45, h - 3.2); ctx.lineTo(2.1, h - 4.15)
                    ctx.lineTo(2.35, h - 4.1); ctx.lineTo(1.75, h - 3.2)
                    ctx.closePath()
                    ctx.fill()
                    ctx.globalAlpha = 1
                } else {
                    ctx.fillStyle = under
                    box(1.8, 0.5, 5.0, 1.2)
                    box(20.0, 0.5, 23.2, 1.2)
                    wheel(2.6); wheel(4.2); wheel(20.8); wheel(22.4)
                    // A coach body with a rounded roof.
                    ctx.fillStyle = body
                    var r = 0.6
                    ctx.beginPath()
                    ctx.moveTo(0, h - 1.15)
                    ctx.lineTo(0, h - (spec.tall - r))
                    ctx.quadraticCurveTo(0, 0, r, 0)
                    ctx.lineTo(spec.length - r, 0)
                    ctx.quadraticCurveTo(spec.length, 0, spec.length, h - (spec.tall - r))
                    ctx.lineTo(spec.length, h - 1.15)
                    ctx.closePath()
                    ctx.fill()
                    // Windows down the side, a few of them dark, and the small
                    // windows in the doors at either end.
                    ctx.fillStyle = warm
                    for (var w = 0; w < 10; w++) {
                        ctx.globalAlpha = ((number * 7 + w * 3) % 5) === 0 ? 0.12 : 0.6
                        box(2.9 + w * 2.05, 2.3, 4.3 + w * 2.05, 3.25)
                    }
                    ctx.globalAlpha = 0.35
                    box(0.7, 2.6, 1.3, 3.25)
                    box(23.7, 2.6, 24.3, 3.25)
                    ctx.globalAlpha = 1
                }
            }
        }
    }

    // The diesel's headlight and the glow it throws down the line ahead, and a
    // ditch light low on its nose that flashes while the horn sounds.
    Item {
        id: lamps
        readonly property real m: root.metre * root.layout[0].unit
        readonly property real nose: root.layout[0].x + 0.3 * m
        readonly property real rail: root.trackY((root.layout[0].x + 8 * m) / Math.max(1, root.width))
        visible: root.running && root.headX > -0.05 && root.headX < 1.05
        opacity: root.fade

        SoftGlow {
            width: 26 * lamps.m
            height: 6 * lamps.m
            x: lamps.nose - width * 0.92
            y: lamps.rail - 2.7 * lamps.m - height / 2
            color: "#ffe8b0"
            strength: 0.5
        }
        SoftGlow {
            width: 5 * lamps.m
            height: width
            x: lamps.nose - width / 2
            y: lamps.rail - 2.7 * lamps.m - height / 2
            color: "#fff6dc"
        }
        SoftGlow {
            readonly property bool on: !root.horning || Math.floor(root.seconds * 2) % 2 === 0
            width: 2.6 * lamps.m
            height: width
            x: lamps.nose - width / 2
            y: lamps.rail - 1.4 * lamps.m - height / 2
            color: "#fff1cc"
            opacity: on ? 0.9 : 0.15
        }
    }
}
