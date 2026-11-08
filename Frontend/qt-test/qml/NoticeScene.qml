import QtQuick
import EmuSen.Kit as Kit

// A fixture: the notice layer over a big-screen canvas, raised by F2 (a game's own), F3 (any other) and F4 (gameplay on and off) (EmuSen_Kit.md §7.23).
Window {
    id: scene
    objectName: "scene"
    width: 1280
    height: 800
    color: "#000000"
    FontLoader { id: barlowregular; source: "qrc:/emusen/fonts/Barlow-Regular.ttf" }
    Kit.NoticeModel {
        id: notices
        objectName: "notices"
    }
    Kit.NormalizedCanvas {
        id: canvas
        Kit.NoticeLayer {
            objectName: "layer"
            model: notices
            width: canvas.width
            height: canvas.height
            labelFont: Qt.font({ family: barlowregular.font.family, pixelSize: Kit.Tokens.fontBody })
        }
    }
    Shortcut {
        sequence: "F2"
        onActivated: notices.raise("State saved to Slot 3", false, true)
    }
    Shortcut {
        sequence: "F3"
        onActivated: notices.raise("Core approval needed", true, false)
    }
    Shortcut {
        sequence: "F4"
        onActivated: notices.gameRunning = !notices.gameRunning
    }
}
