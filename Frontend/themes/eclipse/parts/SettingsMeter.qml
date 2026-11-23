pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Theme

// Sound's ring: a 340 px disc filled clockwise from 225° over 270° to the row's share, its 14 px rim left, the value and the caption inside (G:166-171; EmuSen_Qt.md §3.4.11).
Item {
    id: root
    // A board pixel in canvas units, Board.unit.
    required property real unit
    // Scene pixels to a canvas unit, so the ring's glow blurs at the pixels drawn.
    property real pixelScale: 1
    // The share filled, Settings.meter, and the words inside.
    property real share: 0
    property string value
    property string caption
    readonly property real side: 340
    readonly property real rim: 14
    objectName: "asideMeter"
    width: parent ? parent.width : 0
    height: root.side * root.unit
    SettingsInk {
        id: own
    }
    Item {
        id: disc
        objectName: "asideRing"
        x: (root.width - width) / 2
        width: root.side * root.unit
        height: width
        Glow {
            objectName: "asideRingGlow"
            anchors.fill: parent
            radius: width / 2
            color: own.ringGlow
            blur: 70 * root.unit
            spread: 12 * root.unit
            pixelScale: root.pixelScale
        }
        // The conic fill drawn at the board's pixels and scaled; mirrored, since Qt's conical gradient turns counter-clockwise (EmuSen_Qt.md §3.4.11).
        Shape {
            id: fill
            objectName: "asideRingFill"
            width: root.side
            height: root.side
            scale: root.unit
            transformOrigin: Item.TopLeft
            preferredRendererType: Shape.CurveRenderer
            transform: Scale { xScale: -1; origin.x: root.side / 2 }
            readonly property real end: 0.75 * Math.max(0, Math.min(1, root.share))
            // The stops in order at every share: the track starts a hair after the fill and ends at 270°, or a hair after a full fill.
            readonly property real trackStart: Math.max(0.0001, fill.end) + 0.0001
            readonly property real trackEnd: Math.max(0.75, fill.trackStart)
            ShapePath {
                strokeColor: "transparent"
                fillGradient: ConicalGradient {
                    centerX: root.side / 2
                    centerY: root.side / 2
                    angle: 315
                    // A hard stop is two stops a hair apart: QGradient keeps one stop at a position.
                    GradientStop { position: 0; color: Tokens.accent }
                    GradientStop { position: Math.max(0.0001, fill.end); color: own.ringEnd }
                    GradientStop { position: fill.trackStart; color: own.ringTrack }
                    GradientStop { position: fill.trackEnd; color: own.ringTrack }
                    GradientStop { position: fill.trackEnd + 0.0001; color: "transparent" }
                    GradientStop { position: 1; color: "transparent" }
                }
                PathAngleArc { centerX: root.side / 2; centerY: root.side / 2; radiusX: root.side / 2; radiusY: root.side / 2; startAngle: 0; sweepAngle: 360 }
            }
        }
        Rectangle {
            objectName: "asideRingInside"
            x: root.rim * root.unit
            y: x
            width: parent.width - 2 * x
            height: width
            radius: width / 2
            color: Tokens.surface
            // The value over the caption, 4 apart, the column centred in the inside (G:167); placed by its own sums, as centring anchors round to whole pixels.
            Column {
                id: words
                x: (parent.width - width) / 2
                y: (parent.height - height) / 2
                spacing: 4 * root.unit
                Line {
                    objectName: "asideMeterValue"
                    x: (words.width - width) / 2
                    boardLine: true
                    unit: root.unit
                    text: root.value
                    color: Tokens.text
                    size: 84
                    spacing: -0.02 * 84
                    weight: Tokens.weightTitle
                }
                Line {
                    objectName: "asideMeterCaption"
                    x: (words.width - width) / 2
                    boardLine: true
                    unit: root.unit
                    visible: text.length > 0
                    text: root.caption
                    color: Tokens.muted
                    size: 20
                }
            }
        }
    }
}
