import QtQuick
import EmuSen.Theme 1.0
import "../parts"

ThemeScreen {
    objectName: "Consoles"
    Shelf {
        anchors.fill: parent
        model: Library.recent
    }
}
