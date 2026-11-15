pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

// One page of a PDF turned to another, to read the picture's state in the same turn of the event loop and what is drawn while it loads (EmuSen_Kit.md §7.110).
Window {
    id: scene
    objectName: "scene"
    visible: true
    width: 640
    height: 480
    color: "#000000"
    property url manual
    property int page
    Kit.PdfPage {
        objectName: "manualPage"
        anchors.fill: parent
        source: scene.manual
        page: scene.page
    }
}
