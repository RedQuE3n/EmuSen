pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme
import "../parts"

// Eclipse's games view as its board draws it (EclipseLibrary.dc.html), each element in qt-app/tests/eclipse/games.tsv; with no game in the library the first-run words over the waiting disc (EmuSen_Qt.md §3.4.10).
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
        required property string consoleName
        required property real consoleHue
        required property int year
        required property bool played
        required property string lastPlayedWords
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
            readonly property var focusedCell: cells.currentItem
            // No console has a game (Choose a console's "With games", Words.consoleShows[1]), so the library has none: the first-run state.
            readonly property bool noLibrary: Library.consolesBy(1, 0).count === 0 && cells.count === 0 && !Browse.searched
            // The board's measures in its pixels, constants of the design (EclipseLibrary:64-73, 88).
            readonly property real margin: 96 * root.px
            readonly property real gridX: 660 * root.px
            readonly property real gridY: 236 * root.px
            readonly property real gridRight: 150 * root.px
            readonly property real tileWidth: 206 * root.px
            readonly property real tileHeight: 170 * root.px
            readonly property real tileGap: 20 * root.px
            readonly property real foot: 1114 * root.px
            // The columns that fit between the grid's left edge and its right margin, and the whole rows above the foot.
            readonly property int columns: Math.max(1, Math.floor((root.width - root.gridX - root.gridRight + root.tileGap) / (root.tileWidth + root.tileGap)))
            readonly property int rowsShown: Math.max(1, Math.floor((root.foot - root.gridY + root.tileGap) / (root.tileHeight + root.tileGap)))
            readonly property real rowsHeight: root.rowsShown * (root.tileHeight + root.tileGap) - root.tileGap
            // How far a tile's ring and glow reach past it, so the list's clip leaves them whole.
            readonly property real reach: (4 + 3 + 40 + 10) * root.px
            // The filter in force's place among the chips, for the walk up from the tiles.
            property int filterAt: 0
            property string lastLetter: ""
            // The help's words this board draws: A's and X's on the buttons, B's on the path, the triggers' at the right.
            property string startWords
            property string pageWords
            property string backWords
            property string letterWords
            readonly property var gamesHelp: Help.entries(Browse.searched ? Help.SectionSearched : Help.Section, { games: true })
            Board {
                id: board
                designHeight: canvas.designHeight
            }
            Ink {
                id: ink
            }
            GamesInk {
                id: own
            }
            // The focused game's hues for the board's two fields and its disc (R-20261115-27, -28).
            ArtHues {
                id: hues
                key: root.focusedCell ? root.focusedCell.key : ""
                consoleHue: root.focusedCell ? root.focusedCell.consoleHue : NaN
            }
            FieldInk {
                id: field0
                board: ink.field0
                hue: hues.hues.length > 0 ? hues.hues[0] : NaN
            }
            FieldInk {
                id: field1
                board: ink.field1
                hue: hues.hues.length > 0 ? hues.hues[1] : NaN
            }
            Backdrop {
                objectName: "gamesBackdrop"
                // Drawn behind everything, as a sky is, not beside it.
                readonly property bool backdrop: true
                anchors.fill: parent
                ground: root.noLibrary ? ink.emptyGround : ink.ground
                pixelScale: canvas.unit
                fields: [
                    BackdropField {
                        x: 700 * root.px
                        y: -200 * root.px
                        width: 900 * root.px
                        height: 800 * root.px
                        color: root.noLibrary ? "transparent" : field0.color
                        blur: 120 * root.px
                        opacity: 0.35
                    },
                    BackdropField {
                        x: 1200 * root.px
                        y: 500 * root.px
                        width: 900 * root.px
                        height: 900 * root.px
                        color: root.noLibrary ? "transparent" : field1.color
                        blur: 130 * root.px
                        opacity: 0.4
                    }
                ]
            }
            CoronaDisc {
                id: disc
                objectName: "gamesDisc"
                visible: !root.noLibrary
                x: -1100 * root.px
                y: -250 * root.px
                width: 1700 * root.px
                height: 1700 * root.px
                // The focused game's first hue at the look's rim's saturation and lightness, its glows at the board's strengths (EclipseLibrary:20).
                corona: hues.hues.length > 0 ? Qt.hsla((((hues.hues[0] % 360) + 360) % 360) / 360, Tokens.coronaColor.hslSaturation, Tokens.coronaColor.hslLightness, 1) : "transparent"
                rim: 2 * root.px
                pixelScale: canvas.unit
                glows: [
                    CoronaGlow {
                        blur: 44 * root.px
                        spread: 8 * root.px
                        color: Qt.rgba(disc.corona.r, disc.corona.g, disc.corona.b, 0.55)
                    },
                    CoronaGlow {
                        blur: 170 * root.px
                        spread: 46 * root.px
                        color: Qt.rgba(disc.corona.r, disc.corona.g, disc.corona.b, 0.25)
                    }
                ]
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
                // The help line below draws the board's entries; the frame keeps every action, the triggers' letters among them.
                helpShown: false
                focus: true
                controller.navigationShown: false
                help: Help.entries(Browse.searched ? Help.SectionSearched : Help.Section, { letters: true })
                Connections {
                    target: frame.controller
                    function onBackDeclined() { Actions.back() }
                    function onActionInvoked(key) { Actions.act(key) }
                }
                Repeater {
                    model: frame.help
                    Item {
                        id: declared
                        required property int button
                        required property string label
                        Binding {
                            when: declared.button === PadGlyph.Screensaver
                            target: root
                            property: "pageWords"
                            value: declared.label
                        }
                        Binding {
                            when: declared.button === PadGlyph.Back
                            target: root
                            property: "backWords"
                            value: declared.label
                        }
                        Binding {
                            when: declared.button === PadGlyph.First
                            target: root
                            property: "letterWords"
                            value: declared.label
                        }
                    }
                }
                Repeater {
                    model: root.gamesHelp
                    Item {
                        id: named
                        required property int button
                        required property string label
                        Binding {
                            when: named.button === PadGlyph.Accept
                            target: root
                            property: "startWords"
                            value: named.label
                        }
                    }
                }
                Item {
                    x: -frame.safeX
                    y: -frame.safeY
                    width: root.width
                    height: root.height

                    // The header: the mark, the path (the place B goes back to, then the section), and the status on its glass (EclipseLibrary:22-35).
                    Brand {
                        id: brand
                        objectName: "gamesBrand"
                        unit: root.px
                        named: root.noLibrary
                        x: root.margin
                        // The header's middle line: the games board's 64-pixel header and the empty board's mark (EclipseLibrary:23, EclipseEmpty:22).
                        y: (root.noLibrary ? 48 + 17 : 48 + 32) * root.px - height / 2
                    }
                    Walkable {
                        id: crumb
                        objectName: "gamesPath"
                        visible: !root.noLibrary
                        controller: frame.controller
                        Accessible.name: crumbBack.text
                        x: brand.x + brand.width + 28 * root.px
                        y: 80 * root.px - height / 2
                        width: crumbHere.x + crumbHere.width
                        height: crumbBack.height
                        onActivated: Actions.back()
                        Keys.onDownPressed: filterChips.itemAt(Math.max(0, root.filterAt))?.forceActiveFocus()
                        Line {
                            id: crumbBack
                            objectName: "gamesPathBack"
                            unit: root.px
                            text: root.backWords
                            color: Tokens.muted
                            size: 24
                        }
                        Line {
                            id: crumbSlash
                            objectName: "gamesPathSlash"
                            unit: root.px
                            x: crumbBack.width + 14 * root.px
                            text: Words.crumb
                            color: own.slash
                            size: 24
                        }
                        Line {
                            id: crumbHere
                            objectName: "gamesPathHere"
                            unit: root.px
                            x: crumbSlash.x + crumbSlash.width + 14 * root.px
                            text: Browse.heading
                            color: Tokens.text
                            size: 24
                            weight: Font.DemiBold
                        }
                        FocusRing {
                            anchors.fill: parent
                            visible: crumb.activeFocus
                        }
                    }
                    Status {
                        objectName: "gamesStatus"
                        unit: root.px
                        controller: frame.controller
                        visible: !root.noLibrary
                        searchable: false
                        battery: true
                        glassColor: ink.glass
                        x: root.width - root.margin - width
                        y: 80 * root.px - height / 2
                    }

                    // The focused game's column: its console and year, its name, its last play, and A and X as buttons (EclipseLibrary:37-53).
                    Column {
                        objectName: "gamesAside"
                        visible: !root.noLibrary && root.focusedCell !== null
                        x: root.margin
                        y: 230 * root.px
                        width: 440 * root.px
                        spacing: 22 * root.px
                        Line {
                            objectName: "gamesConsole"
                            unit: root.px
                            width: parent.width
                            text: root.focusedCell ? Words.line([root.focusedCell.consoleName, root.focusedCell.year > 0 ? String(root.focusedCell.year) : ""]) : ""
                            wrapMode: Text.Wrap
                            color: ink.rose
                            size: 18
                            weight: Font.DemiBold
                            spacing: 0.2 * 18
                            capitalization: Font.AllUppercase
                        }
                        Line {
                            objectName: "gamesTitle"
                            unit: root.px
                            width: parent.width
                            text: root.focusedCell ? root.focusedCell.title : ""
                            wrapMode: Text.Wrap
                            color: Tokens.text
                            size: 62
                            weight: Font.DemiBold
                            spacing: -0.02 * 62
                            lineHeightMode: Text.FixedHeight
                            lineHeight: 62 * 1.04
                        }
                        Item {
                            objectName: "gamesFact.0"
                            visible: root.focusedCell !== null && root.focusedCell.played
                            width: parent.width
                            height: factLabel.height
                            Line {
                                id: factLabel
                                objectName: "gamesFactLabel.0"
                                unit: root.px
                                text: Words.gameFactLabels[0]
                                color: Tokens.muted
                                size: 20
                            }
                            Line {
                                objectName: "gamesFactValue.0"
                                unit: root.px
                                x: parent.width - width
                                text: root.focusedCell ? root.focusedCell.lastPlayedWords : ""
                                color: Tokens.text
                                size: 20
                            }
                        }
                        Row {
                            objectName: "gamesActions"
                            // The board's six pixels over the buttons (EclipseLibrary:50).
                            topPadding: 6 * root.px
                            spacing: 14 * root.px
                            Pill {
                                id: startPill
                                objectName: "gamesStart"
                                controller: frame.controller
                                unit: root.px
                                button: PadGlyph.Accept
                                label: root.startWords
                                filled: true
                                glows: false
                                tall: 64
                                glyphSide: 42
                                leadLetterSize: 19
                                labelSize: 23
                                lead: 12
                                trail: 28
                                gap: 12
                                place: 0
                                pixelScale: canvas.unit
                                onActivated: Actions.play(root.focusedCell?.key ?? "")
                                KeyNavigation.right: pagePill
                                // Up from the column reaches the path above it.
                                KeyNavigation.up: crumb
                            }
                            Pill {
                                id: pagePill
                                objectName: "gamesPage"
                                controller: frame.controller
                                unit: root.px
                                button: PadGlyph.Screensaver
                                label: root.pageWords
                                outline: ink.pillLine
                                tall: 64
                                glyphSide: 40
                                letterSize: 18
                                labelSize: 23
                                lead: 12
                                trail: 26
                                gap: 12
                                place: 1
                                pixelScale: canvas.unit
                                onActivated: Actions.openPage(root.focusedCell?.key ?? "")
                                KeyNavigation.left: startPill
                                Keys.onRightPressed: cells.forceActiveFocus()
                                KeyNavigation.up: crumb
                            }
                        }
                    }

                    // The chips: every game, the favourites and those not played, each with its count, the one in force filled (EclipseLibrary:55-57).
                    Row {
                        objectName: "gamesFilters"
                        visible: !root.noLibrary
                        x: root.gridX
                        y: 150 * root.px
                        spacing: 14 * root.px
                        Repeater {
                            id: filterChips
                            model: Browse.filters
                            delegate: Walkable {
                                id: chip
                                required property int index
                                required property string key
                                required property string chip
                                readonly property bool chosen: chip.key === Browse.filter
                                // A button's box holds its line (CSS's border-box): the chip in force has none.
                                readonly property real line: chip.chosen ? 0 : 2 * root.px
                                // A collection is the List card's alone: the board's chips are the three.
                                visible: chip.chip.length > 0
                                controller: frame.controller
                                objectName: "gamesFilter." + chip.index
                                Accessible.name: chipWords.text
                                Binding {
                                    when: chip.chosen
                                    target: root
                                    property: "filterAt"
                                    value: chip.index
                                }
                                width: chipWords.width + 2 * (22 * root.px + chip.line)
                                height: 48 * root.px
                                onActivated: Actions.setFilter(chip.key)
                                Keys.onLeftPressed: if (chip.index > 0) filterChips.itemAt(chip.index - 1).forceActiveFocus()
                                Keys.onRightPressed: {
                                    const next = filterChips.itemAt(chip.index + 1)
                                    if (next && next.visible)
                                        next.forceActiveFocus()
                                    else
                                        sortPill.forceActiveFocus()
                                }
                                KeyNavigation.up: crumb
                                Keys.onDownPressed: cells.forceActiveFocus()
                                Rectangle {
                                    objectName: "gamesFilterFill." + chip.index
                                    anchors.fill: parent
                                    radius: height / 2
                                    color: chip.chosen ? Tokens.accent : own.chipGround
                                    border.width: chip.line
                                    border.color: ink.chipLine
                                }
                                Line {
                                    id: chipWords
                                    objectName: "gamesFilterWords." + chip.index
                                    unit: root.px
                                    x: 22 * root.px + chip.line
                                    y: (parent.height - height) / 2
                                    text: chip.chip
                                    color: chip.chosen ? Tokens.overAccent : ink.tileFacts
                                    size: 21
                                    weight: chip.chosen ? Font.DemiBold : Font.Normal
                                }
                                FocusRing {
                                    visible: chip.activeFocus
                                    anchors.fill: parent
                                    radius: height / 2
                                }
                            }
                        }
                    }
                    // The order in force on its glass, View its button: pressed, it opens the List card as View does (EclipseLibrary:59-62).
                    Walkable {
                        id: sortPill
                        objectName: "gamesSort"
                        visible: !root.noLibrary && Browse.sortLine.length > 0
                        controller: frame.controller
                        Accessible.name: sortWords.text
                        x: root.width - root.margin - width
                        y: 150 * root.px
                        width: sortWords.x + sortWords.width + 18 * root.px
                        height: 48 * root.px
                        onActivated: frame.controller.offer(PadGlyph.Options)
                        Keys.onLeftPressed: {
                            for (let i = filterChips.count - 1; i >= 0; --i) {
                                if (filterChips.itemAt(i).visible)
                                    return filterChips.itemAt(i).forceActiveFocus()
                            }
                        }
                        Keys.onDownPressed: cells.forceActiveFocus()
                        Rectangle {
                            objectName: "gamesSortFill"
                            anchors.fill: parent
                            radius: height / 2
                            color: ink.glass
                        }
                        Glyph {
                            id: sortGlyph
                            objectName: "gamesSortGlyph"
                            unit: root.px
                            x: 18 * root.px
                            y: (parent.height - height) / 2
                            button: PadGlyph.Options
                            side: 30
                            wordSize: 14
                            padding: 10
                            ink: ink.help
                        }
                        Line {
                            id: sortWords
                            objectName: "gamesSortWords"
                            unit: root.px
                            x: sortGlyph.x + sortGlyph.width + 12 * root.px
                            y: (parent.height - height) / 2
                            text: Browse.sortLine
                            color: ink.help
                            size: 21
                        }
                        FocusRing {
                            visible: sortPill.activeFocus
                            anchors.fill: parent
                            radius: height / 2
                        }
                    }

                    // The section's games as tiles, five columns at the board's width (EclipseLibrary:64-71).
                    GridView {
                        id: cells
                        objectName: "gamesGrid"
                        visible: !root.noLibrary
                        x: root.gridX - root.reach
                        y: root.gridY - root.reach
                        width: root.columns * cells.cellWidth - root.tileGap + 2 * root.reach
                        height: root.rowsHeight + 2 * root.reach
                        leftMargin: root.reach
                        topMargin: root.reach
                        cellWidth: root.tileWidth + root.tileGap
                        cellHeight: root.tileHeight + root.tileGap
                        clip: true
                        focus: true
                        keyNavigationWraps: false
                        highlightFollowsCurrentItem: false
                        // The wheel scrolls the rows, one a notch; the arrows walk the tiles and bring the focused one back in.
                        interactive: false
                        keyNavigationEnabled: true
                        WheelHandler {
                            onWheel: (event) => {
                                // A notch is 120 eighths of a degree (QWheelEvent::angleDelta), at least one row.
                                const notches = Math.max(1, Math.round(Math.abs(event.angleDelta.y) / 120))
                                const last = Math.max(0, Math.ceil(cells.count / root.columns) - root.rowsShown)
                                cells.firstRow = Math.max(0, Math.min(last, cells.firstRow + (event.angleDelta.y < 0 ? notches : -notches)))
                            }
                        }
                        // Left from the first column reaches the buttons.
                        Keys.onLeftPressed: (event) => {
                            if (cells.currentIndex % root.columns === 0)
                                startPill.forceActiveFocus();
                            else
                                event.accepted = false;
                        }
                        // Up from the first row reaches the chip in force.
                        Keys.onUpPressed: (event) => {
                            const chip = filterChips.itemAt(Math.max(0, root.filterAt))
                            if (cells.currentIndex < root.columns && chip)
                                chip.forceActiveFocus();
                            else
                                event.accepted = false;
                        }
                        // The rows shown move only when the focus leaves them, a whole row at a time.
                        property int firstRow: 0
                        readonly property int focusRow: Math.floor(Math.max(0, cells.currentIndex) / root.columns)
                        onFocusRowChanged: cells.firstRow = Math.max(0, Math.min(cells.firstRow, cells.focusRow), cells.focusRow - root.rowsShown + 1)
                        // Set when the rows change, not bound: the view's origin moves as it scrolls, which a binding on it would loop on.
                        function showRows() { cells.contentY = cells.originY - cells.topMargin + cells.cellHeight * cells.firstRow }
                        onFirstRowChanged: cells.showRows()
                        onHeightChanged: cells.showRows()
                        onWidthChanged: cells.showRows()
                        onCountChanged: cells.showRows()
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
                            // A resting tile's shadow, 0 8 px 24 px rgba(0, 0, 0, 0.4) (EclipseLibrary:66); the focused one has its ring instead.
                            Glow {
                                objectName: "gamesTileShadow." + cell.index
                                anchors.fill: parent
                                visible: !cell.lit
                                radius: 16 * root.px
                                color: own.tileShadow
                                blur: 24 * root.px
                                offsetY: 8 * root.px
                                pixelScale: canvas.unit
                            }
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
                            // The tile's foot (EclipseLibrary:67-68): the 84 px shade, the title 12 px over the foot, inset 14, wrapped whole.
                            TileWords {
                                unit: root.px
                                place: cell.index
                                title: cell.title
                                facts: ""
                                titleLines: 0
                                titleLineHeight: 18 * 1.2
                                radius: 16
                                titleSize: 18
                                inset: 14
                                titleFoot: 12
                                fade: 84
                                titleInk: ink.tileTitle
                                shadeInk: Tokens.overArtScrim
                                shadeClear: ink.shadeClear
                            }
                            // The board's ring (EclipseLibrary:93): 4 px of the ground's dark, 3 px of the accent, 40 px of glow 10 px out.
                            FocusRing {
                                objectName: "gamesRing." + cell.index
                                visible: cell.lit
                                anchors.fill: parent
                                radius: 16 * root.px
                                gap: 4 * root.px
                                thickness: 3 * root.px
                                glowColor: ink.ringGlow
                                glowBlur: 40 * root.px
                                glowSpread: 10 * root.px
                                bandRadius: (16 + 7) * root.px
                                pixelScale: canvas.unit
                            }
                        }
                    }
                    // The focus as the program holds it: the theme reports each move (Actions.focus) and the List card moves it too.
                    Binding {
                        cells.currentIndex: Browse.focusRow
                    }

                    // The letter rail: the focused game's letter, the track, the last letter; a press on the track steps to the letter there (EclipseLibrary:75-79).
                    Column {
                        objectName: "gamesRail"
                        visible: !root.noLibrary && cells.count > 0
                        x: root.width - (44 + 34) * root.px
                        y: root.gridY
                        width: 34 * root.px
                        height: root.rowsHeight
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
                            Line {
                                objectName: "gamesRailLetterWords"
                                unit: root.px
                                anchors.centerIn: parent
                                text: Browse.letter
                                color: Tokens.overAccent
                                size: 17
                                weight: Font.Bold
                            }
                        }
                        Rectangle {
                            objectName: "gamesRailTrack"
                            anchors.horizontalCenter: parent.horizontalCenter
                            width: 4 * root.px
                            height: parent.height - parent.width - lastLetterWords.height - 2 * parent.spacing
                            radius: width / 2
                            color: own.railTrack
                            MouseArea {
                                // As wide as the rail, so a press near the thin track lands on it.
                                anchors.centerIn: parent
                                width: 34 * root.px
                                height: parent.height
                                onClicked: (mouse) => {
                                    let at = -1
                                    for (let i = 0; i < letters.count; ++i)
                                        if (letters.itemAt(i).letter === Browse.letter)
                                            at = i
                                    const to = Math.round(mouse.y / height * Math.max(0, letters.count - 1))
                                    if (at >= 0 && to !== at)
                                        Actions.stepLetter(to - at)
                                }
                            }
                        }
                        Line {
                            id: lastLetterWords
                            objectName: "gamesRailLast"
                            unit: root.px
                            anchors.horizontalCenter: parent.horizontalCenter
                            text: root.lastLetter
                            color: ink.help
                            size: 16
                        }
                    }

                    // The foot: A Start, B Back and Y Favourite, and at the right the triggers' letters and the focused game's place (EclipseLibrary:82-91).
                    HelpLine {
                        objectName: "gamesHelp"
                        unit: root.px
                        controller: frame.controller
                        visible: !Browse.typing && !root.noLibrary
                        model: root.gamesHelp
                        ink: ink.help
                        Item {
                            objectName: "gamesLetters"
                            width: letterWords.x + letterWords.width + 18 * root.px
                            height: 46 * root.px
                            Rectangle {
                                objectName: "gamesLettersFill"
                                anchors.fill: parent
                                radius: height / 2
                                color: ink.glass
                            }
                            Glyph {
                                id: letterBack
                                objectName: "gamesLetterBack"
                                unit: root.px
                                x: 18 * root.px
                                y: (parent.height - height) / 2
                                button: PadGlyph.First
                                wordSize: 15
                                ink: ink.help
                                TapHandler {
                                    onTapped: Actions.stepLetter(-1)
                                }
                            }
                            Glyph {
                                id: letterOn
                                objectName: "gamesLetterOn"
                                unit: root.px
                                x: letterBack.x + letterBack.width + 10 * root.px
                                y: (parent.height - height) / 2
                                button: PadGlyph.Last
                                wordSize: 15
                                ink: ink.help
                                TapHandler {
                                    onTapped: Actions.stepLetter(1)
                                }
                            }
                            Line {
                                id: letterWords
                                objectName: "gamesLetterWords"
                                unit: root.px
                                x: letterOn.x + letterOn.width + 10 * root.px
                                y: (parent.height - height) / 2
                                text: Words.line([root.letterWords, cells.count > 0 ? Words.place(Math.max(0, cells.currentIndex), cells.count) : ""])
                                color: ink.help
                                size: 21
                            }
                        }
                    }

                    // An empty list's words, and with no game at all the first-run words (EclipseEmpty.dc.html).
                    Column {
                        visible: cells.count === 0
                        x: root.margin
                        y: 250 * root.px
                        width: 880 * root.px
                        spacing: 14 * root.px
                        Line {
                            objectName: "emptyHeading"
                            unit: root.px
                            width: parent.width
                            text: root.noLibrary ? Words.noGames : Browse.emptyWords
                            wrapMode: Text.Wrap
                            color: Tokens.text
                            size: 84
                            weight: Font.DemiBold
                            spacing: -0.02 * 84
                            lineHeightMode: Text.FixedHeight
                            lineHeight: 84 * 1.04
                        }
                        Line {
                            objectName: "emptyLine"
                            visible: root.noLibrary
                            unit: root.px
                            width: parent.width
                            text: Words.addFolder
                            wrapMode: Text.Wrap
                            color: own.words
                            size: 25
                            weight: Font.Light
                            lineHeightMode: Text.FixedHeight
                            lineHeight: 25 * 1.45
                        }
                    }
                }
            }
        }
    }
}
