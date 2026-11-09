import QtQuick
import EmuSen.Theme

// A game of a list on its orbit: the kit's OrbitSlot places it and stacks it under the covers nearer the apex; its cover drawn whole, the ring when lit (orrery.md §1.2, §2).
OrbitSlot {
    id: slot
    required property Orbit orbit
    required property real pixelScale
    required property string key
    required property string title
    required property string cover
    required property real consoleHue
    required property bool favourite
    required property string facts
    view: slot.orbit
    turning: slot.orbit.turning
    lit: slot.ListView.isCurrentItem && slot.orbit.activeFocus
    Accessible.name: slot.title
    focus: slot.ListView.isCurrentItem
    // A cover given the focus directly, by the pointer or a walk, becomes the list's current one, so the orbit turns to it.
    onActiveFocusChanged: if (slot.activeFocus) slot.orbit.currentIndex = slot.index
    CoverTile {
        anchors.fill: parent
        source: slot.cover
        title: slot.title
        hue: slot.consoleHue
        favourite: slot.favourite
        mark: Marks.heart
        lit: slot.lit
        lifts: false
        // The orbit scales the slot, so the card's title keeps its size on the screen.
        drawnScale: slot.size
        pixelScale: slot.pixelScale
    }
}
