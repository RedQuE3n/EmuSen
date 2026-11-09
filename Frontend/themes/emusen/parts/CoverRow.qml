pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme

// Three covers whole in a row, each at its console's shape no taller than 100 nor wider than 150, 12 apart, their feet on the row's foot: the fan's replacement, so no cover hides another's words (EmuSen_Qt.md §3.4.2).
Row {
    id: root
    // A section's first games (Library.firstOf), three at most.
    required property Rows games
    // Canvas units to pixels, for the covers' decoded size.
    property real pixelScale: 1
    spacing: 12
    Repeater {
        model: root.games
        Item {
            id: card
            required property int index
            required property string cover
            required property string title
            required property real consoleHue
            required property real coverShape
            required property bool favourite
            readonly property real shape: card.coverShape > 0 ? card.coverShape : 1
            objectName: "rowCover." + card.index
            visible: card.index < 3
            width: card.visible ? Math.min(150, Math.round(100 * card.shape)) : 0
            height: Math.round(100 * card.shape) > 150 ? Math.round(150 / card.shape) : 100
            y: root.height - card.height
            CoverTile {
                anchors.fill: parent
                source: card.cover
                title: card.title
                hue: card.consoleHue
                favourite: card.favourite
                mark: Marks.heart
                lifts: false
                pixelScale: root.pixelScale
            }
        }
    }
}
