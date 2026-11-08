pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import EmuSen.Kit

// A menu panel: rows in a ListView on the raised surface, moved by the pad's keys, the wheel and a scroll bar; the current row holds the focus (EmuSen_Kit.md §7.8).
Rectangle {
    id: root
    required property FrameController controller
    property alias model: list.model
    property alias delegate: list.delegate
    property alias currentIndex: list.currentIndex
    property alias view: list
    signal activated(int index)
    radius: Tokens.tileRadius
    color: Tokens.surfaceRaised
    implicitWidth: Tokens.cardWidth
    implicitHeight: Tokens.rowHeight * 6 + Tokens.base * 4
    ListView {
        id: list
        anchors.fill: parent
        anchors.margins: Tokens.base * 2
        clip: true
        focus: true
        keyNavigationEnabled: true
        keyNavigationWraps: false
        highlightMoveDuration: Tokens.instant
        spacing: Tokens.base / 2
        delegate: MenuRow {
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
        ScrollBar.vertical: ScrollBar {}
    }
}
