import QtQuick
import EmuSen.Esde

// The painter's primitives at known places over black, read back by tests/primitives.rs on the GPU tier (EmuSen_Qt.md §4.8).
Window {
    id: root
    objectName: "scene"
    color: "#000000"
    property url white
    property url strip
    property url checker
    property url wide
    property url face
    property url art
    property int reports: 0
    Item {
        anchors.fill: parent
        visible: root.art.toString() === ""
        Picture { objectName: "tinted"; x: 0; y: 0; width: 64; height: 64; fit: Picture.Fit.Fill; source: root.white; tint: "#FF8000" }
        Picture { objectName: "halfTinted"; x: 80; y: 0; width: 64; height: 64; fit: Picture.Fit.Fill; source: root.white; tint: "#80FF8000" }
        Picture { objectName: "grey"; x: 160; y: 0; width: 64; height: 16; fit: Picture.Fit.Fill; nearest: true; saturation: 0; source: root.strip }
        Picture { objectName: "rounded"; x: 0; y: 100; width: 192; height: 128; fit: Picture.Fit.Fill; radius: 32; source: root.white }
        Picture { objectName: "ramp"; x: 220; y: 100; width: 96; height: 96; fit: Picture.Fit.Fill; fadeFrom: 0.5; fadeFalloff: 2; source: root.white }
        Picture { objectName: "sharp"; x: 400; y: 100; width: 32; height: 32; fit: Picture.Fit.Fill; nearest: true; source: root.checker }
        Picture { objectName: "soft"; x: 450; y: 100; width: 32; height: 32; fit: Picture.Fit.Fill; source: root.checker }
        Picture { objectName: "small"; x: 700; y: 100; width: 400; height: 100; fit: Picture.Fit.Contain; source: root.white }
        Picture { objectName: "big"; x: 700; y: 220; width: 128; height: 128; fit: Picture.Fit.Contain; source: root.wide }
        Picture { objectName: "covered"; x: 900; y: 0; width: 300; height: 80; fit: Picture.Fit.Cover; source: root.white }
        Mark { objectName: "star"; x: 600; y: 0; width: 64; height: 64; mark: "star"; color: "#00FF00" }
        TextBlock { objectName: "words"; x: 0; y: 300; width: 400; height: 40; text: "Measured once"; px: 24; color: "#FFFFFF"; fontSource: root.face; onMeasured: root.reports += 1 }
        TextBlock { objectName: "same"; x: 0; y: 350; width: 400; height: 40; text: "The same face"; px: 24; color: "#FFFFFF"; fontSource: root.face }
        TextBlock { objectName: "plain"; x: 0; y: 400; width: 400; height: 40; text: "No face named"; px: 24; color: "#FFFFFF" }
    }
    Picture { objectName: "slice"; anchors.fill: parent; visible: root.art.toString() !== ""; source: root.art; tint: "#DDFFFFFF" }
}
