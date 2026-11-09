import QtQuick
import EmuSen.Kit as Kit

// A fixture: rows of the accent with gaps between them, in a clipping Flickable that the kit's EdgeFade fades at its foot and its right side, over stripes, for the fade test (EmuSen_Kit.md §7.35).
Window {
    id: scene
    objectName: "scene"
    width: 400
    height: 300
    color: "#101820"
    // Off and on again, the fade's layer is made anew; the test grabs its first frame.
    property bool fading: true
    Repeater {
        model: 40
        Rectangle {
            required property int index
            x: index * 10
            width: 5
            height: 300
            color: "#F0E8D8"
        }
    }
    Kit.EdgeFade {
        objectName: "fade"
        x: 40
        y: 40
        width: 320
        height: 160
        fadeBottom: column.fadeBottom
        fadeRight: column.fadeRight
        length: 40
        Flickable {
            id: column
            objectName: "column"
            anchors.fill: parent
            clip: true
            interactive: false
            contentWidth: 320
            contentHeight: 400
            property bool fadeBottom: scene.fading
            property bool fadeRight: scene.fading
            Repeater {
                model: 10
                Rectangle {
                    required property int index
                    y: index * 40 + 10
                    width: 320
                    height: 30
                    color: "#3070E0"
                }
            }
        }
    }
}
