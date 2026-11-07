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
    // The far range is bare granite: lit rock, rock in shadow, and forest along
    // its foot. Which side of each peak is in shadow follows the sun (1 when
    // the right-hand faces are shaded), and relief is how strong the shadows
    // are, from 0 under cloud or at night to about 0.7 in clear sun.
    property color shadeColor: "#4d5a6c"
    property color forestColor: "#2f3e35"
    property int shadowSide: 1
    property real relief: 0.5
    // How much snow the range carries, from a few high patches (0) to fully
    // white (1).
    property real snowCover: 0.3
    property color snowColor: "#f2f6fa"

    onFarColorChanged: requestPaint()
    onMidColorChanged: requestPaint()
    onNearColorChanged: requestPaint()
    onHazeColorChanged: requestPaint()
    onSnowColorChanged: requestPaint()
    onSnowCoverChanged: requestPaint()
    onReliefChanged: requestPaint()
    onShadowSideChanged: requestPaint()
    onForestColorChanged: requestPaint()
    onShadeColorChanged: requestPaint()
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
    // The far range, after the Tetons: no foothills, a wall of blunt, blocky
    // granite summits rising straight from the valley on one broad mass that
    // is highest in the middle, the tallest summit just left of centre. Each
    // peak is its middle, its top (fraction of the height above the horizon),
    // and how steeply it falls away on the left and on the right.
    readonly property var peaks: [
        [0.0, 0.25, 1.3, 1.4], [0.17, 0.27, 1.4, 1.5], [0.32, 0.29, 1.5, 1.4],
        [0.465, 0.335, 1.6, 1.7], [0.585, 0.305, 1.5, 1.5], [0.71, 0.285, 1.4, 1.3],
        [0.85, 0.265, 1.3, 1.5], [1.0, 0.25, 1.4, 1.3]
    ]
    function peakHeight(p, u) {
        // A rounded, blunt summit that steepens into straight flanks.
        var d = u - p[0], r = 0.03
        return p[1] - (d < 0 ? p[2] : p[3]) * (Math.sqrt(d * d + r * r) - r)
    }
    function far(u, H) {
        var centre = (u - 0.5) / 0.32
        var h = 0.19 + 0.05 * Math.exp(-centre * centre)
        for (var i = 0; i < peaks.length; i++) {
            var p = peaks[i]
            h = Math.max(h, peakHeight(p, u))
            // Lesser tops on each flank: granite massifs are never one clean
            // pyramid.
            h = Math.max(h, peakHeight([p[0] - 0.048, p[1] - 0.038, 2.2, 2.4], u))
            h = Math.max(h, peakHeight([p[0] + 0.052, p[1] - 0.045, 2.4, 2.2], u))
        }
        h += 0.004 * (ridged(u * 30 + 7, 9) - 0.45) + 0.002 * (noise(u * 80, 10) - 0.5)
        return H * h
    }
    function mid(u, H) {
        // A shoulder of hill rises behind the level crossing, so the road
        // beyond the rails has a slope to climb into, and a saddle dips in the
        // ridge to the left of it: the pass the road goes over.
        var shoulder = (u - 0.6) / 0.12
        var pass = (u - 0.25) / 0.04
        var west = (u - 0.14) / 0.07
        return H * (0.035 + 0.15 * noise(u * 2.6 + 5, 4)) + H * 0.025 * noise(u * 8.2, 5)
               + H * 0.07 * Math.exp(-shoulder * shoulder) - H * 0.04 * Math.exp(-pass * pass)
               + H * 0.03 * Math.exp(-west * west)
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

        // The far range: rock, hazier toward its foot.
        paintRidge(ctx, far, H * 0.34, 0, farColor, 0.45)
        var snowY = y0 - H * 0.34 * snowline
        ctx.save()
        ctx.beginPath()
        ctx.moveTo(0, H)
        for (var x = 0; x <= W; x += 4)
            ctx.lineTo(x, y0 - far(x / W, H))
        ctx.lineTo(W, H)
        ctx.closePath()
        ctx.clip()

        // Snow. In a snowy season it caps the range above a ragged snowline,
        // whitest on the summits where it is coldest.
        if (snowCover >= 0.5) {
            ctx.beginPath()
            ctx.moveTo(0, 0)
            ctx.lineTo(W, 0)
            for (var sx = W; sx >= 0; sx -= 4) {
                var u = sx / W
                // Each face holds its snow differently: some low, some high.
                var wobble = H * (0.06 * (noise(u * 7, 7) - 0.5) + 0.03 * (noise(u * 23, 13) - 0.5) + 0.012 * (noise(u * 60, 8) - 0.5))
                ctx.lineTo(sx, snowY + wobble)
            }
            ctx.closePath()
            var cap = ctx.createLinearGradient(0, y0 - H * 0.33, 0, snowY + H * 0.02)
            cap.addColorStop(0, snowColor)
            cap.addColorStop(0.45, mix(snowColor, farColor, 0.2))
            cap.addColorStop(1, mix(snowColor, farColor, 0.6))
            ctx.fillStyle = cap
            ctx.fill()
        }
        // Each summit has a face toward the sun and a face away from it,
        // divided along a spur running down from the top. The shaded face runs
        // from that spur round to the next notch along the skyline.
        if (relief > 0.01) {
            ctx.fillStyle = Qt.rgba(shadeColor.r, shadeColor.g, shadeColor.b, relief)
            for (var pk = 0; pk < peaks.length; pk++) {
                var peak = peaks[pk]
                var side = shadowSide
                var su = peak[0]
                // Along the skyline to the low point before the next peak.
                var eu = su, prevH = far(su, H)
                for (var k = 0; k < 60; k++) {
                    var nu = eu + side * 0.003
                    var nh = far(nu, H)
                    if (nh > prevH + 0.5 && k > 4)
                        break
                    eu = nu
                    prevH = nh
                }
                var top = y0 - far(su, H)
                var footU = su + side * (0.03 + 0.015 * noise(pk * 1.7, 14))
                ctx.beginPath()
                ctx.moveTo(W * su, top)
                for (var wu = su; side > 0 ? wu <= eu : wu >= eu; wu += side * 0.003)
                    ctx.lineTo(W * wu, y0 - far(wu, H))
                // From the notch down a diagonal to the foot of the spur, then
                // back up the spur, a little bowed, to the summit.
                ctx.lineTo(W * footU, y0 - H * 0.1)
                ctx.quadraticCurveTo(W * (su + side * 0.006), top + H * 0.06, W * su, top)
                ctx.closePath()
                ctx.fill()
                // Buttresses break up the sunlit face: narrow ribs whose sides
                // away from the sun fall into shade.
                ctx.fillStyle = Qt.rgba(shadeColor.r, shadeColor.g, shadeColor.b, relief * 0.45)
                for (var rb = 0; rb < 2; rb++) {
                    var ru = su - side * (0.022 + 0.026 * rb + 0.008 * noise(pk * 3.1 + rb, 19))
                    var rtop = y0 - far(ru, H) + H * 0.004
                    var rwide = 0.008 + 0.006 * noise(pk + rb * 5, 20)
                    ctx.beginPath()
                    ctx.moveTo(W * ru, rtop)
                    ctx.lineTo(W * (ru + side * rwide), y0 - far(ru + side * rwide, H) + H * 0.006)
                    // Ribs fan out from the summit as they come down.
                    var spread = 0.035 + 0.02 * rb
                    ctx.lineTo(W * (ru - side * spread + side * 0.01), y0 - H * 0.13)
                    ctx.lineTo(W * (ru - side * spread), y0 - H * 0.13)
                    ctx.closePath()
                    ctx.fill()
                }
                ctx.fillStyle = Qt.rgba(shadeColor.r, shadeColor.g, shadeColor.b, relief)
            }
        }

        // Forest along the foot of the range, its top edge ragged with trees.
        ctx.fillStyle = forestColor
        ctx.beginPath()
        ctx.moveTo(0, H)
        for (var fx = 0; fx <= W; fx += 3) {
            var fu = fx / W
            var line = 0.14 + 0.022 * noise(fu * 9, 16) + 0.01 * noise(fu * 31, 17)
            var tips = 0.006 * (hash(Math.floor(fx / 3), 18) - 0.5)
            ctx.lineTo(fx, y0 - H * (line + tips))
        }
        ctx.lineTo(W, H)
        ctx.closePath()
        ctx.fill()

        // Whatever the season, snow lies in the gullies high on the faces,
        // mostly on the side out of the sun: thin, ragged streaks down the
        // fall line, with a few patches beside them. Short and few in summer,
        // long and many in winter.
        for (var pp = 0; pp < peaks.length; pp++) {
            var pkk = peaks[pp]
            var gullies = 1 + Math.round(2 * snowCover)
            for (var n = 0; n < gullies + 1; n++) {
                var seed = pp * 31 + n
                // The last one, if any, is on the sunlit face.
                var pside = n < gullies ? shadowSide : -shadowSide
                if (n === gullies && hash(seed, 26) > 0.5 + snowCover * 0.5)
                    continue
                var pu = pkk[0] + pside * (0.008 + 0.035 * hash(seed, 22))
                var ridgeAt = far(pu, H) / H
                if (ridgeAt < 0.22)
                    continue
                var onShade = pside === shadowSide
                var white = mix(snowColor, shadeColor, onShade ? 0.3 * relief : 0)
                ctx.fillStyle = Qt.rgba(white.r, white.g, white.b, 0.85)
                var gx = W * pu
                var gy = y0 - H * ridgeAt + H * 0.006
                var len = H * (0.018 + 0.03 * hash(seed, 24)) * (0.5 + snowCover)
                var drift = pside * len * 0.35
                // A ragged streak: its edges wander as it runs down.
                var steps = 6
                ctx.beginPath()
                ctx.moveTo(gx, gy)
                for (var st = 1; st <= steps; st++) {
                    var t = st / steps
                    var wide = H * 0.0035 * (1 - t * 0.7) * (0.6 + 0.8 * hash(seed, 50 + st))
                    ctx.lineTo(gx + drift * t + wide, gy + len * t)
                }
                for (var sb = steps; sb >= 1; sb--) {
                    var tb = sb / steps
                    var wb = H * 0.0035 * (1 - tb * 0.7) * (0.6 + 0.8 * hash(seed, 60 + sb))
                    ctx.lineTo(gx + drift * tb - wb, gy + len * tb)
                }
                ctx.closePath()
                ctx.fill()
                // Patches lying beside the gully.
                for (var pb = 0; pb < 2; pb++) {
                    var bt = 0.2 + 0.5 * hash(seed, 70 + pb)
                    var bx = gx + drift * bt + pside * H * (0.006 + 0.006 * hash(seed, 72 + pb)) * (pb === 0 ? 1 : -1)
                    var by = gy + len * bt
                    var bsize = H * (0.002 + 0.003 * hash(seed, 74 + pb)) * (0.6 + snowCover)
                    ctx.beginPath()
                    ctx.ellipse(bx - bsize, by - bsize * 0.6, bsize * 2, bsize * 1.2)
                    ctx.fill()
                }
            }
        }
        ctx.restore()

        paintRidge(ctx, mid, H * 0.2, 0, midColor, 0.4)
        if (part === "all")
            paintRidge(ctx, near, H * 0.14, 0, nearColor, 0.18)
    }
}
