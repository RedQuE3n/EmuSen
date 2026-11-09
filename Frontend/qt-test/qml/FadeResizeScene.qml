import QtQuick
import EmuSen.Kit as Kit

// A fixture: an EdgeFade in a hidden band whose right edge stops fading when the window grows wide enough to show all it holds, for the fade's resize test (EmuSen_Kit.md §7.35).
Window {
    id: scene
    objectName: "scene"
    width: 400
    height: 300
    color: "#101820"
    Item {
        objectName: "band"
        anchors.fill: parent
        visible: false
        opacity: 0
        Kit.EdgeFade {
            anchors.fill: parent
            fadeRight: row.fadeRight
            length: 40
            Flickable {
                id: row
                objectName: "row"
                anchors.fill: parent
                clip: true
                interactive: false
                contentWidth: 2000
                contentHeight: height
                property bool fadeRight: contentWidth > width
                Rectangle {
                    width: 2000
                    height: 30
                    color: "#3070E0"
                }
            }
        }
    }
}
