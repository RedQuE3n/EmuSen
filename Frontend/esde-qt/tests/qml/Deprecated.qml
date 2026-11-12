import QtQuick
import EmuSen.Esde

// The deprecated hover(row) of EmuSen.Esde 1.1, called as a theme of 1.0 called it, set by tests/deprecated.rs (EmuSen_Qt.md §4.8.3).
Window {
    objectName: "scene"
    color: "#000000"
    property int hoverRow: -1
    onHoverRowChanged: view.scene.hover(hoverRow)
    View {
        id: view
        objectName: "view"
        anchors.fill: parent
    }
    Component.onCompleted: view.forceActiveFocus()
}
