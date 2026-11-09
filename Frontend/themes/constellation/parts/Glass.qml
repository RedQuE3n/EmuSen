import QtQuick
import EmuSen.Theme

// The keys' glass: SurfaceRaised at 50% with a 1-unit edge of the ink at 18% by night; white at 60% with the Chart ink's edge at 14% by day; radius 12 (constellation.md §1.2).
Scene {
    id: glass
    readonly property bool light: Tokens.variant === Tokens.Light
    property real radius: 12
    Rectangle {
        anchors.fill: parent
        radius: glass.radius
        color: glass.light ? "#FFFFFF" : Tokens.surfaceRaised
        opacity: glass.light ? 0.6 : 0.5
    }
    Rectangle {
        anchors.fill: parent
        radius: glass.radius
        color: "transparent"
        border.width: 1
        border.color: glass.light ? "#1F2A44" : "#8C96DC"
        opacity: glass.light ? 0.14 : 0.18
    }
}
