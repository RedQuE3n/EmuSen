pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

// Pages of as many PDF files as the test writes, each over the whole window, made and then all destroyed at once, as a closing manual destroys its page (EmuSen_Kit.md §7.110).
Window {
    id: scene
    objectName: "scene"
    visible: true
    width: 1280
    height: 800
    color: "#000000"
    property string folder
    property int count
    property int round
    Repeater {
        model: scene.count
        Kit.PdfPage {
            required property int index
            width: scene.width
            height: scene.height
            source: "file://" + scene.folder + "/" + scene.round + "-" + index + ".pdf"
        }
    }
}
