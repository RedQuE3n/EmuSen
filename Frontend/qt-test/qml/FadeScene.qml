import QtQuick
import EmuSen.Kit as Kit

// A fixture: a column of the accent in a clipping Flickable whose foot fades through the kit's EdgeFade, for the fade test (EmuSen_Kit.md §7.35).
Window {
    id: scene
    objectName: "scene"
    width: 400
    height: 300
    color: Kit.Tokens.surface
    Flickable {
        id: column
        objectName: "column"
        x: 40
        y: 40
        width: 320
        height: 160
        clip: true
        interactive: false
        contentWidth: 320
        contentHeight: 400
        property bool fadeBottom: true
        Rectangle {
            width: 320
            height: 400
            color: Kit.Tokens.accent
        }
    }
    Kit.EdgeFade {
        objectName: "fade"
        source: column
        fadeBottom: column.fadeBottom
        length: 40
        backdrop: scene.color
    }
}
