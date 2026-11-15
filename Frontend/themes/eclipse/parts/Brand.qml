import QtQuick
import QtQuick.Shapes
import EmuSen.Theme

// The header's mark and, where a board writes it, the program's name after it, both centred on the item's middle line (Main:30-31, EclipseConsoles:25-26, EclipseEmpty:23-24); its left edge and middle are the screen's (EmuSen_Qt.md §3.4.8).
Item {
    id: root
    // A board pixel in canvas units, Board.unit.
    required property real unit
    // Whether the name follows the mark; the games board puts its path there instead (EclipseLibrary:25).
    property bool named: true
    // The mark's side and the gap after it, in board pixels.
    property real side: 34
    property real gap: 14
    property real nameSize: 24
    width: root.named ? wordmark.x + wordmark.width : mark.width
    height: Math.max(mark.height, root.named ? wordmark.height : 0)
    Item {
        id: mark
        objectName: "mark"
        // The colour the crescent is filled in.
        readonly property color ink: Tokens.accent
        y: (root.height - height) / 2
        width: root.side * root.unit
        height: width
        Accessible.role: Accessible.Graphic
        Accessible.name: Words.name
        Shape {
            id: shape
            // Marks' paths are drawn in a 24-unit box (qt-models/src/marks.rs).
            readonly property real box: 24
            width: shape.box
            height: shape.box
            scale: parent.width / shape.box
            transformOrigin: Item.TopLeft
            preferredRendererType: Shape.CurveRenderer
            ShapePath {
                fillColor: mark.ink
                strokeColor: "transparent"
                fillRule: ShapePath.OddEvenFill
                PathSvg { path: Marks.crescent }
            }
        }
    }
    Text {
        id: wordmark
        objectName: "wordmark"
        x: mark.width + root.gap * root.unit
        y: (root.height - height) / 2
        visible: root.named
        text: Words.name
        color: Tokens.text
        font.pixelSize: Type.sizeIn(root.nameSize * root.unit, Tokens.textScale, Tokens.faceText, Tokens.weightWordmark)
        font.family: Tokens.faceText
        font.weight: Tokens.weightWordmark
        font.letterSpacing: 0.02 * root.nameSize * root.unit
    }
}
