import QtQuick
import QtQuick.Effects
import EmuSen.Theme

// A star of the chart, centred in its box: 12 in TextStrong (the ink by day) with a 10-unit glow; lit, 18 in Accent with a 4-unit halo, a 24-unit glow and the circle ring; pressed, 14 (constellation.md §1.6).
Scene {
    id: star
    property bool lit
    property bool pressed
    readonly property bool light: Tokens.variant === Tokens.Light
    readonly property real size: star.lit ? (star.pressed ? 14 : 18) : 12
    width: 32
    height: 32
    RectangularShadow {
        anchors.centerIn: dot
        width: dot.width
        height: dot.height
        radius: dot.radius
        blur: star.lit ? 24 : 10
        color: star.lit ? Tokens.accent : (star.light ? "#1F2A44" : "#C7D3F0")
        opacity: star.lit ? 1 : 0.6
    }
    Rectangle {
        anchors.centerIn: dot
        visible: star.lit
        width: dot.width + 8
        height: dot.height + 8
        radius: width / 2
        color: star.light ? "#1F2A44" : "#C7D3F0"
        opacity: 0.18
    }
    Rectangle {
        id: dot
        anchors.centerIn: parent
        width: star.size
        height: star.size
        radius: star.size / 2
        color: star.lit ? Tokens.accent : (star.light ? "#1F2A44" : Tokens.textStrong)
        Behavior on width { NumberAnimation { duration: 120; easing.type: Easing.OutQuad } }
        Behavior on height { NumberAnimation { duration: 120; easing.type: Easing.OutQuad } }
    }
    Rectangle {
        objectName: "starRing"
        anchors.centerIn: parent
        visible: star.lit
        width: 32
        height: 32
        radius: 16
        color: "transparent"
        border.color: Tokens.accent
        border.width: Tokens.ringThickness
    }
}
