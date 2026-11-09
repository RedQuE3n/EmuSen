pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

// A fixture: the help bar's entries change while a frame clock runs, as a theme's help changes with the focus over its sky (EmuSen_Kit.md §7.49).
Window {
    id: scene
    objectName: "scene"
    width: 1280
    height: 800
    color: "#000000"
    property int alt: 0
    ListModel {
        id: offered
        ListElement { button: 0; label: "Open"; enabled: true }
        ListElement { button: 2; label: "Search"; enabled: true }
        ListElement { button: 8; label: "Home"; enabled: true }
    }
    ListModel {
        id: declined
        ListElement { button: 0; label: "Open"; enabled: false }
        ListElement { button: 2; label: "Search"; enabled: true }
        ListElement { button: 8; label: "Home"; enabled: true }
    }
    FrameAnimation {
        objectName: "clock"
        running: true
    }
    Kit.HelpBar {
        objectName: "bar"
        y: 700
        width: 1280
        model: scene.alt === 0 ? offered : declined
    }
}
