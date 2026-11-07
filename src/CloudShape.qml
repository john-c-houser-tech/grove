import QtQuick

// One cloud, painted as many overlapping soft puffs so it has no outline at
// all, only a feathered edge. The seed picks the kind and every lump of it:
// cumulus heaps, long stratus bands, or thin wisps of cirrus.
Canvas {
    id: cloud

    property int seed: 1
    property color tone: "white"
    property color shade: "gray"

    onToneChanged: requestPaint()
    onShadeChanged: requestPaint()
    onWidthChanged: requestPaint()
    onHeightChanged: requestPaint()
    onSeedChanged: requestPaint()

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

    function css(c, alpha) {
        return "rgba(" + Math.round(c.r * 255) + "," + Math.round(c.g * 255) + "," + Math.round(c.b * 255) + "," + alpha + ")"
    }

    onPaint: {
        var ctx = getContext("2d")
        ctx.reset()
        var w = width, h = height
        if (w <= 0 || h <= 0)
            return
        var r = random(seed * 7919 + 13)
        var kind = seed % 3

        // A soft puff: a radial fade, stretched sideways when sx is above 1.
        function puff(cx, cy, rad, sx, color, alpha) {
            // A puff that ran past the edge of the canvas would be sliced off in
            // a hard line, so shrink it to fit.
            rad = Math.max(1, Math.min(rad, cy, h - cy, cx, w - cx))
            sx = Math.max(1, Math.min(sx, cx / rad, (w - cx) / rad))
            ctx.save()
            ctx.translate(cx, cy)
            ctx.scale(sx, 1)
            var g = ctx.createRadialGradient(0, 0, rad * 0.05, 0, 0, rad)
            g.addColorStop(0, css(color, alpha))
            g.addColorStop(0.5, css(color, alpha * 0.45))
            g.addColorStop(1, css(color, 0))
            ctx.fillStyle = g
            ctx.beginPath()
            ctx.arc(0, 0, rad, 0, Math.PI * 2)
            ctx.fill()
            ctx.restore()
        }

        if (kind === 0) {
            // Cumulus: a lumpy heap with a flat-ish base and frayed ends.
            var bottom = h * 0.8
            var n = 10 + Math.floor(r() * 6)
            for (var i = 0; i < n; i++) {
                var t = (i + 0.5) / n
                var profile = Math.sin(Math.PI * t)
                var rad = h * (0.15 + 0.3 * profile * (0.5 + 0.5 * r()))
                var x = w * (0.1 + 0.8 * t) + (r() - 0.5) * w * 0.05
                puff(x, bottom - rad * (0.5 + 0.35 * r()), rad * 1.3, 1 + r() * 0.3, tone, 0.62)
            }
            var tops = 3 + Math.floor(r() * 4)
            for (var m = 0; m < tops; m++)
                puff(w * (0.22 + 0.56 * r()), bottom - h * (0.3 + 0.25 * r()), h * (0.12 + 0.14 * r()), 1 + r() * 0.3, tone, 0.55)
            // The underside sits in shadow.
            puff(w * 0.5, bottom - h * 0.02, h * 0.2, w / (h * 0.2) * 0.42, shade, 0.4)
            var frays = 5 + Math.floor(r() * 4)
            for (var k = 0; k < frays; k++) {
                var left = r() < 0.5
                puff(w * (left ? 0.0 + 0.22 * r() : 0.78 + 0.22 * r()), bottom - h * (0.02 + 0.22 * r()),
                     h * (0.08 + 0.1 * r()), 3 + r() * 5, tone, 0.3)
            }
        } else if (kind === 1) {
            // Stratus: a long, low band with the edges trailing off in streaks.
            var count = 8 + Math.floor(r() * 5)
            for (var s = 0; s < count; s++) {
                var ts = (s + 0.5) / count
                puff(w * (0.08 + 0.84 * ts) + (r() - 0.5) * w * 0.05, h * (0.5 + (r() - 0.5) * 0.14),
                     h * (0.12 + 0.14 * r()), 2.5 + r() * 3, tone, 0.45)
            }
            puff(w * 0.5, h * 0.64, h * 0.16, w / (h * 0.16) * 0.4, shade, 0.28)
            var streaks = 7 + Math.floor(r() * 4)
            for (var q = 0; q < streaks; q++)
                puff(w * (0.02 + 0.96 * r()), h * (0.3 + 0.5 * r()), h * (0.06 + 0.07 * r()), 5 + r() * 5, tone, 0.24)
        } else {
            // Cirrus: thin, hooked streaks with nothing solid in them.
            var strokes = 6 + Math.floor(r() * 5)
            for (var c = 0; c < strokes; c++) {
                var x0 = w * (0.05 + 0.55 * r())
                var y0 = h * (0.2 + 0.6 * r())
                var bend = (r() - 0.5) * 0.5
                var pieces = 3 + Math.floor(r() * 3)
                for (var j = 0; j < pieces; j++)
                    puff(x0 + j * w * 0.09, y0 + j * j * h * 0.03 * bend, h * (0.06 + 0.05 * r()), 4 + r() * 4, tone, 0.26)
            }
        }
    }
}
