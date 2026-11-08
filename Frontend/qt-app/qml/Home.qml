pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models

// Home under the night sky (EmuSen_Interface.md §24.3): the hub the program opens on after Moonrise.
FocusScope {
    id: root
    objectName: "home"
    required property LibraryModel library
    property int padFamily: Kit.Families.Generic
    Kit.Sky {
        id: sky
        objectName: "homeSky"
        anchors.fill: parent
    }
    Kit.Frame {
        id: frame
        anchors.fill: parent
        surfaceDrawn: false
        padFamily: root.padFamily
        focus: true
        Text {
            objectName: "wordmark"
            anchors.right: parent.right
            text: Words.title
            color: Kit.Tokens.textStrong
            font.pixelSize: Kit.Tokens.fontStrong
        }
    }
}
