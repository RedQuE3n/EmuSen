pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models
import EmuSen.Endymion
import EmuSen.Play as Play

// emusen-qt's window: the screens on the big screen's canvas, the game under them, Moonrise over them at the start until it ends and the library is read, the pads through Endymion (EmuSen_Qt.md §7, EmuSen_Interface.md §22).
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
    // The pads the reader opens, by SDL id, where a test names its own; empty for every pad.
    property string onlyPads
    // The splash keeps the focus, and every press, until it has faded, so the press that skipped it is never a screen's.
    readonly property bool splashShown: program.splash && (!moonrise.done || moonrise.opacity > 0)
    // The own look's screens draw a generic pad's glyphs as the Xbox pad's lettered ones, as Mistress does.
    readonly property int padFamily: pads.family === Kit.Families.Generic ? Kit.Families.Xbox : pads.family
    readonly property int shownIndex: library.indexOf(screens.key)
    readonly property bool gameShown: screens.shown === Screens.Game
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
        onButton: (button, down) => Kit.PadRouter.push(button, down)
    }
    // The game, under the screens: in front on its own screen, and behind Home while it is suspended there, where Continue draws its picture.
    Play.Game {
        id: player
        objectName: "game"
        anchors.fill: parent
        visible: program.gameShown || (screens.shown === Screens.Home && player.session.running)
        inFront: program.gameShown
        focus: program.gameShown
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
            visible: screens.shown === Screens.Home
            focus: !program.splashShown && visible
            onPlaceChosen: (index) => screens.place(index)
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
            visible: screens.shown === Screens.Consoles
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
            visible: screens.shown === Screens.Grid
            focus: !program.splashShown && visible
            key: screens.key
            row: consoles.tiles.count > 0 ? consoles.tiles.itemAt(library.indexOf(screens.key)) : null
            previousRow: consoles.tiles.count > 1 && program.shownIndex >= 0 ? consoles.tiles.itemAt((program.shownIndex + consoles.tiles.count - 1) % consoles.tiles.count) : null
            nextRow: consoles.tiles.count > 1 && program.shownIndex >= 0 ? consoles.tiles.itemAt((program.shownIndex + 1) % consoles.tiles.count) : null
            onClosed: screens.back()
            onPlayed: (path) => program.play(path)
        }
    }
    Kit.Moonrise {
        id: moonrise
        objectName: "moonrise"
        readonly property bool done: moonrise.finished && !library.loading
        anchors.fill: parent
        visible: program.splash && moonrise.opacity > 0
        opacity: moonrise.done ? 0 : 1
        running: program.splash && library.splashPlanets > 0
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
            running: moonrise.activeFocus && Kit.PadRouter.clockRunning
            onTriggered: Kit.PadRouter.pump()
        }
    }
}
