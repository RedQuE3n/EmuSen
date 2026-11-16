import QtQuick
import EmuSen.Kit as Kit

// The kit's backdrop: a ground, a blurred ellipse and a turned rounded streak, or a picture in their place; the values are test inputs (EmuSen_Kit.md §7.118).
Window {
    id: scene
    objectName: "scene"
    visible: true
    width: 1200
    height: 800
    color: "#000000"
    property url picture
    Kit.Backdrop {
        objectName: "backdrop"
        anchors.fill: parent
        ground: "#0A1024"
        source: scene.picture
        sourceBlur: 40
        sourceOpacity: 0.5
        fields: [
            Kit.BackdropField {
                x: 200
                y: 150
                width: 400
                height: 300
                color: "#1FC9A8"
                blur: 40
                opacity: 0.85
            },
            Kit.BackdropField {
                x: 700
                y: 500
                width: 300
                height: 40
                color: "#D9C8FF"
                blur: 10
                opacity: 0.5
                rotation: -18
                corner: 26
            }
        ]
    }
}
