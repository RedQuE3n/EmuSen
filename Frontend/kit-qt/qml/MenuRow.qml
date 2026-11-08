import QtQuick
import EmuSen.Kit as Kit

// A menu's row: a label and a value, the focus bar at its left when focused, the hover border; walkable, so a click is A there (EmuSen_Kit.md §7.8).
Kit.Walkable {
    id: root
    property string label
    property string value
    property bool current: false
    implicitHeight: Kit.Tokens.rowHeight
    Accessible.role: Accessible.ListItem
    Accessible.name: root.label
    Rectangle {
        anchors.fill: parent
        radius: Kit.Tokens.rowRadius
        color: root.activeFocus || root.current ? Kit.Tokens.surfaceRaised : Kit.Tokens.surface
        border.width: root.hovered ? Kit.Tokens.hoverBorder : 0
        border.color: Kit.Tokens.border
        Rectangle {
            width: Kit.Tokens.barWidth
            height: parent.height
            radius: Kit.Tokens.rowRadius
            color: Kit.Tokens.accent
            visible: root.activeFocus
        }
    }
    Text {
        objectName: "label"
        text: root.label
        color: root.activeFocus ? Kit.Tokens.textStrong : Kit.Tokens.text
        font.pixelSize: Kit.Tokens.fontRow
        elide: Text.ElideRight
        anchors.left: parent.left
        anchors.leftMargin: Kit.Tokens.base * 4
        anchors.right: valueText.left
        anchors.rightMargin: Kit.Tokens.base * 2
        anchors.verticalCenter: parent.verticalCenter
    }
    Text {
        id: valueText
        objectName: "value"
        text: root.value
        color: Kit.Tokens.muted
        font.pixelSize: Kit.Tokens.fontRow
        anchors.right: parent.right
        anchors.rightMargin: Kit.Tokens.base * 4
        anchors.verticalCenter: parent.verticalCenter
    }
}
