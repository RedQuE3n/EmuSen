pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Theme
import "../parts"

// Orrery's splash, "The orbits", 7 s: the stars, the moon, one orbit a maker inking itself about the moon (the fewest innermost), each world sliding from the moon to its place with a bell, then the wordmark (orrery.md §5).
ThemeScreen {
    id: screen
    objectName: "splashScreen"
    role: ThemeScreen.Splash
    length: 7000
    still: Tokens.reducedMotion
    // A maker's orbit, drawn by its first console's row: the radii of its rank among the makers.
    component Ring: Scene {
        id: ring
        required property int group
        required property int groupIndex
        readonly property int rank: layout.ranks[ring.group] ?? 0
        readonly property real rx: 208 + root.ringStep * ring.rank
        readonly property real ink: Math.max(0, Math.min(1, (root.t - 1600 - 300 * ring.rank) / 1200))
        visible: ring.groupIndex === 0 && ring.ink > 0
        anchors.fill: parent
        Shape {
            anchors.fill: parent
            preferredRendererType: Shape.CurveRenderer
            ShapePath {
                strokeColor: root.lineInk
                strokeWidth: 1
                fillColor: "transparent"
                PathAngleArc {
                    centerX: root.width / 2
                    centerY: 160
                    radiusX: ring.rx
                    radiusY: ring.rx * 0.846
                    startAngle: 160
                    sweepAngle: -140 * ring.ink
                }
            }
        }
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
            readonly property bool light: Tokens.variant === Tokens.Light
            readonly property color lineInk: root.light ? Qt.rgba(74 / 255, 58 / 255, 40 / 255, 0.5) : Qt.rgba(140 / 255, 150 / 255, 220 / 255, 0.32)
            // Three orbits 144 apart from 208; more makers share the span from 208 to 496.
            readonly property real ringStep: layout.groups <= 3 ? 144 : 288 / Math.max(1, layout.groups - 1)
            // The worlds set out 300 ms apart, closer when there are too many to settle by 6 s.
            readonly property real gap: Math.min(300, 2900 / Math.max(1, root.n))
            readonly property real moon: Math.max(0, Math.min(1, (root.t - 800) / 1000))
            readonly property real words: Math.max(0, Math.min(1, (root.t - 6000) / 800))
            WheelLayout {
                id: layout
                width: root.width
                model: Library.consoles
                role: "group"
            }
            Connections {
                target: Library.consoles
                function onModelReset() { layout.read() }
            }
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
            }
            Moon {
                objectName: "splashMoon"
                x: root.width / 2 - 32
                y: 160 - 32
                width: 26
                height: 26
                opacity: root.moon
                scale: 0.6 + 0.4 * root.moon
            }
            Item {
                objectName: "splashOrbits"
                anchors.fill: parent
                Repeater {
                    model: Library.consoles
                    Ring {}
                }
            }
            Repeater {
                id: worlds
                model: Library.consoles
                onCountChanged: Bells.ladder(worlds.count)
                Item {
                    id: world
                    required property int index
                    required property string body
                    required property real hue
                    required property int group
                    required property int groupIndex
                    required property int groupSize
                    readonly property real rx: 208 + root.ringStep * (layout.ranks[world.group] ?? 0)
                    // Worlds 130 apart along the orbit about its lowest point, closer when a maker's span would pass 140°.
                    readonly property real spread: Math.min(130 / world.rx * 180 / Math.PI, 140 / Math.max(1, world.groupSize))
                    readonly property real theta: (90 + ((world.groupSize - 1) / 2 - world.groupIndex) * world.spread) * Math.PI / 180
                    readonly property real start: 2600 + root.gap * world.index
                    readonly property real v: Math.max(0, Math.min(1, (root.t - world.start) / 500))
                    readonly property real spring: 1 - Math.cos(world.v * Math.PI * 2.4) * Math.exp(-5 * world.v)
                    readonly property real size: root.n > 33 ? 24 : 48
                    readonly property bool settled: world.v >= 1
                    objectName: "splashWorld." + world.index
                    visible: world.v > 0
                    x: root.width / 2 + world.rx * Math.cos(world.theta) * world.spring - world.size / 2
                    y: 160 + (world.rx * 0.846 * Math.sin(world.theta)) * world.spring - world.size / 2
                    width: world.size
                    height: world.size
                    onSettledChanged: if (world.settled && !Tokens.reducedMotion) Bells.strike(world.index)
                    Body {
                        anchors.fill: parent
                        world: world.body
                        hue: world.hue
                        focused: world.v > 0 && world.v < 1
                        opacity: Math.min(1, world.v * 3)
                    }
                }
            }
            Text {
                objectName: "splashWordmark"
                x: 0
                // Baseline 690: Barlow Condensed in a fixed 72-unit line puts its baseline 62 below the line's top.
                y: 690 - 62 + 12 * (1 - root.words)
                width: root.width
                horizontalAlignment: Text.AlignHCenter
                opacity: root.words
                text: Words.title
                color: Tokens.textStrong
                leftPadding: 18
                font.family: faces.light
                font.weight: Font.Light
                font.pixelSize: 72
                font.letterSpacing: 18
                lineHeight: 72
                lineHeightMode: Text.FixedHeight
            }
            Text {
                objectName: "splashCaption"
                x: 0
                y: 716
                width: root.width
                horizontalAlignment: Text.AlignHCenter
                opacity: root.words
                text: Words.splashCaption(root.n)
                color: Tokens.muted
                leftPadding: 6
                font.family: faces.medium
                font.pixelSize: 19
                font.letterSpacing: 6
                font.capitalization: Font.AllUppercase
                lineHeight: 22.8
                lineHeightMode: Text.FixedHeight
            }
        }
    }
}
