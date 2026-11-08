import QtQuick
import EmuSen.Kit

// A row whose whole width is its content (a slider, a switch with its words): the row's height and focus marks, the content in its slot (EmuSen_Kit.md §7.8).
Walkable {
    id: root
    default property alias content: slot.data
    property bool current: false
    implicitHeight: Tokens.rowHeight
    Accessible.role: Accessible.ListItem
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
    Item {
        id: slot
        anchors.fill: parent
        anchors.leftMargin: Tokens.base * 4
        anchors.rightMargin: Tokens.base * 4
        anchors.topMargin: Tokens.base
        anchors.bottomMargin: Tokens.base
    }
}
