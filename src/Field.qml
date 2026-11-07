import QtQuick

// The ploughed field in the foreground, seen in true perspective from low
// down, about 30 cm above the ground, as if sitting at the edge of it.
//
// The rows run straight away from the viewer and meet at the horizon. They
// are ploughed furrows 40 cm apart with ridges 12 cm high and a trough
// between them. The plant is a sapling about
// 2.5 m tall, standing in the furrow between two rows, 4 m away. Every size
// here follows from those numbers, so the rows, the plant and the trunk agree.
Canvas {
    id: field

    // The horizon, as a fraction of the height, and where the field's far
    // edge meets the meadow, in pixels from the top.
    property real horizon: 0.88
    property real groundTop: height * 0.874
    // Where the rows meet on the horizon, and where the plant stands, as
    // fractions of the width.
    property real vanishX: 0.5
    property real plantX: 0.4
    // "far" draws the ground and the rows beyond the plant, "near" the rows in
    // front of it, which are drawn over the plant: a ridge between the eye and
    // the trunk hides the trunk's foot, as it would in a real field.
    property string part: "far"
    property color soilColor: "#2a2018"
    // The sky at the horizon, for the haze over the far side of the field.
    property color hazeColor: "#9fb7c4"
    // Snow lying on the field, from 0 to 1, and frost on the crests in winter.
    property real snow: 0
    property bool frost: false

    // The camera. A metre at the plant's distance is this many pixels, which
    // with the plant 4 m off fixes the focal length; the plant's foot sitting
    // 6% of the height below the horizon then fixes the eye height.
    readonly property real metreAtPlant: height * 0.21
    readonly property real plantDepth: 4
    readonly property real focal: metreAtPlant * plantDepth
    readonly property real eye: height * 0.06 / metreAtPlant
    // How far the plant stands to the side of straight ahead, in metres.
    readonly property real plantLateral: (plantX - vanishX) * width / metreAtPlant
    // The field ends this far away, where the meadow takes over, and this is
    // where that edge falls on the screen.
    readonly property real fieldDepth: 40
    readonly property real farEdgeY: height * horizon + focal * eye / fieldDepth
    // Where the plant's foot is, in pixels from the top.
    readonly property real plantBaseY: height * horizon + focal * eye / plantDepth

    readonly property real rowSpacing: 0.4
    readonly property real ridgeHeight: 0.12
    // Ridges 30 cm across at the base, leaving a 10 cm trough between them.
    readonly property real ridgeHalfWidth: 0.15

    onSoilColorChanged: repaintSoon()
    onHazeColorChanged: repaintSoon()
    onSnowChanged: repaintSoon()
    onFrostChanged: repaintSoon()
    onWidthChanged: requestPaint()
    onHeightChanged: requestPaint()
    onGroundTopChanged: requestPaint()
    onPlantXChanged: requestPaint()
    onPartChanged: requestPaint()

    // Thousands of clods take most of a tenth of a second to draw, and the
    // light and weather change slowly, so colour changes are gathered and the
    // field redrawn at most every two seconds.
    Timer {
        id: throttle
        interval: 2000
        onTriggered: field.requestPaint()
    }
    function repaintSoon() {
        if (!throttle.running)
            throttle.start()
    }

    // A fixed random number from 0 to 1 for any pair of numbers, so every clod
    // is the same clod each time it is drawn.
    function hash(a, b) {
        var x = Math.sin(a * 127.1 + b * 311.7) * 43758.5453
        return x - Math.floor(x)
    }
    // Smooth noise along a ridge, from 0 to 1.
    function noise(x, seed) {
        var i = Math.floor(x), f = x - i
        var u = f * f * (3 - 2 * f)
        return hash(i, seed) * (1 - u) + hash(i + 1, seed) * u
    }

    function mix(a, b, t) {
        return Qt.rgba(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t, 1)
    }

    // A point in the field, in metres (across, up, ahead), on the screen.
    function project(across, up, ahead) {
        return { x: width * vanishX + focal * across / ahead, y: height * horizon + focal * (eye - up) / ahead }
    }

    onPaint: {
        var ctx = getContext("2d")
        ctx.reset()
        var W = width, H = height
        if (W <= 0 || H <= 0)
            return

        var snowColor = Qt.rgba(0.9, 0.93, 0.96, 1)
        var cover = Math.max(snow * 0.85, frost ? 0.25 : 0)
        // The furrow floors are in shadow; the faces of the ridges that turn
        // toward the light are brighter, and snow settles on those first.
        var floorColor = mix(Qt.darker(soilColor, 1.9), snowColor, cover * 0.6)
        var litFace = mix(Qt.lighter(soilColor, 1.9), snowColor, cover)
        var shadeFace = mix(Qt.lighter(soilColor, 1.15), snowColor, cover * 0.75)
        var furrowShade = Qt.darker(floorColor, 1.35)

        ctx.save()
        ctx.beginPath()
        ctx.rect(0, groundTop, W, H - groundTop)
        ctx.clip()
        // Far off, ridges and furrows blur into one average tone with a little
        // haze in it, so the floor lightens toward the far edge of the field.
        var distantTone = mix(mix(litFace, shadeFace, 0.45), hazeColor, 0.06)
        if (part === "far") {
            var fade = ctx.createLinearGradient(0, groundTop, 0, project(0, 0, 10).y)
            fade.addColorStop(0, distantTone)
            fade.addColorStop(1, floorColor)
            ctx.fillStyle = fade
            ctx.fillRect(0, groundTop, W, H - groundTop)
        }

        // From just in front of the bottom of the window out to where the rows
        // are too close together to see, split at the plant.
        // The rows start well in front of the bottom of the window: a ridge
        // stands up from the ground, so its near end would show if they began
        // where the flat ground leaves the window.
        var nearest = 0.45
        var from = part === "far" ? plantDepth : nearest
        // The near rows run a hair past the plant so the two halves meet without a seam.
        var to = part === "far" ? fieldDepth : plantDepth * 1.01
        var steps = part === "far" ? 40 : 30
        var depths = []
        for (var i = 0; i <= steps; i++)
            depths.push(from * Math.pow(to / from, i / steps))

        // Each ridge is ploughed unevenly: its crest rises and dips and wanders
        // a little to the side along its length, and so does its foot.
        function crestHeight(j, d) { return ridgeHeight * (0.65 + 0.5 * noise(d * 1.6 + j * 13.1, 21)) }
        function crestShift(j, d) { return 0.07 * (noise(d * 1.2 + j * 7.7, 22) - 0.5) }
        function footShift(j, d) { return 0.035 * (noise(d * 1.4 + j * 5.3, 23) - 0.5) }

        // Ridge crests sit half a row either side of the plant's furrow. Only the
        // face of each ridge that turns toward us can be seen, so the outer rows
        // are drawn first and the inner ones over them.
        var ridges = []
        for (var j = -48; j <= 48; j++)
            ridges.push({ j: j, c: plantLateral + rowSpacing / 2 + j * rowSpacing })
        ridges.sort(function (a, b) { return Math.abs(b.c) - Math.abs(a.c) })

        // One clod: an irregular lump with a shadow under it, a body, and a
        // lighter top where the light catches it. Sizes are in metres.
        function clod(across, up, ahead, size, seed, body, top, shade) {
            var at = project(across, up, ahead)
            var r = focal * size / ahead
            if (r < 0.9 || at.x < -r * 2 || at.x > W + r * 2)
                return
            var corners = 6
            function outline(cx, cy, scale) {
                ctx.beginPath()
                for (var v = 0; v < corners; v++) {
                    var angle = (v + 0.35 * hash(seed, v)) / corners * Math.PI * 2
                    var reach = r * scale * (0.65 + 0.5 * hash(seed, v + 9))
                    var x = cx + Math.cos(angle) * reach, y = cy + Math.sin(angle) * reach * 0.72
                    v === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y)
                }
                ctx.closePath()
            }
            ctx.fillStyle = shade
            outline(at.x + r * 0.18, at.y + r * 0.32, 1.0)
            ctx.fill()
            ctx.fillStyle = body
            outline(at.x, at.y, 1.0)
            ctx.fill()
            if (r >= 2.5) {
                ctx.fillStyle = top
                outline(at.x - r * 0.18, at.y - r * 0.24, 0.55)
                ctx.fill()
            }
        }

        // Clods are drawn out to where they would be smaller than a pixel.
        var clodLimit = Math.min(to, 16)

        for (var r = 0; r < ridges.length; r++) {
            var j0 = ridges[r].j, c = ridges[r].c
            var side = c > 0 ? -1 : 1
            // The face toward the middle of the view: a ridge to the right of
            // straight ahead shows its left face, and the other way round.
            ctx.beginPath()
            for (var k = 0; k < depths.length; k++) {
                var p = project(c + side * ridgeHalfWidth + footShift(j0, depths[k]), 0, depths[k])
                k === 0 ? ctx.moveTo(p.x, p.y) : ctx.lineTo(p.x, p.y)
            }
            for (var q = depths.length - 1; q >= 0; q--) {
                var dq = depths[q]
                var t = project(c + crestShift(j0, dq), crestHeight(j0, dq), dq)
                ctx.lineTo(t.x, t.y)
            }
            ctx.closePath()
            // The light comes from the left, so faces looking left are lit.
            var face = c > 0 ? litFace : shadeFace
            ctx.fillStyle = face
            ctx.fill()
            // The foot of the face lies in the furrow's shadow, a dark line that
            // runs off to the horizon with every row.
            ctx.strokeStyle = furrowShade
            for (var e = 0; e < depths.length - 1; e++) {
                var a = project(c + side * ridgeHalfWidth * 0.8 + footShift(j0, depths[e]), 0.012, depths[e])
                var b = project(c + side * ridgeHalfWidth * 0.8 + footShift(j0, depths[e + 1]), 0.012, depths[e + 1])
                ctx.beginPath()
                ctx.moveTo(a.x, a.y)
                ctx.lineTo(b.x, b.y)
                ctx.lineWidth = Math.max(0.6, Math.min(4, focal * 0.025 / depths[e]))
                ctx.stroke()
            }

            // Clods along the ridge, from far to near so nearer ones lie over
            // farther ones, and a few loose ones in the furrow at its foot.
            var list = []
            var n = 0
            for (var d = Math.max(from, 0.9); d < clodLimit; n++) {
                var h1 = hash(j0 * 3.1 + n, 1.7)
                list.push({ d: d, n: n })
                d += 0.1 + 0.2 * h1
            }
            for (var i2 = list.length - 1; i2 >= 0; i2--) {
                var dd = list[i2].d, nn = list[i2].n
                var seed = j0 * 1000 + nn
                var frac = hash(seed, 2)
                // Up the face from foot to crest; most clods sit high on it.
                var place = 0.5 + 0.5 * Math.sqrt(frac)
                var across = c + crestShift(j0, dd) + side * ridgeHalfWidth * (1 - place)
                var up = crestHeight(j0, dd) * place
                var size = 0.018 + 0.035 * hash(seed, 3)
                var tone = 0.82 + 0.36 * hash(seed, 4)
                var body = Qt.rgba(face.r * tone, face.g * tone, face.b * tone, 1)
                var top = mix(Qt.lighter(body, c > 0 ? 1.35 : 1.15), snowColor, cover)
                clod(across, up, dd, size, seed, body, top, Qt.darker(face, 1.6))
                if (hash(seed, 5) < 0.18) {
                    var loose = c + side * (ridgeHalfWidth + 0.02 + 0.06 * hash(seed, 6))
                    clod(loose, 0.01, dd, size * 0.7, seed + 0.5, Qt.darker(floorColor, 0.85), floorColor, furrowShade)
                }
            }
        }
        ctx.restore()
    }
}
