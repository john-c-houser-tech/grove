import QtQuick
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
        case "fog": return qsTr("Fog")
        default: return qsTr("Clear")
        }
    }
    // Dates and numbers take the language's format, and re-render on a switch.
    function dateText(when) {
        var lang = translator.language
        return when.toLocaleDateString(Qt.locale(), Locale.ShortFormat)
    }
    function number(n) {
        var lang = translator.language
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
                text: qsTr("%1 grown · %2 wilted · %3").arg(root.number(garden.grown)).arg(root.number(garden.wilted))
                          .arg(qsTr("%n minutes", "", garden.focusedMinutes))
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
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: count > 0
            clip: true
            model: garden
            focus: root.visible
            keyNavigationEnabled: true
            ScrollBar.vertical: ScrollBar {}

            // Whole tiles per row, so the grid fills the width evenly.
            readonly property int columns: Math.max(1, Math.floor(width / 150))
            cellWidth: Math.floor(width / columns)
            cellHeight: cellWidth + 34

            delegate: Item {
                width: grid.cellWidth
                height: grid.cellHeight

                Rectangle {
                    anchors.fill: parent
                    anchors.margins: 6
                    radius: theme.radius
                    color: "#161d19"
                    border.color: GridView.isCurrentItem && grid.activeFocus ? theme.accent : "#222c26"
                    border.width: 1

                    Rectangle {
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.bottom: caption.top
                        height: 6
                        color: model.season === "winter" ? "#cfdde0" : "#2a2018"
                    }

                    GrowingPlant {
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.top: parent.top
                        anchors.bottom: caption.top
                        anchors.bottomMargin: 6
                        progress: model.progress
                        seed: model.seed
                        wilted: model.outcome === "wilted"
                        season: model.season
                        flowerColor: theme.accent
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
                            text: root.dateText(model.when)
                        }
                        Label {
                            Layout.fillWidth: true
                            horizontalAlignment: Text.AlignHCenter
                            font.pixelSize: 11
                            opacity: 0.5
                            elide: Text.ElideRight
                            text: model.outcome === "wilted" ? qsTr("Wilted") : root.weatherLabel(model.weather)
                        }
                    }
                }
            }
        }
    }
}
