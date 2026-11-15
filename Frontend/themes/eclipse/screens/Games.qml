pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme
import "../parts"

// Eclipse's games view: the focused game's words at the left, the section's games as tiles beside them, and with no game in the library the board's first-run words over the waiting disc; each element and its board line in qt-app/tests/eclipse/games.tsv (EmuSen_Qt.md §3.4.10).
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
        required property string hero
    }
    NormalizedCanvas {
        id: canvas
        objectName: "gridCanvas"
        FocusScope {
            id: root
            objectName: "grid"
            anchors.fill: parent
            focus: true
            // A board pixel in canvas units, Board's.
            readonly property real px: board.unit
            // Every size of words is Type's: the board's at the player's text scale, never under the legibility floor.
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
            // The pill that plays, for the walk left from the tiles' first column.
            property Item firstPill: null
            property Item pagePill: null
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
                color: root.noLibrary ? shared.emptyGround : shared.ground
            }
            // The focused game's art blurred behind the disc, its hero else its cover as the row gives it; none for a game with no art.
            Backdrop {
                objectName: "gamesBackdrop"
                anchors.fill: parent
                visible: !root.noLibrary
                source: root.focusedCell ? root.focusedCell.hero : ""
                // The board's blur and opacity over its sample art (EclipseLibrary:19), the same on the software backend; provisional, the tester's to judge.
                sourceBlur: 120 * root.px
                sourceOpacity: 0.35
                pixelScale: canvas.unit
            }
            CoronaDisc {
                objectName: "gamesDisc"
                visible: !root.noLibrary
                x: -1100 * root.px
                y: -250 * root.px
                width: 1700 * root.px
                height: 1700 * root.px
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
                // The help line below draws the help; the frame keeps its actions.
                helpShown: false
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
                    Brand {
                        unit: root.px
                        named: root.noLibrary
                        x: 96 * root.px
                        // The header's middle line: the games board's 64-pixel header and the empty board's mark (EclipseLibrary:23, EclipseEmpty:22).
                        y: (root.noLibrary ? 48 + 17 : 48 + 32) * root.px - height / 2
                    }
                    Status {
                        unit: root.px
                        controller: frame.controller
                        visible: !root.noLibrary
                        searchable: false
                        glassColor: shared.glass
                        x: root.width - 96 * root.px - width
                        y: (48 + 32) * root.px - height / 2
                    }
                    HelpLine {
                        unit: root.px
                        controller: frame.controller
                        visible: !Browse.typing && !root.noLibrary
                        model: frame.help
                        ink: shared.help
                    }
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
                            color: shared.rose
                            font.weight: Font.DemiBold
                            font.pixelSize: Type.size(18 * root.px, Tokens.textScale)
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
                            font.weight: Font.DemiBold
                            font.pixelSize: Type.size(62 * root.px, Tokens.textScale)
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
                            color: own.words
                            font.weight: Font.Light
                            font.pixelSize: Type.size(20 * root.px, Tokens.textScale)
                            lineHeight: 1.5
                        }
                        Row {
                            objectName: "gamesActions"
                            // The board's six pixels over the buttons (EclipseLibrary:50).
                            topPadding: 6 * root.px
                            spacing: 14 * root.px
                            // A and X as the help names them: the program's words for playing and for the game's page.
                            Repeater {
                                id: actionPills
                                model: frame.help
                                delegate: Pill {
                                    id: pill
                                    required property int index
                                    readonly property bool plays: pill.button === PadGlyph.Accept
                                    controller: frame.controller
                                    objectName: "gamesAction." + pill.index
                                    visible: pill.plays || pill.button === PadGlyph.Screensaver
                                    place: pill.index
                                    unit: root.px
                                    filled: pill.plays
                                    tall: 64
                                    glyphSide: pill.plays ? 42 : 40
                                    labelSize: 23
                                    lead: 12
                                    trail: pill.plays ? 28 : 26
                                    gap: 12
                                    onActivated: pill.plays ? Actions.play(root.focusedCell.key) : Actions.openPage(root.focusedCell.key)
                                    Keys.onRightPressed: pill.plays && root.pagePill ? root.pagePill.forceActiveFocus() : cells.forceActiveFocus()
                                    Keys.onLeftPressed: if (!pill.plays && root.firstPill) root.firstPill.forceActiveFocus()
                                    Binding {
                                        when: pill.button === PadGlyph.Screensaver
                                        target: root
                                        property: "pagePill"
                                        value: pill
                                    }
                                    Binding {
                                        when: pill.plays
                                        target: root
                                        property: "firstPill"
                                        value: pill
                                    }
                                }
                            }
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
                                    color: chip.chosen ? Tokens.accent : own.chipGround
                                    border.width: chip.chosen ? 0 : 2 * root.px
                                    border.color: own.chipLine
                                }
                                Text {
                                    id: chipWords
                                    objectName: "gamesFilterWords." + chip.index
                                    anchors.centerIn: parent
                                    text: chip.line
                                    color: chip.chosen ? Tokens.overAccent : shared.tileFacts
                                    font.weight: chip.chosen ? Font.DemiBold : Font.Normal
                                    font.pixelSize: Type.size(21 * root.px, Tokens.textScale)
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
                            color: shared.glass
                        }
                        Text {
                            id: sortWords
                            objectName: "gamesSortWords"
                            anchors.centerIn: parent
                            text: Browse.sort
                            color: shared.help
                            font.pixelSize: Type.size(21 * root.px, Tokens.textScale)
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
                                font.weight: Font.DemiBold
                                font.pixelSize: Type.size(17 * root.px, Tokens.textScale)
                            }
                        }
                        Rectangle {
                            objectName: "gamesRailTrack"
                            anchors.horizontalCenter: parent.horizontalCenter
                            width: 4 * root.px
                            height: parent.height - parent.width - lastLetter.height - 2 * parent.spacing
                            radius: width / 2
                            color: own.railTrack
                        }
                        Text {
                            id: lastLetter
                            objectName: "gamesRailLast"
                            anchors.horizontalCenter: parent.horizontalCenter
                            text: root.lastLetter
                            color: shared.help
                            font.pixelSize: Type.size(16 * root.px, Tokens.textScale)
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
                        // Left from the first column reaches the pills.
                        Keys.onLeftPressed: (event) => {
                            if (cells.currentIndex % root.columns === 0 && root.firstPill)
                                root.firstPill.forceActiveFocus();
                            else
                                event.accepted = false;
                        }
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
                            // The shared foot (TileWords): the board's 84 px fade to Tokens.overArtScrim, held under the title's two lines.
                            TileWords {
                                unit: root.px
                                kind: "gamesTile"
                                place: cell.index
                                title: cell.title
                                facts: ""
                                titleLines: 2
                                titleLineHeight: 1.2
                                radius: 16
                                titleSize: 18
                                inset: 14
                                fade: 84
                                titleInk: Tokens.overArt
                                shadeInk: Tokens.overArtScrim
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
                            font.weight: Font.DemiBold
                            font.pixelSize: Type.size(84 * root.px, Tokens.textScale)
                            font.letterSpacing: -84 * 0.02 * root.px
                            lineHeight: 1.04
                        }
                        Text {
                            objectName: "emptyLine"
                            visible: root.noLibrary
                            width: parent.width
                            text: Words.addFolder
                            wrapMode: Text.Wrap
                            color: own.words
                            font.weight: Font.Light
                            font.pixelSize: Type.size(25 * root.px, Tokens.textScale)
                            lineHeight: 1.45
                        }
                    }
                }
            }
            GamesInk {
                id: own
            }
            Ink {
                id: shared
            }
            Board {
                id: board
                designHeight: canvas.designHeight
            }
        }
    }
}
