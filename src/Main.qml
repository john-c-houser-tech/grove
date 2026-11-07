import QtQuick
import QtQuick.Controls
import QtQuick.Controls.Material
import QtQuick.Layouts

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

    function seasonName() {
        switch (weather.season) {
        case "spring": return qsTr("Spring")
        case "summer": return qsTr("Summer")
        case "autumn": return qsTr("Autumn")
        default: return qsTr("Winter")
        }
    }
    function weatherName() {
        switch (weather.kind) {
        case "cloudy": return qsTr("Cloudy")
        case "rain": return qsTr("Rain")
        case "snow": return qsTr("Snow")
        case "fog": return qsTr("Fog")
        default: return qsTr("Clear")
        }
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

        SkyBackdrop { anchors.fill: parent }

        GrowingPlant {
            anchors.fill: parent
            anchors.topMargin: 90
            progress: weather.growth
            seed: backend.seed
            wilted: backend.state === "wilted"
            flowerColor: theme.accent
            nightTint: light.nightTint
            warmTint: light.warmTint
            weatherTint: win.weatherTint
            season: weather.season
        }

        // The ground the plant stands in.
        Rectangle {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            height: parent.height * 0.12 + 6
            color: Qt.tint(Qt.tint(Qt.tint("#2a2018", win.weatherTint), light.nightTint), light.warmTint)
            Rectangle {
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.top: parent.top
                height: 4 + 16 * weather.snow
                color: weather.season === "winter" || weather.snow > 0.05 ? Qt.tint(Qt.tint("#cfdde0", light.nightTint), light.warmTint) : Qt.tint(Qt.tint(Qt.tint("#3a2d20", win.weatherTint), light.nightTint), light.warmTint)
            }
        }

        Precipitation {
            anchors.fill: parent
            rain: weather.rain
            snow: weather.snow
            fog: weather.fog
            ambient: light.ambient
        }

        ColumnLayout {
            anchors.top: parent.top
            anchors.topMargin: 24
            anchors.horizontalCenter: parent.horizontalCenter
            spacing: 4

            Label {
                Layout.alignment: Qt.AlignHCenter
                text: backend.state === "idle" ? qsTr("%n minutes", "", backend.minutes)
                    : win.clock(backend.state === "growing" ? backend.remaining : 0)
                font.pixelSize: 44
                font.weight: Font.Light
                opacity: 0.92
            }
            Label {
                Layout.alignment: Qt.AlignHCenter
                opacity: 0.6
                text: backend.state === "idle" ? qsTr("Press Space to plant a seed")
                    : backend.state === "growing" ? qsTr("Growing. Stay with it.")
                    : backend.state === "done" ? qsTr("Fully grown. Space plants another.")
                    : qsTr("It wilted. Space tries again.")
            }
            Label {
                Layout.alignment: Qt.AlignHCenter
                opacity: 0.4
                font.pixelSize: 12
                text: win.seasonName() + " · " + win.weatherName()
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

        GardenView {
            anchors.fill: parent
            visible: win.gardenVisible
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
