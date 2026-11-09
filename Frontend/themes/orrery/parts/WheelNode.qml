import QtQuick
import EmuSen.Theme

// A node of Wheelwork's walk, a world or a stop of the dial, with its four neighbours as KeyNavigation and the shoulders to the previous and next maker's stop, all the layout's and still while the wheel turns; focused, it turns the wheel (orrery.md §3; EmuSen_Kit.md §7.32).
Walkable {
    id: node
    required property WheelLayout layout
    required property Repeater worlds
    required property Repeater shelves
    required property Repeater makers
    // How many delegates are made; a neighbour is looked up again as each is.
    required property int made
    // The node in the layout: a console's row, count + a shelf's place, or count + 3 + a maker's group.
    property int nodeIndex: -1
    readonly property int consoleCount: node.layout?.count ?? 0
    // The lists a neighbour is looked up in, which the engine clears as the screen is torn down.
    readonly property bool ready: node.worlds !== null && node.shelves !== null && node.makers !== null && node.layout !== null
    readonly property int leftIndex: node.made < 0 || !node.ready ? -1 : node.layout?.left[node.nodeIndex] ?? -1
    readonly property int rightIndex: node.made < 0 || !node.ready ? -1 : node.layout?.right[node.nodeIndex] ?? -1
    readonly property int upIndex: node.made < 0 || !node.ready ? -1 : node.layout?.up[node.nodeIndex] ?? -1
    readonly property int downIndex: node.made < 0 || !node.ready ? -1 : node.layout?.down[node.nodeIndex] ?? -1
    readonly property int previousIndex: node.made < 0 || !node.ready ? -1 : node.layout?.previous[node.nodeIndex] ?? -1
    readonly property int nextIndex: node.made < 0 || !node.ready ? -1 : node.layout?.next[node.nodeIndex] ?? -1
    readonly property Item leftItem: node.leftIndex < 0 ? null : (node.leftIndex < node.consoleCount ? node.worlds.itemAt(node.leftIndex) : (node.leftIndex < node.consoleCount + 3 ? node.shelves.itemAt(node.leftIndex - node.consoleCount) : node.makers.itemAt(node.layout.firsts[node.leftIndex - node.consoleCount - 3])))
    readonly property Item rightItem: node.rightIndex < 0 ? null : (node.rightIndex < node.consoleCount ? node.worlds.itemAt(node.rightIndex) : (node.rightIndex < node.consoleCount + 3 ? node.shelves.itemAt(node.rightIndex - node.consoleCount) : node.makers.itemAt(node.layout.firsts[node.rightIndex - node.consoleCount - 3])))
    readonly property Item upItem: node.upIndex < 0 ? null : (node.upIndex < node.consoleCount ? node.worlds.itemAt(node.upIndex) : (node.upIndex < node.consoleCount + 3 ? node.shelves.itemAt(node.upIndex - node.consoleCount) : node.makers.itemAt(node.layout.firsts[node.upIndex - node.consoleCount - 3])))
    readonly property Item downItem: node.downIndex < 0 ? null : (node.downIndex < node.consoleCount ? node.worlds.itemAt(node.downIndex) : (node.downIndex < node.consoleCount + 3 ? node.shelves.itemAt(node.downIndex - node.consoleCount) : node.makers.itemAt(node.layout.firsts[node.downIndex - node.consoleCount - 3])))
    readonly property Item previousItem: node.previousIndex < node.consoleCount + 3 || !node.ready ? null : node.makers.itemAt(node.layout.firsts[node.previousIndex - node.consoleCount - 3])
    readonly property Item nextItem: node.nextIndex < node.consoleCount + 3 || !node.ready ? null : node.makers.itemAt(node.layout.firsts[node.nextIndex - node.consoleCount - 3])
    KeyNavigation.left: node.leftItem
    KeyNavigation.right: node.rightItem
    KeyNavigation.up: node.upItem
    KeyNavigation.down: node.downItem
    Keys.onPressed: (event) => {
        const to = event.key === Qt.Key_PageUp ? node.previousItem : (event.key === Qt.Key_PageDown ? node.nextItem : null)
        event.accepted = to !== null
        if (to !== null)
            to.forceActiveFocus()
    }
    onActiveFocusChanged: if (node.activeFocus) node.layout.current = node.nodeIndex
}
