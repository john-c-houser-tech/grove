import QtQuick

// A plant drawn as a pure function of progress and seed: the same seed always
// gives the same plant, and progress only decides how much of it exists yet.
Canvas {
    id: plant

    property double progress: 0
    property int seed: 1
    property bool wilted: false
    property color flowerColor: "#FFD60A"
    // The light on the plant: a blue wash at night, a golden one at dawn and dusk.
    property color nightTint: "transparent"
    property color warmTint: "transparent"

    // Segments are cached per seed and size; growing never rebuilds them.
    property var segments: []
    property var tips: []
    property int builtSeed: -1
    property real builtHeight: 0

    onProgressChanged: requestPaint()
    onWiltedChanged: requestPaint()
    onFlowerColorChanged: requestPaint()
    onNightTintChanged: requestPaint()
    onWarmTintChanged: requestPaint()

    function lit(color) { return Qt.tint(Qt.tint(color, nightTint), warmTint) }
    onWidthChanged: requestPaint()
    onHeightChanged: requestPaint()

    // Small deterministic generator (mulberry32), so a seed is a plant.
    function random(seedValue) {
        var a = seedValue >>> 0
        return function () {
            a = (a + 0x6D2B79F5) >>> 0
            var t = a
            t = Math.imul(t ^ (t >>> 15), t | 1)
            t ^= t + Math.imul(t ^ (t >>> 7), t | 61)
            return ((t ^ (t >>> 14)) >>> 0) / 4294967296
        }
    }

    function clamp(v) { return Math.max(0, Math.min(1, v)) }

    // The trunk takes the first fifth of the session; each level of branches
    // takes an equal share of the rest, so the last twigs finish as time ends.
    function build() {
        var r = random(seed)
        var segs = [], ends = []
        var maxDepth = 6
        var unit = height

        function grow(x, y, angle, length, depth, t0) {
            var t1 = depth === 0 ? 0.22 : t0 + 0.13
            var ex = x + Math.cos(angle) * length
            var ey = y + Math.sin(angle) * length
            var leaves = []
            if (depth >= 2) {
                var count = 1 + Math.floor(r() * 2)
                for (var i = 0; i < count; i++)
                    leaves.push({ f: 0.35 + r() * 0.6, side: r() < 0.5 ? -1 : 1, size: 0.75 + r() * 0.5 })
            }
            segs.push({ x: x, y: y, ex: ex, ey: ey, t0: t0, t1: t1,
                        width: Math.max(1.4, 9 * Math.pow(0.62, depth)), leaves: leaves })
            if (depth === maxDepth) {
                // Most twigs stay leafy; only a few carry a flower.
                if (r() < 0.1)
                    ends.push({ x: ex, y: ey, size: 1.0 + r() * 0.6 })
                return
            }
            var n = depth === 0 ? 3 : (r() < 0.7 ? 2 : 3)
            for (var k = 0; k < n; k++) {
                var turn = (k - (n - 1) / 2) * 0.62 + (r() - 0.5) * 0.35
                var shrink = 0.68 + r() * 0.1
                grow(ex, ey, angle + turn, length * shrink, depth + 1, t1)
            }
        }

        grow(width / 2, height * 0.88, -Math.PI / 2, unit * 0.2, 0, 0)
        segments = segs
        tips = ends
        builtSeed = seed
        builtHeight = height
    }

    function drawLeaf(ctx, x, y, angle, size, color) {
        ctx.save()
        ctx.translate(x, y)
        ctx.rotate(angle)
        ctx.beginPath()
        ctx.moveTo(0, 0)
        ctx.quadraticCurveTo(size * 0.5, -size * 0.45, size, 0)
        ctx.quadraticCurveTo(size * 0.5, size * 0.45, 0, 0)
        ctx.fillStyle = color
        ctx.fill()
        ctx.restore()
    }

    function drawFlower(ctx, x, y, size) {
        for (var i = 0; i < 5; i++) {
            var a = i * Math.PI * 2 / 5
            ctx.beginPath()
            ctx.arc(x + Math.cos(a) * size * 0.55, y + Math.sin(a) * size * 0.55, size * 0.5, 0, Math.PI * 2)
            ctx.fillStyle = lit(flowerColor)
            ctx.fill()
        }
        ctx.beginPath()
        ctx.arc(x, y, size * 0.32, 0, Math.PI * 2)
        ctx.fillStyle = lit("#fff4c4")
        ctx.fill()
    }

    onPaint: {
        var ctx = getContext("2d")
        ctx.reset()
        if (width <= 0 || height <= 0)
            return
        if (seed !== builtSeed || Math.abs(height - builtHeight) > 0.5)
            build()

        // A wilted plant keeps the shape it reached but loses its colour.
        var stem = lit(wilted ? "#7d6a45" : "#5f8f4a")
        var leaf = lit(wilted ? "#8a7647" : "#79b05e")
        var leafScale = wilted ? 0.6 : 1
        ctx.lineCap = "round"

        if (progress <= 0)
            return

        for (var i = 0; i < segments.length; i++) {
            var s = segments[i]
            var p = clamp((progress - s.t0) / (s.t1 - s.t0))
            if (p <= 0)
                continue
            ctx.beginPath()
            ctx.moveTo(s.x, s.y)
            ctx.lineTo(s.x + (s.ex - s.x) * p, s.y + (s.ey - s.y) * p)
            ctx.lineWidth = s.width
            ctx.strokeStyle = stem
            ctx.stroke()

            var heading = Math.atan2(s.ey - s.y, s.ex - s.x)
            for (var j = 0; j < s.leaves.length; j++) {
                var l = s.leaves[j]
                var grown = clamp((p - l.f) * 4)
                if (grown <= 0)
                    continue
                var lx = s.x + (s.ex - s.x) * l.f
                var ly = s.y + (s.ey - s.y) * l.f
                var tilt = wilted ? 1.5 : 0.9
                drawLeaf(ctx, lx, ly, heading + l.side * tilt, height * 0.035 * l.size * grown * leafScale, leaf)
            }
        }

        if (!wilted) {
            var bloom = clamp((progress - 0.9) / 0.1)
            if (bloom > 0)
                for (var t = 0; t < tips.length; t++)
                    drawFlower(ctx, tips[t].x, tips[t].y, height * 0.026 * tips[t].size * bloom)
        }
    }
}
