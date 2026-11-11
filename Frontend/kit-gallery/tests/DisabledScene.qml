import QtQuick
import EmuSen.Kit as Kit

// A card holding the focus beside a screen disabled under it, whose walkable the pointer comes to rest on, for disabled.rs (EmuSen_Kit.md §7.107).
Window {
    id: scene
    objectName: "scene"
    visible: true
    width: 640
    height: 240
    color: Kit.Tokens.voidFill
    Kit.FrameController {
        id: controller
    }
    FocusScope {
        objectName: "under"
        x: 320
        width: 320
        height: 240
        enabled: false
        Kit.Walkable {
            objectName: "cell"
            width: 200
            height: 200
            controller: controller
        }
    }
    FocusScope {
        objectName: "card"
        width: 300
        height: 240
        focus: true
        Item {
            objectName: "row"
            width: 300
            height: 60
            focus: true
        }
    }
}
