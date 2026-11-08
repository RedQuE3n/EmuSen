pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import EmuSen.Kit as Kit

// The help bar on the foot: every entry a button that sends its press, its glyph the pad's own in the pad's family, so a pointer has every action the bar names (EmuSen_Qt.md §1.7, EmuSen_Kit.md §7.12).
Rectangle {
    id: root
    required property Kit.FrameController controller
    property alias model: entries.model
    property real leftInset: 0
    property int family: Kit.Families.Generic
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
                leftPadding: 0
                rightPadding: 0
                focusPolicy: Qt.NoFocus
                text: entry.label
                onClicked: root.controller.offer(entry.button)
                contentItem: Row {
                    spacing: Kit.Tokens.base * 2
                    Kit.PadGlyph {
                        objectName: "glyph-" + entry.label
                        anchors.verticalCenter: parent.verticalCenter
                        family: root.family
                        button: entry.button
                        swap: Kit.PadRouter.swapAcceptBack
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
