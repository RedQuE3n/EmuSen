pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import EmuSen.Kit as Kit

// The help bar on the foot: every entry a button that sends its press, so a pointer has every action the bar names (EmuSen_Qt.md §1.7).
Rectangle {
    id: root
    required property Kit.FrameController controller
    property alias model: entries.model
    property real leftInset: 0
    implicitHeight: Kit.Tokens.helpBar
    color: Kit.Tokens.surface
    Rectangle {
        anchors.top: parent.top
        width: parent.width
        height: Kit.Tokens.hoverBorder
        color: Kit.Tokens.rule
    }
    Row {
        x: root.leftInset
        anchors.verticalCenter: parent.verticalCenter
        spacing: Kit.Tokens.paneGap
        Repeater {
            id: entries
            Button {
                id: entry
                required property int button
                required property string label
                objectName: "help-" + entry.label
                flat: true
                focusPolicy: Qt.NoFocus
                text: entry.label
                onClicked: root.controller.offer(entry.button)
                contentItem: Row {
                    spacing: Kit.Tokens.base * 2
                    Rectangle {
                        anchors.verticalCenter: parent.verticalCenter
                        width: Kit.Tokens.fontGlyph * 1.4
                        height: width
                        radius: width / 2
                        color: Kit.Tokens.inputSurface
                        border.width: Kit.Tokens.hoverBorder * 2
                        border.color: entry.activeFocus ? Kit.Tokens.accent : Kit.Tokens.muted
                    }
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: entry.label
                        color: Kit.Tokens.text
                        font.pixelSize: Kit.Tokens.fontBody
                    }
                }
            }
        }
    }
}
