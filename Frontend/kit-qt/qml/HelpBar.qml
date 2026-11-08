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
    // The labels' face, the gap between entries and the gap from a glyph to its label; a theme may set its own.
    property font labelFont: Qt.font({ pixelSize: Kit.Tokens.fontBody })
    property real entrySpacing: Kit.Tokens.paneGap
    property real glyphGap: Kit.Tokens.base * 2
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
        spacing: root.entrySpacing
        Repeater {
            id: entries
            Button {
                id: entry
                required property int button
                required property string label
                required property var model
                readonly property int pair: entry.model.pair ?? -1
                objectName: "help-" + entry.label
                flat: true
                leftPadding: 0
                rightPadding: 0
                implicitWidth: entry.implicitContentWidth
                focusPolicy: Qt.NoFocus
                text: entry.label
                onClicked: root.controller.offer(entry.button)
                contentItem: Row {
                    spacing: root.glyphGap
                    Kit.PadGlyph {
                        objectName: "glyph-" + entry.label
                        anchors.verticalCenter: parent.verticalCenter
                        family: root.family
                        button: entry.button
                        swap: Kit.PadRouter.swapAcceptBack
                    }
                    Kit.PadGlyph {
                        objectName: "glyph2-" + entry.label
                        anchors.verticalCenter: parent.verticalCenter
                        visible: entry.pair >= 0
                        width: entry.pair >= 0 ? implicitWidth : 0
                        family: root.family
                        button: Math.max(0, entry.pair)
                        swap: Kit.PadRouter.swapAcceptBack
                    }
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: entry.label
                        color: Kit.Tokens.text
                        font: root.labelFont
                    }
                }
            }
        }
    }
}
