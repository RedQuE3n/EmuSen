import QtQuick
import EmuSen.Theme 1.0
import "../parts"
import QtMultimedia

Screen {
    objectName: "Home"
    Shelf {
        anchors.fill: parent
        model: Library.recent
    }
}
