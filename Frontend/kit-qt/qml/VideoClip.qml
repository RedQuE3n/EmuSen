pragma ComponentBehavior: Bound
import QtQuick
import QtMultimedia
import EmuSen.Kit as Kit

// VideoTile's player, made only while a clip is to play, so a window that shows none never starts Qt Multimedia: the clip after the poster's delay, looped, drawn by VideoOutput on the GPU and by VideoFrames on the software backend (EmuSen_Kit.md §7.53).
Item {
    id: root
    property url source
    property bool muted: true
    property real volume: 1
    readonly property bool software: GraphicsInfo.api === GraphicsInfo.Software
    readonly property bool showing: player.playbackState === MediaPlayer.PlayingState && (root.software ? frames.frames > 0 : player.position > 0)
    readonly property int frames: frames.frames
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
    // The poster's delay, then the clip (EmuSen_Interface.md §5.9).
    Timer {
        interval: Kit.Tokens.poster
        running: true
        onTriggered: {
            player.source = root.source
            player.play()
        }
    }
}
