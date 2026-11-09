pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Effects
import QtQuick.Shapes
import EmuSen.Theme
import "../parts"

// Horizon's Home, Rise: the focused game's world risen on the horizon at the right, its words in the sky, Continue, the recent covers and the places on the land's foot line under the clamped camera (EmuSen_Qt.md §3.4.2).
ThemeScreen {
    id: screen
    objectName: "horizonHome"
    role: ThemeScreen.Home
    // Continue while a game is suspended, else the first recent cover, else the first place.
    focusItems: Session.hasGame ? [continuePlaying] : [root.firstShelf ?? root.firstPlace]
    NormalizedCanvas {
        id: canvas
        objectName: "homeCanvas"
        FocusScope {
            id: root
            objectName: "home"
            anchors.fill: parent
            focus: true
            readonly property real pixelScale: canvas.unit
            // The strip's item the focus is on, read through the roles every item kind declares.
            property var focused: null
            property Item firstShelf: null
            property Item firstPlace: null
            readonly property int kind: root.focused ? root.focused.kind : 2
            readonly property real hue: root.focused ? root.focused.hue : -1
            readonly property real wordsWidth: Math.min(820, root.width - 480)
            // The suspended game's hero, from its row among the games played.
            property string continueHero: ""
            Repeater {
                model: Library.recent
                Item {
                    id: played
                    required property string key
                    required property string hero
                    Binding {
                        root.continueHero: played.hero
                        when: Session.hasGame && played.key === Session.key
                    }
                }
            }
            Backdrop {
                anchors.fill: parent
                hue: root.kind === 2 ? -1 : root.hue
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
                help: Help.entries(root.kind === 0 ? Help.Continue : (root.kind === 1 ? Help.Game : Help.Place), { group: true, accept: root.focused ? root.focused.available : true })
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
                        color: Tokens.muted
                        font.family: faces.regular
                        font.pixelSize: 20
                        lineHeight: 24
                        lineHeightMode: Text.FixedHeight
                    }
                    Glyph {
                        objectName: "crescent"
                        x: wordmark.x - 12 - width
                        y: 38
                        width: 26
                        height: 26
                        path: Marks.crescent
                        filled: true
                        ink: Tokens.variant === Tokens.Dark ? "#E9E4D4" : Tokens.muted
                    }
                    Text {
                        id: wordmark
                        objectName: "wordmark"
                        x: root.width - 64 - width
                        y: 37.6
                        text: Words.title
                        color: Tokens.textStrong
                        font.family: faces.light
                        font.weight: Font.Light
                        font.pixelSize: 24
                        font.letterSpacing: 6
                        lineHeight: 28.8
                        lineHeightMode: Text.FixedHeight
                    }
                    World {
                        x: root.width - 364
                        y: 140
                        key: root.focused ? root.focused.key : ""
                        place: root.kind === 2
                        picture: root.focused ? root.focused.picture : ""
                        mark: root.focused ? root.focused.mark : ""
                        hue: root.hue
                        pixelScale: root.pixelScale
                    }
                    Column {
                        objectName: "homeWords"
                        x: 64
                        y: 408 - height
                        width: root.wordsWidth
                        spacing: 8
                        Text {
                            objectName: "homeEyebrow"
                            text: root.kind === 0 ? Words.continuePlaying : (root.kind === 1 ? Words.recent : Words.places)
                            color: Tokens.muted
                            font.family: faces.regular
                            font.pixelSize: 20
                            font.letterSpacing: 2
                            lineHeight: 24
                            lineHeightMode: Text.FixedHeight
                        }
                        Text {
                            objectName: "homeTitle"
                            width: parent.width
                            text: root.focused ? root.focused.title : ""
                            wrapMode: root.kind === 2 ? Text.NoWrap : Text.Wrap
                            maximumLineCount: 2
                            elide: Text.ElideRight
                            color: Tokens.textStrong
                            font.family: faces.condensed
                            font.weight: Font.Medium
                            font.pixelSize: 44
                            lineHeight: 52.8
                            lineHeightMode: Text.FixedHeight
                        }
                        Row {
                            objectName: "homeFacts"
                            visible: root.kind !== 2
                            spacing: 10
                            Body {
                                anchors.verticalCenter: parent.verticalCenter
                                width: 14
                                height: 14
                                visible: root.hue >= 0
                                hue: root.hue
                            }
                            Text {
                                width: root.wordsWidth - 24
                                text: root.focused ? root.focused.facts : ""
                                elide: Text.ElideRight
                                color: Tokens.muted
                                font.family: faces.regular
                                font.pixelSize: 22
                                lineHeight: 26.4
                                lineHeightMode: Text.FixedHeight
                            }
                        }
                        Row {
                            objectName: "homePlaceLine"
                            visible: root.kind === 2
                            spacing: 8
                            Glyph {
                                anchors.verticalCenter: parent.verticalCenter
                                visible: root.focused ? root.focused.attention : false
                                width: 22
                                height: 22
                                path: Marks.attention
                                filled: true
                                ink: Tokens.attention
                            }
                            Text {
                                width: root.wordsWidth - 30
                                text: root.focused ? root.focused.line : ""
                                elide: Text.ElideRight
                                color: root.focused && root.focused.attention ? Tokens.attention : Tokens.muted
                                font.family: faces.regular
                                font.pixelSize: 26
                                lineHeight: 31.2
                                lineHeightMode: Text.FixedHeight
                            }
                        }
                    }
                    HorizonLine {
                        width: root.width
                        total: strip.total
                        off: strip.off
                    }
                    Strip {
                        id: strip
                        width: root.width
                        total: row.width
                        focusX: root.focused ? root.focused.stripX : 0
                        focusWidth: root.focused ? root.focused.width : 0
                        onStepped: (by) => (by > 0 ? root.focused?.rightItem : root.focused?.leftItem)?.forceActiveFocus()
                        Row {
                            id: row
                            spacing: 28
                            StripItem {
                                id: continuePlaying
                                objectName: "continue"
                                controller: frame.controller
                                strip: strip
                                visible: Session.hasGame
                                focus: Session.hasGame
                                activeFocusOnTab: Session.hasGame
                                kind: 0
                                key: Session.key
                                title: Session.title
                                picture: root.continueHero
                                hue: Session.hue
                                // The paused game's words follow the clock, said again as its minute turns.
                                facts: Session.hasGame && Session.pausedSince > 0 && Device.clock.length >= 0 ? Words.pausedFacts(Session.console, Session.pausedSince) : ""
                                stripX: continuePlaying.x
                                y: strip.foot - height
                                width: 304
                                height: 190
                                Accessible.name: Session.title
                                onActivated: Actions.play(Session.key)
                                onActiveFocusChanged: if (continuePlaying.activeFocus) root.focused = continuePlaying
                                rightItem: root.firstShelf ?? root.firstPlace
                                nextItem: root.firstShelf ?? root.firstPlace
                                RectangularShadow {
                                    x: 0
                                    y: 10
                                    width: 304
                                    height: 190
                                    radius: Tokens.tileRadius
                                    blur: 24
                                    color: "#73000000"
                                }
                                Rectangle {
                                    id: frameBox
                                    objectName: "continueFrame"
                                    width: 304
                                    height: 190
                                    radius: Tokens.tileRadius
                                    color: Tokens.voidFill
                                }
                                // The game's own picture at the window's resolution, cut to 16:10 at its centre and rounded as a cover.
                                ShaderEffectSource {
                                    id: pausedTexture
                                    readonly property Item frame: Session.frame
                                    readonly property real cutWidth: pausedTexture.frame ? Math.min(pausedTexture.frame.width, pausedTexture.frame.height * 304 / 190) : 0
                                    readonly property real cutHeight: pausedTexture.cutWidth * 190 / 304
                                    sourceItem: Session.hasGame && root.visible ? pausedTexture.frame : null
                                    hideSource: true
                                    visible: false
                                    sourceRect: pausedTexture.frame ? Qt.rect((pausedTexture.frame.width - pausedTexture.cutWidth) / 2, (pausedTexture.frame.height - pausedTexture.cutHeight) / 2, pausedTexture.cutWidth, pausedTexture.cutHeight) : Qt.rect(0, 0, 0, 0)
                                    textureSize: Qt.size(Math.ceil(304 * root.pixelScale), Math.ceil(190 * root.pixelScale))
                                }
                                Shape {
                                    objectName: "continuePicture"
                                    width: 304
                                    height: 190
                                    visible: Session.hasGame && Session.frame !== null
                                    preferredRendererType: Shape.CurveRenderer
                                    ShapePath {
                                        strokeColor: "transparent"
                                        fillColor: "transparent"
                                        fillItem: pausedTexture
                                        fillTransform: PlanarTransform.fromScale(1 / root.pixelScale, 1 / root.pixelScale)
                                        PathRectangle {
                                            width: 304
                                            height: 190
                                            radius: Tokens.tileRadius
                                        }
                                    }
                                }
                                Rectangle {
                                    objectName: "continueOnArt"
                                    y: 95
                                    width: 304
                                    height: 95
                                    radius: Tokens.tileRadius
                                    gradient: Gradient {
                                        GradientStop { position: 0; color: "transparent" }
                                        GradientStop { position: 1; color: "#B8000000" }
                                    }
                                }
                                Repeater {
                                    model: Help.entries(Help.Continue)
                                    Row {
                                        id: chip
                                        required property int index
                                        required property int button
                                        required property string label
                                        objectName: "continueChip"
                                        visible: chip.index === 0
                                        x: 12
                                        y: 190 - 10 - height
                                        spacing: 8
                                        PadGlyph {
                                            anchors.verticalCenter: parent.verticalCenter
                                            width: 26
                                            height: 26
                                            family: Device.family
                                            button: chip.button
                                            ink: "#FFFFFF"
                                        }
                                        Text {
                                            anchors.verticalCenter: parent.verticalCenter
                                            text: chip.label
                                            color: Tokens.overArt
                                            font.family: faces.regular
                                            font.pixelSize: 19
                                            lineHeight: 22.8
                                            lineHeightMode: Text.FixedHeight
                                        }
                                    }
                                }
                            }
                            Text {
                                objectName: "noRecent"
                                visible: Library.shelf.count === 0
                                y: strip.foot - height
                                text: Words.noRecent
                                color: Tokens.muted
                                font.family: faces.regular
                                font.pixelSize: 20
                                lineHeight: 24
                                lineHeightMode: Text.FixedHeight
                            }
                            Repeater {
                                id: shelf
                                // The games played, the suspended one left out since Continue holds it.
                                model: Library.shelf
                                onItemAdded: (index, item) => root.firstShelf = index === 0 ? item : root.firstShelf
                                StripItem {
                                    id: cover
                                    required property int index
                                    required key
                                    required title
                                    required facts
                                    required property string cover
                                    required property real coverShape
                                    required property real consoleHue
                                    required property bool favourite
                                    required property string hero
                                    controller: frame.controller
                                    strip: strip
                                    objectName: "shelfCover." + cover.index
                                    focus: !Session.hasGame && cover.index === 0
                                    kind: 1
                                    picture: cover.hero
                                    hue: cover.consoleHue
                                    stripX: cover.x
                                    y: strip.foot - height
                                    width: 176 * cover.coverShape
                                    height: 176
                                    Accessible.name: cover.title
                                    onActivated: Actions.play(cover.key)
                                    onActiveFocusChanged: if (cover.activeFocus) root.focused = cover
                                    leftItem: cover.index > 0 ? shelf.itemAt(cover.index - 1) : (Session.hasGame ? continuePlaying : null)
                                    rightItem: shelf.count > cover.index + 1 ? shelf.itemAt(cover.index + 1) : root.firstPlace
                                    previousItem: Session.hasGame ? continuePlaying : null
                                    nextItem: root.firstPlace
                                    Cover {
                                        anchors.fill: parent
                                        source: cover.cover
                                        title: cover.title
                                        hue: cover.consoleHue
                                        favourite: cover.favourite
                                        pixelScale: root.pixelScale
                                    }
                                }
                            }
                            Row {
                                id: placesRow
                                leftPadding: 24
                                spacing: 28
                                Repeater {
                                    id: places
                                    model: Library.places
                                    onItemAdded: (index, item) => root.firstPlace = index === 0 ? item : root.firstPlace
                                    StripItem {
                                        id: place
                                        required property int index
                                        required key
                                        required line
                                        required attention
                                        required available
                                        required property string name
                                        controller: frame.controller
                                        strip: strip
                                        objectName: "place." + place.index
                                        focus: !Session.hasGame && Library.shelf.count === 0 && place.index === 0
                                        kind: 2
                                        title: place.name
                                        mark: Marks.path(place.key)
                                        hue: -1
                                        stripX: placesRow.x + place.x
                                        y: strip.foot - height
                                        width: 112
                                        height: 88
                                        boxX: 28
                                        boxWidth: 56
                                        boxHeight: 56
                                        round: true
                                        Accessible.name: place.name
                                        Accessible.description: place.line
                                        onActivated: Actions.openPlace(place.key)
                                        onActiveFocusChanged: if (place.activeFocus) root.focused = place
                                        leftItem: place.index > 0 ? places.itemAt(place.index - 1) : (shelf.count > 0 ? shelf.itemAt(shelf.count - 1) : (Session.hasGame ? continuePlaying : null))
                                        rightItem: places.count > place.index + 1 ? places.itemAt(place.index + 1) : null
                                        previousItem: root.firstShelf ?? (Session.hasGame ? continuePlaying : null)
                                        PlaceDisc {
                                            x: 28
                                            mark: place.mark
                                            attention: place.attention
                                        }
                                        Text {
                                            objectName: "placeName." + place.index
                                            y: 64
                                            width: 112
                                            horizontalAlignment: Text.AlignHCenter
                                            text: place.name
                                            elide: Text.ElideRight
                                            color: Tokens.textStrong
                                            font.family: faces.condensed
                                            font.weight: Font.Medium
                                            font.pixelSize: 20
                                            font.letterSpacing: 1.2
                                            font.capitalization: Font.AllUppercase
                                            lineHeight: 24
                                            lineHeightMode: Text.FixedHeight
                                        }
                                    }
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
    }
}
