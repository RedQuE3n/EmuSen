import QtQuick
import EmuSen.Theme

// One item of a free strip: what the world and the words show of it, where it stands in the strip, and its walk on one axis, the shoulders jumping a group (EmuSen_Qt.md §3.4.2).
Walkable {
    id: item
    property int kind: 0
    property string key
    property string title
    property string facts
    property string picture
    property string world
    property string mark
    property string line
    property bool attention: false
    property bool available: true
    property real hue: -1
    // The item's left edge in the strip's content, which the camera follows.
    property real stripX: 0
    property Item leftItem: null
    property Item rightItem: null
    property Item previousItem: null
    property Item nextItem: null
    property alias boxX: lift.boxX
    property alias boxY: lift.boxY
    property alias boxWidth: lift.boxWidth
    property alias boxHeight: lift.boxHeight
    property alias round: lift.round
    default property alias content: lift.content
    readonly property bool lit: item.activeFocus
    // The strip it stands in: an item wholly outside the window is not drawn, though the row still places it.
    property var strip: null
    readonly property bool onScreen: !item.strip || (item.stripX + 64 - item.strip.off < item.strip.width + 40 && item.stripX + item.width + 64 - item.strip.off > -40)
    opacity: item.onScreen ? 1 : 0
    // Whether the item steps and jumps itself; a list's item leaves Left, Right and the shoulders to its list and frame.
    property bool steps: true
    // Whether the focused item rests unlifted, as the search's matches do.
    property bool still: false
    // A walking key the item took, for a screen that walks by place in the strip rather than by neighbour.
    signal walked(int key)
    // Left and Right step, the shoulders jump a group, Up and Down do nothing: one axis.
    Keys.onPressed: (event) => {
        const to = event.key === Qt.Key_Left ? item.leftItem : (event.key === Qt.Key_Right ? item.rightItem : (event.key === Qt.Key_PageUp ? item.previousItem : (event.key === Qt.Key_PageDown ? item.nextItem : null)))
        if (to && item.steps)
            to.forceActiveFocus()
        event.accepted = event.key === Qt.Key_Up || event.key === Qt.Key_Down || item.steps && (event.key === Qt.Key_Left || event.key === Qt.Key_Right || event.key === Qt.Key_PageUp || event.key === Qt.Key_PageDown)
        if (event.accepted)
            item.walked(event.key)
    }
    PointHandler {
        id: point
    }
    Lift {
        id: lift
        anchors.fill: parent
        lit: item.lit
        held: point.active || item.still
        hovered: item.hovered
        available: item.available
    }
}
