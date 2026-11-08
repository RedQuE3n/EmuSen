pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models

// A game's cover drawn whole at its own shape in a box, on the box's foot, with the favourite's heart; a game with no art shows the generated card (EmuSen_Interface.md §21, §24.2).
Item {
    id: root
    property string source
    property string title
    property real hue
    property bool favourite
    property bool lit
    // How far a lit cover grows, about its centre, and rises, as the gallery lifts it.
    property real litScale: 1.06
    property real litRise: 6
    // Whether the picture stands at the box's right rather than its middle.
    property bool alignRight: false
    // Whether the picture was read and drawn, not the generated card.
    readonly property bool drawn: picture.status === Image.Ready
    readonly property real paintedWidth: picture.status === Image.Ready ? picture.paintedWidth : card.width
    readonly property real paintedHeight: picture.status === Image.Ready ? picture.paintedHeight : card.height
    readonly property real paintedX: root.alignRight ? root.width - root.paintedWidth : (root.width - root.paintedWidth) / 2
    readonly property real paintedY: root.height - root.paintedHeight
    Item {
        id: drawn
        width: root.width
        height: root.height
        y: root.lit ? -root.litRise : 0
        scale: root.lit ? root.litScale : 1
        transformOrigin: Item.Center
        RoundPicture {
            id: picture
            objectName: "coverPicture"
            Accessible.role: Accessible.Graphic
            Accessible.name: root.drawn ? root.title : ""
            anchors.fill: parent
            visible: root.source.length > 0
            source: root.source
            radius: Kit.Tokens.tileRadius
            verticalAlignment: Image.AlignBottom
            horizontalAlignment: root.alignRight ? Image.AlignRight : Image.AlignHCenter
        }
        GameCard {
            id: card
            visible: root.source.length === 0 || picture.status === Image.Error
            width: root.width
            height: Math.min(root.height, root.width)
            y: root.height - height
            title: root.title
            hue: root.hue
        }
        Glyph {
            objectName: "coverHeart"
            visible: root.favourite
            x: root.paintedX + root.paintedWidth - 8 - width
            y: root.paintedY + 8
            width: 24
            height: 24
            path: Marks.heart
            filled: true
            ink: Kit.Tokens.accent
        }
        Rectangle {
            objectName: "coverRing"
            visible: root.lit
            x: root.paintedX - 7
            y: root.paintedY - 7
            width: root.paintedWidth + 14
            height: root.paintedHeight + 14
            radius: Kit.Tokens.tileRadius + 4
            color: "transparent"
            border.color: Kit.Tokens.accent
            border.width: Kit.Tokens.ringThickness
        }
    }
}
