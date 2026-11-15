import QtQuick

// A fixture: a list clicked and then turned by the wheel at once, as a settings screen is (EmuSen_Kit.md §7.36).
Window {
    id: scene
    objectName: "scene"
    width: 320
    height: 200
    visible: true
    ListView {
        id: list
        objectName: "list"
        anchors.fill: parent
        model: 50
        delegate: Rectangle {
            required property int index
            width: ListView.view.width
            height: 40
            color: index % 2 ? "white" : "black"
        }
    }
}
