import QtQuick
import EmuSen.Kit as Kit

// A row whose whole width is its content (a slider, a switch with its words): the row's height, or the content's where taller, and focus marks, the content in its slot (EmuSen_Kit.md §7.8).
Kit.Walkable {
    id: root
    default property alias content: slot.data
    property bool current: false
    implicitHeight: Math.max(Kit.Tokens.rowHeight, slot.childrenRect.height + 2 * Kit.Tokens.base)
    Accessible.role: Accessible.ListItem
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
    Item {
        id: slot
        anchors.fill: parent
        anchors.leftMargin: Kit.Tokens.base * 4
        anchors.rightMargin: Kit.Tokens.base * 4
        anchors.topMargin: Kit.Tokens.base
        anchors.bottomMargin: Kit.Tokens.base
    }
}
