import QtQuick
import EmuSen.Esde

// An EsdeModel alone, for its load: the texts set by the tests, nothing drawn (EmuSen_Qt.md §4.9).
Window {
    objectName: "scene"
    width: 320
    height: 200
    EsdeModel {
        objectName: "model"
        width: 1280
        height: 800
    }
}
