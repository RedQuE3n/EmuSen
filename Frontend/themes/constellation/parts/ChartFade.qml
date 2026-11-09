import QtQuick

// The chart's viewport mask: opaque, fading 24 units to nothing at an edge the chart runs past, so a row there is faded and never cut (constellation.md §3.1).
Item {
    id: mask
    property bool atTop
    property bool atBottom
    layer.enabled: true
    visible: false
    Rectangle {
        y: mask.atTop ? 24 : 0
        width: mask.width
        height: mask.height - (mask.atTop ? 24 : 0) - (mask.atBottom ? 24 : 0)
        color: "#FFFFFF"
    }
    Rectangle {
        visible: mask.atTop
        width: mask.width
        height: 24
        gradient: Gradient {
            GradientStop { position: 0; color: "transparent" }
            GradientStop { position: 1; color: "#FFFFFF" }
        }
    }
    Rectangle {
        visible: mask.atBottom
        y: mask.height - 24
        width: mask.width
        height: 24
        gradient: Gradient {
            GradientStop { position: 0; color: "#FFFFFF" }
            GradientStop { position: 1; color: "transparent" }
        }
    }
}
