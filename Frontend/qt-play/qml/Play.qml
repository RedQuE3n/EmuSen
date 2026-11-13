import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Endymion

// emusen-qt-play's window: one game through Game, the pads' one reader, and the chord pausing and resuming the game until the Pause Card exists (EmuSen_Qt.md §1.8).
Window {
    id: root
    objectName: "play"
    width: 1280
    height: 800
    visible: false
    // Black before the first frame, as the game's bars are.
    color: "#000000"
    property double feed: 0
    property string platformLibrary
    property string game
    // A console started with nothing in it, by its key, as Start Console starts one.
    property string consoleKey
    onConsoleKeyChanged: if (root.consoleKey !== "") player.session.openConsole(root.consoleKey)
    property string cores
    property string engine
    property string onlyPads

    // The pads' one reader: the interface's presses into the router, and the game's side handed to the session (EmuSen_Kit.md §7.17).
    PadReader {
        id: pads
        objectName: "pads"
        platformLibrary: root.platformLibrary
        onlyPads: root.onlyPads
        onButton: (button, down) => Kit.PadRouter.push(button, down)
    }
    Game {
        id: player
        anchors.fill: parent
        feed: root.feed
        platformLibrary: root.platformLibrary
        cores: root.cores
        engine: root.engine
        pads: pads.gamePads
        game: root.game
        onMenu: player.session.togglePause()
    }
}
