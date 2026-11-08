import QtQuick
import EmuSen.Kit as Kit

// A fixture: a question over a big-screen frame, opened by F1, for the sheet rule (EmuSen_Kit.md §7.9).
Window {
    id: scene
    objectName: "scene"
    width: 1280
    height: 800
    property int chosen: -1
    Kit.Frame {
        id: frame
        anchors.fill: parent
        form: Kit.Forms.BigScreen
        focus: true
        Kit.Question {
            id: question
            objectName: "question"
            controller: frame.controller
            title: "Leave the game?"
            message: "Unsaved progress is lost."
            choices: ["Stay", "Leave"]
            onChosen: (index) => scene.chosen = index
        }
        Shortcut {
            sequence: "F1"
            onActivated: question.open()
        }
    }
}
