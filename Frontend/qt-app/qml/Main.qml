pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Dialogs
import EmuSen.Kit as Kit
import EmuSen.Models
import EmuSen.Endymion
import QtQuick.Controls
import EmuSen.Play as Play
import EmuSen.App
import EmuSen.Esde as Esde

// emusen-qt's window: the screens on the big screen's canvas, the game under them and the Pause Card over it, Moonrise over them at the start until it ends and the library is read, the pads through Endymion (EmuSen_Qt.md §7, EmuSen_Interface.md §22).
Window {
    id: program
    objectName: "program"
    // Shown by the program once its scene graph has adopted the device, as a test harness shows it.
    visible: false
    width: 1280
    height: 800
    color: Kit.Tokens.voidFill
    property string platformLibrary
    // The game picture's feed, which the program's start names after adopting Serenity's device; 0 draws no game.
    property double feed: 0
    property bool splash: true
    property bool reducedMotion: false
    // Steam's Game Mode holds the session: Power holds Quit alone.
    property bool gameMode: false
    // AppSettings.GuideChords as the start read it: On, or Auto away from Steam's Game Mode.
    property bool guideChords: false
    // What the Fullscreen hotkey last asked of the window, apart from the visibility the compositor gives it.
    property bool fullScreenAsked: false
    // The EmuSen theme's folder, empty for the reference, and the folders its pictures may come from (EmuSen_Qt.md §3.5).
    property string theme
    property list<string> coverStores
    // The pads the reader opens, by SDL id, where a test names its own; empty for every pad.
    property string onlyPads
    // A file holding Cores' world in place of the back end's, as the recordings script theirs; empty for the back end.
    property string coresWorld
    // The scraper's test seam, its settings' `test` member as JSON (a loopback server and its root), as a test names it.
    property string scraperTest
    // The splash keeps the focus, and every press, until it has faded, so the press that skipped it is never a screen's.
    readonly property bool splashShown: program.splash && (!moonrise.done || moonrise.opacity > 0)
    // The help bar's family: Controller type's, or with Automatic the pad last used; a generic pad's glyphs drawn as the Xbox pad's lettered ones, as Mistress does.
    readonly property int padFamily: {
        const family = settingsModel.controllerFamily >= 0 ? settingsModel.controllerFamily : pads.family
        return family === Kit.Families.Generic ? Kit.Families.Xbox : family
    }
    readonly property bool gameShown: screens.shown === Screens.Game
    readonly property bool pauseShown: screens.shown === Screens.Pause
    // The chosen theme's host while one is made.
    readonly property ThemeHost themeHost: chosenTheme.item as ThemeHost
    // Why the reference is shown in place of the theme the settings choose, said once the splash has gone.
    property string themeWords
    // An ES-DE theme is the library's look, as the start read the settings, and its navigation sounds are on (EmuSen_Qt.md §4.15).
    property bool esdeChosen: false
    property bool navigationSounds: true
    readonly property Esde.View esdeView: esdeLoader.item as Esde.View
    readonly property bool esdeWanted: program.esdeChosen || library.themed
    // A theme that failed to load, or loaded with nothing to show, gives way to the own look.
    readonly property bool esdeFailed: (program.esdeView?.scene.problem.length ?? 0) > 0 || ((program.esdeView?.scene.revision ?? 0) > 0 && !(program.esdeView?.scene.themed ?? false))
    readonly property bool esdeLook: program.esdeWanted && !program.esdeFailed
    // The splash waits for the look it gives way to: the own look's at once, the theme's once it has loaded or failed.
    readonly property bool lookSettled: !program.esdeWanted || program.esdeFailed || (program.esdeView?.scene.themed ?? false)
    // The game in hand, whose page X opens: the one the theme's focus is on, else the grid's focused game.
    property string inHand
    // A game chosen: asked about where its core waits for approval or its firmware is missing, then recorded as started,
    // opened (resumed where it is the one suspended) and shown.
    signal play(string path)
    onPlay: (path) => {
        player.session.startState = ""
        launch.prepare(path)
    }
    // A game chosen with a saved state to start from, by its file: a game's page's Resume or state (EmuSen_Qt.md §7.8).
    signal playFrom(string path, string state)
    onPlayFrom: (path, state) => {
        player.session.startState = state
        launch.prepare(path)
    }
    // Every notice is raised here, and words that say nothing (a model's cleared problem) raise none (EmuSen_Qt.md §7.9).
    signal said(string words, bool attention, bool overGame)
    onSaid: (words, attention, overGame) => { if (words.length > 0) notices.raise(words, attention, overGame) }
    // Every item under one, so a key no screen takes reaches it: a hotkey's action away from the game, as Mistress's
    // window heard a key its views passed on (EmuSen_Qt.md §7.7.H8). Over the game, Game.qml hears the keys first.
    Item {
        id: shell
        objectName: "shell"
        anchors.fill: parent
        Keys.onPressed: (event) => event.accepted = !program.gameShown && player.session.awayKey(event.key, event.modifiers, true, event.isAutoRepeat, !program.pauseShown)
        Keys.onReleased: (event) => event.accepted = !program.gameShown && player.session.awayKey(event.key, event.modifiers, false, event.isAutoRepeat, !program.pauseShown)

        LaunchModel {
            id: launch
            objectName: "launch"
            platformLibrary: program.platformLibrary
            cores: library.cores
            onReady: (path) => {
                library.started(path)
                // The game's screen asked for first, so an open that fails at once gives way to the library after it.
                screens.play()
                player.game = path
                player.session.resume()
            }
            onNoticeChanged: program.said(launch.notice, launch.noticeAttention, true)
        }
        Screens {
            id: screens
            objectName: "screens"
            gameRunning: player.session.running
            lastPlayed: library.lastPlayed
            found: games.count > 0
            themed: program.esdeLook
            onQuit: Qt.quit()
            // The suspended game resumes as its screen comes back: the Home card's Continue, or resume() (a library's Continue, Exit to Library over the library).
            onResumed: player.session.resume()
            // Start Console's row: the console starts with nothing in it, and its screen follows the session's start.
            onConsoleStarted: (key) => player.session.openConsole(key)
        }
        // Reduced motion as the start read it until Settings has read the settings, then as they say; the text size, which
        // the kit's tokens scale by, and the variant as they say (EmuSen_Qt.md §7.7).
        Binding {
            target: Kit.Tokens
            property: "reducedMotion"
            value: settingsModel.ready ? settingsModel.reducedMotion : program.reducedMotion
        }
        Binding {
            target: Kit.Tokens
            property: "textSize"
            value: settingsModel.textSize
        }
        Binding {
            target: Kit.Tokens
            property: "variant"
            value: settingsModel.light ? Kit.Variants.Light : Kit.Variants.Dark
            when: settingsModel.ready
        }
        LibraryModel {
            id: library
            objectName: "library"
            platformLibrary: program.platformLibrary
            // The grid's Year and Rating sort by the scraper's facts, read for the own look only while one of them is chosen.
            byFacts: games.byFacts
        }
        PadReader {
            id: pads
            objectName: "pads"
            platformLibrary: program.platformLibrary
            onlyPads: program.onlyPads
            // While Guide is held for its chords the game hears nothing, as Mistress keeps the pad from it.
            gameMuted: Kit.PadRouter.guideHeld
            // A theme's screens hear the pad through their own engine's router (EmuSen_Qt.md §3.5.2); while a binding listens
            // for the pad's press, the interface hears none of it, and while the tester runs, only its B and the releases (EmuSen_Qt.md §7.7.H).
            onButton: (button, down) => settingsModel.listeningPad ? undefined : settingsModel.testing ? (settingsModel.testerButton(button, down, Kit.PadRouter.swapAcceptBack) ? Kit.PadRouter.push(button, down) : undefined) : program.themeHost?.holdsFocus ? program.themeHost.feed.push(button, down) : referenceHost.holdsFocus ? referenceHost.feed.push(button, down) : Kit.PadRouter.push(button, down)
            onSeatsChanged: settingsModel.seatsChanged()
            // A pad come or gone, told where Controller notices is on, over a game too (EmuSen_Input.md §8.13.5).
            onNotice: (words) => {
                if (settingsModel.controllerNotices)
                    program.said(words, false, true)
            }
        }
        // Swap A and B in menus, once Settings has read the settings; the tester and the help bar's glyphs read the router's (EmuSen_Qt.md §7.7.H3).
        Binding {
            target: Kit.PadRouter
            property: "swapAcceptBack"
            value: settingsModel.swapButtons
            when: settingsModel.ready
        }
        // Rumble turned while a game runs reaches it at once.
        Binding {
            target: player.session
            property: "rumble"
            value: settingsModel.rumble
            when: settingsModel.ready
        }
        // Volume, Output, Sync to display and Pause in background changed while a game runs reach it at once.
        Binding {
            target: player.session
            property: "volume"
            value: settingsModel.volume
            when: settingsModel.ready
        }
        Binding {
            target: player.session
            property: "output"
            value: settingsModel.output
            when: settingsModel.ready
        }
        Binding {
            target: player.session
            property: "syncToDisplay"
            value: settingsModel.syncToDisplay
            when: settingsModel.ready
        }
        Binding {
            target: player.session
            property: "pauseInBackground"
            value: settingsModel.pauseInBackground
            when: settingsModel.ready
        }
        // Home's lines that change with the time: the status line and the paused game's facts.
        Clock {
            id: clock
            objectName: "clock"
            controllers: pads.connected
            console: player.session.running ? library.consoleOf(player.session.game) : ""
            gamePaused: player.session.running && player.session.paused
        }
        Timer {
            interval: 15000
            repeat: true
            running: true
            onTriggered: clock.tick()
        }
        // The game, under the screens: in front on its own screen, paused under the Pause Card, and behind Home while it is suspended there, where Continue draws its picture.
        Play.Game {
            id: player
            objectName: "game"
            anchors.fill: parent
            visible: program.gameShown || program.pauseShown || (screens.shown === Screens.Home && player.session.running)
            inFront: program.gameShown && !player.session.resumeAsked
            // As the start read them until the settings are read, then as they say, changed on the screen or not.
            guideChords: settingsModel.ready ? settingsModel.guideChords : program.guideChords
            showStatusText: settingsModel.showStatusText
            showFps: settingsModel.showFps
            showStatusBar: settingsModel.showStatusBar
            focus: program.gameShown && !player.session.resumeAsked
            feed: program.feed
            platformLibrary: program.platformLibrary
            cores: library.cores
            pads: pads.gamePads
            onMenu: {
                player.session.pause()
                screens.menu()
            }
            // The window's hotkeys: Exit to Library suspends the game to the library as the card's Back to Library does,
            // Fullscreen turns the window's full screen, as Mistress's `ToggleFullScreen`.
            onExitAsked: {
                player.session.pause()
                screens.library()
            }
            onFullScreenAsked: {
                program.fullScreenAsked = program.visibility !== Window.FullScreen
                program.visibility = program.fullScreenAsked ? Window.FullScreen : Window.Windowed
            }
        }
        // The grid's games: the section shown, sorted, narrowed and searched as the List card and the search ask; a theme's
        // Games screen draws them (EmuSen_Qt.md §3.2, Browse).
        GameModel {
            id: games
            objectName: "gridGames"
            games: library.loading ? "" : library.gamesOf(screens.key)
            section: screens.key
            sectionName: library.headingOf(screens.key)
            startable: library.startable
            collections: library.collections
            onPassedBack: screens.back()
            // A section read anew starts at its first game, as the grid's own did.
            onCountChanged: games.focused = games.count > 0 ? 0 : -1
        }
        // The reference theme, EmuSen's own look, drawing every role the chosen theme does not (EmuSen_Qt.md §3.5).
        ThemeHost {
            id: referenceHost
            objectName: "reference"
            anchors.fill: parent
            visible: !program.gameShown
            folder: ReferenceTheme.folder
            coverStores: program.coverStores
            screens: screens
            library: library
            session: player.session
            clock: clock
            grid: games
            picture: player.picture
            controllers: pads.connected
            family: program.padFamily
            reducedMotion: program.reducedMotion
            swap: Kit.PadRouter.swapAcceptBack
            inputs: !program.splashShown
            homeWanted: !program.esdeLook && !(program.themeHost?.home ?? false)
            consolesWanted: !program.esdeLook && !(program.themeHost?.consoles ?? false)
            gamesWanted: !program.esdeLook && !(program.themeHost?.games ?? false)
            onPlayed: (path) => program.play(path)
            onResumed: screens.resume()
            onBacked: screens.shown === Screens.Consoles ? screens.back() : games.back()
            onFocused: (game) => {
                program.inHand = game
                // Home's games are in hand too, and leave the grid's focus where it was.
                if (screens.shown !== Screens.Home && games.rowOf(game) >= 0)
                    games.focused = games.rowOf(game)
            }
        }
        // The chosen EmuSen theme's screens over the reference's, each where the theme provides it and has not failed; the host and
        // its engine are made only while a theme other than the reference is chosen (EmuSen_Qt.md §3.5.2, §9.3.5).
        Loader {
            id: chosenTheme
            objectName: "chosenTheme"
            anchors.fill: parent
            active: program.theme.length > 0 && program.theme !== ReferenceTheme.folder && !program.esdeLook
            // A Loader is a focus scope: it takes the focus while one of the theme's screens is to hold it.
            focus: program.themeHost?.takesFocus ?? false
            sourceComponent: ThemeHost {
                objectName: "theme"
                visible: !program.gameShown
                // Loaded once the host is whole, so its connections hear the new engine and feed it the models as they stand.
                folder: chosenTheme.status === Loader.Ready ? program.theme : ""
                coverStores: program.coverStores
                screens: screens
                library: library
                session: player.session
                clock: clock
                grid: games
                picture: player.picture
                controllers: pads.connected
                family: program.padFamily
                reducedMotion: program.reducedMotion
                swap: Kit.PadRouter.swapAcceptBack
                inputs: !program.splashShown
                splashShown: program.splashShown
                splashLayer: splashLayer
                onPlayed: (path) => program.play(path)
                onResumed: screens.resume()
                onBacked: screens.shown === Screens.Consoles ? screens.back() : games.back()
                onFocused: (game) => {
                    program.inHand = game
                    // Home's games are in hand too, and leave the grid's focus where it was.
                    if (screens.shown !== Screens.Home && games.rowOf(game) >= 0)
                        games.focused = games.rowOf(game)
                }
            }
        }
        // The ES-DE theme's view in place of Home, Choose a console and the grid while it is the library's look, in window pixels;
        // kept under the cards and while a game plays, so the library comes back where it was left (EmuSen_Qt.md §4.15).
        Loader {
            id: esdeLoader
            objectName: "esde"
            // Under a card or another screen the view is drawn and nothing on it is reached: a backdrop to the gate, as the
            // own look's screens are under a card (ThemeHost's slots; EmuSen_Qt.md §7.9).
            property bool backdrop: screens.shown !== Screens.Esde
            anchors.fill: parent
            enabled: !esdeLoader.backdrop
            active: program.esdeWanted
            visible: program.esdeLook && !program.gameShown && !program.pauseShown
            focus: program.esdeLook && screens.shown === Screens.Esde && !program.splashShown
            sourceComponent: Esde.View {
                settings: library.esdeSettings
                shelves: library.esdeShelves
                media: library.esdeMedia
                presence: library.esdePresence
                family: program.padFamily
            }
        }
        // The view hears the pad through the router, which pumps while the view holds the focus, as a Frame's does.
        Timer {
            interval: 8
            repeat: true
            running: esdeLoader.activeFocus && Kit.PadRouter.clockRunning && Kit.PadRouter.pending
            onTriggered: Kit.PadRouter.pump()
        }
        // What the view asks of the program: a game played, a favourite turned, Start's Home card, Select's List card, a sound.
        Connections {
            target: program.esdeView?.scene ?? null
            function onPlayed(path) { program.play(path) }
            function onFavourite(path) { library.toggleFavourite(path) }
            function onMenu() { screens.home() }
            function onOptions() { screens.list() }
            function onCued(name) { cues.play(name, program.esdeView.scene.view) }
        }
        Connections {
            target: program.esdeView?.scene ?? null
            function onPlayed(path) { cues.launch() }
            function onFavourite(path) { cues.favourite() }
        }
        // The theme's navigation sounds, else the kit's, through Endymion's interface player; made only for an ES-DE theme.
        EsdeCues {
            id: cues
            objectName: "cues"
            platformLibrary: program.esdeWanted ? program.platformLibrary : ""
            sounds: program.esdeView?.scene.sounds ?? ""
            on: program.navigationSounds
        }
        // Sound volume, once Settings has read the settings; a step of its row is heard from the next sound.
        Binding {
            target: cues
            property: "volume"
            value: settingsModel.navigationVolume
            when: settingsModel.ready
        }
        // A sound of the theme's that did not open or play, said once the splash has gone.
        Timer {
            interval: Kit.Tokens.fade
            running: !program.splashShown && program.esdeLook && cues.problem.length > 0
            onTriggered: notices.raise(cues.problem, true, false)
        }
        // Why the ES-DE theme gave way to the own look, said once the splash has gone.
        Timer {
            interval: Kit.Tokens.fade
            running: !program.splashShown && program.esdeWanted && (program.esdeView?.scene.problem.length ?? 0) > 0
            onTriggered: program.said(program.esdeView.scene.problem, true, false)
        }
        SearchScreen {
            screens: screens
            games: games
            host: referenceHost
            padFamily: program.padFamily
            gameShown: program.gameShown
            splashShown: program.splashShown
        }
        ListCardScreen {
            screens: screens
            library: library
            games: games
            padFamily: program.padFamily
            esdeLook: program.esdeLook
            esdeView: program.esdeView
        }
        // Settings, from Home's Settings place and the Home card's Settings row (EmuSen_Qt.md §7.7); its Library ▸ Themes opens
        // the Themes tab.
        SettingsModel {
            id: settingsModel
            objectName: "settingsModel"
            platformLibrary: program.platformLibrary
            cores: library.cores
            open: screens.shown === Screens.Settings || program.pauseShown
            facts: library.loading ? "" : library.settingsFacts(player.session.running ? player.session.game : "")
            gamePads: pads.gamePads
            gameMode: program.gameMode
            onDoor: (name) => screens.act(name)
            // A shader's lever moved: the running game's picture follows it at once (EmuSen_Qt.md §1.8.10).
            onLever: (system, id, value) => player.session.setLever(system, id, value)
            onOpenFolder: (path) => folders.open(path)
            onChooseFolder: (key, start) => folders.choose(key, start)
            // What Settings saved reaches the library's read as it closes: the media viewer plays at the Volume saved (EmuSen_Qt.md §4.14),
            // and a suspended game's picture is drawn as its display settings now ask (§1.8.10).
            onSaved: {
                library.refresh()
                player.session.readPicture()
            }
        }
        // The program's folder actions (EmuSen_Qt.md §7.7.E): one shown in the system's file manager, one chosen in the
        // system's own folder dialog (through the desktop portal where there is one), whose answer the row's model saves.
        Folders {
            id: folders
            objectName: "folders"
            onDialogAsked: folderDialog.open()
            onChosen: (key, path) => settingsModel.setFolder(key, path)
        }
        FolderDialog {
            id: folderDialog
            currentFolder: folders.dialogFolder
            onAccepted: folders.accepted(folderDialog.selectedFolder)
            onRejected: folders.rejected()
        }
        Settings {
            objectName: "settings"
            visible: screens.shown === Screens.Settings
            settings: settingsModel
            padFamily: program.padFamily
            inputs: visible && !program.splashShown
            onBacked: screens.back()
            onActed: (key) => screens.act(key)
        }
        ThemesScreen {
            objectName: "themes"
            screens: screens
            library: library
            platformLibrary: program.platformLibrary
            padFamily: program.padFamily
            splashShown: program.splashShown
            onUsed: (folder) => program.theme = folder
            // The start's choice of an ES-DE theme is let go once another look is stored.
            onRestyled: (esde) => program.esdeChosen = program.esdeChosen && esde
        }
        // Cores on the tool screen, from Home's place (EmuSen_Interface.md §15; EmuSen_Qt.md §7.7).
        ToolScreen {
            objectName: "cores"
            kind: ToolModel.Cores
            shownAs: Screens.Cores
            screens: screens
            platformLibrary: program.platformLibrary
            cores: library.cores
            world: program.coresWorld
            padFamily: program.padFamily
            splashShown: program.splashShown
            // Open Folder on a firmware page, through the program's one action for it (EmuSen_Qt.md §7.7.E).
            onOpenFolder: (path) => folders.open(path)
            onSaid: (words) => program.said(words, true, false)
        }
        // A tool of jobs on the same tool screen: Library Manager, Compressed Discs or Packages, or the Scraper, as Screens names it (tools/jobs.rs, tools/scraper.rs).
        ToolScreen {
            objectName: "jobs"
            kind: ToolModel.Jobs
            shownAs: Screens.Tool
            jobs: screens.tool
            screens: screens
            platformLibrary: program.platformLibrary
            cores: library.cores
            scraperTest: program.scraperTest
            padFamily: program.padFamily
            splashShown: program.splashShown
            // The Scraper's run ended: the library shows what it fetched.
            onLibraryChanged: library.refresh()
        }
        PageScreen {
            objectName: "gamePage"
            screens: screens
            library: library
            platformLibrary: program.platformLibrary
            inHand: program.inHand
            scraperTest: program.scraperTest
            padFamily: program.padFamily
            splashShown: program.splashShown
            onPlay: (path) => program.play(path)
            onPlayFrom: (path, state) => program.playFrom(path, state)
            onOpenFolder: (path) => folders.open(path)
            onSaid: (words) => program.said(words, true, false)
        }
        HomeCardScreen {
            screens: screens
            library: library
            session: player.session
            clock: clock
            gameMode: program.gameMode
            padFamily: program.padFamily
        }
        PauseScreen {
            screens: screens
            library: library
            session: player.session
            settings: settingsModel
            padFamily: program.padFamily
            pauseShown: program.pauseShown
            gameShown: program.gameShown
        }
        ResumeQuestion {
            session: player.session
            padFamily: program.padFamily
        }
        LaunchQuestion {
            launch: launch
            padFamily: program.padFamily
        }
        // The session's notices (saved, loaded, the screenshot, what failed), at the top right over everything; over running
        // gameplay they are the game's own, so they show (EmuSen_Interface.md §18.6).
        Kit.NoticeModel {
            id: notices
            objectName: "notices"
            gameRunning: program.gameShown
        }
        // The theme the settings choose that cannot be shown, said once, after the splash has faded.
        Timer {
            interval: Kit.Tokens.fade
            running: !program.splashShown && program.themeWords.length > 0
            onTriggered: program.said(program.themeWords, true, false)
        }
        Connections {
            target: player.session
            function onNoticeChanged() { program.said(player.session.notice, player.session.noticeAttention, true) }
            // Exit to Library's key over the library: the suspended game again, as Continue plays it.
            function onGameAsked() { screens.resume() }
            // A game stopped: how long it ran is its play time, up to Play time counted up to.
            function onPlayed(path, seconds) { library.played(path, seconds, settingsModel.playHours) }
        }
        // What failed, from every model that can fail, said where the player is: a game that does not open (over the game),
        // the library unread, a setting, a game's page's change, or Cores; a cleared problem raises nothing (EmuSen_Qt.md §7.9).
        Connections {
            target: player.session
            function onProblemChanged() { program.said(player.session.problem, true, true) }
            function onOpenFailed() { screens.failed() }
        }
        Connections {
            target: library
            function onProblemChanged() { program.said(library.problem, true, false) }
        }
        Connections {
            target: settingsModel
            function onProblemChanged() { program.said(settingsModel.problem, true, false) }
        }
        // The pads unread: opening them or a tick failed, said once until they read again (EmuSen_Qt.md §7.7.H5).
        Connections {
            target: pads
            function onProblemChanged() { program.said(pads.problem, true, false) }
        }
        NoticeScreen {
            notices: notices
        }
        Kit.Moonrise {
            id: moonrise
            objectName: "moonrise"
            readonly property bool done: moonrise.finished && !library.loading && program.lookSettled
            anchors.fill: parent
            visible: program.splash && moonrise.opacity > 0
            opacity: moonrise.done ? 0 : 1
            // The clock runs from the window's first frame; the worlds, read in milliseconds, are in place long before the row appears.
            running: program.splash
            focus: program.splashShown
            title: Words.title
            caption: library.splashCaption
            worlds: library.splashWorlds
            Behavior on opacity {
                NumberAnimation {
                    duration: Kit.Tokens.fade
                }
            }
            Keys.onPressed: (event) => event.accepted = moonrise.model.takeKey(event.key, event.modifiers)
            TapHandler {
                enabled: program.splashShown
                acceptedButtons: Qt.AllButtons
                onTapped: moonrise.model.tick(moonrise.model.length)
            }
            Connections {
                target: Kit.PadRouter
                enabled: moonrise.activeFocus
                function onPressed(button) { moonrise.model.tick(moonrise.model.length) }
            }
            Timer {
                interval: 8
                repeat: true
                running: moonrise.activeFocus && Kit.PadRouter.clockRunning && Kit.PadRouter.pending
                onTriggered: Kit.PadRouter.pump()
            }
        }
        // A theme's own splash, over Moonrise and faded with it; Moonrise keeps the clock, the skip and the fade (EmuSen_Interface.md §22).
        Item {
            id: splashLayer
            objectName: "splashLayer"
            anchors.fill: parent
            visible: (program.themeHost?.splash ?? false) && program.splash && moonrise.opacity > 0
            opacity: moonrise.opacity
        }
    }
}
