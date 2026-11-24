import QtQuick
import QtQuick.Shapes
import EmuSen.Theme

// An outlined pill with no pad glyph: its words, and the heart before them where it is Favourite, the heart and the line in the rose while on (EclipseGame:48-50, EclipseGameNoState:48-49); every measure in board pixels.
Walkable {
    id: root
    // A board pixel in canvas units, Board.unit.
    property real unit: 1
    required property string label
    property bool heart: false
    property bool on: false
    // The line at rest and while on, and the words' and the heart's colour at rest.
    property color outline: Tokens.border
    property color onColour: Tokens.accent
    property color ink: Tokens.text
    // Scene pixels to a canvas unit, for the focus ring's glow.
    property real pixelScale: 1
    // The pill's place in its row, which names its parts.
    property int place
    // The board's measures: height, line, padding either side, the heart's side and the gap after it, the words' size.
    property real tall: 72
    property real lineWidth: 2
    property real lead: root.heart ? 22 : 30
    property real trail: root.heart ? 28 : 30
    property real heartSide: 26
    property real gap: 12
    property real labelSize: 24
    readonly property color paint: root.on ? root.onColour : root.outline
    readonly property Item words: wordsItem
    width: wordsItem.x + wordsItem.implicitWidth + (root.trail + root.lineWidth) * root.unit
    height: root.tall * root.unit
    Accessible.name: root.label
    Accessible.checkable: root.heart
    Accessible.checked: root.on
    Rectangle {
        objectName: "pillLine." + root.place
        anchors.fill: parent
        radius: height / 2
        color: "transparent"
        border.color: root.paint
        border.width: root.lineWidth * root.unit
    }
    FocusRing {
        anchors.fill: parent
        radius: height / 2
        pixelScale: root.pixelScale
        visible: root.activeFocus
    }
    // The board's heart, its 24-unit path at 26 px, stroked 1.6 and filled while on (EclipseGame:50).
    Shape {
        id: heartItem
        objectName: "pillHeart." + root.place
        visible: root.heart
        x: (root.lineWidth + root.lead) * root.unit
        y: (root.height - height) / 2
        width: root.heartSide * root.unit
        height: width
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            scale: Qt.size(heartItem.width / 24, heartItem.height / 24)
            strokeWidth: 1.6 * heartItem.width / 24
            strokeColor: root.on ? root.onColour : root.ink
            fillColor: root.on ? root.onColour : "transparent"
            joinStyle: ShapePath.RoundJoin
            startX: 12
            startY: 20.5
            PathCubic { control1X: 5; control1Y: 15.5; control2X: 3; control2Y: 12; x: 3; y: 8.8 }
            PathArc { x: 12; y: 6.6; radiusX: 4.8; radiusY: 4.8 }
            PathArc { x: 21; y: 8.8; radiusX: 4.8; radiusY: 4.8 }
            PathCubic { control1X: 21; control1Y: 12; control2X: 19; control2Y: 15.5; x: 12; y: 20.5 }
        }
    }
    Line {
        id: wordsItem
        objectName: "pillLabel." + root.place
        unit: root.unit
        x: root.heart ? heartItem.x + heartItem.width + root.gap * root.unit : (root.lineWidth + root.lead) * root.unit
        y: (root.height - height) / 2
        text: root.label
        color: root.ink
        size: root.labelSize
    }
}
