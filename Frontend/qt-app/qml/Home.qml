pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models

// Home under the night sky in canvas units, as the accepted render draws it: Continue while a game is suspended, else the shelf's game at the top, the Recent shelf of whole covers and the seven places with their lines (EmuSen_Interface.md §24.3; EmuSen_Qt.md §7.1).
FocusScope {
    id: root
    objectName: "home"
    required property LibraryModel library
    property int padFamily: Kit.Families.Generic
    property int controllers: 0
    readonly property real placeWidth: 150
    readonly property real placePitch: 167
    // The program's game, suspended while Home shows, and the item its picture is drawn in; none before a game.
    property SessionModel session: null
    property Item gamePicture: null
    // Canvas units to pixels, so the paused frame's texture is drawn at the window's own resolution.
    property real pixelScale: 1
    // Continue stands only while a game is suspended, as Mistress's Home has it; its row is found again on each read of the games.
    readonly property bool suspended: root.session !== null && root.session.running
    readonly property int suspendedRow: root.suspended && recent.revision >= 0 ? recent.rowOf(root.session.game) : -1
    // The shelf's last focused game, which the top shows when nothing is suspended (the shelf is then the games played).
    property int shelfRow: 0
    readonly property int topRow: root.suspended ? root.suspendedRow : (recent.count > 0 ? Math.min(root.shelfRow, recent.count - 1) : -1)
    readonly property bool hasTop: root.suspended || root.topRow >= 0
    readonly property string topTitle: recent.revision >= 0 && root.topRow >= 0 ? recent.titleAt(root.topRow) : (root.suspended ? root.library.titleOf(root.session.game) : "")
    readonly property string topPicture: recent.revision >= 0 && root.topRow >= 0 ? recent.heroAt(root.topRow) : ""
    readonly property string topFacts: root.suspended && clock.paused.length > 0 ? clock.paused : (recent.revision >= 0 && root.topRow >= 0 ? recent.factsAt(root.topRow) : "")
    readonly property real topHue: recent.revision >= 0 && root.topRow >= 0 ? recent.hueAt(root.topRow) : -1
    readonly property var lines: JSON.parse(root.library.homeLines)
    // The shelf's first cover shown, found again after each read of the games, whose count alone may not change.
    readonly property bool shelfFocused: shelf.count > 0 && root.shelfRow >= 0 && root.shelfRow < shelf.count && shelf.itemAt(root.shelfRow)?.activeFocus === true
    readonly property Item firstShelf: shelved.revision >= 0 && shelf.count > 0 ? shelf.itemAt(0) : null
    // Counts Start pressed on Home: each takes the focus back to where Home starts (EmuSen_BigPicture.md §51.2).
    property int startAgain: 0
    // The suspended game's console and pause in words, as the Home card's Continue says them.
    readonly property string pausedWords: clock.paused
    signal placeChosen(int index)
    signal played(string path)
    signal resumed()
    signal acted(string key)
    onStartAgainChanged: (root.suspended ? continuePlaying : (root.firstShelf ?? places.itemAt(0))).forceActiveFocus()
    GameModel {
        id: recent
        games: root.library.recent
    }
    // The shelf: the games played, the suspended one left out since Continue holds it.
    GameModel {
        id: shelved
        games: root.library.recent
        leaveOut: root.suspended ? root.session.game : ""
    }
    Clock {
        id: clock
        controllers: root.controllers
        console: root.suspended && recent.revision >= 0 ? recent.consoleAt(root.suspendedRow) : ""
        gamePaused: root.suspended && root.session.paused
    }
    Kit.Sky {
        objectName: "homeSky"
        anchors.fill: parent
        glowHue: root.topHue
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
        helpGlyphGap: 10
        focus: true
        controller.navigationShown: false
        help: HelpModel {
            screen: continuePlaying.activeFocus ? HelpModel.Home : (root.shelfFocused ? HelpModel.HomeShelf : HelpModel.HomePlaces)
        }
        Connections {
            target: frame.controller
            function onActionInvoked(key) { root.acted(key) }
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
                // Continue holds the focus while a game is suspended; otherwise the shelf's first game does, else the first place.
                focus: root.suspended
                activeFocusOnTab: root.suspended
                Accessible.name: Words.continuePlaying
                onActivated: if (root.suspended) root.resumed()
                KeyNavigation.down: root.firstShelf ?? (places.count > 0 ? places.itemAt(0) : null)
                Rectangle {
                    objectName: "continuePanel"
                    anchors.fill: parent
                    visible: root.suspended
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
                    visible: root.hasTop
                    Kit.RoundPicture {
                        objectName: "continueArt"
                        anchors.fill: parent
                        visible: !root.suspended
                        source: root.topPicture
                        radius: Kit.Tokens.tileRadius
                        crop: true
                        pixelScale: root.pixelScale
                    }
                    // The game's own picture drawn into a texture at the window's resolution, cut to the box's shape at its centre as Mistress's poster slot cuts the paused frame, so its corners round as a cover's.
                    ShaderEffectSource {
                        id: pausedTexture
                        readonly property Item frame: root.gamePicture
                        readonly property real cutWidth: pausedTexture.frame ? Math.min(pausedTexture.frame.width, pausedTexture.frame.height * 373 / 280) : 0
                        readonly property real cutHeight: pausedTexture.cutWidth * 280 / 373
                        sourceItem: root.suspended && root.visible ? pausedTexture.frame : null
                        hideSource: true
                        visible: false
                        sourceRect: pausedTexture.frame ? Qt.rect((pausedTexture.frame.width - pausedTexture.cutWidth) / 2, (pausedTexture.frame.height - pausedTexture.cutHeight) / 2, pausedTexture.cutWidth, pausedTexture.cutHeight) : Qt.rect(0, 0, 0, 0)
                        textureSize: Qt.size(Math.ceil(373 * root.pixelScale), Math.ceil(280 * root.pixelScale))
                    }
                    Shape {
                        id: pausedRounded
                        objectName: "continueFrame"
                        anchors.fill: parent
                        visible: root.suspended && root.gamePicture !== null
                        preferredRendererType: Shape.CurveRenderer
                        ShapePath {
                            strokeColor: "transparent"
                            fillColor: "transparent"
                            fillItem: pausedTexture
                            fillTransform: PlanarTransform.fromScale(1 / root.pixelScale, 1 / root.pixelScale)
                            PathRectangle {
                                width: 373
                                height: 280
                                radius: Kit.Tokens.tileRadius
                            }
                        }
                    }
                }
                Text {
                    objectName: "continueHeading"
                    visible: root.suspended
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
                    text: root.topTitle
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
                    visible: root.suspended && root.topHue >= 0
                    hue: root.topHue
                }
                Text {
                    objectName: "continueFacts"
                    x: root.suspended && root.topHue >= 0 ? 433 : 409
                    y: 180
                    width: parent.width - x - 12
                    text: root.topFacts
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
                id: shelfBox
                objectName: "shelf"
                x: 64
                y: 424
                width: root.width - 128
                height: 160
                Repeater {
                    id: shelf
                    model: shelved
                    Kit.Walkable {
                        id: cover
                        required property int index
                        required property string path
                        required property string title
                        required property string cover
                        required property real hue
                        required property bool favourite
                        controller: frame.controller
                        objectName: "shelfCover." + cover.index
                        visible: cover.x + width <= shelfBox.width
                        focus: !root.suspended && cover.index === 0
                        Accessible.name: cover.title
                        x: cover.index * 144
                        width: 120
                        height: 160
                        onActivated: root.played(cover.path)
                        onActiveFocusChanged: if (cover.activeFocus) root.shelfRow = cover.index
                        KeyNavigation.left: cover.index > 0 ? shelf.itemAt(cover.index - 1) : null
                        KeyNavigation.right: shelf.count > cover.index + 1 ? shelf.itemAt(cover.index + 1) : null
                        KeyNavigation.up: root.suspended ? continuePlaying : null
                        KeyNavigation.down: places.count === 0 ? null : places.itemAt(Math.min(places.count - 1, Math.round(cover.index * 144 / root.placePitch)))
                        Cover {
                            anchors.fill: parent
                            source: cover.cover
                            title: cover.title
                            hue: cover.hue
                            favourite: cover.favourite
                            lit: cover.activeFocus
                            pixelScale: root.pixelScale
                        }
                    }
                }
            }
            Text {
                objectName: "shelfTitle"
                x: 64
                y: 586
                width: root.width - 128
                visible: root.shelfFocused
                text: shelved.revision >= 0 ? shelved.titleAt(root.shelfRow) : ""
                elide: Text.ElideRight
                color: Kit.Tokens.text
                font.family: faces.regular
                font.pixelSize: 22
                lineHeight: 26.4
                lineHeightMode: Text.FixedHeight
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
                    focus: !root.suspended && shelf.count === 0 && place.index === 0
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
