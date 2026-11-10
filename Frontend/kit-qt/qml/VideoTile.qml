pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

// A clip over its still: the still first, the clip faded in over it the poster's delay after `playing` turns true, and the still alone again when it turns false or the clip changes; its player (VideoClip) is made only while the clip is to play; silent unless `muted` is false (EmuSen_Kit.md §7.53).
Item {
    id: root
    property url source
    property url still
    // Whether the clip is to play: the host's focus, the player's setting and the screen shown.
    property bool playing: false
    property bool muted: true
    property real volume: 1
    // How the clip and the still meet the tile: cropped to fill it (the default), fitted inside it or stretched to it; and how long after
    // `playing` the clip starts, the poster's delay by default (EmuSen_Kit.md §7.61).
    property int fillMode: Image.PreserveAspectCrop
    property int delay: Kit.Tokens.poster
    // Whether the clip's player is made, and whether a frame of the clip stands over the still.
    readonly property bool running: clipLoader.status === Loader.Ready
    readonly property bool showing: (clipLoader.item as Kit.VideoClip)?.showing ?? false
    // The frames the kit's sink has drawn: the software backend's, none on the GPU.
    readonly property int frames: (clipLoader.item as Kit.VideoClip)?.frames ?? 0
    clip: true
    Image {
        objectName: "videoStill"
        anchors.fill: parent
        source: root.still
        fillMode: root.fillMode
        asynchronous: true
    }
    // Reduced motion keeps the still (EmuSen_Interface.md §5.9).
    Loader {
        id: clipLoader
        objectName: "videoClip"
        anchors.fill: parent
        active: root.playing && root.source.toString().length > 0 && !Kit.Tokens.reducedMotion
        sourceComponent: Kit.VideoClip {
            source: root.source
            muted: root.muted
            volume: root.volume
            fillMode: root.fillMode
            delay: root.delay
        }
    }
}
