import QtQuick
import EmuSen.Kit as Kit

// A scroller's faded edges, soft and the same on every backend: the backdrop's colour ramped over each faded edge (EmuSen_Kit.md §7.35).
Item {
    id: root
    // The scroller whose edges fade, standing under the same parent; the ramps lie over it.
    required property Item source
    property bool fadeTop
    property bool fadeBottom
    property bool fadeLeft
    property bool fadeRight
    // How far each fade reaches into the scroller.
    property real length: Kit.Tokens.base * 7
    // What stands behind the scroller, the colour each ramp reaches at the edge.
    property color backdrop: Kit.Tokens.surface
    x: root.source.x
    y: root.source.y
    width: root.source.width
    height: root.source.height
    z: root.source.z + 1
    Kit.EdgeRamp { edge: Qt.TopEdge; shown: root.fadeTop; length: root.length; ink: root.backdrop }
    Kit.EdgeRamp { edge: Qt.BottomEdge; shown: root.fadeBottom; length: root.length; ink: root.backdrop }
    Kit.EdgeRamp { edge: Qt.LeftEdge; shown: root.fadeLeft; length: root.length; ink: root.backdrop }
    Kit.EdgeRamp { edge: Qt.RightEdge; shown: root.fadeRight; length: root.length; ink: root.backdrop }
}
