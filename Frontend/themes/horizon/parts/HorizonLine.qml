import QtQuick
import EmuSen.Theme

// The horizon line at 440: a line clear at its ends, or the strip's position track from 64 to W - 64 when the row is longer than it (EmuSen_Qt.md §3.4.2).
Item {
    id: root
    // The row's length, the camera's offset and the room it is shown in, in canvas units.
    property real total: 0
    property real off: 0
    readonly property real view: root.width - 128
    readonly property bool tracks: root.total > root.view + 0.5
    readonly property color line: Tokens.variant === Tokens.Dark ? "#528C96DC" : "#471F2A44"
    readonly property real share: Math.max(24 / root.view, root.view / Math.max(1, root.total))
    readonly property real at: Math.min(1 - root.share, root.off / Math.max(1, root.total))
    y: 440
    height: 1
    Rectangle {
        objectName: "horizonLine"
        visible: !root.tracks
        width: root.width
        height: 1
        gradient: Gradient {
            orientation: Gradient.Horizontal
            GradientStop { position: 0; color: "transparent" }
            GradientStop { position: 0.12; color: root.line }
            GradientStop { position: 0.88; color: root.line }
            GradientStop { position: 1; color: "transparent" }
        }
    }
    Rectangle {
        objectName: "horizonTrack"
        visible: root.tracks
        x: 64
        y: -1
        width: root.view
        height: 2
        radius: 1
        color: root.line
        Rectangle {
            objectName: "horizonThumb"
            x: root.at * root.view
            y: -0.5
            width: root.share * root.view
            height: 3
            radius: 1.5
            color: Tokens.border
        }
    }
}
