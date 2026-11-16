import QtQuick

// An item blurred as CSS's filter blur() blurs it: a Gaussian of `deviation`, out to three deviations past the item's box,
// drawn in two separable passes at a texel of up to an eighth of the deviation (EmuSen_Kit.md §7.118). Its box is the item's
// grown by `reach` on every side; the item itself is hidden.
Item {
    id: root
    required property Item source
    // The Gaussian's standard deviation, in this item's units.
    property real deviation
    // Scene pixels to one of this item's units, so the passes run at the pixels the card draws.
    property real pixelScale: 1
    readonly property real reach: 3 * root.deviation
    readonly property real deviationPixels: root.deviation * root.pixelScale
    // Scene pixels a texel: one up to eight pixels of deviation, then so that each pass runs eight texels of deviation.
    readonly property real texel: Math.max(1, root.deviationPixels / 8)
    // The passes' deviation in texels: drawing the item at a coarser texel and reading it back bilinearly each add a
    // variance, a twelfth and a sixth of a texel squared, which the passes leave out.
    readonly property real texels: root.texel > 1 ? Math.sqrt(Math.max(0, Math.pow(root.deviationPixels / root.texel, 2) - 0.25)) : root.deviationPixels
    readonly property size grid: Qt.size(Math.max(1, Math.ceil(root.width * root.pixelScale / root.texel)), Math.max(1, Math.ceil(root.height * root.pixelScale / root.texel)))
    x: root.source.x - root.reach
    y: root.source.y - root.reach
    width: root.source.width + 2 * root.reach
    height: root.source.height + 2 * root.reach
    ShaderEffectSource {
        id: drawn
        anchors.fill: parent
        visible: false
        sourceItem: root.source
        hideSource: true
        smooth: true
        sourceRect: Qt.rect(-root.reach, -root.reach, root.width, root.height)
        textureSize: root.grid
    }
    ShaderEffect {
        id: across
        anchors.fill: parent
        visible: false
        property var source: drawn
        property vector2d step: Qt.vector2d(1 / root.grid.width, 0)
        property real sigma: root.texels
        fragmentShader: "qrc:/qt/qml/EmuSen/Kit/shaders/gaussian.frag.qsb"
        layer.enabled: true
        layer.smooth: true
        layer.textureSize: root.grid
    }
    ShaderEffect {
        objectName: "gaussianDown"
        anchors.fill: parent
        property var source: across
        property vector2d step: Qt.vector2d(0, 1 / root.grid.height)
        property real sigma: root.texels
        fragmentShader: "qrc:/qt/qml/EmuSen/Kit/shaders/gaussian.frag.qsb"
    }
}
