import QtQuick

// A fixture: a square whose colour a timer changes, once or twice, with or without a new minute on the scene's clock (EmuSen_Qt.md §9.3.2).
Window {
    id: scene
    objectName: "scene"
    width: 320
    height: 200
    property bool withMinute: false
    QtObject {
        id: clock
        objectName: "clock"
        property string time: "12:00"
    }
    Rectangle {
        id: square
        width: 40
        height: 40
        property bool lit: true
        color: square.lit ? "white" : "black"
    }
    Timer {
        id: change
        objectName: "change"
        interval: 400
        // The changes still to come, and the wait before each after the first.
        property int left: 1
        property int gap: 400
        onTriggered: {
            square.lit = !square.lit;
            if (scene.withMinute)
                clock.time = "12:0" + change.left;
            change.left -= 1;
            if (change.left > 0) {
                change.interval = change.gap;
                change.start();
            }
        }
    }
}
