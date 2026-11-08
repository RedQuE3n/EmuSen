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
    readonly property string heading: root.row ? root.row.heading : ""
    readonly property real hue: root.row ? root.row.hue : 0
    readonly property string world: root.row ? root.row.world : ""
    readonly property string countWords: root.row ? root.row.countWords : ""
    // The rows beside it on Choose a console, named by the shoulders.
    property var previousRow
    property var nextRow
    signal closed()
    signal played(string path)

    Kit.Sky {
        objectName: "gridSky"
        anchors.fill: parent
        glowHue: games.count > 0 && cells.currentIndex >= 0 ? games.hueAt(cells.currentIndex) : root.hue
        glowX: 0.5
        glowY: 0.1
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
                spacing: 16
                Text {
                    objectName: "gridPath"
                    y: 7.2
                    text: Words.gamesPath
                    font.letterSpacing: 2
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
                    text: root.heading
                    color: Kit.Tokens.textStrong
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
                    text: root.countWords
                    color: Kit.Tokens.muted
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
                Kit.PadGlyph {
                    objectName: "gridPreviousShoulder"
                    anchors.verticalCenter: parent.verticalCenter
                    width: 30
                    height: 30
                    family: root.padFamily
                    button: Kit.Pad.PageUp
                }
                Text {
                    objectName: "gridPrevious"
                    anchors.verticalCenter: parent.verticalCenter
                    text: root.previousRow ? root.previousRow.name : ""
                    color: Kit.Tokens.muted
                    font.family: faces.regular
                    font.pixelSize: 18
                }
                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: root.previousRow !== null && root.nextRow !== null
                    text: Words.between
                    color: Kit.Tokens.muted
                    font.family: faces.regular
                    font.pixelSize: 18
                }
                Text {
                    objectName: "gridNext"
                    anchors.verticalCenter: parent.verticalCenter
                    text: root.nextRow ? root.nextRow.name : ""
                    color: Kit.Tokens.muted
                    font.family: faces.regular
                    font.pixelSize: 18
                }
                Kit.PadGlyph {
                    objectName: "gridNextShoulder"
                    anchors.verticalCenter: parent.verticalCenter
                    width: 30
                    height: 30
                    family: root.padFamily
                    button: Kit.Pad.PageDown
                }
            }
            GridView {
                id: cells
                objectName: "gridCells"
                // Sixteen units of margin about the gallery, so a lifted cover's ring is drawn whole.
                x: 142 - 16
                y: 108 - 16
                width: 5 * 204 + 32
                height: 2 * 264 - 24 + 32
                leftMargin: 16
                topMargin: 16
                cellWidth: 204
                cellHeight: 264
                clip: true
                focus: true
                keyNavigationWraps: false
                currentIndex: games.count > 0 ? 0 : -1
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
                    required property string cover
                    required property real hue
                    required property bool favourite
                    readonly property bool lit: cell.GridView.isCurrentItem && cells.activeFocus
                    controller: frame.controller
                    objectName: "gridCell." + cell.index
                    Accessible.name: cell.title
                    width: 180
                    height: 240
                    z: cell.lit ? 1 : 0
                    onActivated: root.played(cell.path)
                    onActiveFocusChanged: if (cell.activeFocus) cells.currentIndex = cell.index
                    Cover {
                        objectName: "gridCover." + cell.index
                        anchors.fill: parent
                        source: cell.cover
                        title: cell.title
                        hue: cell.hue
                        favourite: cell.favourite
                        lit: cell.lit
                    }
                }
            }
            Text {
                objectName: "gridFocusTitle"
                x: 64
                y: 628
                width: root.width - 128
                text: games.count > 0 && cells.currentIndex >= 0 ? games.titleAt(cells.currentIndex) : ""
                elide: Text.ElideRight
                color: Kit.Tokens.textStrong
                font.family: faces.condensed
                font.weight: Font.Medium
                font.pixelSize: 32
                lineHeight: 38.4
                lineHeightMode: Text.FixedHeight
            }
            Text {
                objectName: "gridFocusFacts"
                x: 64
                y: 670
                width: root.width - 128
                text: games.count > 0 && cells.currentIndex >= 0 ? games.factsAt(cells.currentIndex) : ""
                elide: Text.ElideRight
                color: Kit.Tokens.muted
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
}
