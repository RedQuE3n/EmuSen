pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme
import "../parts"

// Constellation's splash, "The chart draws", 6 s: the stars fade in, the consoles appear as points in maker order on up to three arched rows, a line runs from point to point and each flares into its body with a bell as the line arrives, and the wordmark and the count settle; under reduced motion, the finished chart (constellation.md §5).
ThemeScreen {
    id: screen
    objectName: "splashScreen"
    role: ThemeScreen.Splash
    length: 6000
    still: Tokens.reducedMotion
    NormalizedCanvas {
        id: canvas
        objectName: "splashCanvas"
        Item {
            id: root
            anchors.fill: parent
            // The splash's clock in s; under reduced motion the finished frame for the length.
            property real clock: 0
            readonly property real t: Tokens.reducedMotion ? screen.length / 1000 : root.clock / 1000
            readonly property int n: points.count
            // At most three rows: eight a row at the 144 pitch as the canvas has it, and past 24 consoles the pitch shrinks to fit a third of them a row.
            readonly property int fits: Math.max(1, Math.floor((root.width - 128 + 16) / 144))
            readonly property int rows: root.n > 0 ? Math.min(3, Math.ceil(root.n / root.fits)) : 0
            readonly property int per: root.rows > 0 ? Math.ceil(root.n / root.rows) : 0
            readonly property real pitch: root.per > 0 ? Math.min(144, (root.width - 128) / root.per) : 144
            // A segment of the line takes 0.3 s, less when there are too many to reach by 4.6 s.
            readonly property real segment: Math.min(0.3, 3.3 / Math.max(1, root.n - 1))
            readonly property real words: Math.max(0, Math.min(1, (root.t - 4.6) / 0.8))
            readonly property color ink: Tokens.variant === Tokens.Light ? "#1F2A44" : Qt.hsla(220 / 360, 0.6, 0.8, 1)
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
                resting: false
                starsOpacity: Math.max(0, Math.min(1, root.t))
            }
            Repeater {
                id: points
                model: Library.consoles
                onCountChanged: Bells.ladder(points.count)
                Item {
                    id: point
                    required property int index
                    required property string body
                    required property int group
                    required property int groupIndex
                    required property int groupStart
                    // The place in maker order, its row and its place in the row, and the previous point's, on the row's arch.
                    readonly property int m: point.groupStart + point.groupIndex
                    readonly property int row: Math.floor(point.m / root.per)
                    readonly property int k: point.m - point.row * root.per
                    readonly property int rowLength: Math.min(root.per, root.n - point.row * root.per)
                    readonly property real u: point.rowLength > 1 ? 2 * point.k / (point.rowLength - 1) - 1 : 0
                    readonly property real rowY: root.rows <= 1 ? 400 : (root.rows === 2 ? 330 + 140 * point.row : 290 + 110 * point.row)
                    readonly property real cx: root.width / 2 - (point.rowLength - 1) * root.pitch / 2 + point.k * root.pitch
                    readonly property real cy: point.rowY - 28 * point.u * point.u
                    readonly property int pm: point.m - 1
                    readonly property int prow: Math.floor(point.pm / root.per)
                    readonly property int pk: point.pm - point.prow * root.per
                    readonly property int prowLength: Math.min(root.per, root.n - point.prow * root.per)
                    readonly property real pu: point.prowLength > 1 ? 2 * point.pk / (point.prowLength - 1) - 1 : 0
                    readonly property real px: root.width / 2 - (point.prowLength - 1) * root.pitch / 2 + point.pk * root.pitch
                    readonly property real py: (root.rows <= 1 ? 400 : (root.rows === 2 ? 330 + 140 * point.prow : 290 + 110 * point.prow)) - 28 * point.pu * point.pu
                    readonly property real arrive: 1.2 + root.segment * point.m
                    readonly property real reach: Math.max(0, Math.min(1, (root.t - point.arrive + root.segment) / root.segment))
                    readonly property real shown: Math.max(0, Math.min(1, (root.t - 0.8) / 0.4))
                    readonly property real flare: Math.max(0, Math.min(1, (root.t - point.arrive) / 0.4))
                    readonly property real size: 48 * (1 - Math.cos(point.flare * Math.PI * 2.4) * Math.exp(-5 * point.flare))
                    readonly property bool arrived: root.t >= point.arrive
                    onArrivedChanged: if (point.arrived && !Tokens.reducedMotion) Bells.strike(point.m)
                    ChartLine {
                        objectName: "splashLine." + point.m
                        visible: point.m > 0 && point.reach > 0
                        x1: point.px
                        y1: point.py
                        x2: point.px + (point.cx - point.px) * point.reach
                        y2: point.py + (point.cy - point.py) * point.reach
                        dashed: point.groupIndex === 0
                        ink: root.ink
                        opacity: 0.35
                    }
                    Rectangle {
                        objectName: "splashPoint." + point.m
                        visible: point.shown > 0 && point.flare < 1
                        x: point.cx - 2
                        y: point.cy - 2
                        width: 4
                        height: 4
                        radius: 2
                        color: Tokens.textStrong
                        opacity: point.shown * (1 - point.flare)
                    }
                    Body {
                        objectName: "splashBody." + point.m
                        visible: point.flare > 0
                        x: point.cx - width / 2
                        y: point.cy - height / 2
                        width: Math.max(1, point.size)
                        height: Math.max(1, point.size)
                        world: point.body
                    }
                }
            }
            Text {
                objectName: "splashWordmark"
                y: 220 - 88 * 0.78
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
                y: 600
                width: root.width
                leftPadding: 6
                horizontalAlignment: Text.AlignHCenter
                opacity: root.words
                text: Words.splashCaption(root.n)
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
