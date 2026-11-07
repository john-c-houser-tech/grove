import QtQuick

// The ploughed field in the foreground, seen in true perspective from low
// down, about 30 cm above the ground, as if sitting at the edge of it.
//
// The rows run straight away from the viewer and meet at the horizon. They
// are ploughed furrows 40 cm apart with ridges 12 cm high. The plant is a sapling about
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
    // Where the plant's foot is, in pixels from the top.
    readonly property real plantBaseY: height * horizon + focal * eye / plantDepth

    readonly property real rowSpacing: 0.4
    readonly property real ridgeHeight: 0.12
    readonly property real ridgeHalfWidth: 0.2

    onSoilColorChanged: requestPaint()
    onSnowChanged: requestPaint()
    onFrostChanged: requestPaint()
    onWidthChanged: requestPaint()
    onHeightChanged: requestPaint()
    onGroundTopChanged: requestPaint()
    onPlantXChanged: requestPaint()
    onPartChanged: requestPaint()

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
        var floorColor = mix(Qt.darker(soilColor, 1.5), snowColor, cover * 0.6)
        var litFace = mix(Qt.lighter(soilColor, 1.9), snowColor, cover)
        var shadeFace = mix(Qt.lighter(soilColor, 1.15), snowColor, cover * 0.75)
        var crestColor = mix(Qt.lighter(soilColor, 2.6), snowColor, cover)

        ctx.save()
        ctx.beginPath()
        ctx.rect(0, groundTop, W, H - groundTop)
        ctx.clip()
        if (part === "far") {
            ctx.fillStyle = floorColor
            ctx.fillRect(0, groundTop, W, H - groundTop)
        }

        // From just in front of the bottom of the window out to where the rows
        // are too close together to see, split at the plant.
        var nearest = focal * eye / (H - H * horizon) * 0.85
        var from = part === "far" ? plantDepth : nearest
        // The near rows run a hair past the plant so the two halves meet without a seam.
        var to = part === "far" ? 80 : plantDepth * 1.01
        var steps = part === "far" ? 22 : 10
        var depths = []
        for (var i = 0; i <= steps; i++)
            depths.push(from * Math.pow(to / from, i / steps))

        // Ridge crests sit half a row either side of the plant's furrow. Only the
        // face of each ridge that turns toward us can be seen, so the outer rows
        // are drawn first and the inner ones over them.
        var ridges = []
        for (var j = -48; j <= 48; j++) {
            var crest = plantLateral + rowSpacing / 2 + j * rowSpacing
            ridges.push(crest)
        }
        ridges.sort(function (a, b) { return Math.abs(b) - Math.abs(a) })

        for (var r = 0; r < ridges.length; r++) {
            var c = ridges[r]
            // The face toward the middle of the view: a ridge to the right of
            // straight ahead shows its left face, and the other way round.
            var foot = c > 0 ? c - ridgeHalfWidth : c + ridgeHalfWidth
            ctx.beginPath()
            for (var k = 0; k < depths.length; k++) {
                var p = project(foot, 0, depths[k])
                k === 0 ? ctx.moveTo(p.x, p.y) : ctx.lineTo(p.x, p.y)
            }
            for (var q = depths.length - 1; q >= 0; q--) {
                var t = project(c, ridgeHeight, depths[q])
                ctx.lineTo(t.x, t.y)
            }
            ctx.closePath()
            // The light comes from the left, so faces looking left are lit.
            ctx.fillStyle = c > 0 ? litFace : shadeFace
            ctx.fill()
            // The crest catches the light; it thins with distance.
            ctx.strokeStyle = crestColor
            for (var e = 0; e < depths.length - 1; e++) {
                var a = project(c, ridgeHeight, depths[e]), b = project(c, ridgeHeight, depths[e + 1])
                ctx.beginPath()
                ctx.moveTo(a.x, a.y)
                ctx.lineTo(b.x, b.y)
                ctx.lineWidth = Math.max(0.6, Math.min(3, focal * 0.012 / depths[e]))
                ctx.stroke()
            }
        }
        ctx.restore()
    }
}
