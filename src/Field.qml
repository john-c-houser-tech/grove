import QtQuick

// The ploughed field in the foreground, seen in true perspective by someone
// standing at the edge of it, eyes 1.6 m above the ground.
//
// The rows run straight away from the viewer and meet at the horizon. They
// are rounded ridges 70 cm apart and 20 cm high, with a narrow trough
// between them, wavering a little along their length as ploughed rows do. The plant is a sapling about
// 2.5 m tall, standing in the furrow between two rows, 4 m away. Every size
// here follows from those numbers, so the rows, the plant and the trunk agree.
Canvas {
    id: field

    // The field takes a while to draw; drawing it on its own thread keeps
    // the rest of the window moving meanwhile.
    renderStrategy: Canvas.Threaded

    // The horizon, as a fraction of the height, and where the field's far
    // edge meets the meadow, in pixels from the top.
    property real horizon: 0.6
    property real groundTop: height * 0.874
    // Where the rows meet on the horizon, and where the plant stands, as
    // fractions of the width.
    property real vanishX: 0.5
    property real plantX: 0.33
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
    // with the plant 4 m off fixes the focal length. The eye is at standing
    // height, so the horizon crosses the plant 1.6 m up its trunk.
    readonly property real metreAtPlant: height * 0.21
    readonly property real plantDepth: 4
    readonly property real focal: metreAtPlant * plantDepth
    readonly property real eye: 1.6
    // How far the plant stands to the side of straight ahead, in metres.
    readonly property real plantLateral: (plantX - vanishX) * width / metreAtPlant
    // The field ends this far away, where the meadow takes over, and this is
    // where that edge falls on the screen.
    readonly property real fieldDepth: 40
    readonly property real farEdgeY: height * horizon + focal * eye / fieldDepth
    // Where the plant's foot is, in pixels from the top.
    readonly property real plantBaseY: height * horizon + focal * eye / plantDepth

    readonly property real rowSpacing: 0.7
    readonly property real ridgeHeight: 0.2
    // Ridges 60 cm across at the base, leaving a 10 cm trough between them.
    readonly property real ridgeHalfWidth: 0.3

    onSoilColorChanged: repaintSoon()
    onHazeColorChanged: repaintSoon()
    onSnowChanged: repaintSoon()
    onFrostChanged: repaintSoon()
    onWidthChanged: requestPaint()
    onHeightChanged: requestPaint()
    onGroundTopChanged: requestPaint()
    onPlantXChanged: requestPaint()
    onPartChanged: requestPaint()

    // The field takes the better part of a tenth of a second to draw, and the
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
        // The soil's own colour runs from the dark of the troughs to the warm
        // brown of a crest in sun; snow lightens it all, the tops most.
        var deep = Qt.darker(soilColor, 1.5)
        var bright = Qt.lighter(soilColor, 2.7)
        var floorColor = mix(mix(deep, bright, 0.14), snowColor, cover * 0.6)
        var furrowShade = Qt.darker(floorColor, 1.35)
        function tone(level, height) {
            var c = mix(deep, bright, Math.max(0, Math.min(1, level)))
            return mix(c, snowColor, cover * (0.55 + 0.45 * height))
        }
        var distantTone = mix(tone(0.55, 0.6), hazeColor, 0.06)
        ctx.save()
        ctx.beginPath()
        ctx.rect(0, groundTop, W, H - groundTop)
        ctx.clip()
        // Far off, ridges and furrows blur into one average tone with a little
        // haze in it, so the floor lightens toward the far edge of the field.
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
        var nearest = 2.4
        var from = part === "far" ? plantDepth : nearest
        // The near rows run a hair past the plant so the two halves meet without a seam.
        var to = part === "far" ? fieldDepth : plantDepth * 1.01
        var steps = part === "far" ? 40 : 30
        var depths = []
        for (var i = 0; i <= steps; i++)
            depths.push(from * Math.pow(to / from, i / steps))

        // The rows waver together in long, gentle curves, as a plough drawn by
        // eye does, each with a little unevenness of its own. The curve is
        // measured from the plant, so the plant stays in the middle of its trough.
        function wave(d) { return 0.07 * Math.sin(d * 0.35 + 1.3) + 0.025 * Math.sin(d * 0.9 + 0.4) }
        function shift(j, d) { return wave(d) - wave(plantDepth) + 0.012 * (noise(d * 0.6 + j * 7.7, 22) - 0.5) }
        function crestHeight(j, d) { return ridgeHeight * (0.85 + 0.3 * noise(d * 0.9 + j * 13.1, 21)) }

        // Crests sit half a row either side of the plant's trough. Only the side
        // of each ridge that turns toward us can be seen, so the outer rows are
        // drawn first and the inner ones over them.
        var ridges = []
        for (var j = -36; j <= 36; j++)
            ridges.push({ j: j, c: plantLateral + rowSpacing / 2 + j * rowSpacing })
        ridges.sort(function (a, b) { return Math.abs(b.c) - Math.abs(a.c) })

        // Light comes from the upper left. Across a rounded ridge the slope
        // turns from flat at the crest to steep at the foot, so the side facing
        // the light brightens down from the crest, the other side darkens, and
        // both fall into shadow in the trough.
        var bands = 8
        function profile(t) { return 1 - t * t }
        function brightness(t, facingLight) {
            var slope = 2 * ridgeHeight * t / ridgeHalfWidth
            var len = Math.sqrt(1 + slope * slope)
            var nx = (facingLight ? -slope : slope) / len, nz = 1 / len
            var lit = Math.max(0, nx * -0.55 + nz * 0.83)
            return (0.25 + 0.75 * lit) * (1 - 0.45 * t * t)
        }

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

        // Clods and grain are drawn out to where they would be under a pixel.
        var clodLimit = Math.min(to, 14)
        var segments = 8

        for (var r = 0; r < ridges.length; r++) {
            var j0 = ridges[r].j, c = ridges[r].c
            var side = c > 0 ? -1 : 1
            var facingLight = c > 0
            // From standing height you look down into the rows, so the far side
            // of a ridge shows too, down to where it is steeper than your line
            // of sight across it. Looking straight along a row you see all of
            // it; rows off to the side show only their tops.
            var across = Math.max(0.05, Math.abs(c))
            var backReach = Math.min(1, (eye / across) * ridgeHalfWidth / (2 * ridgeHeight))
            function strip(sideSign, t0, t1, fill, lo, hi) {
                ctx.beginPath()
                for (var k = lo; k <= hi; k++) {
                    var dk = depths[k], ch = crestHeight(j0, dk)
                    var p = project(c + shift(j0, dk) + sideSign * ridgeHalfWidth * t0, ch * profile(t0), dk)
                    k === lo ? ctx.moveTo(p.x, p.y) : ctx.lineTo(p.x, p.y)
                }
                for (var q = hi; q >= lo; q--) {
                    var dq = depths[q], cq = crestHeight(j0, dq)
                    var pq = project(c + shift(j0, dq) + sideSign * ridgeHalfWidth * t1, cq * profile(t1), dq)
                    ctx.lineTo(pq.x, pq.y)
                }
                ctx.closePath()
                ctx.fillStyle = fill
                ctx.fill()
            }
            // A ridge runs straight in depth, so if both ends of a stretch are
            // off the same side of the window, all of it is.
            function offscreen(lo, hi) {
                var a = project(c, 0, depths[lo]).x, b = project(c, 0, depths[hi]).x
                var margin = focal * ridgeHalfWidth * 1.5 / depths[lo]
                return (a < -margin && b < -margin) || (a > W + margin && b > W + margin)
            }
            for (var gb = 0; gb < segments; gb++) {
                var blo = Math.floor(gb * (depths.length - 1) / segments)
                var bhi = Math.floor((gb + 1) * (depths.length - 1) / segments)
                if (offscreen(blo, bhi))
                    continue
                var bmid = depths[Math.floor((blo + bhi) / 2)]
                var bhaze = Math.min(1, Math.max(0, (bmid - 8) / 30))
                // As many shading bands as the ridge is wide on screen allows.
                var bwide = focal * ridgeHalfWidth * backReach / bmid
                if (bwide < 1)
                    continue
                var bn = Math.max(1, Math.min(bands, Math.round(bwide / 4)))
                for (var bb = bn - 1; bb >= 0; bb--) {
                    var b0 = bb / bn * backReach, b1 = (bb + 1) / bn * backReach
                    var bm = (b0 + b1) / 2
                    strip(-side, b0, b1, mix(tone(brightness(bm, !facingLight) * 1.15, profile(bm)), distantTone, bhaze), blo, bhi)
                }
            }

            // The ridge in bands from crest to foot, and in stretches from far to
            // near so the far ones can take on the soil's distant tone.
            for (var g = 0; g < segments; g++) {
                var lo = Math.floor(g * (depths.length - 1) / segments)
                var hi = Math.floor((g + 1) * (depths.length - 1) / segments)
                if (offscreen(lo, hi))
                    continue
                var mid = depths[Math.floor((lo + hi) / 2)]
                var haze = Math.min(1, Math.max(0, (mid - 8) / 30))
                var wide = focal * ridgeHalfWidth / mid
                var fn = Math.max(2, Math.min(bands, Math.round(wide / 4)))
                for (var band = 0; band < fn; band++) {
                    var t0 = band / fn, t1 = (band + 1) / fn
                    var tm = (t0 + t1) / 2
                    strip(side, t0, t1, mix(tone(brightness(tm, facingLight) * 1.15, profile(tm)), distantTone, haze), lo, hi)
                }
            }

            // Fine grain over the near part of the ridge: specks of soil a
            // centimetre or two across, a little lighter or darker than the
            // ground under them.
            var count = 0
            for (var gd = Math.max(from, 0.8); gd < Math.min(clodLimit, 9) && count < 120; count++) {
                var gseed = j0 * 977 + count
                var gt = Math.sqrt(hash(gseed, 11))
                var ghere = crestHeight(j0, gd)
                var gp = project(c + shift(j0, gd) + side * ridgeHalfWidth * gt, ghere * profile(gt), gd)
                var speck = Math.max(1, focal * (0.008 + 0.012 * hash(gseed, 12)) / gd)
                if (gp.x > -4 && gp.x < W + 4) {
                    var base = tone(brightness(gt, facingLight) * 1.15, profile(gt))
                    ctx.fillStyle = hash(gseed, 13) < 0.5 ? Qt.lighter(base, 1.25) : Qt.darker(base, 1.3)
                    ctx.fillRect(gp.x, gp.y, speck, speck * 0.7)
                }
                gd += 0.025 + 0.06 * hash(gseed, 14) * gd / 3
            }

            // A few small clods close by, high on the ridge.
            for (var cd = Math.max(from, 0.9), cn = 0; cd < Math.min(to, 6); cn++) {
                var cseed = j0 * 1000 + cn
                var ct = 0.15 + 0.6 * hash(cseed, 2)
                var chh = crestHeight(j0, cd)
                var csize = 0.015 + 0.025 * hash(cseed, 3)
                var cbody = tone(brightness(ct, facingLight) * 1.05, profile(ct))
                clod(c + shift(j0, cd) + side * ridgeHalfWidth * ct, chh * profile(ct), cd, csize, cseed,
                     cbody, Qt.lighter(cbody, 1.25), Qt.darker(cbody, 1.5))
                cd += 0.35 + 0.6 * hash(cseed, 1)
            }
        }
        ctx.restore()
    }
}
