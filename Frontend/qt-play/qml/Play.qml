import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models
import EmuSen.Endymion

Window {
    id: root
    objectName: "play"
    width: 1280
    height: 800
    visible: false
    property double feed: 0
    property string platformLibrary
    property string game
    property string cores
    property string engine
    property string onlyPads
    readonly property bool gameOwnsPad: session.running && !session.paused

    // The pads' one reader: the interface's presses into the router, and the game's side handed to the session (EmuSen_Kit.md §7.17).
    PadReader {
        id: pads
        objectName: "pads"
        platformLibrary: root.platformLibrary
        onlyPads: root.onlyPads
        onButton: (button, down) => Kit.PadRouter.push(button, down)
    }

    // While the game owns the pad only the chord leaves the router; it reaches the program, where the Pause Card will open.
    Binding {
        target: Kit.PadRouter
        property: "chordOnly"
        value: root.gameOwnsPad
    }
    Connections {
        target: Kit.PadRouter
        function onMenuChord() { session.togglePause() }
    }
    Timer {
        interval: 8
        repeat: true
        running: Kit.PadRouter.clockRunning
        onTriggered: Kit.PadRouter.pump()
    }

    SessionModel {
        id: session
        objectName: "session"
        platformLibrary: root.platformLibrary
        feed: root.feed
        cores: root.cores
        engine: root.engine
        pads: pads.gamePads
        game: root.game
    }

    Kit.GamePicture {
        objectName: "picture"
        anchors.fill: parent
        feed: root.feed
        live: root.feed !== 0
    }
}
