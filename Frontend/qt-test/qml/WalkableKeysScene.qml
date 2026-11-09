import QtQuick
import EmuSen.Kit as Kit

// A fixture: three walkables, one plain, one whose own Keys.onPressed hears A and passes it on, one whose handler takes it, inside an item that counts every key reaching it (EmuSen_Kit.md §7.37).
Window {
    id: scene
    objectName: "scene"
    width: 1280
    height: 800
    property int plain: 0
    property int heard: 0
    property int passed: 0
    property int taken: 0
    property int leaked: 0
    Kit.Frame {
        id: frame
        anchors.fill: parent
        form: Kit.Forms.BigScreen
        focus: true
        Item {
            objectName: "outer"
            width: 300
            height: 300
            focus: true
            Keys.onPressed: scene.leaked += 1
            Column {
                Kit.Walkable {
                    objectName: "plain"
                    controller: frame.controller
                    width: 300
                    height: 60
                    focus: true
                    onActivated: scene.plain += 1
                }
                Kit.Walkable {
                    objectName: "passes"
                    controller: frame.controller
                    width: 300
                    height: 60
                    Keys.onPressed: (event) => {
                        scene.heard += event.key === Qt.Key_Return || event.key === Qt.Key_Enter ? 1 : 0
                        event.accepted = false
                    }
                    onActivated: scene.passed += 1
                }
                Kit.Walkable {
                    objectName: "takes"
                    controller: frame.controller
                    width: 300
                    height: 60
                    Keys.onPressed: (event) => event.accepted = event.key === Qt.Key_Return
                    onActivated: scene.taken += 1
                }
            }
        }
    }
}
