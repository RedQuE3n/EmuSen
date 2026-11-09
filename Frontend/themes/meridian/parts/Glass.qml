import QtQuick
import EmuSen.Theme

// The rail's glass: SurfaceRaised at 50% (white at 80% by day) with a 1-unit edge of #8C96DC at 18%, radius 12 (meridian.md §1.4).
Scene {
    id: glass
    readonly property bool light: Tokens.variant === Tokens.Light
    Rectangle {
        objectName: "railGlass"
        anchors.fill: parent
        radius: 12
        color: glass.light ? "#FFFFFF" : Tokens.surfaceRaised
        opacity: glass.light ? 0.8 : 0.5
    }
    Rectangle {
        objectName: "railEdge"
        anchors.fill: parent
        radius: 12
        color: "transparent"
        border.width: 1
        border.color: "#8C96DC"
        opacity: 0.18
    }
}
