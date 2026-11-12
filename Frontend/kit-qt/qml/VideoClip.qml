pragma ComponentBehavior: Bound
import QtQuick
import QtMultimedia
import EmuSen.Kit as Kit

// VideoTile's player, made only while a clip is to play, so a window that shows none never starts Qt Multimedia: the clip after the poster's delay, looped, drawn by VideoOutput on the GPU and by VideoFrames on the software backend (EmuSen_Kit.md §7.53).
Item {
    id: root
    objectName: "videoPlayback"
    property url source
    property bool muted: true
    property real volume: 1
    // VideoTile's fill and delay (EmuSen_Kit.md §7.61).
    property int fillMode: Image.PreserveAspectCrop
    property int delay: Kit.Tokens.poster
    // How long the clip fades in once its first frame shows, in ms: the poster's fade by default (EmuSen_Kit.md §7.53).
    property int fade: Kit.Tokens.posterFade
    readonly property bool software: GraphicsInfo.api === GraphicsInfo.Software
    readonly property bool showing: player.playbackState === MediaPlayer.PlayingState && (root.software ? frames.frames > 0 : player.position > 0)
    readonly property int frames: frames.frames
    // Where the clip is, in ms, whether its sound is let out and at what share of full: what the player sees and hears of it (EmuSen_Kit.md §7.53).
    readonly property real position: player.position
    readonly property bool heard: !sound.muted
    readonly property real level: sound.volume
    opacity: root.showing ? 1 : 0
    Behavior on opacity {
        NumberAnimation {
            duration: root.fade
        }
    }
    VideoOutput {
        id: output
        anchors.fill: parent
        visible: !root.software
        fillMode: root.fillMode === Image.Stretch ? VideoOutput.Stretch : root.fillMode === Image.PreserveAspectFit ? VideoOutput.PreserveAspectFit : VideoOutput.PreserveAspectCrop
    }
    Kit.VideoFrames {
        id: frames
        objectName: "videoFrames"
        anchors.fill: parent
        fitted: root.fillMode === Image.PreserveAspectFit
        stretched: root.fillMode === Image.Stretch
        visible: root.software
    }
    MediaPlayer {
        id: player
        objectName: "videoPlayer"
        loops: MediaPlayer.Infinite
        videoOutput: root.software ? frames.videoSink : output
        audioOutput: AudioOutput {
            id: sound
            muted: root.muted
            volume: root.volume
        }
    }
    // The poster's delay, then the clip (EmuSen_Interface.md §5.9).
    Timer {
        interval: Math.max(1, root.delay)
        running: true
        onTriggered: {
            player.source = root.source
            player.play()
        }
    }
}
