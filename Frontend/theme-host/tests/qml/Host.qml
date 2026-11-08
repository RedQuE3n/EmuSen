import QtQuick
import EmuSen.Models
import EmuSen.ThemeHost

// The host's test scene: one slot for Home, placed again each time the host loads; the program's models imported, so their
// types are registered in the process as they are in the program (EmuSen_Qt.md §3.5).
Window {
    id: scene
    objectName: "scene"
    visible: false
    width: 640
    height: 400
    color: "black"
    property bool placed: false
    ThemeHost {
        id: host
        objectName: "host"
        onGenerationChanged: scene.placed = host.ready && host.place(ThemeHost.Home, slot)
    }
    Item {
        id: slot
        objectName: "slot"
        anchors.fill: parent
    }
}
