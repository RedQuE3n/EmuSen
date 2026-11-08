pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models
import EmuSen.Endymion
import EmuSen.Play as Play

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
    readonly property int shownIndex: library.indexOf(screens.key)
    readonly property bool gameShown: screens.shown === Screens.Game
    readonly property bool pauseShown: screens.shown === Screens.Pause
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
        found: grid.games.count > 0
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
        onButton: (button, down) => Kit.PadRouter.push(button, down)
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
    Kit.NormalizedCanvas {
        id: canvas
        objectName: "canvas"
        visible: !program.gameShown
        Home {
            id: home
            anchors.fill: parent
            library: library
            padFamily: program.padFamily
            controllers: pads.connected
            session: player.session
            gamePicture: player.picture
            pixelScale: canvas.unit
            visible: (screens.shown === Screens.Home || screens.shown === Screens.HomeCard) && !themeHost.home
            focus: !program.splashShown && screens.shown === Screens.Home && !themeHost.home
            startAgain: screens.homeAgain
            // Under the Home card Home is drawn dimmed and nothing on it is reached: a backdrop to the gate.
            property bool backdrop: screens.shown === Screens.HomeCard
            enabled: !backdrop
            onPlaceChosen: (index) => screens.place(index)
            onActed: (key) => screens.act(key)
            onPlayed: (path) => program.play(path)
            onResumed: {
                player.session.resume()
                screens.play()
            }
        }
        Consoles {
            id: consoles
            anchors.fill: parent
            library: library
            padFamily: program.padFamily
            pixelScale: canvas.unit
            visible: screens.shown === Screens.Consoles && !themeHost.consoles
            focus: !program.splashShown && visible
            onClosed: screens.back()
            onOpened: (key) => screens.open(key)
            onActed: (key) => screens.act(key)
        }
        GameGrid {
            id: grid
            anchors.fill: parent
            library: library
            padFamily: program.padFamily
            pixelScale: canvas.unit
            visible: (screens.shown === Screens.Grid || screens.shown === Screens.List || grid.searching) && !themeHost.games
            focus: !program.splashShown && visible
            listOpen: screens.shown === Screens.List
            searching: screens.shown === Screens.Search || screens.shown === Screens.SearchGrid
            typing: screens.shown === Screens.Search
            key: screens.key
            row: consoles.tiles.count > 0 ? consoles.tiles.itemAt(library.indexOf(screens.key)) : null
            previousRow: consoles.tiles.count > 1 && program.shownIndex >= 0 ? consoles.tiles.itemAt((program.shownIndex + consoles.tiles.count - 1) % consoles.tiles.count) : null
            nextRow: consoles.tiles.count > 1 && program.shownIndex >= 0 ? consoles.tiles.itemAt((program.shownIndex + 1) % consoles.tiles.count) : null
            onClosed: screens.back()
            onPlayed: (path) => program.play(path)
            onActed: (key) => screens.act(key)
            onCardClosed: screens.close()
            onSearchClosed: screens.back()
            onFocusMoved: (keys) => screens.keysFocused(keys)
        }
    }
    // The chosen EmuSen theme's screens over the reference's, each where the theme provides it and has not failed (EmuSen_Qt.md §3.5).
    ThemeHost {
        id: themeHost
        objectName: "theme"
        anchors.fill: parent
        visible: !program.gameShown
        folder: program.theme
        coverStores: program.coverStores
        inputs: !program.splashShown
        homeShown: screens.shown === Screens.Home || screens.shown === Screens.HomeCard
        consolesShown: screens.shown === Screens.Consoles
        gamesShown: screens.shown === Screens.Grid || screens.shown === Screens.List || screens.shown === Screens.Search || screens.shown === Screens.SearchGrid
        splashShown: program.splashShown
        splashLayer: splashLayer
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
            facts: library.homeCardFacts(player.session.running ? player.session.game : "", home.pausedWords, program.gameMode)
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
        visible: themeHost.splash && program.splash && moonrise.opacity > 0
        opacity: moonrise.opacity
    }
}
