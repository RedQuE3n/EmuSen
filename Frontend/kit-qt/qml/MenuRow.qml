import QtQuick
import EmuSen.Kit

// A menu's row: a label and a value, the focus bar at its left when focused, the hover border; walkable, so a click is A there (EmuSen_Kit.md §7.8).
Walkable {
    id: root
    property string label
    property string value
    property bool current: false
    implicitHeight: Tokens.rowHeight
    Accessible.role: Accessible.ListItem
    Accessible.name: root.label
    Rectangle {
        anchors.fill: parent
        radius: Tokens.rowRadius
        color: root.activeFocus || root.current ? Tokens.surfaceRaised : Tokens.surface
        border.width: root.hovered ? Tokens.hoverBorder : 0
        border.color: Tokens.border
        Rectangle {
            width: Tokens.barWidth
            height: parent.height
            radius: Tokens.rowRadius
            color: Tokens.accent
            visible: root.activeFocus
        }
    }
    Text {
        objectName: "label"
        text: root.label
        color: root.activeFocus ? Tokens.textStrong : Tokens.text
        font.pixelSize: Tokens.fontRow
        elide: Text.ElideRight
        anchors.left: parent.left
        anchors.leftMargin: Tokens.base * 4
        anchors.right: valueText.left
        anchors.rightMargin: Tokens.base * 2
        anchors.verticalCenter: parent.verticalCenter
    }
    Text {
        id: valueText
        objectName: "value"
        text: root.value
        color: Tokens.muted
        font.pixelSize: Tokens.fontRow
        anchors.right: parent.right
        anchors.rightMargin: Tokens.base * 4
        anchors.verticalCenter: parent.verticalCenter
    }
}
