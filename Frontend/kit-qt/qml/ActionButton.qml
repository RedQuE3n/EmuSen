pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

// An action on the frame: the pad's glyph where it has one, its word and its value, filled in the accent where it is the screen's main action, ringed when focused; a click or A is the action (EmuSen_Kit.md §7.40).
Kit.Walkable {
    id: root
    property string label
    property string value
    // The pad button whose glyph leads the words; below zero, none.
    property int button: -1
    property bool filled: false
    property int family: Kit.Families.Generic
    property font labelFont: Qt.font({ pixelSize: Kit.Tokens.fontRow })
    readonly property color ink: root.filled ? Kit.Tokens.overAccent : (root.enabled ? Kit.Tokens.text : Kit.Tokens.muted)
    readonly property real padding: Kit.Tokens.base * 4
    implicitWidth: words.implicitWidth + 2 * root.padding
    implicitHeight: Kit.Tokens.controlHeight
    Accessible.role: Accessible.Button
    Accessible.name: root.label
    Rectangle {
        anchors.fill: parent
        radius: Kit.Tokens.rowRadius
        color: root.filled ? Kit.Tokens.accent : (root.hovered ? Kit.Tokens.surfaceRaised : Kit.Tokens.surface)
        border.width: root.filled ? 0 : Kit.Tokens.hoverBorder
        border.color: Kit.Tokens.border
    }
    Rectangle {
        objectName: "ring"
        visible: root.activeFocus
        anchors.fill: parent
        anchors.margins: -Kit.Tokens.ringOffset
        radius: Kit.Tokens.rowRadius + Kit.Tokens.ringOffset
        color: "transparent"
        border.width: Kit.Tokens.ringThickness
        border.color: Kit.Tokens.accent
    }
    Row {
        id: words
        anchors.centerIn: parent
        spacing: Kit.Tokens.base * 2
        Kit.PadGlyph {
            visible: root.button >= 0
            anchors.verticalCenter: parent.verticalCenter
            button: Math.max(0, root.button)
            family: root.family
            ink: root.ink
        }
        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: root.label
            color: root.ink
            font: root.labelFont
        }
        Text {
            visible: root.value.length > 0
            anchors.verticalCenter: parent.verticalCenter
            text: root.value
            color: root.filled ? root.ink : Kit.Tokens.muted
            font: root.labelFont
        }
    }
}
