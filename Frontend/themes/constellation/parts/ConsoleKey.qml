pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme

// The key under the chart, a glass panel: the focused console's body at 48, its full name in Display and its facts on one line each, and three covers whole in a row, 88 tall, right-anchored; a Pointer's mark in a glass disc, its name, its line and its first three games (constellation.md §3).
Glass {
    id: key
    required property Faces faces
    property var node: null
    property real pixelScale: 1
    readonly property bool isPointer: key.node !== null && key.node.isPointer
    readonly property string gamesKey: key.node ? key.node.key : ""
    Body {
        objectName: "keyBody"
        visible: key.node !== null && !key.isPointer
        x: 20
        y: 38
        width: 48
        height: 48
        world: key.node && !key.isPointer ? key.node.body : ""
    }
    Item {
        objectName: "keyDisc"
        visible: key.isPointer
        x: 20
        y: 38
        width: 48
        height: 48
        Glass {
            anchors.fill: parent
            radius: 24
        }
        Glyph {
            x: 12
            y: 12
            width: 24
            height: 24
            path: Marks.path(key.gamesKey)
            ink: Tokens.textStrong
        }
    }
    Text {
        objectName: "keyName"
        x: 88
        y: 20
        width: covers.x - 88 - 20
        text: key.node ? (key.isPointer ? key.node.name : key.node.displayName) ?? "" : ""
        elide: Text.ElideRight
        color: Tokens.textStrong
        font.family: key.faces?.condensed ?? ""
        font.weight: Font.Medium
        font.pixelSize: 44
        font.letterSpacing: 2
        font.capitalization: Font.AllUppercase
        lineHeight: 52.8
        lineHeightMode: Text.FixedHeight
    }
    Text {
        objectName: "keyFacts"
        x: 88
        y: 76.8
        width: covers.x - 88 - 20
        text: key.node ? (key.isPointer ? key.node.line : key.node.facts) ?? "" : ""
        elide: Text.ElideRight
        color: Tokens.muted
        font.family: key.faces?.regular ?? ""
        font.pixelSize: 22
        lineHeight: 26.4
        lineHeightMode: Text.FixedHeight
    }
    Row {
        id: covers
        objectName: "keyCovers"
        x: key.width - 20 - width
        y: 18
        height: 88
        spacing: 12
        Repeater {
            model: Library.firstOf(key.gamesKey, 3)
            Item {
                id: cover
                required property int index
                required property var model
                readonly property real shape: cover.model.coverShape
                objectName: "keyCover." + cover.index
                width: Math.min(132, Math.round(88 * cover.shape))
                height: 88
                CoverTile {
                    anchors.fill: parent
                    source: cover.model.cover
                    title: cover.model.title
                    hue: cover.model.consoleHue
                    favourite: cover.model.favourite
                    mark: Marks.heart
                    lifts: false
                    pixelScale: key.pixelScale
                }
            }
        }
    }
}
