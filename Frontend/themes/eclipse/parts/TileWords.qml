import QtQuick
import EmuSen.Theme

// A tile's foot as the board draws it (Main:84-86): a shade the board's 96 px tall from rgba(4, 5, 10, 0.92) at the foot to
// clear, the console line 12 px above the tile's foot and the title on it, each inset 18 px.
Item {
    id: root
    // A board pixel in canvas units, Board.unit.
    required property real unit
    required property string title
    required property string facts
    // The tile's place in its row, which names its parts for a reader and the tests.
    property int place
    // The title's lines and their height; a tile with no facts stands its title on the foot.
    property int titleLines: 1
    property real titleLineHeight: 1
    // The shade's corners, the tile's where nothing clips it.
    property real radius: 0
    property real titleSize: 21
    property real factsSize: 15
    property real inset: 18
    property real foot: 12
    property real fade: 96
    property color titleInk: "transparent"
    property color factsInk: "transparent"
    property color shadeInk: "transparent"
    property color shadeClear: "transparent"
    anchors.fill: parent
    Rectangle {
        id: shade
        objectName: "tileShade." + root.place
        // The shade's dark colour, which the test reads.
        readonly property color paint: root.shadeInk
        width: root.width
        height: root.fade * root.unit
        y: root.height - height
        radius: root.radius * root.unit
        gradient: Gradient {
            GradientStop { position: 0; color: root.shadeClear }
            GradientStop { position: 1; color: shade.paint }
        }
    }
    Text {
        id: titleItem
        objectName: "tileTitle." + root.place
        x: root.inset * root.unit
        // On the console line: the board's 34 px foot would overlap the line raised to the legibility floor (eclipse_home.tsv).
        y: (factsItem.visible ? factsItem.y : root.height - root.foot * root.unit) - height
        width: root.width - 2 * root.inset * root.unit
        text: root.title
        wrapMode: root.titleLines > 1 ? Text.Wrap : Text.NoWrap
        maximumLineCount: root.titleLines
        lineHeight: root.titleLineHeight
        elide: Text.ElideRight
        color: root.titleInk
        font.pixelSize: Type.sizeIn(root.titleSize * root.unit, Tokens.textScale, Tokens.faceText, Font.DemiBold)
        font.family: Tokens.faceText
        font.weight: Font.DemiBold
    }
    Text {
        id: factsItem
        objectName: "tileFacts." + root.place
        x: root.inset * root.unit
        y: root.height - root.foot * root.unit - height
        visible: root.facts.length > 0
        width: root.width - 2 * root.inset * root.unit
        text: root.facts
        elide: Text.ElideRight
        color: root.factsInk
        font.pixelSize: Type.sizeIn(root.factsSize * root.unit, Tokens.textScale, Tokens.faceText, Font.Normal)
        font.family: Tokens.faceText
        font.letterSpacing: 0.08 * root.factsSize * root.unit
        font.capitalization: Font.AllUppercase
    }
}
