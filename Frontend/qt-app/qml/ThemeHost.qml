pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.ThemeHost as Host

// The chosen EmuSen theme: its engine, sandboxed, and one slot for each screen it may provide (EmuSen_Qt.md §3.5). A role the theme omits, and every role once the theme fails, is drawn by the reference, which the window shows where `themed` says no.
Item {
    id: themed
    property alias folder: host.folder
    property alias coverStores: host.coverStores
    property bool homeShown: false
    property bool consolesShown: false
    property bool gamesShown: false
    property bool splashShown: false
    // Whether a shown screen takes the keys: not while the splash holds them.
    property bool inputs: true
    // Where the splash's slot is drawn: over Moonrise, which keeps the splash's clock, its skip and its fade (EmuSen_Interface.md §22).
    property Item splashLayer: themed
    readonly property bool home: homeSlot.themed
    readonly property bool consoles: consolesSlot.themed
    readonly property bool games: gamesSlot.themed
    readonly property bool splash: splashSlot.themed
    readonly property alias failed: host.failed
    readonly property alias warnings: host.warnings
    readonly property alias host: host
    // Whether one of the theme's screens holds the focus, so the pad's presses go to the theme's engine.
    readonly property bool holdsFocus: homeSlot.activeFocus || consolesSlot.activeFocus || gamesSlot.activeFocus
    // One screen of the theme: a slot made while the theme provides its role, the screen created into it in the theme's own engine.
    component Slot: Loader {
        id: slot
        required property int role
        required property Host.ThemeHost host
        property bool inputs: true
        property bool shown: false
        // Whether the slot is made at all: the splash's only while the splash is shown, so it does not run on after.
        property bool wanted: true
        property bool placed: false
        // The theme's screen is in place and the theme has not failed; else the reference draws this role.
        readonly property bool themed: slot.active && slot.status === Loader.Ready && slot.placed && !slot.host.failed
        anchors.fill: parent
        active: slot.wanted && slot.host.ready && slot.host.provides(slot.role)
        visible: slot.shown && slot.themed
        focus: slot.visible && slot.inputs
        sourceComponent: FocusScope {
            focus: true
        }
        onLoaded: slot.placed = slot.host.place(slot.role, slot.item)
    }
    Host.ThemeHost {
        id: host
        objectName: "themeHost"
    }
    Slot {
        id: homeSlot
        objectName: "themeHome"
        role: Host.ThemeHost.Home
        host: host
        inputs: themed.inputs
        shown: themed.homeShown
    }
    Slot {
        id: consolesSlot
        objectName: "themeConsoles"
        role: Host.ThemeHost.Consoles
        host: host
        inputs: themed.inputs
        shown: themed.consolesShown
    }
    Slot {
        id: gamesSlot
        objectName: "themeGames"
        role: Host.ThemeHost.Games
        host: host
        inputs: themed.inputs
        shown: themed.gamesShown
    }
    Slot {
        id: splashSlot
        objectName: "themeSplash"
        role: Host.ThemeHost.Splash
        host: host
        inputs: themed.inputs
        parent: themed.splashLayer
        wanted: themed.splashShown
        shown: themed.splashShown
    }
}
