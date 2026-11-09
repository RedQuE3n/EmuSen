pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Theme
import "../parts"

// Home as a star chart: the paused game the brightest block, the four most recent covers a square of stars, the places on one line, and faint lines along every edge the pad can walk, brightening from the focus in its console's hue (constellation.md §2).
ThemeScreen {
    id: screen
    objectName: "homeScreen"
    role: ThemeScreen.Home
    // Where the focus starts and where Start on Home takes it back: the block while a game is suspended, else cover 1, else place 1.
    focusItems: Session.hasGame ? [block] : [root.cover1 ?? root.place0]
    NormalizedCanvas {
        id: canvas
        objectName: "homeCanvas"
        FocusScope {
            id: root
            objectName: "home"
            anchors.fill: parent
            focus: true
            readonly property real pixelScale: canvas.unit
            // The composition's shift at a wider window, and the places' pitch across the safe area.
            readonly property real ox: (root.width - 1280) / 2
            readonly property real pitch: (root.width - 128) / 7
            readonly property bool resting: !Tokens.reducedMotion && !(Device.battery >= 0 && !Device.charging)
            readonly property bool light: Tokens.variant === Tokens.Light
            // The four covers by their place in the square, and the places' first and each by index, as the Repeaters make them.
            property var cover1: null
            property var cover2: null
            property var cover3: null
            property var cover4: null
            property var place0: null
            // How many places the Repeater has made, so a lookup by index is read again as each is made.
            property int placesMade: 0
            // The focused node: its centre, its hue (-1 for a place) and, for a cover, its row.
            property var focused: null
            readonly property real focusHue: root.focused ? root.focused.hue : -1
            readonly property bool coverFocused: root.focused !== null && root.focused.isCover
            // The rest lines breathe 10% +- 2% over 4 s while resting.
            property real breath: 0
            SequentialAnimation on breath {
                running: root.resting && root.visible
                loops: Animation.Infinite
                NumberAnimation { from: 0; to: 1; duration: 2000; easing.type: Easing.InOutSine }
                NumberAnimation { from: 1; to: 0; duration: 2000; easing.type: Easing.InOutSine }
            }
            readonly property real restOpacity: 0.08 + 0.04 * root.breath
            readonly property color restInk: root.light ? "#1F2A44" : Qt.hsla(220 / 360, 0.6, 0.8, 1)
            readonly property color litInk: root.focusHue >= 0 ? Qt.hsla(root.focusHue / 360, root.light ? 0.6 : 0.8, root.light ? 0.35 : 0.7, 1) : Tokens.accent
            // The hero the block shows with nothing suspended: the focused cover's, else the first cover's.
            readonly property var heroCover: root.coverFocused ? root.focused : root.cover1
            Backdrop {
                objectName: "homeSky"
                anchors.fill: parent
                resting: root.resting
                glowHue: root.focused && !root.focused.isPlace ? root.focusHue : -1
                glowX: root.focused ? root.focused.cx / root.width : 0.5
                glowY: root.focused ? root.focused.cy / root.height : 0.4
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
                help: block.activeFocus ? Help.entries(Help.Continue) : (root.coverFocused ? Help.entries(Help.Game) : Help.entries(Help.Place, { accept: root.focused ? root.focused.available !== false : true }))
                Connections {
                    target: frame.controller
                    function onActionInvoked(key) { Actions.act(key) }
                }
                Item {
                    id: chart
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
                            ink: root.light ? "#5F5F5F" : "#E9E4D4"
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
                    // The chart's lines under the nodes: each edge of the walk once, from the node that names it.
                    Scene {
                        id: lines
                        objectName: "homeLines"
                        anchors.fill: parent
                        component Edge: ChartLine {
                            required property var from
                            required property var to
                            readonly property bool lit: root.focused !== null && (root.focused === from || root.focused === to)
                            visible: from !== null && to !== null
                            x1: from ? from.cx : 0
                            y1: from ? from.cy : 0
                            x2: to ? to.cx : 0
                            y2: to ? to.cy : 0
                            ink: lit ? root.litInk : root.restInk
                            opacity: lit ? 0.45 : root.restOpacity
                        }
                        Edge { from: Session.hasGame ? block : null; to: root.cover1 }
                        Edge { from: Session.hasGame ? block : null; to: root.cover3 }
                        Edge { from: root.cover1; to: root.cover2 }
                        Edge { from: root.cover1; to: root.cover3 }
                        Edge { from: root.cover2; to: root.cover4 }
                        Edge { from: root.cover3; to: root.cover4 }
                        Repeater {
                            model: places.count
                            Item {
                                id: placeEdges
                                required property int index
                                readonly property var place: root.placesMade > placeEdges.index ? places.itemAt(placeEdges.index) : null
                                Edge { from: placeEdges.place; to: placeEdges.place ? placeEdges.place.upItem : null }
                                Edge { from: placeEdges.place; to: placeEdges.place ? placeEdges.place.rightItem : null }
                            }
                        }
                    }
                    // The block: the paused game, else the focused cover's hero, else the empty words on glass.
                    Walkable {
                        id: block
                        objectName: "continue"
                        readonly property real cx: x + width / 2
                        readonly property real cy: y + height / 2
                        readonly property real hue: Session.hue
                        readonly property bool isCover: false
                        readonly property bool isPlace: false
                        controller: frame.controller
                        x: 64 + root.ox
                        y: 96
                        width: 740
                        height: 416
                        focus: Session.hasGame
                        activeFocusOnTab: Session.hasGame
                        enabled: Session.hasGame
                        Accessible.name: Words.continuePlaying
                        onActivated: if (Session.hasGame) Actions.play(Session.key)
                        onActiveFocusChanged: root.focused = block.activeFocus ? block : (root.focused === block ? null : root.focused)
                        KeyNavigation.right: root.cover1
                        KeyNavigation.down: root.placesMade > 0 ? places.itemAt(Math.max(0, Math.min(places.count - 1, Math.floor((block.cx - 64) / root.pitch)))) : null
                        Glass {
                            objectName: "emptyBlock"
                            anchors.fill: parent
                            visible: !Session.hasGame && shelf.count === 0
                        }
                        Text {
                            objectName: "emptyWords"
                            visible: !Session.hasGame && shelf.count === 0
                            x: 32
                            y: 208 - 4 - 52.8
                            width: parent.width - 64
                            text: Words.noGames
                            elide: Text.ElideRight
                            color: Tokens.textStrong
                            font.family: faces.condensed
                            font.weight: Font.Medium
                            font.pixelSize: 44
                            lineHeight: 52.8
                            lineHeightMode: Text.FixedHeight
                        }
                        Text {
                            objectName: "emptyFolder"
                            visible: !Session.hasGame && shelf.count === 0
                            x: 32
                            y: 208 + 4
                            width: parent.width - 64
                            text: Words.addFolder
                            elide: Text.ElideRight
                            color: Tokens.muted
                            font.family: faces.regular
                            font.pixelSize: 22
                            lineHeight: 26.4
                            lineHeightMode: Text.FixedHeight
                        }
                        Item {
                            id: art
                            objectName: "blockArt"
                            anchors.fill: parent
                            visible: Session.hasGame || shelf.count > 0
                            Rectangle {
                                anchors.fill: parent
                                radius: 12
                                color: Tokens.inputSurface
                            }
                            RoundPicture {
                                objectName: "blockHero"
                                anchors.fill: parent
                                visible: !Session.hasGame
                                source: root.heroCover ? root.heroCover.hero : ""
                                radius: 12
                                crop: true
                                pixelScale: root.pixelScale
                            }
                            // The game's own picture at the window's resolution, cut to the block's shape at its centre.
                            ShaderEffectSource {
                                id: pausedTexture
                                readonly property Item frame: Session.frame
                                readonly property real cutWidth: pausedTexture.frame ? Math.min(pausedTexture.frame.width, pausedTexture.frame.height * 740 / 416) : 0
                                readonly property real cutHeight: pausedTexture.cutWidth * 416 / 740
                                sourceItem: Session.hasGame && root.visible ? pausedTexture.frame : null
                                hideSource: true
                                visible: false
                                sourceRect: pausedTexture.frame ? Qt.rect((pausedTexture.frame.width - pausedTexture.cutWidth) / 2, (pausedTexture.frame.height - pausedTexture.cutHeight) / 2, pausedTexture.cutWidth, pausedTexture.cutHeight) : Qt.rect(0, 0, 0, 0)
                                textureSize: Qt.size(Math.ceil(740 * root.pixelScale), Math.ceil(416 * root.pixelScale))
                            }
                            Shape {
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
                                        width: 740
                                        height: 416
                                        radius: 12
                                    }
                                }
                            }
                            // The caption bar on the block's foot: CaptionSurface with the glass edge along its top, its lower corners the block's.
                            Shape {
                                objectName: "captionBar"
                                y: 328
                                width: 740
                                height: 88
                                preferredRendererType: Shape.CurveRenderer
                                ShapePath {
                                    strokeColor: "transparent"
                                    fillColor: Tokens.captionSurface
                                    PathRectangle {
                                        width: 740
                                        height: 88
                                        topLeftRadius: 0
                                        topRightRadius: 0
                                        bottomLeftRadius: 12
                                        bottomRightRadius: 12
                                    }
                                }
                            }
                            Rectangle {
                                objectName: "captionEdge"
                                y: 328
                                width: 740
                                height: 1
                                color: "#8C96DC"
                                opacity: 0.18
                            }
                            Text {
                                id: eyebrow
                                objectName: "captionEyebrow"
                                x: 20
                                y: 340 + 1
                                text: Session.hasGame ? Words.continuePlaying : Words.recent
                                color: Tokens.overArt
                                opacity: 0.72
                                font.family: faces.regular
                                font.pixelSize: 19
                                font.letterSpacing: 2
                                font.capitalization: Font.AllUppercase
                                lineHeight: 22.8
                                lineHeightMode: Text.FixedHeight
                            }
                            Chip {
                                objectName: "captionChip"
                                visible: Session.hasGame
                                x: 720 - width
                                y: 338
                                kind: Help.Continue
                                faces: faces
                            }
                            Text {
                                id: captionTitle
                                objectName: "captionTitle"
                                x: 20
                                y: 372
                                width: Math.min(implicitWidth, 400)
                                text: Session.hasGame ? Session.title : (root.heroCover ? root.heroCover.title : "")
                                elide: Text.ElideRight
                                color: Tokens.overArt
                                font.family: faces.condensed
                                font.weight: Font.Medium
                                font.pixelSize: 26
                                lineHeight: 31
                                lineHeightMode: Text.FixedHeight
                            }
                            Text {
                                objectName: "captionFacts"
                                x: captionTitle.x + captionTitle.width + 18
                                y: 372 + 5
                                width: 720 - x
                                text: Session.hasGame ? (Session.pausedSince > 0 && Device.clock.length >= 0 ? Words.pausedFacts(Session.console, Session.pausedSince) : "") : (root.heroCover ? root.heroCover.facts : "")
                                elide: Text.ElideRight
                                color: Tokens.overArt
                                opacity: 0.72
                                font.family: faces.regular
                                font.pixelSize: 20
                                lineHeight: 24
                                lineHeightMode: Text.FixedHeight
                            }
                        }
                        Rectangle {
                            objectName: "continueRing"
                            anchors.fill: parent
                            anchors.margins: -(Tokens.ringOffset + Tokens.ringThickness)
                            visible: block.activeFocus
                            radius: 16
                            color: "transparent"
                            border.color: Tokens.accent
                            border.width: Tokens.ringThickness
                        }
                    }
                    TextMetrics {
                        id: recentRun
                        font: recentLabel.font
                        text: Words.recent
                    }
                    Item {
                        objectName: "recentChartLabel"
                        visible: shelf.count > 0
                        x: 820 + root.ox - 11.4
                        y: 304 - recentLabel.width / 2
                        width: 22.8
                        height: recentLabel.width
                    Text {
                        id: recentLabel
                        objectName: "recentLabel"
                        // The unturned run, one line high, turned about its centre inside a box of the turned footprint.
                        x: (parent.width - width) / 2
                        y: (parent.height - height) / 2
                        width: recentRun.width + 2
                        height: 22.8
                        rotation: -90
                        text: Words.recent
                        color: Tokens.muted
                        font.family: faces.condensed
                        font.weight: Font.Medium
                        font.pixelSize: 19
                        font.letterSpacing: 2
                        font.capitalization: Font.AllUppercase
                        lineHeight: 22.8
                        lineHeightMode: Text.FixedHeight
                    }
                    }
                    Repeater {
                        id: shelf
                        // The four most recent games, the suspended one left out since the block holds it.
                        model: Library.shelf
                        onItemAdded: (index, item) => {
                            root.cover1 = index === 0 ? item : root.cover1
                            root.cover2 = index === 1 ? item : root.cover2
                            root.cover3 = index === 2 ? item : root.cover3
                            root.cover4 = index === 3 ? item : root.cover4
                        }
                        onItemRemoved: (index, item) => {
                            root.cover1 = root.cover1 === item ? null : root.cover1
                            root.cover2 = root.cover2 === item ? null : root.cover2
                            root.cover3 = root.cover3 === item ? null : root.cover3
                            root.cover4 = root.cover4 === item ? null : root.cover4
                        }
                        Walkable {
                            id: cover
                            required property int index
                            required property string key
                            required property string title
                            required property string cover
                            required property real consoleHue
                            required property bool favourite
                            required property string hero
                            required property string facts
                            readonly property int column: cover.index % 2
                            readonly property int row: Math.floor(cover.index / 2)
                            readonly property real cx: x + width / 2
                            readonly property real cy: y + height / 2
                            readonly property real hue: cover.consoleHue
                            readonly property bool isCover: true
                            readonly property bool isPlace: false
                            controller: frame.controller
                            objectName: "cover." + cover.index
                            visible: cover.index < 4
                            enabled: cover.index < 4
                            focus: !Session.hasGame && cover.index === 0
                            Accessible.name: cover.title
                            x: 836 + root.ox + cover.column * 200
                            y: 96 + cover.row * 220
                            width: 180
                            height: 196
                            onActivated: Actions.play(cover.key)
                            onActiveFocusChanged: {
                                root.focused = cover.activeFocus ? cover : (root.focused === cover ? null : root.focused)
                                if (cover.activeFocus) Actions.focus(cover.key)
                            }
                            KeyNavigation.left: cover.column === 1 ? (cover.row === 0 ? root.cover1 : root.cover3) : (Session.hasGame ? block : null)
                            KeyNavigation.right: cover.column === 0 ? (cover.row === 0 ? root.cover2 : root.cover4) : null
                            KeyNavigation.up: cover.row === 1 ? (cover.column === 0 ? root.cover1 : root.cover2) : null
                            KeyNavigation.down: (cover.row === 0 ? (cover.column === 0 ? root.cover3 : root.cover4) : null) ?? (root.placesMade > 0 ? places.itemAt(Math.max(0, Math.min(places.count - 1, Math.floor((cover.cx - 64) / root.pitch)))) : null)
                            CoverTile {
                                objectName: "coverTile." + cover.index
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
                    Text {
                        id: capTitle
                        objectName: "capTitle"
                        x: 836 + root.ox
                        y: 528
                        width: 380
                        opacity: root.coverFocused ? 1 : 0
                        Behavior on opacity { NumberAnimation { duration: 150 } }
                        text: root.coverFocused ? root.focused.title : ""
                        elide: Text.ElideRight
                        color: Tokens.textStrong
                        font.family: faces.condensed
                        font.weight: Font.Medium
                        font.pixelSize: 26
                        lineHeight: 31
                        lineHeightMode: Text.FixedHeight
                    }
                    Text {
                        objectName: "capFacts"
                        x: 836 + root.ox
                        y: 528 + 31 + 4
                        width: 380
                        opacity: capTitle.opacity
                        text: root.coverFocused ? root.focused.facts : ""
                        elide: Text.ElideRight
                        color: Tokens.muted
                        font.family: faces.regular
                        font.pixelSize: 20
                        lineHeight: 24
                        lineHeightMode: Text.FixedHeight
                    }
                    Rectangle {
                        objectName: "placesLine"
                        x: 64
                        y: 616
                        width: root.width - 128
                        height: 1
                        gradient: Gradient {
                            orientation: Gradient.Horizontal
                            GradientStop { position: 0; color: "transparent" }
                            GradientStop { position: 0.08; color: root.restInk }
                            GradientStop { position: 0.92; color: root.restInk }
                            GradientStop { position: 1; color: "transparent" }
                        }
                        opacity: 0.35
                    }
                    Repeater {
                        id: places
                        model: Library.places
                        onItemAdded: (index, item) => { root.place0 = index === 0 ? item : root.place0; root.placesMade = root.placesMade + 1 }
                        onItemRemoved: root.placesMade = root.placesMade - 1
                        Walkable {
                            id: place
                            required property int index
                            required property string key
                            required property string name
                            required property string line
                            required property bool attention
                            required property bool available
                            readonly property real cx: 64 + (place.index + 0.5) * root.pitch
                            readonly property real cy: 616
                            readonly property real hue: -1
                            readonly property bool isCover: false
                            readonly property bool isPlace: true
                            // Up: the block over its centre, else the lower cover of its column, else the upper, else the block.
                            readonly property Item upItem: Session.hasGame && place.cx < 804 + root.ox ? block : ((place.cx < 1036 + root.ox ? (root.cover3 ?? root.cover1) : (root.cover4 ?? root.cover2)) ?? root.cover3 ?? root.cover1 ?? (Session.hasGame ? block : null))
                            readonly property Item rightItem: root.placesMade > place.index + 1 ? places.itemAt(place.index + 1) : null
                            controller: frame.controller
                            objectName: "place." + place.index
                            focus: !Session.hasGame && shelf.count === 0 && place.index === 0
                            Accessible.name: place.name
                            Accessible.description: place.line
                            x: place.cx - root.pitch / 2
                            y: 607
                            width: root.pitch
                            height: 79
                            onActivated: Actions.openPlace(place.key)
                            onActiveFocusChanged: root.focused = place.activeFocus ? place : (root.focused === place ? null : root.focused)
                            KeyNavigation.left: place.index > 0 && root.placesMade > 0 ? places.itemAt(place.index - 1) : null
                            KeyNavigation.right: place.rightItem
                            KeyNavigation.up: place.upItem
                            Star {
                                objectName: "placeStar." + place.index
                                x: (place.width - width) / 2
                                y: 9 - height / 2
                                lit: place.activeFocus
                            }
                            Text {
                                objectName: "placeName." + place.index
                                x: 8
                                y: 29
                                width: place.width - 16
                                horizontalAlignment: Text.AlignHCenter
                                opacity: place.available ? 1 : 0.38
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
                            Row {
                                x: (place.width - width) / 2
                                y: 59
                                spacing: 4
                                opacity: place.available ? 1 : 0.38
                                Glyph {
                                    objectName: "placeAttention." + place.index
                                    visible: place.attention
                                    anchors.verticalCenter: parent.verticalCenter
                                    width: 14
                                    height: 14
                                    path: Marks.attention
                                    filled: true
                                    ink: Tokens.attention
                                }
                                Text {
                                    objectName: "placeLine." + place.index
                                    width: Math.min(implicitWidth, place.width - 16 - (place.attention ? 18 : 0))
                                    text: place.line
                                    elide: Text.ElideRight
                                    color: place.attention ? Tokens.attention : Tokens.muted
                                    font.family: faces.regular
                                    font.pixelSize: 19
                                    lineHeight: 22.8
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
    }
}
