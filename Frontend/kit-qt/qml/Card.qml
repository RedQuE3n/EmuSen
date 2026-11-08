import QtQuick
import EmuSen.Kit

// A card: a raised column of the card width with the tile radius, its content inside the pane gutter (EmuSen_Kit.md §7.8).
Rectangle {
    id: root
    default property alias content: slot.data
    implicitWidth: Tokens.cardWidth
    implicitHeight: slot.childrenRect.height + 2 * Tokens.paneGutter
    radius: Tokens.tileRadius
    color: Tokens.surfaceRaised
    Item {
        id: slot
        anchors.fill: parent
        anchors.margins: Tokens.paneGutter
    }
}
