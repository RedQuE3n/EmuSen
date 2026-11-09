import QtQuick
import EmuSen.Theme

// A console's engine line: the mark, the engine's name and the EMUSEN or LIBRETRO tag, in the console's hue on the night in both variants, a Scene since the hue is the console's (EmuSen_Interface.md §24.6; EmuSen_Qt.md §3.4.2).
Item {
    id: root
    property string name
    property string mark
    property bool libretro
    property real hue
    property string family
    // The name's widest before it elides.
    property real nameWidth: 150
    readonly property color ink: root.libretro ? "#C9C9D0" : Qt.hsla(root.hue / 360, 0.85, 0.8, 1)
    implicitWidth: line.width
    implicitHeight: 20
    Scene {
        anchors.fill: parent
        Row {
            id: line
            // Centred on the line's 20 units, the tag's box reaching 2 above and below (EmuSen_Qt.md §3.4.2).
            anchors.verticalCenter: parent.verticalCenter
            spacing: 6
            EngineMark {
                objectName: "engineMark"
                anchors.verticalCenter: parent.verticalCenter
                width: 15
                height: 15
                mark: root.mark
                libretro: root.libretro
                ink: root.ink
            }
            Text {
                objectName: "engineName"
                anchors.verticalCenter: parent.verticalCenter
                width: Math.min(implicitWidth, root.nameWidth)
                text: root.name
                elide: Text.ElideRight
                color: root.ink
                font.family: root.family
                font.weight: Font.Medium
                font.pixelSize: 19
                font.letterSpacing: 1.5
                font.capitalization: Font.AllUppercase
            }
            Rectangle {
                objectName: "engineTag"
                anchors.verticalCenter: parent.verticalCenter
                width: tagWord.implicitWidth + 12
                height: 24
                radius: 4
                color: root.libretro ? "transparent" : Qt.hsla(root.hue / 360, 0.6, 0.5, 0.14)
                border.color: root.libretro ? "#6E6E78" : Qt.hsla(root.hue / 360, 0.45, 0.52, 1)
                Text {
                    id: tagWord
                    anchors.centerIn: parent
                    text: root.libretro ? Words.libretroTag : Words.emusenTag
                    color: root.libretro ? "#A9A9B2" : Qt.hsla(root.hue / 360, 0.7, 0.82, 1)
                    font.family: root.family
                    font.weight: Font.Medium
                    font.pixelSize: 19
                    font.letterSpacing: 0.8
                }
            }
        }
    }
}
