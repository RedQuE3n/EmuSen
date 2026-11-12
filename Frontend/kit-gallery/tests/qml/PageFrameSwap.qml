import QtQuick
import EmuSen.Kit as Kit

// A page frame whose tile model is replaced while it resets: the keyboard's model, relaid out, resets, and the scene
// hands the frame another model from inside that reset (EmuSen_Kit.md §7.58).
Window {
    id: scene
    objectName: "scene"
    visible: true
    width: 1280
    height: 800
    property bool swapping: false
    Kit.KeyboardModel {
        id: first
        objectName: "first"
    }
    ListModel {
        id: second
        ListElement { label: "Slot 1"; line: "2 h ago"; picture: "" }
        ListElement { label: "Slot 2"; line: "Yesterday"; picture: "" }
    }
    Kit.PageFrame {
        id: frame
        objectName: "frame"
        anchors.fill: parent
        title: "Cobalt Harbor"
        tileModel: first
        rowModel: ListModel {}
        readonly property bool tilesShown: frame.tileView.model === second
        readonly property bool handed: frame.tileModel === second
        readonly property int tileCount: frame.tileView.count
    }
    Connections {
        target: first
        enabled: scene.swapping
        function onModelAboutToBeReset() { frame.tileModel = second }
    }
}
