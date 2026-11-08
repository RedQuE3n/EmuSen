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
    readonly property bool splashShown: program.splash && !moonrise.done
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
            padFamily: pads.family
            focus: !program.splashShown
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
        hues: library.splashHues
        Behavior on opacity {
            NumberAnimation {
                duration: Kit.Tokens.fade
            }
        }
        Keys.onPressed: (event) => moonrise.model.tick(moonrise.model.length)
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
