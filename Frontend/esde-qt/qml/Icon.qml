pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Esde

// A theme's picture file or a built-in mark in one box, multiplied by one colour or two graded across it; the one not named draws nothing.
Item {
    id: root
    property url source
    property string mark
    property color tint: Qt.rgba(1, 1, 1, 1)
    property color tintEnd: root.tint
    property bool gradientVertical: false
    property bool nearest: false
    Picture {
        anchors.fill: parent
        source: root.source
        tint: root.tint
        tintEnd: root.tintEnd
        gradientVertical: root.gradientVertical
        nearest: root.nearest
    }
    // A graded mark is drawn white and multiplied by the picture's own shader pass, since a Shape's stroke takes no gradient (EmuSen_Qt.md §4.5.7).
    Mark {
        id: drawn
        readonly property bool graded: !Qt.colorEqual(root.tint, root.tintEnd)
        anchors.fill: parent
        mark: root.mark
        color: drawn.graded ? Qt.rgba(1, 1, 1, 1) : root.tint
        layer.enabled: drawn.graded && root.mark.length > 0
        layer.effect: ShaderEffect {
            readonly property color tint: root.tint
            readonly property color tintEnd: root.tintEnd
            readonly property real vertical: root.gradientVertical ? 1 : 0
            readonly property real saturation: 1
            readonly property real dim: 1
            readonly property real radius: 0
            readonly property size size: Qt.size(width, height)
            readonly property rect uvRect: Qt.rect(0, 0, 1, 1)
            readonly property size tiles: Qt.size(1, 1)
            readonly property point tileOrigin: Qt.point(0, 0)
            readonly property real tiled: 0
            readonly property real fadeFrom: 1
            readonly property real fadeFalloff: 0
            readonly property real brightness: 0
            fragmentShader: "qrc:/emusen/esde/picture.frag.qsb"
        }
    }
}
