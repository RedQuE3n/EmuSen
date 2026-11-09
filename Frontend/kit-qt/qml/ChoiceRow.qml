import QtQuick
import EmuSen.Kit as Kit

// One choice of a page of choices: a menu row with the choice in force marked by a dot before its label (EmuSen_Kit.md §7.41).
Kit.MenuRow {
    id: root
    property bool marked: false
    inset: Kit.Tokens.base * 8
    Accessible.checkable: true
    Accessible.checked: root.marked
    Rectangle {
        objectName: "mark"
        visible: root.marked
        width: Kit.Tokens.base * 2.5
        height: width
        radius: width / 2
        x: (root.inset - width) / 2
        anchors.verticalCenter: parent.verticalCenter
        color: Kit.Tokens.accent
    }
}
