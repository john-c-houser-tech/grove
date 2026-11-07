import QtQuick

// The sky, stars, sun and moon, all from the garden's light. The ground
// covers the bottom of this item, so a sinking sun slips behind it.
Item {
    id: root

    // Where the horizon sits, as a fraction of the height.
    property real horizon: 0.88

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

    // Sun: a glow around a bright disc, warmer near the horizon.
    Item {
        visible: light.sunUp
        opacity: Math.max(0.05, 1 - 0.55 * weather.cloud - 0.5 * weather.fog)
        readonly property real size: Math.min(root.width, root.height) * 0.07
        x: root.width * (0.08 + 0.84 * light.sunX) - width / 2
        y: root.height * (root.horizon - 0.66 * light.elevation) - height / 2
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

    // Moon: a crescent cut out of a disc, so it needs no matching sky colour.
    Item {
        visible: !light.sunUp
        readonly property real size: Math.min(root.width, root.height) * 0.055
        x: root.width * (0.08 + 0.84 * light.moonX) - width / 2
        y: root.height * (root.horizon - 0.62 * light.moonElevation) - height / 2
        width: size
        height: size
        opacity: Math.min(1, 0.25 + light.stars) * Math.max(0.05, 1 - 0.7 * weather.cloud - 0.5 * weather.fog)
        Rectangle {
            anchors.centerIn: parent
            width: parent.width * 2
            height: width
            radius: width / 2
            color: "#dfe8ff"
            opacity: 0.05
        }
        Canvas {
            anchors.fill: parent
            onWidthChanged: requestPaint()
            onPaint: {
                var ctx = getContext("2d")
                ctx.reset()
                var r = width / 2
                ctx.fillStyle = "#eef2ff"
                ctx.beginPath()
                ctx.arc(r, r, r, 0, Math.PI * 2)
                ctx.fill()
                ctx.globalCompositeOperation = "destination-out"
                ctx.beginPath()
                ctx.arc(r * 1.55, r * 0.85, r * 0.92, 0, Math.PI * 2)
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
        readonly property real snowline: weather.season === "winter" ? 0.42 : weather.season === "spring" ? 0.58
                                       : weather.season === "autumn" ? 0.66 : 0.8
        readonly property color farColor: lit("#697d9a", 0.38)
        readonly property color midColor: lit("#44605a", 0.24)
        readonly property color nearColor: lit("#2f4a3a", 0.08)
        // The valley floor the railway runs along, between the ridges and the
        // near hills.
        readonly property color floorColor: lit("#56704c", 0.16)
        readonly property color snowColor: Qt.tint(Qt.tint(Qt.tint("#f4f7fb", weatherTint), light.nightTint), light.warmTint)
    }

    Hills {
        part: "back"
        anchors.fill: parent
        horizon: root.horizon
        snowline: look.snowline
        hazeColor: light.skyBottom
        farColor: look.farColor
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
        horizon: root.horizon
        carColor: Qt.darker(look.midColor, 2.2)
        floorColor: look.floorColor
        ballastColor: Qt.darker(look.floorColor, 1.35)
        roadColor: Qt.tint(look.floorColor, Qt.rgba(0.62, 0.55, 0.42, 0.45))
        railColor: Qt.tint(look.floorColor, Qt.rgba(light.skyBottom.r, light.skyBottom.g, light.skyBottom.b, 0.55))
    }
    Connections {
        target: sounds
        function onTrainPassing(milliseconds) { train.go(milliseconds) }
    }

    Hills {
        part: "front"
        anchors.fill: parent
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
    }
}
