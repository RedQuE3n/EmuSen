import QtQuick
import EmuSen.ThemeHost

// theme-audit's scene: the host the program uses and one slot filling the window; each time the host loads, the screen of `role` is placed (man theme-check).
Window {
    id: scene
    objectName: "scene"
    visible: false
    width: 1280
    height: 800
    color: "black"
    property int role: ThemeHost.Home
    property bool placed: false
    ThemeHost {
        id: host
        objectName: "host"
        onGenerationChanged: scene.placed = host.ready && host.place(scene.role, slot)
    }
    Item {
        id: slot
        objectName: "slot"
        anchors.fill: parent
    }
}
