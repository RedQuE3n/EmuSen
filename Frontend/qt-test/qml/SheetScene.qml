import QtQuick
import EmuSen.Kit as Kit

// A fixture: a question over a big-screen frame, opened by F1, for the sheet rule and the question form (EmuSen_Kit.md §7.9, §7.24), in the words of Mistress's Cores plan.
Window {
    id: scene
    objectName: "scene"
    width: 1280
    height: 800
    color: "#000000"
    FontLoader { id: barlowregular; source: "qrc:/emusen/fonts/Barlow-Regular.ttf" }
    FontLoader { id: barlowcondensedregular; source: "qrc:/emusen/fonts/BarlowCondensed-Regular.ttf" }
    property int chosen: -2
    Kit.Frame {
        id: frame
        anchors.fill: parent
        form: Kit.Forms.BigScreen
        focus: true
        Kit.Question {
            id: question
            objectName: "question"
            controller: frame.controller
            title: "Apply 2 changes?"
            message: "Download and install 1 core from buildbot.libretro.com: LRPS2."
            sentences: ["Update Beetle PSX; the build installed now is kept for Roll Back.", "Each core is checked before it is kept, and nothing downloaded runs until you approve it."]
            choices: ["No", "Yes"]
            backIndex: 0
            titleFont: Qt.font({ family: barlowcondensedregular.font.family, pixelSize: Kit.Tokens.fontTitle })
            bodyFont: Qt.font({ family: barlowregular.font.family, pixelSize: Kit.Tokens.fontBody })
            rowFont: Qt.font({ family: barlowregular.font.family, pixelSize: Kit.Tokens.fontRow })
            help: ListModel {
                ListElement { button: 4; label: "Choose" }
                ListElement { button: 5; label: "No" }
            }
            onChosen: (index) => scene.chosen = index
        }
        Shortcut {
            sequence: "F1"
            onActivated: question.open()
        }
    }
}
