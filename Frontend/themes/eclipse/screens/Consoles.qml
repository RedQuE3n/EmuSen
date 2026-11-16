pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Theme
import "../parts"

// Eclipse's Choose a console: the chips and the order at the top, the makers in a column, the chosen maker's consoles as plates laid out by the kit's GroupGridLayout, the focused console's disc, name and facts beside them (EmuSen_Qt.md §3.4.9).
ThemeScreen {
    id: screen
    objectName: "consolesScreen"
    role: ThemeScreen.Consoles
    // The plate it opens on, else (no console listed) the pressed chip.
    focusItems: [
        root.firstPlate ?? root.firstChip
    ]
    NormalizedCanvas {
        id: canvas
        objectName: "consolesCanvas"
        FocusScope {
            id: root
            objectName: "consoles"
            anchors.fill: parent
            focus: true
            // A board pixel in canvas units (the table is eclipse/consoles.tsv).
            readonly property real px: board.unit
            // The board's measures in its pixels, constants of the design itself.
            readonly property real margin: 96 * root.px
            readonly property real listTop: 250 * root.px
            readonly property real foot: 1126 * root.px
            readonly property real makerWidth: 250 * root.px
            readonly property real makerHeight: 60 * root.px
            readonly property real makerPitch: 66 * root.px
            readonly property real plateWidth: 236 * root.px
            readonly property real plateHeight: 178 * root.px
            readonly property real plateGap: 20 * root.px
            readonly property real asideWidth: 374 * root.px
            readonly property real asideGap: 56 * root.px
            readonly property real gridX: 390 * root.px
            // Board colours no token holds, drawn only here: the plate's edge (board line 64) and the track (line 66).
            readonly property color plateEdge: "#262A3A"
            readonly property color trackInk: "#2C3044"
            readonly property real asideX: root.width - root.margin - root.asideWidth
            readonly property real gridWidth: root.asideX - root.asideGap - root.gridX
            readonly property int columns: Math.max(1, Math.floor((root.gridWidth + root.plateGap) / (root.plateWidth + root.plateGap)))
            // As many whole rows of plates and of makers as stand between the list's top and the foot, so none is cut by the
            // view's edge or lies under the help line; the focus brings the next row in whole.
            readonly property real platesHeight: Math.max(1, Math.floor((root.foot - root.listTop + root.plateGap) / (root.plateHeight + root.plateGap))) * (root.plateHeight + root.plateGap) - root.plateGap
            readonly property real makersHeight: Math.max(1, Math.floor((root.foot - root.listTop + root.makerPitch - root.makerHeight) / root.makerPitch)) * root.makerPitch - (root.makerPitch - root.makerHeight)
            // The chip pressed (Words.consoleShows' order) and the order in force (Words.consoleSorts'), the theme's own.
            property int show: 0
            property int order: 0
            // The consoles the chip and the order give, each maker's together.
            readonly property var listing: Library.consolesBy(root.show, root.order)
            // The chosen maker's group, the console the aside shows, and the plate last focused.
            property int chosenGroup: 0
            property Item shown: null
            property Item lastPlate: null
            // The plate the screen opens on: the console last opened, else the first maker's first console; and the maker focused.
            property Item firstPlate: null
            property Item firstChip: null
            property Item focusedMaker: null
            // The help's actions for Y and X (Help.entries' `action`), read from the frame's own list.
            property string sortAction
            property string optionsAction
            // The plate under the chips: the one last focused where its maker is still the chosen one, else the chosen maker's first
            // (a maker focused chooses its group and hides the plate last focused, which KeyNavigation would skip past).
            property Item chosenFirst: null
            readonly property Item belowChips: (root.lastPlate !== null && root.lastPlate.group === root.chosenGroup ? root.lastPlate : root.chosenFirst) ?? root.firstPlate
            // The frame's declared action, by its key: Y the next order, X the console's options, any other the host's.
            signal invoked(string key)
            onInvoked: (key) => {
                if (key === root.sortAction)
                    root.order = (root.order + 1) % Words.consoleSorts.length
                else if (key === root.optionsAction)
                    Actions.consoleOptions(root.shown?.key ?? "")
                else
                    Actions.act(key)
            }
            Board {
                id: board
                designHeight: canvas.designHeight
            }
            Ink {
                id: ink
            }
            GroupGridLayout {
                id: layout
                model: root.listing
                columns: root.columns
            }
            Connections {
                target: root.listing
                function onModelReset() { layout.read() }
                function onDataChanged() { layout.read() }
            }
            // The two colour fields behind (board lines 19 and 20): each the board's place, size, blur, saturation, lightness and
            // opacity, the hue the focused console's (R-20261115-28).
            FieldInk {
                id: field0
                board: "#5B3FD6"
                hue: root.shown ? Qt.color(root.shown.glow).hslHue * 360 : NaN
            }
            FieldInk {
                id: field1
                board: "#C0486F"
                hue: root.shown ? Qt.color(root.shown.glow).hslHue * 360 : NaN
            }
            Backdrop {
                objectName: "consolesBackdrop"
                // Drawn behind everything, as a sky is, not beside it.
                readonly property bool backdrop: true
                anchors.fill: parent
                ground: Tokens.surface
                pixelScale: canvas.unit
                fields: [
                    BackdropField {
                        x: 1250 * root.px
                        y: 120 * root.px
                        width: 900 * root.px
                        height: 900 * root.px
                        color: root.shown ? field0.color : "transparent"
                        blur: 140 * root.px
                        opacity: 0.5
                    },
                    BackdropField {
                        x: 1500 * root.px
                        y: 520 * root.px
                        width: 620 * root.px
                        height: 620 * root.px
                        color: root.shown ? field1.color : "transparent"
                        blur: 150 * root.px
                        opacity: 0.35
                    }
                ]
            }
            Frame {
                id: frame
                anchors.fill: parent
                surfaceDrawn: false
                helpShown: false
                padFamily: Device.family
                focus: true
                controller.navigationShown: false
                // Declares the screen's buttons: Y the order and X the options, as the board gives them (EclipseConsoles:46-50, 94).
                help: Help.entries(Help.Console, {group: true, triggers: true, options: true})
                Connections {
                    target: frame.controller
                    function onBackDeclined() { Actions.back() }
                    function onActionInvoked(key) { root.invoked(key) }
                }
                Repeater {
                    model: frame.help
                    Item {
                        id: declared
                        required property int button
                        required property string action
                        Binding {
                            when: declared.button === PadGlyph.Search
                            target: root
                            property: "sortAction"
                            value: declared.action
                        }
                        Binding {
                            when: declared.button === PadGlyph.Screensaver
                            target: root
                            property: "optionsAction"
                            value: declared.action
                        }
                    }
                }
                Item {
                    x: -frame.safeX
                    y: -frame.safeY
                    width: root.width
                    height: root.height

                    // The header: the mark and the program's name, and the battery and the clock at the right (board lines 22 to 40); the sections' place is left empty (R-20261115-17).
                    Brand {
                        objectName: "consolesBrand"
                        unit: root.px
                        x: root.margin
                        y: 80 * root.px - height / 2
                    }
                    Status {
                        objectName: "consolesStatus"
                        unit: root.px
                        controller: frame.controller
                        searchable: false
                        glass: false
                        battery: true
                        // No glass, so no inset: the clock ends at the margin (board line 35).
                        pad: 0
                        x: root.width - root.margin - width
                        y: 80 * root.px - height / 2
                    }

                    // The chips: every console a core runs, those with games, those holding a favourite, each with its count (board lines 42 to 47).
                    Row {
                        id: chips
                        objectName: "chips"
                        x: root.margin
                        y: 150 * root.px
                        spacing: 14 * root.px
                        Repeater {
                            id: chipRow
                            model: Words.consoleShows
                            onItemAdded: (index, item) => root.firstChip = index === 0 ? item : root.firstChip
                            Walkable {
                                id: chip
                                required property int index
                                required property string modelData
                                readonly property bool pressed: chip.index === root.show
                                // A button's box holds its line (CSS's border-box): the pressed chip has none (board lines 43 to 45).
                                readonly property real line: chip.pressed ? 0 : 2 * root.px
                                controller: frame.controller
                                objectName: "chip." + chip.index
                                // With no console listed the pressed chip holds the focus, as a plate does otherwise.
                                focus: root.listing.count === 0 && chip.pressed
                                Accessible.name: chipWords.text
                                width: chipWords.width + 2 * (22 * root.px + chip.line)
                                height: 48 * root.px
                                // Along the row, Down to the plates; the pressed chip shows its consoles.
                                KeyNavigation.left: chipRow.itemAt(chip.index - 1)
                                KeyNavigation.right: chip.index + 1 < chipRow.count ? chipRow.itemAt(chip.index + 1) : sortEntry
                                Keys.onDownPressed: root.belowChips?.forceActiveFocus()
                                onActivated: root.show = chip.index
                                Rectangle {
                                    objectName: "chipFill." + chip.index
                                    anchors.fill: parent
                                    radius: height / 2
                                    color: chip.pressed ? Tokens.accent : "transparent"
                                    border.width: chip.line
                                    border.color: ink.chipLine
                                }
                                Line {
                                    id: chipWords
                                    objectName: "chipWords." + chip.index
                                    unit: root.px
                                    x: 22 * root.px + chip.line
                                    y: (parent.height - height) / 2
                                    text: Words.line([chip.modelData, Words.count(Library.consolesBy(chip.index, 0).count)])
                                    color: chip.pressed ? Tokens.overAccent : ink.tileFacts
                                    size: 21
                                    weight: chip.pressed ? Font.DemiBold : Font.Normal
                                }
                                FocusRing {
                                    anchors.fill: parent
                                    radius: height / 2
                                    visible: chip.activeFocus
                                }
                            }
                        }
                    }
                    // The order in force at the right, Y its button: pressed, it steps to the next (board lines 48 to 51).
                    Walkable {
                        id: sortEntry
                        objectName: "sort"
                        controller: frame.controller
                        Accessible.name: sortWords.text
                        x: root.width - root.margin - width
                        y: chips.y + (48 * root.px - height) / 2
                        width: sortWords.x + sortWords.width
                        height: sortGlyph.height
                        KeyNavigation.left: chipRow.itemAt(chipRow.count - 1)
                        Keys.onDownPressed: root.belowChips?.forceActiveFocus()
                        onActivated: root.invoked(root.sortAction)
                        Glyph {
                            id: sortGlyph
                            objectName: "sortGlyph"
                            unit: root.px
                            button: PadGlyph.Search
                            side: 34
                            letterSize: 16
                            ink: ink.help
                        }
                        Line {
                            id: sortWords
                            objectName: "sortWords"
                            unit: root.px
                            x: sortGlyph.width + 12 * root.px
                            y: (parent.height - height) / 2
                            text: Words.consoleSorts[root.order]
                            color: ink.help
                            size: 21
                        }
                        FocusRing {
                            anchors.fill: parent
                            radius: height / 2
                            visible: sortEntry.activeFocus
                        }
                    }

                    // The makers, one row each at its group's place, the chosen one filled (board lines 54 to 60).
                    Flickable {
                        id: makerList
                        objectName: "makers"
                        x: root.margin
                        y: root.listTop
                        width: root.makerWidth
                        height: root.makersHeight
                        clip: true
                        // The wheel and a drag scroll it; the binding brings the chosen maker back into sight as another is chosen.
                        boundsBehavior: Flickable.StopAtBounds
                        contentHeight: layout.groups * root.makerPitch
                        contentY: Math.max(0, Math.min(root.chosenGroup * root.makerPitch + root.makerHeight - height, contentHeight - height))
                        Repeater {
                            id: makers
                            model: root.listing
                            Walkable {
                                id: maker
                                required property int index
                                required property string maker
                                required property int group
                                required property int groupIndex
                                required property int groupSize
                                readonly property bool chosen: maker.group === root.chosenGroup
                                readonly property int before: layout.firsts[maker.group - 1] ?? -1
                                readonly property int after: layout.firsts[maker.group + 1] ?? -1
                                readonly property int first: layout.firsts[maker.group] ?? -1
                                controller: frame.controller
                                objectName: "maker." + maker.group
                                visible: maker.groupIndex === 0
                                Accessible.name: maker.maker
                                Accessible.description: Words.games(maker.groupSize)
                                y: maker.group * root.makerPitch
                                width: root.makerWidth
                                height: root.makerHeight
                                onActiveFocusChanged: {
                                    root.focusedMaker = maker.activeFocus ? maker : (root.focusedMaker === maker ? null : root.focusedMaker)
                                    if (maker.activeFocus) {
                                        root.chosenGroup = maker.group
                                        root.chosenFirst = plates.itemAt(maker.first)
                                    }
                                }
                                // Right its first console, LT (the key Home) and RT (End) the makers' first consoles, Up and Down the
                                // makers, Up from the first the chips; a plate focused chooses its own maker.
                                Keys.onPressed: (event) => {
                                    const at = [Qt.Key_Right, Qt.Key_Home, Qt.Key_End, Qt.Key_Up, Qt.Key_Down].indexOf(event.key)
                                    const to = at >= 0 && at < 3 ? [maker.first, maker.before, maker.after][at] : -1
                                    const item = to >= 0 ? plates.itemAt(to) : (at === 3 ? (makers.itemAt(maker.before) ?? chipRow.itemAt(root.show)) : (at === 4 ? makers.itemAt(maker.after) : null))
                                    event.accepted = item !== null
                                    item?.forceActiveFocus()
                                }
                                onActivated: plates.itemAt(maker.first).forceActiveFocus()
                                Rectangle {
                                    objectName: "makerFill." + maker.group
                                    anchors.fill: parent
                                    radius: 14 * root.px
                                    color: maker.chosen ? Tokens.accent : "transparent"
                                }
                                Line {
                                    objectName: "makerName." + maker.group
                                    unit: root.px
                                    x: 18 * root.px
                                    y: (parent.height - height) / 2
                                    width: parent.width - 2 * 18 * root.px - makerCount.width - 12 * root.px
                                    elide: Text.ElideRight
                                    text: maker.maker
                                    color: maker.chosen ? Tokens.overAccent : ink.help
                                    size: 22
                                    weight: maker.chosen ? Font.DemiBold : Font.Normal
                                }
                                Line {
                                    id: makerCount
                                    objectName: "makerCount." + maker.group
                                    unit: root.px
                                    x: parent.width - 18 * root.px - width
                                    y: (parent.height - height) / 2
                                    text: maker.groupSize
                                    color: maker.chosen ? Tokens.overAccent : ink.help
                                    size: 18
                                }
                            }
                        }
                    }

                    // The chosen maker's consoles as plates, each at its column and row in its group's grid (board lines 62 to 75).
                    Flickable {
                        id: grid
                        objectName: "plates"
                        // Half a gap past the grid on each side, so the clip never shaves an outer plate's edge.
                        x: root.gridX - root.plateGap / 2
                        y: root.listTop
                        width: root.gridWidth + root.plateGap
                        height: root.platesHeight
                        leftMargin: root.plateGap / 2
                        rightMargin: root.plateGap / 2
                        clip: true
                        // The wheel and a drag scroll the plates; the binding brings the focused plate back into sight as the focus moves.
                        boundsBehavior: Flickable.StopAtBounds
                        readonly property real pitch: root.plateHeight + root.plateGap
                        contentHeight: Math.max(0, (layout.rows[root.chosenGroup] ?? 0) * grid.pitch - root.plateGap)
                        contentY: root.lastPlate ? Math.max(0, Math.min(root.lastPlate.y + root.plateHeight - height, contentHeight - height)) : 0
                        Repeater {
                            id: plates
                            model: root.listing
                            onItemAdded: (index, item) => root.firstPlate = item.opening ? item : root.firstPlate
                            Walkable {
                                id: plate
                                required property int index
                                required property string key
                                required property string displayName
                                required property string maker
                                required property int year
                                required property int count
                                required property int played
                                required property string glow
                                required property string engineName
                                required property string engineKind
                                required property string engineMark
                                required property string screen
                                required property int group
                                required property int groupIndex
                                required property int groupSize
                                readonly property int leftIndex: layout.left[plate.index] ?? -1
                                readonly property int rightIndex: layout.right[plate.index] ?? -1
                                readonly property int upIndex: layout.up[plate.index] ?? -1
                                readonly property int downIndex: layout.down[plate.index] ?? -1
                                readonly property int previousFirst: layout.firsts[plate.group - 1] ?? -1
                                readonly property int nextFirst: layout.firsts[plate.group + 1] ?? -1
                                readonly property int rowOf: layout.rowOf[plate.index] ?? 0
                                readonly property int rows: layout.rows[plate.group] ?? 0
                                // The share of its games played, at least the board's sliver (0.04) for a console with games (board line 130).
                                readonly property real share: plate.count > 0 ? Math.max(plate.played / plate.count, 0.04) : 0
                                controller: frame.controller
                                objectName: "plate." + plate.index
                                visible: plate.group === root.chosenGroup
                                // The console last opened has the focus, else the first maker's first console.
                                readonly property bool opening: Browse.section.length > 0 ? plate.key === Browse.section : plate.group === 0 && plate.groupIndex === 0
                                focus: plate.opening
                                Accessible.name: plate.displayName
                                Accessible.description: Words.facts(plate.maker, plate.year, plate.count, Words.runsOn(plate.engineName, plate.engineKind))
                                x: (layout.columnOf[plate.index] ?? 0) * (root.plateWidth + root.plateGap)
                                y: (layout.rowOf[plate.index] ?? 0) * grid.pitch
                                width: root.plateWidth
                                height: root.plateHeight
                                // The layout's walk, past the left edge the maker, past the right Open, up from the first row the chips,
                                // Down from the last row the next maker's first console; LT and RT the makers' first consoles
                                // (R-20261115-17); LB and RB are left to the sections. No KeyNavigation, which writes back links of its
                                // own and skips a plate not shown (EmuSen_Qt.md §3.4.9).
                                Keys.onPressed: (event) => {
                                    const at = [Qt.Key_Left, Qt.Key_Right, Qt.Key_Up, Qt.Key_Down, Qt.Key_Home, Qt.Key_End].indexOf(event.key)
                                    const below = plate.downIndex >= 0 ? plate.downIndex : (plate.rowOf + 1 >= plate.rows ? plate.nextFirst : -1)
                                    const to = at >= 0 ? [plate.leftIndex, plate.rightIndex, plate.upIndex, below, plate.previousFirst, plate.nextFirst][at] : -1
                                    const item = to >= 0 ? plates.itemAt(to) : ([makers.itemAt(layout.firsts[plate.group] ?? -1), openButton, chipRow.itemAt(root.show), null, null, null][at] ?? null)
                                    event.accepted = item !== null
                                    item?.forceActiveFocus()
                                }
                                onActiveFocusChanged: {
                                    if (!plate.activeFocus)
                                        return
                                    root.chosenGroup = plate.group
                                    root.shown = plate
                                    root.lastPlate = plate
                                    Actions.focus(plate.key)
                                }
                                onActivated: Actions.openConsole(plate.key)
                                Rectangle {
                                    objectName: "plateFill." + plate.index
                                    anchors.fill: parent
                                    radius: 18 * root.px
                                    color: Tokens.inputSurface
                                    border.width: plate.activeFocus ? 0 : 1 * root.px
                                    border.color: root.plateEdge
                                }
                                // The share of its games played: the track, and over it the arc from the top (board lines 66 and 67).
                                Shape {
                                    id: shareRing
                                    objectName: "plateShare." + plate.index
                                    x: 20 * root.px
                                    y: 20 * root.px
                                    width: 52 * root.px
                                    height: 52 * root.px
                                    preferredRendererType: Shape.CurveRenderer
                                    ShapePath {
                                        fillColor: "transparent"
                                        strokeColor: root.trackInk
                                        strokeWidth: 5 * root.px
                                        PathAngleArc {
                                            centerX: 26 * root.px
                                            centerY: 26 * root.px
                                            radiusX: 21 * root.px
                                            radiusY: 21 * root.px
                                            startAngle: 0
                                            sweepAngle: 360
                                        }
                                    }
                                    ShapePath {
                                        fillColor: "transparent"
                                        strokeColor: Tokens.accent
                                        strokeWidth: 5 * root.px
                                        capStyle: ShapePath.RoundCap
                                        PathAngleArc {
                                            centerX: 26 * root.px
                                            centerY: 26 * root.px
                                            radiusX: 21 * root.px
                                            radiusY: 21 * root.px
                                            startAngle: -90
                                            sweepAngle: 360 * plate.share
                                        }
                                    }
                                }
                                Column {
                                    x: 20 * root.px
                                    width: parent.width - 40 * root.px
                                    // At the plate's foot as the board stands it, never over the ring when a long name takes two lines.
                                    y: Math.max(shareRing.y + shareRing.height, parent.height - 20 * root.px - height)
                                    spacing: 6 * root.px
                                    Line {
                                        objectName: "plateName." + plate.index
                                        unit: root.px
                                        width: parent.width
                                        text: plate.displayName
                                        wrapMode: Text.WordWrap
                                        maximumLineCount: 2
                                        elide: Text.ElideRight
                                        color: Tokens.text
                                        size: 21
                                        weight: Font.DemiBold
                                        // CSS's line-height 1.15 is of the type's size; Qt's proportional one is of the face's own height.
                                        lineHeightMode: Text.FixedHeight
                                        lineHeight: 1.15 * 21
                                    }
                                    // "1990 · 214 games", or "1986 · no games yet" in the dimmer ink (board lines 128, 129).
                                    Line {
                                        objectName: "plateLine." + plate.index
                                        unit: root.px
                                        width: parent.width
                                        elide: Text.ElideRight
                                        text: Words.line([plate.year > 0 ? String(plate.year) : "", plate.count > 0 ? Words.games(plate.count) : Words.noGamesLine])
                                        color: plate.count > 0 ? Tokens.muted : ink.emptyLine
                                        size: 16
                                    }
                                }
                            }
                        }
                    }
                    // The focused maker's ring, drawn over the column's clip so its gap and glow stand whole.
                    FocusRing {
                        objectName: "makerRing"
                        visible: root.focusedMaker !== null
                        x: makerList.x
                        y: makerList.y + (root.focusedMaker ? root.focusedMaker.y : 0) - makerList.contentY
                        width: root.makerWidth
                        height: root.makerHeight
                        radius: 14 * root.px
                    }
                    // The focused plate's ring, drawn over the grid's clip so its gap and glow stand whole.
                    FocusRing {
                        objectName: "plateRing"
                        visible: root.lastPlate !== null && root.lastPlate.activeFocus
                        x: grid.x + grid.leftMargin + (root.lastPlate ? root.lastPlate.x : 0)
                        y: grid.y + (root.lastPlate ? root.lastPlate.y : 0) - grid.contentY
                        width: root.plateWidth
                        height: root.plateHeight
                        radius: 18 * root.px
                        // The board's ring (line 64's box-shadow): a gap of 4, a band of 3, a glow of 40 spread 10.
                        gap: 4 * root.px
                        thickness: 3 * root.px
                        glowColor: ink.ringGlow
                        glowBlur: 40 * root.px
                        glowSpread: 10 * root.px
                        bandRadius: (18 + 7) * root.px
                        pixelScale: canvas.unit
                    }

                    // The focused console: its disc, its maker and year, its name, its facts with its core's mark, Open and Options (board lines 77 to 97).
                    Item {
                        id: aside
                        objectName: "consoleAside"
                        visible: root.shown !== null
                        x: root.asideX
                        y: 230 * root.px
                        width: root.asideWidth
                        height: root.foot - y
                        CoronaDisc {
                            id: disc
                            objectName: "consoleDisc"
                            x: (parent.width - width) / 2
                            width: 300 * root.px
                            height: 300 * root.px
                            // The focused console's own colour for the rim and both glows (R-20261115-9), at this board's rim of 2,
                            // glow of 40 spread 8 at 0.6 and halo of 150 spread 40 at 0.4 (line 78), not Home's.
                            corona: root.shown ? root.shown.glow : "transparent"
                            rim: 2 * root.px
                            pixelScale: canvas.unit
                            glows: [
                                CoronaGlow {
                                    blur: 40 * root.px
                                    spread: 8 * root.px
                                    color: Qt.rgba(disc.corona.r, disc.corona.g, disc.corona.b, 0.6)
                                },
                                CoronaGlow {
                                    blur: 150 * root.px
                                    spread: 40 * root.px
                                    color: Qt.rgba(disc.corona.r, disc.corona.g, disc.corona.b, 0.4)
                                }
                            ]
                        }
                        Column {
                            y: disc.height + (22 + 12) * root.px
                            width: parent.width
                            spacing: 8 * root.px
                            Line {
                                id: eyebrow
                                objectName: "consoleEyebrow"
                                unit: root.px
                                width: parent.width
                                elide: Text.ElideRight
                                text: root.shown ? Words.line([root.shown.maker, root.shown.year > 0 ? String(root.shown.year) : ""]) : ""
                                color: ink.rose
                                size: 18
                                weight: Font.DemiBold
                                // The board's 0.2 em (line 80).
                                spacing: 0.2 * 18
                                capitalization: Font.AllUppercase
                            }
                            Line {
                                objectName: "consoleName"
                                unit: root.px
                                width: parent.width
                                wrapMode: Text.WordWrap
                                maximumLineCount: 2
                                elide: Text.ElideRight
                                text: root.shown ? root.shown.displayName : ""
                                color: Tokens.text
                                size: 46
                                weight: Font.DemiBold
                                lineHeightMode: Text.FixedHeight
                                lineHeight: 1.08 * 46
                            }
                            // The aside's gap of 22 (board line 77) less the column's two spacings of 8 either side.
                            Item {
                                width: parent.width
                                height: (22 - 2 * 8) * root.px
                            }
                            // The four facts, a label at the left and its value at the right of each row (board lines 83 to 88).
                            Column {
                                id: facts
                                objectName: "consoleFacts"
                                width: parent.width
                                spacing: 14 * root.px
                                Repeater {
                                    model: Words.factLabels
                                    Item {
                                        id: fact
                                        required property int index
                                        required property string modelData
                                        readonly property var values: root.shown ? [Words.count(root.shown.count), Words.count(root.shown.played), root.shown.engineName, root.shown.screen] : ["", "", "", ""]
                                        objectName: "consoleFact." + fact.index
                                        width: facts.width
                                        height: factLabel.height
                                        Line {
                                            id: factLabel
                                            objectName: "consoleFactLabel." + fact.index
                                            unit: root.px
                                            text: fact.modelData
                                            color: Tokens.muted
                                            size: 21
                                        }
                                        Line {
                                            id: factValue
                                            objectName: "consoleFactValue." + fact.index
                                            unit: root.px
                                            x: parent.width - width
                                            text: fact.values[fact.index]
                                            color: Tokens.text
                                            size: 21
                                        }
                                        // The core's planet before its name (board line 86).
                                        PlanetMark {
                                            objectName: "consoleCoreMark"
                                            visible: fact.index === 2 && root.shown !== null && root.shown.engineMark.length > 0
                                            x: factValue.x - 10 * root.px - width
                                            y: (parent.height - height) / 2
                                            width: 24 * root.px
                                            height: 24 * root.px
                                            mark: root.shown ? root.shown.engineMark : ""
                                        }
                                    }
                                }
                            }
                            // The gap of 22 and the buttons' margin of 8 (board lines 77, 89) less the two spacings of 8.
                            Item {
                                width: parent.width
                                height: (22 + 8 - 2 * 8) * root.px
                            }
                            Row {
                                spacing: 14 * root.px
                                // Open, the help bar's own A entry, as a pill (board line 90).
                                Pill {
                                    id: openButton
                                    objectName: "consoleOpen"
                                    controller: frame.controller
                                    unit: root.px
                                    button: PadGlyph.Accept
                                    label: acceptWords.text
                                    filled: true
                                    tall: 64
                                    glyphSide: 42
                                    leadLetterSize: 19
                                    // This board's Open has no halo and no glow (board line 90).
                                    glows: false
                                    labelSize: 23
                                    lead: 12
                                    trail: 28
                                    gap: 12
                                    place: 0
                                    Keys.onLeftPressed: root.lastPlate?.forceActiveFocus()
                                    KeyNavigation.right: optionsButton
                                    onActivated: Actions.openConsole(root.shown?.key ?? "")
                                }
                                // Options, X: Settings' page of the console (board line 93).
                                Pill {
                                    id: optionsButton
                                    objectName: "consoleOptions"
                                    controller: frame.controller
                                    unit: root.px
                                    button: PadGlyph.Screensaver
                                    label: optionsWords.text
                                    outline: ink.pillLine
                                    tall: 64
                                    glyphSide: 40
                                    letterSize: 18
                                    labelSize: 23
                                    lead: 12
                                    trail: 26
                                    gap: 12
                                    place: 1
                                    KeyNavigation.left: openButton
                                    onActivated: root.invoked(root.optionsAction)
                                }
                            }
                            // The A and X entries' words, read from the help's own lists.
                            Repeater {
                                model: frame.help
                                Item {
                                    id: entry
                                    required property int button
                                    required property string label
                                    Binding {
                                        when: entry.button === PadGlyph.Accept
                                        target: acceptWords
                                        property: "text"
                                        value: entry.label
                                    }
                                    Binding {
                                        when: entry.button === PadGlyph.Screensaver
                                        target: optionsWords
                                        property: "text"
                                        value: entry.label
                                    }
                                }
                            }
                            QtObject {
                                id: acceptWords
                                property string text
                            }
                            QtObject {
                                id: optionsWords
                                property string text
                            }
                        }
                    }

                    // No console is shown: the program's empty words (a chip that holds none, or no core at all).
                    Column {
                        objectName: "noConsoles"
                        visible: root.listing.count === 0
                        y: 537 * root.px
                        width: root.width
                        spacing: 12 * root.px
                        Line {
                            unit: root.px
                            width: parent.width
                            horizontalAlignment: Text.AlignHCenter
                            text: root.show === 0 ? Words.noConsoles : Words.noGames
                            color: Tokens.muted
                            size: 22
                        }
                        Line {
                            unit: root.px
                            width: parent.width
                            visible: root.show === 0
                            horizontalAlignment: Text.AlignHCenter
                            text: Words.addFolder
                            color: Tokens.muted
                            size: 22
                        }
                    }
                }
            }
            // The help line at the foot, and at its right the chosen maker and where the focused console stands in it (board lines 99 to 105).
            HelpLine {
                objectName: "consolesHelp"
                unit: root.px
                controller: frame.controller
                // The board's A, B and LT RT Maker: the screen's own actions, Y and X, are drawn above, not in the bar (board lines 99 to 104).
                model: Help.entries(Help.Console, {group: true, triggers: true, actions: false})
                ink: ink.help
                // This board's foot: its row 40 px over the bottom (board line 99), 38 px tall for the ringed B (34 and its 2 px ring, line 102).
                middle: 1141
                Line {
                    objectName: "consolePlace"
                    unit: root.px
                    visible: root.shown !== null
                    text: root.shown ? Words.line([root.shown.maker, Words.place(root.shown.groupIndex, root.shown.groupSize)]) : ""
                    color: ink.help
                    size: 21
                }
            }
        }
    }
}
