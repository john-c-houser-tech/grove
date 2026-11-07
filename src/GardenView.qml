pragma ComponentBehavior: Bound
import QtQuick
import Grove
import QtQuick.Controls
import QtQuick.Controls.Material
import QtQuick.Layouts

// Every plant from past sessions, newest first, each as it ended: in full
// bloom or wilted where it stopped.
Rectangle {
    id: root

    signal closed()

    color: "#0e1210"

    function weatherLabel(kind) {
        switch (kind) {
        case "cloudy": return qsTr("Cloudy")
        case "rain": return qsTr("Rain")
        case "snow": return qsTr("Snow")
        case "fog": return qsTr("Fog")
        default: return qsTr("Clear")
        }
    }
    // Dates and numbers take the language's format, and re-render on a switch.
    function dateText(when) {
        var lang = Translator.language
        return when.toLocaleDateString(Qt.locale(), Locale.ShortFormat)
    }
    function number(n) {
        var lang = Translator.language
        return n.toLocaleString(Qt.locale(), 'f', 0)
    }

    MouseArea { anchors.fill: parent }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 24
        spacing: 16

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 4
            Label { text: qsTr("Garden"); font.pixelSize: 28; font.weight: Font.Light }
            Label {
                opacity: 0.6
                text: qsTr("%1 grown · %2 wilted · %3").arg(root.number(Garden.grown)).arg(root.number(Garden.wilted))
                          .arg(qsTr("%n minutes", "", Garden.focusedMinutes))
            }
        }

        Label {
            visible: grid.count === 0
            Layout.fillWidth: true
            Layout.topMargin: 40
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            opacity: 0.55
            text: qsTr("Nothing here yet. Finish a session and the plant stays.")
        }

        GridView {
            id: grid
            objectName: "gardenGrid"
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: count > 0
            clip: true
            model: Garden
            // It is made each time the garden opens, and takes the keys then.
            Component.onCompleted: forceActiveFocus()
            keyNavigationEnabled: true
            ScrollBar.vertical: ScrollBar {}

            // Whole tiles per row, so the grid fills the width evenly.
            readonly property int columns: Math.max(1, Math.floor(width / 150))
            cellWidth: Math.floor(width / columns)
            cellHeight: cellWidth + 34

            delegate: Item {
                id: tile
                // The garden's roles, one per plant.
                required property int seed
                required property string outcome
                required property string season
                required property string weather
                required property real progress
                required property date when
                width: grid.cellWidth
                height: grid.cellHeight

                Rectangle {
                    anchors.fill: parent
                    anchors.margins: 6
                    radius: Theme.radius
                    color: "#161d19"
                    // The view sets isCurrentItem on the delegate itself, not on what is inside it.
                    border.color: tile.GridView.isCurrentItem && grid.activeFocus ? Theme.accent : "#222c26"
                    border.width: 1

                    Rectangle {
                        id: soil
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.bottom: caption.top
                        height: 8
                        color: tile.season === "winter" ? "#cfdde0" : "#2a2018"
                    }

                    GrowingPlant {
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.top: parent.top
                        anchors.bottom: caption.top
                        // The trunk goes into the middle of the strip of soil.
                        base: (height - soil.height / 2) / height
                        progress: tile.progress
                        seed: tile.seed
                        wilted: tile.outcome === "wilted"
                        season: tile.season
                        flowerColor: Theme.accent
                    }

                    ColumnLayout {
                        id: caption
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.bottom: parent.bottom
                        anchors.margins: 6
                        spacing: 0
                        Label {
                            Layout.fillWidth: true
                            horizontalAlignment: Text.AlignHCenter
                            font.pixelSize: 12
                            elide: Text.ElideRight
                            text: root.dateText(tile.when)
                        }
                        Label {
                            Layout.fillWidth: true
                            horizontalAlignment: Text.AlignHCenter
                            font.pixelSize: 11
                            opacity: 0.5
                            elide: Text.ElideRight
                            text: tile.outcome === "wilted" ? qsTr("Wilted") : root.weatherLabel(tile.weather)
                        }
                    }
                }
            }
        }
    }
}
