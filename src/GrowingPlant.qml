import QtQuick

// A plant drawn as a pure function of progress and seed: the same seed always
// gives the same plant, and progress only decides how much of it exists yet.
Canvas {
    id: plant

    property double progress: 0
    property int seed: 1
    property bool wilted: false
    property color flowerColor: "#FFD60A"
    // Where the trunk meets the ground, as a fraction of the height.
    property real base: 0.88
    // Where the trunk is across the width, as a fraction of it.
    property real rootX: 0.5
    // The light on the plant: a blue wash at night, a golden one at dawn and dusk.
    property color nightTint: "transparent"
    property color warmTint: "transparent"
    // Grey and damp when it is overcast; the plant sits in the weather too.
    property color weatherTint: "transparent"
    // Leaves change colour with the season, and frost settles in winter.
    property string season: "summer"

    // Segments are cached per seed and size; growing never rebuilds them.
    property var segments: []
    property var tips: []
    property int builtSeed: -1
    property real builtHeight: 0
    property real builtBase: 0
    property real builtRootX: 0

    onProgressChanged: requestPaint()
    onBaseChanged: requestPaint()
    onRootXChanged: requestPaint()
    onWiltedChanged: requestPaint()
    onFlowerColorChanged: requestPaint()
    onNightTintChanged: requestPaint()
    onWarmTintChanged: requestPaint()
    onWeatherTintChanged: requestPaint()
    onSeasonChanged: requestPaint()

    function lit(color) { return Qt.tint(Qt.tint(Qt.tint(color, weatherTint), nightTint), warmTint) }

    // A leaf's colour is the season's palette indexed by its own tone, so the
    // same leaf is always the same colour.
    readonly property var palettes: ({
        spring: ["#8cc46a", "#9ad07a", "#7ab85c"],
        summer: ["#79b05e", "#6aa352", "#86b866"],
        autumn: ["#d9822b", "#c4452c", "#e0b13a", "#8f9a3c"],
        winter: ["#8fa6a8", "#a9bcc0", "#7e9396"]
    })
    function leafColor(tone) {
        var list = palettes[season] || palettes.summer
        return list[Math.min(list.length - 1, Math.floor(tone * list.length))]
    }
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
        // The bow of each stem comes from a generator of its own, so adding it
        // left every plant's branching exactly as it was.
        var bends = random((seed ^ 0x5bd1e995) >>> 0)
        var segs = [], ends = []
        var maxDepth = 6
        var unit = height
        // Stroke widths were drawn for a plant about 670 pixels tall; a tile in
        // the garden is much smaller and its branches must thin with it.
        var scale = Math.max(0.15, height / 670)

        function grow(x, y, angle, length, depth, t0) {
            var t1 = depth === 0 ? 0.22 : t0 + 0.13
            var ex = x + Math.cos(angle) * length
            var ey = y + Math.sin(angle) * length
            var leaves = []
            if (depth >= 2) {
                var count = 1 + Math.floor(r() * 2)
                for (var i = 0; i < count; i++)
                    leaves.push({ f: 0.35 + r() * 0.6, side: r() < 0.5 ? -1 : 1, size: 0.75 + r() * 0.5, tone: r() })
            }
            // No stem grows dead straight: each one bows a little to one side,
            // the main stem least. The control point sits off the middle.
            var lean = bends()
            var bow = depth === 0
                    ? (lean < 0.5 ? -1 : 1) * (0.05 + 0.06 * Math.abs(lean - 0.5) * 2) * length
                    : (lean - 0.5) * 0.22 * length
            var cx = (x + ex) / 2 - Math.sin(angle) * bow
            var cy = (y + ey) / 2 + Math.cos(angle) * bow
            var w = Math.max(1.4 * scale, 9 * scale * Math.pow(0.62, depth))
            // The main stem and first branches thin toward their tips.
            segs.push({ x: x, y: y, cx: cx, cy: cy, ex: ex, ey: ey, t0: t0, t1: t1,
                        width: w, tipWidth: depth <= 1 ? w * 0.68 : w, leaves: leaves })
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

        grow(width * rootX, height * base, -Math.PI / 2, unit * 0.2, 0, 0)
        segments = segs
        tips = ends
        builtSeed = seed
        builtHeight = height
        builtBase = base
        builtRootX = rootX
    }

    // A point along a stem, which is a gentle curve, t from 0 at its foot to 1.
    function along(s, t) {
        var u = 1 - t
        return { x: u * u * s.x + 2 * u * t * s.cx + t * t * s.ex, y: u * u * s.y + 2 * u * t * s.cy + t * t * s.ey }
    }
    // Which way the stem is heading at t.
    function headingAt(s, t) {
        var dx = 2 * (1 - t) * (s.cx - s.x) + 2 * t * (s.ex - s.cx)
        var dy = 2 * (1 - t) * (s.cy - s.y) + 2 * t * (s.ey - s.cy)
        return Math.atan2(dy, dx)
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
        if (seed !== builtSeed || Math.abs(height - builtHeight) > 0.5 || Math.abs(base - builtBase) > 0.0005
                || Math.abs(rootX - builtRootX) > 0.0005)
            build()

        // A wilted plant keeps the shape it reached but loses its colour.
        var stem = lit(wilted ? "#7d6a45" : "#5f8f4a")
        var wiltedLeaf = lit("#8a7647")
        var leafScale = wilted ? 0.6 : 1
        ctx.lineCap = "round"

        if (progress <= 0)
            return

        for (var i = 0; i < segments.length; i++) {
            var s = segments[i]
            var p = clamp((progress - s.t0) / (s.t1 - s.t0))
            if (p <= 0)
                continue
            // The stem as far as it has grown, in short pieces so a tapering one
            // can thin as it goes; round ends hide the joins.
            var pieces = s.tipWidth < s.width ? 6 : 3
            ctx.strokeStyle = stem
            for (var k = 0; k < pieces; k++) {
                var ta = p * k / pieces, tb = p * (k + 1) / pieces
                var a = along(s, ta), b = along(s, tb)
                ctx.beginPath()
                ctx.moveTo(a.x, a.y)
                ctx.lineTo(b.x, b.y)
                ctx.lineWidth = s.width + (s.tipWidth - s.width) * (ta + tb) / 2
                ctx.stroke()
            }
            if (season === "winter" && !wilted) {
                // Frost rimes the upper edge of each branch.
                ctx.strokeStyle = lit("#e6f1f4")
                ctx.lineWidth = Math.max(1, s.width * 0.3)
                ctx.beginPath()
                for (var fk = 0; fk <= pieces; fk++) {
                    var fp = along(s, p * fk / pieces)
                    fk === 0 ? ctx.moveTo(fp.x, fp.y - s.width * 0.25) : ctx.lineTo(fp.x, fp.y - s.width * 0.25)
                }
                ctx.stroke()
            }

            for (var j = 0; j < s.leaves.length; j++) {
                var l = s.leaves[j]
                // Winter strips most of the leaves.
                if (season === "winter" && !wilted && l.tone > 0.3)
                    continue
                var grown = clamp((p - l.f) * 4)
                if (grown <= 0)
                    continue
                var at = along(s, l.f)
                var tilt = wilted ? 1.5 : 0.9
                drawLeaf(ctx, at.x, at.y, headingAt(s, l.f) + l.side * tilt, height * 0.035 * l.size * grown * leafScale, wilted ? wiltedLeaf : lit(leafColor(l.tone)))
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
