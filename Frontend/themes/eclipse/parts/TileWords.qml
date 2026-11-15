import QtQuick
import EmuSen.Theme

// A tile's foot: its title standing on its facts line over the tile's art, and the shade under them (Main:84-86). The facts sit
// the board's 12 px above the foot; the shade fades in over the board's 96 px and holds the board's 0.92 from the title's top to
// the foot, so words raised to the legibility floor keep the look's 4.5:1 over any art (provisional, the tester's to judge).
Item {
    id: root
    // A board pixel in canvas units, Board.unit.
    required property real unit
    required property string title
    required property string facts
    // The tile's place in its row, which names its parts for a reader and the tests.
    property int place
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
        readonly property real solid: root.height - titleItem.y
        width: root.width
        height: Math.max(root.fade * root.unit, shade.solid + root.fade * root.unit / 2)
        y: root.height - height
        gradient: Gradient {
            GradientStop { position: 0; color: root.shadeClear }
            GradientStop { position: Math.max(0, 1 - shade.solid / shade.height); color: shade.paint }
            GradientStop { position: 1; color: shade.paint }
        }
    }
    Text {
        id: titleItem
        objectName: "tileTitle." + root.place
        x: root.inset * root.unit
        y: factsItem.y - height
        width: root.width - 2 * root.inset * root.unit
        text: root.title
        elide: Text.ElideRight
        color: root.titleInk
        font.pixelSize: Type.size(root.titleSize * root.unit, Tokens.textScale)
        font.weight: Font.DemiBold
    }
    Text {
        id: factsItem
        objectName: "tileFacts." + root.place
        x: root.inset * root.unit
        y: root.height - root.foot * root.unit - height
        width: root.width - 2 * root.inset * root.unit
        text: root.facts
        elide: Text.ElideRight
        color: root.factsInk
        font.pixelSize: Type.size(root.factsSize * root.unit, Tokens.textScale)
        font.letterSpacing: 0.08 * root.factsSize * root.unit
    }
}
