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
    // The EmuSen theme's folder, empty for the reference, and the folders its pictures may come from (EmuSen_Qt.md §3.5).
    property string theme
    property list<string> coverStores
    // The pads the reader opens, by SDL id, where a test names its own; empty for every pad.
    property string onlyPads
    // A file holding Cores' world in place of the back end's, as the recordings script theirs; empty for the back end.
    property string coresWorld
    // The splash keeps the focus, and every press, until it has faded, so the press that skipped it is never a screen's.
    readonly property bool splashShown: program.splash && (!moonrise.done || moonrise.opacity > 0)
    // The own look's screens draw a generic pad's glyphs as the Xbox pad's lettered ones, as Mistress does.
    readonly property int padFamily: pads.family === Kit.Families.Generic ? Kit.Families.Xbox : pads.family
    readonly property bool gameShown: screens.shown === Screens.Game
    readonly property bool pauseShown: screens.shown === Screens.Pause
    // The chosen theme's host while one is made.
    readonly property ThemeHost themeHost: chosenTheme.item as ThemeHost
    // Why the reference is shown in place of the theme the settings choose, said once the splash has gone.
    property string themeWords
    // An ES-DE theme is the library's look, as the start read the settings, and its navigation sounds are on (EmuSen_Qt.md §4.9).
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
        found: games.count > 0
        themed: program.esdeLook
        onQuit: Qt.quit()
        // The Home card's Continue: the suspended game resumes as its screen comes back.
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
        // A theme's screens hear the pad through their own engine's router (EmuSen_Qt.md §3.5.2).
        onButton: (button, down) => program.themeHost?.holdsFocus ? program.themeHost.feed.push(button, down) : referenceHost.holdsFocus ? referenceHost.feed.push(button, down) : Kit.PadRouter.push(button, down)
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
        guideChords: program.guideChords
        focus: program.gameShown && !player.session.resumeAsked
        feed: program.feed
        platformLibrary: program.platformLibrary
        cores: library.cores
        pads: pads.gamePads
        onMenu: {
            player.session.pause()
            screens.menu()
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
        inputs: !program.splashShown
        homeWanted: !program.esdeLook && !(program.themeHost?.home ?? false)
        consolesWanted: !program.esdeLook && !(program.themeHost?.consoles ?? false)
        gamesWanted: !program.esdeLook && !(program.themeHost?.games ?? false)
        onPlayed: (path) => program.play(path)
        onResumed: {
            player.session.resume()
            screens.play()
        }
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
            inputs: !program.splashShown
            splashShown: program.splashShown
            splashLayer: splashLayer
            onPlayed: (path) => program.play(path)
            onResumed: {
                player.session.resume()
                screens.play()
            }
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
    // kept under the cards and while a game plays, so the library comes back where it was left (EmuSen_Qt.md §4.9).
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
    // The grid's search: its words, which the keyboard types and the grid's model narrows by, the field over the one row of
    // covers and the keyboard docked beneath, over the theme's Games screen (EmuSen_Interface.md §18.1; EmuSen_Qt.md §7.6).
    Kit.KeyboardModel {
        id: typist
        objectName: "typist"
        onTextChanged: games.search = typist.text
        onClosed: screens.back()
    }
    Connections {
        target: games
        function onSearchChanged() { typist.setText(games.search, games.search.length) }
    }
    Connections {
        target: referenceHost
        function onSearchingChanged() { typist.setText(games.search, games.search.length) }
    }
    Kit.NormalizedCanvas {
        id: searchCanvas
        objectName: "searchCanvas"
        visible: !program.gameShown && referenceHost.searching
        Kit.Frame {
            id: searchFrame
            anchors.fill: parent
            surfaceDrawn: false
            padFamily: program.padFamily
            helpFont: Qt.font({ family: searchFaces.regular, pixelSize: 20 })
            helpSpacing: 40
            helpGlyphGap: 10
            // Its help stands while the keyboard holds the focus; on the covers the theme's own does.
            helpShown: screens.shown === Screens.Search
            focus: !program.splashShown && screens.shown === Screens.Search
            controller.navigationShown: false
            help: HelpModel {
                screen: HelpModel.Search
            }
            // The keyboard's own actions, Y Space and X Done, and the screen's, the shoulders to the covers; its close is the screen's way out.
            Connections {
                target: searchFrame.controller
                function onActionInvoked(key) { typist.act(key) }
            }
            Connections {
                target: searchFrame.controller
                function onActionInvoked(key) { screens.act(key) }
            }
            Item {
                x: -searchFrame.safeX
                y: -searchFrame.safeY
                width: searchCanvas.width
                height: searchCanvas.height
                // The search's field over the one row of covers, never focused: the keyboard types into it (LunaP's GallerySearch).
                TextField {
                    objectName: "GallerySearch"
                    x: 64
                    y: 100
                    width: 1152
                    height: 44
                    focusPolicy: Qt.NoFocus
                    activeFocusOnTab: false
                    readOnly: true
                    leftPadding: 16
                    rightPadding: 16
                    text: typist.text
                    cursorPosition: typist.caret
                    color: Kit.Tokens.textStrong
                    font.family: searchFaces.regular
                    font.pixelSize: 22
                    Accessible.name: Words.search
                }
                Kit.Keyboard {
                    id: keyboard
                    objectName: "GalleryKeyboard"
                    model: typist
                    y: 398
                    width: searchCanvas.width
                    focus: true
                    labelFont: Qt.font({ family: searchFaces.regular, pixelSize: Kit.Tokens.fontBody })
                    onActiveFocusChanged: if (referenceHost.searching && keyboard.activeFocus !== referenceHost.typing) screens.keysFocused(keyboard.activeFocus)
                }
            }
        }
        Faces {
            id: searchFaces
        }
    }
    // The List card over the grid (EmuSen_Interface.md §18.1): its actions are the grid's model's, and its jumps move the
    // focus the theme's Games screen follows.
    Kit.NormalizedCanvas {
        id: listCardCanvas
        objectName: "listCardCanvas"
        readonly property alias slide: listCard.slide
        visible: listCard.open || listCard.leaving
        CardModel {
            id: listCardModel
            objectName: "listCardModel"
            card: CardModel.List
            shown: screens.shown === Screens.List
            facts: program.esdeLook ? library.esdeListFacts(program.esdeView?.scene.shelf ?? "", program.esdeView?.scene.selected ?? 0) : games.listFacts
            // Under an ES-DE theme the own grid is hidden and acted on by nothing; the theme's gamelist is acted on below.
            onActed: (key) => { if (!program.esdeLook) games.focused = games.listAct(key) }
            onStayed: (key) => { if (!program.esdeLook) games.focused = games.listAct(key) }
            onClosed: screens.close()
        }
        Connections {
            target: listCardModel
            function onActed(key) { screens.act(key) }
        }
        // Under an ES-DE theme the card acts on the shown gamelist, and the view selects the entry it answers.
        Connections {
            target: listCardModel
            enabled: program.esdeLook
            function onActed(key) { program.esdeView.scene.select(library.esdeListAct(program.esdeView.scene.shelf, key, program.esdeView.scene.selected)) }
            function onStayed(key) { program.esdeView.scene.select(library.esdeListAct(program.esdeView.scene.shelf, key, program.esdeView.scene.selected)) }
        }
        MenuCard {
            id: listCard
            anchors.fill: parent
            open: screens.shown === Screens.List
            focus: open
            padFamily: program.padFamily
            rows: listCardModel
            title: listCardModel.title
            breadcrumb: listCardModel.breadcrumb
            footer: listCardModel.footer
            current: listCardModel.current
            paneLines: listCardModel.paneLines
            paneMuted: listCardModel.paneMuted
            panePicture: listCardModel.panePicture
            paneCaption: listCardModel.paneCaption
            help: HelpModel {
                screen: listCardModel.help
            }
            onChosen: (index) => listCardModel.choose(index)
            onStepped: (index, by, held) => listCardModel.step(index, by)
            onFocused: (index) => listCardModel.select(index)
            onBacked: listCardModel.back()
            onActed: (key) => screens.act(key)
        }
    }
    // Settings, from Home's Settings place and the Home card's Settings row (EmuSen_Qt.md §7.7); its Library ▸ Themes opens
    // the Themes tab.
    SettingsModel {
        id: settingsModel
        objectName: "settingsModel"
        platformLibrary: program.platformLibrary
        cores: library.cores
        open: screens.shown === Screens.Settings
        facts: library.loading ? "" : library.settingsFacts(player.session.running ? player.session.game : "")
        onDoor: (name) => screens.act(name)
        onOpenFolder: (path) => folders.open(path)
        onChooseFolder: (key, start) => folders.choose(key, start)
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
    // The Themes tab, under Settings ▸ Library ▸ Themes (EmuSen_Qt.md §3.8): Use stores a theme's id and the chosen
    // theme's host switches at once.
    ThemesModel {
        id: themesModel
        objectName: "themesModel"
        platformLibrary: program.platformLibrary
        reference: ReferenceTheme.manifest
        referenceFolder: ReferenceTheme.folder
        open: screens.shown === Screens.Themes
        onUsed: (folder) => program.theme = folder
    }
    Themes {
        objectName: "themes"
        visible: screens.shown === Screens.Themes
        themes: themesModel
        padFamily: program.padFamily
        inputs: visible && !program.splashShown
        onBacked: screens.back()
        onActed: (key) => screens.act(key)
    }
    // Cores on the tool screen, from Home's place (EmuSen_Interface.md §15; EmuSen_Qt.md §7.7).
    ToolModel {
        id: coresModel
        objectName: "coresModel"
        tool: ToolModel.Cores
        platformLibrary: program.platformLibrary
        cores: library.cores
        world: program.coresWorld
        open: screens.shown === Screens.Cores
        // Open Folder on a firmware page, through the program's one action for it (EmuSen_Qt.md §7.7.E).
        onOpenFolder: (path) => folders.open(path)
    }
    Cores {
        objectName: "cores"
        visible: screens.shown === Screens.Cores
        tool: coresModel
        padFamily: program.padFamily
        inputs: visible && !program.splashShown
        onBacked: screens.back()
    }
    // A game's page, from X on Home and the grid, a theme's openPage and the Home card's Selected game (EmuSen_Qt.md §7.8);
    // its doors open later pieces' pages.
    GamePageModel {
        id: pageModel
        objectName: "pageModel"
        platformLibrary: program.platformLibrary
        cores: library.cores
        game: screens.shown === Screens.Page ? library.gameFile(program.inHand) : ""
        onPlayed: (path, state) => state.length > 0 ? program.playFrom(path, state) : program.play(path)
        onHidden: screens.back()
        onOpenFolder: (path) => folders.open(path)
    }
    Connections {
        target: pageModel
        function onHidden() { library.refresh() }
    }
    GamePage {
        objectName: "gamePage"
        visible: screens.shown === Screens.Page
        page: pageModel
        padFamily: program.padFamily
        inputs: visible && !program.splashShown
        onBacked: screens.back()
        onActed: (key) => screens.act(key)
    }
    // The Home card over Home, opened at Power by its place (EmuSen_Interface.md §2.3; EmuSen_BigPicture.md §51.2).
    Kit.NormalizedCanvas {
        id: homeCardCanvas
        objectName: "homeCardCanvas"
        readonly property alias slide: homeCard.slide
        visible: homeCard.open || homeCard.leaving
        CardModel {
            id: homeCardModel
            objectName: "homeCardModel"
            card: CardModel.Home
            start: screens.page
            shown: screens.shown === Screens.HomeCard
            // Every fact is an argument, so the card follows the library's consoles and tools as they are read.
            facts: library.homeCardFacts(player.session.running ? player.session.game : "", player.session.running ? player.session.console : "", clock.paused, program.gameMode, library.startable, library.toolSummaries)
            onActed: (key) => screens.act(key)
            onClosed: screens.close()
        }
        MenuCard {
            id: homeCard
            anchors.fill: parent
            open: screens.shown === Screens.HomeCard
            focus: open
            padFamily: program.padFamily
            rows: homeCardModel
            title: homeCardModel.title
            breadcrumb: homeCardModel.breadcrumb
            footer: homeCardModel.footer
            current: homeCardModel.current
            paneLines: homeCardModel.paneLines
            paneMuted: homeCardModel.paneMuted
            panePicture: homeCardModel.panePicture
            paneCaption: homeCardModel.paneCaption
            help: HelpModel {
                screen: homeCardModel.help
            }
            onChosen: (index) => homeCardModel.choose(index)
            onStepped: (index, by, held) => homeCardModel.step(index, by)
            onFocused: (index) => homeCardModel.select(index)
            onBacked: homeCardModel.back()
            onActed: (key) => screens.act(key)
        }
    }
    // The Pause Card, opened by the chord over the game; the game resumes once the card has slid away (EmuSen_Interface.md §20.6).
    Kit.NormalizedCanvas {
        id: cardCanvas
        objectName: "cardCanvas"
        readonly property alias slide: card.slide
        visible: card.open || card.leaving
        PauseCard {
            id: card
            anchors.fill: parent
            library: library
            session: player.session
            padFamily: program.padFamily
            open: program.pauseShown
            focus: program.pauseShown
            onResumed: screens.play()
            onBackToLibrary: screens.library()
            onQuitGame: player.session.close()
            onQuitAll: screens.quit()
            onGone: if (program.gameShown) player.session.resume()
        }
    }
    // The Resume question at a game's start, over the game held unplayed until it is answered (EmuSen_Interface.md §18.3).
    Kit.NormalizedCanvas {
        id: resumeCanvas
        objectName: "resumeCanvas"
        visible: player.session.resumeAsked
        QuestionModel {
            id: resumeWords
            subject: player.session.resumeSubject
        }
        Kit.Frame {
            id: resumeFrame
            anchors.fill: parent
            form: Kit.Forms.BigScreen
            surfaceDrawn: false
            helpShown: false
            focus: player.session.resumeAsked
            controller.navigationShown: false
            Kit.Question {
                id: resumeQuestion
                objectName: "resumeQuestion"
                controller: resumeFrame.controller
                title: resumeWords.title
                sentences: resumeWords.sentences
                caption: resumeWords.caption
                choices: resumeWords.choices
                backIndex: resumeWords.backIndex
                padFamily: program.padFamily
                titleFont: Qt.font({ family: resumeFaces.condensedRegular, pixelSize: Kit.Tokens.fontTitle })
                bodyFont: Qt.font({ family: resumeFaces.regular, pixelSize: Kit.Tokens.fontBody })
                captionFont: Qt.font({ family: resumeFaces.regular, pixelSize: Kit.Tokens.fontCaption })
                rowFont: Qt.font({ family: resumeFaces.regular, pixelSize: Kit.Tokens.fontRow })
                helpFont: Qt.font({ family: resumeFaces.regular, pixelSize: 20 })
                help: HelpModel {
                    screen: HelpModel.QuestionBack
                }
                onChosen: (index) => player.session.answerResume(index)
            }
            // Opened each time it is asked; a Binding, since B closing the popup would undo a plain one.
            Binding {
                resumeQuestion.visible: player.session.resumeAsked
            }
        }
        Faces {
            id: resumeFaces
        }
    }
    // The questions before a game opens: its core's approval, then the firmware it needs (EmuSen_Qt.md §7.7).
    Kit.NormalizedCanvas {
        id: launchCanvas
        objectName: "launchCanvas"
        visible: launch.asking
        Kit.Frame {
            id: launchFrame
            anchors.fill: parent
            form: Kit.Forms.BigScreen
            surfaceDrawn: false
            helpShown: false
            focus: launch.asking
            controller.navigationShown: false
            Kit.Question {
                id: launchQuestion
                objectName: "launchQuestion"
                controller: launchFrame.controller
                title: launch.title
                sentences: launch.sentences
                caption: launch.caption
                choices: launch.choices
                backIndex: launch.backIndex
                padFamily: program.padFamily
                titleFont: Qt.font({ family: resumeFaces.condensedRegular, pixelSize: Kit.Tokens.fontTitle })
                bodyFont: Qt.font({ family: resumeFaces.regular, pixelSize: Kit.Tokens.fontBody })
                captionFont: Qt.font({ family: resumeFaces.regular, pixelSize: Kit.Tokens.fontCaption })
                rowFont: Qt.font({ family: resumeFaces.regular, pixelSize: Kit.Tokens.fontRow })
                helpFont: Qt.font({ family: resumeFaces.regular, pixelSize: 20 })
                help: HelpModel {
                    screen: launch.help
                }
                onChosen: (index) => launch.answer(index)
            }
            Binding {
                launchQuestion.visible: launch.asking
            }
        }
    }
    // The session's notices (saved, loaded, the screenshot, what failed), at the top right over everything; over running
    // gameplay they are the game's own, so they show (EmuSen_Interface.md §18.6).
    Kit.NoticeModel {
        id: notices
        objectName: "notices"
        gameRunning: program.gameShown
    }
    // Every notice is raised here, and words that say nothing (a model's cleared problem) raise none (EmuSen_Qt.md §7.9).
    signal said(string words, bool attention, bool overGame)
    onSaid: (words, attention, overGame) => { if (words.length > 0) notices.raise(words, attention, overGame) }
    // The theme the settings choose that cannot be shown, said once, after the splash has faded.
    Timer {
        interval: Kit.Tokens.fade
        running: !program.splashShown && program.themeWords.length > 0
        onTriggered: program.said(program.themeWords, true, false)
    }
    Connections {
        target: player.session
        function onNoticeChanged() { program.said(player.session.notice, player.session.noticeAttention, true) }
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
    Connections {
        target: pageModel
        function onProblemChanged() { program.said(pageModel.problem, true, false) }
    }
    Connections {
        target: coresModel
        function onProblemChanged() { program.said(coresModel.problem, true, false) }
    }
    Kit.NormalizedCanvas {
        id: noticeCanvas
        objectName: "noticeCanvas"
        Kit.NoticeLayer {
            model: notices
            width: noticeCanvas.width
            height: noticeCanvas.height
            labelFont: Qt.font({ family: resumeFaces.regular, pixelSize: Kit.Tokens.fontBody })
        }
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
