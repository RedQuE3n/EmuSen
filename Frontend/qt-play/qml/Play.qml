import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Endymion

// emusen-qt-play's window: one game through Game, the pads' one reader, and the chord pausing and resuming the game until the Pause Card exists (EmuSen_Qt.md §1.8).
Window {
    id: root
    objectName: "play"
    // graphics.json's WindowWidth, WindowHeight and WindowResizable, as the program's start read them (EmuSen_Qt.md §7.7.H7,
    // The window's size); 0 leaves the size to Qt.
    property int windowedWidth: 0
    property int windowedHeight: 0
    property bool windowResizable: true
    width: root.windowedWidth
    height: root.windowedHeight
    // WindowResizable off holds the window at the setting's size, Qt's way: its least and greatest sizes made equal.
    Binding { target: root; property: "minimumWidth"; value: root.windowedWidth; when: !root.windowResizable }
    Binding { target: root; property: "maximumWidth"; value: root.windowedWidth; when: !root.windowResizable }
    Binding { target: root; property: "minimumHeight"; value: root.windowedHeight; when: !root.windowResizable }
    Binding { target: root; property: "maximumHeight"; value: root.windowedHeight; when: !root.windowResizable }
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
