import QtQuick
import QtQuick.Controls
import EmuSen.Kit as Kit

// A fixture: Mistress's search as the docked keyboard's oracle draws it, a field at 64,100 and the band from 398 to the help bar, in canvas units (EmuSen_Kit.md §7.22).
Window {
    id: scene
    objectName: "scene"
    width: 1280
    height: 800
    color: "#000000"
    FontLoader { id: barlowregular; source: "qrc:/emusen/fonts/Barlow-Regular.ttf" }
    property string closings: ""
    Kit.KeyboardModel {
        id: typist
        objectName: "typist"
        onClosed: (kept) => scene.closings = scene.closings + (kept ? "kept " : "dropped ")
    }
    Kit.NormalizedCanvas {
        id: canvas
        Kit.Frame {
            id: frame
            width: canvas.width
            height: canvas.height
            form: Kit.Forms.BigScreen
            focus: true
            surfaceDrawn: false
            controller.navigationShown: false
            help: ListModel {
                property string actionLines: "space Search\ndone Screensaver"
                ListElement { button: 4; label: "Type" }
                ListElement { button: 5; label: "Delete" }
                ListElement { button: 13; label: "Space" }
                ListElement { button: 15; label: "Done" }
            }
            Connections {
                target: frame.controller
                function onActionInvoked(key) { typist.act(key) }
            }
            Item {
                x: -frame.safeX
                y: -frame.safeY
                width: canvas.width
                height: canvas.height
            TextField {
                id: field
                objectName: "field"
                x: 64
                y: 100
                width: 1152
                height: 44
                focusPolicy: Qt.NoFocus
                activeFocusOnTab: false
                text: typist.text
                cursorPosition: typist.caret
                onTextEdited: typist.setText(field.text, field.cursorPosition)
            }
            Kit.Keyboard {
                id: keyboard
                objectName: "keyboard"
                model: typist
                y: 398
                width: canvas.width
                focus: true
                labelFont: Qt.font({ family: barlowregular.font.family, pixelSize: Kit.Tokens.fontBody })
            }
            }
        }
    }
}
