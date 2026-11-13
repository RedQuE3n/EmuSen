pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Models
import EmuSen.ThemeHost as Host
import EmuSen.App

// One EmuSen theme: its engine, sandboxed, one slot for each screen it may provide, and the feed that hands the engine the program's models and carries its Actions back (EmuSen_Qt.md §3.5). The window holds two: the reference theme, and the theme the player chose over it.
Item {
    id: themed
    property alias folder: host.folder
    property alias coverStores: host.coverStores
    required property Screens screens
    required property LibraryModel library
    required property SessionModel session
    required property Clock clock
    // The grid's games as the program holds them: the section shown, sorted, narrowed and searched.
    required property GameModel grid
    // The item the game's picture is drawn in, for Continue.
    property Item picture
    property alias controllers: feed.controllers
    property alias family: feed.family
    property alias reducedMotion: feed.reducedMotion
    property alias swap: feed.swap
    // Whether a shown screen takes the keys: not while the splash holds them.
    property bool inputs: true
    // Whether each role's slot is made: the reference's only where the chosen theme does not draw that role.
    property bool homeWanted: true
    property bool consolesWanted: true
    property bool gamesWanted: true
    property bool splashShown: false
    // Where the splash's slot is drawn: over Moonrise, which keeps the splash's clock, its skip and its fade (EmuSen_Interface.md §22).
    property Item splashLayer: themed
    readonly property bool searching: themed.screens.shown === Screens.Search || themed.screens.shown === Screens.SearchGrid
    readonly property bool typing: themed.screens.shown === Screens.Search
    readonly property bool listOpen: themed.screens.shown === Screens.List
    readonly property bool home: homeSlot.themed
    readonly property bool consoles: consolesSlot.themed
    readonly property bool games: gamesSlot.themed
    readonly property bool splash: splashSlot.themed
    readonly property alias failed: host.failed
    readonly property alias warnings: host.warnings
    readonly property alias host: host
    readonly property alias feed: feed
    // Whether one of the theme's screens holds the focus, so the pad's presses go to the theme's engine.
    readonly property bool holdsFocus: homeSlot.activeFocus || consolesSlot.activeFocus || gamesSlot.activeFocus
    // Whether one of its screens is to take the keys, for a host made inside a focus scope (the window's Loader).
    readonly property bool takesFocus: homeSlot.focus || consolesSlot.focus || gamesSlot.focus || splashSlot.focus
    // The Home screen placed, read as a theme's ThemeScreen (its engine's type, so not named here).
    readonly property var homeScreen: homeSlot.placed ? (homeSlot.item as Item)?.children[0] ?? null : null
    signal played(string path)
    signal resumed()
    signal backed()
    signal focused(string game)
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
        // Under a card the screen is drawn and nothing on it is reached: a backdrop to the gate, as the rules read it.
        property bool backdrop: false
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
        objectName: "host." + themed.objectName
    }
    ThemeFeed {
        id: feed
        objectName: "feed." + themed.objectName
        host: host
        clock: themed.clock.time
        now: themed.clock.now
        onPlayed: (game) => themed.played(game)
        onResumed: themed.resumed()
        onPlaceChosen: (index) => themed.screens.place(index)
        onOpened: (key) => themed.screens.open(key)
        onBacked: themed.backed()
        onActed: (key) => themed.screens.act(key)
        onFocused: (game) => themed.focused(game)
        onSelected: (index) => themed.library.select(index)
    }
    // A new engine is fed everything the feed holds, and the models as they stand, since a host made at run time (the
    // Themes tab's Use) has held nothing yet; each feed after is the one model that changed.
    Connections {
        target: host
        function onGenerationChanged() { feed.attach() }
    }
    Connections {
        target: host
        function onGenerationChanged() { feed.showLibrary(themed.library) }
    }
    Connections {
        target: host
        function onGenerationChanged() { feed.showSession(themed.session.game, themed.session.running, themed.clock.pausedAt(), themed.picture) }
    }
    Connections {
        target: host
        function onGenerationChanged() { feed.showBrowse(themed.grid, themed.screens.key, themed.searching, themed.typing, themed.listOpen) }
    }
    Connections {
        target: themed.library
        function onLoadingChanged() { feed.showLibrary(themed.library) }
        function onHomeLinesChanged() { feed.showLibrary(themed.library) }
    }
    Connections {
        target: themed.session
        function onRunningChanged() { feed.showSession(themed.session.game, themed.session.running, themed.clock.pausedAt(), themed.picture) }
        function onGameChanged() { feed.showSession(themed.session.game, themed.session.running, themed.clock.pausedAt(), themed.picture) }
    }
    Connections {
        target: themed.clock
        function onPausedChanged() { feed.showSession(themed.session.game, themed.session.running, themed.clock.pausedAt(), themed.picture) }
    }
    Connections {
        target: themed.grid
        function onRevisionChanged() { feed.showBrowse(themed.grid, themed.screens.key, themed.searching, themed.typing, themed.listOpen) }
        function onFocusedChanged() { feed.showBrowse(themed.grid, themed.screens.key, themed.searching, themed.typing, themed.listOpen) }
        function onCountLineChanged() { feed.showBrowse(themed.grid, themed.screens.key, themed.searching, themed.typing, themed.listOpen) }
        function onSearchChanged() { feed.showBrowse(themed.grid, themed.screens.key, themed.searching, themed.typing, themed.listOpen) }
    }
    Connections {
        target: themed.screens
        function onShownChanged() { feed.showBrowse(themed.grid, themed.screens.key, themed.searching, themed.typing, themed.listOpen) }
        function onKeyChanged() { feed.showBrowse(themed.grid, themed.screens.key, themed.searching, themed.typing, themed.listOpen) }
    }
    Slot {
        id: homeSlot
        objectName: "home." + themed.objectName
        role: Host.ThemeHost.Home
        host: host
        wanted: themed.homeWanted
        inputs: themed.inputs && themed.screens.shown === Screens.Home
        backdrop: themed.screens.shown === Screens.HomeCard
        enabled: !homeSlot.backdrop
        shown: themed.screens.shown === Screens.Home || themed.screens.shown === Screens.HomeCard
    }
    Slot {
        id: consolesSlot
        objectName: "consoles." + themed.objectName
        role: Host.ThemeHost.Consoles
        host: host
        wanted: themed.consolesWanted
        inputs: themed.inputs
        shown: themed.screens.shown === Screens.Consoles
    }
    Slot {
        id: gamesSlot
        objectName: "games." + themed.objectName
        role: Host.ThemeHost.Games
        host: host
        wanted: themed.gamesWanted
        // The search's keyboard and the List card hold the keys over it while they are open; under the card nothing on it is reached.
        inputs: themed.inputs && !themed.typing && !themed.listOpen
        backdrop: themed.listOpen
        enabled: !gamesSlot.backdrop
        shown: themed.screens.shown === Screens.Grid || themed.listOpen || themed.searching
    }
    Slot {
        id: splashSlot
        objectName: "splash." + themed.objectName
        role: Host.ThemeHost.Splash
        host: host
        inputs: themed.inputs
        parent: themed.splashLayer
        wanted: themed.splashShown
        shown: themed.splashShown
    }
}
