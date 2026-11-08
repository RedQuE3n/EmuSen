pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models

// A row's games under the night sky, the Grid canvas in canvas units: the heading with the console's planet, the cards in five columns, the focused game's words beneath (EmuSen_Interface.md §24.2; EmuSen_Qt.md §7.1).
FocusScope {
    id: root
    objectName: "grid"
    required property LibraryModel library
    property int padFamily: Kit.Families.Generic
    property string key
    // The row the grid opened, a tile of Choose a console.
    property var row
    readonly property string name: root.row ? root.row.name : ""
    readonly property real hue: root.row ? root.row.hue : 0
    readonly property string world: root.row ? root.row.world : ""
    readonly property string countWords: root.row ? root.row.countWords : ""
    readonly property real cellWidth: (root.width - 128 - 4 * 24) / 5
    signal closed()
    signal played(string path)

    Kit.Sky {
        objectName: "gridSky"
        anchors.fill: parent
        glowHue: games.count > 0 && cells.currentIndex >= 0 ? games.hueAt(cells.currentIndex) : root.hue
        glowX: cells.currentItem ? (cells.x + cells.currentItem.x + root.cellWidth / 2) / Math.max(1, root.width) : 0.5
        glowY: 0.3
    }
    Kit.Frame {
        id: frame
        anchors.fill: parent
        surfaceDrawn: false
        padFamily: root.padFamily
        focus: true
        controller.navigationShown: false
        help: HelpModel {
            screen: HelpModel.Grid
        }
        Connections {
            target: frame.controller
            function onBackDeclined() { root.closed() }
        }
        Item {
            x: -frame.safeX
            y: -frame.safeY
            width: root.width
            height: root.height
            Row {
                x: 64
                y: 40.8
                spacing: 12
                Text {
                    objectName: "gridPath"
                    y: 7.2
                    text: Words.gamesPath
                    color: Kit.Tokens.muted
                    font.family: faces.regular
                    font.pixelSize: 20
                    lineHeight: 24
                    lineHeightMode: Text.FixedHeight
                }
                Kit.Body {
                    objectName: "gridPlanet"
                    y: 2.2
                    width: 34
                    height: 34
                    world: root.world
                    visible: root.world.length > 0
                }
                Text {
                    objectName: "gridTitle"
                    text: root.name
                    color: Kit.Tokens.textStrong
                    font.family: faces.condensed
                    font.pixelSize: 32
                    font.capitalization: Font.AllUppercase
                    lineHeight: 38.4
                    lineHeightMode: Text.FixedHeight
                }
                Text {
                    objectName: "gridCount"
                    y: 7.2
                    text: root.countWords
                    color: Kit.Tokens.muted
                    font.family: faces.regular
                    font.pixelSize: 20
                    lineHeight: 24
                    lineHeightMode: Text.FixedHeight
                }
            }
            GridView {
                id: cells
                objectName: "gridCells"
                x: 64
                y: 116
                width: root.width - 128 + 24
                height: 3 * 174 - 28
                cellWidth: root.cellWidth + 24
                cellHeight: 174
                clip: true
                focus: true
                keyNavigationWraps: false
                highlightFollowsCurrentItem: false
                model: GameModel {
                    id: games
                    games: root.library.loading ? "" : root.library.gamesOf(root.key)
                }
                delegate: Kit.Walkable {
                    id: cell
                    required property int index
                    required property string path
                    required property string title
                    required property real hue
                    readonly property bool lit: cell.GridView.isCurrentItem && cells.activeFocus
                    controller: frame.controller
                    objectName: "gridCell." + cell.index
                    Accessible.name: cell.title
                    width: root.cellWidth
                    height: 146
                    onActivated: root.played(cell.path)
                    onActiveFocusChanged: if (cell.activeFocus) cells.currentIndex = cell.index
                    GameCard {
                        objectName: "gridCard." + cell.index
                        anchors.centerIn: parent
                        anchors.verticalCenterOffset: cell.lit ? -6 : 0
                        width: 208
                        height: 146
                        title: cell.title
                        hue: cell.hue
                    }
                    Rectangle {
                        objectName: "gridRing." + cell.index
                        visible: cell.lit
                        anchors.centerIn: parent
                        anchors.verticalCenterOffset: -6
                        width: 214
                        height: 152
                        radius: Kit.Tokens.rowRadius + 3
                        color: "transparent"
                        border.color: Kit.Tokens.accent
                        border.width: Kit.Tokens.ringThickness
                    }
                }
            }
            Row {
                x: 64
                y: 640
                spacing: 16
                Text {
                    objectName: "gridFocusTitle"
                    text: cells.currentItem ? games.titleAt(cells.currentIndex) : ""
                    color: Kit.Tokens.textStrong
                    font.family: faces.condensed
                    font.pixelSize: 32
                    lineHeight: 38.4
                    lineHeightMode: Text.FixedHeight
                }
                Text {
                    objectName: "gridFocusFacts"
                    y: 10
                    text: root.name
                    color: Kit.Tokens.muted
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
