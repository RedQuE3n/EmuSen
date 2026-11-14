import QtQuick
import EmuSen.Kit as Kit

// A help bar with a theme's own picture and a page of a PDF on the big screen's canvas, to read each picture's decoded size against the pixels it covers (EmuSen_Kit.md §7.111).
Window {
    id: scene
    objectName: "scene"
    visible: true
    width: 1280
    height: 800
    color: "#000000"
    property url manual
    Kit.NormalizedCanvas {
        objectName: "canvas"
        Kit.HelpBar {
            objectName: "help"
            x: 40
            y: 40
            fitted: true
            family: Kit.Families.Xbox
            model: ListModel {
                ListElement { button: 6; label: "Menu"; icon: "qrc:/scenes/button-picture.svg"; enabled: true }
            }
        }
        Kit.PdfPage {
            objectName: "manualPage"
            x: 40
            y: 200
            width: 300
            height: 400
            source: scene.manual
        }
    }
}
