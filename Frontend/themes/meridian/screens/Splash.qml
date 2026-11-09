pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme
import "../parts"

// Meridian's splash, "The rail fills", 6 s: the meridian draws down the rail's centre, the glass widens about it, the worlds drop into the rail with a bell each, and the wordmark settles into the band; the last frame is Home's composition (meridian.md §5).
ThemeScreen {
    id: screen
    objectName: "splashScreen"
    role: ThemeScreen.Splash
    length: 6000
    still: Tokens.reducedMotion
    // A console's world in the rail, at its place in maker order.
    component World: Item {
        required property string body
        required property real hue
        required property int groupStart
        required property int groupIndex
    }
    NormalizedCanvas {
        id: canvas
        objectName: "splashCanvas"
        Item {
            id: root
            // First, so the parts read the faces from their first binding.
            Faces {
                id: faces
            }
            anchors.fill: parent
            // The splash's clock in ms; under reduced motion the finished frame for the length.
            property real clock: 0
            readonly property real t: Tokens.reducedMotion ? screen.length : root.clock
            readonly property int n: worlds.count
            // One column of up to eleven at the rail's pitch, then two, then three 80 apart about the meridian; past thirty-three the pitch shrinks and the worlds become points.
            readonly property int cols: Math.max(1, Math.min(3, Math.ceil(root.n / 11)))
            readonly property int rows: Math.max(1, Math.ceil(root.n / root.cols))
            readonly property real pitch: Math.min(54, 598 / root.rows)
            readonly property real size: Math.max(6, Math.min(40, root.pitch - 8))
            // The worlds land 250 ms apart, closer when there are too many to land by 4.6 s.
            readonly property real gap: Math.min(250, 2300 / Math.max(1, root.n))
            readonly property real line: Math.max(0, Math.min(1, (root.t - 600) / 600))
            readonly property real glassOpen: 1 - Math.pow(1 - Math.max(0, Math.min(1, (root.t - 1200) / 600)), 2)
            readonly property real words: Math.max(0, Math.min(1, (root.t - 4600) / 800))
            NumberAnimation on clock {
                from: 0
                to: screen.length
                duration: screen.length
                running: !Tokens.reducedMotion
                onFinished: screen.finished()
            }
            Backdrop {
                objectName: "splashSky"
                anchors.fill: parent
                starsOpacity: Math.max(0, Math.min(1, root.t / 1000))
                moonOpacity: Math.max(0, Math.min(1, root.t / 1000))
            }
            Glass {
                objectName: "splashRail"
                visible: root.glassOpen > 0
                x: 204 - 140 * root.glassOpen
                y: 96
                width: 280 * root.glassOpen
                height: 630
            }
            Scene {
                objectName: "meridian"
                x: 203.5
                y: 96
                width: 1
                height: 630 * root.line
                Rectangle {
                    anchors.fill: parent
                    color: "#8C96DC"
                    opacity: 0.45
                }
            }
            Item {
                objectName: "splashWorlds"
                x: 64
                y: 96
                width: 280
                height: 630
                clip: true
                Repeater {
                    id: worlds
                    model: Library.consoles
                    onCountChanged: Bells.ladder(worlds.count)
                    World {
                        id: world
                        readonly property int place: world.groupStart + world.groupIndex
                        readonly property real start: 1800 + root.gap * world.place
                        readonly property real v: Math.max(0, Math.min(1, (root.t - world.start) / 500))
                        readonly property real spring: 1 - Math.cos(world.v * Math.PI * 2.4) * Math.exp(-5 * world.v)
                        readonly property real restY: 16 + root.pitch * Math.floor(world.place / root.cols) + root.pitch / 2
                        readonly property bool landed: world.v >= 1
                        objectName: "splashWorld." + world.place
                        visible: world.v > 0
                        x: 140 + (root.cols === 1 ? 0 : (root.cols === 2 ? (world.place % 2) * 80 - 40 : (world.place % 3) * 80 - 80)) - root.size / 2
                        y: -root.size + (world.restY + root.size) * world.spring - root.size / 2
                        width: root.size
                        height: root.size
                        onLandedChanged: if (world.landed && !Tokens.reducedMotion) Bells.strike(world.place)
                        Body {
                            anchors.fill: parent
                            world: world.body
                            hue: world.hue
                            focused: world.v > 0.3 && world.v < 1
                            opacity: Math.min(1, world.v * 3)
                        }
                    }
                }
            }
            Text {
                objectName: "splashWordmark"
                x: 376
                // Baseline 400: Barlow Condensed in a fixed 88-unit line puts its baseline 76 below the line's top.
                y: 400 - 76 + 12 * (1 - root.words)
                width: root.width - 440
                horizontalAlignment: Text.AlignHCenter
                opacity: root.words
                text: Words.title
                color: Tokens.textStrong
                leftPadding: 24
                font.family: faces.light
                font.weight: Font.Light
                font.pixelSize: 88
                font.letterSpacing: 24
                lineHeight: 88
                lineHeightMode: Text.FixedHeight
            }
            Text {
                objectName: "splashCaption"
                x: 376
                y: 440
                width: root.width - 440
                horizontalAlignment: Text.AlignHCenter
                opacity: root.words
                text: Words.splashCaption(root.n)
                color: Tokens.muted
                leftPadding: 6
                font.family: faces.medium
                font.pixelSize: 19
                font.letterSpacing: 6
                lineHeight: 22.8
                lineHeightMode: Text.FixedHeight
            }
        }
    }
}
