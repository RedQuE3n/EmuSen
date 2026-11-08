pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models

// Home under the night sky, the Home canvas in canvas units: Continue, the Recent shelf and the seven places (EmuSen_Interface.md §24.3; EmuSen_Qt.md §7.1).
FocusScope {
    id: root
    objectName: "home"
    required property LibraryModel library
    property int padFamily: Kit.Families.Generic
    readonly property real placeWidth: (root.width - 128 - 6 * 14) / 7
    readonly property bool hasLast: recent.count > 0
    signal placeChosen(int index)
    signal played(string path)

    GameModel {
        id: recent
        games: root.library.recent
    }
    Clock {
        id: clock
    }
    Kit.Sky {
        objectName: "homeSky"
        anchors.fill: parent
        glowHue: root.hasLast ? recent.hueAt(0) : -1
    }
    Kit.Frame {
        id: frame
        anchors.fill: parent
        surfaceDrawn: false
        padFamily: root.padFamily
        focus: true
        controller.navigationShown: false
        help: HelpModel {
            screen: HelpModel.Home
        }
        Item {
            x: -frame.safeX
            y: -frame.safeY
            width: root.width
            height: root.height
            Text {
                id: status
                objectName: "homeStatus"
                x: 64
                y: 40
                text: clock.time
                color: Kit.Tokens.muted
                font.family: faces.regular
                font.pixelSize: 20
                lineHeight: 24
                lineHeightMode: Text.FixedHeight
                Timer {
                    interval: 15000
                    repeat: true
                    running: root.visible
                    onTriggered: clock.tick()
                }
            }
            Kit.Body {
                objectName: "crescent"
                x: wordmark.x - 12 - width
                y: 39
                width: 26
                height: 26
                hue: 48
            }
            Text {
                id: wordmark
                objectName: "wordmark"
                x: root.width - 64 - width
                y: 37.6
                text: Words.title
                color: Kit.Tokens.textStrong
                font.family: faces.light
                font.pixelSize: 24
                font.letterSpacing: 2
                lineHeight: 28.8
                lineHeightMode: Text.FixedHeight
            }
            Kit.Walkable {
                id: continuePlaying
                objectName: "continue"
                controller: frame.controller
                x: 52
                y: 80
                width: root.width - 104
                height: 304
                focus: true
                Accessible.name: Words.continuePlaying
                onActivated: root.played(recent.pathAt(0))
                KeyNavigation.down: shelf.count > 0 ? shelf.itemAt(0) : places.count > 0 ? places.itemAt(0) : null
                Rectangle {
                    objectName: "continuePanel"
                    anchors.fill: parent
                    radius: Kit.Tokens.tileRadius
                    color: Kit.Tokens.surfaceRaised
                    opacity: 0.55
                }
                Rectangle {
                    objectName: "continueRing"
                    anchors.fill: parent
                    visible: continuePlaying.activeFocus
                    radius: Kit.Tokens.tileRadius
                    color: "transparent"
                    border.color: Kit.Tokens.accent
                    border.width: Kit.Tokens.ringThickness
                }
                GameCard {
                    objectName: "continuePicture"
                    x: 12
                    y: 12
                    width: 373
                    height: 280
                    visible: root.hasLast
                    title: recent.titleAt(0)
                    hue: recent.hueAt(0)
                }
                Text {
                    objectName: "continueHeading"
                    x: 417
                    y: 20
                    text: Words.continuePlaying
                    color: Kit.Tokens.muted
                    font.family: faces.regular
                    font.pixelSize: 20
                    lineHeight: 24
                    lineHeightMode: Text.FixedHeight
                }
                Text {
                    objectName: "continueTitle"
                    x: 417
                    y: 54
                    width: parent.width - 429
                    text: recent.titleAt(0)
                    elide: Text.ElideRight
                    color: Kit.Tokens.textStrong
                    font.family: faces.condensed
                    font.pixelSize: 44
                    lineHeight: 52.8
                    lineHeightMode: Text.FixedHeight
                }
            }
            Text {
                objectName: "recentHeading"
                x: 64
                y: 392
                text: Words.recent
                color: Kit.Tokens.muted
                font.family: faces.regular
                font.pixelSize: 20
                lineHeight: 24
                lineHeightMode: Text.FixedHeight
            }
            Item {
                objectName: "shelf"
                x: 64
                y: 424
                width: root.width - 128
                height: 160
                Repeater {
                    id: shelf
                    model: recent
                    Kit.Walkable {
                        id: cover
                        required property int index
                        required property string path
                        required property string title
                        required property real hue
                        controller: frame.controller
                        objectName: "shelfCover." + cover.index
                        visible: cover.x + width <= parent.width
                        Accessible.name: cover.title
                        x: cover.index * 144
                        y: cover.activeFocus ? -6 : 0
                        width: 120
                        height: 160
                        onActivated: root.played(cover.path)
                        KeyNavigation.left: shelf.count > cover.index ? shelf.itemAt(cover.index - 1) : null
                        KeyNavigation.right: shelf.count > cover.index ? shelf.itemAt(cover.index + 1) : null
                        KeyNavigation.up: continuePlaying
                        KeyNavigation.down: places.count === 0 ? null : places.itemAt(Math.min(places.count - 1, Math.round(cover.index * 144 / (root.placeWidth + 14))))
                        GameCard {
                            anchors.fill: parent
                            title: cover.title
                            hue: cover.hue
                        }
                        Rectangle {
                            anchors.fill: parent
                            anchors.margins: -3
                            visible: cover.activeFocus
                            radius: Kit.Tokens.rowRadius + 3
                            color: "transparent"
                            border.color: Kit.Tokens.accent
                            border.width: Kit.Tokens.ringThickness
                        }
                    }
                }
            }
            Repeater {
                id: places
                model: Words.homePlaces
                Kit.Walkable {
                    id: place
                    required property int index
                    required property string modelData
                    controller: frame.controller
                    objectName: "place." + place.index
                    Accessible.name: place.modelData
                    x: 64 + place.index * (root.placeWidth + 14)
                    y: 632
                    width: root.placeWidth
                    height: 64
                    onActivated: root.placeChosen(place.index)
                    KeyNavigation.left: places.count > place.index ? places.itemAt(place.index - 1) : null
                    KeyNavigation.right: places.count > place.index ? places.itemAt(place.index + 1) : null
                    KeyNavigation.up: shelf.count > 0 ? shelf.itemAt(0) : continuePlaying
                    Rectangle {
                        objectName: "placeTile." + place.index
                        anchors.fill: parent
                        radius: Kit.Tokens.tileRadius
                        color: Kit.Tokens.surfaceRaised
                        opacity: 0.5
                    }
                    Rectangle {
                        anchors.fill: parent
                        anchors.margins: -3
                        visible: place.activeFocus
                        radius: Kit.Tokens.tileRadius + 3
                        color: "transparent"
                        border.color: Kit.Tokens.accent
                        border.width: Kit.Tokens.ringThickness
                    }
                    Text {
                        objectName: "placeName." + place.index
                        x: 14
                        y: 9.4
                        width: parent.width - 28
                        text: place.modelData
                        elide: Text.ElideRight
                        color: Kit.Tokens.textStrong
                        font.family: faces.regular
                        font.pixelSize: 20
                        lineHeight: 24
                        lineHeightMode: Text.FixedHeight
                    }
                }
            }
        }
    }
    Faces {
        id: faces
    }
}
