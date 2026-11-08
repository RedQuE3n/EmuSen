pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models
import EmuSen.Endymion

// emusen-qt's window: the screens on the big screen's canvas, Moonrise over them at the start until it ends and the library is read, the pads through Endymion (EmuSen_Qt.md §7, EmuSen_Interface.md §22).
Window {
    id: program
    objectName: "program"
    visible: true
    width: 1280
    height: 800
    color: Kit.Tokens.voidFill
    property string platformLibrary
    property bool splash: true
    property bool reducedMotion: false
    readonly property bool splashShown: program.splash && !moonrise.done
    // The own look's screens draw a generic pad's glyphs as the Xbox pad's lettered ones, as Mistress does.
    readonly property int padFamily: pads.family === Kit.Families.Generic ? Kit.Families.Xbox : pads.family
    readonly property int shownIndex: library.indexOf(screens.key)
    Screens {
        id: screens
        objectName: "screens"
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
        onButton: (button, down) => Kit.PadRouter.push(button, down)
    }
    Kit.NormalizedCanvas {
        id: canvas
        objectName: "canvas"
        Home {
            id: home
            anchors.fill: parent
            library: library
            padFamily: program.padFamily
            controllers: pads.connected
            visible: screens.shown === Screens.Home
            focus: !program.splashShown && visible
            onPlaceChosen: (index) => screens.place(index)
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
