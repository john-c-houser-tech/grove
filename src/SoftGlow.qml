import QtQuick

// A soft pool of light with no edge, for headlights and signal lamps. Make it
// wider than tall for a beam.
Canvas {
    id: glow

    property color color: "white"
    property real strength: 1

    onColorChanged: requestPaint()
    onStrengthChanged: requestPaint()
    onWidthChanged: requestPaint()
    onHeightChanged: requestPaint()

    onPaint: {
        var ctx = getContext("2d")
        ctx.reset()
        if (width <= 0 || height <= 0)
            return
        var c = "rgba(" + Math.round(color.r * 255) + "," + Math.round(color.g * 255) + "," + Math.round(color.b * 255) + ","
        var rx = width / 2, ry = height / 2
        ctx.translate(rx, ry)
        ctx.scale(1, ry / rx)
        var g = ctx.createRadialGradient(0, 0, 0, 0, 0, rx)
        g.addColorStop(0, c + strength + ")")
        g.addColorStop(0.25, c + strength * 0.55 + ")")
        g.addColorStop(0.6, c + strength * 0.15 + ")")
        g.addColorStop(1, c + "0)")
        ctx.fillStyle = g
        ctx.beginPath()
        ctx.arc(0, 0, rx, 0, Math.PI * 2)
        ctx.fill()
    }
}
