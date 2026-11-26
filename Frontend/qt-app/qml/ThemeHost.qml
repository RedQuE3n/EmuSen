pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
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
    // The window's Settings and game page models, for a theme that draws either screen; none, and neither slot is made.
    property SettingsModel settingsModel: null
    property GamePageModel pageModel: null
    property alias controllers: feed.controllers
    property alias family: feed.family
    // The machine's battery, -1 for none, and whether it charges, as the window's Battery reads them.
    property alias battery: feed.battery
    property alias charging: feed.charging
    property alias reducedMotion: feed.reducedMotion
    property alias swap: feed.swap
    // Whether a shown screen takes the keys: not while the splash holds them.
    property bool inputs: true
    // Whether each role's slot is made: the reference's only where the chosen theme does not draw that role.
    property bool homeWanted: true
    property bool consolesWanted: true
    property bool gamesWanted: true
    property bool bigScreen: true
    property int desktopView: 0
    property bool desktopGameShown: false
    property bool desktopLibraryWanted: true
    property bool desktopTableWanted: true
    property bool desktopPlayingWanted: true
    property bool desktopSettingsWanted: true
    property bool desktopPlayingInputs: false
    property bool splashShown: false
    // Where the splash's slot is drawn: over Moonrise, which keeps the splash's clock, its skip and its fade (EmuSen_Interface.md §22).
    property Item splashLayer: themed
    // A splash that enters Home on the window's clock: whether it is shown, and its time in ms, which the theme's `Splash` reads (EmuSen_Qt.md §3.5.3).
    property bool splashLeading: false
    property real splashTime: 0
    // From 1.10: the hand-over's length in ms after Home whole, the theme's `Splash.ease`.
    property real splashEase: 0
    readonly property bool searching: themed.screens.shown === Screens.Search || themed.screens.shown === Screens.SearchGrid
    readonly property bool typing: themed.screens.shown === Screens.Search
    readonly property bool listOpen: themed.screens.shown === Screens.List
    readonly property bool home: homeSlot.themed
    readonly property bool consoles: consolesSlot.themed
    readonly property bool games: gamesSlot.themed
    readonly property bool splash: splashSlot.themed
    readonly property bool settings: settingsSlot.themed
    readonly property bool game: gameSlot.themed
    readonly property bool desktopLibrary: desktopLibrarySlot.themed
    readonly property bool desktopTable: desktopTableSlot.themed
    readonly property bool desktopPlaying: desktopPlayingSlot.themed
    readonly property bool desktopSettings: desktopSettingsSlot.themed
    readonly property bool hasDesktopFace: host.ready && !host.failed && host.hasDesktopFace()
    // The theme draws the game's page as it stands: its screen is placed and names the tab shown, or names none (1.13).
    readonly property bool pageDrawn: gameSlot.themed && (themed.pageModel?.themeDraws ?? true)
    readonly property alias failed: host.failed
    readonly property alias warnings: host.warnings
    readonly property alias host: host
    readonly property alias feed: feed
    // Whether one of the theme's screens holds the focus, so the pad's presses go to the theme's engine.
    readonly property bool holdsFocus: homeSlot.activeFocus || consolesSlot.activeFocus || gamesSlot.activeFocus || settingsSlot.activeFocus || gameSlot.activeFocus || desktopLibrarySlot.activeFocus || desktopTableSlot.activeFocus || desktopPlayingSlot.activeFocus || desktopSettingsSlot.activeFocus
    // Whether one of its screens is to take the keys, for a host made inside a focus scope (the window's Loader).
    readonly property bool takesFocus: homeSlot.focus || consolesSlot.focus || gamesSlot.focus || splashSlot.focus || settingsSlot.focus || gameSlot.focus || desktopLibrarySlot.focus || desktopTableSlot.focus || desktopPlayingSlot.focus || desktopSettingsSlot.focus
    // The Home screen placed, read as a theme's ThemeScreen (its engine's type, so not named here).
    readonly property var homeScreen: homeSlot.placed ? (homeSlot.item as Placing)?.screen.children[0] ?? null : null
    // The splash placed, read as a theme's ThemeScreen; from 1.10 one that enters Home leads the start, with no Moonrise under it.
    readonly property var splashScreen: splashSlot.placed ? (splashSlot.item as Placing)?.screen.children[0] ?? null : null
    // The game's page placed, read as a theme's ThemeScreen, whose pageSections the page's model hears (1.13).
    readonly property var gameScreen: gameSlot.placed ? (gameSlot.item as Placing)?.screen.children[0] ?? null : null
    readonly property bool leads: themed.splash && (themed.splashScreen?.entersHome ?? false)
    readonly property bool splashComing: themed.bigScreen && themed.splashShown && host.ready && !host.failed && host.provides(Host.ThemeHost.Splash) && !themed.splash
    signal played(string path)
    signal resumed()
    signal backed()
    signal focused(string game)
    signal favourite(string game)
    signal bigScreenRequested(bool bigScreen)
    signal focusActiveSlot()
    onFocusActiveSlot: {
        if (desktopLibrarySlot.focus) desktopLibrarySlot.forceActiveFocus()
        else if (desktopTableSlot.focus) desktopTableSlot.forceActiveFocus()
        else if (desktopPlayingSlot.focus) desktopPlayingSlot.forceActiveFocus()
        else if (desktopSettingsSlot.focus) desktopSettingsSlot.forceActiveFocus()
        else if (homeSlot.focus) homeSlot.forceActiveFocus()
        else if (consolesSlot.focus) consolesSlot.forceActiveFocus()
        else if (gamesSlot.focus) gamesSlot.forceActiveFocus()
        else if (settingsSlot.focus) settingsSlot.forceActiveFocus()
        else if (gameSlot.focus) gameSlot.forceActiveFocus()
    }
    // What a slot holds: the scope the theme's screen is made in, out of the window's items while not shown, so a forceActiveFocus
    // in it climbs no scope of the window's, the slot's included, and takes no focus from the screen shown (EmuSen_Qt.md §3.5.2).
    component Placing: Item {
        id: placing
        property bool shown: false
        readonly property FocusScope screen: scope
        FocusScope {
            id: scope
            parent: placing.shown ? placing : null
            // Without a parent an item is visible by its own flag alone, and a screen's animations bound to it would run on.
            visible: placing.shown
            width: placing.width
            height: placing.height
            focus: true
        }
    }
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
        // Under a card nothing on the screen is reached.
        enabled: !slot.backdrop
        active: slot.wanted && slot.host.ready && slot.host.provides(slot.role)
        visible: slot.shown && slot.themed
        focus: slot.visible && slot.inputs
        sourceComponent: Placing {
            shown: slot.shown
        }
        onLoaded: slot.placed = slot.host.place(slot.role, (slot.item as Placing).screen)
    }
    Host.ThemeHost {
        id: host
        objectName: "host." + themed.objectName
    }
    ThemeFeed {
        id: feed
        objectName: "feed." + themed.objectName
        host: host
        bigScreen: themed.bigScreen
        clock: themed.clock.time
        shortTime: themed.clock.shortTime
        now: themed.clock.now
        settingsShown: settingsSlot.shown || desktopSettingsSlot.shown
        pageShown: gameSlot.shown || desktopPlayingSlot.shown
        shownSection: themed.screens.section
        splashShown: themed.splashLeading
        splashTime: themed.splashTime
        splashEase: themed.splashEase
        onPlayed: (game) => themed.played(game)
        onResumed: themed.resumed()
        onPlaceChosen: (index) => themed.screens.place(index)
        onOpened: (key) => themed.screens.open(key)
        onSectionOpened: (key) => themed.screens.openSection(key)
        onBacked: themed.backed()
        onActed: (key) => themed.screens.act(key)
        onListed: (key) => themed.grid.focused = themed.grid.listAct(key)
        onFocused: (game) => themed.focused(game)
        onFavourite: (game) => themed.favourite(game)
        onSelected: (index) => themed.library.select(index)
        onBigScreenRequested: (bigScreen) => themed.bigScreenRequested(bigScreen)
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
        target: host
        function onGenerationChanged() { feed.showSettings(themed.settingsModel) }
    }
    Connections {
        target: host
        function onGenerationChanged() { feed.showPage(themed.pageModel) }
    }
    // Each change of Settings' or the page's model, which the feed queues, so the changes of one press feed once.
    Connections {
        target: themed.settingsModel
        function onRowsInserted() { feed.showSettings(themed.settingsModel) }
        function onRowsRemoved() { feed.showSettings(themed.settingsModel) }
        function onDataChanged() { feed.showSettings(themed.settingsModel) }
        function onModelReset() { feed.showSettings(themed.settingsModel) }
        function onSectionChanged() { feed.showSettings(themed.settingsModel) }
        function onTrailChanged() { feed.showSettings(themed.settingsModel) }
        function onFocusedChanged() { feed.showSettings(themed.settingsModel) }
        function onHelpChanged() { feed.showSettings(themed.settingsModel) }
        function onSentenceChanged() { feed.showSettings(themed.settingsModel) }
        function onPaneLinesChanged() { feed.showSettings(themed.settingsModel) }
        function onPaneMutedChanged() { feed.showSettings(themed.settingsModel) }
        function onPreviewChanged() { feed.showSettings(themed.settingsModel) }
        function onCaptionChanged() { feed.showSettings(themed.settingsModel) }
        function onMeterChanged() { feed.showSettings(themed.settingsModel) }
        function onSamplesChanged() { feed.showSettings(themed.settingsModel) }
    }
    Connections {
        target: themed.pageModel
        function onDataChanged() { feed.showPage(themed.pageModel) }
        function onModelReset() { feed.showPage(themed.pageModel) }
        function onReadyChanged() { feed.showPage(themed.pageModel) }
        function onTitleChanged() { feed.showPage(themed.pageModel) }
        function onFactsChanged() { feed.showPage(themed.pageModel) }
        function onCoverChanged() { feed.showPage(themed.pageModel) }
        function onHeroChanged() { feed.showPage(themed.pageModel) }
        function onClipChanged() { feed.showPage(themed.pageModel) }
        function onSectionChanged() { feed.showPage(themed.pageModel) }
        function onShowsChanged() { feed.showPage(themed.pageModel) }
        function onWordsChanged() { feed.showPage(themed.pageModel) }
        function onPictureChanged() { feed.showPage(themed.pageModel) }
        function onActionsChanged() { feed.showPage(themed.pageModel) }
        function onHelpChanged() { feed.showPage(themed.pageModel) }
        function onCoverArtChanged() { feed.showPage(themed.pageModel) }
        function onAskingChanged() { feed.showPage(themed.pageModel) }
        function onQuestionTitleChanged() { feed.showPage(themed.pageModel) }
        function onFocusMoved() { feed.showPage(themed.pageModel) }
        function onViewingChanged() { feed.showPage(themed.pageModel) }
        function onViewKindChanged() { feed.showPage(themed.pageModel) }
        function onViewSourceChanged() { feed.showPage(themed.pageModel) }
        function onViewManualChanged() { feed.showPage(themed.pageModel) }
        function onViewCaptionChanged() { feed.showPage(themed.pageModel) }
        function onViewPageChanged() { feed.showPage(themed.pageModel) }
        function onViewZoomChanged() { feed.showPage(themed.pageModel) }
        function onViewPanXChanged() { feed.showPage(themed.pageModel) }
        function onViewPanYChanged() { feed.showPage(themed.pageModel) }
        function onViewHelpChanged() { feed.showPage(themed.pageModel) }
        function onThemeDrawsChanged() { feed.showPage(themed.pageModel) }
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
        shown: themed.bigScreen && (themed.screens.shown === Screens.Home || themed.screens.shown === Screens.HomeCard)
    }
    Slot {
        id: consolesSlot
        objectName: "consoles." + themed.objectName
        role: Host.ThemeHost.Consoles
        host: host
        wanted: themed.consolesWanted
        inputs: themed.inputs
        shown: themed.bigScreen && themed.screens.shown === Screens.Consoles
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
        shown: themed.bigScreen && (themed.screens.shown === Screens.Grid || themed.listOpen || themed.searching)
    }
    Slot {
        id: settingsSlot
        objectName: "settings." + themed.objectName
        role: Host.ThemeHost.Settings
        host: host
        wanted: themed.settingsModel !== null
        inputs: themed.inputs && !(themed.settingsModel?.listening ?? false) && !(themed.settingsModel?.typing ?? false) && !(themed.settingsModel?.bindingsShown ?? false) && !(themed.settingsModel?.shadersShown ?? false)
        shown: themed.bigScreen && themed.screens.shown === Screens.Settings && !(themed.settingsModel?.bindingsShown ?? false) && !(themed.settingsModel?.shadersShown ?? false)
    }
    // Trusted host stage for themed Media Viewer clips: keeps QtMultimedia out of the theme sandbox.
    Loader {
        id: viewerClipStage
        objectName: "themeViewerClipStage"
        anchors.fill: parent
        active: themed.bigScreen && gameSlot.shown && (themed.pageModel?.viewing ?? false) && (themed.pageModel?.viewKind ?? 0) === 1
        sourceComponent: Rectangle {
            color: Kit.Tokens.voidFill
            Kit.NormalizedCanvas {
                anchors.fill: parent
                Kit.VideoTile {
                    objectName: "themeViewerClip"
                    anchors.left: parent.left
                    anchors.right: parent.right
                    y: 34 * parent.unit
                    height: 990 * parent.unit
                    source: themed.pageModel.viewSource
                    still: themed.pageModel.hero
                    playing: true
                    muted: false
                    volume: themed.pageModel.clipVolume
                    fillMode: Image.PreserveAspectFit
                    delay: 1
                    fade: 0
                }
            }
        }
    }
    Slot {
        id: gameSlot
        objectName: "game." + themed.objectName
        role: Host.ThemeHost.Game
        host: host
        wanted: themed.pageModel !== null
        inputs: themed.inputs && !(themed.pageModel?.asking ?? false)
        backdrop: themed.pageModel?.asking ?? false
        shown: themed.bigScreen && themed.screens.shown === Screens.Page && ((themed.pageModel?.themeDraws ?? true) || (themed.pageModel?.asking ?? false))
    }
    Binding {
        target: themed.pageModel
        property: "themeSections"
        value: themed.gameScreen?.pageSections ?? 0
        when: themed.pageModel !== null
    }
    Binding {
        target: themed.pageModel
        property: "themeSubpages"
        value: themed.gameScreen?.pageSubpages ?? 0
        when: themed.pageModel !== null
    }
    Slot {
        id: splashSlot
        objectName: "splash." + themed.objectName
        role: Host.ThemeHost.Splash
        host: host
        // A splash takes no keys: the window keeps its skip, and one placed in Home would take Home's focus as it goes.
        inputs: false
        // A splash that enters Home is drawn in the place Home names, between what Home draws under it and over it (1.10).
        parent: themed.leads && themed.homeScreen?.splashPlace ? themed.homeScreen.splashPlace : themed.splashLayer
        wanted: themed.bigScreen && themed.splashShown
        shown: themed.bigScreen && themed.splashShown
    }
    Slot {
        id: desktopLibrarySlot
        objectName: "desktopLibrary." + themed.objectName
        role: Host.ThemeHost.DesktopLibrary
        host: host
        wanted: !themed.bigScreen && themed.desktopLibraryWanted
        inputs: themed.inputs
        shown: !themed.bigScreen && themed.desktopView === 0 && !themed.desktopGameShown
    }
    Slot {
        id: desktopTableSlot
        objectName: "desktopTable." + themed.objectName
        role: Host.ThemeHost.DesktopTable
        host: host
        wanted: !themed.bigScreen && themed.desktopTableWanted
        inputs: themed.inputs
        shown: !themed.bigScreen && themed.desktopView === 1 && !themed.desktopGameShown
    }
    Slot {
        id: desktopPlayingSlot
        objectName: "desktopPlaying." + themed.objectName
        role: Host.ThemeHost.DesktopPlaying
        host: host
        wanted: !themed.bigScreen && themed.desktopPlayingWanted
        inputs: themed.inputs && themed.desktopPlayingInputs
        shown: !themed.bigScreen && themed.desktopGameShown
    }
    Slot {
        id: desktopSettingsSlot
        objectName: "desktopSettings." + themed.objectName
        role: Host.ThemeHost.DesktopSettings
        host: host
        wanted: !themed.bigScreen && themed.desktopSettingsWanted && themed.settingsModel !== null
        inputs: themed.inputs && !(themed.settingsModel?.listening ?? false) && !(themed.settingsModel?.typing ?? false) && !(themed.settingsModel?.bindingsShown ?? false) && !(themed.settingsModel?.shadersShown ?? false)
        shown: !themed.bigScreen && themed.desktopView === 2 && !themed.desktopGameShown && !(themed.settingsModel?.bindingsShown ?? false) && !(themed.settingsModel?.shadersShown ?? false)
    }
}
