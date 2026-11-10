import QtQuick
import EmuSen.Theme

// A connector across the gap from the focused cover to a neighbour: 1.5 units in the hue at 60%, ending in a 2.5-radius point at 80% (constellation.md §4).
Scene {
    id: connector
    property real x1
    property real y1
    property real x2
    property real y2
    property color ink: "#FFFFFF"
    ChartLine {
        x1: connector.x1
        y1: connector.y1
        x2: connector.x2
        y2: connector.y2
        ink: connector.ink
        opacity: 0.6
    }
    Rectangle {
        x: connector.x2 - 2.5
        y: connector.y2 - 2.5
        width: 5
        height: 5
        radius: 2.5
        color: connector.ink
        opacity: 0.8
    }
}
