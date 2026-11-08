import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models

Window {
    id: root
    objectName: "play"
    width: 1280
    height: 800
    visible: false
    property double feed: 0
    property string platformLibrary
    property string game
    property string cores
    property string engine

    SessionModel {
        id: session
        platformLibrary: root.platformLibrary
        feed: root.feed
        cores: root.cores
        engine: root.engine
        game: root.game
    }

    Kit.GamePicture {
        objectName: "picture"
        anchors.fill: parent
        feed: root.feed
        live: root.feed !== 0
    }
}
