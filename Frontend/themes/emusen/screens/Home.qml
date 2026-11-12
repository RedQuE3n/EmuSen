pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Theme
import "../parts"

// Home under the night sky in canvas units, as the accepted render draws it: Continue while a game is suspended, else the shelf's game at the top, the Recent shelf of whole covers and the seven places with their lines (EmuSen_Interface.md §24.3; EmuSen_Qt.md §7.1).
ThemeScreen {
    id: screen
    objectName: "homeScreen"
    role: ThemeScreen.Home
    // Where the focus starts, and where Start on Home takes it back: Continue, else the shelf's first game, else the first place.
    focusItems: Session.hasGame ? [continuePlaying] : [root.firstShelf ?? root.firstPlace]
    // A game of the shelf, as its row gives it.
    component ShelfCover: Walkable {
        required property int index
        required property string key
        required property string title
        required property string cover
        required property real consoleHue
        required property bool favourite
        required property string hero
        required property string facts
    }
    NormalizedCanvas {
        id: canvas
        objectName: "homeCanvas"
        FocusScope {
            id: root
            objectName: "home"
            anchors.fill: parent
            focus: true
            readonly property real placeWidth: 150
            readonly property real placePitch: 167
            // Canvas units to pixels, so the paused frame's texture is drawn at the window's own resolution.
            readonly property real pixelScale: canvas.unit
            // The shelf's last focused game, which the top shows when nothing is suspended (the shelf is then the games played).
            property int shelfRow: 0
            property var shelfCover: null
            readonly property var topItem: !Session.hasGame && shelf.count > 0 ? shelf.itemAt(Math.min(root.shelfRow, shelf.count - 1)) : null
            readonly property bool hasTop: Session.hasGame || root.topItem !== null
            readonly property string topTitle: Session.hasGame ? Session.title : root.topItem?.title ?? ""
            readonly property string topPicture: root.topItem?.hero ?? ""
            // The paused game's words follow the clock, as the program's clock says them again each minute.
            readonly property string topFacts: Session.hasGame ? (Session.pausedSince > 0 && Device.clock.length >= 0 ? Words.pausedFacts(Session.console, Session.pausedSince) : "") : root.topItem?.facts ?? ""
            readonly property real topHue: Session.hasGame ? Session.hue : root.topItem?.consoleHue ?? -1
            readonly property bool shelfFocused: shelf.count > 0 && root.shelfRow >= 0 && root.shelfRow < shelf.count && shelf.itemAt(root.shelfRow)?.activeFocus === true
            // The shelf's and the places' first items, as their Repeaters make them, so a list read anew is followed.
            property Item firstShelf: null
            property Item firstPlace: null
            Sky {
                objectName: "homeSky"
                anchors.fill: parent
                glowHue: root.topHue
                glowX: 0.2
                glowY: 0.42
            }
            Frame {
                id: frame
                anchors.fill: parent
                surfaceDrawn: false
                padFamily: Device.family
                helpFont: Qt.font({ family: faces.regular, pixelSize: 20 })
                helpSpacing: 40
                helpGlyphGap: 10
                focus: true
                controller.navigationShown: false
                help: Help.entries(continuePlaying.activeFocus ? Help.Continue : (root.shelfFocused ? Help.Game : Help.Place))
                Connections {
                    target: frame.controller
                    function onActionInvoked(key) { Actions.act(key) }
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
                        text: Words.statusLine(Device.controllers, Device.clock)
                        color: night.muted
                        font.family: faces.regular
                        font.pixelSize: 20
                        lineHeight: 24
                        lineHeightMode: Text.FixedHeight
                    }
                    Scene {
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
                        color: night.textStrong
                        font.family: faces.light
                        font.weight: Font.Light
                        font.pixelSize: 24
                        font.letterSpacing: 6
                        lineHeight: 28.8
                        lineHeightMode: Text.FixedHeight
                    }
                    Walkable {
                        id: continuePlaying
                        objectName: "continue"
                        controller: frame.controller
                        x: 52
                        y: 72
                        width: root.width - 104
                        height: 296
                        // Continue holds the focus while a game is suspended; otherwise the shelf's first game does, else the first place.
                        focus: Session.hasGame
                        activeFocusOnTab: Session.hasGame
                        Accessible.name: Words.continuePlaying
                        onActivated: if (Session.hasGame) Actions.play(Session.key)
                        // The game in hand, whose page X opens.
                        onActiveFocusChanged: if (continuePlaying.activeFocus && Session.hasGame) Actions.focus(Session.key)
                        KeyNavigation.down: root.firstShelf ?? root.firstPlace
                        Rectangle {
                            objectName: "continuePanel"
                            anchors.fill: parent
                            visible: Session.hasGame
                            radius: Tokens.tileRadius + 4
                            color: Tokens.surfaceRaised
                            // A panel: dark glass by night, light by day, its words the variant's own (EmuSen_Qt.md §3.4.1).
                            opacity: night.day ? 0.94 : 0.55
                        }
                        Rectangle {
                            objectName: "continueRing"
                            anchors.fill: parent
                            visible: continuePlaying.activeFocus
                            radius: Tokens.tileRadius + 4
                            color: "transparent"
                            border.color: Tokens.accent
                            border.width: Tokens.ringThickness
                        }
                        Rectangle {
                            objectName: "continuePicture"
                            x: 12
                            y: 12
                            width: 373
                            height: 280
                            radius: Tokens.tileRadius
                            color: Tokens.surface
                            visible: root.hasTop
                            RoundPicture {
                                objectName: "continueArt"
                                anchors.fill: parent
                                visible: !Session.hasGame
                                source: root.topPicture
                                radius: Tokens.tileRadius
                                crop: true
                                pixelScale: root.pixelScale
                            }
                            // The game's own picture drawn into a texture at the window's resolution, cut to the box's shape at its centre as Mistress's poster slot cuts the paused frame, so its corners round as a cover's.
                            ShaderEffectSource {
                                id: pausedTexture
                                readonly property Item frame: Session.frame
                                readonly property real cutWidth: pausedTexture.frame ? Math.min(pausedTexture.frame.width, pausedTexture.frame.height * 373 / 280) : 0
                                readonly property real cutHeight: pausedTexture.cutWidth * 280 / 373
                                sourceItem: Session.hasGame && root.visible ? pausedTexture.frame : null
                                hideSource: true
                                visible: false
                                sourceRect: pausedTexture.frame ? Qt.rect((pausedTexture.frame.width - pausedTexture.cutWidth) / 2, (pausedTexture.frame.height - pausedTexture.cutHeight) / 2, pausedTexture.cutWidth, pausedTexture.cutHeight) : Qt.rect(0, 0, 0, 0)
                                textureSize: Qt.size(Math.ceil(373 * root.pixelScale), Math.ceil(280 * root.pixelScale))
                            }
                            Shape {
                                id: pausedRounded
                                objectName: "continueFrame"
                                anchors.fill: parent
                                visible: Session.hasGame && Session.frame !== null
                                preferredRendererType: Shape.CurveRenderer
                                ShapePath {
                                    strokeColor: "transparent"
                                    fillColor: "transparent"
                                    fillItem: pausedTexture
                                    fillTransform: PlanarTransform.fromScale(1 / root.pixelScale, 1 / root.pixelScale)
                                    PathRectangle {
                                        width: 373
                                        height: 280
                                        radius: Tokens.tileRadius
                                    }
                                }
                            }
                        }
                        Text {
                            objectName: "continueHeading"
                            visible: Session.hasGame
                            x: 409
                            y: 40
                            text: Words.continuePlaying
                            color: Tokens.muted
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
                            // On Continue's panel while a game is suspended, else on the world beside the shelf's game.
                            color: Session.hasGame ? Tokens.textStrong : night.textStrong
                            font.family: faces.condensed
                            font.weight: Font.Medium
                            font.pixelSize: 44
                            lineHeight: 52.8
                            lineHeightMode: Text.FixedHeight
                        }
                        Body {
                            objectName: "continuePlanet"
                            x: 409
                            y: 187
                            width: 14
                            height: 14
                            visible: Session.hasGame && root.topHue >= 0
                            hue: root.topHue
                        }
                        Text {
                            objectName: "continueFacts"
                            x: Session.hasGame && root.topHue >= 0 ? 433 : 409
                            y: 180
                            width: parent.width - x - 12
                            text: root.topFacts
                            elide: Text.ElideRight
                            color: Session.hasGame ? Tokens.muted : night.muted
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
                        color: night.muted
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
                            // The games played, the suspended one left out since Continue holds it.
                            model: Library.shelf
                            onItemAdded: (index, item) => root.firstShelf = index === 0 ? item : root.firstShelf
                            ShelfCover {
                                id: cover
                                controller: frame.controller
                                objectName: "shelfCover." + cover.index
                                visible: cover.x + width <= shelfBox.width
                                focus: !Session.hasGame && cover.index === 0
                                Accessible.name: cover.title
                                x: cover.index * 144
                                width: 120
                                height: 160
                                onActivated: Actions.play(cover.key)
                                onActiveFocusChanged: {
                                    if (!cover.activeFocus)
                                        return
                                    root.shelfRow = cover.index
                                    root.shelfCover = cover
                                    Actions.focus(cover.key)
                                }
                                KeyNavigation.left: cover.index > 0 ? shelf.itemAt(cover.index - 1) : null
                                KeyNavigation.right: shelf.count > cover.index + 1 ? shelf.itemAt(cover.index + 1) : null
                                KeyNavigation.up: Session.hasGame ? continuePlaying : null
                                KeyNavigation.down: places.count === 0 ? null : places.itemAt(Math.min(places.count - 1, Math.round(cover.index * 144 / root.placePitch)))
                                CoverTile {
                                    anchors.fill: parent
                                    source: cover.cover
                                    title: cover.title
                                    hue: cover.consoleHue
                                    favourite: cover.favourite
                                    mark: Marks.heart
                                    lit: cover.activeFocus
                                    pixelScale: root.pixelScale
                                }
                            }
                        }
                    }
                    Text {
                        objectName: "shelfTitle"
                        x: 64
                        // Clear of the focused cover's ring, which a 16:9 window draws closer.
                        y: 592
                        width: root.width - 128
                        visible: root.shelfFocused
                        text: root.shelfFocused ? root.shelfCover?.title ?? "" : ""
                        elide: Text.ElideRight
                        color: night.text
                        font.family: faces.regular
                        font.pixelSize: 22
                        lineHeight: 26.4
                        lineHeightMode: Text.FixedHeight
                    }
                    Repeater {
                        id: places
                        model: Library.places
                        onItemAdded: (index, item) => root.firstPlace = index === 0 ? item : root.firstPlace
                        Walkable {
                            id: place
                            required property int index
                            required property string key
                            required property string name
                            required property string line
                            required property bool attention
                            // A place the host declines is still walked to, its words drawn at 38% (EmuSen_Qt.md §3.4.1).
                            required property bool available
                            controller: frame.controller
                            objectName: "place." + place.index
                            focus: !Session.hasGame && shelf.count === 0 && place.index === 0
                            Accessible.name: place.name
                            Accessible.description: place.line
                            x: 64 + place.index * root.placePitch
                            y: 622
                            width: root.placeWidth
                            height: 64
                            onActivated: Actions.openPlace(place.key)
                            KeyNavigation.left: places.count > place.index ? places.itemAt(place.index - 1) : null
                            KeyNavigation.right: places.count > place.index ? places.itemAt(place.index + 1) : null
                            KeyNavigation.up: root.firstShelf ?? continuePlaying
                            Rectangle {
                                objectName: "placeTile." + place.index
                                anchors.fill: parent
                                radius: Tokens.tileRadius
                                color: Tokens.surfaceRaised
                                opacity: night.day ? 0.94 : 0.5
                            }
                            Rectangle {
                                anchors.fill: parent
                                radius: Tokens.tileRadius
                                color: "transparent"
                                border.color: Tokens.border
                                border.width: 1
                                opacity: 0.4
                            }
                            Rectangle {
                                anchors.fill: parent
                                anchors.margins: -3
                                visible: place.activeFocus
                                radius: Tokens.tileRadius + 3
                                color: "transparent"
                                border.color: Tokens.accent
                                border.width: Tokens.ringThickness
                            }
                            Text {
                                objectName: "placeName." + place.index
                                x: 12
                                y: 8
                                width: parent.width - 24
                                text: place.name
                                elide: Text.ElideRight
                                opacity: place.available ? 1 : 0.38
                                color: Tokens.textStrong
                                font.family: faces.regular
                                font.pixelSize: 20
                                lineHeight: 24
                                lineHeightMode: Text.FixedHeight
                            }
                            Row {
                                x: 12
                                y: 32
                                spacing: 8
                                opacity: place.available ? 1 : 0.38
                                Glyph {
                                    objectName: "placeAttention." + place.index
                                    visible: place.attention
                                    anchors.verticalCenter: parent.verticalCenter
                                    width: 16
                                    height: 16
                                    path: Marks.attention
                                    filled: true
                                    ink: Tokens.attention
                                }
                                Text {
                                    objectName: "placeStatus." + place.index
                                    // To 4 units short of the tile's edge, condensed so the program's longest line, "Light · 100% sound", is whole (EmuSen_Qt.md §3.4.1).
                                    width: place.width - 16 - (place.attention ? 24 : 0)
                                    text: place.line
                                    elide: Text.ElideRight
                                    color: place.attention ? Tokens.attention : Tokens.muted
                                    font.family: faces.condensedRegular
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
            Night {
                id: night
            }
        }
    }
}
