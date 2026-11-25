import QtQuick
import EmuSen.Theme

// A media tile: its picture with its shadow at rest and the board's ring when focused, and its name underneath (EclipseGameMedia:64-70, 93).
Walkable {
    id: root
    property real unit: 1
    property real pixelScale: 1
    required property string label
    required property string picture
    property int place
    property color shadow: "transparent"
    property color ringGlow: "transparent"
    property real cardWidth: 316
    property real artHeight: 198
    property real corner: 18
    property real wordsGap: 14
    property real nameSize: 22
    width: root.cardWidth * root.unit
    height: nameItem.y + nameItem.height
    Accessible.name: root.label
    Item {
        id: artItem
        objectName: "mediaArt." + root.place
        width: root.width
        height: root.artHeight * root.unit
        Glow {
            objectName: "mediaShadow." + root.place
            anchors.fill: parent
            visible: !root.activeFocus
            radius: root.corner * root.unit
            color: root.shadow
            blur: 24 * root.unit
            offsetY: 8 * root.unit
            pixelScale: root.pixelScale
        }
        RoundPicture {
            objectName: "mediaPicture." + root.place
            anchors.fill: parent
            source: root.picture
            radius: root.corner * root.unit
            crop: true
            pixelScale: root.pixelScale
        }
        FocusRing {
            objectName: "mediaRing." + root.place
            anchors.fill: parent
            radius: root.corner * root.unit
            gap: 4 * root.unit
            thickness: 3 * root.unit
            glowColor: root.ringGlow
            glowBlur: 40 * root.unit
            glowSpread: 10 * root.unit
            bandRadius: (root.corner + 7) * root.unit
            pixelScale: root.pixelScale
            visible: root.activeFocus
        }
    }
    Line {
        id: nameItem
        objectName: "mediaName." + root.place
        unit: root.unit
        y: artItem.height + root.wordsGap * root.unit
        width: root.width
        text: root.label
        boardLine: true
        color: Tokens.text
        elide: Text.ElideRight
        size: root.nameSize
        weight: Font.DemiBold
    }
}
