import QtQuick
import EmuSen.Theme

// The land's strip for a free arrangement: items placed in its content from x 0 on the foot line 700, drawn at 64 - off under the clamped camera, faded over the outer 40 units at each edge (EmuSen_Qt.md §3.4.2).
Item {
    id: root
    // The focused item's place and width, and the row's length, in content units.
    property real focusX: 0
    property real focusWidth: 0
    property real total: 0
    readonly property real view: root.width - 128
    readonly property real off: Math.max(0, Math.min(root.focusX + root.focusWidth / 2 - root.view / 2, Math.max(0, root.total - root.view)))
    // Where an item's foot stands in the strip: the foot line 700 under the strip's top at 440.
    readonly property real foot: 260
    default property alias content: flick.flickableData
    // A wheel's notch steps the focus, as Left and Right do: positive to the right.
    signal stepped(int by)
    y: 440
    height: 302
    EdgeFade {
        anchors.fill: parent
        fadeLeft: flick.fadeLeft
        fadeRight: flick.fadeRight
        length: 40
        Flickable {
            id: flick
            objectName: "strip"
            // The fit audit's edge rule reads a faded edge from these.
            property bool fadeLeft: true
            property bool fadeRight: true
            anchors.fill: parent
            flickableDirection: Flickable.HorizontalFlick
            boundsBehavior: Flickable.StopAtBounds
            leftMargin: 64
            rightMargin: 64
            contentWidth: root.total
            contentHeight: root.height
            // The camera follows the focus; a drag may look along the row, and it returns once let go.
            Binding on contentX {
                value: root.off - 64
                when: !flick.moving
                restoreMode: Binding.RestoreNone
            }
            WheelHandler {
                onWheel: (event) => root.stepped(event.angleDelta.y + event.angleDelta.x < 0 ? 1 : -1)
            }
            Behavior on contentX {
                NumberAnimation {
                    duration: Tokens.reel
                    easing.type: Easing.OutQuad
                }
            }
        }
    }
}
