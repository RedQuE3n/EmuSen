pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme
import "../parts"

// A section's games under the night sky, the Grid canvas in canvas units: the heading with the console's planet, the covers in five columns, the focused game's words beneath; the search's field and keyboard and the List card are EmuSen's, drawn over it (EmuSen_Interface.md §24.2; EmuSen_Qt.md §7.1).
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
            Sky {
                objectName: "gridSky"
                anchors.fill: parent
                glowHue: root.focusedCell ? root.focusedCell.consoleHue : Browse.hue
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
                            color: night.muted
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
                            color: night.textStrong
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
                            color: night.muted
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
                            anchors.verticalCenter: parent.verticalCenter
                            text: Browse.previous
                            color: night.muted
                            font.family: faces.regular
                            font.pixelSize: 19
                        }
                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            visible: Browse.previous.length > 0 && Browse.next.length > 0
                            text: Words.between
                            color: night.muted
                            font.family: faces.regular
                            font.pixelSize: 19
                        }
                        Text {
                            objectName: "gridNext"
                            anchors.verticalCenter: parent.verticalCenter
                            text: Browse.next
                            color: night.muted
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
                        x: 142 - 16
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
                            controller: frame.controller
                            objectName: "gridCell." + cell.index
                            Accessible.name: cell.title
                            width: 180
                            height: 240
                            z: cell.lit ? 1 : 0
                            onActivated: Actions.play(cell.key)
                            onActiveFocusChanged: if (cell.activeFocus) Actions.focus(cell.key)
                            CoverTile {
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
                    Text {
                        objectName: "gridFocusTitle"
                        // Under the docked keyboard while a search is typed, as Mistress's keyboard covers them.
                        visible: !Browse.searching
                        x: 64
                        y: 628
                        width: root.width - 128
                        text: root.focusedCell ? root.focusedCell.title : ""
                        elide: Text.ElideRight
                        color: night.textStrong
                        font.family: faces.condensed
                        font.weight: Font.Medium
                        font.pixelSize: 32
                        lineHeight: 38.4
                        lineHeightMode: Text.FixedHeight
                    }
                    Text {
                        objectName: "gridFocusFacts"
                        // Under the docked keyboard while a search is typed, as Mistress's keyboard covers them.
                        visible: !Browse.searching
                        x: 64
                        y: 670
                        width: root.width - 128
                        text: root.focusedCell ? root.focusedCell.facts : ""
                        elide: Text.ElideRight
                        color: night.muted
                        font.family: faces.regular
                        font.pixelSize: 22
                        lineHeight: 26.4
                        lineHeightMode: Text.FixedHeight
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
