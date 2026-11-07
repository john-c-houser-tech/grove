import QtQuick
import QtQuick.Controls
import QtQuick.Controls.Material
import QtQuick.Layouts
import Grove

ApplicationWindow {
    id: win
    width: 760
    height: 760
    minimumWidth: 420
    minimumHeight: 480
    visible: true
    title: "Grove"
    color: "#0e1210"

    Material.theme: Material.Dark
    Material.accent: theme.accent
    Material.roundedScale: theme.radius

    property bool helpVisible: false
    property bool gardenVisible: false
    property bool quitConfirmed: false
    readonly property bool growing: backend.state === "growing"
    // Overcast, rain and fog dull the light on the plant and the ground.
    readonly property color weatherTint: Qt.rgba(0.42, 0.47, 0.52, Math.min(0.5, 0.14 * weather.cloud + 0.14 * weather.rain + 0.14 * weather.snow + 0.28 * weather.fog))

    // Numbers take the language's digits and separators, and re-render when
    // the language changes.
    function number(n) {
        var lang = translator.language
        return n.toLocaleString(Qt.locale(), 'f', 0)
    }
    function clock(seconds) {
        var m = Math.floor(seconds / 60), s = seconds % 60
        return number(m) + ":" + (s < 10 ? number(0) : "") + number(s)
    }


    function ask(action) {
        confirm.action = action
        confirm.open()
    }
    function quit() {
        if (growing) ask("quit")
        else Qt.quit()
    }

    onClosing: function (close) {
        if (growing && !quitConfirmed) {
            close.accepted = false
            ask("quit")
        }
    }

    Shortcut { sequence: "?"; context: Qt.ApplicationShortcut; enabled: !confirm.opened && !win.gardenVisible; onActivated: win.helpVisible = !win.helpVisible }
    Shortcut { sequence: "G"; context: Qt.ApplicationShortcut; enabled: !confirm.opened && !win.helpVisible; onActivated: win.gardenVisible = !win.gardenVisible }
    Shortcut {
        sequence: "Escape"
        context: Qt.ApplicationShortcut
        enabled: (win.helpVisible || win.gardenVisible) && !confirm.opened
        onActivated: { win.helpVisible = false; win.gardenVisible = false }
    }
    Shortcut { sequence: "M"; context: Qt.ApplicationShortcut; enabled: !confirm.opened; onActivated: sounds.muted = !sounds.muted }
    Shortcut { sequence: "Q"; context: Qt.ApplicationShortcut; enabled: !confirm.opened; onActivated: win.quit() }
    Shortcut {
        sequence: "Space"
        context: Qt.ApplicationShortcut
        enabled: !confirm.opened && !win.helpVisible && !win.gardenVisible
        onActivated: {
            // Giving up wilts the plant, so it is the one action that asks.
            if (win.growing) win.ask("giveup")
            else backend.start()
        }
    }

    // Right-to-left languages mirror the window; the Translator sets the
    // application's layout direction.
    Item {
        anchors.fill: parent
        LayoutMirroring.enabled: Qt.application.layoutDirection === Qt.RightToLeft
        LayoutMirroring.childrenInherit: true

        SkyBackdrop {
            id: sky
            anchors.fill: parent
            groundY: field.farEdgeY
        }

        // The ground the plant stands in. It is drawn first, so the plant
        // grows out of it rather than from behind it.
        Rectangle {
            id: ground
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            // From where the field ends, out at 40 m, to the bottom.
            height: parent.height - field.farEdgeY
            color: Qt.tint(Qt.tint(Qt.tint("#2a2018", win.weatherTint), light.nightTint), light.warmTint)
        }

        // The ploughed field the plant grows in.
        Field {
            id: field
            anchors.fill: parent
            groundTop: farEdgeY
            part: "far"
            soilColor: ground.color
            hazeColor: light.skyBottom
            snow: weather.snow
            frost: weather.season === "winter"
        }

        GrowingPlant {
            id: plant
            anchors.fill: parent
            anchors.topMargin: 90
            // The trunk stands in the furrow the field puts it in.
            base: (field.plantBaseY - y) / height
            rootX: field.plantX
            progress: weather.growth
            seed: backend.seed
            wilted: backend.state === "wilted"
            flowerColor: theme.accent
            nightTint: light.nightTint
            warmTint: light.warmTint
            weatherTint: win.weatherTint
            season: weather.season
        }

        // The rows nearer than the plant, over its foot.
        Field {
            anchors.fill: parent
            part: "near"
            groundTop: farEdgeY
            soilColor: ground.color
            hazeColor: light.skyBottom
            snow: weather.snow
            frost: weather.season === "winter"
        }

        Precipitation {
            anchors.fill: parent
            rain: weather.rain
            snow: weather.snow
            fog: weather.fog
            sky: light.skyTop
            ambient: light.ambient
        }

        // Small and quiet, so the scene comes first. It sits at the top right,
        // in open sky, and crosses to the left while the sun or moon is there;
        // they move slowly, so it crosses at most once each way.
        ColumnLayout {
            id: heading
            anchors.top: parent.top
            anchors.topMargin: 18
            spacing: 2

            readonly property real margin: 24
            // How close the sun's glow may come before the text gives way.
            readonly property real clearance: 12
            readonly property bool sunOnRight: {
                var body = sky.bodyArea, left = parent.width - margin - width - clearance
                return body.x + body.width > left && body.x < parent.width
                    && body.y < y + height + clearance && body.y + body.height > y - clearance
            }
            // 1 on the right, 0 on the left, eased between so the move is
            // gentle while a resize still follows at once.
            property real side: sunOnRight ? 0 : 1
            Behavior on side { NumberAnimation { duration: 1500; easing.type: Easing.InOutQuad } }
            x: margin + side * (parent.width - 2 * margin - width)
            readonly property int lineAlignment: side > 0.5 ? Qt.AlignRight : Qt.AlignLeft
            // The side follows the sky, not the reading direction, so the
            // lines keep to it in right-to-left languages too.
            LayoutMirroring.enabled: false
            LayoutMirroring.childrenInherit: true

            Label {
                Layout.alignment: heading.lineAlignment
                text: backend.state === "idle" ? qsTr("%n minutes", "", backend.minutes)
                    : win.clock(backend.state === "growing" ? backend.remaining : 0)
                font.pixelSize: 24
                font.weight: Font.Light
                opacity: 0.75
            }
            Label {
                Layout.alignment: heading.lineAlignment
                opacity: 0.5
                font.pixelSize: 12
                text: backend.state === "idle" ? qsTr("Press Space to plant a seed")
                    : backend.state === "growing" ? qsTr("Growing. Stay with it.")
                    : backend.state === "done" ? qsTr("Fully grown. Space plants another.")
                    : qsTr("It wilted. Space tries again.")
            }
        }

        // Sound is the one thing that can be off or broken without showing, so say so.
        Label {
            anchors.left: parent.left
            anchors.bottom: parent.bottom
            anchors.margins: 12
            visible: sounds.problem !== "" || sounds.muted
            text: sounds.problem !== "" ? sounds.problem : qsTr("Sounds are off. M turns them on.")
            opacity: 0.6
            font.pixelSize: 12
        }

        Label {
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.margins: 12
            text: qsTr("G garden · ? for keys")
            opacity: 0.35
            font.pixelSize: 12
        }

        // A garden that cannot be read or saved says so rather than hiding it.
        Label {
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.bottom: parent.bottom
            anchors.margins: 12
            width: parent.width - 220
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            visible: garden.problem !== "" || notifier.problem !== ""
            text: garden.problem !== "" ? garden.problem : notifier.problem
            color: "#ffb4a8"
            font.pixelSize: 12
        }

        // Made only while it is open, so the tiles and their pictures are not
        // kept the rest of the time.
        Loader {
            anchors.fill: parent
            active: win.gardenVisible
            sourceComponent: GardenView {}
        }

        // Shortcut overlay.
        Rectangle {
            anchors.fill: parent
            visible: win.helpVisible
            color: "#cc0e1210"
            MouseArea { anchors.fill: parent; onClicked: win.helpVisible = false }
            Rectangle {
                anchors.centerIn: parent
                width: Math.min(parent.width - 48, 360)
                height: keys.implicitHeight + 48
                radius: theme.radius
                color: "#1a211d"
                border.color: theme.accent
                border.width: 1
                ColumnLayout {
                    id: keys
                    anchors.fill: parent
                    anchors.margins: 24
                    spacing: 10
                    Label { text: qsTr("Keys"); font.pixelSize: 20 }
                    Repeater {
                        model: [
                            { k: "Space", d: qsTr("Plant a seed, or give up") },
                            { k: "?", d: qsTr("Show or hide this help") },
                            { k: "G", d: qsTr("Show or hide the garden") },
                            { k: "M", d: qsTr("Mute or unmute distant sounds") },
                            { k: "Q", d: qsTr("Quit") }
                        ]
                        delegate: RowLayout {
                            Layout.fillWidth: true
                            Label { text: modelData.k; color: theme.accent; Layout.preferredWidth: 70 }
                            Label { text: modelData.d; Layout.fillWidth: true; wrapMode: Text.WordWrap }
                        }
                    }
                }
            }
        }
    }

    Dialog {
        id: confirm
        property string action: "giveup"
        anchors.centerIn: parent
        modal: true
        focus: true
        title: action === "quit" ? qsTr("Quit and wilt this plant?") : qsTr("Give up this session?")
        standardButtons: Dialog.Yes | Dialog.No
        Label {
            text: qsTr("The plant will wilt, and the garden keeps it.")
            wrapMode: Text.WordWrap
        }
        // A Popup is not an Item, so Enter is a shortcut rather than a key handler.
        Shortcut { sequence: "Return"; enabled: confirm.opened; onActivated: confirm.accept() }
        onAccepted: {
            if (action === "quit") {
                // Leaving mid-session wilts the plant, and the garden keeps it.
                backend.giveUp()
                win.quitConfirmed = true
                Qt.quit()
            } else {
                backend.giveUp()
            }
        }
    }
}
