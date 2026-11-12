pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Esde

// A video played at once in its box, fitted or stretched, with its sound or without, and blurred across by the sigma the scene gives, as ES-DE's media viewer plays a game's video under its settings (EmuSen_Qt.md §4.10, §4.14); the kit's tile and its one decoder.
ElementBox {
    id: root
    required property url source
    required property bool stretch
    required property bool audio
    required property real volume
    required property real blur
    required property real fade
    Kit.VideoTile {
        anchors.fill: parent
        source: root.source
        playing: true
        muted: !root.audio
        volume: root.volume
        fillMode: root.stretch ? Image.Stretch : Image.PreserveAspectFit
        delay: 0
        fade: root.fade
        layer.enabled: root.blur > 0
        layer.effect: ShaderEffect {
            property real sigma: root.blur
            property real texel: 1 / Math.max(1, root.width)
            fragmentShader: "qrc:/emusen/esde/hblur.frag.qsb"
        }
    }
}
