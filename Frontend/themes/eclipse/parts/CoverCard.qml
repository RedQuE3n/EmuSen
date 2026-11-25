pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme

// One card of Cover Art's 4-column grid (EclipseGameCover:45-50): 230x270 picture, optional In use pill, 19px label below.
Walkable {
    id: root
    required property real unit
    required property int index
    required property string label
    required property string picture
    required property bool marked
    required property string badge
    property real pixelScale: 1
    property color shadow: "transparent"
    property color ringGlow: "transparent"
    readonly property bool lit: root.activeFocus
    objectName: "coverCard." + root.index
    Accessible.name: root.label
    Accessible.description: root.marked ? root.badge : ""
    width: 230 * root.unit
    height: 331.4 * root.unit
    Item {
        id: art
        objectName: "coverArt." + root.index
        width: 230 * root.unit
        height: 270 * root.unit
        Glow {
            objectName: "coverShadow." + root.index
            anchors.fill: parent
            visible: !root.lit
            radius: 16 * root.unit
            color: root.shadow
            blur: 30 * root.unit
            offsetY: 10 * root.unit
            pixelScale: root.pixelScale
        }
        FocusRing {
            objectName: "coverRing." + root.index
            anchors.fill: parent
            visible: root.lit
            radius: 16 * root.unit
            gap: 4 * root.unit
            thickness: 3 * root.unit
            glowColor: root.ringGlow
            glowBlur: 40 * root.unit
            glowSpread: 10 * root.unit
            bandRadius: 23 * root.unit
            pixelScale: root.pixelScale
        }
        Rectangle {
            objectName: "coverPlate." + root.index
            anchors.fill: parent
            radius: 16 * root.unit
            color: Tokens.inputSurface
        }
        RoundPicture {
            objectName: "coverPicture." + root.index
            anchors.fill: parent
            visible: root.picture.length > 0
            source: root.picture
            radius: 16 * root.unit
            crop: true
            pixelScale: root.pixelScale
        }
        Rectangle {
            id: inUse
            objectName: "coverBadge." + root.index
            visible: root.marked && root.badge.length > 0
            x: 12 * root.unit
            y: 12 * root.unit
            width: badgeLine.width + 28 * root.unit
            height: 34 * root.unit
            radius: 17 * root.unit
            color: Tokens.accent
            Line {
                id: badgeLine
                objectName: "coverBadgeWords." + root.index
                unit: root.unit
                x: 14 * root.unit
                y: (inUse.height - height) / 2
                boardLine: true
                text: root.badge
                color: Tokens.overAccent
                size: 16
                weight: Font.Bold
            }
        }
    }
    Line {
        objectName: "coverLabel." + root.index
        unit: root.unit
        x: 0
        y: art.height + 12 * root.unit
        width: root.width
        text: root.label
        color: Tokens.text
        size: 19
        weight: Font.Normal
        lineHeightMode: Text.FixedHeight
        lineHeight: 1.3 * 19
        wrapMode: Text.WordWrap
        maximumLineCount: 2
        elide: Text.ElideRight
    }
}
