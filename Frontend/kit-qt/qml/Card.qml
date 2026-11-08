import QtQuick
import EmuSen.Kit as Kit

// A card: a raised column of the card width with the tile radius, its content inside the pane gutter (EmuSen_Kit.md §7.8).
Rectangle {
    id: root
    default property alias content: slot.data
    implicitWidth: Kit.Tokens.cardWidth
    implicitHeight: slot.childrenRect.height + 2 * Kit.Tokens.paneGutter
    radius: Kit.Tokens.tileRadius
    color: Kit.Tokens.surfaceRaised
    Item {
        id: slot
        anchors.fill: parent
        anchors.margins: Kit.Tokens.paneGutter
    }
}
