import QtQuick
import EmuSen.Theme 1.0

// A theme written for EmuSen.Theme 1.0: a 1.0 type and the tokens found by a 1.0 import, and a part of 1.1 asked for by one (EmuSen_Qt.md §3.2, rule T).
Window {
    id: scene
    objectName: "scene"
    visible: true
    width: 400
    height: 300
    color: Tokens.voidFill
    readonly property color accent: Tokens.accent
    readonly property bool framed: frame.width > 0
    EdgeFade {
        id: frame
        objectName: "fade"
        width: 200
        height: 100
    }
    // Handed a file the test writes, a 1.0 import asking for FocusRing, a part of 1.1.
    Loader {
        objectName: "later"
    }
}
