import QtQuick
import EmuSen.Kit as Kit

// A card's row holding the focus beside places a resting pointer comes to without moving, for pointer.rs (EmuSen_Kit.md §7.108).
Window {
    id: scene
    objectName: "scene"
    visible: true
    width: 960
    height: 240
    color: Kit.Tokens.voidFill
    property int returns: 0
    Kit.FrameController {
        id: controller
    }
    FocusScope {
        objectName: "card"
        width: 200
        height: 40
        focus: true
        Item {
            objectName: "row"
            anchors.fill: parent
            focus: true
            Keys.onReturnPressed: scene.returns += 1
        }
    }
    Kit.Walkable {
        objectName: "near"
        y: 60
        width: 200
        height: 160
        controller: controller
    }
    // A card of its own controller, made under the pointer as Start makes the Home card.
    Loader {
        objectName: "late"
        x: 320
        width: 200
        height: 200
        active: false
        sourceComponent: Item {
            Kit.FrameController {
                id: own
            }
            Kit.Walkable {
                objectName: "arrived"
                width: 200
                height: 200
                controller: own
            }
        }
    }
    FocusScope {
        objectName: "under"
        x: 640
        width: 300
        height: 240
        enabled: false
        Kit.Walkable {
            objectName: "cell"
            width: 200
            height: 200
            controller: controller
        }
    }
    // A window made after every controller, so none started its watch, a walkable of the scene's controller in it.
    Loader {
        id: elsewhere
        objectName: "elsewhere"
        property bool shown: false
        readonly property bool awayFocused: item ? item.awayFocused : false
        readonly property Kit.FrameController walk: controller
        active: false
        sourceComponent: Window {
            objectName: "otherWindow"
            readonly property bool awayFocused: away.focus
            visible: true
            width: 300
            height: 200
            Kit.Walkable {
                id: away
                width: 200
                height: 200
                visible: elsewhere.shown
                controller: elsewhere.walk
            }
        }
    }
}
