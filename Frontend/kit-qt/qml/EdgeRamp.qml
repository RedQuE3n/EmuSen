import QtQuick

// One edge's ramp of an EdgeFade: its ink at the edge, none at its length in (EmuSen_Kit.md §7.35).
Rectangle {
    id: ramp
    property int edge: Qt.TopEdge
    property bool shown
    property real length
    property color ink
    readonly property bool across: ramp.edge === Qt.TopEdge || ramp.edge === Qt.BottomEdge
    readonly property bool far: ramp.edge === Qt.BottomEdge || ramp.edge === Qt.RightEdge
    visible: ramp.shown
    enabled: false
    x: !ramp.across && ramp.far ? parent.width - ramp.length : 0
    y: ramp.across && ramp.far ? parent.height - ramp.length : 0
    width: ramp.across ? parent.width : ramp.length
    height: ramp.across ? ramp.length : parent.height
    gradient: Gradient {
        orientation: ramp.across ? Gradient.Vertical : Gradient.Horizontal
        GradientStop { position: 0; color: ramp.far ? Qt.alpha(ramp.ink, 0) : ramp.ink }
        GradientStop { position: 1; color: ramp.far ? ramp.ink : Qt.alpha(ramp.ink, 0) }
    }
}
