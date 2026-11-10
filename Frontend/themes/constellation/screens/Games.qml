pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme
import "../parts"

// A section's games as the reference grid with the chart's connectors from the focused cover to its neighbours, and the focused game's title and facts in a key of glass; the search's field and keyboard and the List card are EmuSen's, drawn over it (constellation.md §4).
ThemeScreen {
    id: screen
    objectName: "gridScreen"
    role: ThemeScreen.Games
    // A game of the section, as its row gives it.
    component Cell: Walkable {
        required property int index
        required property string key
        required property string title
        required property string cover
        required property real consoleHue
        required property bool favourite
        required property string facts
        readonly property real hue: consoleHue
        property Item tile: null
    }
    NormalizedCanvas {
        id: canvas
        objectName: "gridCanvas"
        FocusScope {
            id: root
            objectName: "grid"
            anchors.fill: parent
            focus: true
            // Canvas units to pixels, for the covers' decoded size.
            readonly property real pixelScale: canvas.unit
            readonly property var focusedCell: cells.currentItem
            readonly property bool light: Tokens.variant === Tokens.Light
            readonly property bool resting: !Tokens.reducedMotion && !(Device.battery >= 0 && !Device.charging)
            // The grid's left edge, centred: 142 at 1280, 213 at 1422.
            readonly property real x0: (root.width - 996) / 2
            readonly property int rows: Math.ceil(cells.count / 5)
            readonly property real focusHue: root.focusedCell ? root.focusedCell.consoleHue : Browse.hue
            readonly property color connectorInk: Qt.hsla(Math.max(0, root.focusHue) / 360, root.light ? 0.6 : 0.8, root.light ? 0.35 : 0.7, 1)
            Backdrop {
                objectName: "gridSky"
                anchors.fill: parent
                resting: root.resting
                glowHue: root.focusHue
                glowX: 0.5
                glowY: 0.1
            }
            Frame {
                id: frame
                anchors.fill: parent
                surfaceDrawn: false
                padFamily: Device.family
                helpFont: Qt.font({ family: faces.regular, pixelSize: 20 })
                helpSpacing: 40
                helpGlyphGap: 10
                // While the search's keyboard is typed on, its help is EmuSen's, drawn with it.
                helpShown: !Browse.typing
                focus: true
                controller.navigationShown: false
                help: Help.entries(Browse.searched ? Help.SectionSearched : Help.Section)
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
                    Row {
                        x: 64
                        y: 40.8
                        spacing: 16
                        Text {
                            objectName: "gridPath"
                            y: 7.2
                            text: Words.gamesPath
                            font.letterSpacing: 2
                            color: Tokens.muted
                            font.family: faces.regular
                            font.pixelSize: 20
                            lineHeight: 24
                            lineHeightMode: Text.FixedHeight
                        }
                        Body {
                            objectName: "gridPlanet"
                            y: 2.2
                            width: 34
                            height: 34
                            world: Browse.world
                            visible: Browse.world.length > 0
                        }
                        Text {
                            objectName: "gridTitle"
                            text: Browse.heading
                            color: Tokens.textStrong
                            font.family: faces.condensed
                            font.weight: Font.Medium
                            font.pixelSize: 32
                            font.letterSpacing: 3
                            font.capitalization: Font.AllUppercase
                            lineHeight: 38.4
                            lineHeightMode: Text.FixedHeight
                        }
                        Text {
                            objectName: "gridCount"
                            y: 7.2
                            text: Browse.countLine
                            color: Tokens.muted
                            font.family: faces.regular
                            font.pixelSize: 20
                            lineHeight: 24
                            lineHeightMode: Text.FixedHeight
                        }
                    }
                    Row {
                        objectName: "gridNeighbours"
                        x: root.width - 64 - width
                        y: 34
                        height: 52
                        spacing: 12
                        PadGlyph {
                            objectName: "gridPreviousShoulder"
                            anchors.verticalCenter: parent.verticalCenter
                            width: 30
                            height: 30
                            family: Device.family
                            button: PadGlyph.PageUp
                        }
                        Text {
                            objectName: "gridPrevious"
                            visible: text.length > 0
                            anchors.verticalCenter: parent.verticalCenter
                            text: Browse.previous
                            color: Tokens.muted
                            font.family: faces.regular
                            font.pixelSize: 19
                        }
                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            visible: Browse.previous.length > 0 && Browse.next.length > 0
                            text: Words.between
                            color: Tokens.muted
                            font.family: faces.regular
                            font.pixelSize: 19
                        }
                        Text {
                            objectName: "gridNext"
                            visible: text.length > 0
                            anchors.verticalCenter: parent.verticalCenter
                            text: Browse.next
                            color: Tokens.muted
                            font.family: faces.regular
                            font.pixelSize: 19
                        }
                        PadGlyph {
                            objectName: "gridNextShoulder"
                            anchors.verticalCenter: parent.verticalCenter
                            width: 30
                            height: 30
                            family: Device.family
                            button: PadGlyph.PageDown
                        }
                    }
                    GridView {
                        id: cells
                        objectName: "gridCells"
                        // Sixteen units of margin about the gallery, so a lifted cover's ring is drawn whole.
                        x: root.x0 - 16
                        y: (Browse.searching ? 156 : 108) - 16
                        width: 5 * 204 + 32
                        height: (Browse.searching ? 1 : 2) * 264 - 24 + 32
                        leftMargin: 16
                        topMargin: 16
                        cellWidth: 204
                        cellHeight: 264
                        clip: true
                        focus: true
                        keyNavigationWraps: false
                        highlightFollowsCurrentItem: false
                        // The two rows shown move only when the focus leaves them, a whole row at a time and at once, as Mistress's gallery moves.
                        property int firstRow: 0
                        readonly property int focusRow: Math.floor(Math.max(0, cells.currentIndex) / 5)
                        readonly property real firstRowY: cells.originY - cells.topMargin + cells.cellHeight * cells.firstRow
                        onFocusRowChanged: cells.firstRow = Math.max(0, Math.min(cells.firstRow, cells.focusRow), cells.focusRow - 1)
                        onFirstRowYChanged: cells.contentY = cells.firstRowY
                        model: Browse.games
                        delegate: Cell {
                            id: cell
                            readonly property bool lit: cell.GridView.isCurrentItem && cells.activeFocus
                            // The painted cover's box in the cell at rest, and the focused one's as lifted and scaled about its centre.
                            readonly property real px: coverTile.paintedX
                            readonly property real py: coverTile.paintedY
                            readonly property real pw: coverTile.paintedWidth
                            readonly property real ph: coverTile.paintedHeight
                            readonly property real litX0: cell.px - 0.03 * cell.pw
                            readonly property real litX1: cell.px + 1.03 * cell.pw
                            readonly property real litY0: cell.py - 6 - 0.03 * cell.ph
                            readonly property real litY1: cell.py + cell.ph - 6 + 0.03 * cell.ph
                            readonly property bool connected: cell.lit && !Browse.searching && cells.count > 1
                            readonly property var rightCell: cell.lit && cell.index % 5 < 4 ? cells.itemAtIndex(cell.index + 1) : null
                            readonly property var leftCell: cell.lit && cell.index % 5 > 0 ? cells.itemAtIndex(cell.index - 1) : null
                            readonly property var downCell: cell.lit ? cells.itemAtIndex(cell.index + 5) : null
                            readonly property var upCell: cell.lit && cell.index >= 5 ? cells.itemAtIndex(cell.index - 5) : null
                            tile: coverTile
                            controller: frame.controller
                            objectName: "gridCell." + cell.index
                            Accessible.name: cell.title
                            width: 180
                            height: 240
                            z: cell.lit ? 1 : 0
                            onActivated: Actions.play(cell.key)
                            onActiveFocusChanged: if (cell.activeFocus) Actions.focus(cell.key)
                            Connector {
                                objectName: "connectorRight"
                                visible: cell.connected && cell.rightCell !== null
                                x1: cell.litX1 + 7
                                x2: 204 + (cell.rightCell ? cell.rightCell.px : 0) - 4
                                y1: (Math.max(cell.litY0, cell.rightCell ? cell.rightCell.py : 0) + Math.min(cell.litY1, cell.rightCell ? cell.rightCell.py + cell.rightCell.ph : 0)) / 2
                                y2: y1
                                ink: root.connectorInk
                            }
                            Connector {
                                objectName: "connectorLeft"
                                visible: cell.connected && cell.leftCell !== null
                                x1: cell.litX0 - 7
                                x2: -204 + (cell.leftCell ? cell.leftCell.px + cell.leftCell.pw : 0) + 4
                                y1: (Math.max(cell.litY0, cell.leftCell ? cell.leftCell.py : 0) + Math.min(cell.litY1, cell.leftCell ? cell.leftCell.py + cell.leftCell.ph : 0)) / 2
                                y2: y1
                                ink: root.connectorInk
                            }
                            Connector {
                                objectName: "connectorDown"
                                visible: cell.connected && cell.downCell !== null
                                x1: cell.px + cell.pw / 2
                                x2: x1
                                y1: cell.litY1 + 7
                                y2: 264 + (cell.downCell ? cell.downCell.py : 0) - 4
                                ink: root.connectorInk
                            }
                            Connector {
                                objectName: "connectorUp"
                                visible: cell.connected && cell.upCell !== null
                                x1: cell.px + cell.pw / 2
                                x2: x1
                                y1: cell.litY0 - 7
                                y2: -264 + (cell.upCell ? cell.upCell.py + cell.upCell.ph : 0) + 4
                                ink: root.connectorInk
                            }
                            CoverTile {
                                id: coverTile
                                objectName: "gridCover." + cell.index
                                anchors.fill: parent
                                source: cell.cover
                                title: cell.title
                                hue: cell.consoleHue
                                favourite: cell.favourite
                                mark: Marks.heart
                                lit: cell.lit
                                lifts: !Browse.searching
                                pixelScale: root.pixelScale
                            }
                        }
                    }
                    // The focus as the program holds it: the theme reports each move (Actions.focus) and the List card moves it too.
                    Binding {
                        cells.currentIndex: Browse.focusRow
                    }
                    Rectangle {
                        objectName: "gridTrack"
                        visible: root.rows > 2 && !Browse.searching
                        x: root.x0 + 1012
                        y: 108
                        width: 2
                        height: 504
                        radius: 1
                        color: Tokens.rule
                        Rectangle {
                            objectName: "gridThumb"
                            width: 2
                            radius: 1
                            height: Math.max(24, 504 * 2 / Math.max(1, root.rows))
                            y: Math.min(504 - height, 504 * cells.firstRow / Math.max(1, root.rows))
                            color: Tokens.muted
                        }
                    }
                    Text {
                        objectName: "gridEmpty"
                        visible: cells.count === 0 && !Browse.searching
                        x: 64
                        y: 108
                        width: root.width - 128
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
                        objectName: "gridEmptyFolder"
                        visible: cells.count === 0 && !Browse.searching
                        x: 64
                        y: 108 + 52.8 + 8
                        width: root.width - 128
                        text: Words.addFolder
                        elide: Text.ElideRight
                        color: Tokens.muted
                        font.family: faces.regular
                        font.pixelSize: 22
                        lineHeight: 26.4
                        lineHeightMode: Text.FixedHeight
                    }
                    Glass {
                        objectName: "gridKey"
                        // Not drawn while a search is typed, as the docked keyboard covers it.
                        visible: !Browse.searching && cells.count > 0
                        x: 64
                        y: 626
                        width: root.width - 128
                        height: 100
                        Text {
                            objectName: "gridFocusTitle"
                            x: 24
                            y: 16
                            width: parent.width - 48
                            text: root.focusedCell ? root.focusedCell.title : ""
                            elide: Text.ElideRight
                            color: Tokens.textStrong
                            font.family: faces.condensed
                            font.weight: Font.Medium
                            font.pixelSize: 32
                            lineHeight: 38.4
                            lineHeightMode: Text.FixedHeight
                        }
                        Text {
                            objectName: "gridFocusFacts"
                            x: 24
                            y: 58
                            width: parent.width - 48
                            text: root.focusedCell ? root.focusedCell.facts : ""
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
            Faces {
                id: faces
            }
        }
    }
}
