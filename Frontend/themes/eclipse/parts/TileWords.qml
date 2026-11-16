import QtQuick
import EmuSen.Theme

// A tile's foot as the board draws it (Main:84-86): a shade the board's 96 px tall from rgba(4, 5, 10, 0.92) at the foot to
// clear, the console line 12 px above the tile's foot and the title 34 px above it, each inset 18 px.
Item {
    id: root
    // A board pixel in canvas units, Board.unit.
    required property real unit
    required property string title
    required property string facts
    // The tile's place in its row, which names its parts for a reader and the tests.
    property int place
    // The shade's corners, the tile's where nothing clips it.
    property real radius: 0
    property real titleSize: 21
    property real factsSize: 15
    property real inset: 18
    property real foot: 12
    property real titleFoot: 34
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
    Line {
        id: titleItem
        name: "tileTitle." + root.place
        unit: root.unit
        x: root.inset * root.unit
        // Its foot the board's 34 px over the tile's (Main:85).
        y: root.height - root.titleFoot * root.unit - height
        width: root.width - 2 * root.inset * root.unit
        text: root.title
        elide: Text.ElideRight
        color: root.titleInk
        size: root.titleSize
        weight: Font.DemiBold
    }
    Line {
        id: factsItem
        name: "tileFacts." + root.place
        unit: root.unit
        x: root.inset * root.unit
        y: root.height - root.foot * root.unit - height
        visible: root.facts.length > 0
        // As wide as its words, as the board's line placed by its left alone is, and no wider than the tile less its insets.
        width: Math.min(implicitWidth, root.width - 2 * root.inset * root.unit)
        text: root.facts
        elide: Text.ElideRight
        color: root.factsInk
        size: root.factsSize
        spacing: 0.08 * root.factsSize
        capitalization: Font.AllUppercase
    }
}
