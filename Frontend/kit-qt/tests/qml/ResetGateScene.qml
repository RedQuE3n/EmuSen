import QtQuick
import EmuSen.Kit as Kit

// A reset gate over the keyboard's model, which resets when it is relaid out; while `swapping`, the scene hands the gate a
// second model the moment the first one's reset begins (EmuSen_Kit.md §7.58).
Window {
    id: scene
    objectName: "scene"
    visible: true
    width: 640
    height: 480
    property bool swapping: false
    property bool swapped: false
    Kit.KeyboardModel {
        id: first
        objectName: "first"
    }
    Kit.KeyboardModel {
        id: second
        objectName: "second"
    }
    Kit.ResetGate {
        id: gate
        objectName: "gate"
        model: scene.swapped ? second : first
        readonly property bool holdsSecond: gate.model === second
        onSettledChanged: if (!gate.settled) scene.swapped = scene.swapping
    }
    // A second gate handed its model by assignment, no binding to be told when the model is deleted (a binding on the
    // model's id, or through a var property, is told and hands the gate null itself); `later` is made only after the
    // deletion, the first model of its kind made since, so the allocator may place it where `doomed` was.
    property bool handing: false
    property bool handLater: false
    onHandingChanged: lone.model = doomed
    onHandLaterChanged: lone.model = maker.createObject(scene, { objectName: "later" })
    Kit.KeyboardModel {
        id: doomed
        objectName: "doomed"
    }
    Component {
        id: maker
        Kit.KeyboardModel {}
    }
    Kit.ResetGate {
        id: lone
        objectName: "lone"
        readonly property bool holdsNone: lone.model === null
    }
}
