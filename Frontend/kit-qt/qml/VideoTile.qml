pragma ComponentBehavior: Bound
import QtQuick
import QtMultimedia
import EmuSen.Kit as Kit

// A clip over its still: the still first, the clip faded in over it the poster's delay after `playing` turns true, stopped and the still shown again when it turns false; drawn by VideoOutput on the GPU and by VideoFrames on the software backend, silent unless `muted` is false (EmuSen_Kit.md §7.48).
Item {
    id: root
    property url source
    property url still
    // Whether the clip is to play: the host's focus, the player's setting and the screen shown.
    property bool playing: false
    property bool muted: true
    property real volume: 1
    readonly property bool software: GraphicsInfo.api === GraphicsInfo.Software
    // Whether a frame of the clip stands over the still.
    readonly property bool showing: player.playbackState === MediaPlayer.PlayingState && (root.software ? frames.frames > 0 : player.position > 0)
    readonly property alias player: player
    readonly property alias frames: frames
    clip: true
    Image {
        objectName: "videoStill"
        anchors.fill: parent
        source: root.still
        fillMode: Image.PreserveAspectCrop
        asynchronous: true
    }
    Item {
        objectName: "videoClip"
        anchors.fill: parent
        opacity: root.showing ? 1 : 0
        Behavior on opacity {
            NumberAnimation {
                duration: Kit.Tokens.posterFade
            }
        }
        VideoOutput {
            id: output
            anchors.fill: parent
            visible: !root.software
            fillMode: VideoOutput.PreserveAspectCrop
        }
        Kit.VideoFrames {
            id: frames
            objectName: "videoFrames"
            anchors.fill: parent
            visible: root.software
        }
    }
    MediaPlayer {
        id: player
        objectName: "videoPlayer"
        loops: MediaPlayer.Infinite
        videoOutput: root.software ? frames.videoSink : output
        audioOutput: AudioOutput {
            muted: root.muted
            volume: root.volume
        }
    }
    // The poster's delay, then the clip; reduced motion keeps the still (EmuSen_Interface.md §5.9).
    Timer {
        interval: Kit.Tokens.poster
        running: root.playing && root.source.toString().length > 0 && !Kit.Tokens.reducedMotion
        onTriggered: {
            player.source = root.source
            player.play()
        }
    }
    // Stopped and cleared when it is not to play, or plays another clip: the still stands alone again.
    onPlayingChanged: if (!root.playing) player.source = ""
    onSourceChanged: player.source = ""
    Connections {
        target: player
        function onSourceChanged() { frames.clear() }
    }
}
