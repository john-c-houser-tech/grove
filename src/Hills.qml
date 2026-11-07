import QtQuick

// The view beyond the garden: distant snow-capped peaks, rolling ridges and a
// nearer hillside. The ridgelines come from fixed noise, so the landscape is
// always the same one; only its light changes. Each layer is hazier the
// farther away it is, fading toward the colour of the sky at the horizon.
Canvas {
    id: hills

    // Where the horizon sits, as a fraction of the height.
    property real horizon: 0.88
    // "back" draws the peaks and ridges, "front" the hillside nearest to us, so
    // something can pass between them.
    property string part: "all"
    // How low the snow reaches on the far peaks, as a fraction of their height.
    property real snowline: 0.65

    property color farColor: "#6b7f9c"
    property color midColor: "#46605a"
    property color nearColor: "#30493b"
    property color hazeColor: "#9fb7c4"
    property color snowColor: "#f2f6fa"

    onFarColorChanged: requestPaint()
    onMidColorChanged: requestPaint()
    onNearColorChanged: requestPaint()
    onHazeColorChanged: requestPaint()
    onSnowColorChanged: requestPaint()
    onSnowlineChanged: requestPaint()
    onWidthChanged: requestPaint()
    onHeightChanged: requestPaint()

    function hash(i, seed) {
        var x = Math.sin(i * 127.1 + seed * 311.7) * 43758.5453
        return x - Math.floor(x)
    }
    // Smooth one-dimensional value noise.
    function noise(x, seed) {
        var i = Math.floor(x), f = x - i
        var u = f * f * (3 - 2 * f)
        return hash(i, seed) * (1 - u) + hash(i + 1, seed) * u
    }
    // Ridged noise has sharp peaks and broad valleys, like a mountain range.
    function ridged(x, seed) {
        var v = 1 - Math.abs(2 * noise(x, seed) - 1)
        return v * v
    }
    function mix(a, b, t) {
        return Qt.rgba(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t, 1)
    }

    // Height of each layer's ridge above the horizon, in pixels, at fraction u
    // of the way across.
    // The far range, after the Tetons: no foothills, a wall of steep pointed
    // peaks rising straight from the valley, tallest in a cluster just right of
    // the middle and lower, still jagged, out to both edges. Each peak is its
    // middle, its top (fraction of the height above the horizon), and how
    // steeply it falls away on the left and on the right.
    readonly property var peaks: [
        [0.0, 0.215, 1.6, 1.8], [0.08, 0.235, 1.5, 2.0], [0.16, 0.222, 1.8, 1.6],
        [0.24, 0.25, 1.6, 2.1], [0.32, 0.265, 1.9, 1.7], [0.39, 0.285, 2.0, 2.4],
        [0.455, 0.335, 2.6, 2.2], [0.515, 0.302, 2.3, 1.9], [0.57, 0.287, 2.1, 1.8],
        [0.64, 0.255, 1.7, 1.9], [0.72, 0.242, 1.6, 1.7], [0.8, 0.228, 1.7, 1.5],
        [0.88, 0.24, 1.5, 1.9], [0.96, 0.222, 1.8, 1.6], [1.03, 0.21, 1.6, 1.6]
    ]
    function far(u, H) {
        var h = 0.13
        for (var i = 0; i < peaks.length; i++) {
            var p = peaks[i]
            var fall = u < p[0] ? (p[0] - u) * p[2] : (u - p[0]) * p[3]
            h = Math.max(h, p[1] - fall)
        }
        // A serrated skyline: granite breaks into teeth and notches.
        h += 0.011 * (ridged(u * 38 + 7, 9) - 0.45) + 0.005 * (noise(u * 95, 10) - 0.5)
        return H * h
    }
    function mid(u, H) {
        // A shoulder of hill rises behind the level crossing, so the road
        // beyond the rails has a slope to climb into.
        var shoulder = (u - 0.6) / 0.12
        return H * (0.035 + 0.15 * noise(u * 2.6 + 5, 4)) + H * 0.025 * noise(u * 8.2, 5)
               + H * 0.07 * Math.exp(-shoulder * shoulder)
    }
    function near(u, H) {
        // Two hills, one at each edge, with a low meadow between them. The
        // railway runs behind them, so a train comes out from behind one and
        // goes in behind the other.
        var left = Math.pow(1 - u, 3), right = Math.pow(u, 3)
        return H * (0.008 + 0.12 * left + 0.11 * right) + H * 0.012 * noise(u * 6.5 + 2, 6)
    }

    function paintRidge(ctx, heightAt, top, base, color, mist) {
        var W = width, H = height, y0 = H * horizon
        ctx.beginPath()
        ctx.moveTo(0, H)
        for (var x = 0; x <= W; x += 4)
            ctx.lineTo(x, y0 - heightAt(x / W, H))
        ctx.lineTo(W, H)
        ctx.closePath()
        // Mist gathers at the foot of each layer.
        var g = ctx.createLinearGradient(0, y0 - top, 0, y0 + 2)
        g.addColorStop(0, color)
        g.addColorStop(1, mix(color, hazeColor, mist))
        ctx.fillStyle = g
        ctx.fill()
    }

    onPartChanged: requestPaint()

    onPaint: {
        var ctx = getContext("2d")
        ctx.reset()
        var W = width, H = height
        if (W <= 0 || H <= 0)
            return
        var y0 = H * horizon
        if (part === "front") {
            paintRidge(ctx, near, H * 0.14, 0, nearColor, 0.18)
            return
        }

        // Far peaks, with snow above a ragged snowline.
        paintRidge(ctx, far, H * 0.34, 0, farColor, 0.55)
        ctx.save()
        ctx.beginPath()
        ctx.moveTo(0, H)
        for (var x = 0; x <= W; x += 4)
            ctx.lineTo(x, y0 - far(x / W, H))
        ctx.lineTo(W, H)
        ctx.closePath()
        ctx.clip()
        var snowY = y0 - H * 0.34 * snowline
        ctx.beginPath()
        ctx.moveTo(0, 0)
        ctx.lineTo(W, 0)
        for (var sx = W; sx >= 0; sx -= 4) {
            // The snowline wanders with the rock, so caps have jagged feet.
            var u = sx / W
            // Each face holds its snow differently: some low, some high.
            var wobble = H * (0.06 * (noise(u * 7, 7) - 0.5) + 0.03 * (noise(u * 23, 13) - 0.5) + 0.012 * (noise(u * 60, 8) - 0.5))
            ctx.lineTo(sx, snowY + wobble)
        }
        ctx.closePath()
        var cap = ctx.createLinearGradient(0, y0 - H * 0.34, 0, snowY)
        cap.addColorStop(0, snowColor)
        cap.addColorStop(1, mix(snowColor, farColor, 0.35))
        ctx.fillStyle = cap
        ctx.fill()
        // Below the snowfields, snow runs on down the steep gullies that score
        // the faces: long thin streaks leaving the snow, angling down the face
        // and thinning out to nothing.
        for (var pk = 0; pk < peaks.length; pk++) {
            var peak = peaks[pk]
            if (y0 - H * peak[1] > snowY - H * 0.015)
                continue
            for (var g = 0; g < 3; g++) {
                var side = g === 1 ? 1 : -1
                var slope = side < 0 ? peak[2] : peak[3]
                var gu = peak[0] + side * (0.006 + 0.016 * noise(pk * 2.1 + g * 5, 11))
                var gx = W * gu
                // Start inside the snow, a little above where it ends here.
                var gtop = Math.max(y0 - far(gu, H) + H * 0.01, snowY - H * 0.02)
                var len = H * (0.03 + 0.03 * noise(pk * 3.3 + g, 12))
                var drift = side * len * (0.18 + 0.25 / slope)
                var streak = ctx.createLinearGradient(0, gtop, 0, gtop + len)
                streak.addColorStop(0, Qt.rgba(snowColor.r, snowColor.g, snowColor.b, 0.65))
                streak.addColorStop(1, Qt.rgba(snowColor.r, snowColor.g, snowColor.b, 0))
                ctx.fillStyle = streak
                var half = H * 0.0018
                ctx.beginPath()
                ctx.moveTo(gx - half, gtop)
                ctx.lineTo(gx + half, gtop)
                ctx.quadraticCurveTo(gx + drift * 0.4 + half * 0.5, gtop + len * 0.5, gx + drift, gtop + len)
                ctx.quadraticCurveTo(gx + drift * 0.4 - half * 0.5, gtop + len * 0.5, gx - half, gtop)
                ctx.closePath()
                ctx.fill()
            }
        }
        ctx.restore()

        paintRidge(ctx, mid, H * 0.2, 0, midColor, 0.4)
        if (part === "all")
            paintRidge(ctx, near, H * 0.14, 0, nearColor, 0.18)
    }
}
