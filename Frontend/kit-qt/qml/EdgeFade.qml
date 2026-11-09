pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

// What it holds, faded to transparency over `length` at each edge it is told to fade, on every backend (EmuSen_Kit.md §7.35).
Item {
    id: root
    property bool fadeTop
    property bool fadeBottom
    property bool fadeLeft
    property bool fadeRight
    // How far each fade reaches in.
    property real length: Kit.Tokens.base * 7
    default property alias content: held.data
    readonly property bool fading: root.fadeTop || root.fadeBottom || root.fadeLeft || root.fadeRight
    // The software backend draws no shader effect; there the EdgeMask inside the layer fades it.
    readonly property bool software: GraphicsInfo.api === GraphicsInfo.Software
    // On at once, off only once the event queue is clear: Qt deletes a layer when it goes off, under a geometry change still telling it (EmuSen_Kit.md §7.35).
    layer.enabled: root.fading || held.layered
    layer.effect: root.software ? null : ramps
    Item {
        id: held
        anchors.fill: parent
        // A shade under whole until the software layer holds an alpha channel, so Qt makes the layer with one (EmuSen_Kit.md §7.35).
        opacity: root.software && !mask.primed ? 0.999 : 1
        property bool layered
        Binding on layered {
            value: root.fading
            delayed: true
        }
    }
    Kit.EdgeMask {
        id: mask
        anchors.fill: parent
        visible: root.fading && root.software
        fadeTop: root.fadeTop
        fadeBottom: root.fadeBottom
        fadeLeft: root.fadeLeft
        fadeRight: root.fadeRight
        length: root.length
    }
    Component {
        id: ramps
        ShaderEffect {
            readonly property real across: Math.max(1, root.height)
            readonly property real along: Math.max(1, root.width)
            property vector4d reach: Qt.vector4d(root.fadeTop ? root.length / across : 0, root.fadeBottom ? root.length / across : 0, root.fadeLeft ? root.length / along : 0, root.fadeRight ? root.length / along : 0)
            fragmentShader: "qrc:/qt/qml/EmuSen/Kit/shaders/edge_fade.frag.qsb"
        }
    }
}
