pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models

// Choose a console under the night sky, the courts canvas in canvas units: the places, then each console with games as its body in rows, the focused one's words and covers beneath (EmuSen_Interface.md §24.1, §24.6; EmuSen_Qt.md §7.1).
FocusScope {
    id: root
    objectName: "consoles"
    required property LibraryModel library
    property int padFamily: Kit.Families.Generic
    readonly property alias tiles: tiles
    readonly property var focusedTile: tiles.count > root.library.selectedIndex ? tiles.itemAt(root.library.selectedIndex) : null
    signal opened(string key)
    signal closed()
    signal acted(string key)

    Kit.PlanetLayout {
        id: layout
        width: root.width
        leading: root.library.places
        bodies: tiles.count - root.library.places
    }
    Kit.Sky {
        id: sky
        objectName: "consolesSky"
        anchors.fill: parent
        glowHue: root.focusedTile && root.focusedTile.planet ? root.focusedTile.hue : -1
        glowX: 0.5
        glowY: 0.77
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
            screen: HelpModel.Consoles
        }
        Connections {
            target: frame.controller
            function onBackDeclined() { root.closed() }
            function onActionInvoked(key) { root.acted(key) }
        }
        Item {
            id: canvas
            x: -frame.safeX
            y: -frame.safeY
            width: root.width
            height: root.height
            Text {
                id: title
                objectName: "consolesTitle"
                x: 64
                y: 40
                text: Words.games
                color: Kit.Tokens.textStrong
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
                color: Kit.Tokens.muted
                font.family: faces.regular
                font.pixelSize: 20
                lineHeight: 24
                lineHeightMode: Text.FixedHeight
            }
            Text {
                objectName: "consolesSummary"
                x: root.width - 64 - width
                y: 52
                text: root.library.summary
                color: Kit.Tokens.muted
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
                id: tiles
                model: root.library
                Kit.Walkable {
                    id: tile
                    required property int index
                    required property string kind
                    required property string key
                    required property string name
                    required property string fullName
                    required property string status
                    required property string facts
                    required property real hue
                    required property string world
                    required property string engineName
                    required property bool libretro
                    required property string countWords
                    required property string engineMark
                    required property string heading
                    readonly property bool planet: tile.index >= root.library.places
                    readonly property int n: tile.index - root.library.places
                    readonly property bool lit: tile.activeFocus
                    controller: frame.controller
                    objectName: "tile." + tile.index
                    Accessible.name: tile.name
                    Accessible.description: tile.status
                    focus: tile.index === Math.max(0, root.library.selectedIndex)
                    x: layout.xs[tile.index] ?? 0
                    y: (layout.ys[tile.index] ?? 0) - (tile.lit && tile.planet ? layout.lift : 0)
                    width: layout.widths[tile.index] ?? 0
                    height: layout.heights[tile.index] ?? 0
                    onActiveFocusChanged: if (tile.activeFocus) root.library.select(tile.index)
                    onActivated: root.opened(tile.key)
                    KeyNavigation.left: tiles.count > tile.index ? tiles.itemAt(layout.left[tile.index] ?? -1) : null
                    KeyNavigation.right: tiles.count > tile.index ? tiles.itemAt(layout.right[tile.index] ?? -1) : null
                    KeyNavigation.up: tiles.count > tile.index ? tiles.itemAt(layout.up[tile.index] ?? -1) : null
                    KeyNavigation.down: tiles.count > tile.index ? tiles.itemAt(layout.down[tile.index] ?? -1) : null
                    Rectangle {
                        objectName: "placeTile." + tile.index
                        anchors.fill: parent
                        visible: !tile.planet
                        radius: Kit.Tokens.tileRadius
                        color: Kit.Tokens.surfaceRaised
                        opacity: 0.55
                    }
                    Rectangle {
                        objectName: "consoleTile." + tile.n
                        anchors.fill: parent
                        visible: tile.planet
                        radius: Kit.Tokens.tileRadius
                        color: tile.lit ? Kit.Tokens.surfaceRaised : "transparent"
                        opacity: 0.88
                    }
                    Rectangle {
                        objectName: "placeRing." + tile.index
                        visible: tile.lit && !tile.planet
                        x: -3
                        y: -3
                        width: parent.width + 6
                        height: parent.height + 6
                        radius: Kit.Tokens.tileRadius + 3
                        color: "transparent"
                        border.color: Kit.Tokens.accent
                        border.width: Kit.Tokens.ringThickness
                    }
                    Rectangle {
                        objectName: "consoleRing." + tile.n
                        visible: tile.lit && tile.planet
                        x: -3
                        y: -3
                        width: parent.width + 6
                        height: parent.height + 6
                        radius: Kit.Tokens.tileRadius + 3
                        color: "transparent"
                        border.color: Kit.Tokens.accent
                        border.width: Kit.Tokens.ringThickness
                    }
                    Rectangle {
                        objectName: "placeIcon." + tile.index
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
                            color: Kit.Tokens.textStrong
                            opacity: 0.06
                        }
                        Glyph {
                            anchors.centerIn: parent
                            width: 22
                            height: 22
                            path: Marks.path(tile.key)
                            ink: tile.lit ? Kit.Tokens.textStrong : Kit.Tokens.muted
                        }
                    }
                    Text {
                        objectName: "placeName." + tile.index
                        visible: !tile.planet
                        x: 76
                        y: 9.8
                        width: parent.width - 96
                        text: tile.name
                        elide: Text.ElideRight
                        color: Kit.Tokens.textStrong
                        font.family: faces.condensed
                        font.weight: Font.Medium
                        font.pixelSize: 24
                        font.capitalization: Font.AllUppercase
                        lineHeight: 28.8
                        lineHeightMode: Text.FixedHeight
                    }
                    Text {
                        objectName: "placeStatus." + tile.index
                        visible: !tile.planet
                        x: 76
                        y: 40.6
                        width: parent.width - 96
                        text: tile.status
                        elide: Text.ElideRight
                        color: Kit.Tokens.muted
                        font.family: faces.regular
                        font.pixelSize: 18
                        lineHeight: 21.6
                        lineHeightMode: Text.FixedHeight
                    }
                    Kit.Body {
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
                            ink: Kit.Tokens.textStrong
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
                        color: Kit.Tokens.textStrong
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
                        color: Kit.Tokens.muted
                        font.family: faces.regular
                        font.pixelSize: 17
                        lineHeight: 20.4
                        lineHeightMode: Text.FixedHeight
                    }
                    Kit.Scene {
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
            }
            Text {
                objectName: "consoleDetailName"
                x: 64
                y: 633.6
                height: 44
                width: root.width - 128 - fan.width
                text: root.focusedTile ? root.focusedTile.fullName : ""
                clip: true
                color: Kit.Tokens.textStrong
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
                width: root.width - 128 - fan.width
                text: root.focusedTile ? root.focusedTile.facts : ""
                elide: Text.ElideRight
                color: Kit.Tokens.muted
                font.family: faces.regular
                font.pixelSize: 22
                lineHeight: 26.4
                lineHeightMode: Text.FixedHeight
            }
            CoverFan {
                id: fan
                objectName: "coverFan"
                x: root.width - 64 - width
                y: 612
                width: 420
                height: 104
                games: GameModel {
                    games: root.library.loading || !root.focusedTile ? "" : root.library.gamesOf(root.focusedTile.key)
                }
            }
        }
    }
    Faces {
        id: faces
    }
}
