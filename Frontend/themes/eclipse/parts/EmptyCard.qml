import QtQuick
import QtQuick.Shapes
import EmuSen.Theme

// The empty slot's card: a dashed line round the picture's place, a plus at its middle and its words under it; shown, never pressed, as no game runs to save (EclipseGame:78-83, EclipseGameNoState:64-68; R-20261116-36).
Item {
    id: root
    // A board pixel in canvas units, Board.unit.
    property real unit: 1
    required property string label
    // The dashed line's colour and the plus's and the words'.
    property color line: Tokens.border
    property color ink: Tokens.text
    // The board's measures: width, the box's height, corner and line, the plus's side and stroke, the gap under the box, the words' size.
    property real cardWidth: 316
    property real artHeight: 198
    property real corner: 18
    property real lineWidth: 2
    property real plusSide: 44
    property real plusStroke: 1.8
    property real wordsGap: 14
    property real nameSize: 22
    width: root.cardWidth * root.unit
    height: nameItem.y + nameItem.height
    Accessible.name: root.label
    // The border drawn inside the box, dashed as Chrome dashed the board's 2 px border, 6 on and 4 off (measured 2026-11-23): Eclipse's Tokens.dashPattern.
    Shape {
        id: box
        objectName: "emptyBox"
        width: root.width
        height: root.artHeight * root.unit
        preferredRendererType: Shape.CurveRenderer
        readonly property real half: root.lineWidth * root.unit / 2
        ShapePath {
            strokeColor: root.line
            strokeWidth: root.lineWidth * root.unit
            strokeStyle: ShapePath.DashLine
            dashPattern: Tokens.dashPattern
            fillColor: "transparent"
            PathRectangle {
                x: box.half
                y: box.half
                width: box.width - 2 * box.half
                height: box.height - 2 * box.half
                radius: root.corner * root.unit - box.half
            }
        }
    }
    // The plus, two strokes of the board's 24-unit icon from 5 to 19, round-capped (EclipseGame:80).
    Shape {
        id: plus
        objectName: "emptyPlus"
        x: (box.width - width) / 2
        y: (box.height - height) / 2
        width: root.plusSide * root.unit
        height: width
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            scale: Qt.size(plus.width / 24, plus.height / 24)
            strokeColor: root.ink
            strokeWidth: root.plusStroke * plus.width / 24
            capStyle: ShapePath.RoundCap
            fillColor: "transparent"
            startX: 12
            startY: 5
            PathLine { x: 12; y: 19 }
            PathMove { x: 5; y: 12 }
            PathLine { x: 19; y: 12 }
        }
    }
    Line {
        id: nameItem
        objectName: "emptyName"
        unit: root.unit
        y: box.height + root.wordsGap * root.unit
        text: root.label
        boardLine: true
        color: root.ink
        size: root.nameSize
        weight: Font.DemiBold
    }
}
