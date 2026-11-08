import QtQuick
import EmuSen.Esde

// The ES-DE view filling a window, its texts set by tests/view.rs (EmuSen_Qt.md §4.8.4).
Window {
    objectName: "scene"
    color: "#000000"
    View {
        id: view
        objectName: "view"
        anchors.fill: parent
    }
    Component.onCompleted: view.forceActiveFocus()
}
