pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models

// Home under the night sky in canvas units, as the accepted render draws it: Continue, the Recent shelf of whole covers and the seven places with their lines (EmuSen_Interface.md §24.3; EmuSen_Qt.md §7.1).
FocusScope {
    id: root
    objectName: "home"
    required property LibraryModel library
    property int padFamily: Kit.Families.Generic
    property int controllers: 0
    readonly property real placeWidth: 150
    readonly property real placePitch: 167
    // The game played last, read again whenever the games are, which an invokable alone would not see.
    readonly property bool hasLast: recent.revision >= 0 && recent.count > 0
    readonly property string lastTitle: root.hasLast && recent.revision >= 0 ? recent.titleAt(0) : ""
    readonly property string lastCover: root.hasLast && recent.revision >= 0 ? recent.coverAt(0) : ""
    readonly property string lastFacts: root.hasLast && recent.revision >= 0 ? recent.factsAt(0) : ""
    readonly property string lastPath: root.hasLast && recent.revision >= 0 ? recent.pathAt(0) : ""
    readonly property real lastHue: root.hasLast && recent.revision >= 0 ? recent.hueAt(0) : 0
    readonly property var lines: JSON.parse(root.library.homeLines)
    // The shelf's first cover, found again after each read of the games, whose count alone may not change.
    readonly property Item firstShelf: recent.revision >= 0 && shelf.count > 1 ? shelf.itemAt(1) : null
    signal placeChosen(int index)
    signal played(string path)
    GameModel {
        id: recent
        games: root.library.recent
    }
    Clock {
        id: clock
        controllers: root.controllers
    }
    Kit.Sky {
        objectName: "homeSky"
        anchors.fill: parent
        glowHue: root.hasLast ? root.lastHue : -1
        glowX: 0.2
        glowY: 0.42
    }
    Kit.Frame {
        id: frame
        anchors.fill: parent
        surfaceDrawn: false
        padFamily: root.padFamily
        helpFont: Qt.font({ family: faces.regular, pixelSize: 20 })
        helpSpacing: 40
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
                objectName: "homeStatus"
                x: 64
                y: 40
                text: clock.line
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
            Kit.Scene {
                objectName: "crescent"
                x: wordmark.x - 12 - width
                y: 38
                width: 26
                height: 26
                Glyph {
                    anchors.fill: parent
                    path: Marks.crescent
                    filled: true
                    ink: "#E9E4D4"
                }
            }
            Text {
                id: wordmark
                objectName: "wordmark"
                x: root.width - 64 - width
                y: 37.6
                text: Words.title
                color: Kit.Tokens.textStrong
                font.family: faces.light
                font.weight: Font.Light
                font.pixelSize: 24
                font.letterSpacing: 6
                lineHeight: 28.8
                lineHeightMode: Text.FixedHeight
            }
            Kit.Walkable {
                id: continuePlaying
                objectName: "continue"
                controller: frame.controller
                x: 52
                y: 72
                width: root.width - 104
                height: 296
                focus: true
                Accessible.name: Words.continuePlaying
                onActivated: if (root.hasLast) root.played(root.lastPath)
                KeyNavigation.down: root.firstShelf ?? (places.count > 0 ? places.itemAt(0) : null)
                Rectangle {
                    objectName: "continuePanel"
                    anchors.fill: parent
                    radius: Kit.Tokens.tileRadius + 4
                    color: Kit.Tokens.surfaceRaised
                    opacity: 0.55
                }
                Rectangle {
                    objectName: "continueRing"
                    anchors.fill: parent
                    visible: continuePlaying.activeFocus
                    radius: Kit.Tokens.tileRadius + 4
                    color: "transparent"
                    border.color: Kit.Tokens.accent
                    border.width: Kit.Tokens.ringThickness
                }
                Rectangle {
                    objectName: "continuePicture"
                    x: 12
                    y: 12
                    width: 373
                    height: 280
                    radius: Kit.Tokens.tileRadius
                    color: Kit.Tokens.surface
                    visible: root.hasLast
                    Image {
                        anchors.fill: parent
                        source: root.lastCover
                        fillMode: Image.PreserveAspectFit
                        asynchronous: false
                        smooth: true
                        mipmap: true
                    }
                }
                Text {
                    objectName: "continueHeading"
                    x: 409
                    y: 40
                    text: Words.continuePlaying
                    color: Kit.Tokens.muted
                    font.family: faces.regular
                    font.pixelSize: 20
                    lineHeight: 24
                    lineHeightMode: Text.FixedHeight
                }
                Text {
                    objectName: "continueTitle"
                    x: 409
                    y: 64
                    width: parent.width - 421
                    text: root.lastTitle
                    elide: Text.ElideRight
                    color: Kit.Tokens.textStrong
                    font.family: faces.condensed
                    font.weight: Font.Medium
                    font.pixelSize: 44
                    lineHeight: 52.8
                    lineHeightMode: Text.FixedHeight
                }
                Kit.Body {
                    objectName: "continuePlanet"
                    x: 409
                    y: 187
                    width: 14
                    height: 14
                    visible: root.hasLast
                    hue: root.lastHue
                }
                Text {
                    objectName: "continueFacts"
                    x: 433
                    y: 180
                    width: parent.width - 445
                    text: root.lastFacts
                    elide: Text.ElideRight
                    color: Kit.Tokens.muted
                    font.family: faces.regular
                    font.pixelSize: 22
                    lineHeight: 26.4
                    lineHeightMode: Text.FixedHeight
                }
            }
            Text {
                objectName: "recentHeading"
                x: 64
                y: 376
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
                        required property string cover
                        required property real hue
                        required property bool favourite
                        controller: frame.controller
                        objectName: "shelfCover." + (cover.index - 1)
                        visible: cover.index > 0 && cover.x + width <= parent.width
                        Accessible.name: cover.title
                        x: (cover.index - 1) * 144
                        width: 120
                        height: 160
                        onActivated: root.played(cover.path)
                        KeyNavigation.left: cover.index > 1 ? shelf.itemAt(cover.index - 1) : null
                        KeyNavigation.right: shelf.count > cover.index + 1 ? shelf.itemAt(cover.index + 1) : null
                        KeyNavigation.up: continuePlaying
                        KeyNavigation.down: places.count === 0 ? null : places.itemAt(Math.min(places.count - 1, Math.round((cover.index - 1) * 144 / root.placePitch)))
                        Cover {
                            anchors.fill: parent
                            source: cover.cover
                            title: cover.title
                            hue: cover.hue
                            favourite: cover.favourite
                            lit: cover.activeFocus
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
                    readonly property string status: root.lines[place.index]?.status ?? ""
                    readonly property bool attention: root.lines[place.index]?.attention ?? false
                    controller: frame.controller
                    objectName: "place." + place.index
                    Accessible.name: place.modelData
                    Accessible.description: place.status
                    x: 64 + place.index * root.placePitch
                    y: 622
                    width: root.placeWidth
                    height: 64
                    onActivated: root.placeChosen(place.index)
                    KeyNavigation.left: places.count > place.index ? places.itemAt(place.index - 1) : null
                    KeyNavigation.right: places.count > place.index ? places.itemAt(place.index + 1) : null
                    KeyNavigation.up: root.firstShelf ?? continuePlaying
                    Rectangle {
                        objectName: "placeTile." + place.index
                        anchors.fill: parent
                        radius: Kit.Tokens.tileRadius
                        color: Kit.Tokens.surfaceRaised
                        opacity: 0.5
                    }
                    Rectangle {
                        anchors.fill: parent
                        radius: Kit.Tokens.tileRadius
                        color: "transparent"
                        border.color: Kit.Tokens.border
                        border.width: 1
                        opacity: 0.4
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
                        x: 12
                        y: 8
                        width: parent.width - 24
                        text: place.modelData
                        elide: Text.ElideRight
                        color: Kit.Tokens.textStrong
                        font.family: faces.regular
                        font.pixelSize: 20
                        lineHeight: 24
                        lineHeightMode: Text.FixedHeight
                    }
                    Row {
                        x: 12
                        y: 32
                        spacing: 8
                        Glyph {
                            objectName: "placeAttention." + place.index
                            visible: place.attention
                            anchors.verticalCenter: parent.verticalCenter
                            width: 16
                            height: 16
                            path: Marks.attention
                            filled: true
                            ink: Kit.Tokens.attention
                        }
                        Text {
                            objectName: "placeStatus." + place.index
                            width: place.width - 24 - (place.attention ? 24 : 0)
                            text: place.status
                            elide: Text.ElideRight
                            color: place.attention ? Kit.Tokens.attention : Kit.Tokens.muted
                            font.family: faces.regular
                            font.pixelSize: 20
                            lineHeight: 24
                            lineHeightMode: Text.FixedHeight
                        }
                    }
                }
            }
        }
    }
    Faces {
        id: faces
    }
}
