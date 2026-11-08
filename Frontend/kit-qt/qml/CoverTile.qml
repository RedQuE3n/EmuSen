pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit

// A game's cover drawn whole at its own shape in a box, on the box's foot, with the favourite's mark; a game with no art shows the generated card in its console's hue (EmuSen_Interface.md §21, §24.2; EmuSen_Kit.md §7.29).
Item {
    id: root
    property string source
    property string title
    property real hue
    property bool favourite
    property bool lit
    // The favourite's mark as an SVG path in a 24-unit box, the host's or the theme's (Marks.heart).
    property string mark
    // Canvas units to pixels, for the picture's decoded size.
    property real pixelScale: 1
    // How far a lit cover grows, about its centre, and rises, as the gallery lifts it.
    property real litScale: 1.06
    property real litRise: 6
    // Whether a lit cover lifts at all; not under the docked keyboard, whose room holds the cover and no more (EmuSen_Qt.md §7.6).
    property bool lifts: true
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
        y: root.lit && root.lifts ? -root.litRise : 0
        scale: root.lit && root.lifts ? root.litScale : 1
        transformOrigin: Item.Center
        Kit.RoundPicture {
            id: picture
            objectName: "coverPicture"
            Accessible.role: Accessible.Graphic
            Accessible.name: root.drawn ? root.title : ""
            anchors.fill: parent
            visible: root.source.length > 0
            source: root.source
            radius: Kit.Tokens.tileRadius
            pixelScale: root.pixelScale
            verticalAlignment: Image.AlignBottom
            horizontalAlignment: root.alignRight ? Image.AlignRight : Image.AlignHCenter
        }
        // The generated card: the title on the console's hue, as Mistress draws a game it has no art for.
        Rectangle {
            id: card
            visible: root.source.length === 0 || picture.status === Image.Error
            width: root.width
            height: Math.min(root.height, root.width)
            y: root.height - height
            radius: Kit.Tokens.rowRadius
            gradient: Gradient {
                GradientStop { position: 0; color: Qt.hsla(root.hue / 360, 0.45, 0.32, 1) }
                GradientStop { position: 1; color: Qt.hsla(root.hue / 360, 0.5, 0.16, 1) }
            }
            border.color: Kit.Tokens.rule
            Text {
                anchors.fill: parent
                anchors.margins: 6
                text: root.title
                wrapMode: Text.Wrap
                elide: Text.ElideRight
                verticalAlignment: Text.AlignBottom
                color: Kit.Tokens.textStrong
                font.pixelSize: Kit.Tokens.fontBody
            }
        }
        Item {
            objectName: "coverHeart"
            Accessible.role: Accessible.Graphic
            visible: root.favourite
            x: root.paintedX + root.paintedWidth - 8 - width
            y: root.paintedY + 8
            width: 24
            height: 24
            Shape {
                width: 24
                height: 24
                preferredRendererType: Shape.CurveRenderer
                ShapePath {
                    fillColor: Kit.Tokens.accent
                    strokeColor: "transparent"
                    strokeWidth: -1
                    fillRule: ShapePath.OddEvenFill
                    capStyle: ShapePath.RoundCap
                    joinStyle: ShapePath.RoundJoin
                    PathSvg { path: root.mark }
                }
            }
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
