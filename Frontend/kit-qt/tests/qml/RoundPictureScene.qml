import QtQuick
import EmuSen.Kit as Kit

// RoundPicture asynchronous loading cleanly on the software backend.
Window {
    id: scene
    objectName: "scene"
    visible: true
    width: 240
    height: 240
    color: "#000000"
    Kit.RoundPicture {
        id: picture
        objectName: "picture"
        x: 20
        y: 20
        width: 200
        height: 200
        radius: 24
    }
}
