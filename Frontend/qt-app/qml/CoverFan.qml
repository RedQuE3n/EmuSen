pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Models

// Up to four of a row's games fanned at the right, each a card in its console's hue with its title, until covers are read (EmuSen_Interface.md §24.1).
Item {
    id: root
    required property GameModel games
    readonly property int shown: Math.min(4, root.games.count)
    Repeater {
        model: root.games
        GameCard {
            id: card
            required property int index
            objectName: "fanCover." + card.index
            visible: card.index < root.shown
            width: 76
            height: 102
            x: root.width - (root.shown - card.index) * 70 - 8
            y: (root.height - height) / 2 + (card.index % 2) * 2
            rotation: (card.index - (root.shown - 1) / 2) * 4
        }
    }
}
