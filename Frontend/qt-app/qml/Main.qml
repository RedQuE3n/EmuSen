pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models
import EmuSen.Endymion
import QtQuick.Controls
import EmuSen.Play as Play
import EmuSen.App

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
    // A game chosen: recorded as started, opened (resumed where it is the one suspended) and shown.
    signal play(string path)
    onPlay: (path) => {
        library.started(path)
        player.game = path
        player.session.resume()
        screens.play()
    }
    Screens {
        id: screens
        objectName: "screens"
        gameRunning: player.session.running
        found: games.count > 0
        onQuit: Qt.quit()
    }
    Binding {
        target: Kit.Tokens
        property: "reducedMotion"
        value: program.reducedMotion
    }
    LibraryModel {
        id: library
        objectName: "library"
        platformLibrary: program.platformLibrary
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
        startConsole: library.startsEmpty
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
        homeWanted: !(program.themeHost?.home ?? false)
        consolesWanted: !(program.themeHost?.consoles ?? false)
        gamesWanted: !(program.themeHost?.games ?? false)
        onPlayed: (path) => program.play(path)
        onResumed: {
            player.session.resume()
            screens.play()
        }
        onBacked: screens.shown === Screens.Consoles ? screens.back() : games.back()
        onFocused: (game) => games.focused = games.rowOf(game)
    }
    // The chosen EmuSen theme's screens over the reference's, each where the theme provides it and has not failed; the host and
    // its engine are made only while a theme other than the reference is chosen (EmuSen_Qt.md §3.5.2, §9.3.5).
    Loader {
        id: chosenTheme
        objectName: "chosenTheme"
        anchors.fill: parent
        active: program.theme.length > 0 && program.theme !== ReferenceTheme.folder
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
            onFocused: (game) => games.focused = games.rowOf(game)
        }
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
            facts: games.listFacts
            onActed: (key) => games.focused = games.listAct(key)
            onStayed: (key) => games.focused = games.listAct(key)
            onClosed: screens.close()
        }
        Connections {
            target: listCardModel
            function onActed(key) { screens.act(key) }
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
                screen: HelpModel.Card
            }
            onChosen: (index) => listCardModel.choose(index)
            onStepped: (index, by, held) => listCardModel.step(index, by)
            onFocused: (index) => listCardModel.select(index)
            onBacked: listCardModel.back()
            onActed: (key) => screens.act(key)
        }
    }
    // The Themes tab, reached from the Home card's Settings row until Settings is built (EmuSen_Qt.md §3.8): Use stores a
    // theme's id and the chosen theme's host switches at once.
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
            facts: library.homeCardFacts(player.session.running ? player.session.game : "", clock.paused, program.gameMode)
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
                screen: HelpModel.Card
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
        onTriggered: notices.raise(program.themeWords, true, false)
    }
    Connections {
        target: player.session
        function onNoticeChanged() { notices.raise(player.session.notice, player.session.noticeAttention, true) }
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
        readonly property bool done: moonrise.finished && !library.loading
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
