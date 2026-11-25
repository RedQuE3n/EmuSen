import QtQuick
import EmuSen.Kit as Kit

// Focused Eclipse pill halo ring and soft glow (EclipseDialog:39, EclipsePower:58, R-20261115-2).
Item {
    id: root
    required property real unit
    required property real pixelScale
    required property real radius
    required property int rowIndex
    Kit.Glow {
        objectName: "pillGlow." + root.rowIndex
        anchors.fill: parent
        radius: root.radius
        color: Kit.Tokens.glowColor
        blur: 46 * root.unit
        spread: 6 * root.unit
        pixelScale: root.pixelScale
    }
    Rectangle {
        objectName: "pillRing." + root.rowIndex
        x: -6 * root.unit
        y: -6 * root.unit
        width: root.width + 12 * root.unit
        height: root.height + 12 * root.unit
        radius: root.radius + 6 * root.unit
        color: Qt.alpha(Kit.Tokens.accent, 0.18)
    }
}
