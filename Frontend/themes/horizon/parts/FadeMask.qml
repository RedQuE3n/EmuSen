import QtQuick

// The strip's mask: clear at each edge, whole from 40 units in, read by a MultiEffect and never drawn itself (EmuSen_Qt.md §3.4.2).
Rectangle {
    id: root
    readonly property real edge: 40 / Math.max(1, root.width)
    visible: false
    layer.enabled: true
    gradient: Gradient {
        orientation: Gradient.Horizontal
        GradientStop { position: 0; color: "transparent" }
        GradientStop { position: root.edge; color: "#FFFFFFFF" }
        GradientStop { position: 1 - root.edge; color: "#FFFFFFFF" }
        GradientStop { position: 1; color: "transparent" }
    }
}
