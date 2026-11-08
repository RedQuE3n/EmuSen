pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme

// Up to four of a row's games fanned at the right, each cover whole at its own shape and a little turned (EmuSen_Interface.md §24.1).
Item {
    id: root
    // A section's first games (Library.firstOf), four at most fanned.
    required property Rows games
    // Canvas units to pixels, for the covers' decoded size.
    property real pixelScale: 1
    readonly property int shown: Math.min(4, root.games.count)
    Repeater {
        model: root.games
        Item {
            id: card
            required property int index
            required property string cover
            required property string title
            required property real consoleHue
            // The slot from the right of the canvas's four, so two covers take its last two, as the accepted render fans them.
            readonly property int slot: 4 - root.shown + card.index
            objectName: "fanCover." + card.index
            visible: card.index < root.shown
            width: 100
            height: 100
            x: root.width - 100 - (3 - card.slot) * 74
            y: root.height - 100
            z: card.index
            rotation: (card.slot - 1.5) * 4
            CoverTile {
                anchors.fill: parent
                source: card.cover
                title: card.title
                hue: card.consoleHue
                mark: Marks.heart
                alignRight: true
                pixelScale: root.pixelScale
            }
        }
    }
}
