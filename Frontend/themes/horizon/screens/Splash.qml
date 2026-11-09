pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme
import "../parts"

// Horizon's splash, Horizon rise, six seconds: the stars fade in, the horizon draws across, each console's body rises over it and rings its bell, then EMUSEN settles with its caption; under reduced motion the finished frame (EmuSen_Qt.md §3.4.2).
ThemeScreen {
    id: screen
    objectName: "horizonSplash"
    role: ThemeScreen.Splash
    length: 6000
    still: Tokens.reducedMotion
    NormalizedCanvas {
        id: canvas
        objectName: "splashCanvas"
        Item {
            id: root
            anchors.fill: parent
            // The splash's clock in milliseconds; the finished frame under reduced motion.
            property real time: screen.still ? 6000 : 0
            readonly property int bodies: Library.consoles.count
            readonly property real pitch: Math.min(96, root.bodies > 1 ? (root.width - 128) / root.bodies : 96)
            readonly property real size: Math.min(64, root.pitch - 16)
            readonly property real rowStart: root.width / 2 - (root.bodies * root.pitch - (root.pitch - root.size)) / 2
            readonly property real line: Math.max(0, Math.min(1, (root.time - 600) / 1200))
            readonly property real words: Math.max(0, Math.min(1, (root.time - 4600) / 800))
            NumberAnimation on time {
                running: !screen.still && root.visible
                from: 0
                to: 6000
                duration: 6000
                onStarted: Bells.ladder(root.bodies)
                onFinished: screen.finished()
            }
            Backdrop {
                anchors.fill: parent
                starsEnd: 800
                starsOpacity: Math.max(0, Math.min(1, root.time / 1000))
            }
            Rectangle {
                objectName: "splashLine"
                y: 480
                width: root.width * root.line
                height: 1
                gradient: Gradient {
                    orientation: Gradient.Horizontal
                    GradientStop { position: 0; color: "transparent" }
                    GradientStop { position: 0.12; color: Tokens.variant === Tokens.Dark ? "#528C96DC" : "#471F2A44" }
                    GradientStop { position: 0.88; color: Tokens.variant === Tokens.Dark ? "#528C96DC" : "#471F2A44" }
                    GradientStop { position: 1; color: "transparent" }
                }
            }
            // The bodies rise inside a box that ends at the line, so what is still below it is not drawn.
            Item {
                objectName: "splashRise"
                width: root.width
                height: 480
                clip: true
                layer.enabled: GraphicsInfo.api === GraphicsInfo.Software
                Repeater {
                    model: Library.consoles
                    Item {
                        id: rising
                        required property int index
                        required property string body
                        readonly property real start: 1800 + 280 * rising.index
                        readonly property real v: Math.max(0, Math.min(1, (root.time - rising.start) / 500))
                        // The kit's spring: overshoots the rest and settles within the half second.
                        readonly property real spring: 1 - Math.cos(rising.v * Math.PI * 2.4) * Math.exp(-5 * rising.v)
                        readonly property bool struck: root.time >= rising.start + 250
                        objectName: "splashBody." + rising.index
                        visible: rising.v > 0
                        x: root.rowStart + rising.index * root.pitch
                        y: 540 + (420 - 540) * rising.spring - root.size / 2
                        width: root.size
                        height: root.size
                        opacity: Math.min(1, rising.v * 3)
                        onStruckChanged: if (rising.struck && !screen.still) Bells.strike(rising.index)
                        Body {
                            id: shape
                            world: rising.body
                            focused: rising.v > 0.4 && rising.v < 1
                            height: root.size
                            width: root.size * shape.tileWidth / Math.max(1, shape.tileHeight)
                            x: (root.size - width) / 2
                        }
                    }
                }
            }
            Text {
                objectName: "splashWordmark"
                y: 330 - 88 * 0.78 + (1 - (1 - (1 - root.words) * (1 - root.words))) * 12
                width: root.width
                leftPadding: 24
                horizontalAlignment: Text.AlignHCenter
                opacity: root.words
                text: Words.title
                color: Tokens.textStrong
                font.family: faces.light
                font.weight: Font.Light
                font.pixelSize: 88
                font.letterSpacing: 24
                lineHeight: 88
                lineHeightMode: Text.FixedHeight
            }
            Text {
                objectName: "splashCaption"
                y: 520
                width: root.width
                leftPadding: 6
                horizontalAlignment: Text.AlignHCenter
                opacity: root.words
                text: Words.splashCaption(root.bodies)
                color: Tokens.muted
                font.family: faces.medium
                font.weight: Font.Medium
                font.pixelSize: 19
                font.letterSpacing: 6
                lineHeight: 22.8
                lineHeightMode: Text.FixedHeight
            }
            Faces {
                id: faces
            }
        }
    }
}
