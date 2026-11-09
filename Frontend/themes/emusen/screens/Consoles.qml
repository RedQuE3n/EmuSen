pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme
import "../parts"

// Choose a console under the night sky, the courts canvas in canvas units: the shelves, then each console with games as its body in rows, the focused one's words and covers beneath (EmuSen_Interface.md §24.1, §24.6; EmuSen_Qt.md §7.1).
ThemeScreen {
    id: screen
    objectName: "consolesScreen"
    role: ThemeScreen.Consoles
    // One tile of the rows, a shelf or a console, at its place in the layout; its neighbours are the layout's, across both lists.
    component Tile: Walkable {
        id: tile
        required property int index
        required property string key
        required property string name
        required property int count
        // The console's hue; a shelf has none (-1).
        required property real hue
        // The engine a console runs on, its mark and whether it is a libretro core; a shelf has none.
        required property string engineName
        required property string engineMark
        required property bool libretro
        property int tileIndex
        property bool planet
        property string fullName
        property string status
        property string facts
        property string world
        readonly property int n: tile.tileIndex - shelfTiles.count
        readonly property bool lit: tile.activeFocus
        readonly property int leftIndex: layout.left[tile.tileIndex] ?? -1
        readonly property int rightIndex: layout.right[tile.tileIndex] ?? -1
        readonly property int upIndex: layout.up[tile.tileIndex] ?? -1
        readonly property int downIndex: layout.down[tile.tileIndex] ?? -1
        controller: frame.controller
        objectName: "tile." + tile.tileIndex
        Accessible.name: tile.name
        Accessible.description: tile.status
        // The section last opened has the focus, else the first shelf.
        focus: Browse.section.length > 0 ? tile.key === Browse.section : tile.tileIndex === 0
        x: layout.xs[tile.tileIndex] ?? 0
        y: (layout.ys[tile.tileIndex] ?? 0) - (tile.lit && tile.planet ? layout.lift : 0)
        width: layout.widths[tile.tileIndex] ?? 0
        height: layout.heights[tile.tileIndex] ?? 0
        onActiveFocusChanged: {
            if (!tile.activeFocus)
                return
            root.focusedTile = tile
            Actions.focus(tile.key)
        }
        KeyNavigation.left: tile.leftIndex < shelfTiles.count ? shelfTiles.itemAt(tile.leftIndex) : consoleTiles.itemAt(tile.leftIndex - shelfTiles.count)
        KeyNavigation.right: tile.rightIndex < shelfTiles.count ? shelfTiles.itemAt(tile.rightIndex) : consoleTiles.itemAt(tile.rightIndex - shelfTiles.count)
        KeyNavigation.up: tile.upIndex < shelfTiles.count ? shelfTiles.itemAt(tile.upIndex) : consoleTiles.itemAt(tile.upIndex - shelfTiles.count)
        KeyNavigation.down: tile.downIndex < shelfTiles.count ? shelfTiles.itemAt(tile.downIndex) : consoleTiles.itemAt(tile.downIndex - shelfTiles.count)
        Rectangle {
            objectName: "placeTile." + tile.tileIndex
            anchors.fill: parent
            visible: !tile.planet
            radius: Tokens.tileRadius
            color: Tokens.surfaceRaised
            opacity: 0.55
        }
        Rectangle {
            objectName: "consoleTile." + tile.n
            anchors.fill: parent
            visible: tile.planet
            radius: Tokens.tileRadius
            color: tile.lit ? Tokens.surfaceRaised : "transparent"
            opacity: 0.88
        }
        Rectangle {
            objectName: "placeRing." + tile.tileIndex
            visible: tile.lit && !tile.planet
            x: -3
            y: -3
            width: parent.width + 6
            height: parent.height + 6
            radius: Tokens.tileRadius + 3
            color: "transparent"
            border.color: Tokens.accent
            border.width: Tokens.ringThickness
        }
        Rectangle {
            objectName: "consoleRing." + tile.n
            visible: tile.lit && tile.planet
            x: -3
            y: -3
            width: parent.width + 6
            height: parent.height + 6
            radius: Tokens.tileRadius + 3
            color: "transparent"
            border.color: Tokens.accent
            border.width: Tokens.ringThickness
        }
        Rectangle {
            objectName: "placeIcon." + tile.tileIndex
            visible: !tile.planet
            x: 20
            y: 16
            width: 40
            height: 40
            radius: 20
            color: "transparent"
            Rectangle {
                anchors.fill: parent
                radius: 20
                color: Tokens.textStrong
                opacity: 0.06
            }
            Glyph {
                anchors.centerIn: parent
                width: 22
                height: 22
                path: Marks.path(tile.key)
                ink: tile.lit ? Tokens.textStrong : Tokens.muted
            }
        }
        Text {
            objectName: "placeName." + tile.tileIndex
            visible: !tile.planet
            x: 76
            y: 9.8
            width: parent.width - 96
            text: tile.name
            elide: Text.ElideRight
            color: Tokens.textStrong
            font.family: faces.condensed
            font.weight: Font.Medium
            font.pixelSize: 24
            font.capitalization: Font.AllUppercase
            lineHeight: 28.8
            lineHeightMode: Text.FixedHeight
        }
        Text {
            objectName: "placeStatus." + tile.tileIndex
            visible: !tile.planet
            x: 76
            y: 40.6
            width: parent.width - 96
            text: tile.status
            elide: Text.ElideRight
            color: Tokens.muted
            font.family: faces.regular
            font.pixelSize: 18
            lineHeight: 21.6
            lineHeightMode: Text.FixedHeight
        }
        Body {
            id: body
            objectName: "consoleBody." + tile.n
            visible: tile.planet
            world: tile.world
            focused: tile.lit
            width: body.tileWidth
            height: body.tileHeight
            x: (parent.width - width) / 2
            y: 52 - height / 2
        }
        Repeater {
            // Each sparkle's side (-1 left of the body, 1 right), its top as a share of the body's height, and its size.
            model: tile.lit && tile.planet ? [[-1, 0.08, 13], [1, 0.43, 10], [-1, 0.88, 7]] : []
            Glyph {
                required property var modelData
                objectName: "consoleSparkle"
                x: modelData[0] < 0 ? body.x - width - 2 : body.x + body.width + 2
                y: body.y + modelData[1] * body.height
                width: modelData[2]
                height: modelData[2]
                path: Marks.sparkle
                filled: true
                ink: Tokens.textStrong
            }
        }
        Text {
            objectName: "consoleName." + tile.n
            visible: tile.planet
            y: 105
            width: parent.width - 16
            x: 8
            horizontalAlignment: Text.AlignHCenter
            text: tile.name
            elide: Text.ElideRight
            color: Tokens.textStrong
            font.family: faces.condensed
            font.weight: Font.Medium
            font.pixelSize: 22
            font.capitalization: Font.AllUppercase
            lineHeight: 26.4
            lineHeightMode: Text.FixedHeight
        }
        Text {
            objectName: "consoleCount." + tile.n
            visible: tile.planet
            y: 132.4
            width: parent.width
            horizontalAlignment: Text.AlignHCenter
            text: tile.status
            color: Tokens.muted
            font.family: faces.regular
            font.pixelSize: 17
            lineHeight: 20.4
            lineHeightMode: Text.FixedHeight
        }
        Scene {
            objectName: "consoleEngine." + tile.n
            visible: tile.planet && tile.engineName.length > 0
            width: engineRow.width
            height: engineRow.height
            anchors.horizontalCenter: parent.horizontalCenter
            y: 158
            Row {
                id: engineRow
                spacing: 6
                readonly property color ink: tile.libretro ? "#C9C9D0" : Qt.hsla(tile.hue / 360, tile.lit ? 0.85 : 0.35, tile.lit ? 0.8 : 0.7, 1)
                EngineMark {
                    objectName: "consoleEngineMark." + tile.n
                    anchors.verticalCenter: parent.verticalCenter
                    width: 15
                    height: 15
                    mark: tile.engineMark
                    libretro: tile.libretro
                    ink: engineRow.ink
                }
                Text {
                    objectName: "consoleEngineName." + tile.n
                    anchors.verticalCenter: parent.verticalCenter
                    text: tile.engineName
                    elide: Text.ElideRight
                    width: Math.min(implicitWidth, tile.width - tag.width - 37)
                    color: engineRow.ink
                    font.family: faces.condensed
                    font.weight: Font.Medium
                    font.pixelSize: 17
                    font.letterSpacing: 1.5
                    font.capitalization: Font.AllUppercase
                }
                Rectangle {
                    id: tag
                    objectName: "consoleEngineTag." + tile.n
                    anchors.verticalCenter: parent.verticalCenter
                    width: tagWord.implicitWidth + 10
                    height: 22
                    radius: 4
                    color: tile.libretro ? "transparent" : Qt.hsla(tile.hue / 360, 0.6, 0.5, 0.14)
                    border.color: tile.libretro ? "#6E6E78" : Qt.hsla(tile.hue / 360, 0.45, 0.52, 1)
                    Text {
                        id: tagWord
                        anchors.centerIn: parent
                        text: tile.libretro ? Words.libretroTag : Words.emusenTag
                        color: tile.libretro ? "#A9A9B2" : Qt.hsla(tile.hue / 360, 0.7, 0.82, 1)
                        font.family: faces.condensed
                        font.weight: Font.Medium
                        font.pixelSize: 17
                        font.letterSpacing: 0.8
                    }
                }
            }
        }
    }
    NormalizedCanvas {
        id: canvas
        objectName: "consolesCanvas"
        FocusScope {
            id: root
            objectName: "consoles"
            anchors.fill: parent
            focus: true
            // Canvas units to pixels, for the covers' decoded size.
            readonly property real pixelScale: canvas.unit
            // The focused tile, a shelf or a console, and the first shelf (All games), whose count is every game.
            property var focusedTile: null
            property var allGames: null
            PlanetLayout {
                id: layout
                width: root.width
                leading: Library.shelves.count
                bodies: Library.consoles.count
            }
            Sky {
                id: sky
                objectName: "consolesSky"
                anchors.fill: parent
                glowHue: root.focusedTile && root.focusedTile.planet ? root.focusedTile.hue : -1
                glowX: 0.5
                glowY: 0.77
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
                help: Help.entries(Help.Console)
                Connections {
                    target: frame.controller
                    function onBackDeclined() { Actions.back() }
                    function onActionInvoked(key) { Actions.act(key) }
                }
                Item {
                    x: -frame.safeX
                    y: -frame.safeY
                    width: root.width
                    height: root.height
                    Text {
                        id: title
                        objectName: "consolesTitle"
                        x: 64
                        y: 40
                        text: Words.gamesHeading
                        color: Tokens.textStrong
                        font.family: faces.condensed
                        font.weight: Font.Medium
                        font.pixelSize: 32
                        lineHeight: 38.4
                        lineHeightMode: Text.FixedHeight
                    }
                    Text {
                        objectName: "consolesSubtitle"
                        x: title.x + title.contentWidth + 20
                        y: 52
                        text: Words.chooseAConsole
                        color: Tokens.muted
                        font.family: faces.regular
                        font.pixelSize: 20
                        lineHeight: 24
                        lineHeightMode: Text.FixedHeight
                    }
                    Text {
                        objectName: "consolesSummary"
                        x: root.width - 64 - width
                        y: 52
                        text: Words.summary(Library.consoles.count, root.allGames ? root.allGames.count : 0)
                        color: Tokens.muted
                        font.family: faces.regular
                        font.pixelSize: 20
                        lineHeight: 24
                        lineHeightMode: Text.FixedHeight
                    }
                    Repeater {
                        model: layout.orbits
                        Rectangle {
                            id: orbit
                            required property int index
                            required property real modelData
                            objectName: "orbit." + orbit.index
                            x: 0
                            y: orbit.modelData
                            width: root.width
                            height: 1
                            color: "#8C96DC"
                            opacity: 0.25
                        }
                    }
                    Repeater {
                        id: shelfTiles
                        model: Library.shelves
                        onItemAdded: (index, item) => root.allGames = index === 0 ? item : root.allGames
                        Tile {
                            id: shelf
                            required property string line
                            hue: -1
                            engineName: ""
                            engineMark: ""
                            libretro: false
                            tileIndex: shelf.index
                            planet: false
                            fullName: shelf.name
                            status: shelf.line
                            facts: Words.games(shelf.count)
                            onActivated: Actions.openPlace(shelf.key)
                        }
                    }
                    Repeater {
                        id: consoleTiles
                        model: Library.consoles
                        Tile {
                            id: consoleTile
                            required property string displayName
                            required property string maker
                            required property int year
                            required property string body
                            required property string engineKind
                            tileIndex: consoleTile.index + shelfTiles.count
                            planet: true
                            fullName: consoleTile.displayName
                            status: Words.games(consoleTile.count)
                            facts: Words.facts(consoleTile.maker, consoleTile.year, consoleTile.count, Words.runsOn(consoleTile.engineName, consoleTile.engineKind))
                            world: consoleTile.body
                            onActivated: Actions.openConsole(consoleTile.key)
                        }
                    }
                    Text {
                        objectName: "consoleDetailName"
                        x: 64
                        y: 633.6
                        height: 44
                        width: root.width - 128 - coverFan.width
                        text: root.focusedTile ? root.focusedTile.fullName : ""
                        clip: true
                        color: Tokens.textStrong
                        font.family: faces.condensed
                        font.weight: Font.Medium
                        font.pixelSize: 44
                        font.capitalization: Font.AllUppercase
                        lineHeight: 44
                        lineHeightMode: Text.FixedHeight
                    }
                    Text {
                        objectName: "consoleDetailFacts"
                        x: 64
                        y: 683.6
                        width: root.width - 128 - coverFan.width
                        text: root.focusedTile ? root.focusedTile.facts : ""
                        elide: Text.ElideRight
                        color: Tokens.muted
                        font.family: faces.regular
                        font.pixelSize: 22
                        lineHeight: 26.4
                        lineHeightMode: Text.FixedHeight
                    }
                    CoverFan {
                        id: coverFan
                        objectName: "coverFan"
                        pixelScale: root.pixelScale
                        x: root.width - 64 - width
                        y: 612
                        width: 420
                        height: 104
                        games: Library.firstOf(root.focusedTile ? root.focusedTile.key : "", 4)
                    }
                }
            }
            Faces {
                id: faces
            }
        }
    }
}
