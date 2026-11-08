pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import EmuSen.Kit

// The help bar on the foot: every entry a button that sends its press, so a pointer has every action the bar names (EmuSen_Qt.md §1.7).
Rectangle {
    id: root
    required property FrameController controller
    property alias model: entries.model
    property real leftInset: 0
    implicitHeight: Tokens.helpBar
    color: Tokens.surface
    Rectangle {
        anchors.top: parent.top
        width: parent.width
        height: Tokens.hoverBorder
        color: Tokens.rule
    }
    Row {
        x: root.leftInset
        anchors.verticalCenter: parent.verticalCenter
        spacing: Tokens.paneGap
        Repeater {
            id: entries
            Button {
                id: entry
                required property int button
                required property string label
                objectName: "help-" + entry.label
                flat: true
                text: entry.label
                onClicked: root.controller.offer(entry.button)
                contentItem: Row {
                    spacing: Tokens.base * 2
                    Rectangle {
                        anchors.verticalCenter: parent.verticalCenter
                        width: Tokens.fontGlyph * 1.4
                        height: width
                        radius: width / 2
                        color: Tokens.inputSurface
                        border.width: Tokens.hoverBorder * 2
                        border.color: entry.activeFocus ? Tokens.accent : Tokens.muted
                    }
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: entry.label
                        color: Tokens.text
                        font.pixelSize: Tokens.fontBody
                    }
                }
            }
        }
    }
}
