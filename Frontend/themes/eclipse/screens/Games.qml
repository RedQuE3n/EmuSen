pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme
import "../parts"

// Eclipse's games view: the focused game's words at the left, the section's games as tiles beside them, and with no game in the library the board's first-run words over the waiting disc; each element and its board line in qt-app/tests/eclipse/games.tsv (EmuSen_Qt.md §3.4.7.2).
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
        required property string consoleId
        required property string description
    }
    NormalizedCanvas {
        id: canvas
        objectName: "gridCanvas"
        FocusScope {
            id: root
            objectName: "grid"
            anchors.fill: parent
            focus: true
            // A board pixel in canvas units: the boards are drawn 1200 high, a constant of the design.
            readonly property real px: canvas.designHeight / 1200
            // A type size is held whole (QFont::setPixelSize takes an int), so each is the nearest whole unit.
            readonly property var focusedCell: cells.currentItem
            // No console has a game, so the library has none: the first-run state.
            readonly property bool noLibrary: Library.consoles.count === 0 && cells.count === 0 && !Browse.searched
            // The tile's box, the gaps between tiles, the margin right of them and the rows shown: the board's, constants of the design.
            readonly property real tileWidth: 206 * root.px
            readonly property real tileHeight: 170 * root.px
            readonly property real tileGap: 20 * root.px
            readonly property int rowsShown: 4
            // The columns that fit between the board's left edge of the tiles and its right margin.
            readonly property int columns: Math.max(1, Math.floor((root.width - (660 + 150) * root.px + root.tileGap) / (root.tileWidth + root.tileGap)))
            // How far a tile's ring and glow reach past it, so the list's clip leaves them whole.
            readonly property real reach: Tokens.ringOffset + Tokens.ringThickness + Tokens.glowSpread + Tokens.glowBlur
            // The filter in force's place among the choices, for the walk up from the tiles, and the shown games' last letter.
            property int filterAt: 0
            property string lastLetter: ""
            // The focused game's console's name, set by its row of Library.consoles.
            property string consoleText: ""
            Repeater {
                model: Library.consoles
                delegate: Item {
                    id: consoleRow
                    required property string key
                    required property string displayName
                    Binding {
                        when: root.focusedCell !== null && root.focusedCell.consoleId === consoleRow.key
                        target: root
                        property: "consoleText"
                        value: consoleRow.displayName
                    }
                }
            }
            Rectangle {
                objectName: "gridGround"
                anchors.fill: parent
                // The board's ground is Home's (Main:17), with its commit; the empty state's is its own.
                color: root.noLibrary ? ink.emptyGround : Tokens.surface
            }
            CoronaDisc {
                objectName: "emptyDisc"
                visible: root.noLibrary
                x: root.width - (190 + 600) * root.px
                y: 250 * root.px
                width: 600 * root.px
                height: 600 * root.px
            }
            Frame {
                id: frame
                anchors.fill: parent
                surfaceDrawn: false
                padFamily: Device.family
                helpShown: !Browse.typing && !root.noLibrary
                focus: true
                controller.navigationShown: false
                help: Help.entries(Browse.searched ? Help.SectionSearched : Help.Section, { letters: true })
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
                    Column {
                        objectName: "gamesAside"
                        visible: !root.noLibrary && root.focusedCell !== null
                        x: 96 * root.px
                        y: 230 * root.px
                        width: 440 * root.px
                        spacing: 22 * root.px
                        Text {
                            objectName: "gamesConsole"
                            width: parent.width
                            text: root.consoleText
                            wrapMode: Text.Wrap
                            color: ink.rose
                            font.family: medium.font.family
                            font.pixelSize: Math.round(18 * root.px)
                            font.capitalization: Font.AllUppercase
                            font.letterSpacing: 18 * 0.2 * root.px
                        }
                        Text {
                            objectName: "gamesTitle"
                            width: parent.width
                            text: root.focusedCell ? root.focusedCell.title : ""
                            wrapMode: Text.Wrap
                            maximumLineCount: 3
                            elide: Text.ElideRight
                            color: Tokens.text
                            font.family: medium.font.family
                            font.pixelSize: Math.round(62 * root.px)
                            font.letterSpacing: -62 * 0.02 * root.px
                            lineHeight: 1.04
                        }
                        Text {
                            objectName: "gamesDescription"
                            visible: text.length > 0
                            width: parent.width
                            text: root.focusedCell ? root.focusedCell.description : ""
                            wrapMode: Text.Wrap
                            maximumLineCount: 6
                            elide: Text.ElideRight
                            color: ink.words
                            font.family: regular.font.family
                            font.pixelSize: Math.round(20 * root.px)
                            lineHeight: 1.5
                        }
                    }
                    Row {
                        objectName: "gamesFilters"
                        visible: !root.noLibrary
                        x: 660 * root.px
                        y: 150 * root.px
                        spacing: 14 * root.px
                        Repeater {
                            id: filterChips
                            model: Browse.filters
                            delegate: Walkable {
                                id: chip
                                required property int index
                                required property string key
                                required property string name
                                required property int count
                                required property string line
                                readonly property bool chosen: chip.key === Browse.filter
                                controller: frame.controller
                                objectName: "gamesFilter." + chip.index
                                Accessible.name: chipWords.text
                                Binding {
                                    when: chip.chosen
                                    target: root
                                    property: "filterAt"
                                    value: chip.index
                                }
                                width: chipWords.implicitWidth + 2 * 22 * root.px
                                height: 48 * root.px
                                onActivated: Actions.setFilter(chip.key)
                                Keys.onLeftPressed: if (chip.index > 0) filterChips.itemAt(chip.index - 1).forceActiveFocus()
                                Keys.onRightPressed: chip.index + 1 < filterChips.count ? filterChips.itemAt(chip.index + 1).forceActiveFocus() : sortPill.forceActiveFocus()
                                Keys.onDownPressed: cells.forceActiveFocus()
                                Rectangle {
                                    objectName: "gamesFilterGround." + chip.index
                                    anchors.fill: parent
                                    radius: height / 2
                                    color: chip.chosen ? Tokens.accent : ink.chipGround
                                    border.width: chip.chosen ? 0 : 2 * root.px
                                    border.color: ink.chipLine
                                }
                                Text {
                                    id: chipWords
                                    objectName: "gamesFilterWords." + chip.index
                                    anchors.centerIn: parent
                                    text: chip.line
                                    color: chip.chosen ? Tokens.overAccent : ink.chipWords
                                    font.family: chip.chosen ? medium.font.family : regular.font.family
                                    font.pixelSize: Math.round(21 * root.px)
                                }
                                FocusRing {
                                    visible: chip.activeFocus
                                    anchors.fill: parent
                                    radius: height / 2
                                }
                            }
                        }
                    }
                    Walkable {
                        id: sortPill
                        objectName: "gamesSort"
                        visible: !root.noLibrary && Browse.sort.length > 0
                        controller: frame.controller
                        Accessible.name: sortWords.text
                        x: root.width - 96 * root.px - width
                        y: 150 * root.px
                        width: sortWords.implicitWidth + 2 * 18 * root.px
                        height: 48 * root.px
                        onActivated: Actions.stepSort(1)
                        Keys.onLeftPressed: if (filterChips.count > 0) filterChips.itemAt(filterChips.count - 1).forceActiveFocus()
                        Keys.onDownPressed: cells.forceActiveFocus()
                        Rectangle {
                            anchors.fill: parent
                            radius: height / 2
                            color: ink.pillGround
                        }
                        Text {
                            id: sortWords
                            objectName: "gamesSortWords"
                            anchors.centerIn: parent
                            text: Browse.sort
                            color: ink.pillWords
                            font.family: regular.font.family
                            font.pixelSize: Math.round(21 * root.px)
                        }
                        FocusRing {
                            visible: sortPill.activeFocus
                            anchors.fill: parent
                            radius: height / 2
                        }
                    }
                    Column {
                        objectName: "gamesRail"
                        visible: !root.noLibrary && cells.count > 0
                        x: root.width - (44 + 34) * root.px
                        y: 236 * root.px
                        width: 34 * root.px
                        height: 740 * root.px
                        spacing: 10 * root.px
                        Repeater {
                            id: letters
                            model: Browse.letters
                            delegate: Item {
                                id: entry
                                required property int index
                                required property string letter
                                Binding {
                                    when: entry.index === letters.count - 1
                                    target: root
                                    property: "lastLetter"
                                    value: entry.letter
                                }
                            }
                        }
                        Rectangle {
                            objectName: "gamesRailLetter"
                            width: parent.width
                            height: parent.width
                            radius: width / 2
                            color: Tokens.accent
                            Text {
                                objectName: "gamesRailLetterWords"
                                anchors.centerIn: parent
                                text: Browse.letter
                                color: Tokens.overAccent
                                font.family: medium.font.family
                                font.pixelSize: Math.round(17 * root.px)
                            }
                        }
                        Rectangle {
                            objectName: "gamesRailTrack"
                            anchors.horizontalCenter: parent.horizontalCenter
                            width: 4 * root.px
                            height: parent.height - parent.width - lastLetter.height - 2 * parent.spacing
                            radius: width / 2
                            color: ink.railTrack
                        }
                        Text {
                            id: lastLetter
                            objectName: "gamesRailLast"
                            anchors.horizontalCenter: parent.horizontalCenter
                            text: root.lastLetter
                            color: ink.pillWords
                            font.family: regular.font.family
                            font.pixelSize: Math.round(16 * root.px)
                        }
                    }
                    GridView {
                        id: cells
                        objectName: "gamesGrid"
                        visible: !root.noLibrary
                        x: 660 * root.px - root.reach
                        y: 236 * root.px - root.reach
                        width: root.columns * cells.cellWidth - root.tileGap + 2 * root.reach
                        height: root.rowsShown * cells.cellHeight - root.tileGap + 2 * root.reach
                        leftMargin: root.reach
                        topMargin: root.reach
                        cellWidth: root.tileWidth + root.tileGap
                        cellHeight: root.tileHeight + root.tileGap
                        clip: true
                        focus: true
                        keyNavigationWraps: false
                        highlightFollowsCurrentItem: false
                        // Up from the first row reaches the filter in force.
                        Keys.onUpPressed: (event) => {
                            const chip = filterChips.itemAt(Math.max(0, root.filterAt));
                            if (cells.currentIndex < root.columns && chip)
                                chip.forceActiveFocus();
                            else
                                event.accepted = false;
                        }
                        // The rows shown move only when the focus leaves them, a whole row at a time.
                        property int firstRow: 0
                        readonly property int focusRow: Math.floor(Math.max(0, cells.currentIndex) / root.columns)
                        readonly property real firstRowY: cells.originY - cells.topMargin + cells.cellHeight * cells.firstRow
                        onFocusRowChanged: cells.firstRow = Math.max(0, Math.min(cells.firstRow, cells.focusRow), cells.focusRow - root.rowsShown + 1)
                        onFirstRowYChanged: cells.contentY = cells.firstRowY
                        model: Browse.games
                        delegate: Cell {
                            id: cell
                            readonly property bool lit: cell.GridView.isCurrentItem && cells.activeFocus
                            readonly property int row: Math.floor(cell.index / root.columns)
                            // Only the rows shown are drawn: the list's margin for the ring would show a part of the next.
                            visible: cell.row >= cells.firstRow && cell.row < cells.firstRow + root.rowsShown
                            controller: frame.controller
                            objectName: "gamesTile." + cell.index
                            Accessible.name: cell.title
                            width: root.tileWidth
                            height: root.tileHeight
                            z: cell.lit ? 1 : 0
                            onActivated: Actions.play(cell.key)
                            onActiveFocusChanged: if (cell.activeFocus) Actions.focus(cell.key)
                            Rectangle {
                                objectName: "gamesTileGround." + cell.index
                                anchors.fill: parent
                                visible: !picture.drawn
                                radius: 16 * root.px
                                color: Tokens.surfaceRaised
                            }
                            RoundPicture {
                                id: picture
                                objectName: "gamesTileCover." + cell.index
                                readonly property bool drawn: picture.status === Image.Ready
                                anchors.fill: parent
                                visible: cell.cover.length > 0
                                source: cell.cover
                                crop: true
                                radius: 16 * root.px
                                pixelScale: canvas.unit
                            }
                            Rectangle {
                                objectName: "gamesTileFoot." + cell.index
                                anchors.bottom: parent.bottom
                                width: parent.width
                                height: 84 * root.px
                                radius: 16 * root.px
                                gradient: Gradient {
                                    GradientStop { position: 0; color: "transparent" }
                                    GradientStop { position: 1; color: Tokens.overArtScrim }
                                }
                            }
                            Text {
                                objectName: "gamesTileTitle." + cell.index
                                x: 14 * root.px
                                width: parent.width - 28 * root.px
                                anchors.bottom: parent.bottom
                                anchors.bottomMargin: 12 * root.px
                                text: cell.title
                                wrapMode: Text.Wrap
                                maximumLineCount: 2
                                elide: Text.ElideRight
                                color: Tokens.overArt
                                font.family: medium.font.family
                                font.pixelSize: Math.round(18 * root.px)
                                lineHeight: 1.2
                            }
                            FocusRing {
                                objectName: "gamesRing." + cell.index
                                visible: cell.lit
                                anchors.fill: parent
                                radius: 16 * root.px
                            }
                        }
                    }
                    // The focus as the program holds it: the theme reports each move (Actions.focus) and the List card moves it too.
                    Binding {
                        cells.currentIndex: Browse.focusRow
                    }
                    Column {
                        visible: cells.count === 0
                        x: 96 * root.px
                        y: 250 * root.px
                        width: 880 * root.px
                        spacing: 14 * root.px
                        Text {
                            objectName: "emptyHeading"
                            width: parent.width
                            text: root.noLibrary ? Words.noGames : Browse.emptyWords
                            wrapMode: Text.Wrap
                            color: Tokens.text
                            font.family: medium.font.family
                            font.pixelSize: Math.round(84 * root.px)
                            font.letterSpacing: -84 * 0.02 * root.px
                            lineHeight: 1.04
                        }
                        Text {
                            objectName: "emptyLine"
                            visible: root.noLibrary
                            width: parent.width
                            text: Words.addFolder
                            wrapMode: Text.Wrap
                            color: ink.words
                            font.family: regular.font.family
                            font.pixelSize: Math.round(25 * root.px)
                            lineHeight: 1.45
                        }
                    }
                }
            }
            GamesInk {
                id: ink
            }
            FontLoader {
                id: regular
                source: "qrc:/emusen/fonts/Barlow-Regular.ttf"
            }
            FontLoader {
                id: medium
                source: "qrc:/emusen/fonts/Barlow-Medium.ttf"
            }
        }
    }
}
