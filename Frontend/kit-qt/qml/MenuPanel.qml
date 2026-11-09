pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import EmuSen.Kit as Kit

// A menu panel: rows in a ListView on the raised surface, moved by the pad's keys, the wheel and a scroll bar beside them; the focused row is the current one, and the edges that hide rows fade (EmuSen_Kit.md §7.8).
Rectangle {
    id: root
    required property Kit.FrameController controller
    property alias model: list.model
    property alias delegate: list.delegate
    property alias currentIndex: list.currentIndex
    property alias view: list
    signal activated(int index)
    radius: Kit.Tokens.tileRadius
    color: Kit.Tokens.surfaceRaised
    implicitWidth: Kit.Tokens.cardWidth
    implicitHeight: Kit.Tokens.rowHeight * 6 + Kit.Tokens.base * 4
    ListView {
        id: list
        objectName: "rows"
        anchors.fill: parent
        anchors.margins: Kit.Tokens.base * 2
        anchors.rightMargin: Kit.Tokens.base * 4
        clip: true
        focus: true
        keyNavigationEnabled: true
        keyNavigationWraps: false
        highlightMoveDuration: Kit.Tokens.instant
        highlightResizeDuration: Kit.Tokens.instant
        spacing: Kit.Tokens.base / 2
        property bool fadeTop: !list.atYBeginning
        property bool fadeBottom: !list.atYEnd
        delegate: Kit.MenuRow {
            id: row
            required property var model
            required property int index
            width: ListView.view.width
            controller: root.controller
            label: row.model.label
            value: row.model.value
            current: ListView.isCurrentItem
            focus: ListView.isCurrentItem
            onActivated: root.activated(row.index)
        }
        ScrollBar.vertical: ScrollBar {
            parent: root
            x: root.width - width - Kit.Tokens.base
            y: list.y
            height: list.height
        }
    }
    Rectangle {
        anchors.left: list.left
        anchors.right: list.right
        anchors.top: list.top
        height: Kit.Tokens.rowHeight / 2
        visible: list.fadeTop
        enabled: false
        gradient: Gradient {
            GradientStop { position: 0; color: Kit.Tokens.surfaceRaised }
            GradientStop { position: 1; color: "transparent" }
        }
    }
    Rectangle {
        anchors.left: list.left
        anchors.right: list.right
        anchors.bottom: list.bottom
        height: Kit.Tokens.rowHeight / 2
        visible: list.fadeBottom
        enabled: false
        gradient: Gradient {
            GradientStop { position: 0; color: "transparent" }
            GradientStop { position: 1; color: Kit.Tokens.surfaceRaised }
        }
    }
}
