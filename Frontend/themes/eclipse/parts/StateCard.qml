import QtQuick
import EmuSen.Theme

// A saved state's card: its picture with the resume badge where Resume loads it, its name and when it was saved, its shadow at rest and the board's ring when focused (EclipseGame:66-76, 105-106); every measure in board pixels.
Walkable {
    id: root
    // A board pixel in canvas units, Board.unit.
    property real unit: 1
    // Scene pixels to a canvas unit, so the glows' blur runs at the pixels drawn.
    property real pixelScale: 1
    required property string label
    required property string line
    required property string picture
    required property bool marked
    // The badge's words, the program's.
    property string badge
    // The card's place in its row, which names its parts.
    property int place
    property color shadow: "transparent"
    property color ringGlow: "transparent"
    property color lineInk: Tokens.muted
    // The board's measures: width, the picture's height and corner, the gap under it and between the words, the words' sizes.
    property real cardWidth: 316
    property real artHeight: 198
    property real corner: 18
    property real wordsGap: 14
    property real linesGap: 4
    property real nameSize: 22
    property real whenSize: 18
    readonly property Item art: artItem
    width: root.cardWidth * root.unit
    height: whenItem.y + whenItem.height
    Accessible.name: root.label
    Accessible.description: root.line
    Item {
        id: artItem
        objectName: "cardArt." + root.place
        width: root.width
        height: root.artHeight * root.unit
        Glow {
            objectName: "cardShadow." + root.place
            anchors.fill: parent
            visible: !root.activeFocus
            radius: root.corner * root.unit
            color: root.shadow
            blur: 24 * root.unit
            offsetY: 8 * root.unit
            pixelScale: root.pixelScale
        }
        RoundPicture {
            objectName: "cardPicture." + root.place
            anchors.fill: parent
            source: root.picture
            radius: root.corner * root.unit
            crop: true
            pixelScale: root.pixelScale
        }
        // The badge (EclipseGame:69): 14 in from the corner, 34 tall, 14 either side, its words 15 px bold at 0.08em.
        Rectangle {
            objectName: "cardBadge." + root.place
            visible: root.marked
            x: 14 * root.unit
            y: 14 * root.unit
            width: badgeWords.width + 2 * 14 * root.unit
            height: 34 * root.unit
            radius: height / 2
            color: Tokens.accent
            Line {
                id: badgeWords
                objectName: "cardBadgeWords." + root.place
                unit: root.unit
                x: 14 * root.unit
                y: (parent.height - height) / 2
                text: root.badge
                capitalization: Font.AllUppercase
                color: Tokens.overAccent
                size: 15
                weight: Font.Bold
                spacing: 0.08 * 15
            }
        }
        // The board's ring for a focused card: 4 px of the ground's dark, 3 px of the accent, 40 px of glow 10 px out (EclipseGame:106).
        FocusRing {
            objectName: "cardRing." + root.place
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
        objectName: "cardName." + root.place
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
    Line {
        id: whenItem
        objectName: "cardWhen." + root.place
        unit: root.unit
        y: nameItem.y + nameItem.height + root.linesGap * root.unit
        width: root.width
        text: root.line
        boardLine: true
        color: root.lineInk
        elide: Text.ElideRight
        size: root.whenSize
    }
}
