import QtQuick
import QtQuick.Effects
import EmuSen.Theme

// The moon: the crescent mark filled in the moon's ink with a 24-unit glow at 35% by night; by day the Atlas's ink and no glow (orrery.md §1.4).
Scene {
    id: moon
    readonly property bool light: Tokens.variant === Tokens.Light
    RectangularShadow {
        visible: !moon.light
        anchors.centerIn: parent
        width: moon.width * 0.6
        height: moon.height * 0.6
        radius: width / 2
        blur: 24
        color: Qt.rgba(233 / 255, 228 / 255, 212 / 255, 0.35)
    }
    Glyph {
        anchors.fill: parent
        path: Marks.crescent
        filled: true
        ink: moon.light ? "#6B5B4A" : "#E9E4D4"
    }
}
