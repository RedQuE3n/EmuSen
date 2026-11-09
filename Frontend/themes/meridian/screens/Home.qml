pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Effects
import QtQuick.Shapes
import EmuSen.Theme
import "../parts"

// Meridian's Home: the glass rail of destinations at the left, the band showing what the focused entry holds, Continue's paused frame, the Recent shelf, the Games preview or a place (meridian.md §2).
ThemeScreen {
    id: screen
    objectName: "homeScreen"
    role: ThemeScreen.Home
    focusRole: ThemeScreen.Navigation
    // Where the focus starts and where Start on Home takes it back: Continue, else Recent with a library, else Games.
    focusItems: Session.hasGame ? [continueEntry] : (Library.consoles.count > 0 ? [recentEntry] : [root.firstPlace ?? recentEntry])
    // A game of the shelf, as its row gives it.
    component ShelfCover: Walkable {
        required property int index
        required property string key
        required property string title
        required property string cover
        required property real coverShape
        required property real consoleHue
        required property bool favourite
        required property string facts
    }
    // One of Home's places, as its row gives it.
    component PlaceEntry: RailEntry {
        required property int index
        required property string key
        required name
        required line
        required attention
        required available
    }
    // A world of the Games preview, as its row gives it.
    component PreviewWorld: Item {
        required property string name
        required property string body
        required property real hue
        required property int count
        required property int groupStart
        required property int groupIndex
    }
    NormalizedCanvas {
        id: canvas
        objectName: "homeCanvas"
        FocusScope {
            id: root
            // First, so the parts read the faces from their first binding.
            Faces {
                id: faces
            }
            objectName: "home"
            anchors.fill: parent
            focus: true
            readonly property real pixelScale: canvas.unit
            // The band: x 376 to W - 64.
            readonly property real bandX: 376
            readonly property real bandWidth: root.width - 440
            // The rail's entry the band shows: 0 Continue, 1 Recent, 2 + i the place i (Games is place 0).
            property int railAt: Session.hasGame ? 0 : 1
            readonly property bool shelfHeld: shelfList.activeFocus
            readonly property var shelfItem: shelfList.currentItem
            property Item firstPlace: null
            // The Continue frame: at most 840 wide, at 840 x 472's shape, centred in the band.
            readonly property real frameWidth: Math.min(840, root.bandWidth)
            readonly property real frameHeight: root.frameWidth * 472 / 840
            readonly property real cols: Math.max(1, Math.floor((root.bandWidth + 12) / 140))
            // Every game, the All shelf's count, and the Recent shelf's key for its mark (All, Favourites, Recent in the library's order).
            property int allGames: 0
            property string recentKey
            Backdrop {
                objectName: "homeSky"
                anchors.fill: parent
                glowHue: root.railAt === 0 ? Session.hue : (root.railAt === 1 && root.shelfItem ? root.shelfItem.consoleHue : -1)
                glowX: (root.bandX + root.bandWidth / 2) / Math.max(1, root.width)
                glowY: (root.railAt === 0 ? 330 : 250) / 800
            }
            // The library's All shelf, read for its count alone.
            Repeater {
                id: allShelf
                model: Library.shelves
                Item {
                    id: shelfRow
                    required property int index
                    required property int count
                    required property string key
                    Binding {
                        root.allGames: shelfRow.count
                        when: shelfRow.index === 0
                    }
                    Binding {
                        root.recentKey: shelfRow.key
                        when: shelfRow.index === 2
                    }
                }
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
                help: continueEntry.activeFocus ? Help.entries(Help.Continue) : (root.shelfHeld ? Help.entries(Help.Game, { back: true }) : Help.entries(Help.Place))
                Connections {
                    target: frame.controller
                    function onActionInvoked(key) { Actions.act(key) }
                }
                // B on the shelf returns to the rail's Recent.
                Connections {
                    target: frame.controller
                    enabled: root.shelfHeld
                    function onBackDeclined() { recentEntry.forceActiveFocus() }
                }
                Item {
                    x: -frame.safeX
                    y: -frame.safeY
                    width: root.width
                    height: root.height
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
                            ink: Tokens.variant === Tokens.Light ? "#55555F" : "#E9E4D4"
                        }
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
                    Glass {
                        objectName: "rail"
                        x: 64
                        y: 96
                        width: 280
                        height: 630
                    }
                    Column {
                        objectName: "railEntries"
                        x: 72
                        y: 112
                        RailEntry {
                            id: continueEntry
                            objectName: "entry.continue"
                            controller: frame.controller
                            faces: faces
                            visible: Session.hasGame
                            focus: Session.hasGame
                            activeFocusOnTab: Session.hasGame
                            name: Words.continueEntry
                            line: Session.pausedSince > 0 && Device.clock.length >= 0 ? Words.pausedAgo(Session.pausedSince) : ""
                            mark: Marks.crescent
                            held: root.railAt === 0 && !continueEntry.activeFocus
                            onActivated: Actions.play(Session.key)
                            onActiveFocusChanged: if (continueEntry.activeFocus) root.railAt = 0
                            KeyNavigation.down: recentEntry
                        }
                        RailEntry {
                            id: recentEntry
                            objectName: "entry.recent"
                            controller: frame.controller
                            faces: faces
                            focus: !Session.hasGame && Library.consoles.count > 0
                            name: Words.recentEntry
                            line: shelfList.count > 0 ? Words.games(shelfList.count) : Words.noRecent
                            mark: Marks.path(root.recentKey)
                            held: root.railAt === 1 && root.shelfHeld
                            onActivated: if (shelfList.count > 0) shelfList.forceActiveFocus()
                            onActiveFocusChanged: if (recentEntry.activeFocus) root.railAt = 1
                            Keys.onRightPressed: if (shelfList.count > 0) shelfList.forceActiveFocus()
                            KeyNavigation.up: Session.hasGame ? continueEntry : null
                            KeyNavigation.down: root.firstPlace
                        }
                        Repeater {
                            id: places
                            model: Library.places
                            onItemAdded: (index, item) => root.firstPlace = index === 0 ? item : root.firstPlace
                            PlaceEntry {
                                id: place
                                objectName: "entry.place." + place.index
                                controller: frame.controller
                                faces: faces
                                focus: !Session.hasGame && Library.consoles.count === 0 && place.index === 0
                                mark: Marks.path(place.key)
                                held: false
                                onActivated: Actions.openPlace(place.key)
                                onActiveFocusChanged: if (place.activeFocus) root.railAt = 2 + place.index
                                KeyNavigation.up: place.index > 0 ? places.itemAt(place.index - 1) : recentEntry
                                KeyNavigation.down: places.count > place.index + 1 ? places.itemAt(place.index + 1) : null
                            }
                        }
                    }
                    Text {
                        objectName: "homeStatus"
                        x: 80
                        // At 19 units the line box ends 16 above the rail's foot, as the canvas's 17-unit line sits.
                        y: 688
                        width: 248
                        text: Words.statusLine(Device.controllers, Device.clock)
                        elide: Text.ElideRight
                        color: Tokens.muted
                        font.family: faces.regular
                        font.pixelSize: 19
                        lineHeight: 22.8
                        lineHeightMode: Text.FixedHeight
                    }

                    // The band: Continue.
                    Item {
                        id: continueBand
                        objectName: "band.continue"
                        anchors.fill: parent
                        opacity: root.railAt === 0 && Session.hasGame ? 1 : 0
                        visible: continueBand.opacity > 0
                        Behavior on opacity {
                            NumberAnimation {
                                duration: Tokens.reducedMotion ? 0 : Tokens.fade
                                easing.type: Tokens.fadeCurve
                            }
                        }
                        RectangularShadow {
                            anchors.fill: pausedBox
                            offset.y: 10
                            blur: 24
                            radius: 12
                            color: "#73000000"
                        }
                        Item {
                            id: pausedBox
                            objectName: "continueFrame"
                            x: root.bandX + (root.bandWidth - root.frameWidth) / 2
                            y: 96
                            width: root.frameWidth
                            height: root.frameHeight
                            Rectangle {
                                anchors.fill: parent
                                radius: 12
                                color: Tokens.voidFill
                            }
                            // The game's own picture drawn into a texture at the window's resolution, cut to the frame's shape at its centre, its corners rounded as the frame's.
                            ShaderEffectSource {
                                id: pausedTexture
                                readonly property Item frame: Session.frame
                                readonly property real cutWidth: pausedTexture.frame ? Math.min(pausedTexture.frame.width, pausedTexture.frame.height * 840 / 472) : 0
                                readonly property real cutHeight: pausedTexture.cutWidth * 472 / 840
                                sourceItem: Session.hasGame && root.visible ? pausedTexture.frame : null
                                hideSource: true
                                visible: false
                                sourceRect: pausedTexture.frame ? Qt.rect((pausedTexture.frame.width - pausedTexture.cutWidth) / 2, (pausedTexture.frame.height - pausedTexture.cutHeight) / 2, pausedTexture.cutWidth, pausedTexture.cutHeight) : Qt.rect(0, 0, 0, 0)
                                textureSize: Qt.size(Math.ceil(root.frameWidth * root.pixelScale), Math.ceil(root.frameHeight * root.pixelScale))
                            }
                            Shape {
                                objectName: "continuePicture"
                                anchors.fill: parent
                                visible: Session.hasGame && Session.frame !== null
                                preferredRendererType: Shape.CurveRenderer
                                ShapePath {
                                    strokeColor: "transparent"
                                    fillColor: "transparent"
                                    fillItem: pausedTexture
                                    fillTransform: PlanarTransform.fromScale(1 / root.pixelScale, 1 / root.pixelScale)
                                    PathRectangle {
                                        width: root.frameWidth
                                        height: root.frameHeight
                                        radius: 12
                                    }
                                }
                            }
                            Rectangle {
                                objectName: "continueOnArt"
                                x: 0
                                y: parent.height - 120
                                width: parent.width
                                height: 120
                                topLeftRadius: 0
                                topRightRadius: 0
                                bottomLeftRadius: 12
                                bottomRightRadius: 12
                                gradient: Gradient {
                                    GradientStop { position: 0; color: "transparent" }
                                    GradientStop { position: 1; color: Tokens.overArtScrim }
                                }
                            }
                            Chip {
                                objectName: "continueChip"
                                x: 16
                                y: parent.height - 14 - 26
                                faces: faces
                                kind: Help.Continue
                                onArt: true
                            }
                        }
                        Column {
                            x: pausedBox.x
                            y: pausedBox.y + pausedBox.height + 16
                            width: pausedBox.width
                            spacing: 8
                            Text {
                                objectName: "continueEyebrow"
                                text: Words.continuePlaying
                                color: Tokens.muted
                                font.family: faces.regular
                                font.pixelSize: 20
                                font.letterSpacing: 2
                                lineHeight: 24
                                lineHeightMode: Text.FixedHeight
                            }
                            Text {
                                objectName: "continueTitle"
                                width: parent.width
                                text: Session.title
                                elide: Text.ElideRight
                                color: Tokens.textStrong
                                font.family: faces.condensed
                                font.weight: Font.Medium
                                font.pixelSize: 44
                                lineHeight: 52.8
                                lineHeightMode: Text.FixedHeight
                            }
                            Item {
                                width: parent.width
                                height: 26.4
                                Body {
                                    objectName: "continueWorld"
                                    y: 6
                                    width: 14
                                    height: 14
                                    visible: Session.hue >= 0
                                    hue: Session.hue
                                }
                                Text {
                                    objectName: "continueFacts"
                                    x: Session.hue >= 0 ? 24 : 0
                                    width: parent.width - x
                                    text: Session.pausedSince > 0 && Device.clock.length >= 0 ? Words.pausedFacts(Session.console, Session.pausedSince) : ""
                                    elide: Text.ElideRight
                                    color: Tokens.muted
                                    font.family: faces.regular
                                    font.pixelSize: 22
                                    lineHeight: 26.4
                                    lineHeightMode: Text.FixedHeight
                                }
                            }
                        }
                    }

                    // The band: Recent, its shelf of whole covers.
                    Item {
                        id: recentBand
                        objectName: "band.recent"
                        anchors.fill: parent
                        opacity: root.railAt === 1 ? 1 : 0
                        visible: recentBand.opacity > 0
                        Behavior on opacity {
                            NumberAnimation {
                                duration: Tokens.reducedMotion ? 0 : Tokens.fade
                                easing.type: Tokens.fadeCurve
                            }
                        }
                        Text {
                            objectName: "recentEyebrow"
                            x: root.bandX
                            y: 96
                            text: Words.recent
                            color: Tokens.muted
                            font.family: faces.regular
                            font.pixelSize: 20
                            font.letterSpacing: 2
                            lineHeight: 24
                            lineHeightMode: Text.FixedHeight
                        }
                        ListView {
                            id: shelfList
                            objectName: "shelf"
                            x: root.bandX - 16
                            y: 150
                            width: root.bandWidth + 32
                            height: 230
                            orientation: ListView.Horizontal
                            leftMargin: 16
                            rightMargin: 16
                            spacing: 24
                            clip: true
                            keyNavigationEnabled: true
                            keyNavigationWraps: false
                            highlightFollowsCurrentItem: true
                            highlightMoveDuration: Tokens.reel
                            highlight: Item {}
                            boundsBehavior: Flickable.StopAtBounds
                            property bool fadeLeft: false
                            property bool fadeRight: false
                            model: Library.shelf
                            Keys.priority: Keys.AfterItem
                            Keys.onLeftPressed: recentEntry.forceActiveFocus()
                            delegate: ShelfCover {
                                id: cover
                                readonly property bool lit: cover.ListView.isCurrentItem && shelfList.activeFocus
                                controller: frame.controller
                                objectName: "shelfCover." + cover.index
                                Accessible.name: cover.title
                                focus: cover.ListView.isCurrentItem
                                y: 10
                                width: Math.round(200 * Math.min(1.6, Math.max(0.5, cover.coverShape > 0 ? cover.coverShape : 0.75)))
                                height: 200
                                z: cover.lit ? 1 : 0
                                onActivated: Actions.play(cover.key)
                                onActiveFocusChanged: {
                                    if (!cover.activeFocus)
                                        return
                                    shelfList.currentIndex = cover.index
                                    Actions.focus(cover.key)
                                }
                                CoverTile {
                                    anchors.fill: parent
                                    source: cover.cover
                                    title: cover.title
                                    hue: cover.consoleHue
                                    favourite: cover.favourite
                                    mark: Marks.heart
                                    lit: cover.lit
                                    pixelScale: root.pixelScale
                                }
                            }
                        }
                        Track {
                            objectName: "shelfTrack"
                            horizontal: true
                            x: root.bandX
                            y: 392
                            width: root.bandWidth
                            height: 2
                            shown: shelfList.contentWidth > 0 ? shelfList.width / (shelfList.contentWidth + 32) : 1
                            at: shelfList.contentWidth > 0 ? (shelfList.contentX + 16) / (shelfList.contentWidth + 32) : 0
                        }
                        Column {
                            x: root.bandX
                            y: 408
                            width: root.bandWidth
                            spacing: 8
                            visible: shelfList.count > 0
                            Text {
                                objectName: "shelfTitle"
                                width: parent.width
                                text: root.shelfItem ? root.shelfItem.title : ""
                                elide: Text.ElideRight
                                color: Tokens.textStrong
                                font.family: faces.condensed
                                font.weight: Font.Medium
                                font.pixelSize: 32
                                lineHeight: 38.4
                                lineHeightMode: Text.FixedHeight
                            }
                            Item {
                                width: parent.width
                                height: 26.4
                                Body {
                                    objectName: "shelfWorld"
                                    y: 6
                                    width: 14
                                    height: 14
                                    visible: root.shelfItem !== null && root.shelfItem.consoleHue >= 0
                                    hue: root.shelfItem ? root.shelfItem.consoleHue : -1
                                }
                                Text {
                                    objectName: "shelfFacts"
                                    x: 24
                                    width: parent.width - 24
                                    text: root.shelfItem ? root.shelfItem.facts : ""
                                    elide: Text.ElideRight
                                    color: Tokens.muted
                                    font.family: faces.regular
                                    font.pixelSize: 22
                                    lineHeight: 26.4
                                    lineHeightMode: Text.FixedHeight
                                }
                            }
                        }
                        Column {
                            x: root.bandX
                            y: 150
                            width: root.bandWidth
                            spacing: 8
                            visible: shelfList.count === 0
                            Text {
                                objectName: "shelfEmpty"
                                width: parent.width
                                text: Words.noRecent
                                elide: Text.ElideRight
                                color: Tokens.textStrong
                                font.family: faces.condensed
                                font.weight: Font.Medium
                                font.pixelSize: 44
                                lineHeight: 52.8
                                lineHeightMode: Text.FixedHeight
                            }
                            Text {
                                objectName: "shelfAddFolder"
                                width: parent.width
                                text: Words.addFolder
                                elide: Text.ElideRight
                                color: Tokens.muted
                                font.family: faces.regular
                                font.pixelSize: 22
                                lineHeight: 26.4
                                lineHeightMode: Text.FixedHeight
                            }
                        }
                    }

                    // The band: Games, a read-only preview of Choose a console.
                    Item {
                        id: gamesBand
                        objectName: "band.games"
                        anchors.fill: parent
                        opacity: root.railAt === 2 ? 1 : 0
                        visible: gamesBand.opacity > 0
                        Behavior on opacity {
                            NumberAnimation {
                                duration: Tokens.reducedMotion ? 0 : Tokens.fade
                                easing.type: Tokens.fadeCurve
                            }
                        }
                        Column {
                            x: root.bandX
                            y: 96
                            width: root.bandWidth
                            spacing: 8
                            Text {
                                objectName: "gamesEyebrow"
                                text: Words.gamesHeading
                                color: Tokens.muted
                                font.family: faces.regular
                                font.pixelSize: 20
                                font.letterSpacing: 2
                                lineHeight: 24
                                lineHeightMode: Text.FixedHeight
                            }
                            Text {
                                objectName: "gamesTitle"
                                width: parent.width
                                text: Library.consoles.count > 0 ? Words.chooseAConsole : Words.noConsoles
                                elide: Text.ElideRight
                                color: Tokens.textStrong
                                font.family: faces.condensed
                                font.weight: Font.Medium
                                font.pixelSize: 44
                                lineHeight: 52.8
                                lineHeightMode: Text.FixedHeight
                            }
                            Text {
                                objectName: "gamesSummary"
                                width: parent.width
                                text: Library.consoles.count > 0 ? Words.summary(Library.consoles.count, root.allGames) : Words.addFolder
                                elide: Text.ElideRight
                                color: Tokens.muted
                                font.family: faces.regular
                                font.pixelSize: 22
                                lineHeight: 26.4
                                lineHeightMode: Text.FixedHeight
                            }
                        }
                        Repeater {
                            model: Library.consoles
                            PreviewWorld {
                                id: world
                                readonly property int place: world.groupStart + world.groupIndex
                                readonly property int row: Math.floor(world.place / root.cols)
                                objectName: "previewWorld." + world.place
                                visible: world.row < 2
                                x: root.bandX + (world.place % root.cols) * 140
                                y: 300 + world.row * 170
                                width: 128
                                height: 150
                                Body {
                                    x: 32
                                    width: 64
                                    height: 64
                                    world: world.body
                                    hue: world.hue
                                }
                                Text {
                                    objectName: "previewName." + world.place
                                    y: 72
                                    width: 128
                                    horizontalAlignment: Text.AlignHCenter
                                    text: world.name
                                    elide: Text.ElideRight
                                    color: Tokens.text
                                    font.family: faces.condensed
                                    font.weight: Font.Medium
                                    font.pixelSize: 20
                                    font.letterSpacing: 1.2
                                    font.capitalization: Font.AllUppercase
                                    lineHeight: 24
                                    lineHeightMode: Text.FixedHeight
                                }
                                Text {
                                    y: 96
                                    width: 128
                                    horizontalAlignment: Text.AlignHCenter
                                    text: Words.games(world.count)
                                    elide: Text.ElideRight
                                    color: Tokens.muted
                                    font.family: faces.regular
                                    font.pixelSize: 19
                                    lineHeight: 22.8
                                    lineHeightMode: Text.FixedHeight
                                }
                            }
                        }
                        Chip {
                            objectName: "gamesChip"
                            x: root.bandX
                            y: 690
                            faces: faces
                            kind: Help.Place
                        }
                    }

                    // The band: another place, its name and its line.
                    Repeater {
                        model: Library.places
                        Item {
                            id: placeBand
                            required property int index
                            required property string name
                            required property string line
                            required property bool attention
                            objectName: "band.place." + placeBand.index
                            anchors.fill: parent
                            opacity: placeBand.index > 0 && root.railAt === 2 + placeBand.index ? 1 : 0
                            visible: placeBand.opacity > 0
                            Behavior on opacity {
                                NumberAnimation {
                                    duration: Tokens.reducedMotion ? 0 : Tokens.fade
                                    easing.type: Tokens.fadeCurve
                                }
                            }
                            Column {
                                x: root.bandX
                                y: 296
                                width: root.bandWidth
                                spacing: 8
                                Text {
                                    text: Words.places
                                    color: Tokens.muted
                                    font.family: faces.regular
                                    font.pixelSize: 20
                                    font.letterSpacing: 2
                                    lineHeight: 24
                                    lineHeightMode: Text.FixedHeight
                                }
                                Text {
                                    objectName: "placeTitle." + placeBand.index
                                    width: parent.width
                                    text: placeBand.name
                                    elide: Text.ElideRight
                                    color: Tokens.textStrong
                                    font.family: faces.condensed
                                    font.weight: Font.Medium
                                    font.pixelSize: 44
                                    lineHeight: 52.8
                                    lineHeightMode: Text.FixedHeight
                                }
                                Row {
                                    spacing: 8
                                    Glyph {
                                        visible: placeBand.attention
                                        anchors.verticalCenter: parent.verticalCenter
                                        width: 20
                                        height: 20
                                        path: Marks.attention
                                        filled: true
                                        ink: Tokens.attention
                                    }
                                    Text {
                                        objectName: "placeLine." + placeBand.index
                                        width: root.bandWidth - (placeBand.attention ? 28 : 0)
                                        text: placeBand.line
                                        elide: Text.ElideRight
                                        color: placeBand.attention ? Tokens.attention : Tokens.muted
                                        font.family: faces.regular
                                        font.pixelSize: 26
                                        lineHeight: 31.2
                                        lineHeightMode: Text.FixedHeight
                                    }
                                }
                            }
                            Chip {
                                objectName: "placeChip." + placeBand.index
                                x: root.bandX
                                y: 444
                                faces: faces
                                kind: Help.Place
                            }
                        }
                    }
                }
            }
        }
    }
}
