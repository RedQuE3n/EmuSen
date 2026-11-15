import QtQuick
import EmuSen.Kit as Kit

// A fixture: a pad press that starts an animation through a binding, as the ES-DE view's clock starts, and then queues
// work longer than a short wait, so the animation's queued start comes after it (EmuSen_Kit.md §7.36).
Window {
    id: scene
    objectName: "scene"
    width: 320
    height: 200
    visible: true
    property bool moving: false
    // How long the queued work keeps the GUI thread, in ms of the wall clock: a fixture's figure.
    property int busy: 60
    QtObject {
        id: probe
        objectName: "probe"
        property real value: 0
    }
    NumberAnimation {
        target: probe
        property: "value"
        running: scene.moving
        from: 0
        to: 1e9
        duration: 1e9
    }
    // The press sets `moving` at once; the work comes a turn of the event loop later, as a delayed binding queues it.
    PropertyAction {
        id: press
        target: scene
        property: "moving"
        value: true
    }
    QtObject {
        id: work
        property bool on: false
        onOnChanged: {
            const until = Date.now() + scene.busy
            while (Date.now() < until) {}
        }
    }
    Binding {
        target: work
        property: "on"
        value: scene.moving
        delayed: true
    }
    Connections {
        target: Kit.PadRouter
        function onPressed(button, repeat, jump) { press.start() }
    }
}
