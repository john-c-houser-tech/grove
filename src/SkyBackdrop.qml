import QtQuick
import Grove

// The sky, stars, sun and moon, all from the garden's light. The ground
// covers the bottom of this item, so a sinking sun slips behind it.
Item {
    id: root

    // Where the horizon sits, as a fraction of the height.
    // At standing eye level: everything below is ground, everything above sky.
    property real horizon: 0.6
    // Where the opaque ground in front begins, in pixels from the top. The land
    // here is hidden below it, so it is not drawn there.
    property real groundY: -1

    Rectangle {
        anchors.fill: parent
        gradient: Gradient {
            GradientStop { position: 0.0; color: light.skyTop }
            GradientStop { position: root.horizon; color: light.skyBottom }
            GradientStop { position: 1.0; color: light.skyBottom }
        }
    }

    // Cloud, rain and fog wash the colour out of the sky.
    Rectangle {
        anchors.fill: parent
        color: Qt.rgba(0.28 + 0.3 * light.ambient, 0.31 + 0.3 * light.ambient, 0.36 + 0.28 * light.ambient, 1)
        opacity: Math.min(0.7, 0.42 * weather.cloud + 0.3 * weather.rain + 0.3 * weather.snow + 0.35 * weather.fog)
    }

    // Fixed scatter of stars that fade in as the sky darkens.
    Repeater {
        model: 70
        delegate: Rectangle {
            // A deterministic hash keeps every star in the same place.
            readonly property real a: (Math.sin(index * 12.9898) * 43758.5453) % 1
            readonly property real b: (Math.sin(index * 78.233) * 12345.6789) % 1
            x: Math.abs(a) * root.width
            y: Math.abs(b) * root.height * (root.horizon - 0.1)
            width: index % 7 === 0 ? 3 : 2
            height: width
            radius: width / 2
            color: "white"
            opacity: light.stars * (1 - weather.cloud * 0.9) * (1 - weather.fog * 0.8) * (0.35 + 0.5 * Math.abs((index * 37) % 10) / 10)
        }
    }

    // Where the sun and its glow are, or the moon at night, so that text can
    // keep clear of it.
    readonly property rect bodyArea: light.sunUp
        ? Qt.rect(sun.x - sun.width * 0.8, sun.y - sun.height * 0.8, sun.width * 2.6, sun.height * 2.6)
        : Qt.rect(moon.x, moon.y, moon.width, moon.height)

    // Sun: a glow around a bright disc, warmer near the horizon.
    Item {
        id: sun
        visible: light.sunUp
        opacity: Math.max(0.05, 1 - 0.55 * weather.cloud - 0.5 * weather.fog)
        readonly property real size: Math.min(root.width, root.height) * 0.07
        x: root.width * (0.08 + 0.84 * light.sunX) - width / 2
        y: root.height * (root.horizon - 0.5 * light.elevation) - height / 2
        width: size
        height: size
        Rectangle {
            anchors.centerIn: parent
            width: parent.width * 2.6
            height: width
            radius: width / 2
            color: "#fff1c9"
            opacity: 0.12 + 0.1 * light.warmth
        }
        Rectangle {
            anchors.centerIn: parent
            width: parent.width * 1.6
            height: width
            radius: width / 2
            color: "#fff1c9"
            opacity: 0.2
        }
        Rectangle {
            anchors.fill: parent
            radius: width / 2
            color: Qt.tint("#fff6dc", Qt.rgba(light.warmTint.r, light.warmTint.g, light.warmTint.b, light.warmth * 0.5))
        }
    }

    // Moon: the whole disk, with the sunlit part a crescent.
    Item {
        id: moon
        visible: !light.sunUp
        readonly property real size: Math.min(root.width, root.height) * 0.055
        x: root.width * (0.08 + 0.84 * light.moonX) - width / 2
        y: root.height * (root.horizon - 0.48 * light.moonElevation) - height / 2
        width: size
        height: size
        opacity: Math.min(1, 0.25 + light.stars) * Math.max(0.05, 1 - 0.7 * weather.cloud - 0.5 * weather.fog)
        // The whole disk is there, faintly lit by earthshine; only the
        // crescent catches the sun.
        Canvas {
            id: moonDisk
            anchors.fill: parent
            readonly property color shadow: Qt.tint(light.skyTop, Qt.rgba(0.8, 0.85, 1, 0.13))
            onShadowChanged: requestPaint()
            onWidthChanged: requestPaint()
            onPaint: {
                var ctx = getContext("2d")
                ctx.reset()
                var r = width / 2
                // The lit crescent: a bright disk with an offset disk cut out of it.
                ctx.fillStyle = "#eef2ff"
                ctx.beginPath()
                ctx.arc(r, r, r, 0, Math.PI * 2)
                ctx.fill()
                ctx.globalCompositeOperation = "destination-out"
                ctx.beginPath()
                ctx.arc(r * 1.5, r, r, 0, Math.PI * 2)
                ctx.fill()
                // The rest of the disk, behind the crescent.
                ctx.globalCompositeOperation = "destination-over"
                ctx.fillStyle = shadow
                ctx.beginPath()
                ctx.arc(r, r, r, 0, Math.PI * 2)
                ctx.fill()
            }
        }
    }

    // How the land is lit, shared by both layers of it: dimmed at night, warmed
    // at dusk, greyed by weather, and hazier with distance.
    QtObject {
        id: look
        readonly property color weatherTint: Qt.rgba(0.42, 0.47, 0.52, Math.min(0.5, 0.2 * weather.cloud + 0.2 * weather.rain + 0.15 * weather.snow))
        function lit(c, haze) {
            return Qt.tint(Qt.tint(Qt.tint(Qt.tint(c, Qt.rgba(light.skyBottom.r, light.skyBottom.g, light.skyBottom.b, haze)),
                                          weatherTint), light.nightTint), light.warmTint)
        }
        // Winter snow reaches low; by summer only the highest peaks keep any.
        readonly property real snowline: weather.season === "winter" ? 0.45 : 0.6
        // Bare granite with a few high patches most of the year; a white range
        // in winter, and plenty left in spring.
        readonly property real snowCover: weather.season === "winter" ? 1 : weather.season === "spring" ? 0.65
                                        : weather.season === "autumn" ? 0.3 : 0.15
        readonly property color rockColor: Qt.tint(lit("#8c97a3", 0.3), Qt.rgba(0.02, 0.03, 0.06, 0.5 * dark))
        readonly property color rockShade: Qt.tint(lit("#4a5668", 0.22), Qt.rgba(0.02, 0.03, 0.06, 0.5 * dark))
        readonly property color forestColor: Qt.tint(lit("#2c3d33", 0.3), Qt.rgba(0.02, 0.03, 0.06, 0.5 * dark))
        // After dark the land goes to silhouette against the sky, so the far
        // peaks keep an edge even when cloud washes the sky toward their colour.
        readonly property real dark: 1 - light.ambient
        readonly property color farColor: Qt.tint(lit("#697d9a", 0.38), Qt.rgba(0.02, 0.03, 0.06, 0.5 * dark))
        readonly property color midColor: lit("#44605a", 0.24)
        readonly property color nearColor: lit("#2f4a3a", 0.08)
        // The valley floor the railway runs along, between the ridges and the
        // near hills.
        readonly property color floorColor: lit("#56704c", 0.16)
        // Snow only shines where there is light on it. At night it sinks toward
        // the colour of the mountain, so a cap never glows on its own.
        readonly property color snowColor: Qt.tint(Qt.tint(Qt.tint(Qt.tint("#f4f7fb", weatherTint), light.nightTint), light.warmTint),
                                                   Qt.rgba(farColor.r, farColor.g, farColor.b, 0.6 * dark))
    }

    Hills {
        part: "back"
        anchors.fill: parent
        coveredFrom: root.groundY
        horizon: root.horizon
        snowline: look.snowline
        snowCover: look.snowCover
        hazeColor: light.skyBottom
        farColor: look.rockColor
        shadeColor: look.rockShade
        forestColor: look.forestColor
        // The faces away from the sun are in shadow, strongly in clear sun,
        // hardly at all under cloud or at night.
        shadowSide: light.sunX < 0.5 ? 1 : -1
        relief: light.sunUp ? 0.7 * Math.min(1, light.ambient) * (1 - 0.85 * weather.cloud) * (1 - weather.fog) : 0.08
        midColor: look.midColor
        nearColor: look.nearColor
        snowColor: look.snowColor
    }

    // The railway across the valley, and the night train on it, between the
    // ridges and the near hills.
    Train {
        id: train
        objectName: "train"
        anchors.fill: parent
        coveredFrom: root.groundY
        horizon: root.horizon
        carColor: Qt.darker(look.midColor, 2.2)
        floorColor: look.floorColor
        ballastColor: Qt.darker(look.floorColor, 1.35)
        // Asphalt, lit like everything else.
        roadColor: look.lit("#4c4f54", 0.1)
        // Bare earth in the cut and the tipped fill below the road, lit like the land.
        cutColor: look.lit("#9a8160", 0.22)
        fillColor: look.lit("#6f5f47", 0.22)
        hillColor: look.midColor
        hazeColor: light.skyBottom
        railColor: Qt.tint(look.floorColor, Qt.rgba(light.skyBottom.r, light.skyBottom.g, light.skyBottom.b, 0.55))
    }
    Connections {
        target: sounds
        function onTrainPassing(milliseconds) { train.go(milliseconds) }
        function onFadingOut() { train.leave(); plane.leave() }
    }

    Hills {
        part: "front"
        anchors.fill: parent
        coveredFrom: root.groundY
        horizon: root.horizon
        hazeColor: light.skyBottom
        farColor: look.farColor
        midColor: look.midColor
        nearColor: look.nearColor
        snowColor: look.snowColor
    }

    Plane {
        id: plane
        anchors.fill: parent
        bodyColor: Qt.tint("#2a3441", Qt.rgba(light.skyBottom.r, light.skyBottom.g, light.skyBottom.b, 0.25))
    }
    Connections {
        target: sounds
        function onPlaneFlyover(direction, milliseconds, altitude) { plane.fly(direction, milliseconds, altitude) }
    }

    Clouds {
        anchors.fill: parent
        opacity: 1 - 0.85 * weather.fog
        cover: weather.cloud
        rain: weather.rain
        ambient: light.ambient
        sky: light.skyTop
    }
}
