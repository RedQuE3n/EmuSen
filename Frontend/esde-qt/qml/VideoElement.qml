pragma ComponentBehavior: Bound
import QtQuick
import QtMultimedia
import EmuSen.Kit as Kit
import EmuSen.Esde

// A theme's video element (THEMES.md, video; EmuSen_Qt.md §4.5.6): its still until the scene says the delay has passed, then the video over its black frame, faded in each time it starts, sized from its own resolution, cropped, tinted and rounded in the picture's one shader pass, played its iterations and then gone or its still again.
ElementBox {
    id: root
    // The still, a picture's roles; an empty source draws none.
    required property url source
    required property int fit
    required property real cropX
    required property real cropY
    required property color tint
    required property color tintEnd
    required property bool gradientVertical
    required property real saturation
    required property real radius
    required property bool nearest
    required property real brightness
    // The video.
    required property url videoSource
    required property bool playing
    required property bool audio
    required property real volume
    required property real fade
    required property bool blackFrame
    required property bool pillarboxes
    required property real thresholdX
    required property real thresholdY
    required property int iterations
    required property bool thenStill
    required property int videoFit
    required property real targetWidth
    required property real targetHeight
    required property real areaWidth
    required property real areaHeight
    required property real anchorX
    required property real anchorY
    required property real placeX
    required property real placeY
    required property real videoCropX
    required property real videoCropY
    required property real videoRadius
    required property bool videoNearest
    required property color videoTint
    required property color videoTintEnd
    required property bool videoGradientVertical
    required property real videoSaturation

    // Whether the video's player is made: the scene's delay has passed, there is a video, the view is shown and its plays are not done.
    readonly property bool started: root.playing && root.visible && root.videoSource.toString().length > 0 && !root.done
    // How many times it has played to its end since it last started.
    property int plays: 0
    readonly property bool done: root.iterations > 0 && root.plays >= root.iterations
    onVideoSourceChanged: root.plays = 0
    onPlayingChanged: if (!root.playing) root.plays = 0

    Picture {
        id: still
        objectName: "videoStill"
        anchors.fill: parent
        visible: !root.started && (!root.done || root.thenStill)
        source: root.source
        fit: root.fit
        cropX: root.cropX
        cropY: root.cropY
        tint: root.tint
        tintEnd: root.tintEnd
        gradientVertical: root.gradientVertical
        saturation: root.saturation
        radius: root.radius
        nearest: root.nearest
        brightness: root.brightness
        onOwnHeightChanged: root.esde.measured(root.elementId, still.ownWidth, still.ownHeight)
        onStatusChanged: if (still.status === Image.Ready) root.esde.measured(root.elementId, still.ownWidth, still.ownHeight)
    }

    Loader {
        id: player
        objectName: "themeVideo"
        active: root.started
        sourceComponent: Item {
            id: shown
            objectName: "themeVideoPlayback"
            readonly property bool software: GraphicsInfo.api === GraphicsInfo.Software
            // The video's own resolution once its file is read; 0 until then.
            property size own: Qt.size(0, 0)
            readonly property real ownW: shown.own.width
            readonly property real ownH: shown.own.height
            readonly property bool known: shown.ownW > 0 && shown.ownH > 0
            // The video's drawn size by THEMES.md's sizing: stretched, one axis by its aspect, fitted, covered, or its own.
            readonly property real k: root.videoFit === 3 ? Math.min(root.targetWidth / Math.max(1, shown.ownW), root.targetHeight / Math.max(1, shown.ownH)) : 1
            readonly property real drawnW: root.videoFit === 0 || root.videoFit === 1 || root.videoFit === 4 ? root.targetWidth : root.videoFit === 2 ? root.targetHeight * shown.ownW / Math.max(1, shown.ownH) : root.videoFit === 3 ? shown.ownW * shown.k : shown.ownW
            readonly property real drawnH: root.videoFit === 0 || root.videoFit === 2 || root.videoFit === 4 ? root.targetHeight : root.videoFit === 1 ? root.targetWidth * shown.ownH / Math.max(1, shown.ownW) : root.videoFit === 3 ? shown.ownH * shown.k : shown.ownH
            // The black frame by ES-DE's rule (VideoFFmpegComponent::calculateBlackFrame, ES-DE 3.4.1): the whole area on an
            // axis where the video falls short of the threshold, else the video's own extent, and the video moved with it off centre.
            readonly property bool framed: root.pillarboxes && root.areaWidth > 0 && root.areaHeight > 0
            readonly property bool landscape: shown.drawnW > shown.drawnH
            readonly property real frameW: shown.framed && (shown.landscape ? shown.drawnW < root.areaWidth : shown.drawnW <= root.areaWidth) && shown.drawnW / root.areaWidth < root.thresholdX ? root.areaWidth : shown.drawnW
            readonly property real frameH: shown.framed && shown.landscape && shown.drawnH < root.areaHeight && shown.drawnH / root.areaHeight < root.thresholdY ? root.areaHeight : shown.drawnH
            readonly property bool centred: root.placeX === 0.5 && root.placeY === 0.5
            readonly property real offX: shown.framed && !shown.centred && shown.frameW > shown.drawnW ? -(shown.frameW - shown.drawnW) * (root.placeX - 0.5) : 0
            readonly property real offY: shown.framed && !shown.centred && !(shown.frameW > shown.drawnW) && shown.frameH > shown.drawnH ? -(shown.frameH - shown.drawnH) * (root.placeY - 0.5) : 0
            // Its fade, restarted at each play from its first frame.
            property real lit: 0
            property bool lighting: false
            readonly property real position: media.position
            readonly property bool heard: !sound.muted
            readonly property real level: sound.volume
            readonly property int drawnFrames: frames.frames
            x: root.anchorX - root.placeX * shown.drawnW + shown.offX
            y: root.anchorY - root.placeY * shown.drawnH + shown.offY
            width: shown.drawnW
            height: shown.drawnH
            visible: shown.known
            Rectangle {
                objectName: "themeVideoFrame"
                visible: root.blackFrame
                x: -(shown.frameW - shown.drawnW) / 2
                y: -(shown.frameH - shown.drawnH) / 2
                width: shown.frameW
                height: shown.frameH
                color: Qt.rgba(0, 0, 0, 1)
            }
            // The video at its own resolution, drawn only through the shader below.
            Item {
                id: raw
                width: Math.max(1, shown.ownW)
                height: Math.max(1, shown.ownH)
                visible: false
                VideoOutput {
                    id: output
                    anchors.fill: parent
                    visible: !shown.software
                    fillMode: VideoOutput.Stretch
                }
                Kit.VideoFrames {
                    id: frames
                    anchors.fill: parent
                    stretched: true
                    visible: shown.software
                }
            }
            ShaderEffectSource {
                id: texture
                sourceItem: raw
                hideSource: true
                visible: false
                live: true
                smooth: !root.videoNearest
            }
            ShaderEffect {
                readonly property var source: texture
                readonly property color tint: root.videoTint
                readonly property color tintEnd: root.videoTintEnd
                readonly property real vertical: root.videoGradientVertical ? 1 : 0
                readonly property real saturation: Math.max(0, Math.min(1, root.videoSaturation))
                readonly property real dim: 1
                readonly property real brightness: root.brightness
                // ES-DE leaves the video's corners square when the pillarboxes stand clear of them.
                readonly property real radius: (shown.frameW - shown.drawnW >= 2 * root.videoRadius && shown.frameW > shown.drawnW) || (shown.frameH - shown.drawnH >= 2 * root.videoRadius && shown.frameH > shown.drawnH) ? 0 : root.videoRadius
                readonly property size size: Qt.size(width, height)
                // A covered video's crop, placed by cropPos: 0 the left or top, 1 the right or bottom.
                readonly property real cropW: root.videoFit === 4 ? Math.min(1, (root.targetWidth / Math.max(1, root.targetHeight)) / (shown.ownW / Math.max(1, shown.ownH))) : 1
                readonly property real cropH: root.videoFit === 4 ? Math.min(1, (shown.ownW / Math.max(1, shown.ownH)) / (root.targetWidth / Math.max(1, root.targetHeight))) : 1
                readonly property rect uvRect: Qt.rect((1 - cropW) * root.videoCropX, (1 - cropH) * root.videoCropY, cropW, cropH)
                readonly property size tiles: Qt.size(1, 1)
                readonly property point tileOrigin: Qt.point(0, 0)
                readonly property real tiled: 0
                readonly property real fadeFrom: 1
                readonly property real fadeFalloff: 0
                anchors.fill: parent
                opacity: shown.lit
                fragmentShader: "qrc:/emusen/esde/picture.frag.qsb"
            }
            NumberAnimation {
                id: fadeIn
                target: shown
                property: "lit"
                from: 0
                to: 1
                duration: Math.max(1, root.fade)
            }
            MediaPlayer {
                id: media
                objectName: "themeVideoPlayer"
                autoPlay: true
                source: root.videoSource
                videoOutput: shown.software ? frames.videoSink : output
                audioOutput: AudioOutput {
                    id: sound
                    muted: !root.audio
                    volume: root.volume
                }
                onMetaDataChanged: {
                    const own = media.metaData.value(MediaMetaData.Resolution)
                    if (own && own.width > 0 && own.height > 0)
                        shown.own = own
                }
                onPositionChanged: {
                    if (!shown.lighting && media.position > 0) {
                        shown.lighting = true
                        fadeIn.restart()
                    }
                }
                // Each play to its end counts; the next starts again from black, as ES-DE's restart does.
                onMediaStatusChanged: {
                    if (media.mediaStatus !== MediaPlayer.EndOfMedia)
                        return
                    root.plays += 1
                    shown.lit = 0
                    shown.lighting = false
                    if (!root.done) {
                        media.position = 0
                        media.play()
                    }
                }
            }
        }
    }
}
