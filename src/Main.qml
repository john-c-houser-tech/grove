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
    property bool quitConfirmed: false
    readonly property bool growing: backend.state === "growing"

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

    Shortcut { sequence: "?"; context: Qt.ApplicationShortcut; enabled: !confirm.opened; onActivated: win.helpVisible = !win.helpVisible }
    Shortcut { sequence: "Escape"; context: Qt.ApplicationShortcut; enabled: win.helpVisible && !confirm.opened; onActivated: win.helpVisible = false }
    Shortcut { sequence: "Q"; context: Qt.ApplicationShortcut; enabled: !confirm.opened; onActivated: win.quit() }
    Shortcut {
        sequence: "Space"
        context: Qt.ApplicationShortcut
        enabled: !confirm.opened && !win.helpVisible
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

        Rectangle {
            id: sky
            anchors.fill: parent
            gradient: Gradient {
                GradientStop { position: 0.0; color: "#121a1f" }
                GradientStop { position: 1.0; color: "#1b2a22" }
            }
        }

        GrowingPlant {
            anchors.fill: parent
            anchors.topMargin: 90
            progress: backend.progress
            seed: backend.seed
            wilted: backend.state === "wilted"
            flowerColor: theme.accent
        }

        // The ground the plant stands in.
        Rectangle {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            height: parent.height * 0.12 + 6
            color: "#2a2018"
            Rectangle {
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.top: parent.top
                height: 4
                color: "#3a2d20"
            }
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
        }

        Label {
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.margins: 12
            text: qsTr("? for keys")
            opacity: 0.35
            font.pixelSize: 12
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
        title: action === "quit" ? qsTr("Quit and lose this plant?") : qsTr("Give up this session?")
        standardButtons: Dialog.Yes | Dialog.No
        Label {
            text: qsTr("The plant will wilt.")
            wrapMode: Text.WordWrap
        }
        // A Popup is not an Item, so Enter is a shortcut rather than a key handler.
        Shortcut { sequence: "Return"; enabled: confirm.opened; onActivated: confirm.accept() }
        onAccepted: {
            if (action === "quit") {
                win.quitConfirmed = true
                Qt.quit()
            } else {
                backend.giveUp()
            }
        }
    }
}
