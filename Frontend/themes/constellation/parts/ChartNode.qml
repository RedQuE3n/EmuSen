import QtQuick
import EmuSen.Theme

// A node of the Figures chart at its place in the layout, in the chart's content units, with its four neighbours across bodies and Pointers as KeyNavigation, and the shoulders to the previous and next maker's first body (constellation.md §3; EmuSen_Kit.md §7.31).
Walkable {
    id: node
    required property FigureLayout layout
    required property Repeater bodies
    required property Repeater pointers
    required property Faces faces
    required property int consoleCount
    // How many nodes are made; a neighbour is looked up again as each is.
    required property int made
    // The chart's shown span in content units; a node wholly outside it is not drawn.
    required property real viewTop
    readonly property bool inView: node.y + node.height > node.viewTop - 24 && node.y < node.viewTop + 468 + 24
    // The node's index in the layout: a console's row, or count + a Pointer's place.
    property int nodeIndex
    property int figure: -1
    property bool isPointer
    readonly property real cx: node.layout?.xs[node.nodeIndex] ?? 0
    readonly property real cy: (node.layout?.ys[node.nodeIndex] ?? 96) - 96
    readonly property int leftIndex: node.layout?.left[node.nodeIndex] ?? -1
    readonly property int rightIndex: node.layout?.right[node.nodeIndex] ?? -1
    readonly property int upIndex: node.layout?.up[node.nodeIndex] ?? -1
    readonly property int downIndex: node.layout?.down[node.nodeIndex] ?? -1
    readonly property Item leftItem: node.leftIndex < 0 || node.made < 0 ? null : (node.leftIndex < node.consoleCount ? node.bodies.itemAt(node.leftIndex) : node.pointers.itemAt(node.leftIndex - node.consoleCount))
    readonly property Item rightItem: node.rightIndex < 0 || node.made < 0 ? null : (node.rightIndex < node.consoleCount ? node.bodies.itemAt(node.rightIndex) : node.pointers.itemAt(node.rightIndex - node.consoleCount))
    readonly property Item upItem: node.upIndex < 0 || node.made < 0 ? null : (node.upIndex < node.consoleCount ? node.bodies.itemAt(node.upIndex) : node.pointers.itemAt(node.upIndex - node.consoleCount))
    readonly property Item downItem: node.downIndex < 0 || node.made < 0 ? null : (node.downIndex < node.consoleCount ? node.bodies.itemAt(node.downIndex) : node.pointers.itemAt(node.downIndex - node.consoleCount))
    // LB and RB: the previous and next maker's first body.
    readonly property int previousFirst: node.figure > 0 ? node.layout?.firsts[node.figure - 1] ?? -1 : -1
    readonly property int nextFirst: node.layout?.firsts[node.figure + 1] ?? -1
    KeyNavigation.left: node.leftItem
    KeyNavigation.right: node.rightItem
    KeyNavigation.up: node.upItem
    KeyNavigation.down: node.downItem
    Keys.onPressed: (event) => {
        const to = event.key === Qt.Key_PageUp ? node.previousFirst : (event.key === Qt.Key_PageDown ? node.nextFirst : -1)
        event.accepted = to >= 0
        if (to >= 0)
            node.bodies.itemAt(to).forceActiveFocus()
    }
}
