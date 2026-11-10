import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Esde

// A video played fitted to its box at once, with its sound, as ES-DE's media viewer plays a game's video (EmuSen_Qt.md §4.10); the kit's tile and its one decoder.
ElementBox {
    id: root
    required property url source
    Kit.VideoTile {
        anchors.fill: parent
        source: root.source
        playing: true
        muted: false
        fillMode: Image.PreserveAspectFit
        delay: 0
    }
}
