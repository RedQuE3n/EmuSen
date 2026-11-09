pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit

// A pad button's glyph: kit-core's marks for the control that gives the press in the pad's family and the style's set, in one colour, the letter in the glyph type centred on its ink or cut out of the mark that holds it, a mark's holes cut out with it (EmuSen_Kit.md §7.12, §7.28).
Item {
    id: root
    property alias family: marks.family
    property alias button: marks.button
    property alias swap: marks.swap
    property alias style: marks.style
    // A second press drawn with `button` as one glyph where the kit has one for the two (a d-pad axis, the shoulders, the triggers); -1 for none (EmuSen_Kit.md §7.54).
    property alias pair: marks.pair
    readonly property alias paired: marks.paired
    property color ink: Kit.Tokens.text
    readonly property alias name: marks.name
    // The glyph's own side over the glyph type's size.
    readonly property real sideShare: 1.4
    implicitWidth: Kit.Tokens.fontGlyph * root.sideShare
    implicitHeight: root.implicitWidth
    Accessible.role: Accessible.Graphic
    Accessible.name: marks.name
    Kit.PadGlyphMarks {
        id: marks
        size: Math.min(root.width, root.height)
    }
    TextMetrics {
        id: inked
        // The letter in proportion to the drawn glyph, read from the marks so no binding reads the item's own size (EmuSen_Kit.md §7.28).
        font.pixelSize: Math.max(1, marks.size * marks.letterScale / root.sideShare)
        font.bold: marks.letterBold
        text: marks.letter
    }
    Repeater {
        model: marks
        Shape {
            id: drawn
            required property var outline
            required property bool filled
            required property real stroke
            required property string cutLetter
            required property var holes
            anchors.fill: parent
            // The software backend paints a Shape wholly past its clip unclipped, so there it is not drawn while wholly past (EmuSen_Kit.md §7.34).
            visible: GraphicsInfo.api !== GraphicsInfo.Software || shown.inside
            preferredRendererType: Shape.CurveRenderer
            Kit.InsideClips {
                id: shown
                x: drawn.boundingRect.x
                y: drawn.boundingRect.y
                width: drawn.boundingRect.width
                height: drawn.boundingRect.height
            }
            ShapePath {
                fillColor: drawn.filled ? root.ink : Qt.alpha(root.ink, 0)
                strokeColor: drawn.stroke > 0 ? root.ink : Qt.alpha(root.ink, 0)
                strokeWidth: drawn.stroke > 0 ? drawn.stroke : -1
                fillRule: ShapePath.OddEvenFill
                capStyle: ShapePath.RoundCap
                joinStyle: ShapePath.RoundJoin
                PathPolyline { path: drawn.outline }
                PathMultiline { paths: drawn.holes }
                PathText {
                    x: (root.width - inked.tightBoundingRect.width) / 2 - inked.tightBoundingRect.x
                    y: (root.height - inked.tightBoundingRect.height) / 2
                    font.pixelSize: inked.font.pixelSize
                    font.bold: inked.font.bold
                    text: drawn.cutLetter
                }
            }
        }
    }
    Shape {
        id: letter
        anchors.fill: parent
        visible: marks.letter.length > 0 && !marks.cut && (GraphicsInfo.api !== GraphicsInfo.Software || letterShown.inside)
        preferredRendererType: Shape.CurveRenderer
        Kit.InsideClips {
            id: letterShown
            x: letter.boundingRect.x
            y: letter.boundingRect.y
            width: letter.boundingRect.width
            height: letter.boundingRect.height
        }
        ShapePath {
            fillColor: root.ink
            strokeColor: Qt.alpha(root.ink, 0)
            PathText {
                x: (root.width - inked.tightBoundingRect.width) / 2 - inked.tightBoundingRect.x
                y: (root.height - inked.tightBoundingRect.height) / 2
                font.pixelSize: inked.font.pixelSize
                font.bold: inked.font.bold
                text: marks.letter
            }
        }
    }
}
